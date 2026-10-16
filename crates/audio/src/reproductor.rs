//! Reproduccion con reloj por cuadros entregados (CONTRATO seccion 7).
//!
//! Hilos:
//! - El callback de `cpal` es el dueno de [`Mezclador`]. Solo lee atomicos,
//!   saca pistas de un ring `rtrb` y escribe en el bufer del dispositivo. No
//!   reserva memoria ni libera: el [`Reproductor`] guarda una copia de cada
//!   `Arc<Pistas>` hasta que el callback confirma que la soltó, asi el
//!   `Arc` que suelta el callback nunca es el ultimo.
//! - El hilo que posee el [`Reproductor`] decodifica, convierte y manda
//!   ordenes por atomicos.
//!
//! Modos del reloj:
//! - **Libre** (no hay cancion cargada): el reloj avanza siempre que el stream
//!   esta abierto. Sirve para calibrar: los pitidos tienen tiempo de reloj.
//! - **Cancion**: el reloj avanza solo mientras suena y se detiene al final.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, OutputCallbackInfo, Sample, SizedSample};

use crate::ErrorAudio;
use crate::wav::{adaptar, leer_wav};

/// Amplitud de los pitidos (pico).
const AMPLITUD_PITIDO: f32 = 0.5;
/// Rampa de entrada y salida de los pitidos, para que no chasqueen.
const RAMPA_PITIDO_S: f64 = 0.001;
/// Espera maxima a que el callback atienda una orden.
const ESPERA_MAXIMA: Duration = Duration::from_secs(1);
/// Ordenes de carga en vuelo como maximo.
const CAPACIDAD_ENVIOS: usize = 4;

/// Audio de una cancion ya convertido al formato del dispositivo
/// (intercalado, `canales` del dispositivo, su frecuencia). Inmutable.
#[derive(Debug)]
pub(crate) struct Pistas {
    pub(crate) musica: Box<[f32]>,
    pub(crate) guia: Option<Box<[f32]>>,
    pub(crate) cuadros: u64,
}

/// Estado compartido entre el callback y el [`Reproductor`]. Solo atomicos.
#[derive(Debug)]
pub(crate) struct Compartido {
    /// Reloj: cuadros entregados al dispositivo (de la cancion o, en modo
    /// libre, desde que se abrio o se descargo la cancion).
    pub(crate) cuadros: AtomicU64,
    pub(crate) sonando: AtomicBool,
    /// `f32` en bits.
    pub(crate) vol_musica: AtomicU32,
    pub(crate) vol_guia: AtomicU32,
    /// Latencia de salida estimada en segundos (`f64` en bits). NaN = sin dato.
    pub(crate) latencia_salida: AtomicU64,
    /// Ordenes de carga atendidas por el callback.
    pub(crate) adopciones: AtomicU64,
    /// Pitido: el control escribe frecuencia y duracion y sube `peticion`;
    /// el callback lo empieza al inicio de su siguiente bloque, escribe
    /// `inicio` (cuadro de reloj de su primera muestra) y sube `atendido`.
    pub(crate) pitido_peticion: AtomicU64,
    pub(crate) pitido_atendido: AtomicU64,
    pub(crate) pitido_frecuencia: AtomicU32,
    pub(crate) pitido_cuadros: AtomicU64,
    pub(crate) pitido_inicio: AtomicU64,
    /// El dispositivo de salida se perdio (desconectado).
    pub(crate) perdido: AtomicBool,
}

impl Compartido {
    pub(crate) fn new() -> Compartido {
        Compartido {
            cuadros: AtomicU64::new(0),
            sonando: AtomicBool::new(false),
            vol_musica: AtomicU32::new(1.0f32.to_bits()),
            vol_guia: AtomicU32::new(0.0f32.to_bits()),
            latencia_salida: AtomicU64::new(f64::NAN.to_bits()),
            adopciones: AtomicU64::new(0),
            pitido_peticion: AtomicU64::new(0),
            pitido_atendido: AtomicU64::new(0),
            pitido_frecuencia: AtomicU32::new(0),
            pitido_cuadros: AtomicU64::new(0),
            pitido_inicio: AtomicU64::new(0),
            perdido: AtomicBool::new(false),
        }
    }
}

fn leer_f32(a: &AtomicU32) -> f32 {
    f32::from_bits(a.load(Ordering::Relaxed))
}

