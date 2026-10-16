//! `records.json` de una cancion: la mejor puntuacion por dificultad.
//!
//! ```json
//! { "facil": null, "normal": { "puntos": 8120, "fecha": "2026-10-15T18:30:00Z" }, "dificil": null }
//! ```

use std::fs;
use std::path::Path;

use puntuacion::Dificultad;
use serde::{Deserialize, Serialize};

pub(crate) const ARCHIVO: &str = "records.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Record {
    pub puntos: u32,
    /// UTC, RFC 3339 (`2026-10-15T18:30:00Z`).
    pub fecha: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct Records {
    pub facil: Option<Record>,
    pub normal: Option<Record>,
    pub dificil: Option<Record>,
}

impl Records {
    pub fn de(&self, dificultad: Dificultad) -> Option<&Record> {
        match dificultad {
            Dificultad::Facil => self.facil.as_ref(),
            Dificultad::Normal => self.normal.as_ref(),
            Dificultad::Dificil => self.dificil.as_ref(),
        }
    }

    pub fn fijar(&mut self, dificultad: Dificultad, record: Record) {
        let hueco = match dificultad {
            Dificultad::Facil => &mut self.facil,
            Dificultad::Normal => &mut self.normal,
            Dificultad::Dificil => &mut self.dificil,
        };
        *hueco = Some(record);
    }

    /// Mejor puntuacion en cualquier dificultad.
    pub fn mejor(&self) -> Option<u32> {
        [&self.facil, &self.normal, &self.dificil]
            .into_iter()
            .flatten()
            .map(|r| r.puntos)
            .max()
    }
}

/// Lee `records.json` de la carpeta. Sin archivo o ilegible: vacios (los
/// records no impiden jugar).
pub(crate) fn leer(carpeta: &Path) -> Records {
    fs::read_to_string(carpeta.join(ARCHIVO))
        .ok()
        .and_then(|texto| serde_json::from_str(&texto).ok())
        .unwrap_or_default()
}
