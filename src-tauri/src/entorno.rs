//! Entorno de la fabrica que instala la propia app, para que el usuario no
//! tenga que preparar nada a mano. Usa `uv` (empaquetado con la app) para
//! bajar Python 3.12, crear un venv, instalar PyTorch (CUDA si hay NVIDIA en
//! Windows; CPU sin NVIDIA; MPS en Mac), instalar la fabrica (empaquetada como
//! recurso de la app) y descargar los modelos (`karaoke_fabrica preparar`).
//! Todo queda en `<datos>/entorno/`; `listo.json` marca que termino bien.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc;
use std::thread;

use serde::Serialize;
use serde_json::Value;

use crate::fabrica::python_de_venv;
use crate::herramientas;

const PYTHON: &str = "3.12";
const TORCH: [&str; 3] = ["torch==2.14.1", "torchaudio==2.11.0", "torchvision==0.29.1"];
const INDICE_CUDA: &str = "https://download.pytorch.org/whl/cu130";
const INDICE_CPU: &str = "https://download.pytorch.org/whl/cpu";
const MARCA: &str = "listo.json";

/// Proceso de la instalacion en curso (0 = ninguno). Solo hay una instalacion
/// a la vez (`Estado::instalando`): al cerrar la app se mata este proceso.
static PID_EN_CURSO: AtomicU32 = AtomicU32::new(0);

/// Mata el proceso de instalacion en curso, si lo hay (al cerrar la app).
pub fn detener_en_curso() {
    let pid = PID_EN_CURSO.swap(0, Ordering::AcqRel);
    if pid != 0 {
        crate::fabrica::matar(pid);
    }
}

pub fn carpeta(datos: &Path) -> PathBuf {
    datos.join("entorno")
}

pub fn python(datos: &Path) -> PathBuf {
    python_de_venv(&carpeta(datos).join("venv"))
}

pub fn instalado(datos: &Path) -> bool {
    carpeta(datos).join(MARCA).is_file() && python(datos).is_file()
}

/// Version de la app; cada version lleva su codigo de la fabrica (recurso).
pub const VERSION_APP: &str = env!("CARGO_PKG_VERSION");

/// Version de la app que instalo la fabrica (en `listo.json`); `None` si la
/// marca es de antes de guardarla (0.2.6 o anterior).
pub fn version_instalada(datos: &Path) -> Option<String> {
    let texto = fs::read_to_string(carpeta(datos).join(MARCA)).ok()?;
    let v: Value = serde_json::from_str(&texto).ok()?;
    v.get("version").and_then(Value::as_str).map(str::to_string)
}

/// La fabrica instalada es la de esta version de la app. Si no, hay que
/// reinstalarla: rapido, porque PyTorch y los modelos ya estan.
pub fn actualizada(datos: &Path) -> bool {
    version_instalada(datos).as_deref() == Some(VERSION_APP)
}

/// Progreso de `instalar_fabrica` (CONTRATO seccion 6).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum EventoInstalacion {
    /// Empieza el paso `indice` (desde 1) de `total`.
    Paso {
        indice: u32,
        total: u32,
        nombre: String,
        mensaje: String,
    },
    /// Una linea de salida del paso en curso (para mostrar actividad).
    Linea { texto: String },
}

/// Hay GPU NVIDIA (Windows): decide entre PyTorch con CUDA o solo CPU.
fn hay_nvidia() -> bool {
    herramientas::en_sistema("nvidia-smi").is_some()
        || Path::new(r"C:\Windows\System32\nvidia-smi.exe").is_file()
}

/// Variante de PyTorch para este equipo y sus argumentos para `uv pip install`.
fn torch_para_este_equipo() -> (&'static str, Vec<String>) {
    let mut args: Vec<String> = TORCH.iter().map(|s| s.to_string()).collect();
    if cfg!(target_os = "macos") {
        return ("mps", args); // PyPI trae la version con MPS
    }
    let (variante, indice) = if hay_nvidia() {
        ("cuda", INDICE_CUDA)
    } else {
        ("cpu", INDICE_CPU)
    };
    args.extend(["--index-url".to_string(), indice.to_string()]);
    (variante, args)
}

