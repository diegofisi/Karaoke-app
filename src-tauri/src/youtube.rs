//! Sesion de YouTube del usuario (adaptado de diegofisi/youtube-downloader,
//! features/session). La app abre una ventana con el inicio de sesion de
//! Google; al llegar a youtube.com con la sesion iniciada lee sus cookies
//! (HttpOnly incluidas) y las guarda en `<datos>/youtube/cookies.txt`
//! (formato Netscape). La fabrica las usa cuando YouTube pide "confirmar que
//! no eres un bot".

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// User-Agent de navegador para las ventanas de inicio de sesion y las
/// peticiones autenticadas: con el del webview, Google rechaza el inicio de
/// sesion. En Mac, el de Safari (el webview es WebKit); sin probar en un Mac.
#[cfg(not(target_os = "macos"))]
pub const AGENTE_NAVEGADOR: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
#[cfg(target_os = "macos")]
pub const AGENTE_NAVEGADOR: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15";

/// Nombre de la copia de cookies que la fabrica usa en cada ejecucion.
pub const COPIA_EJECUCION: &str = "cookies-ejecucion.txt";

/// Borra las copias que quedaron en las carpetas de canciones.
pub fn limpiar_copias(canciones: &Path) {
    let Ok(entradas) = fs::read_dir(canciones) else {
        return;
    };
    for e in entradas.flatten() {
        let _ = fs::remove_file(e.path().join(COPIA_EJECUCION));
    }
}

/// Inicio de sesion "pasivo": si Google ya tiene sesion, redirige a youtube.com sin pedir nada.
pub const URL_INICIO_SESION: &str = "https://accounts.google.com/ServiceLogin?service=youtube&passive=true&continue=https%3A%2F%2Fwww.youtube.com%2F";

/// Dominios de los que se leen cookies: juntos traen todas las de sesion.
pub const DOMINIOS_COOKIES: [&str; 4] = [
    "https://www.youtube.com",
    "https://youtube.com",
    "https://accounts.google.com",
    "https://www.google.com",
];

pub fn ruta_cookies(datos: &Path) -> PathBuf {
    datos.join("youtube").join("cookies.txt")
}

/// `EstadoSesion` de CONTRATO seccion 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EstadoSesion {
    /// Hay una cookie fuerte de `.youtube.com` vigente.
    Conectada,
    /// Solo hay cookies fuertes caducadas.
    Caducada,
    /// Sin archivo, o solo cookies anonimas.
    Ninguna,
}

/// `CuentaYoutube` de CONTRATO seccion 5.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CuentaYoutube {
    pub nombre: String,
    pub usuario: Option<String>,
    /// data URL (base64) o URL de la foto.
    pub foto: Option<String>,
}

/// Una cookie del archivo Netscape (solo los campos que se usan).
struct Registro<'a> {
    dominio: &'a str,
    nombre: &'a str,
    valor: &'a str,
    /// Caducidad unix; 0 = cookie de sesion (sin fecha).
    caduca: i64,
}

fn leer_netscape(texto: &str) -> impl Iterator<Item = Registro<'_>> {
    texto.lines().filter_map(|crudo| {
        let linea = crudo.trim();
        if linea.is_empty() {
            return None;
        }
        let linea = linea.strip_prefix("#HttpOnly_").unwrap_or(linea);
        if linea.starts_with('#') {
            return None;
        }
        let c: Vec<&str> = linea.split('\t').collect();
        if c.len() < 7 {
            return None;
        }
        Some(Registro {
            dominio: c[0],
            nombre: c[5],
            valor: c[6],
            caduca: c[4].parse().unwrap_or(0),
        })
    })
}

fn ahora() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Cookies fuertes de `.youtube.com`: (alguna vigente, alguna caducada). yt-dlp
/// necesita la autenticacion en `.youtube.com`; las de `.google.com` no bastan.
fn cookies_fuertes(texto: &str) -> (bool, bool) {
    const FUERTES: [&str; 4] = ["SAPISID", "__Secure-3PAPISID", "LOGIN_INFO", "SSID"];
    let t = ahora();
    let (mut vigente, mut caducada) = (false, false);
    for c in leer_netscape(texto) {
        if !c.dominio.contains("youtube.com") || !FUERTES.contains(&c.nombre) {
            continue;
        }
        if c.caduca != 0 && c.caduca < t {
            caducada = true;
        } else {
            vigente = true;
        }
    }
    (vigente, caducada)
}

