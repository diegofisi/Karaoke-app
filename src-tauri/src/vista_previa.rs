//! Vista previa de una URL de YouTube antes de procesar (adaptado de
//! diegofisi/youtube-downloader, features/preview): un video o una lista, con
//! titulo, canal, duracion y miniatura de cada entrada. Usa el yt-dlp del
//! entorno de la fabrica (`python -m yt_dlp -J --flat-playlist`).

use std::path::Path;
use std::process::{Command, Stdio};

use serde::Serialize;
use serde_json::Value;

use crate::fabrica::{SesionYoutube, Ubicacion, sin_ventana};
use crate::herramientas;

/// Un video de la vista previa (CONTRATO seccion 5).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct VideoPrevio {
    pub id: String,
    pub url: String,
    pub titulo: String,
    pub canal: String,
    pub duracion: Option<f64>,
    pub miniatura: Option<String>,
    /// `None` = disponible. Si no: "privado", "necesita_sesion", "no_disponible"...
    pub problema: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum VistaPrevia {
    Video(VideoPrevio),
    Lista {
        id: String,
        titulo: String,
        canal: String,
        entradas: Vec<VideoPrevio>,
    },
}

/// Tope para mixes y radios (`list=RD...`): YouTube las genera sin fin.
const TOPE_MIX: u32 = 25;
/// Maximo de entradas de una lista normal que se muestran.
const TOPE_LISTA: u32 = 200;

/// Lo que devuelve `analizar`: la vista previa y si YouTube rechazo la sesion
/// guardada (se reintento sin ella; la interfaz debe renovarla).
pub struct Analisis {
    pub vista: VistaPrevia,
    pub sesion_rechazada: bool,
}

fn es_mix(url: &str) -> bool {
    url.contains("list=RD") || url.contains("start_radio=")
}

/// Una URL de video con `list=` apunta al video, no a la lista, salvo que sea
/// una URL de lista pura (`/playlist?list=`). Igual que en youtube-downloader:
/// asi "pegue una lista pero era una sola cancion" no procesa toda la lista.
pub fn normalizar(url: &str) -> String {
    let limpia = url.trim();
    if limpia.contains("/playlist?") || es_mix(limpia) {
        return limpia.to_string();
    }
    let Ok(mut u) = url::Url::parse(limpia) else {
        return limpia.to_string();
    };
    let tiene_video = u.query_pairs().any(|(k, _)| k == "v") || u.host_str() == Some("youtu.be");
    if tiene_video {
        let pares: Vec<(String, String)> = u
            .query_pairs()
            .filter(|(k, _)| k != "list" && k != "index" && k != "start_radio" && k != "pp")
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        u.set_query(None);
        if !pares.is_empty() {
            u.query_pairs_mut().extend_pairs(pares);
        }
    }
    u.to_string()
}

fn ejecutar(u: &Ubicacion, url: &str, sesion: Option<&Path>) -> Result<Value, String> {
    let mut c = Command::new(&u.python);
    c.args([
        "-m",
        "yt_dlp",
        "-J",
        "--flat-playlist",
        "--no-warnings",
        "--no-playlist",
        // Solo hacen falta los metadatos: sin formatos (sin runtime de JS,
        // bloqueo parcial) la vista previa sigue saliendo.
        "--ignore-no-formats-error",
    ]);
    // --no-playlist: una URL de video con list= es el video. Las listas puras no
    // tienen "v=" y yt-dlp las trata como lista igualmente.
    if es_mix(url) {
        c.args(["--playlist-end", &TOPE_MIX.to_string()]);
    } else {
        c.args(["--playlist-end", &TOPE_LISTA.to_string()]);
    }
    if let Some(js) = herramientas::runtime_js() {
        c.args([
            "--js-runtimes",
            &format!("{}:{}", nombre_runtime(&js), js.display()),
        ]);
    }
    if let Some(s) = sesion {
        c.args(["--cookies", &s.display().to_string()]);
    }
    c.arg("--").arg(url);
    c.current_dir(&u.carpeta)
        .env("PATH", herramientas::path_para_procesos())
        .env("PYTHONIOENCODING", "utf-8")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    sin_ventana(&mut c);
    let salida = c
        .output()
        .map_err(|e| format!("No se pudo analizar la URL: {e}"))?;
    if !salida.status.success() {
        let err = String::from_utf8_lossy(&salida.stderr);
        let detalle = err
            .lines()
            .find(|l| l.trim_start().starts_with("ERROR:"))
            .map(|l| l.trim().trim_start_matches("ERROR:").trim().to_string())
            .unwrap_or_else(|| err.trim().to_string());
        return Err(explicar_error(&detalle));
    }
    serde_json::from_slice(&salida.stdout)
        .map_err(|e| format!("No se pudo leer la respuesta de yt-dlp: {e}"))
}

