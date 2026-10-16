//! Carpeta de canciones (`<datos>/canciones/<id>/`), records y ajustes
//! (docs/CONTRATO.md secciones 2 y 5).
//!
//! Rust solo lee `cancion.json` y los `.wav`; escribe `records.json` y
//! `ajustes.json`, siempre con tmp + rename.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod ajustes;
pub mod fecha;
pub mod grabaciones;
mod records;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use karaoke_core::{Cancion, ErrorCancion};
use serde::{Deserialize, Serialize};

pub use ajustes::Ajustes;
pub use grabaciones::{DatosGrabacion, ErrorGrabacion, Grabacion};
pub use puntuacion::Dificultad;
pub use records::{Record, Records};

/// `ResumenCancion` de CONTRATO seccion 5: el resumen de `karaoke-core` mas
/// el record. En JSON los campos van al mismo nivel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResumenCancion {
    #[serde(flatten)]
    pub cancion: karaoke_core::ResumenCancion,
    /// Mejor puntuacion en cualquier dificultad; `None` si nunca se jugo.
    pub record: Option<u32>,
}

/// Error de la biblioteca. El mensaje (Display) va tal cual a la interfaz.
#[derive(Debug, thiserror::Error)]
pub enum ErrorBiblioteca {
    #[error("No se pudo usar la cancion: id no valido \"{0}\"")]
    IdInvalido(String),
    #[error("No se pudo encontrar la cancion {0}")]
    NoExiste(String),
    #[error(transparent)]
    Cancion(#[from] ErrorCancion),
    #[error("No se pudo leer la carpeta de canciones {}: {fuente}", ruta.display())]
    Listar {
        ruta: PathBuf,
        #[source]
        fuente: std::io::Error,
    },
    #[error("No se pudo borrar la cancion {id}: {fuente}")]
    Borrar {
        id: String,
        #[source]
        fuente: std::io::Error,
    },
    #[error("No se pudo guardar {}: {detalle}", ruta.display())]
    Guardar { ruta: PathBuf, detalle: String },
}

/// Acceso a `<datos>`: canciones, records y ajustes. No guarda estado: cada
/// llamada lee el disco.
#[derive(Debug, Clone)]
pub struct Biblioteca {
    datos: PathBuf,
}

impl Biblioteca {
    /// `datos` es `app_data_dir` de Tauri. No hace falta que exista.
    pub fn new(datos: PathBuf) -> Biblioteca {
        Biblioteca { datos }
    }

    pub fn datos(&self) -> &Path {
        &self.datos
    }

    /// `<datos>/canciones`.
    pub fn carpeta_canciones(&self) -> PathBuf {
        self.datos.join("canciones")
    }

    /// `<datos>/canciones/<id>`. Rechaza ids que podrian salir de la carpeta.
    pub fn carpeta(&self, id: &str) -> Result<PathBuf, ErrorBiblioteca> {
        validar_id(id)?;
        Ok(self.carpeta_canciones().join(id))
    }

    /// Canciones listas (con `cancion.json` valido), la mas reciente primero
    /// (fecha de modificacion de `cancion.json`; a igualdad, por id). Las
    /// carpetas sin `cancion.json` (en proceso o fallidas) y las que tienen un
    /// `cancion.json` que no se puede leer se saltan. Sin carpeta de
    /// canciones, lista vacia.
    pub fn listar(&self) -> Result<Vec<ResumenCancion>, ErrorBiblioteca> {
        let raiz = self.carpeta_canciones();
        let entradas = match fs::read_dir(&raiz) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(fuente) => return Err(ErrorBiblioteca::Listar { ruta: raiz, fuente }),
        };
        let mut lista: Vec<(SystemTime, ResumenCancion)> = Vec::new();
        for entrada in entradas.flatten() {
            let carpeta = entrada.path();
            let ruta = carpeta.join("cancion.json");
            let Ok(meta) = fs::metadata(&ruta) else {
                continue;
            };
            let Ok(cancion) = Cancion::leer(&ruta) else {
                continue;
            };
            let record = records::leer(&carpeta).mejor();
            let fecha = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            lista.push((
                fecha,
                ResumenCancion {
                    cancion: cancion.resumen(),
                    record,
                },
            ));
        }
        lista.sort_by(|(fa, a), (fb, b)| fb.cmp(fa).then_with(|| a.cancion.id.cmp(&b.cancion.id)));
        Ok(lista.into_iter().map(|(_, r)| r).collect())
    }

    /// Lee y valida `cancion.json` de una cancion.
    pub fn obtener(&self, id: &str) -> Result<Cancion, ErrorBiblioteca> {
        let ruta = self.carpeta(id)?.join("cancion.json");
        if !ruta.is_file() {
            return Err(ErrorBiblioteca::NoExiste(id.to_string()));
        }
        Ok(Cancion::leer(&ruta)?)
    }