pub fn tiene_sesion(texto: &str) -> bool {
    cookies_fuertes(texto).0
}

pub fn estado(datos: &Path) -> EstadoSesion {
    let Ok(texto) = fs::read_to_string(ruta_cookies(datos)) else {
        return EstadoSesion::Ninguna;
    };
    match cookies_fuertes(&texto) {
        (true, _) => EstadoSesion::Conectada,
        (false, true) => EstadoSesion::Caducada,
        (false, false) => EstadoSesion::Ninguna,
    }
}

/// Una cookie leida de la ventana, ya en campos de texto (independiente de tauri, para probarla).
pub struct CookieLeida {
    pub dominio: String,
    pub ruta: String,
    pub nombre: String,
    pub valor: String,
    pub segura: bool,
    pub solo_http: bool,
    /// Caducidad unix; 0 = de sesion.
    pub caduca: i64,
}

/// Archivo Netscape con las cookies, sin duplicados (dominio|ruta|nombre).
/// Las HttpOnly llevan el prefijo `#HttpOnly_` (formato de yt-dlp).
pub fn a_netscape(cookies: &[CookieLeida]) -> (String, usize) {
    let mut salida = String::from(
        "# Netscape HTTP Cookie File\n# Generado por Karaoke (inicio de sesion en la app)\n\n",
    );
    let mut vistas = HashSet::new();
    let mut n = 0;
    for c in cookies {
        if !vistas.insert(format!("{}|{}|{}", c.dominio, c.ruta, c.nombre)) {
            continue;
        }
        let subdominios = if c.dominio.starts_with('.') {
            "TRUE"
        } else {
            "FALSE"
        };
        let segura = if c.segura { "TRUE" } else { "FALSE" };
        let dominio = if c.solo_http {
            format!("#HttpOnly_{}", c.dominio)
        } else {
            c.dominio.clone()
        };
        salida.push_str(&format!(
            "{dominio}\t{subdominios}\t{}\t{segura}\t{}\t{}\t{}\n",
            c.ruta, c.caduca, c.nombre, c.valor
        ));
        n += 1;
    }
    (salida, n)
}

/// Guarda el archivo de forma atomica (tmp + rename).
pub fn guardar(datos: &Path, contenido: &str) -> Result<(), String> {
    let ruta = ruta_cookies(datos);
    if let Some(carpeta) = ruta.parent() {
        fs::create_dir_all(carpeta)
            .map_err(|e| format!("No se pudo crear la carpeta de la sesion: {e}"))?;
    }
    let tmp = ruta.with_extension("txt.tmp");
    fs::write(&tmp, contenido)
        .map_err(|e| format!("No se pudo guardar la sesion de YouTube: {e}"))?;
    fs::rename(&tmp, &ruta).map_err(|e| format!("No se pudo guardar la sesion de YouTube: {e}"))
}

pub fn cerrar(datos: &Path) -> Result<(), String> {
    let ruta = ruta_cookies(datos);
    if ruta.exists() {
        fs::remove_file(&ruta)
            .map_err(|e| format!("No se pudo cerrar la sesion de YouTube: {e}"))?;
    }
    Ok(())
}

/// Cabecera `Cookie` solo con las de youtube.com (mezclar las de google.com
/// estropea la respuesta de account_menu) y el SAPISID para firmar.
fn cabecera_y_sapisid(texto: &str) -> (String, Option<String>) {
    let mut pares: Vec<String> = Vec::new();
    let mut vistas = HashSet::new();
    let mut sapisid = None;
    for c in leer_netscape(texto) {
        if !c.dominio.contains("youtube.com") {
            continue;
        }
        if vistas.insert(c.nombre.to_string()) {
            pares.push(format!("{}={}", c.nombre, c.valor));
        }
        if (c.nombre == "SAPISID" || c.nombre == "__Secure-3PAPISID") && sapisid.is_none() {
            sapisid = Some(c.valor.to_string());
        }
    }
    (pares.join("; "), sapisid)
}

