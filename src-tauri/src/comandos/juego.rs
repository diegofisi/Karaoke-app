use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::estado::{Estado, UsoAudio, candado, candado_limpieza};
use crate::juego::{self, Configuracion, Control};
use crate::tipos::{FinPartida, FrameJuego, Modo};

/// Async: abrir el audio decodifica los wav (~1 s) y no debe congelar la
/// ventana. El audio queda en `Arrancando` mientras tanto: ni otra partida ni
/// la calibracion pueden entrar, y un `detener_juego` en ese intervalo hace
/// que la partida recien abierta se descarte.
#[tauri::command(rename_all = "snake_case")]
pub async fn iniciar_juego(
    app: AppHandle,
    id: String,
    modo: Modo,
    desde: f64,
    canal: Channel<FrameJuego>,
) -> Result<(), String> {
    let config = {
        let estado = app.state::<Estado>();
        let config = Configuracion {
            carpeta: estado.biblioteca.carpeta(&id).map_err(|e| e.to_string())?,
            id,
            modo,
            desde: desde.max(0.0),
            ajustes: candado(&estado.ajustes)?.clone(),
            biblioteca: estado.biblioteca.clone(),
            carpeta_grabaciones: estado.carpeta_grabaciones.clone(),
        };
        let mut audio = candado(&estado.audio)?;
        if !audio.disponible() {
            return Err(match *audio {
                UsoAudio::Calibrando => "No se pudo empezar: hay una calibracion en curso.",
                _ => "No se pudo empezar: ya hay una partida en marcha.",
            }
            .into());
        }
        *audio = UsoAudio::Arrancando;
        config
    };

    let arranque = tauri::async_runtime::spawn_blocking(move || {
        juego::arrancar(config, move |frame| canal.send(frame).is_ok())
    })
    .await
    .map_err(|e| format!("No se pudo empezar: {e}"))
    .and_then(|r| r);

    let estado = app.state::<Estado>();
    let mut audio = candado_limpieza(&estado.audio);
    let sigue_pedida = matches!(*audio, UsoAudio::Arrancando);
    match arranque {
        Ok(partida) if sigue_pedida => {
            *audio = UsoAudio::Jugando(partida);
            Ok(())
        }
        Ok(partida) => {
            // Se pidio detener mientras arrancaba: se descarta sin guardar nada.
            let _ = partida.control.send(Control::Descartar);
            Err("No se pudo empezar: la partida se detuvo mientras arrancaba.".into())
        }
        Err(e) => {
            if sigue_pedida {
                *audio = UsoAudio::Libre;
            }
            Err(e)
        }
    }
}

fn enviar(estado: &Estado, control: Control) -> Result<(), String> {
    let audio = candado(&estado.audio)?;
    let UsoAudio::Jugando(p) = &*audio else {
        return Err("No se pudo: no hay ninguna partida en marcha.".into());
    };
    p.control
        .send(control)
        .map_err(|_| "No se pudo: la partida ya termino.".to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn pausar_juego(estado: State<'_, Estado>) -> Result<(), String> {
    enviar(&estado, Control::Pausar)
}

#[tauri::command(rename_all = "snake_case")]
pub fn reanudar_juego(estado: State<'_, Estado>) -> Result<(), String> {
    enviar(&estado, Control::Reanudar)
}

#[tauri::command(rename_all = "snake_case")]
pub fn buscar_juego(estado: State<'_, Estado>, t: f64) -> Result<(), String> {
    enviar(&estado, Control::Buscar(t))
}

/// Detiene y espera al hilo (si se grababa, tambien a que se guarde la
/// grabacion). `null` si no habia partida.
#[tauri::command(rename_all = "snake_case")]
pub async fn detener_juego(estado: State<'_, Estado>) -> Result<Option<FinPartida>, String> {
    let partida = {
        let mut audio = candado(&estado.audio)?;
        match std::mem::replace(&mut *audio, UsoAudio::Libre) {
            UsoAudio::Jugando(p) => p,
            // Arrancando: queda Libre y iniciar_juego descartara la partida.
            UsoAudio::Arrancando | UsoAudio::Libre => return Ok(None),
            UsoAudio::Calibrando => {
                *audio = UsoAudio::Calibrando;
                return Ok(None);
            }
        }
    };
    let _ = partida.control.send(Control::Detener);
    tauri::async_runtime::spawn_blocking(move || partida.hilo.join())
        .await
        .map_err(|e| format!("No se pudo detener la partida: {e}"))?
        .map_err(|_| "No se pudo detener la partida: el hilo del juego fallo.".to_string())?
}