/// Ejecuta un comando y manda cada linea de stdout y stderr a `al_texto`.
fn ejecutar(mut cmd: Command, mut al_texto: impl FnMut(String)) -> Result<(), String> {
    crate::fabrica::sin_ventana(&mut cmd);
    let nombre = format!("{:?}", cmd.get_program());
    let mut hijo = cmd
        .env("PATH", herramientas::path_para_procesos())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("No se pudo ejecutar {nombre}: {e}"))?;
    PID_EN_CURSO.store(hijo.id(), Ordering::Release);
    let (tx, rx) = mpsc::channel::<String>();
    let mut lectores = Vec::new();
    if let Some(salida) = hijo.stdout.take() {
        let tx = tx.clone();
        lectores.push(thread::spawn(move || {
            for l in BufReader::new(salida).lines().map_while(Result::ok) {
                let _ = tx.send(l);
            }
        }));
    }
    if let Some(errores) = hijo.stderr.take() {
        let tx = tx.clone();
        lectores.push(thread::spawn(move || {
            for l in BufReader::new(errores).lines().map_while(Result::ok) {
                let _ = tx.send(l);
            }
        }));
    }
    drop(tx);
    let mut ultimas: Vec<String> = Vec::new();
    for linea in rx {
        let linea = linea.trim().to_string();
        if linea.is_empty() {
            continue;
        }
        ultimas.push(linea.clone());
        if ultimas.len() > 6 {
            ultimas.remove(0);
        }
        al_texto(linea);
    }
    for l in lectores {
        let _ = l.join();
    }
    let estado = hijo
        .wait()
        .map_err(|e| format!("No se pudo esperar a {nombre}: {e}"));
    PID_EN_CURSO.store(0, Ordering::Release);
    let estado = estado?;
    if estado.success() {
        Ok(())
    } else {
        Err(format!(
            "{nombre} fallo ({estado}). Ultimas lineas:\n{}",
            ultimas.join("\n")
        ))
    }
}

