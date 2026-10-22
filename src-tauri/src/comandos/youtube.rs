//! Comandos de la sesion de YouTube (CONTRATO seccion 5). La logica pura esta
//! en `crate::youtube`; aqui solo las ventanas de inicio de sesion.

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::ipc::Channel;
use tauri::webview::{PageLoadEvent, WebviewWindow, WebviewWindowBuilder};
use tauri::{AppHandle, Manager, State, Url, WebviewUrl, WindowEvent};

use crate::estado::Estado;
use crate::youtube::{self, CookieLeida, CuentaYoutube, EstadoSesion};

pub const VENTANA_LOGIN: &str = "youtube-login";
pub const VENTANA_SILENCIOSA: &str = "youtube-login-silencioso";
const ESPERA_SILENCIOSA: Duration = Duration::from_secs(20);

/// `EventoSesion` de CONTRATO seccion 6.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum EventoSesion {
    /// Cookies guardadas: hay sesion.
    Conectada,
    /// El usuario cerro la ventana (con o sin sesion).
    Cerrada,
    Error {
        mensaje: String,
    },
}

fn datos(app: &AppHandle) -> PathBuf {
    app.state::<Estado>().biblioteca.datos().to_path_buf()
}

/// Lee las cookies de la ventana y, si hay sesion, las guarda. `Ok(true)` =
/// guardadas; `Ok(false)` = todavia sin sesion (paso intermedio de Google o
/// pagina anonima): no se toca el archivo, para no pisar una sesion valida.
fn guardar_cookies(ventana: &WebviewWindow, datos: &std::path::Path) -> Result<bool, String> {
    let mut leidas = Vec::new();
    let mut fallos = 0;
    for crudo in youtube::DOMINIOS_COOKIES {
        let Ok(url) = crudo.parse::<Url>() else {
            continue;
        };
        match ventana.cookies_for_url(url) {
            Ok(cookies) => leidas.extend(cookies.iter().map(|c| CookieLeida {
                dominio: c.domain().unwrap_or(".youtube.com").to_string(),
                ruta: c.path().unwrap_or("/").to_string(),
                nombre: c.name().to_string(),
                valor: c.value().to_string(),
                segura: c.secure().unwrap_or(false),
                solo_http: c.http_only().unwrap_or(false),
                caduca: match c.expires() {
                    Some(cookie::Expiration::DateTime(f)) => f.unix_timestamp(),
                    _ => 0,
                },
            })),
            Err(_) => fallos += 1,
        }
    }
    if fallos == youtube::DOMINIOS_COOKIES.len() {
        return Err("No se pudo leer la sesion: la ventana no devolvio cookies".into());
    }
    let (texto, _) = youtube::a_netscape(&leidas);
    if !youtube::tiene_sesion(&texto) {
        return Ok(false);
    }
    youtube::guardar(datos, &texto)?;
    Ok(true)
}

fn url_login() -> Result<Url, String> {
    youtube::URL_INICIO_SESION
        .parse()
        .map_err(|e| format!("No se pudo abrir el inicio de sesion: {e}"))
}

#[tauri::command(rename_all = "snake_case")]
pub fn estado_sesion_youtube(estado: State<'_, Estado>) -> EstadoSesion {
    youtube::estado(estado.biblioteca.datos())
}

/// Bloquea hasta 10 s (red): en un hilo de trabajo.
#[tauri::command(rename_all = "snake_case")]
pub async fn cuenta_youtube(app: AppHandle) -> Result<Option<CuentaYoutube>, String> {
    let d = datos(&app);
    tauri::async_runtime::spawn_blocking(move || youtube::cuenta(&d))
        .await
        .map_err(|e| format!("No se pudo consultar la cuenta de YouTube: {e}"))?
}

#[tauri::command(rename_all = "snake_case")]
pub fn cerrar_sesion_youtube(estado: State<'_, Estado>) -> Result<(), String> {
    youtube::cerrar(estado.biblioteca.datos())
}