/// Firma SAPISIDHASH: sha1 hex de "<ts> <SAPISID> <origen>".
fn sapisidhash(sapisid: &str, origen: &str) -> String {
    let ts = ahora();
    let hex = sha1_smol::Sha1::from(format!("{ts} {sapisid} {origen}"))
        .digest()
        .to_string();
    format!("SAPISIDHASH {ts}_{hex}")
}

/// Nombre, @usuario y foto de la cuenta (endpoint interno account_menu).
/// `Ok(None)` sin cookies o sin cuenta en la respuesta.
pub fn cuenta(datos: &Path) -> Result<Option<CuentaYoutube>, String> {
    let Ok(texto) = fs::read_to_string(ruta_cookies(datos)) else {
        return Ok(None);
    };
    let (cabecera, sapisid) = cabecera_y_sapisid(&texto);
    let Some(sapisid) = sapisid else {
        return Ok(None);
    };
    const ORIGEN: &str = "https://www.youtube.com";
    let cliente = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("No se pudo crear el cliente HTTP: {e}"))?;
    let resp = cliente
        .post("https://www.youtube.com/youtubei/v1/account/account_menu?prettyPrint=false")
        .header("Authorization", sapisidhash(&sapisid, ORIGEN))
        .header("Cookie", cabecera)
        .header("Content-Type", "application/json")
        .header("Origin", ORIGEN)
        .header("X-Origin", ORIGEN)
        .header("User-Agent", AGENTE_NAVEGADOR)
        .body(r#"{"context":{"client":{"clientName":"WEB","clientVersion":"2.20250101.00.00"}}}"#)
        .send()
        .map_err(|e| format!("No se pudo consultar la cuenta de YouTube: {e}"))?;
    if !resp.status().is_success() {
        return Ok(None); // sesion no valida: la interfaz sigue funcionando
    }
    let json: serde_json::Value = resp
        .json()
        .map_err(|e| format!("No se pudo leer la cuenta de YouTube: {e}"))?;
    Ok(leer_cuenta(&json).map(|mut c| {
        c.foto = c
            .foto
            .map(|u| foto_como_data_url(&cliente, &u).unwrap_or(u));
        c
    }))
}

/// Rutas comprobadas en youtube-downloader (2026-07).
fn leer_cuenta(json: &serde_json::Value) -> Option<CuentaYoutube> {
    let h = json.pointer(
        "/actions/0/openPopupAction/popup/multiPageMenuRenderer/header/activeAccountHeaderRenderer",
    )?;
    let nombre = h.pointer("/accountName/simpleText")?.as_str()?.to_string();
    let usuario = h
        .pointer("/channelHandle/simpleText")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let foto = h
        .pointer("/accountPhoto/thumbnails")
        .and_then(|v| v.as_array())
        .and_then(|lista| {
            let mut c: Vec<(u64, &str)> = lista
                .iter()
                .filter_map(|t| {
                    Some((
                        t.get("width").and_then(|w| w.as_u64()).unwrap_or(0),
                        t.get("url")?.as_str()?,
                    ))
                })
                .collect();
            c.sort_by_key(|(w, _)| *w);
            c.iter()
                .find(|(w, _)| *w >= 88)
                .or_else(|| c.last())
                .map(|(_, u)| u.to_string())
        });
    Some(CuentaYoutube {
        nombre,
        usuario,
        foto,
    })
}

