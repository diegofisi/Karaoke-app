use std::path::PathBuf;

/// Error del crate de audio. El mensaje (Display) va tal cual a la interfaz.
#[derive(Debug, thiserror::Error)]
pub enum ErrorAudio {
    #[error("No se pudo abrir la salida de audio: no hay ningun dispositivo de salida")]
    SinSalida,
    #[error("No se pudo abrir la salida de audio: {0}")]
    Salida(String),
    #[error("No se pudo abrir el microfono: no hay ningun dispositivo de entrada")]
    SinEntrada,
    #[error("No se pudo abrir el microfono: no se encontro el dispositivo \"{0}\"")]
    EntradaNoEncontrada(String),
    #[error("No se pudo abrir el microfono: {0}")]
    Microfono(String),
    #[error("No se pudo leer el audio {}: {detalle}", ruta.display())]
    Wav { ruta: PathBuf, detalle: String },
    #[error("No se pudo convertir el audio: {0}")]
    Conversion(String),
    #[error("No se pudo usar el audio: el dispositivo de salida no responde")]
    SinRespuesta,
    #[error("No se pudo calibrar la latencia: {0}")]
    Calibracion(String),
}
