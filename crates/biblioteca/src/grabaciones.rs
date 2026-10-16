//! Grabaciones de partidas: un audio (mp3, o wav si no hubo ffmpeg) y a su
//! lado un `.json` con sus datos, en la carpeta de grabaciones (Musica/Karaoke).
//! Solo se listan los audios que tienen su `.json`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// `Grabacion` de CONTRATO seccion 5.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Grabacion {
    /// Ruta absoluta del audio.
    pub ruta: String,
    /// Nombre del archivo, sin carpeta.
    pub nombre: String,
    pub id_cancion: String,
    pub titulo: String,
    pub artista: String,
    /// RFC 3339.
    pub fecha: String,
    /// Segundos.
    pub duracion: f64,
}

/// Datos guardados en el `.json` (la ruta y el nombre salen del disco).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DatosGrabacion {
    pub id_cancion: String,
    pub titulo: String,
    pub artista: String,
    pub fecha: String,
    pub duracion: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorGrabacion {
    #[error("No se pudo leer la carpeta de grabaciones {}: {fuente}", ruta.display())]
    Leer {
        ruta: PathBuf,
        #[source]
        fuente: std::io::Error,
    },
    #[error("No se pudo usar la grabacion: no esta en la carpeta de grabaciones")]
    FueraDeCarpeta,
    #[error("No se pudo borrar la grabacion: {0}")]
    Borrar(#[source] std::io::Error),
    #[error("No se pudo guardar los datos de la grabacion: {0}")]
    Guardar(String),
}

const EXTENSIONES: [&str; 2] = ["mp3", "wav"];

fn sidecar(audio: &Path) -> PathBuf {
    audio.with_extension("json")
}

/// Nombre de archivo seguro en Windows y macOS: sin `\ / : * ? " < > |` ni control.
pub fn nombre_seguro(texto: &str) -> String {
    let limpio: String = texto
        .chars()
        .map(|c| {
            if c.is_control() || r#"\/:*?"<>|"#.contains(c) {
                ' '
            } else {
                c
            }
        })
        .collect();
    let recortado = limpio.split_whitespace().collect::<Vec<_>>().join(" ");
    let corto: String = recortado.chars().take(80).collect();
    let final_ = corto.trim_end_matches(['.', ' ']).to_string();
    if final_.is_empty() {
        "grabacion".into()
    } else {
        final_
    }
}

/// Lista las grabaciones, la mas reciente primero (por fecha guardada).
pub fn listar(carpeta: &Path) -> Result<Vec<Grabacion>, ErrorGrabacion> {
    if !carpeta.exists() {
        return Ok(Vec::new());
    }
    let entradas = fs::read_dir(carpeta).map_err(|fuente| ErrorGrabacion::Leer {
        ruta: carpeta.to_path_buf(),
        fuente,
    })?;
    let mut lista: Vec<Grabacion> = entradas
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| EXTENSIONES.contains(&e.to_ascii_lowercase().as_str()))
        })
        .filter_map(|audio| {
            let texto = fs::read_to_string(sidecar(&audio)).ok()?;
            let d: DatosGrabacion = serde_json::from_str(&texto).ok()?;
            Some(Grabacion {
                nombre: audio.file_name()?.to_string_lossy().into_owned(),
                ruta: audio.to_string_lossy().into_owned(),
                id_cancion: d.id_cancion,
                titulo: d.titulo,
                artista: d.artista,
                fecha: d.fecha,
                duracion: d.duracion,
            })
        })
        .collect();
    lista.sort_by(|a, b| b.fecha.cmp(&a.fecha));
    Ok(lista)
}

