//! Hilo del juego. Un solo hilo por partida: crea el reproductor y el
//! microfono DENTRO del hilo (en macOS el stream de cpal no se puede mover de
//! hilo), y en cada vuelta (~16 ms) lee el microfono, detecta el tono, pasa
//! cada frame a tiempo de cancion (CONTRATO seccion 7), puntua y manda un
//! FrameJuego. El ritmo lo marca `recv_timeout` sobre el canal de control:
//! sin `thread::sleep`.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use audio::{GrabacionVoz, Microfono, Reloj, Reproductor};
use biblioteca::{Ajustes, Biblioteca, Dificultad};
use karaoke_core::Cancion;
use puntuacion::{Entrada, Puntuador};
use tono::DetectorTono;

use crate::grabar;
use crate::tipos::{EstadoJuego, FinPartida, FrameJuego, Modo, ResultadoJuego, TonoMicrofono};

const PERIODO: Duration = Duration::from_millis(16);

pub enum Control {
    Pausar,
    Reanudar,
    Buscar(f64),
    Detener,
    /// Volumen de la voz guia (0 = apagada), al vuelo.
    VozGuia(f64),
    /// Termina sin guardar resultado (partida abierta de mas o cancelada al arrancar).
    Descartar,
}

/// Partida en marcha: el canal de control y el hilo, que devuelve el resultado.
pub struct Partida {
    pub control: Sender<Control>,
    /// `None` si la partida se descarto.
    pub hilo: JoinHandle<Result<Option<FinPartida>, String>>,
}

pub struct Configuracion {
    pub id: String,
    pub carpeta: PathBuf,
    pub modo: Modo,
    pub desde: f64,
    pub ajustes: Ajustes,
    pub biblioteca: Biblioteca,
    /// Donde se guardan las grabaciones (Musica/Karaoke).
    pub carpeta_grabaciones: PathBuf,
}

/// Arranca el hilo. `enviar` recibe cada frame; si devuelve `false` (la
/// interfaz cerro el canal) la partida se detiene.
pub fn arrancar(
    config: Configuracion,
    enviar: impl FnMut(FrameJuego) -> bool + Send + 'static,
) -> Result<Partida, String> {
    let (tx, rx) = mpsc::channel();
    // El arranque (abrir audio, cargar wav) se hace dentro del hilo; su error
    // vuelve por este canal antes de dar la partida por iniciada.
    let (tx_listo, rx_listo) = mpsc::channel::<Result<(), String>>();
    let hilo = thread::Builder::new()
        .name("juego".into())
        .spawn(move || bucle(config, rx, tx_listo, enviar))
        .map_err(|e| format!("No se pudo arrancar el hilo del juego: {e}"))?;
    match rx_listo.recv() {
        Ok(Ok(())) => Ok(Partida { control: tx, hilo }),
        Ok(Err(e)) => {
            let _ = hilo.join();
            Err(e)
        }
        Err(_) => {
            let _ = hilo.join();
            Err("No se pudo arrancar el juego: el hilo termino sin responder".into())
        }
    }
}

struct Audio {
    reproductor: Reproductor,
    microfono: Microfono,
    reloj: Reloj,
    detector: DetectorTono,
    /// Indice de microfono de la primera muestra que recibio este detector:
    /// el detector cuenta desde 0 y el microfono desde que se abrio.
    base_detector: u64,
    /// Indice de microfono que el detector espera a continuacion. Si la
    /// siguiente lectura no empieza ahi (pausa, salto, muestras perdidas), el
    /// detector se rehace para no mezclar audio que no es contiguo.
    siguiente_detector: u64,
}

impl Audio {
    fn alimentar(
        &mut self,
        primera: u64,
        muestras: &[f32],
        mut salida: impl FnMut(u64, Option<f32>),
    ) {
        if primera != self.siguiente_detector {
            self.detector = DetectorTono::new(self.microfono.frecuencia());
            self.base_detector = primera;
        }
        self.siguiente_detector = primera + muestras.len() as u64;
        let base = self.base_detector;
        self.detector
            .empujar(muestras, |f| salida(base + f.muestra, f.midi));
    }
}

fn abrir_audio(config: &Configuracion) -> Result<Audio, String> {
    let mut reproductor = Reproductor::abrir().map_err(|e| e.to_string())?;
    let instrumental = config.carpeta.join("instrumental.wav");
    let voz = config.carpeta.join("voz.wav");
    reproductor
        .cargar(&instrumental, voz.exists().then_some(voz.as_path()))
        .map_err(|e| e.to_string())?;
    reproductor.fijar_volumenes(
        config.ajustes.volumen_musica as f32,
        config.ajustes.volumen_guia as f32,
    );
    let microfono =
        Microfono::abrir(config.ajustes.microfono.as_deref()).map_err(|e| e.to_string())?;
    let fs = microfono.frecuencia();
    Ok(Audio {
        reproductor,
        microfono,
        reloj: Reloj::new(fs),
        detector: DetectorTono::new(fs),
        base_detector: 0,
        siguiente_detector: 0,
    })
}

