use std::path::Path;

use biblioteca::Grabacion;
use biblioteca::grabaciones;
use tauri::State;

use crate::estado::Estado;

#[tauri::command(rename_all = "snake_case")]
pub fn listar_grabaciones(estado: State<'_, Estado>) -> Result<Vec<Grabacion>, String> {
    grabaciones::listar(&estado.carpeta_grabaciones).map_err(|e| e.to_string())
}

/// Abre la grabacion con el reproductor del sistema.
#[tauri::command(rename_all = "snake_case")]
pub fn abrir_grabacion(estado: State<'_, Estado>, ruta: String) -> Result<(), String> {
    let real = grabaciones::validar(&estado.carpeta_grabaciones, Path::new(&ruta))
        .map_err(|e| e.to_string())?;
    tauri_plugin_opener::open_path(real, None::<&str>)
        .map_err(|e| format!("No se pudo abrir la grabacion: {e}"))
}

/// Muestra la grabacion seleccionada en el Explorador (Windows) o el Finder (Mac).
#[tauri::command(rename_all = "snake_case")]
pub fn mostrar_grabacion(estado: State<'_, Estado>, ruta: String) -> Result<(), String> {
    let real = grabaciones::validar(&estado.carpeta_grabaciones, Path::new(&ruta))
        .map_err(|e| e.to_string())?;
    tauri_plugin_opener::reveal_item_in_dir(real)
        .map_err(|e| format!("No se pudo mostrar la grabacion: {e}"))
}

#[tauri::command(rename_all = "snake_case")]
pub fn borrar_grabacion(estado: State<'_, Estado>, ruta: String) -> Result<(), String> {
    grabaciones::borrar(&estado.carpeta_grabaciones, Path::new(&ruta)).map_err(|e| e.to_string())
}