/// Etiqueta que yt-dlp acepta en `--js-runtimes` (deno, node, bun). El sidecar
/// en desarrollo se llama `deno-x86_64-pc-windows-msvc.exe`: con ese nombre
/// yt-dlp lo ignoraba y no sacaba ningun formato.
fn nombre_runtime(ruta: &std::path::Path) -> &'static str {
    let stem = ruta
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_lowercase();
    ["deno", "node", "bun"]
        .into_iter()
        .find(|n| stem.starts_with(n))
        .unwrap_or("deno")
}

/// Mensaje en espanol para el fallo de yt-dlp; los casos corrientes se
/// traducen y el resto lleva el texto original detras.
fn explicar_error(detalle: &str) -> String {
    let d = detalle.to_lowercase();
    let motivo = if d.contains("private video") {
        "el vídeo es privado"
    } else if d.contains("video unavailable") || d.contains("has been removed") {
        "el vídeo no está disponible"
    } else if d.contains("not a bot") || d.contains("sign in to confirm") {
        "YouTube pide confirmar que no eres un bot (inicia sesión en Ajustes → YouTube)"
    } else if d.contains("is not a valid url") || d.contains("unsupported url") {
        "no es una URL de YouTube válida"
    } else if d.contains("getaddrinfo") || d.contains("network") || d.contains("timed out") {
        "no hay conexión con YouTube"
    } else {
        return format!("No se pudo analizar la URL: {detalle}");
    };
    format!("No se pudo analizar la URL: {motivo}.")
}

pub fn sesion_rechazada(mensaje: &str) -> bool {
    let m = mensaje.to_lowercase();
    m.contains("cookies are no longer valid")
        || m.contains("not a bot")
        || m.contains("sign in to confirm")
}

/// Analiza la URL. Con sesion guardada la usa; si YouTube la rechaza, reintenta
/// sin ella (una URL publica sale igual) y lo marca.
pub fn analizar(u: &Ubicacion, url: &str, sesion: &SesionYoutube) -> Result<Analisis, String> {
    let url = normalizar(url);
    let cookies = sesion
        .cookies
        .as_deref()
        .map(Path::new)
        .filter(|p| p.is_file());
    match ejecutar(u, &url, cookies) {
        Ok(json) => Ok(Analisis {
            vista: desde_json(&json, &url),
            sesion_rechazada: false,
        }),
        Err(e) if cookies.is_some() && sesion_rechazada(&e) => match ejecutar(u, &url, None) {
            Ok(json) => Ok(Analisis {
                vista: desde_json(&json, &url),
                sesion_rechazada: true,
            }),
            Err(_) => Err(e),
        },
        Err(e) => Err(e),
    }
}

