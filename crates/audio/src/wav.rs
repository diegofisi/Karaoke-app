//! Lectura de WAV a memoria y conversion de canales y frecuencia.
//!
//! Todo esto corre fuera del hilo de audio (al cargar una cancion).

use std::path::Path;

use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler};

use crate::ErrorAudio;

/// Tamano de bloque del remuestreador FFT, en cuadros.
const BLOQUE_REMUESTREO: usize = 1024;

/// Audio PCM intercalado en memoria, muestras `f32` en `[-1, 1]`.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioPcm {
    pub muestras: Vec<f32>,
    pub canales: usize,
    pub frecuencia: u32,
}

impl AudioPcm {
    /// Numero de cuadros (muestras por canal).
    pub fn cuadros(&self) -> usize {
        self.muestras.len().checked_div(self.canales).unwrap_or(0)
    }
}

/// Lee un WAV completo: PCM entero de 8 a 32 bits (incluido 24) o flotante
/// de 32 bits.
pub fn leer_wav(ruta: &Path) -> Result<AudioPcm, ErrorAudio> {
    let error = |detalle: String| ErrorAudio::Wav {
        ruta: ruta.to_path_buf(),
        detalle,
    };
    let lector = hound::WavReader::open(ruta).map_err(|e| error(e.to_string()))?;
    let spec = lector.spec();
    if spec.channels == 0 {
        return Err(error("el archivo no tiene canales".to_string()));
    }
    let muestras: Result<Vec<f32>, hound::Error> = match (spec.sample_format, spec.bits_per_sample)
    {
        (hound::SampleFormat::Float, 32) => lector.into_samples::<f32>().collect(),
        (hound::SampleFormat::Int, bits @ 1..=32) => {
            let escala = 1.0 / (1u64 << (bits - 1)) as f32;
            lector
                .into_samples::<i32>()
                .map(|m| m.map(|v| v as f32 * escala))
                .collect()
        }
        (formato, bits) => {
            return Err(error(format!(
                "formato no soportado ({formato:?} de {bits} bits)"
            )));
        }
    };
    Ok(AudioPcm {
        muestras: muestras.map_err(|e| error(e.to_string()))?,
        canales: usize::from(spec.channels),
        frecuencia: spec.sample_rate,
    })
}

/// Cambia el numero de canales de un audio intercalado.
///
/// - Igual numero: copia.
/// - A mono: media de todos los canales.
/// - Desde mono: la misma muestra en todos los canales.
/// - Otro caso: el canal `c` va al canal `c`; los que sobran se descartan y
///   los que faltan quedan en silencio.
pub fn adaptar_canales(muestras: &[f32], de: usize, a: usize) -> Vec<f32> {
    if de == 0 || a == 0 {
        return Vec::new();
    }
    if de == a {
        return muestras.to_vec();
    }
    let cuadros = muestras.len() / de;
    let mut salida = Vec::with_capacity(cuadros * a);
    for cuadro in muestras.chunks_exact(de) {
        if a == 1 {
            salida.push(cuadro.iter().sum::<f32>() / de as f32);
        } else if de == 1 {
            let valor = cuadro.first().copied().unwrap_or(0.0);
            salida.extend(std::iter::repeat_n(valor, a));
        } else {
            salida.extend((0..a).map(|c| cuadro.get(c).copied().unwrap_or(0.0)));
        }
    }
    salida
}

/// Remuestrea un audio intercalado de `de` Hz a `a` Hz (FFT de `rubato`,
/// sin el retardo inicial del filtro).
pub fn remuestrear(
    muestras: Vec<f32>,
    canales: usize,
    de: u32,
    a: u32,
) -> Result<Vec<f32>, ErrorAudio> {
    if de == a || muestras.is_empty() || canales == 0 {
        return Ok(muestras);
    }
    let error = |e: &dyn std::fmt::Display| ErrorAudio::Conversion(e.to_string());
    let cuadros = muestras.len() / canales;
    let mut remuestreador = Fft::<f32>::new(
        de as usize,
        a as usize,
        BLOQUE_REMUESTREO,
        canales,
        FixedSync::Input,
    )
    .map_err(|e| error(&e))?;
    let entrada = InterleavedSlice::new(&muestras, canales, cuadros).map_err(|e| error(&e))?;
    let salida = remuestreador
        .process_all(&entrada, cuadros, None)
        .map_err(|e| error(&e))?;
    Ok(salida.take_data())
}

