//! Tipos de `cancion.json` (docs/CONTRATO.md seccion 3) y su lectura.
//!
//! Espejo de `fabrica/karaoke_fabrica/modelo.py`. Un cambio aqui es un cambio
//! de contrato: modelo Python, interfaz TypeScript y CONTRATO.md cambian en el
//! mismo commit. Rust solo lee `cancion.json`; nunca lo escribe.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Version de `cancion.json` que entiende este crate.
pub const VERSION: u32 = 1;

/// Tolerancia en segundos al comprobar orden y solapes (los tiempos llegan
/// redondeados a milisegundos por la fabrica).
pub const TOLERANCIA: f64 = 1e-6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Idioma {
    Es,
    En,
    Ja,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrigenLetra {
    Lrclib,
    Usuario,
    Transcrita,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metodo {
    Alineada,
    Respaldo,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoNota {
    #[default]
    Normal,
    Dorada,
    Libre,
}

/// Cancion procesada por la fabrica. El orden de los campos sigue al contrato.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Cancion {
    pub version: u32,
    pub id: String,
    pub titulo: String,
    pub artista: String,
    pub duracion: f64,
    pub idioma: Idioma,
    pub origen: Origen,
    pub lineas: Vec<Linea>,
    pub tono: CurvaTono,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Origen {
    pub youtube_id: Option<String>,
    pub letra: OrigenLetra,
    pub separador: String,
    pub alineador: String,
    pub tono: String,
    pub fabrica_version: String,
    pub creada: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Linea {
    pub inicio: f64,
    pub fin: f64,
    pub texto: String,
    /// Solo japones.
    #[serde(default)]
    pub romaji: Option<String>,
    /// Calidad de la alineacion de la linea, 0.0 a 1.0.
    pub confianza: f64,
    pub metodo: Metodo,
    pub palabras: Vec<Palabra>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Palabra {
    pub texto: String,
    /// Solo japones: lectura en hiragana (furigana).
    #[serde(default)]
    pub lectura: Option<String>,
    pub inicio: f64,
    pub fin: f64,
    /// Vacia = palabra sin tono fiable: se canta pero no puntua tono.
    #[serde(default)]
    pub notas: Vec<Nota>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Nota {
    pub inicio: f64,
    pub fin: f64,
    /// Mediana del tono de referencia en la nota, MIDI con decimales.
    pub midi: f64,
    #[serde(default)]
    pub tipo: TipoNota,
}

/// Tono de referencia frame a frame: el frame `i` esta en `inicio + i * paso`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CurvaTono {
    pub inicio: f64,
    pub paso: f64,
    pub midi: Vec<Option<f64>>,
}

/// Una nota con la linea y la palabra a las que pertenece.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NotaPlana {
    pub indice_linea: usize,
    pub indice_palabra: usize,
    pub inicio: f64,
    pub fin: f64,
    pub midi: f64,
    pub tipo: TipoNota,
}

/// Lo que la biblioteca muestra de una cancion sin cargar sus lineas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResumenCancion {
    pub id: String,
    pub titulo: String,
    pub artista: String,
    pub duracion: f64,
    pub idioma: Idioma,
    pub lineas_respaldo: usize,
    pub lineas_total: usize,
}

/// Error al leer o validar `cancion.json`. El mensaje (Display) va tal cual a
/// la interfaz.
#[derive(Debug, thiserror::Error)]
pub enum ErrorCancion {
    #[error("No se pudo leer la cancion: {}: {fuente}", ruta.display())]
    Archivo {
        ruta: PathBuf,
        #[source]
        fuente: std::io::Error,
    },
    #[error("No se pudo leer la cancion: JSON invalido: {0}")]
    Json(#[from] serde_json::Error),
    #[error(
        "No se pudo leer la cancion: version {encontrada} no soportada (se esperaba {VERSION})"
    )]
    Version { encontrada: u32 },
    #[error("No se pudo leer la cancion: {0}")]
    Invalida(String),
}

impl Cancion {
    /// Lee y valida `cancion.json`.
    pub fn leer(ruta: &Path) -> Result<Cancion, ErrorCancion> {
        let texto = std::fs::read_to_string(ruta).map_err(|fuente| ErrorCancion::Archivo {
            ruta: ruta.to_path_buf(),
            fuente,
        })?;
        Cancion::desde_json(&texto)
    }

    /// Interpreta y valida el texto de `cancion.json`.
    pub fn desde_json(texto: &str) -> Result<Cancion, ErrorCancion> {
        // La version se mira antes que el resto: una version futura puede
        // traer campos que esta no entiende.
        #[derive(Deserialize)]
        struct SoloVersion {
            version: u32,
        }
        let SoloVersion { version } = serde_json::from_str(texto)?;
        if version != VERSION {
            return Err(ErrorCancion::Version {
                encontrada: version,
            });
        }
        let cancion: Cancion = serde_json::from_str(texto)?;
        cancion.validar()?;
        Ok(cancion)
    }

    /// Comprueba las garantias del contrato: version, `lineas`, `palabras` y
    /// `notas` ordenadas por `inicio`, sin solapes dentro de una linea,
    /// `inicio <= fin` y `tono.paso > 0`.
    pub fn validar(&self) -> Result<(), ErrorCancion> {
        if self.version != VERSION {
            return Err(ErrorCancion::Version {
                encontrada: self.version,
            });
        }
        if !(self.tono.paso.is_finite() && self.tono.paso > 0.0) {
            return Err(invalida(format!(
                "tono.paso debe ser positivo (es {})",
                self.tono.paso
            )));
        }
        for (i, par) in self.lineas.windows(2).enumerate() {
            if let [a, b] = par
                && b.inicio < a.inicio - TOLERANCIA
            {
                return Err(invalida(format!(
                    "las lineas {} y {} no estan ordenadas por inicio",
                    i,
                    i + 1
                )));
            }
        }
        for (il, linea) in self.lineas.iter().enumerate() {
            comprobar_intervalo(linea.inicio, linea.fin, || format!("linea {il}"))?;
            for (ip, palabra) in linea.palabras.iter().enumerate() {
                comprobar_intervalo(palabra.inicio, palabra.fin, || {
                    format!("linea {il}, palabra {ip}")
                })?;
                for (inota, nota) in palabra.notas.iter().enumerate() {
                    comprobar_intervalo(nota.inicio, nota.fin, || {
                        format!("linea {il}, palabra {ip}, nota {inota}")
                    })?;
                }
            }
            comprobar_secuencia(linea.palabras.iter().map(|p| (p.inicio, p.fin)), |i| {
                format!("linea {il}: palabras {} y {}", i, i + 1)
            })?;
            comprobar_secuencia(
                linea
                    .palabras
                    .iter()
                    .flat_map(|p| p.notas.iter().map(|n| (n.inicio, n.fin))),
                |i| format!("linea {il}: notas {} y {}", i, i + 1),
            )?;
        }
        Ok(())
    }

    /// Todas las notas de la cancion ordenadas por `inicio`.
    pub fn notas_planas(&self) -> Vec<NotaPlana> {
        let mut notas: Vec<NotaPlana> = self
            .lineas
            .iter()
            .enumerate()
            .flat_map(|(il, linea)| {
                linea
                    .palabras
                    .iter()
                    .enumerate()
                    .flat_map(move |(ip, palabra)| {
                        palabra.notas.iter().map(move |n| NotaPlana {
                            indice_linea: il,
                            indice_palabra: ip,
                            inicio: n.inicio,
                            fin: n.fin,
                            midi: n.midi,
                            tipo: n.tipo,
                        })
                    })
            })
            .collect();
        // Estable: a igual inicio se conserva el orden del archivo.
        notas.sort_by(|a, b| a.inicio.total_cmp(&b.inicio));
        notas
    }

    pub fn resumen(&self) -> ResumenCancion {
        ResumenCancion {
            id: self.id.clone(),
            titulo: self.titulo.clone(),
            artista: self.artista.clone(),
            duracion: self.duracion,
            idioma: self.idioma,
            lineas_respaldo: self
                .lineas
                .iter()
                .filter(|l| l.metodo == Metodo::Respaldo)
                .count(),
            lineas_total: self.lineas.len(),
        }
    }
}

impl CurvaTono {
    /// Tono de referencia en el tiempo `t` (segundos): el frame mas cercano,
    /// en O(1). `None` fuera de la curva o si ese frame no tiene voz.
    pub fn en(&self, t: f64) -> Option<f64> {
        if !(self.paso.is_finite() && self.paso > 0.0) {
            return None;
        }
        let posicion = ((t - self.inicio) / self.paso).round();
        if !posicion.is_finite() || posicion < 0.0 || posicion >= self.midi.len() as f64 {
            return None;
        }
        // `posicion` es entera, no negativa y menor que la longitud.
        self.midi.get(posicion as usize).copied().flatten()
    }
}

fn invalida(detalle: String) -> ErrorCancion {
    ErrorCancion::Invalida(detalle)
}

fn comprobar_intervalo(
    inicio: f64,
    fin: f64,
    donde: impl FnOnce() -> String,
) -> Result<(), ErrorCancion> {
    if !(inicio.is_finite() && fin.is_finite()) || fin < inicio - TOLERANCIA {
        return Err(invalida(format!(
            "{}: intervalo invalido ({inicio} a {fin})",
            donde()
        )));
    }
    Ok(())
}

/// Comprueba que los intervalos estan ordenados por inicio y no se solapan.
fn comprobar_secuencia(
    intervalos: impl Iterator<Item = (f64, f64)>,
    donde: impl Fn(usize) -> String,
) -> Result<(), ErrorCancion> {
    let mut anterior: Option<(f64, f64)> = None;
    for (i, (inicio, fin)) in intervalos.enumerate() {
        if let Some((inicio_ant, fin_ant)) = anterior {
            let indice = i.saturating_sub(1);
            if inicio < inicio_ant - TOLERANCIA {
                return Err(invalida(format!(
                    "{} no estan ordenadas por inicio",
                    donde(indice)
                )));
            }
            if inicio < fin_ant - TOLERANCIA {
                return Err(invalida(format!("{} se solapan", donde(indice))));
            }
        }
        anterior = Some((inicio, fin));
    }
    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn curva() -> CurvaTono {
        CurvaTono {
            inicio: 1.0,
            paso: 0.01,
            midi: vec![None, Some(60.0), Some(61.0)],
        }
    }

    #[test]
    fn curva_en_busca_el_frame_mas_cercano() {
        let c = curva();
        assert_eq!(c.en(1.0), None);
        assert_eq!(c.en(1.011), Some(60.0));
        assert_eq!(c.en(1.0249), Some(61.0));
        assert_eq!(c.en(1.03), None);
        assert_eq!(c.en(0.5), None);
        assert_eq!(c.en(f64::NAN), None);
    }
}
