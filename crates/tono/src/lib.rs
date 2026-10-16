//! Tono del microfono para el juego: DSP puro, sin E/S de audio.
//!
//! `DetectorTono` recibe muestras mono `f32` del dispositivo de entrada en
//! bloques de cualquier tamano y emite un [`FrameTono`] cada 10 ms (un salto).
//! Cada frame analiza la ventana de las ultimas `tamano_ventana` muestras con
//! el algoritmo de McLeod (MPM, crate `pitch-detection`).
//!
//! # Latencia
//!
//! `FrameTono::muestra` es el indice del **centro** de la ventana, en muestras
//! del dispositivo de entrada desde el inicio. El frame se emite cuando llega
//! la ultima muestra de la ventana, es decir `tamano_ventana / 2` muestras
//! despues de su centro (~21 ms a 44,1/48 kHz), mas lo que tarde en llegar el
//! bloque. Quien convierta `muestra` en tiempo de cancion debe usar esa marca
//! (no el momento de emision) y restar `latencia_ms` (ARQUITECTURA seccion 4).
//!
//! Ante un inicio de voz en la muestra `s0`, el primer frame sonoro tiene su
//! centro a menos de `salto + tamano_ventana / 2` muestras de `s0`: la ventana
//! se vuelve sonora cuando una parte suficiente de ella contiene la voz. En
//! tiempo de pared, ese frame se emite como mucho `salto + tamano_ventana`
//! muestras despues de `s0` (~53 ms a 48 kHz).
//!
//! # Memoria
//!
//! Los buffers propios (anillo y ventana lineal) se reservan en `new`;
//! `empujar` no reserva memoria por su cuenta. Ojo: `pitch-detection` 0.3 crea
//! un `FftPlanner` (y por tanto reserva) en cada analisis de ventana, unas 100
//! veces por segundo. Por eso `empujar` debe llamarse desde el hilo del juego,
//! no desde el callback de audio.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use pitch_detection::detector::PitchDetector;
use pitch_detection::detector::mcleod::McLeodDetector;

/// Claridad MPM (pico de la NSDF, 0 a 1) necesaria para encender la voz.
pub const UMBRAL_CLARIDAD_ENCENDER: f32 = 0.55;
/// Claridad MPM con la que la voz sigue encendida una vez encendida
/// (histeresis: evita que la nota parpadee en vocales debiles).
pub const UMBRAL_CLARIDAD_SOSTENER: f32 = 0.35;
/// RMS minimo de la ventana (escala de muestras -1..1, unos -50 dBFS) para
/// encender la voz. Por debajo se considera ruido de fondo.
pub const UMBRAL_RMS: f32 = 0.003;
/// Saltos que la voz sigue encendida, con el ultimo tono, despues de que deje
/// de cumplir la condicion (consonantes, cortes breves).
pub const SALTOS_RETENCION: u32 = 3;
/// Rango de tono aceptado. Fuera de el, el frame no tiene voz.
pub const HZ_MINIMO: f32 = 60.0;
pub const HZ_MAXIMO: f32 = 1100.0;
/// Duracion objetivo de la ventana de analisis, en segundos.
pub const DURACION_VENTANA: f64 = 0.043;
/// Saltos (frames) por segundo: un frame cada 10 ms.
pub const FRAMES_POR_SEGUNDO: u32 = 100;

/// Por debajo de este RMS ni siquiera se analiza (la NSDF de una ventana en
/// silencio digital es 0/0).
const RMS_PISO: f32 = 1e-6;
/// Frecuencia de muestreo minima admitida; valores menores se tratan como esta.
const FRECUENCIA_MINIMA: u32 = 8_000;

/// Resultado de un salto de 10 ms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameTono {
    /// Indice del centro de la ventana, en muestras del dispositivo de entrada
    /// desde el inicio.
    pub muestra: u64,
    /// Tono en MIDI con decimales; `None` = sin voz.
    pub midi: Option<f32>,
    /// Claridad del pico elegido por MPM; 0 si no se encontro ninguno o la
    /// ventana era demasiado debil para analizarla.
    pub claridad: f32,
    /// RMS de la ventana.
    pub rms: f32,
}