/// Abre la ventana de inicio de sesion de Google. Cuando el usuario llega a
/// youtube.com con sesion, se guardan las cookies y la ventana se cierra sola.
#[tauri::command(rename_all = "snake_case")]
pub fn iniciar_sesion_youtube(app: AppHandle, canal: Channel<EventoSesion>) -> Result<(), String> {
    if let Some(abierta) = app.get_webview_window(VENTANA_LOGIN) {
        let _ = abierta.set_focus();
        return Ok(());
    }
    let d = datos(&app);
    let canal_carga = canal.clone();
    let ventana =
        WebviewWindowBuilder::new(&app, VENTANA_LOGIN, WebviewUrl::External(url_login()?))
            .title("YouTube: iniciar sesión")
            .inner_size(1000.0, 720.0)
            .center()
            .user_agent(youtube::AGENTE_NAVEGADOR)
            .on_page_load(move |ventana, carga| {
                let url = carga.url().to_string();
                if carga.event() != PageLoadEvent::Finished
                    || !url.contains("youtube.com")
                    || url.contains("accounts.google.com")
                {
                    return;
                }
                let (ventana, d, canal) = (ventana.clone(), d.clone(), canal_carga.clone());
                tauri::async_runtime::spawn(async move {
                    match guardar_cookies(&ventana, &d) {
                        Ok(true) => {
                            let _ = canal.send(EventoSesion::Conectada);
                            let _ = ventana.close();
                        }
                        Ok(false) => {} // aun sin sesion: la ventana sigue abierta
                        Err(mensaje) => {
                            let _ = canal.send(EventoSesion::Error { mensaje });
                        }
                    }
                });
            })
            .build()
            .map_err(|e| format!("No se pudo abrir la ventana de inicio de sesion: {e}"))?;
    ventana.on_window_event(move |evento| {
        if let WindowEvent::Destroyed = evento {
            let _ = canal.send(EventoSesion::Cerrada);
        }
    });
    Ok(())
}

/// Renueva la sesion SIN que el usuario haga nada: una ventana oculta abre el
/// inicio de sesion pasivo; si Google aun tiene sesion, redirige a youtube.com
/// y se guardan cookies nuevas. `true` solo si se guardo una sesion valida;
/// `false` tras ~20 s sin llegar (Google pide interaccion).
#[tauri::command(rename_all = "snake_case")]
pub async fn renovar_sesion_youtube(app: AppHandle) -> Result<bool, String> {
    if let Some(vieja) = app.get_webview_window(VENTANA_SILENCIOSA) {
        let _ = vieja.close();
    }
    let d = datos(&app);
    let (tx, rx) = mpsc::channel::<bool>();
    let tx = Arc::new(Mutex::new(Some(tx)));
    let tx_carga = Arc::clone(&tx);
    WebviewWindowBuilder::new(&app, VENTANA_SILENCIOSA, WebviewUrl::External(url_login()?))
        .title("YouTube: renovar sesión")
        .visible(false)
        .user_agent(youtube::AGENTE_NAVEGADOR)
        .on_page_load(move |ventana, carga| {
            let url = carga.url().to_string();
            if carga.event() != PageLoadEvent::Finished
                || !url.contains("youtube.com")
                || url.contains("accounts.google.com")
            {
                return;
            }
            let (ventana, d, tx) = (ventana.clone(), d.clone(), Arc::clone(&tx_carga));
            tauri::async_runtime::spawn(async move {
                let resultado = match guardar_cookies(&ventana, &d) {
                    Ok(true) => true,
                    // Solo la pagina final decide: los saltos de Google pueden seguir.
                    Ok(false)
                        if url.starts_with("https://www.youtube.com/")
                            && !url.contains("/signin") =>
                    {
                        false
                    }
                    Ok(false) => return,
                    Err(_) => false,
                };
                if let Ok(mut guarda) = tx.lock()
                    && let Some(enviar) = guarda.take()
                {
                    let _ = enviar.send(resultado);
                }
            });
        })
        .build()
        .map_err(|e| format!("No se pudo renovar la sesion: {e}"))?;
    let ok = tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(ESPERA_SILENCIOSA).unwrap_or(false)
    })
    .await
    .unwrap_or(false);
    if let Ok(mut guarda) = tx.lock() {
        guarda.take(); // una llegada tardia ya no hace nada
    }
    if let Some(v) = app.get_webview_window(VENTANA_SILENCIOSA) {
        let _ = v.close();
    }
    Ok(ok)
}

/// Al cerrar la app: que no quede ninguna ventana de inicio de sesion viva.
pub fn cerrar_ventanas(app: &AppHandle) {
    for etiqueta in [VENTANA_LOGIN, VENTANA_SILENCIOSA] {
        if let Some(v) = app.get_webview_window(etiqueta) {
            let _ = v.close();
        }
    }
}
