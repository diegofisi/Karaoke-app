//! Audio del juego con `cpal` (docs/ARQUITECTURA.md seccion 4, docs/CONTRATO.md
//! seccion 7).
//!
//! - [`Reproductor`]: salida por defecto. Mezcla instrumental + voz guia + pitidos.
//!   Su contador de cuadros entregados es el reloj de la cancion.
//! - [`Microfono`]: entrada en bruto, mono `f32`, en un ring sin bloqueos.
//! - [`Reloj`]: convierte el indice de una muestra del microfono en tiempo de
//!   cancion con la formula de CONTRATO seccion 7.
//! - [`calibrar`]: mide `latencia_ms` con pitidos por la misma salida y la
//!   misma entrada que usa el juego.
//!
//! Los callbacks de `cpal` no reservan memoria, no toman locks y no escriben en
//! consola: solo aritmetica, atomicos y rings `rtrb` reservados antes.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod calibracion;
mod dispositivos;
mod error;
mod grabacion;
mod microfono;
mod reloj;
mod reproductor;
mod wav;

pub use calibracion::{
    DURACION_PITIDO_S, EventoCalibracion, FRECUENCIA_PITIDO_HZ, PITIDOS_POR_DEFECTO,
    ResultadoCalibracion, SEPARACION_PITIDOS_S, calibrar, detectar_inicio, resumir_medidas,
};
pub use dispositivos::{DispositivoEntrada, listar_entradas};
pub use error::ErrorAudio;
pub use grabacion::{GrabacionVoz, Segmento, escribir_wav, ganancia_voz, mezclar};
pub use microfono::Microfono;
pub use reloj::{ALFA_POR_DEFECTO, Reloj, SALTO_MAXIMO_S};
pub use reproductor::Reproductor;
pub use wav::{AudioPcm, adaptar, adaptar_canales, leer_wav, remuestrear};
