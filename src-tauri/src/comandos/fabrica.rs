use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::Value;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::estado::{Estado, Trabajo, candado, candado_limpieza};
use crate::fabrica;
use crate::tipos::{EstadoFabrica, PeticionProceso};

#[tauri::command(rename_all = "snake_case")]
pub async fn estado_fabrica(estado: State<'_, Estado>) -> Result<EstadoFabrica, String> {
    if let Some(guardado) = candado(&estado.diagnostico)?.clone() {
        return Ok(guardado);
    }
    // Arranca Python e importa torch: varios segundos, fuera del hilo principal.
    let resultado = tauri::async_runtime::spawn_blocking(fabrica::diagnostico)
        .await
        .map_err(|e| format!("No se pudo consultar la fabrica: {e}"))?;
    if resultado.lista {
        *candado(&estado.diagnostico)? = Some(resultado.clone());
    }
    Ok(resultado)
}

fn quitar_trabajo(estado: &Estado, id: &str) {
    candado_limpieza(&estado.trabajos).remove(id);
}

#[tauri::command(rename_all = "snake_case")]
pub fn procesar_cancion(
    app: AppHandle,
    estado: State<'_, Estado>,
    peticion: PeticionProceso,
    canal: Channel<Value>,
) -> Result<String, String> {
    let id = fabrica::id_de_peticion(&peticion)?;
    let carpeta = estado.biblioteca.carpeta(&id).map_err(|e| e.to_string())?;
    let cancelado = Arc::new(AtomicBool::new(false));
    // Reserva bajo el mismo lock que la comprobacion: dos peticiones iguales no
    // lanzan dos fabricas sobre la misma carpeta.
    match candado(&estado.trabajos)?.entry(id.clone()) {
        Entry::Occupied(_) => {
            return Err("No se pudo procesar: esa cancion ya se esta procesando.".into());
        }
        Entry::Vacant(hueco) => {
            hueco.insert(Trabajo {
                pid: None,
                cancelado: cancelado.clone(),
            });
        }
    }

    let hijo = match fabrica::arrancar(&peticion, &carpeta) {
        Ok(h) => h,
        Err(e) => {
            quitar_trabajo(&estado, &id);
            return Err(e);
        }
    };
    let pid = hijo.id();
    if let Some(t) = candado_limpieza(&estado.trabajos).get_mut(&id) {
        t.pid = Some(pid);
    }
    // Cancelado mientras arrancaba: se mata ya; el vigilante emitira "cancelado".
    if cancelado.load(Ordering::Acquire) {
        fabrica::matar(pid);
    }

    let id_hilo = id.clone();
    let vigilante = std::thread::Builder::new()
        .name(format!("fabrica-{id}"))
        .spawn(move || {
            fabrica::reenviar(hijo, cancelado, |evento| {
                let _ = canal.send(evento);
            });
            if let Some(estado) = app.try_state::<Estado>() {
                quitar_trabajo(&estado, &id_hilo);
            }
        });
    if let Err(e) = vigilante {
        // Sin vigilante nadie leeria su salida: se mata y se libera la entrada.
        fabrica::matar(pid);
        quitar_trabajo(&estado, &id);
        return Err(format!("No se pudo vigilar la fabrica: {e}"));
    }
    Ok(id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn cancelar_proceso(estado: State<'_, Estado>, id: String) -> Result<(), String> {
    let pid = {
        let trabajos = candado(&estado.trabajos)?;
        let Some(t) = trabajos.get(&id) else {
            return Ok(());
        };
        t.cancelado.store(true, Ordering::Release);
        t.pid
    };
    // Sin pid todavia: procesar_cancion lo vera marcado y lo matara al arrancar.
    if let Some(pid) = pid {
        fabrica::matar(pid);
    }
    Ok(())
}

/// Al cerrar la app: mata las fabricas en marcha y descarta la partida.
pub fn cerrar_todo(estado: &Estado) {
    let pids: Vec<u32> = candado_limpieza(&estado.trabajos)
        .values()
        .filter_map(|t| {
            t.cancelado.store(true, Ordering::Release);
            t.pid
        })
        .collect();
    for pid in pids {
        fabrica::matar(pid);
    }
    let audio = std::mem::replace(
        &mut *candado_limpieza(&estado.audio),
        crate::estado::UsoAudio::Libre,
    );
    if let crate::estado::UsoAudio::Jugando(p) = audio {
        let _ = p.control.send(crate::juego::Control::Descartar);
    }
}
