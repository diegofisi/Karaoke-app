//! Pegamento Tauri: expone los crates y la fabrica como los comandos de
//! CONTRATO seccion 5. Sin logica de dominio aqui.

mod comandos;
mod estado;
mod fabrica;
mod grabar;
mod juego;
mod tipos;

use biblioteca::Biblioteca;
use tauri::{Manager, RunEvent};

use crate::estado::Estado;

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let datos = app.path().app_data_dir()?;
            std::fs::create_dir_all(datos.join("canciones"))?;
            // Musica/Karaoke: C:\Users\<usuario>\Music\Karaoke o ~/Music/Karaoke.
            let grabaciones = app
                .path()
                .audio_dir()
                .map(|musica| musica.join("Karaoke"))
                .unwrap_or_else(|_| datos.join("grabaciones"));
            app.manage(Estado::new(Biblioteca::new(datos), grabaciones));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            comandos::fabrica::estado_fabrica,
            comandos::canciones::listar_canciones,
            comandos::canciones::obtener_cancion,
            comandos::canciones::borrar_cancion,
            comandos::fabrica::procesar_cancion,
            comandos::fabrica::cancelar_proceso,
            comandos::ajustes::listar_microfonos,
            comandos::ajustes::obtener_ajustes,
            comandos::ajustes::guardar_ajustes,
            comandos::ajustes::calibrar_latencia,
            comandos::juego::iniciar_juego,
            comandos::juego::pausar_juego,
            comandos::juego::reanudar_juego,
            comandos::juego::buscar_juego,
            comandos::juego::detener_juego,
            comandos::grabaciones::listar_grabaciones,
            comandos::grabaciones::abrir_grabacion,
            comandos::grabaciones::mostrar_grabacion,
            comandos::grabaciones::borrar_grabacion,
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
        }
    });
}
