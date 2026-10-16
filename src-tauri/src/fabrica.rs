//! Lanza la fabrica Python (CONTRATO seccion 4) y reenvia sus lineas JSON por
//! un canal. Windows y macOS: solo cambian la ruta del Python del venv y la
//! forma de matar el proceso con sus hijos.

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::Ordering;
use std::sync::{Arc, OnceLock};

use regex::Regex;
use serde_json::{Value, json};

use crate::entorno;
use crate::herramientas;
use crate::tipos::{EstadoFabrica, PeticionProceso};

/// Donde esta el Python de la fabrica y desde que carpeta se lanza.
#[derive(Debug, Clone)]
pub struct Ubicacion {
    pub python: PathBuf,
    pub carpeta: PathBuf,
}

pub fn python_de_venv(venv: &Path) -> PathBuf {
    if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}

/// Carpeta de la fabrica elegida en Ajustes: la del repositorio o la propia `fabrica/`.
fn carpeta_fabrica(elegida: &Path) -> PathBuf {
    if elegida.join("fabrica").join("karaoke_fabrica").is_dir() {
        elegida.join("fabrica")
    } else {
        elegida.to_path_buf()
    }
}

/// Orden: `KARAOKE_PYTHON` > carpeta elegida en Ajustes > entorno que instala
/// la app (`<datos>/entorno`) > repositorio de desarrollo. Si no hay ninguno,
/// devuelve el del entorno de la app (aun sin instalar): el diagnostico dira
/// que falta y la interfaz ofrecera instalarlo.
pub fn ubicar(elegida: Option<&Path>, datos: &Path) -> Ubicacion {
    if let Some(ruta) = std::env::var_os("KARAOKE_PYTHON") {
        return Ubicacion {
            python: PathBuf::from(ruta),
            carpeta: datos.to_path_buf(),
        };
    }
    if let Some(c) = elegida {
        let fabrica = carpeta_fabrica(c);
        return Ubicacion {
            python: python_de_venv(&fabrica.join(".venv")),
            carpeta: fabrica,
        };
    }
    if entorno::instalado(datos) {
        return Ubicacion {
            python: entorno::python(datos),
            carpeta: datos.to_path_buf(),
        };
    }
    let desarrollo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("fabrica");
    let python_desarrollo = python_de_venv(&desarrollo.join(".venv"));
    if python_desarrollo.is_file() {
        return Ubicacion {
            python: python_desarrollo,
            carpeta: desarrollo,
        };
    }
    Ubicacion {
        python: entorno::python(datos),
        carpeta: datos.to_path_buf(),
    }
}

fn regex_youtube() -> Option<&'static Regex> {
    static RE: OnceLock<Option<Regex>> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?:v=|youtu\.be/|shorts/|music\.youtube\.com/watch\?v=)([\w-]{11})").ok()
    })
    .as_ref()
}

/// `yt-<id>`, con la misma expresion que `fabrica/karaoke_fabrica/descarga.py`.
pub fn id_de_url(url: &str) -> Option<String> {
    let captura = regex_youtube()?.captures(url)?;
    Some(format!("yt-{}", captura.get(1)?.as_str()))
}

/// `local-<12 hex>`: sha1 de los primeros 4 MiB, como `pipeline.id_cancion`.
pub fn id_de_archivo(ruta: &Path) -> Result<String, String> {
    let mut bytes = Vec::with_capacity(4 * 1024 * 1024);
    File::open(ruta)
        .and_then(|f| f.take(4 * 1024 * 1024).read_to_end(&mut bytes))
        .map_err(|e| format!("No se pudo leer el archivo {}: {e}", ruta.display()))?;
    let hex = sha1_smol::Sha1::from(&bytes).digest().to_string();
    Ok(format!("local-{}", &hex[..12]))
}

pub fn id_de_peticion(p: &PeticionProceso) -> Result<String, String> {
    if let Some(url) = p.url.as_deref().filter(|u| !u.trim().is_empty()) {
        return id_de_url(url)
            .ok_or_else(|| "No se pudo leer el ID del video en la URL.".to_string());
    }
    if let Some(archivo) = p.archivo.as_deref().filter(|a| !a.trim().is_empty()) {
        return id_de_archivo(Path::new(archivo));
    }
    Err("No se pudo procesar: falta la URL o el archivo.".to_string())
}