/// Detector de tono en tiempo real con puerta de voz con histeresis.
pub struct DetectorTono {
    frecuencia_muestreo: u32,
    tamano_ventana: usize,
    salto: usize,
    /// Ultimas `tamano_ventana` muestras; `escritura` es la mas antigua.
    anillo: Vec<f32>,
    escritura: usize,
    /// Copia lineal del anillo para el analisis.
    ventana: Vec<f32>,
    recibidas: u64,
    hasta_siguiente: usize,
    detector: McLeodDetector<f32>,
    sonoro: bool,
    retencion: u32,
    ultimo_midi: Option<f32>,
}

impl DetectorTono {
    /// Crea un detector para `frecuencia_muestreo` Hz (menos de 8000 se trata
    /// como 8000).
    ///
    /// Ventana: la potencia de dos mas cercana a 43 ms (2048 muestras a 44,1 y
    /// 48 kHz, 4096 a 96 kHz), y al menos dos periodos de `HZ_MINIMO`.
    /// Salto: `frecuencia_muestreo / 100` (10 ms).
    pub fn new(frecuencia_muestreo: u32) -> Self {
        let fs = frecuencia_muestreo.max(FRECUENCIA_MINIMA);
        let objetivo = (DURACION_VENTANA * f64::from(fs)).round() as usize;
        let dos_periodos = (2.0 * f64::from(fs) / f64::from(HZ_MINIMO)).ceil() as usize;
        let tamano_ventana =
            potencia_de_dos_cercana(objetivo).max(dos_periodos.next_power_of_two());
        let salto = (f64::from(fs) / f64::from(FRAMES_POR_SEGUNDO)).round() as usize;
        // Relleno de media ventana: la NSDF es exacta hasta un retardo de
        // media ventana, que cubre el periodo de HZ_MINIMO.
        let detector = McLeodDetector::new(tamano_ventana, tamano_ventana / 2);
        DetectorTono {
            frecuencia_muestreo: fs,
            tamano_ventana,
            salto,
            anillo: vec![0.0; tamano_ventana],
            escritura: 0,
            ventana: vec![0.0; tamano_ventana],
            recibidas: 0,
            hasta_siguiente: tamano_ventana,
            detector,
            sonoro: false,
            retencion: 0,
            ultimo_midi: None,
        }
    }

    pub fn frecuencia_muestreo(&self) -> u32 {
        self.frecuencia_muestreo
    }

    /// Muestras por ventana de analisis.
    pub fn tamano_ventana(&self) -> usize {
        self.tamano_ventana
    }

    /// Muestras entre frames consecutivos (10 ms).
    pub fn salto(&self) -> usize {
        self.salto
    }

    /// Recibe muestras mono (cualquier tamano de bloque) y llama a `salida`
    /// una vez por cada salto completado. El primer frame sale cuando hay una
    /// ventana completa.
    pub fn empujar(&mut self, muestras: &[f32], mut salida: impl FnMut(FrameTono)) {
        for &m in muestras {
            if let Some(hueco) = self.anillo.get_mut(self.escritura) {
                *hueco = m;
            }
            self.escritura += 1;
            if self.escritura >= self.tamano_ventana {
                self.escritura = 0;
            }
            self.recibidas += 1;
            self.hasta_siguiente -= 1;
            if self.hasta_siguiente == 0 {
                self.hasta_siguiente = self.salto;
                salida(self.analizar());
            }
        }
    }

