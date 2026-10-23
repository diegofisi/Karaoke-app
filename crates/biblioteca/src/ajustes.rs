//! `Ajustes` de CONTRATO seccion 5, guardados en `<datos>/ajustes.json`.

use puntuacion::Dificultad;
use serde::{Deserialize, Serialize};

pub const LATENCIA_MS_POR_DEFECTO: f64 = 150.0;
/// Limite de `latencia_ms` al normalizar.
pub const LATENCIA_MS_MAXIMA: f64 = 1000.0;

/// Campos que faltan en el JSON toman su valor por defecto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct Ajustes {
    /// Id de `audio::listar_entradas`; `None` = entrada por defecto.
    pub microfono: Option<String>,
    pub latencia_ms: f64,
    pub dificultad: Dificultad,
    /// 0 a 1.
    pub volumen_musica: f64,
    /// 0 a 1; 0 = sin voz guia.
    pub volumen_guia: f64,
    /// Multiplica el volumen de la voz del usuario en la grabacion; 1 = automatico. 0 a 3.
    pub volumen_voz: f64,
    pub mostrar_romaji: bool,
    /// Grabar la voz durante la partida y guardarla mezclada con la musica.
    pub grabar: bool,
    /// Carpeta de la fabrica Python (la del repositorio o `fabrica/`).
    /// `None` = la del repositorio donde se compilo la app (desarrollo).
    pub carpeta_fabrica: Option<String>,
}

impl Default for Ajustes {
    fn default() -> Ajustes {
        Ajustes {
            microfono: None,
            latencia_ms: LATENCIA_MS_POR_DEFECTO,
            dificultad: Dificultad::Normal,
            volumen_musica: 0.9,
            volumen_guia: 0.0,
            volumen_voz: 1.0,
            mostrar_romaji: true,
            grabar: false,
            carpeta_fabrica: None,
        }
    }
}

impl Ajustes {
    /// Corrige valores fuera de rango: volumenes a `[0, 1]`, `latencia_ms` a
    /// `[0, 1000]`, NaN al valor por defecto, microfono vacio a `None`.
    pub fn normalizados(mut self) -> Ajustes {
        let defecto = Ajustes::default();
        let acotar = |v: f64, min: f64, max: f64, defecto: f64| {
            if v.is_finite() {
                v.clamp(min, max)
            } else {
                defecto
            }
        };
        self.latencia_ms = acotar(
            self.latencia_ms,
            0.0,
            LATENCIA_MS_MAXIMA,
            defecto.latencia_ms,
        );
        self.volumen_musica = acotar(self.volumen_musica, 0.0, 1.0, defecto.volumen_musica);
        self.volumen_guia = acotar(self.volumen_guia, 0.0, 1.0, defecto.volumen_guia);
        self.volumen_voz = acotar(self.volumen_voz, 0.0, 3.0, defecto.volumen_voz);
        if self
            .microfono
            .as_deref()
            .is_some_and(|m| m.trim().is_empty())
        {
            self.microfono = None;
        }
        if self
            .carpeta_fabrica
            .as_deref()
            .is_some_and(|c| c.trim().is_empty())
        {
            self.carpeta_fabrica = None;
        }
        self
    }
}