/// La foto como data URL: asi la interfaz no tiene que pedirla a yt3.ggpht.com.
fn foto_como_data_url(cliente: &reqwest::blocking::Client, url: &str) -> Option<String> {
    use base64::Engine;
    let resp = cliente.get(url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let tipo = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();
    let bytes = resp.bytes().ok()?;
    if bytes.is_empty() || bytes.len() > 2 * 1024 * 1024 {
        return None;
    }
    Some(format!(
        "data:{tipo};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}

#[cfg(test)]
mod pruebas {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    fn linea(dominio: &str, caduca: &str, nombre: &str, valor: &str) -> String {
        format!("{dominio}\tTRUE\t/\tTRUE\t{caduca}\t{nombre}\t{valor}")
    }

    fn carpeta(etiqueta: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("karaoke-youtube-{etiqueta}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn lee_httponly_y_descarta_comentarios_y_lineas_cortas() {
        let texto = format!(
            "# Netscape HTTP Cookie File\n\n#HttpOnly_{}\n.youtube.com\tTRUE\t/\tTRUE\t0\tSAPISID\n",
            linea(".youtube.com", "0", "LOGIN_INFO", "x")
        );
        let v: Vec<_> = leer_netscape(&texto).collect();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].nombre, "LOGIN_INFO");
        assert_eq!(v[0].dominio, ".youtube.com");
    }

    #[test]
    fn estado_segun_las_cookies_fuertes() {
        let d = carpeta("estado");
        assert_eq!(estado(&d), EstadoSesion::Ninguna);
        guardar(&d, &linea(".youtube.com", "0", "PREF", "v")).unwrap();
        assert_eq!(
            estado(&d),
            EstadoSesion::Ninguna,
            "solo anonimas no es sesion"
        );
        guardar(&d, &linea(".google.com", "1900000000", "SAPISID", "v")).unwrap();
        assert_eq!(
            estado(&d),
            EstadoSesion::Ninguna,
            "las de google.com no bastan"
        );
        guardar(&d, &linea(".youtube.com", "1000000", "SAPISID", "v")).unwrap();
        assert_eq!(estado(&d), EstadoSesion::Caducada);
        guardar(&d, &linea(".youtube.com", "0", "LOGIN_INFO", "v")).unwrap();
        assert_eq!(estado(&d), EstadoSesion::Conectada);
        cerrar(&d).unwrap();
        assert_eq!(estado(&d), EstadoSesion::Ninguna);
    }

    #[test]
    fn a_netscape_quita_duplicados_y_marca_httponly() {
        let c = |nombre: &str, solo_http: bool| CookieLeida {
            dominio: ".youtube.com".into(),
            ruta: "/".into(),
            nombre: nombre.into(),
            valor: "v".into(),
            segura: true,
            solo_http,
            caduca: 1_900_000_000,
        };
        let (texto, n) = a_netscape(&[
            c("SAPISID", false),
            c("SAPISID", false),
            c("LOGIN_INFO", true),
        ]);
        assert_eq!(n, 2);
        assert!(texto.contains("#HttpOnly_.youtube.com\tTRUE\t/\tTRUE\t1900000000\tLOGIN_INFO\tv"));
        assert!(tiene_sesion(&texto));
    }

    #[test]
    fn limpiar_copias_borra_solo_la_copia() {
        let d = carpeta("copias");
        let cancion = d.join("yt-x");
        fs::create_dir_all(&cancion).unwrap();
        fs::write(cancion.join(COPIA_EJECUCION), "x").unwrap();
        fs::write(cancion.join("cancion.json"), "{}").unwrap();
        limpiar_copias(&d);
        assert!(!cancion.join(COPIA_EJECUCION).exists());
        assert!(cancion.join("cancion.json").exists());
    }

    #[test]
    fn sapisidhash_tiene_el_formato_de_youtube() {
        let firma = sapisidhash("abc", "https://www.youtube.com");
        let resto = firma.strip_prefix("SAPISIDHASH ").unwrap();
        let (ts, hex) = resto.split_once('_').unwrap();
        assert!(ts.parse::<u64>().is_ok());
        assert_eq!(hex.len(), 40);
    }

    #[test]
    fn lee_la_cuenta_de_account_menu() {
        let json = serde_json::json!({"actions":[{"openPopupAction":{"popup":{"multiPageMenuRenderer":{"header":{"activeAccountHeaderRenderer":{
            "accountName":{"simpleText":"Diego"},"channelHandle":{"simpleText":"@diego"},
            "accountPhoto":{"thumbnails":[{"url":"p48","width":48},{"url":"p88","width":88},{"url":"p176","width":176}]}}}}}}}]});
        let c = leer_cuenta(&json).unwrap();
        assert_eq!(c.nombre, "Diego");
        assert_eq!(c.usuario.as_deref(), Some("@diego"));
        assert_eq!(c.foto.as_deref(), Some("p88"));
        assert!(leer_cuenta(&serde_json::json!({})).is_none());
    }
}