/// Argumentos de `python -m karaoke_fabrica procesar ...`.
pub fn argumentos(p: &PeticionProceso, salida: &Path, letra: Option<&Path>) -> Vec<String> {
    let mut a = vec!["-m".into(), "karaoke_fabrica".into(), "procesar".into()];
    if let Some(url) = p.url.as_deref().filter(|u| !u.trim().is_empty()) {
        a.extend(["--url".into(), url.trim().into()]);
    } else if let Some(archivo) = p.archivo.as_deref() {
        a.extend(["--audio".into(), archivo.into()]);
    }
    a.extend(["--salida".into(), salida.display().to_string()]);
    for (opcion, valor) in [
        ("--titulo", &p.titulo),
        ("--artista", &p.artista),
        ("--idioma", &p.idioma),
    ] {
        if let Some(v) = valor.as_deref().filter(|v| !v.trim().is_empty()) {
            a.extend([opcion.into(), v.trim().into()]);
        }
    }
    if let Some(l) = letra {
        a.extend(["--letra".into(), l.display().to_string()]);
    }
    a
}

fn comando_python(u: &Ubicacion) -> Command {
    let mut c = Command::new(&u.python);
    c.current_dir(&u.carpeta)
        // Herramientas empaquetadas primero: yt-dlp encuentra ffmpeg y deno.
        .env("PATH", herramientas::path_para_procesos())
        // stdout sin buffer y en UTF-8 (la consola de Windows no lo es por defecto).
        .env("PYTHONUNBUFFERED", "1")
        .env("PYTHONIOENCODING", "utf-8")
        .stdin(Stdio::null());
    sin_ventana(&mut c);
    c
}

#[cfg(windows)]
pub(crate) fn sin_ventana(c: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    c.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(unix)]
pub(crate) fn sin_ventana(c: &mut Command) {
    use std::os::unix::process::CommandExt;
    // Grupo propio: cancelar mata al proceso y a sus hijos (ffmpeg).
    c.process_group(0);
}

/// Mata el proceso de la fabrica y sus hijos.
pub fn matar(pid: u32) {
    #[cfg(windows)]
    let mut c = {
        let mut c = Command::new("taskkill");
        c.args(["/PID", &pid.to_string(), "/T", "/F"]);
        sin_ventana(&mut c);
        c
    };
    #[cfg(unix)]
    let mut c = {
        let mut c = Command::new("kill");
        c.args(["-TERM", &format!("-{pid}")]);
        c
    };
    // Si ya termino, el comando falla y no importa.
    let _ = c.stdout(Stdio::null()).stderr(Stdio::null()).status();
}

/// Arranca la fabrica. Devuelve el proceso con su stdout entubado.
pub fn arrancar(p: &PeticionProceso, carpeta: &Path, u: &Ubicacion) -> Result<Child, String> {
    fs::create_dir_all(carpeta)
        .map_err(|e| format!("No se pudo crear la carpeta de la cancion: {e}"))?;
    let letra = match p.letra.as_deref().filter(|l| !l.trim().is_empty()) {
        Some(texto) => {
            let ruta = carpeta.join("letra_usuario.txt");
            fs::write(&ruta, texto).map_err(|e| format!("No se pudo guardar la letra: {e}"))?;
            Some(ruta)
        }
        None => None,
    };
    let errores = File::create(carpeta.join("fabrica.stderr.log"))
        .map_err(|e| format!("No se pudo crear el registro de la fabrica: {e}"))?;
    comando_python(u)
        .args(argumentos(p, carpeta, letra.as_deref()))
        .stdout(Stdio::piped())
        .stderr(Stdio::from(errores))
        .spawn()
        .map_err(|e| {
            format!(
                "No se pudo arrancar la fabrica ({}): {e}. Instala su entorno (fabrica/README.md) o elige su carpeta en Ajustes",
                u.python.display()
            )
        })
}

/// Lee el stdout de la fabrica hasta el final y reenvia cada objeto. Al
/// terminar garantiza un evento final: `listo`, `error` o `cancelado`.
pub fn reenviar(
    mut hijo: Child,
    cancelado: Arc<std::sync::atomic::AtomicBool>,
    mut enviar: impl FnMut(Value),
) {
    let mut final_visto = false;
    if let Some(salida) = hijo.stdout.take() {
        for linea in BufReader::new(salida).lines() {
            let Ok(linea) = linea else { break };
            let Ok(valor) = serde_json::from_str::<Value>(&linea) else {
                continue;
            };
            if matches!(
                valor.get("tipo").and_then(Value::as_str),
                Some("listo" | "error")
            ) {
                final_visto = true;
            }
            enviar(valor);
        }
    }
    let codigo = hijo.wait().ok().and_then(|s| s.code());
    if cancelado.load(Ordering::Acquire) {
        enviar(json!({ "tipo": "cancelado" }));
    } else if !final_visto {
        let codigo = codigo.map_or_else(|| "desconocido".to_string(), |c| c.to_string());
        enviar(json!({
            "tipo": "error",
            "etapa": "descarga",
            "mensaje": format!("No se pudo procesar: la fabrica termino sin respuesta (codigo {codigo}). Mira fabrica.stderr.log en la carpeta de la cancion."),
        }));
    }
}