    fn analizar(&mut self) -> FrameTono {
        // Linealiza el anillo: de la mas antigua a la mas reciente.
        let (recientes, antiguas) = self.anillo.split_at(self.escritura);
        let (principio, fin) = self.ventana.split_at_mut(antiguas.len());
        principio.copy_from_slice(antiguas);
        fin.copy_from_slice(recientes);

        let suma: f32 = self.ventana.iter().map(|m| m * m).sum();
        let rms = (suma / self.tamano_ventana as f32).sqrt();
        let muestra = self.recibidas - (self.tamano_ventana / 2) as u64;

        let mut claridad = 0.0;
        let mut hz = f32::NAN;
        let analizable = rms.is_finite() && rms >= RMS_PISO;
        if analizable && (self.sonoro || rms >= UMBRAL_RMS) {
            let fs = self.frecuencia_muestreo as usize;
            // Primero con el umbral alto: MPM toma el primer pico por encima
            // del umbral, y un umbral bajo favorece errores de octava. Solo si
            // la voz ya estaba encendida se acepta un pico mas debil.
            let tono = self
                .detector
                .get_pitch(&self.ventana, fs, 0.0, UMBRAL_CLARIDAD_ENCENDER)
                .or_else(|| {
                    if self.sonoro {
                        self.detector
                            .get_pitch(&self.ventana, fs, 0.0, UMBRAL_CLARIDAD_SOSTENER)
                    } else {
                        None
                    }
                });
            if let Some(p) = tono {
                claridad = p.clarity;
                hz = p.frequency;
            }
        }

        let en_rango = (HZ_MINIMO..=HZ_MAXIMO).contains(&hz);
        let cumple = en_rango
            && if self.sonoro {
                claridad >= UMBRAL_CLARIDAD_SOSTENER
            } else {
                claridad >= UMBRAL_CLARIDAD_ENCENDER && rms >= UMBRAL_RMS
            };

        let midi = if cumple {
            let midi = hz_a_midi(hz);
            self.sonoro = true;
            self.retencion = SALTOS_RETENCION;
            self.ultimo_midi = Some(midi);
            Some(midi)
        } else if self.sonoro && self.retencion > 0 {
            self.retencion -= 1;
            self.ultimo_midi
        } else {
            self.sonoro = false;
            self.retencion = 0;
            self.ultimo_midi = None;
            None
        };

        FrameTono {
            muestra,
            midi,
            claridad,
            rms,
        }
    }
}

/// Potencia de dos mas cercana a `n` en escala logaritmica (`n >= 1`).
fn potencia_de_dos_cercana(n: usize) -> usize {
    let n = n.max(1);
    let arriba = n.next_power_of_two();
    let abajo = (arriba / 2).max(1);
    // Comparacion geometrica: arriba/n frente a n/abajo.
    if (arriba as f64) * (abajo as f64) > (n as f64) * (n as f64) {
        abajo
    } else {
        arriba
    }
}

/// Hz a MIDI con decimales (440 Hz = 69).
pub fn hz_a_midi(hz: f32) -> f32 {
    69.0 + 12.0 * (hz / 440.0).log2()
}

/// MIDI con decimales a Hz (69 = 440 Hz).
pub fn midi_a_hz(midi: f32) -> f32 {
    440.0 * ((midi - 69.0) / 12.0).exp2()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn conversiones() {
        assert!((hz_a_midi(440.0) - 69.0).abs() < 1e-5);
        assert!((hz_a_midi(220.0) - 57.0).abs() < 1e-5);
        assert!((midi_a_hz(57.0) - 220.0).abs() < 1e-3);
        assert!((midi_a_hz(hz_a_midi(123.4)) - 123.4).abs() < 1e-3);
    }

    #[test]
    fn tamanos_de_ventana_y_salto() {
        let d = DetectorTono::new(44_100);
        assert_eq!((d.tamano_ventana(), d.salto()), (2048, 441));
        let d = DetectorTono::new(48_000);
        assert_eq!((d.tamano_ventana(), d.salto()), (2048, 480));
        let d = DetectorTono::new(96_000);
        assert_eq!((d.tamano_ventana(), d.salto()), (4096, 960));
        let d = DetectorTono::new(16_000);
        assert_eq!((d.tamano_ventana(), d.salto()), (1024, 160));
        let d = DetectorTono::new(0);
        assert_eq!(d.frecuencia_muestreo(), 8_000);
    }
}
