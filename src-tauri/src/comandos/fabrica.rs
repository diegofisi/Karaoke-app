use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::Value;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::entorno::{self, EventoInstalacion};
use crate::estado::{Estado, Trabajo, candado, candado_limpieza};
use crate::fabrica;
use crate::herramientas;
use crate::tipos::{EstadoFabrica, PeticionProceso, Requisito};

/// Sesion de YouTube para yt-dlp: la iniciada en la app (Ajustes > YouTube).
pub fn sesion_youtube(estado: &Estado) -> fabrica::SesionYoutube {
    let de_la_app = crate::youtube::ruta_cookies(estado.biblioteca.datos());
    fabrica::SesionYoutube {
        cookies: de_la_app
            .is_file()
            .then(|| de_la_app.to_string_lossy().into_owned()),
    }
}

/// Ubicacion del Python de la fabrica segun Ajustes y el entorno de la app.
pub fn ubicacion(estado: &Estado) -> Result<fabrica::Ubicacion, String> {
    let elegida = candado(&estado.ajustes)?
        .carpeta_fabrica
        .clone()
        .map(PathBuf::from);
    Ok(fabrica::ubicar(
        elegida.as_deref(),
        estado.biblioteca.datos(),
    ))
}

fn requisito(id: &str, nombre: &str, ok: bool, obligatorio: bool, detalle: String) -> Requisito {
    Requisito {
        id: id.into(),
        nombre: nombre.into(),
        ok,
        obligatorio,
        detalle,
    }
}

fn origen(ruta: &std::path::Path) -> String {
    let empaquetada = herramientas::carpeta_app().is_some_and(|c| ruta.starts_with(c));
    if empaquetada {
        "Incluido con la app".into()
    } else {
        format!("Del sistema: {}", ruta.display())
    }
}

/// Comprueba TODO lo necesario para procesar antes de dejar empezar: la
/// fabrica (Python, PyTorch, modelos), ffmpeg, el runtime de JavaScript y la GPU.
#[tauri::command(rename_all = "snake_case")]
pub async fn estado_fabrica(estado: State<'_, Estado>) -> Result<EstadoFabrica, String> {
    if let Some(guardado) = candado(&estado.diagnostico)?.clone() {
        return Ok(guardado);
    }
    let u = ubicacion(&estado)?;
    // Arranca Python e importa torch: varios segundos, fuera del hilo principal.
    let mut r = tauri::async_runtime::spawn_blocking(move || fabrica::diagnostico(&u))
        .await
        .map_err(|e| format!("No se pudo consultar la fabrica: {e}"))?;

    let fabrica_ok = r.lista;
    // Instalar solo sirve si la app va a usar su propio entorno: con una
    // carpeta elegida en Ajustes o KARAOKE_PYTHON, se usaria otro Python.
    let elegida = candado(&estado.ajustes)?.carpeta_fabrica.is_some();
    let por_variable = std::env::var_os("KARAOKE_PYTHON").is_some();
    let ffmpeg = herramientas::buscar("ffmpeg");
    let js = herramientas::runtime_js();
    let gpu = r.gpu.clone();
    let perfil = r.perfil.clone();
    r.puede_instalar = herramientas::buscar("uv").is_some() && !elegida && !por_variable;
    r.requisitos = vec![
        requisito(
            "fabrica",
            "Fábrica (Python, PyTorch y modelos)",
            fabrica_ok,
            true,
            if fabrica_ok {
                format!("Python {}", r.python.clone().unwrap_or_default())
            } else if elegida {
                "La carpeta elegida en Ajustes no tiene la fábrica instalada. Quítala en Ajustes → Fábrica para que la app instale la suya.".into()
            } else if r.puede_instalar {
                "No está instalada. Pulsa «Instalar la fábrica»: la app la descarga sola (~13 GB)."
                    .into()
            } else {
                r.mensaje
                    .clone()
                    .unwrap_or_else(|| "No está instalada.".into())
            },
        ),
        requisito(
            "ffmpeg",
            "ffmpeg (conversión de audio)",
            ffmpeg.is_some(),
            true,
            ffmpeg.as_deref().map_or_else(
                || "No se encontró. Reinstala la app o instala ffmpeg.".into(),
                origen,
            ),
        ),
        requisito(
            "javascript",
            "JavaScript (descargas de YouTube)",
            js.is_some(),
            true,
            js.as_deref().map_or_else(
                || "No se encontró Deno ni Node. Reinstala la app o instala Node.js.".into(),
                origen,
            ),
        ),
        requisito(
            "gpu",
            "GPU",
            fabrica_ok && gpu.is_some(),
            false,
            match (&perfil, &gpu) {
                (Some(p), _) => p.clone(),
                (None, Some(g)) => g.clone(),
                (None, None) if fabrica_ok => {
                    "Sin GPU: procesar una canción puede tardar mucho.".into()
                }
                (None, None) => "Se comprueba cuando la fábrica esté instalada.".into(),
            },
        ),
    ];
    r.lista = r.requisitos.iter().all(|q| q.ok || !q.obligatorio);
    if r.lista {
        *candado(&estado.diagnostico)? = Some(r.clone());
    }
    Ok(r)
}

/// Instala la fabrica con uv (Python, PyTorch, la fabrica, los modelos).
/// Tarda minutos: el progreso llega por el canal.
#[tauri::command(rename_all = "snake_case")]
pub async fn instalar_fabrica(
    app: AppHandle,
    canal: Channel<EventoInstalacion>,
) -> Result<(), String> {
    let estado = app.state::<Estado>();
    if candado(&estado.ajustes)?.carpeta_fabrica.is_some()
        || std::env::var_os("KARAOKE_PYTHON").is_some()
    {
        return Err(
            "No se pudo instalar: hay una carpeta de fabrica elegida en Ajustes; quitala primero."
                .into(),
        );
    }
    // Reinstalar con una cancion procesandose rompe las dos: el Python del
    // entorno esta en uso y su marca de listo se borra al empezar.
    if !candado(&estado.trabajos)?.is_empty() {
        return Err(
            "No se pudo instalar: hay canciones procesandose; espera a que terminen.".into(),
        );
    }
    if estado.instalando.swap(true, Ordering::AcqRel) {
        return Err("No se pudo instalar: ya hay una instalacion en curso.".into());
    }
    let datos = estado.biblioteca.datos().to_path_buf();
    // Codigo de la fabrica: recurso de la app; en desarrollo, la carpeta del repositorio.
    let fuente = app
        .path()
        .resource_dir()
        .map(|r| r.join("fabrica"))
        .ok()
        .filter(|f| f.join("pyproject.toml").is_file())
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("fabrica")
        });
    let resultado = tauri::async_runtime::spawn_blocking(move || {
        entorno::instalar(&datos, &fuente, |e| {
            let _ = canal.send(e);
        })
    })
    .await
    .map_err(|e| format!("No se pudo instalar la fabrica: {e}"))
    .and_then(|r| r);
    estado.instalando.store(false, Ordering::Release);
    *candado_limpieza(&estado.diagnostico) = None;
    resultado
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
    if estado.instalando.load(Ordering::Acquire) {
        return Err(
            "No se pudo procesar: la fabrica se esta instalando; espera a que termine.".into(),
        );
    }
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

    let u = match ubicacion(&estado) {
        Ok(u) => u,
        Err(e) => {
            quitar_trabajo(&estado, &id);
            return Err(e);
        }
    };
    let sesion = sesion_youtube(&estado);
    let hijo = match fabrica::arrancar(&peticion, &carpeta, &u, &sesion) {
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
    entorno::detener_en_curso();
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
