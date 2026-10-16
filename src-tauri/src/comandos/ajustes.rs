use audio::{
    EventoCalibracion, Microfono as MicrofonoAudio, PITIDOS_POR_DEFECTO, Reproductor,
    ResultadoCalibracion,
};
use biblioteca::Ajustes;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::estado::{Estado, UsoAudio, candado, candado_limpieza};
use crate::tipos::Microfono;

#[tauri::command(rename_all = "snake_case")]
pub fn listar_microfonos() -> Vec<Microfono> {
    audio::listar_entradas()
        .into_iter()
        .map(|d| Microfono {
            id: d.id,
            nombre: d.nombre,
            por_defecto: d.por_defecto,
        })
        .collect()
}

#[tauri::command(rename_all = "snake_case")]
pub fn obtener_ajustes(estado: State<'_, Estado>) -> Result<Ajustes, String> {
    Ok(candado(&estado.ajustes)?.clone())
}

#[tauri::command(rename_all = "snake_case")]
pub fn guardar_ajustes(estado: State<'_, Estado>, ajustes: Ajustes) -> Result<Ajustes, String> {
    let guardados = estado
        .biblioteca
        .guardar_ajustes(&ajustes)
        .map_err(|e| e.to_string())?;
    *candado(&estado.ajustes)? = guardados.clone();
    Ok(guardados)
}

/// Devuelve el audio a `Libre` al salir de la calibracion por cualquier camino.
struct LiberarAudio(AppHandle);

impl Drop for LiberarAudio {
    fn drop(&mut self) {
        let estado = self.0.state::<Estado>();
        let mut audio = candado_limpieza(&estado.audio);
        if matches!(*audio, UsoAudio::Calibrando) {
            *audio = UsoAudio::Libre;
        }
    }
}

/// Bloquea unos 5 s: corre en un hilo de trabajo, con su propio audio.
#[tauri::command(rename_all = "snake_case")]
pub async fn calibrar_latencia(
    app: AppHandle,
    canal: Channel<EventoCalibracion>,
) -> Result<ResultadoCalibracion, String> {
    let microfono = {
        let estado = app.state::<Estado>();
        let microfono = candado(&estado.ajustes)?.microfono.clone();
        let mut audio = candado(&estado.audio)?;
        if !audio.disponible() {
            return Err(match *audio {
                UsoAudio::Calibrando => "No se pudo calibrar: ya hay una calibracion en curso.",
                _ => "No se pudo calibrar: hay una partida en marcha.",
            }
            .into());
        }
        *audio = UsoAudio::Calibrando;
        microfono
    };
    let _liberar = LiberarAudio(app.clone());
    tauri::async_runtime::spawn_blocking(move || {
        // El audio se crea en este hilo y muere en el (en macOS no se puede mover).
        let mut reproductor = Reproductor::abrir().map_err(|e| e.to_string())?;
        let mut mic = MicrofonoAudio::abrir(microfono.as_deref()).map_err(|e| e.to_string())?;
        audio::calibrar(&mut reproductor, &mut mic, PITIDOS_POR_DEFECTO, |evento| {
            let _ = canal.send(evento);
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("No se pudo calibrar: {e}"))?
}