    /// Borra la carpeta de la cancion. Si no existe, no hace nada.
    pub fn borrar(&self, id: &str) -> Result<(), ErrorBiblioteca> {
        let carpeta = self.carpeta(id)?;
        match fs::remove_dir_all(&carpeta) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(fuente) => Err(ErrorBiblioteca::Borrar {
                id: id.to_string(),
                fuente,
            }),
        }
    }

    /// Records de una cancion (vacios si no hay `records.json` o no se puede
    /// leer).
    pub fn records(&self, id: &str) -> Result<Records, ErrorBiblioteca> {
        Ok(records::leer(&self.carpeta(id)?))
    }

    /// Anota una partida terminada. Devuelve `(record_anterior, es_record)`
    /// para esa dificultad; si es record, lo guarda con la fecha actual. La
    /// primera partida en una dificultad siempre es record.
    pub fn guardar_resultado(
        &self,
        id: &str,
        dificultad: Dificultad,
        puntos: u32,
    ) -> Result<(Option<u32>, bool), ErrorBiblioteca> {
        let carpeta = self.carpeta(id)?;
        if !carpeta.is_dir() {
            return Err(ErrorBiblioteca::NoExiste(id.to_string()));
        }
        let mut records = records::leer(&carpeta);
        let anterior = records.de(dificultad).map(|r| r.puntos);
        let es_record = anterior.is_none_or(|a| puntos > a);
        if es_record {
            records.fijar(
                dificultad,
                Record {
                    puntos,
                    fecha: fecha::ahora_rfc3339(),
                },
            );
            escribir_json(&carpeta.join(records::ARCHIVO), &records)?;
        }
        Ok((anterior, es_record))
    }

    /// `<datos>/ajustes.json`.
    pub fn ruta_ajustes(&self) -> PathBuf {
        self.datos.join("ajustes.json")
    }

    /// Ajustes guardados. Sin archivo o con un archivo que no se puede leer:
    /// los de por defecto. Campos que faltan: su valor por defecto. Los
    /// valores fuera de rango se corrigen.
    pub fn leer_ajustes(&self) -> Ajustes {
        fs::read_to_string(self.ruta_ajustes())
            .ok()
            .and_then(|texto| serde_json::from_str::<Ajustes>(&texto).ok())
            .unwrap_or_default()
            .normalizados()
    }

    /// Guarda los ajustes (corregidos) de forma atomica y los devuelve.
    pub fn guardar_ajustes(&self, ajustes: &Ajustes) -> Result<Ajustes, ErrorBiblioteca> {
        let ajustes = ajustes.clone().normalizados();
        escribir_json(&self.ruta_ajustes(), &ajustes)?;
        Ok(ajustes)
    }
}

/// Un id valido es un solo componente de ruta: letras y digitos ASCII, `-`,
/// `_` y `.`, sin `..`. Los ids del contrato (`yt-...`, `local-...`) cumplen.
fn validar_id(id: &str) -> Result<(), ErrorBiblioteca> {
    let valido = !id.is_empty()
        && id != "."
        && !id.contains("..")
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if valido {
        Ok(())
    } else {
        Err(ErrorBiblioteca::IdInvalido(id.to_string()))
    }
}

/// Escribe JSON con tmp + rename, creando la carpeta si falta.
fn escribir_json<T: Serialize>(ruta: &Path, valor: &T) -> Result<(), ErrorBiblioteca> {
    let error = |detalle: String| ErrorBiblioteca::Guardar {
        ruta: ruta.to_path_buf(),
        detalle,
    };
    let texto = serde_json::to_string_pretty(valor).map_err(|e| error(e.to_string()))?;
    if let Some(padre) = ruta.parent() {
        fs::create_dir_all(padre).map_err(|e| error(e.to_string()))?;
    }
    let mut tmp = ruta.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, texto).map_err(|e| error(e.to_string()))?;
    fs::rename(&tmp, ruta).map_err(|e| {
        // El tmp huerfano no sirve para nada; si no se puede borrar, se
        // sobrescribe en el proximo intento.
        let _ = fs::remove_file(&tmp);
        error(e.to_string())
    })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn ids_validos_e_invalidos() {
        for id in ["yt-dQw4w9WgXcQ", "local-3fa9c2", "yt-abc_-.x"] {
            assert!(validar_id(id).is_ok(), "{id}");
        }
        for id in [
            "",
            ".",
            "..",
            "../x",
            "a/b",
            "a\\b",
            "C:",
            "x..y",
            "/abs",
            "con espacio",
            "ñ",
        ] {
            assert!(validar_id(id).is_err(), "{id}");
        }
    }
}