/// `python -m karaoke_fabrica diagnostico`.
pub fn diagnostico(u: &Ubicacion) -> EstadoFabrica {
    let salida = comando_python(u)
        .args(["-m", "karaoke_fabrica", "diagnostico"])
        .stderr(Stdio::null())
        .output();
    let sin_python = |mensaje: String| EstadoFabrica {
        lista: false,
        python: None,
        cuda: false,
        gpu: None,
        mensaje: Some(mensaje),
        requisitos: Vec::new(),
        puede_instalar: false,
    };
    let salida = match salida {
        Ok(s) => s,
        Err(e) => {
            return sin_python(format!(
                "No se encontro el Python de la fabrica en {} ({e}). Instala su entorno (fabrica/README.md) o elige su carpeta en Ajustes",
                u.python.display()
            ));
        }
    };
    let texto = String::from_utf8_lossy(&salida.stdout);
    let Some(v) = texto
        .lines()
        .rev()
        .find_map(|l| serde_json::from_str::<Value>(l).ok())
    else {
        return sin_python("No se pudo leer el diagnostico de la fabrica.".into());
    };
    let aceleracion = v
        .get("aceleracion")
        .and_then(Value::as_str)
        .unwrap_or("cpu");
    EstadoFabrica {
        lista: v.get("torch").is_some_and(|t| !t.is_null()),
        python: v.get("python").and_then(Value::as_str).map(str::to_string),
        cuda: v.get("cuda").and_then(Value::as_bool).unwrap_or(false),
        gpu: v.get("gpu").and_then(Value::as_str).map(str::to_string),
        mensaje: (aceleracion == "cpu")
            .then(|| "Sin GPU: procesar una cancion puede tardar mucho.".to_string()),
        requisitos: Vec::new(),
        puede_instalar: false,
    }
}

#[cfg(test)]
mod pruebas {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    fn peticion() -> PeticionProceso {
        PeticionProceso {
            url: None,
            archivo: None,
            titulo: None,
            artista: None,
            idioma: None,
            letra: None,
        }
    }

    #[test]
    fn id_de_url_como_la_fabrica() {
        assert_eq!(
            id_de_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=x").as_deref(),
            Some("yt-dQw4w9WgXcQ")
        );
        assert_eq!(
            id_de_url("https://youtu.be/dQw4w9WgXcQ?t=3").as_deref(),
            Some("yt-dQw4w9WgXcQ")
        );
        assert_eq!(
            id_de_url("https://www.youtube.com/shorts/abcdefghijk").as_deref(),
            Some("yt-abcdefghijk")
        );
        assert_eq!(id_de_url("https://ejemplo.com"), None);
    }

    #[test]
    fn id_de_archivo_es_sha1_de_los_primeros_4_mib() {
        let dir = std::env::temp_dir().join("karaoke_prueba_id");
        fs::create_dir_all(&dir).unwrap();
        let ruta = dir.join("a.bin");
        fs::write(&ruta, b"hola").unwrap();
        // sha1("hola") = 99800b85d3383e3a2fb45eb7d0066a4879a9dad0
        assert_eq!(id_de_archivo(&ruta).unwrap(), "local-99800b85d338");
    }

    #[test]
    fn argumentos_con_url_letra_e_idioma() {
        let mut p = peticion();
        p.url = Some(" https://youtu.be/dQw4w9WgXcQ ".into());
        p.idioma = Some("es".into());
        p.titulo = Some("  ".into());
        let a = argumentos(&p, Path::new("salida"), Some(Path::new("letra.txt")));
        assert_eq!(
            a,
            [
                "-m",
                "karaoke_fabrica",
                "procesar",
                "--url",
                "https://youtu.be/dQw4w9WgXcQ",
                "--salida",
                "salida",
                "--idioma",
                "es",
                "--letra",
                "letra.txt"
            ]
        );
    }

    #[test]
    fn sin_url_ni_archivo_es_error() {
        assert!(id_de_peticion(&peticion()).is_err());
    }
}
