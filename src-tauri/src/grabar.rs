//! Guarda la grabacion de una partida: mezcla lo que sonaba con la voz del
//! usuario (crate audio), la convierte a MP3 con ffmpeg y la registra con su
//! `.json` en la carpeta de grabaciones (Musica/Karaoke). Si ffmpeg no esta o
//! falla, se queda en WAV: la grabacion no se pierde.

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use audio::{GrabacionVoz, escribir_wav, leer_wav, mezclar};
use biblioteca::grabaciones::{self, nombre_seguro};
use biblioteca::{DatosGrabacion, Grabacion, fecha};

/// Menos voz que esto no se guarda (una partida abierta y cerrada sin cantar).
pub const VOZ_MINIMA_S: f64 = 2.0;

pub struct Datos<'a> {
    pub carpeta_cancion: &'a Path,
    pub carpeta_grabaciones: &'a Path,
    pub id: &'a str,
    pub titulo: &'a str,
    pub artista: &'a str,
    pub volumen_musica: f64,
    /// Volumen de la voz guia al terminar la partida (en practica se puede cambiar al vuelo).
    pub volumen_guia: f64,
    pub volumen_voz: f64,
}

/// `2026-10-15T22:41:07Z` -> `2026-10-15 22-41-07` (los `:` no valen en Windows).
fn sello(rfc3339: &str) -> String {
    rfc3339
        .trim_end_matches('Z')
        .replace('T', " ")
        .replace(':', "-")
}

/// Nombre libre: ni su .wav, ni su .mp3, ni su .json existen. Si hace falta
/// anade " (2)", " (3)"... Un solo nombre para los tres archivos.
fn nombre_libre(carpeta: &Path, base: &str) -> String {
    let libre = |nombre: &str| {
        ["wav", "mp3", "json"]
            .iter()
            .all(|e| !carpeta.join(format!("{nombre}.{e}")).exists())
    };
    if libre(base) {
        return base.to_string();
    }
    (2..1000)
        .map(|n| format!("{base} ({n})"))
        .find(|n| libre(n))
        .unwrap_or_else(|| format!("{base} ({})", std::process::id()))
}

/// Convierte a MP3. Si falla, borra el MP3 a medias.
fn a_mp3(ffmpeg: &Path, wav: &Path, mp3: &Path) -> bool {
    let mut c = Command::new(ffmpeg);
    c.args(["-y", "-loglevel", "error", "-i"])
        .arg(wav)
        .args(["-codec:a", "libmp3lame", "-q:a", "2"])
        .arg(mp3)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let ok = c.status().is_ok_and(|s| s.success()) && mp3.is_file();
    if !ok {
        let _ = fs::remove_file(mp3);
    }
    ok
}

/// Devuelve la grabacion y, si quedo en WAV, un aviso para el usuario.
pub fn guardar(d: &Datos<'_>, voz: &GrabacionVoz) -> Result<(Grabacion, Option<String>), String> {
    if voz.duracion() < VOZ_MINIMA_S {
        return Err("No se pudo guardar la grabacion: el microfono casi no capto voz.".into());
    }
    fs::create_dir_all(d.carpeta_grabaciones)
        .map_err(|e| format!("No se pudo crear la carpeta de grabaciones: {e}"))?;

    let musica =
        leer_wav(&d.carpeta_cancion.join("instrumental.wav")).map_err(|e| e.to_string())?;
    let guia = if d.volumen_guia > 0.0 {
        leer_wav(&d.carpeta_cancion.join("voz.wav")).ok()
    } else {
        None
    };
    let mezcla = mezclar(
        &musica,
        d.volumen_musica as f32,
        guia.as_ref().map(|g| (g, d.volumen_guia as f32)),
        voz,
        d.volumen_voz as f32,
    )
    .map_err(|e| e.to_string())?;
    let duracion = mezcla.muestras.len() as f64 / 2.0 / f64::from(mezcla.frecuencia.max(1));

    let fecha = fecha::ahora_rfc3339();
    let base = nombre_seguro(&format!("{} - {} {}", d.artista, d.titulo, sello(&fecha)));
    let nombre = nombre_libre(d.carpeta_grabaciones, &base);
    let wav = d.carpeta_grabaciones.join(format!("{nombre}.wav"));
    escribir_wav(&wav, &mezcla).map_err(|e| e.to_string())?;
    drop(mezcla);

    let mp3 = d.carpeta_grabaciones.join(format!("{nombre}.mp3"));
    let (audio, aviso) = match crate::herramientas::buscar("ffmpeg") {
        Some(ffmpeg) if a_mp3(&ffmpeg, &wav, &mp3) => {
            let _ = fs::remove_file(&wav);
            (mp3, None)
        }
        Some(_) => (
            wav,
            Some("Se guardo en WAV: ffmpeg no pudo convertirla a MP3.".to_string()),
        ),
        None => (
            wav,
            Some("Se guardo en WAV: no se encontro ffmpeg para convertirla a MP3.".to_string()),
        ),
    };
    let grabacion = grabaciones::registrar(
        &audio,
        DatosGrabacion {
            id_cancion: d.id.to_string(),
            titulo: d.titulo.to_string(),
            artista: d.artista.to_string(),
            fecha,
            duracion,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok((grabacion, aviso))
}

#[cfg(test)]
mod pruebas {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn sello_sin_dos_puntos() {
        assert_eq!(sello("2026-10-15T22:41:07Z"), "2026-10-15 22-41-07");
    }

    #[test]
    fn nombre_libre_mira_las_tres_extensiones() {
        let dir = std::env::temp_dir().join("karaoke_prueba_nombre_libre");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(nombre_libre(&dir, "x"), "x");
        // Un .json suelto basta para ocupar el nombre: no se comparte con otro audio.
        fs::write(dir.join("x.json"), b"{}").unwrap();
        assert_eq!(nombre_libre(&dir, "x"), "x (2)");
    }
}