/// Instala el entorno completo. `fuente_fabrica` es la carpeta con
/// `pyproject.toml` y `karaoke_fabrica/` (recurso de la app). Bloquea varios
/// minutos (descarga ~5 GB de PyTorch y ~8 GB de modelos): llamalo en un hilo.
pub fn instalar(
    datos: &Path,
    fuente_fabrica: &Path,
    mut enviar: impl FnMut(EventoInstalacion),
) -> Result<(), String> {
    let uv = herramientas::buscar("uv")
        .ok_or("No se pudo instalar la fabrica: no se encontro uv (deberia venir con la app)")?;
    if !fuente_fabrica.join("pyproject.toml").is_file() {
        return Err(format!(
            "No se pudo instalar la fabrica: falta su codigo en {}",
            fuente_fabrica.display()
        ));
    }
    let ent = carpeta(datos);
    fs::create_dir_all(&ent).map_err(|e| format!("No se pudo crear {}: {e}", ent.display()))?;
    let _ = fs::remove_file(ent.join(MARCA));
    let venv = ent.join("venv");
    let py = python(datos);
    let (variante, torch) = torch_para_este_equipo();

    let uv_cmd = |args: &[&str]| {
        let mut c = Command::new(&uv);
        c.args(args)
            .env("UV_PYTHON_INSTALL_DIR", ent.join("python"))
            .env("UV_CACHE_DIR", ent.join("cache"))
            .env("UV_NO_PROGRESS", "1");
        c
    };
    const TOTAL: u32 = 5;
    fn paso(enviar: &mut impl FnMut(EventoInstalacion), indice: u32, nombre: &str, mensaje: &str) {
        enviar(EventoInstalacion::Paso {
            indice,
            total: TOTAL,
            nombre: nombre.into(),
            mensaje: mensaje.into(),
        });
    }

    paso(
        &mut enviar,
        1,
        "python",
        &format!("Descargando Python {PYTHON}"),
    );
    ejecutar(uv_cmd(&["python", "install", PYTHON]), |t| {
        enviar(EventoInstalacion::Linea { texto: t })
    })?;

    paso(
        &mut enviar,
        2,
        "entorno",
        "Creando el entorno de la fabrica",
    );
    let venv_texto = venv.to_string_lossy().into_owned();
    ejecutar(
        uv_cmd(&["venv", &venv_texto, "--python", PYTHON, "--allow-existing"]),
        |t| enviar(EventoInstalacion::Linea { texto: t }),
    )?;

    let py_texto = py.to_string_lossy().into_owned();
    paso(
        &mut enviar,
        3,
        "pytorch",
        &format!("Instalando PyTorch ({variante}); es la descarga mas grande"),
    );
    let mut args: Vec<&str> = vec!["pip", "install", "--python", &py_texto];
    args.extend(torch.iter().map(String::as_str));
    ejecutar(uv_cmd(&args), |t| {
        enviar(EventoInstalacion::Linea { texto: t })
    })?;

    paso(
        &mut enviar,
        4,
        "fabrica",
        "Instalando la fabrica y sus librerias",
    );
    // Ruta absoluta: con una relativa ("fabrica"), uv busca un paquete de PyPI
    // con ese nombre (existe uno) en vez de la carpeta.
    let fuente = std::path::absolute(fuente_fabrica)
        .map_err(|e| format!("No se pudo leer la ruta de la fabrica: {e}"))?
        .to_string_lossy()
        .into_owned();
    // --reinstall-package: la version de pyproject.toml no siempre cambia y uv
    // daria la fabrica por instalada aunque su codigo sea de otra version.
    ejecutar(
        uv_cmd(&[
            "pip",
            "install",
            "--python",
            &py_texto,
            "--reinstall-package",
            "karaoke-fabrica",
            &fuente,
        ]),
        |t| enviar(EventoInstalacion::Linea { texto: t }),
    )?;

    paso(
        &mut enviar,
        5,
        "modelos",
        "Descargando los modelos de IA (~8 GB)",
    );
    let mut c = Command::new(&py);
    c.args(["-m", "karaoke_fabrica", "preparar"])
        .current_dir(datos)
        .env("PYTHONUNBUFFERED", "1")
        .env("PYTHONIOENCODING", "utf-8");
    let mut error_preparar: Option<String> = None;
    ejecutar(c, |linea| {
        // Las lineas JSON de la fabrica se muestran por su mensaje.
        match serde_json::from_str::<Value>(&linea) {
            Ok(v) if v.get("tipo").and_then(Value::as_str) == Some("error") => {
                error_preparar = v.get("mensaje").and_then(Value::as_str).map(str::to_string);
            }
            Ok(v) => {
                if let Some(m) = v.get("mensaje").and_then(Value::as_str) {
                    enviar(EventoInstalacion::Linea {
                        texto: m.to_string(),
                    });
                }
            }
            Err(_) => {}
        }
    })
    .map_err(|e| error_preparar.clone().unwrap_or(e))?;

    let marca = serde_json::json!({ "torch": variante, "python": PYTHON, "version": VERSION_APP });
    fs::write(ent.join(MARCA), marca.to_string())
        .map_err(|e| format!("No se pudo terminar la instalacion: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod pruebas {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    fn datos_temporales(nombre: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("karaoke-entorno-{nombre}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(carpeta(&d)).unwrap();
        d
    }

    #[test]
    fn la_marca_sin_version_cuenta_como_desactualizada() {
        let d = datos_temporales("vieja");
        fs::write(
            carpeta(&d).join(MARCA),
            r#"{"torch":"cu130","python":"3.12"}"#,
        )
        .unwrap();
        assert_eq!(version_instalada(&d), None);
        assert!(!actualizada(&d));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn la_marca_con_la_version_de_la_app_esta_al_dia() {
        let d = datos_temporales("nueva");
        let marca = serde_json::json!({ "torch": "cpu", "python": PYTHON, "version": VERSION_APP });
        fs::write(carpeta(&d).join(MARCA), marca.to_string()).unwrap();
        assert_eq!(version_instalada(&d).as_deref(), Some(VERSION_APP));
        assert!(actualizada(&d));
        fs::write(carpeta(&d).join(MARCA), r#"{"version":"0.0.1"}"#).unwrap();
        assert!(!actualizada(&d));
        let _ = fs::remove_dir_all(&d);
    }
}