/// Volumen valido: `[0, 1]`; NaN = 0.
fn volumen(v: f32) -> f32 {
    if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) }
}

/// Lo que vive dentro del callback de salida.
pub(crate) struct Mezclador {
    compartido: Arc<Compartido>,
    pista: Option<Arc<Pistas>>,
    entrantes: rtrb::Consumer<Option<Arc<Pistas>>>,
    canales: usize,
    frecuencia: f64,
    pitido_visto: u64,
    pitido_total: u64,
    pitido_transcurridos: u64,
    /// Radianes por cuadro.
    pitido_paso: f64,
}

impl Mezclador {
    pub(crate) fn new(
        compartido: Arc<Compartido>,
        entrantes: rtrb::Consumer<Option<Arc<Pistas>>>,
        canales: usize,
        frecuencia: u32,
    ) -> Mezclador {
        Mezclador {
            compartido,
            pista: None,
            entrantes,
            canales: canales.max(1),
            frecuencia: f64::from(frecuencia.max(1)),
            pitido_visto: 0,
            pitido_total: 0,
            pitido_transcurridos: 0,
            pitido_paso: 0.0,
        }
    }

    /// Cuerpo del callback. Sin reservas, sin locks, sin E/S.
    pub(crate) fn llenar<T: Sample + FromSample<f32>>(
        &mut self,
        datos: &mut [T],
        latencia_dispositivo: Option<Duration>,
    ) {
        self.adoptar_pistas();
        let c = Arc::clone(&self.compartido);
        let p = c.cuadros.load(Ordering::Acquire);
        self.atender_pitido(p);

        let canales = self.canales;
        let n = (datos.len() / canales) as u64;
        let sonando = c.sonando.load(Ordering::Acquire);
        let vm = leer_f32(&c.vol_musica);
        let vg = leer_f32(&c.vol_guia);
        let pista = self.pista.clone();
        let activa = pista.as_deref().filter(|_| sonando);

        let mut cuadros = datos.chunks_exact_mut(canales);
        for (i, cuadro) in (&mut cuadros).enumerate() {
            let pitido = self.siguiente_pitido();
            let base = (p as usize).saturating_add(i).saturating_mul(canales);
            for (ch, salida) in cuadro.iter_mut().enumerate() {
                let mut v = pitido;
                if let Some(pz) = activa {
                    v += pz.musica.get(base + ch).copied().unwrap_or(0.0) * vm;
                    if let Some(g) = pz.guia.as_deref() {
                        v += g.get(base + ch).copied().unwrap_or(0.0) * vg;
                    }
                }
                *salida = T::from_sample(v.clamp(-1.0, 1.0));
            }
        }
        for resto in cuadros.into_remainder() {
            *resto = T::EQUILIBRIUM;
        }
        // Soltar aqui la copia local nunca libera: el Reproductor guarda otra.
        drop(pista);

        let nuevo = match self.pista.as_deref() {
            None => p.saturating_add(n),
            Some(pz) if sonando && p < pz.cuadros => p.saturating_add(n).min(pz.cuadros),
            Some(_) => p,
        };
        if nuevo != p {
            // Si falla, buscar() cambio el reloj durante este bloque: gana buscar.
            let _ = c
                .cuadros
                .compare_exchange(p, nuevo, Ordering::AcqRel, Ordering::Relaxed);
        }

        if let Some(d) = latencia_dispositivo {
            // Media de lo que tarda en oirse un cuadro de este bloque.
            let latencia = d.as_secs_f64() + n as f64 / (2.0 * self.frecuencia);
            c.latencia_salida
                .store(latencia.to_bits(), Ordering::Relaxed);
        }
    }

    fn adoptar_pistas(&mut self) {
        while let Ok(nueva) = self.entrantes.pop() {
            // El Arc anterior se suelta aqui; el Reproductor conserva otra copia.
            self.pista = nueva;
            self.compartido.cuadros.store(0, Ordering::Release);
            self.compartido.adopciones.fetch_add(1, Ordering::Release);
        }
    }