/// Escribe el `.json` junto al audio y devuelve la `Grabacion`.
pub fn registrar(audio: &Path, datos: DatosGrabacion) -> Result<Grabacion, ErrorGrabacion> {
    let texto =
        serde_json::to_string_pretty(&datos).map_err(|e| ErrorGrabacion::Guardar(e.to_string()))?;
    fs::write(sidecar(audio), texto).map_err(|e| ErrorGrabacion::Guardar(e.to_string()))?;
    Ok(Grabacion {
        ruta: audio.to_string_lossy().into_owned(),
        nombre: audio
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        id_cancion: datos.id_cancion,
        titulo: datos.titulo,
        artista: datos.artista,
        fecha: datos.fecha,
        duracion: datos.duracion,
    })
}

/// Comprueba que `ruta` es un audio dentro de `carpeta` (no se toca nada fuera).
pub fn validar(carpeta: &Path, ruta: &Path) -> Result<PathBuf, ErrorGrabacion> {
    let base = carpeta
        .canonicalize()
        .map_err(|_| ErrorGrabacion::FueraDeCarpeta)?;
    let real = ruta
        .canonicalize()
        .map_err(|_| ErrorGrabacion::FueraDeCarpeta)?;
    let extension_ok = real
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTENSIONES.contains(&e.to_ascii_lowercase().as_str()));
    if real.parent() == Some(base.as_path()) && extension_ok {
        // Se devuelve sin canonicalizar: en Windows canonicalize anade el
        // prefijo `\\?\` que algunos programas no aceptan al abrir el archivo.
        match real.file_name() {
            Some(nombre) => Ok(carpeta.join(nombre)),
            None => Err(ErrorGrabacion::FueraDeCarpeta),
        }
    } else {
        Err(ErrorGrabacion::FueraDeCarpeta)
    }
}

/// Borra el audio y su `.json`.
pub fn borrar(carpeta: &Path, ruta: &Path) -> Result<(), ErrorGrabacion> {
    let real = validar(carpeta, ruta)?;
    fs::remove_file(&real).map_err(ErrorGrabacion::Borrar)?;
    let _ = fs::remove_file(sidecar(&real));
    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn datos(fecha: &str) -> DatosGrabacion {
        DatosGrabacion {
            id_cancion: "yt-abc".into(),
            titulo: "T".into(),
            artista: "A".into(),
            fecha: fecha.into(),
            duracion: 3.0,
        }
    }

    #[test]
    fn nombre_seguro_quita_caracteres_prohibidos() {
        assert_eq!(
            nombre_seguro("AC/DC: Back?  in *Black*"),
            "AC DC Back in Black"
        );
        assert_eq!(nombre_seguro("..."), "grabacion");
    }

    #[test]
    fn listar_registrar_y_borrar() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.mp3");
        let b = dir.path().join("b.mp3");
        let huerfano = dir.path().join("c.mp3");
        for p in [&a, &b, &huerfano] {
            fs::write(p, b"x").unwrap();
        }
        registrar(&a, datos("2026-10-15T10:00:00Z")).unwrap();
        registrar(&b, datos("2026-10-15T11:00:00Z")).unwrap();
        let lista = listar(dir.path()).unwrap();
        assert_eq!(
            lista.iter().map(|g| g.nombre.as_str()).collect::<Vec<_>>(),
            ["b.mp3", "a.mp3"]
        );

        borrar(dir.path(), &a).unwrap();
        assert!(!a.exists() && !sidecar(&a).exists());
        assert_eq!(listar(dir.path()).unwrap().len(), 1);
    }

    #[test]
    fn no_se_borra_nada_fuera_de_la_carpeta() {
        let dir = tempfile::tempdir().unwrap();
        let fuera = tempfile::tempdir().unwrap();
        let ajeno = fuera.path().join("x.mp3");
        fs::write(&ajeno, b"x").unwrap();
        assert!(borrar(dir.path(), &ajeno).is_err());
        assert!(ajeno.exists());
        let otro = dir.path().join("notas.txt");
        fs::write(&otro, b"x").unwrap();
        assert!(borrar(dir.path(), &otro).is_err());
    }
}