fn bucle(
    config: Configuracion,
    control: Receiver<Control>,
    listo: Sender<Result<(), String>>,
    mut enviar: impl FnMut(FrameJuego) -> bool,
) -> Result<Option<FinPartida>, String> {
    let preparado = Cancion::leer(&config.carpeta.join("cancion.json"))
        .map_err(|e| e.to_string())
        .and_then(|cancion| abrir_audio(&config).map(|audio| (cancion, audio)));
    let (cancion, mut audio) = match preparado {
        Ok(v) => {
            let _ = listo.send(Ok(()));
            v
        }
        Err(e) => {
            let _ = listo.send(Err(e.clone()));
            return Err(e);
        }
    };

    let notas = cancion.notas_planas();
    let dificultad: Dificultad = config.ajustes.dificultad;
    let mut puntuador =
        (config.modo == Modo::Puntuacion).then(|| Puntuador::new(&notas, dificultad));
    let latencia_ms = config.ajustes.latencia_ms;

    if config.desde > 0.0 {
        audio.reproductor.buscar(config.desde);
    }
    audio.reproductor.reproducir();
    let mut pausado = false;
    let mut volumen_guia = config.ajustes.volumen_guia;
    let mut descartada = false;
    let mut muestras = Vec::with_capacity(8192);
    let mut voz = config
        .ajustes
        .grabar
        .then(|| GrabacionVoz::new(audio.microfono.frecuencia()));

    loop {
        match control.recv_timeout(PERIODO) {
            Ok(Control::Pausar) => {
                audio.reproductor.pausar();
                audio.reloj.reiniciar();
                pausado = true;
                if let Some(v) = voz.as_mut() {
                    v.cortar();
                }
            }
            Ok(Control::VozGuia(v)) => {
                volumen_guia = v.clamp(0.0, 1.0);
                audio
                    .reproductor
                    .fijar_volumenes(config.ajustes.volumen_musica as f32, volumen_guia as f32);
            }
            Ok(Control::Reanudar) => {
                audio.reloj.reiniciar();
                audio.reproductor.reproducir();
                pausado = false;
            }
            Ok(Control::Buscar(t)) => {
                if config.modo == Modo::Practica {
                    audio.reproductor.buscar(t.max(0.0));
                    audio.reloj.reiniciar();
                    audio.siguiente_detector = u64::MAX; // fuerza un detector nuevo
                    if let Some(v) = voz.as_mut() {
                        v.cortar();
                    }
                }
            }
            Ok(Control::Detener) | Err(RecvTimeoutError::Disconnected) => break,
            Ok(Control::Descartar) => {
                descartada = true;
                break;
            }
            Err(RecvTimeoutError::Timeout) => {}
        }

        // Se lee siempre, tambien en pausa, para que el ring del microfono no se desborde.
        let primera = audio.microfono.leer(&mut muestras);
        audio.reloj.anotar(
            audio.reproductor.posicion(),
            audio.microfono.muestras_totales(),
        );

        let mut tonos = Vec::new();
        let mut eventos = Vec::new();
        if !pausado {
            if let Some(v) = voz.as_mut()
                && !muestras.is_empty()
            {
                v.anadir(
                    primera,
                    audio.reloj.tiempo_de_muestra(primera, latencia_ms),
                    &muestras,
                );
            }
            let mut frames = Vec::new();
            audio.alimentar(primera, &muestras, |k, midi| frames.push((k, midi)));
            for (k, midi) in frames {
                let t = audio.reloj.tiempo_de_muestra(k, latencia_ms);
                tonos.push(TonoMicrofono { t, midi });
                if let Some(p) = puntuador.as_mut() {
                    eventos.extend(p.frame(Entrada { t, midi }));
                }
            }
        }

        let terminado = audio.reproductor.termino();
        let (puntos, combo) = puntuador.as_ref().map_or((0.0, 0), |p| {
            let e = p.estado();
            (e.puntos, e.combo)
        });
        let estado = if terminado {
            EstadoJuego::Terminado
        } else if pausado {
            EstadoJuego::Pausado
        } else {
            EstadoJuego::Sonando
        };
        let frame = FrameJuego {
            t: audio.reproductor.posicion_oida(),
            tonos,
            puntos,
            combo,
            eventos,
            estado,
        };
        if !enviar(frame) || terminado {
            break;
        }
    }

    audio.reproductor.pausar();
    if descartada {
        return Ok(None);
    }
    let resultado = match puntuador {
        Some(p) => {
            let r = p.terminar();
            let (record_anterior, es_record) = config
                .biblioteca
                .guardar_resultado(&config.id, dificultad, r.puntos)
                // Un fallo al guardar el record no debe perder la grabacion.
                .unwrap_or_else(|e| {
                    eprintln!("{e}");
                    (None, false)
                });
            Some(ResultadoJuego {
                puntos: r.puntos,
                nivel: r.nivel,
                mejor_combo: r.mejor_combo,
                precision_media: r.precision_media,
                por_calificacion: r.por_calificacion,
                record_anterior,
                es_record,
            })
        }
        None => None,
    };
    // El audio se suelta antes de mezclar: la mezcla lee los wav otra vez.
    drop(audio);
    let (grabacion, error_grabacion) = match voz {
        Some(v) => {
            let datos = grabar::Datos {
                carpeta_cancion: &config.carpeta,
                carpeta_grabaciones: &config.carpeta_grabaciones,
                id: &config.id,
                titulo: &cancion.titulo,
                artista: &cancion.artista,
                volumen_musica: config.ajustes.volumen_musica,
                volumen_guia,
                volumen_voz: config.ajustes.volumen_voz,
            };
            match grabar::guardar(&datos, &v) {
                Ok((g, aviso)) => (Some(g), aviso),
                Err(e) => (None, Some(e)),
            }
        }
        None => (None, None),
    };
    Ok(Some(FinPartida {
        resultado,
        grabacion,
        error_grabacion,
    }))
}
