use biblioteca::ResumenCancion;
use karaoke_core::Cancion;
use tauri::State;

use crate::estado::Estado;

#[tauri::command(rename_all = "snake_case")]
pub fn listar_canciones(estado: State<'_, Estado>) -> Result<Vec<ResumenCancion>, String> {
    estado.biblioteca.listar().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn obtener_cancion(estado: State<'_, Estado>, id: String) -> Result<Cancion, String> {
    estado.biblioteca.obtener(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn borrar_cancion(estado: State<'_, Estado>, id: String) -> Result<(), String> {
    estado.biblioteca.borrar(&id).map_err(|e| e.to_string())
}