    fn atender_pitido(&mut self, p: u64) {
        let c = &self.compartido;
        let peticion = c.pitido_peticion.load(Ordering::Acquire);
        if peticion == self.pitido_visto {
            return;
        }
        self.pitido_visto = peticion;
        let frecuencia = f64::from(leer_f32(&c.pitido_frecuencia));
        self.pitido_total = c.pitido_cuadros.load(Ordering::Relaxed);
        self.pitido_transcurridos = 0;
        self.pitido_paso = 2.0 * std::f64::consts::PI * frecuencia / self.frecuencia;
        c.pitido_inicio.store(p, Ordering::Relaxed);
        c.pitido_atendido.store(peticion, Ordering::Release);
    }

    fn siguiente_pitido(&mut self) -> f32 {
        if self.pitido_transcurridos >= self.pitido_total {
            return 0.0;
        }
        let k = self.pitido_transcurridos;
        self.pitido_transcurridos += 1;
        let rampa = (RAMPA_PITIDO_S * self.frecuencia).max(1.0);
        let restantes = (self.pitido_total - 1 - k) as f64;
        let envolvente = (k as f64 / rampa).min(restantes / rampa).min(1.0);
        ((k as f64 * self.pitido_paso).sin() * envolvente) as f32 * AMPLITUD_PITIDO
    }
}

