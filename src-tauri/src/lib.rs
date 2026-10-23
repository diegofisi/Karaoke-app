//! Pegamento Tauri: expone los crates y la fabrica como los comandos de
//! CONTRATO seccion 5. Sin logica de dominio aqui.

mod comandos;
pub mod entorno;
mod estado;
pub mod fabrica;
mod grabar;
pub mod herramientas;
mod juego;
mod tipos;
pub mod vista_previa;
mod youtube;

use biblioteca::Biblioteca;
use tauri::{Manager, RunEvent, WindowEvent};

use crate::estado::Estado;

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let datos = app.path().app_data_dir()?;
            std::fs::create_dir_all(datos.join("canciones"))?;
            // Copias de cookies que dejo una fabrica cancelada (su `finally` no corre si se mata).
            youtube::limpiar_copias(&datos.join("canciones"));
            // Musica/Karaoke: C:\Users\<usuario>\Music\Karaoke o ~/Music/Karaoke.
            let grabaciones = app
                .path()
                .audio_dir()
                .map(|musica| musica.join("Karaoke"))
                .unwrap_or_else(|_| datos.join("grabaciones"));
            app.manage(Estado::new(Biblioteca::new(datos), grabaciones));
            Ok(())
        })
        // Al cerrar la ventana principal: Tauri no sale mientras quede otra
        // ventana (la de inicio de sesion), asi que se cierra todo aqui.
        .on_window_event(|ventana, evento| {
            if ventana.label() == "main"
                && let WindowEvent::Destroyed = evento
            {
                let app = ventana.app_handle();
                if let Some(estado) = app.try_state::<Estado>() {
                    comandos::fabrica::cerrar_todo(&estado);
                }
                comandos::youtube::cerrar_ventanas(app);
            }
        })
        .invoke_handler(tauri::generate_handler![
            comandos::fabrica::estado_fabrica,
            comandos::canciones::listar_canciones,
            comandos::canciones::obtener_cancion,
            comandos::canciones::borrar_cancion,
            comandos::fabrica::procesar_cancion,
            comandos::fabrica::cancelar_proceso,
            comandos::fabrica::instalar_fabrica,
            comandos::ajustes::listar_microfonos,
            comandos::ajustes::obtener_ajustes,
            comandos::ajustes::guardar_ajustes,
            comandos::ajustes::calibrar_latencia,
            comandos::juego::iniciar_juego,
            comandos::juego::pausar_juego,
            comandos::juego::reanudar_juego,
            comandos::juego::buscar_juego,
            comandos::juego::fijar_voz_guia,
            comandos::juego::detener_juego,
            comandos::grabaciones::listar_grabaciones,
            comandos::grabaciones::abrir_grabacion,
            comandos::grabaciones::mostrar_grabacion,
            comandos::grabaciones::borrar_grabacion,
            comandos::youtube::analizar_url,
            comandos::youtube::estado_sesion_youtube,
            comandos::youtube::cuenta_youtube,
            comandos::youtube::iniciar_sesion_youtube,
            comandos::youtube::renovar_sesion_youtube,
            comandos::youtube::cerrar_sesion_youtube,
        ])
        .build(tauri::generate_context!());
    // Arranque, no runtime: si la ventana no se puede crear no hay a quien
    // devolver el error. Se informa por stderr y se sale con codigo 1.
    let app = match app {
        Ok(app) => app,
        Err(e) => {
            eprintln!("No se pudo arrancar Karaoke: {e}");
            std::process::exit(1);
        }
    };
    app.run(|app, evento| {
        // Al cerrar: sin esto, Python (con torch cargado) y ffmpeg siguen vivos.
        if let RunEvent::Exit = evento
            && let Some(estado) = app.try_state::<Estado>()
        {
            comandos::fabrica::cerrar_todo(&estado);
            comandos::youtube::cerrar_ventanas(app);
        }
    });
}
