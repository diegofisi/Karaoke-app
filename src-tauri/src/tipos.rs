//! Tipos de CONTRATO seccion 5 y 6 que solo existen en la frontera con la
//! interfaz. Los de la cancion, la puntuacion, los ajustes y la calibracion
//! vienen de sus crates y no se copian aqui.

use biblioteca::Grabacion;
use puntuacion::{Evento, Nivel, PorCalificacion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EstadoFabrica {
    pub lista: bool,
    pub python: Option<String>,
    pub cuda: bool,
    pub gpu: Option<String>,
    pub mensaje: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PeticionProceso {
    pub url: Option<String>,
    pub archivo: Option<String>,
    pub titulo: Option<String>,
    pub artista: Option<String>,
    pub idioma: Option<String>,
    /// Texto pegado (.txt o LRC).
    pub letra: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Microfono {
    pub id: String,
    pub nombre: String,
    pub por_defecto: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modo {
    Practica,
    Puntuacion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EstadoJuego {
    Sonando,
    Pausado,
    Terminado,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TonoMicrofono {
    pub t: f64,
    pub midi: Option<f32>,
}

/// Uno cada ~16 ms mientras el juego corre (CONTRATO seccion 6).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FrameJuego {
    pub t: f64,
    pub tonos: Vec<TonoMicrofono>,
    pub puntos: f64,
    pub combo: u32,
    pub eventos: Vec<Evento>,
    pub estado: EstadoJuego,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ResultadoJuego {
    pub puntos: u32,
    pub nivel: Nivel,
    pub mejor_combo: u32,
    pub precision_media: f64,
    pub por_calificacion: PorCalificacion,
    pub record_anterior: Option<u32>,
    pub es_record: bool,
}

/// Lo que devuelve `detener_juego` (CONTRATO seccion 5).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FinPartida {
    /// `None` en practica.
    pub resultado: Option<ResultadoJuego>,
    /// `None` si no se grababa o si fallo (ver `error_grabacion`).
    pub grabacion: Option<Grabacion>,
    pub error_grabacion: Option<String>,
}

#[cfg(test)]
mod pruebas {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn frame_juego_tiene_la_forma_del_contrato() {
        let frame = FrameJuego {
            t: 1.5,
            tonos: vec![TonoMicrofono {
                t: 1.49,
                midi: None,
            }],
            puntos: 10.0,
            combo: 2,
            eventos: vec![],
            estado: EstadoJuego::Sonando,
        };
        let v = serde_json::to_value(&frame).unwrap();
        assert_eq!(v["estado"], "sonando");
        assert!(v["tonos"][0]["midi"].is_null());
        assert_eq!(v["combo"], 2);
    }

    #[test]
    fn modo_se_lee_en_minusculas() {
        let m: Modo = serde_json::from_str("\"practica\"").unwrap();
        assert_eq!(m, Modo::Practica);
    }
}