/// Convierte un audio al numero de canales y la frecuencia de un dispositivo.
/// Devuelve las muestras intercaladas.
pub fn adaptar(audio: AudioPcm, canales: usize, frecuencia: u32) -> Result<Vec<f32>, ErrorAudio> {
    // Se remuestrea con el menor numero de canales posible.
    if canales < audio.canales {
        let reducidas = adaptar_canales(&audio.muestras, audio.canales, canales);
        remuestrear(reducidas, canales, audio.frecuencia, frecuencia)
    } else {
        let remuestreadas =
            remuestrear(audio.muestras, audio.canales, audio.frecuencia, frecuencia)?;
        Ok(adaptar_canales(&remuestreadas, audio.canales, canales))
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn escribir(
        ruta: &Path,
        formato: hound::SampleFormat,
        bits: u16,
        muestras: &[f32],
        canales: u16,
    ) {
        let spec = hound::WavSpec {
            channels: canales,
            sample_rate: 44_100,
            bits_per_sample: bits,
            sample_format: formato,
        };
        let mut w = hound::WavWriter::create(ruta, spec).unwrap();
        for &m in muestras {
            match formato {
                hound::SampleFormat::Float => w.write_sample(m).unwrap(),
                hound::SampleFormat::Int => {
                    let max = ((1i64 << (bits - 1)) - 1) as f32;
                    w.write_sample((m * max).round() as i32).unwrap();
                }
            }
        }
        w.finalize().unwrap();
    }

    #[test]
    fn lee_pcm16_pcm24_pcm32_y_flotante() {
        let dir = tempfile::tempdir().unwrap();
        let senal = [0.0, 0.5, -0.5, 0.25, -1.0, 0.75];
        for (nombre, formato, bits) in [
            ("p16.wav", hound::SampleFormat::Int, 16),
            ("p24.wav", hound::SampleFormat::Int, 24),
            ("p32.wav", hound::SampleFormat::Int, 32),
            ("f32.wav", hound::SampleFormat::Float, 32),
        ] {
            let ruta = dir.path().join(nombre);
            escribir(&ruta, formato, bits, &senal, 2);
            let audio = leer_wav(&ruta).unwrap();
            assert_eq!(audio.canales, 2, "{nombre}");
            assert_eq!(audio.frecuencia, 44_100, "{nombre}");
            assert_eq!(audio.cuadros(), 3, "{nombre}");
            for (a, b) in audio.muestras.iter().zip(senal) {
                assert!((a - b).abs() < 1e-3, "{nombre}: {a} frente a {b}");
            }
        }
    }

    #[test]
    fn wav_inexistente_da_error_en_espanol() {
        let e = leer_wav(Path::new("no-existe.wav")).unwrap_err();
        assert!(e.to_string().starts_with("No se pudo leer el audio"));
    }

    #[test]
    fn adapta_canales() {
        let estereo = [1.0, 0.0, 0.5, 0.5];
        assert_eq!(adaptar_canales(&estereo, 2, 1), vec![0.5, 0.5]);
        assert_eq!(adaptar_canales(&[0.3, 0.6], 1, 2), vec![0.3, 0.3, 0.6, 0.6]);
        assert_eq!(
            adaptar_canales(&estereo, 2, 3),
            vec![1.0, 0.0, 0.0, 0.5, 0.5, 0.0]
        );
        assert_eq!(adaptar_canales(&estereo, 2, 2), estereo.to_vec());
    }

    /// Frecuencia dominante por cruces por cero.
    fn frecuencia_estimada(senal: &[f32], fs: f64) -> f64 {
        let cruces = senal
            .windows(2)
            .filter(|w| w[0] <= 0.0 && w[1] > 0.0)
            .count();
        cruces as f64 * fs / senal.len() as f64
    }

    #[test]
    fn remuestrea_conservando_duracion_y_tono() {
        let fs_de = 44_100.0;
        let senal: Vec<f32> = (0..44_100)
            .map(|i| (2.0 * std::f64::consts::PI * 440.0 * i as f64 / fs_de).sin() as f32 * 0.5)
            .collect();
        let salida = remuestrear(senal, 1, 44_100, 48_000).unwrap();
        assert!(
            (salida.len() as i64 - 48_000).abs() <= 2,
            "{}",
            salida.len()
        );
        let f = frecuencia_estimada(&salida[1000..47_000], 48_000.0);
        assert!((f - 440.0).abs() < 2.0, "{f}");
    }

    #[test]
    fn adaptar_combina_canales_y_frecuencia() {
        let audio = AudioPcm {
            muestras: vec![0.1; 2 * 22_050],
            canales: 2,
            frecuencia: 22_050,
        };
        let salida = adaptar(audio, 1, 44_100).unwrap();
        assert!((salida.len() as i64 - 44_100).abs() <= 2);
        // Lejos de los bordes la continua se conserva.
        assert!((salida[20_000] - 0.1).abs() < 1e-3);
    }
}