fn texto(v: &Value, clave: &str) -> String {
    v.get(clave)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn canal(v: &Value) -> String {
    [
        "channel",
        "uploader",
        "playlist_uploader",
        "playlist_channel",
        "artist",
    ]
    .iter()
    .map(|k| texto(v, k))
    .find(|s| !s.is_empty())
    .unwrap_or_default()
    .trim_end_matches(" - Topic")
    .to_string()
}

/// Quita "(Official Video)", "[HD]"... y separa "Artista - Titulo".
pub fn artista_y_titulo(titulo: &str, canal: &str) -> (String, String) {
    let limpio = limpiar_titulo(titulo);
    if let Some((a, t)) = limpio.split_once(" - ") {
        let (a, t) = (a.trim(), t.trim());
        if !a.is_empty() && !t.is_empty() {
            return (a.to_string(), t.to_string());
        }
    }
    (canal.to_string(), limpio)
}

const RUIDO_TITULO: [&str; 13] = [
    "official",
    "oficial",
    "video",
    "lyric",
    "letra",
    "audio",
    "hd",
    "4k",
    "remaster",
    "live",
    "visualizer",
    "mv",
    "公式",
];

fn limpiar_titulo(titulo: &str) -> String {
    let mut salida = String::new();
    let mut dentro = 0usize;
    let mut trozo = String::new();
    for c in titulo.chars() {
        match c {
            '(' | '[' | '【' => {
                dentro += 1;
                trozo.clear();
            }
            ')' | ']' | '】' if dentro > 0 => {
                dentro -= 1;
                let t = trozo.to_lowercase();
                if !RUIDO_TITULO.iter().any(|r| t.contains(r)) {
                    let abre = match c {
                        ')' => '(',
                        ']' => '[',
                        _ => '【',
                    };
                    salida.push(abre);
                    salida.push_str(&trozo);
                    salida.push(c);
                }
                trozo.clear();
            }
            _ if dentro > 0 => trozo.push(c),
            _ => salida.push(c),
        }
    }
    let minus = salida.to_lowercase();
    let corte = minus.find(" ft.").or_else(|| minus.find(" feat."));
    let sin_feat = match corte {
        Some(i) => salida[..i].to_string(),
        None => salida,
    };
    sin_feat
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(|c| c == '-' || c == '|' || c == ' ')
        .to_string()
}

fn miniatura(v: &Value) -> Option<String> {
    if let Some(t) = v.get("thumbnail").and_then(Value::as_str) {
        return Some(t.to_string());
    }
    v.get("thumbnails")?
        .as_array()?
        .last()?
        .get("url")?
        .as_str()
        .map(str::to_string)
}

fn problema(v: &Value) -> Option<String> {
    match v.get("availability").and_then(Value::as_str) {
        None | Some("public") | Some("unlisted") => None,
        Some("private") => Some("privado".into()),
        Some("needs_auth") | Some("subscriber_only") | Some("premium_only") => {
            Some("necesita_sesion".into())
        }
        Some(otro) => Some(otro.to_string()),
    }
}

fn video(v: &Value, url_por_defecto: &str) -> Option<VideoPrevio> {
    let id = texto(v, "id");
    if id.is_empty() {
        return None;
    }
    let url = ["webpage_url", "url"]
        .iter()
        .map(|k| texto(v, k))
        .find(|s| s.starts_with("http"))
        .unwrap_or_else(|| {
            if url_por_defecto.is_empty() {
                format!("https://www.youtube.com/watch?v={id}")
            } else {
                url_por_defecto.to_string()
            }
        });
    // Como en la fabrica: "Artista - Titulo" en el titulo manda sobre el canal,
    // que suele ser quien subio el video y no quien canta.
    let (artista, titulo) = artista_y_titulo(&texto(v, "title"), &canal(v));
    Some(VideoPrevio {
        id,
        url,
        titulo: if titulo.is_empty() {
            "(sin título)".into()
        } else {
            titulo
        },
        canal: artista,
        duracion: v.get("duration").and_then(Value::as_f64),
        miniatura: miniatura(v),
        problema: problema(v),
    })
}

pub fn desde_json(v: &Value, url: &str) -> VistaPrevia {
    let es_lista =
        v.get("_type").and_then(Value::as_str) == Some("playlist") || v.get("entries").is_some();
    if es_lista {
        let entradas = v
            .get("entries")
            .and_then(Value::as_array)
            .map(|lista| lista.iter().filter_map(|e| video(e, "")).collect())
            .unwrap_or_default();
        let titulo = texto(v, "title");
        return VistaPrevia::Lista {
            id: texto(v, "id"),
            titulo: if titulo.is_empty() {
                "Lista".into()
            } else {
                titulo
            },
            canal: canal(v),
            entradas,
        };
    }
    match video(v, url) {
        Some(vp) => VistaPrevia::Video(vp),
        None => VistaPrevia::Video(VideoPrevio {
            id: String::new(),
            url: url.to_string(),
            titulo: "(sin título)".into(),
            canal: String::new(),
            duracion: None,
            miniatura: None,
            problema: Some("no_disponible".into()),
        }),
    }
}

#[cfg(test)]
mod pruebas {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn el_runtime_se_nombra_como_lo_acepta_yt_dlp() {
        use std::path::Path;
        assert_eq!(
            nombre_runtime(Path::new(r"C:\app\deno-x86_64-pc-windows-msvc.exe")),
            "deno"
        );
        assert_eq!(nombre_runtime(Path::new("/opt/homebrew/bin/node")), "node");
        assert_eq!(nombre_runtime(Path::new("bun-aarch64-apple-darwin")), "bun");
    }

    #[test]
    fn el_error_de_yt_dlp_sale_en_espanol() {
        let m = explicar_error("[youtube] abc: Video unavailable");
        assert!(m.starts_with("No se pudo analizar la URL: el vídeo no está disponible"));
        let m =
            explicar_error("[youtube] abc: Private video. Sign in if you've been granted access");
        assert!(m.contains("privado"));
        let m = explicar_error("algo raro");
        assert_eq!(m, "No se pudo analizar la URL: algo raro");
    }

    #[test]
    fn un_video_con_list_apunta_al_video() {
        assert_eq!(
            normalizar("https://www.youtube.com/watch?v=abc123def45&list=PLxyz&index=3"),
            "https://www.youtube.com/watch?v=abc123def45"
        );
        // Un mix (list=RD...) si se conserva: es lo que el usuario pego.
        let mix = "https://youtu.be/abc123def45?list=RDabc";
        assert_eq!(normalizar(mix), mix);
    }

    #[test]
    fn una_lista_pura_se_queda_como_lista() {
        let u = "https://www.youtube.com/playlist?list=PLxyz";
        assert_eq!(normalizar(u), u);
    }

    #[test]
    fn lee_un_video() {
        let j = serde_json::json!({"id":"x1","title":"Tema","channel":"Artista - Topic","duration":200.5,
            "thumbnail":"https://i/x.jpg","webpage_url":"https://www.youtube.com/watch?v=x1","availability":"public"});
        match desde_json(&j, "u") {
            VistaPrevia::Video(v) => {
                assert_eq!(v.canal, "Artista");
                assert_eq!(v.duracion, Some(200.5));
                assert_eq!(v.problema, None);
            }
            _ => panic!("debia ser un video"),
        }
    }

    #[test]
    fn lee_una_lista_plana() {
        let j = serde_json::json!({"_type":"playlist","id":"PL1","title":"Mi lista","uploader":"Yo",
            "entries":[{"id":"a","title":"Uno","url":"https://www.youtube.com/watch?v=a","availability":"private"},{"id":"b","title":"Dos"}]});
        match desde_json(&j, "u") {
            VistaPrevia::Lista {
                entradas, titulo, ..
            } => {
                assert_eq!(titulo, "Mi lista");
                assert_eq!(entradas.len(), 2);
                assert_eq!(entradas[0].problema.as_deref(), Some("privado"));
                assert_eq!(entradas[1].url, "https://www.youtube.com/watch?v=b");
            }
            _ => panic!("debia ser una lista"),
        }
    }

    #[test]
    fn artista_y_titulo_desde_el_titulo() {
        assert_eq!(
            artista_y_titulo("Bipoxx - Piscanos (Letra)", "Andrw"),
            ("Bipoxx".into(), "Piscanos".into())
        );
        assert_eq!(
            artista_y_titulo("Light It Up [HD] (Official Video)", "The Product"),
            ("The Product".into(), "Light It Up".into())
        );
        assert_eq!(
            artista_y_titulo("Amor (Remix) ft. Alguien", "X"),
            ("X".into(), "Amor (Remix)".into())
        );
    }

    #[test]
    fn detecta_sesion_rechazada() {
        assert!(sesion_rechazada(
            "ERROR: [youtube] x: Sign in to confirm you're not a bot"
        ));
        assert!(!sesion_rechazada("Video unavailable"));
    }
}
