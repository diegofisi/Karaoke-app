//! Herramientas externas: ffmpeg, deno (JavaScript para YouTube) y uv
//! (instala el Python de la fabrica). Primero las que van DENTRO de la app
//! (sidecars de Tauri, junto al ejecutable; las baja
//! scripts/descargar-binarios.mjs); si no, las del PATH. En macOS se miran
//! tambien las carpetas de Homebrew: una app abierta desde el Finder no
//! hereda el PATH de la terminal.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

const EXE: &str = if cfg!(windows) { ".exe" } else { "" };

/// Carpeta del ejecutable de la app (donde Tauri deja los sidecars).
pub fn carpeta_app() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(Path::to_path_buf)
}

/// Sidecar empaquetado con la app.
pub fn empaquetada(nombre: &str) -> Option<PathBuf> {
    let ruta = carpeta_app()?.join(format!("{nombre}{EXE}"));
    ruta.is_file().then_some(ruta)
}

fn carpetas_path() -> Vec<PathBuf> {
    let mut carpetas: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    if cfg!(target_os = "macos") {
        carpetas.extend(["/opt/homebrew/bin", "/usr/local/bin"].map(PathBuf::from));
        if let Some(casa) = std::env::var_os("HOME").map(PathBuf::from) {
            carpetas.push(casa.join(".deno").join("bin"));
            carpetas.push(casa.join(".bun").join("bin"));
        }
    }
    carpetas
}

/// Instalada en el sistema (PATH y carpetas habituales).
pub fn en_sistema(nombre: &str) -> Option<PathBuf> {
    carpetas_path()
        .into_iter()
        .map(|c| c.join(format!("{nombre}{EXE}")))
        .find(|p| p.is_file())
}

/// La empaquetada o, si no, la del sistema.
pub fn buscar(nombre: &str) -> Option<PathBuf> {
    empaquetada(nombre).or_else(|| en_sistema(nombre))
}

/// Runtime de JavaScript para yt-dlp: deno empaquetado, o deno/node/bun del sistema.
pub fn runtime_js() -> Option<PathBuf> {
    empaquetada("deno").or_else(|| ["deno", "node", "bun"].iter().find_map(|n| en_sistema(n)))
}

/// PATH para los procesos que lanza la app: la carpeta de los sidecars
/// primero (asi yt-dlp encuentra ffmpeg y deno), luego el PATH normal y, en
/// macOS, las carpetas de Homebrew.
pub fn path_para_procesos() -> OsString {
    let mut carpetas: Vec<PathBuf> = carpeta_app().into_iter().collect();
    carpetas.extend(carpetas_path());
    std::env::join_paths(carpetas).unwrap_or_else(|_| std::env::var_os("PATH").unwrap_or_default())
}