/// Espera (en el hilo del llamador, no en el de audio) a que se cumpla una
/// condicion que pone el callback. `false` si no llega en [`ESPERA_MAXIMA`].
fn esperar(condicion: impl Fn() -> bool) -> bool {
    let limite = Instant::now() + ESPERA_MAXIMA;
    loop {
        if condicion() {
            return true;
        }
        if Instant::now() >= limite {
            return false;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Salida de audio por el dispositivo por defecto con reloj por cuadros.
///
/// Todas las consultas son atomicas y baratas. `cargar` y `descargar`
/// bloquean hasta que el callback recoge la orden (un bloque, ~10 ms).
pub struct Reproductor {
    _stream: cpal::Stream,
    compartido: Arc<Compartido>,
    envios: rtrb::Producer<Option<Arc<Pistas>>>,
    enviadas: u64,
    /// Copia de la pista que tiene el callback: el callback nunca libera.
    pista: Option<Arc<Pistas>>,
    /// Pistas cuya retirada no confirmo el callback (no se liberan aun).
    retenidas: Vec<Arc<Pistas>>,
    frecuencia: u32,
    canales: usize,
}

impl Reproductor {
    /// Abre la salida por defecto con su configuracion por defecto (nunca un
    /// tamano de bufer fijo: falla en WASAPI). Empieza en modo reloj libre.
    pub fn abrir() -> Result<Reproductor, ErrorAudio> {
        let host = cpal::default_host();
        let dispositivo = host.default_output_device().ok_or(ErrorAudio::SinSalida)?;
        let soportada = dispositivo
            .default_output_config()
            .map_err(|e| ErrorAudio::Salida(e.to_string()))?;
        let config = soportada.config();
        let canales = usize::from(config.channels).max(1);
        let frecuencia = config.sample_rate;
        let compartido = Arc::new(Compartido::new());
        let (envios, entrantes) = rtrb::RingBuffer::new(CAPACIDAD_ENVIOS);
        let mezclador = Mezclador::new(Arc::clone(&compartido), entrantes, canales, frecuencia);

        use cpal::SampleFormat as F;
        let stream = match soportada.sample_format() {
            F::F32 => construir::<f32>(&dispositivo, &config, mezclador, &compartido),
            F::F64 => construir::<f64>(&dispositivo, &config, mezclador, &compartido),
            F::I16 => construir::<i16>(&dispositivo, &config, mezclador, &compartido),
            F::I32 => construir::<i32>(&dispositivo, &config, mezclador, &compartido),
            F::U16 => construir::<u16>(&dispositivo, &config, mezclador, &compartido),
            otro => Err(ErrorAudio::Salida(format!(
                "formato de muestra no soportado ({otro:?})"
            ))),
        }?;
        stream
            .play()
            .map_err(|e| ErrorAudio::Salida(e.to_string()))?;
        Ok(Reproductor {
            _stream: stream,
            compartido,
            envios,
            enviadas: 0,
            pista: None,
            retenidas: Vec::new(),
            frecuencia,
            canales,
        })
    }

    /// Frecuencia del dispositivo de salida (Hz).
    pub fn frecuencia(&self) -> u32 {
        self.frecuencia
    }

    /// Canales del dispositivo de salida.
    pub fn canales(&self) -> usize {
        self.canales
    }

    /// Lee el instrumental y, si hay, la voz guia; los convierte al formato del
    /// dispositivo y los deja listos en pausa, con el reloj en 0. Tarda
    /// (decodificar y remuestrear una cancion entera): no lo llames desde un
    /// hilo que no pueda esperar.
    pub fn cargar(&mut self, musica: &Path, guia: Option<&Path>) -> Result<(), ErrorAudio> {
        let musica = adaptar(leer_wav(musica)?, self.canales, self.frecuencia)?;
        let guia = match guia {
            Some(ruta) => Some(adaptar(leer_wav(ruta)?, self.canales, self.frecuencia)?),
            None => None,
        };
        let pistas = Pistas {
            cuadros: (musica.len() / self.canales) as u64,
            musica: musica.into_boxed_slice(),
            guia: guia.map(Vec::into_boxed_slice),
        };
        self.enviar(Some(Arc::new(pistas)))
    }

    /// Quita la cancion y vuelve al modo reloj libre (reloj en 0).
    pub fn descargar(&mut self) -> Result<(), ErrorAudio> {
        self.enviar(None)
    }

    fn enviar(&mut self, pista: Option<Arc<Pistas>>) -> Result<(), ErrorAudio> {
        self.compartido.sonando.store(false, Ordering::Release);
        if self.envios.push(pista.clone()).is_err() {
            return Err(ErrorAudio::SinRespuesta);
        }
        self.enviadas += 1;
        let anterior = std::mem::replace(&mut self.pista, pista);
        let enviadas = self.enviadas;
        let compartido = &self.compartido;
        if esperar(|| compartido.adopciones.load(Ordering::Acquire) >= enviadas) {
            // El callback ya solo tiene la pista nueva: liberar aqui es seguro.
            self.retenidas.clear();
            drop(anterior);
            Ok(())
        } else {
            self.retenidas.extend(anterior);
            Err(ErrorAudio::SinRespuesta)
        }
    }

    /// Hay una cancion cargada (si no, el reloj esta en modo libre).
    pub fn cargada(&self) -> bool {
        self.pista.is_some()
    }

    /// Empieza o sigue la cancion. Sin cancion no hace nada.
    pub fn reproducir(&self) {
        if self.pista.is_some() {
            self.compartido.sonando.store(true, Ordering::Release);
        }
    }

    /// Pausa la cancion: el reloj se detiene.
    pub fn pausar(&self) {
        self.compartido.sonando.store(false, Ordering::Release);
    }

    /// Mueve el reloj a `t` segundos (limitado a `[0, duracion]` con cancion).
    pub fn buscar(&self, t: f64) {
        if !t.is_finite() {
            return;
        }
        let maximo = self.pista.as_ref().map_or(u64::MAX, |p| p.cuadros);
        let cuadro = (t.max(0.0) * f64::from(self.frecuencia)).round() as u64;
        self.compartido
            .cuadros
            .store(cuadro.min(maximo), Ordering::Release);
    }

    /// Reloj de la cancion en segundos: cuadros ya entregados por el callback
    /// entre la frecuencia del dispositivo (CONTRATO seccion 7).
    pub fn posicion(&self) -> f64 {
        self.compartido.cuadros.load(Ordering::Acquire) as f64 / f64::from(self.frecuencia)
    }

    /// Tiempo que se oye ahora: [`Reproductor::posicion`] menos la latencia
    /// de salida estimada con las marcas de `cpal` (si el host no las da,
    /// igual a `posicion`). Nunca negativo.
    pub fn posicion_oida(&self) -> f64 {
        let posicion = self.posicion();
        match self.latencia_salida() {
            Some(l) => (posicion - l).max(0.0),
            None => posicion,
        }
    }

    /// Latencia de salida estimada en segundos (`None` sin marcas de tiempo).
    pub fn latencia_salida(&self) -> Option<f64> {
        let l = f64::from_bits(self.compartido.latencia_salida.load(Ordering::Relaxed));
        (l.is_finite() && l >= 0.0).then_some(l)
    }

    /// Duracion de la cancion cargada en segundos (0 sin cancion).
    pub fn duracion(&self) -> f64 {
        self.pista
            .as_ref()
            .map_or(0.0, |p| p.cuadros as f64 / f64::from(self.frecuencia))
    }

    /// La cancion llego al final (falso sin cancion).
    pub fn termino(&self) -> bool {
        self.pista
            .as_ref()
            .is_some_and(|p| self.compartido.cuadros.load(Ordering::Acquire) >= p.cuadros)
    }

    /// Volumenes del instrumental y de la voz guia, en `[0, 1]`.
    pub fn fijar_volumenes(&self, musica: f32, guia: f32) {
        let c = &self.compartido;
        c.vol_musica
            .store(volumen(musica).to_bits(), Ordering::Relaxed);
        c.vol_guia.store(volumen(guia).to_bits(), Ordering::Relaxed);
    }

    /// Suena un pitido sinusoidal desde el inicio del siguiente bloque del
    /// callback y devuelve el tiempo de reloj (unidades de
    /// [`Reproductor::posicion`]) de su primera muestra. Bloquea hasta que el
    /// callback lo empieza (~un bloque). Se mezcla con la cancion si suena;
    /// para calibrar, usalo en modo reloj libre (sin cancion), donde el reloj
    /// avanza siempre.
    pub fn sonar_pitido(&self, frecuencia_hz: f64, duracion_s: f64) -> Result<f64, ErrorAudio> {
        let c = &self.compartido;
        let fs = f64::from(self.frecuencia);
        let cuadros = (duracion_s.max(0.0) * fs).round() as u64;
        c.pitido_frecuencia
            .store((frecuencia_hz as f32).to_bits(), Ordering::Relaxed);
        c.pitido_cuadros.store(cuadros, Ordering::Relaxed);
        let peticion = c.pitido_peticion.fetch_add(1, Ordering::AcqRel) + 1;
        if !esperar(|| c.pitido_atendido.load(Ordering::Acquire) >= peticion) {
            return Err(ErrorAudio::SinRespuesta);
        }
        Ok(c.pitido_inicio.load(Ordering::Acquire) as f64 / fs)
    }

    /// El dispositivo de salida se desconecto o el stream fallo.
    pub fn dispositivo_perdido(&self) -> bool {
        self.compartido.perdido.load(Ordering::Acquire)
    }
}

fn construir<T: SizedSample + FromSample<f32>>(
    dispositivo: &cpal::Device,
    config: &cpal::StreamConfig,
    mut mezclador: Mezclador,
    compartido: &Arc<Compartido>,
) -> Result<cpal::Stream, ErrorAudio> {
    let para_error = Arc::clone(compartido);
    dispositivo
        .build_output_stream(
            config,
            move |datos: &mut [T], info: &OutputCallbackInfo| {
                let marca = info.timestamp();
                mezclador.llenar(datos, marca.playback.duration_since(&marca.callback));
            },
            move |error| {
                if matches!(
                    error,
                    cpal::StreamError::DeviceNotAvailable | cpal::StreamError::StreamInvalidated
                ) {
                    para_error.perdido.store(true, Ordering::Release);
                }
            },
            None,
        )
        .map_err(|e| ErrorAudio::Salida(e.to_string()))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    const FS: u32 = 1000;

    fn mezclador(
        canales: usize,
    ) -> (
        Mezclador,
        Arc<Compartido>,
        rtrb::Producer<Option<Arc<Pistas>>>,
    ) {
        let compartido = Arc::new(Compartido::new());
        let (envios, entrantes) = rtrb::RingBuffer::new(CAPACIDAD_ENVIOS);
        let m = Mezclador::new(Arc::clone(&compartido), entrantes, canales, FS);
        (m, compartido, envios)
    }

    fn pistas(cuadros: usize, valor: f32, guia: Option<f32>) -> Arc<Pistas> {
        Arc::new(Pistas {
            musica: vec![valor; cuadros * 2].into_boxed_slice(),
            guia: guia.map(|g| vec![g; cuadros * 2].into_boxed_slice()),
            cuadros: cuadros as u64,
        })
    }

    #[test]
    fn reloj_libre_avanza_sin_cancion() {
        let (mut m, c, _envios) = mezclador(2);
        let mut bloque = vec![0.0f32; 20];
        m.llenar(&mut bloque, None);
        m.llenar(&mut bloque, None);
        assert_eq!(c.cuadros.load(Ordering::Acquire), 20);
        assert!(bloque.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn cancion_mezcla_con_volumenes_y_para_al_final() {
        let (mut m, c, mut envios) = mezclador(2);
        envios.push(Some(pistas(15, 0.5, Some(0.25)))).unwrap();
        let mut bloque = vec![0.0f32; 20];
        m.llenar(&mut bloque, None);
        // Adoptada en pausa: el reloj vuelve a 0 y no avanza.
        assert_eq!(c.adopciones.load(Ordering::Acquire), 1);
        assert_eq!(c.cuadros.load(Ordering::Acquire), 0);
        assert!(bloque.iter().all(|&v| v == 0.0));

        c.sonando.store(true, Ordering::Release);
        c.vol_musica.store(0.5f32.to_bits(), Ordering::Relaxed);
        c.vol_guia.store(1.0f32.to_bits(), Ordering::Relaxed);
        m.llenar(&mut bloque, None);
        assert_eq!(c.cuadros.load(Ordering::Acquire), 10);
        assert!(bloque.iter().all(|&v| (v - 0.5).abs() < 1e-6));

        m.llenar(&mut bloque, None);
        assert_eq!(c.cuadros.load(Ordering::Acquire), 15);
        // Los 5 cuadros que sobrepasan el final son silencio.
        assert!(bloque[..10].iter().all(|&v| (v - 0.5).abs() < 1e-6));
        assert!(bloque[10..].iter().all(|&v| v == 0.0));
        m.llenar(&mut bloque, None);
        assert_eq!(c.cuadros.load(Ordering::Acquire), 15);
    }

    #[test]
    fn buscar_durante_un_bloque_gana() {
        let (mut m, c, mut envios) = mezclador(2);
        envios.push(Some(pistas(100, 0.1, None))).unwrap();
        let mut bloque = vec![0.0f32; 20];
        m.llenar(&mut bloque, None);
        c.sonando.store(true, Ordering::Release);
        c.cuadros.store(50, Ordering::Release);
        m.llenar(&mut bloque, None);
        assert_eq!(c.cuadros.load(Ordering::Acquire), 60);
    }

    #[test]
    fn pitido_empieza_al_inicio_del_bloque_y_da_su_cuadro() {
        let (mut m, c, _envios) = mezclador(1);
        let mut bloque = vec![0.0f32; 10];
        m.llenar(&mut bloque, None);
        m.llenar(&mut bloque, None);
        c.pitido_frecuencia
            .store(250.0f32.to_bits(), Ordering::Relaxed);
        c.pitido_cuadros.store(15, Ordering::Relaxed);
        c.pitido_peticion.fetch_add(1, Ordering::AcqRel);
        m.llenar(&mut bloque, None);
        assert_eq!(c.pitido_atendido.load(Ordering::Acquire), 1);
        assert_eq!(c.pitido_inicio.load(Ordering::Acquire), 20);
        // 250 Hz a 1 kHz: seno en 0, 1, 0, -1...; la rampa de 1 cuadro
        // anula solo la primera muestra.
        assert_eq!(bloque[0], 0.0);
        assert!((bloque[1] - AMPLITUD_PITIDO).abs() < 1e-6, "{}", bloque[1]);
        m.llenar(&mut bloque, None);
        assert!(bloque[5..].iter().all(|&v| v == 0.0), "{bloque:?}");
    }

    #[test]
    fn latencia_de_salida_suma_medio_bloque() {
        let (mut m, c, _envios) = mezclador(1);
        let mut bloque = vec![0.0f32; 10];
        m.llenar(&mut bloque, Some(Duration::from_millis(20)));
        let l = f64::from_bits(c.latencia_salida.load(Ordering::Relaxed));
        assert!((l - 0.025).abs() < 1e-9, "{l}");
    }

    #[test]
    fn formatos_enteros_reciben_la_mezcla() {
        let (mut m, c, mut envios) = mezclador(2);
        envios.push(Some(pistas(10, 1.0, None))).unwrap();
        let mut bloque = vec![0i16; 4];
        m.llenar(&mut bloque, None);
        c.sonando.store(true, Ordering::Release);
        m.llenar(&mut bloque, None);
        assert!(bloque.iter().all(|&v| v > 30_000), "{bloque:?}");
    }
}
