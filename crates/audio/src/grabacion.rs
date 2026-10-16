//! Grabacion de la voz del usuario durante una partida y su mezcla con lo que
//! sonaba (instrumental y, si estaba activa, la voz guia).
//!
//! La voz se guarda en segmentos continuos. Cada segmento empieza en el tiempo
//! de cancion de su primera muestra, ya corregido por `latencia_ms` (CONTRATO
//! seccion 7): asi la voz cae a tiempo con la musica en la mezcla. Una pausa,
//! un salto o muestras perdidas cortan el segmento.

use std::borrow::Cow;
use std::path::Path;

use crate::error::ErrorAudio;
use crate::wav::{AudioPcm, adaptar_canales, remuestrear};

/// Un tramo continuo de voz.
#[derive(Debug, Clone, PartialEq)]
pub struct Segmento {
    /// Tiempo de cancion (s) de la primera muestra.
    pub inicio: f64,
    /// Mono, a la frecuencia del microfono.
    pub muestras: Vec<f32>,
}

/// Voz grabada durante una partida.
#[derive(Debug, Clone)]
pub struct GrabacionVoz {
    frecuencia: u32,
    segmentos: Vec<Segmento>,
    /// Indice de microfono que continua el segmento abierto.
    siguiente: Option<u64>,
}

impl GrabacionVoz {
    pub fn new(frecuencia_mic: u32) -> GrabacionVoz {
        GrabacionVoz {
            frecuencia: frecuencia_mic,
            segmentos: Vec::new(),
            siguiente: None,
        }
    }

    /// Anade muestras leidas del microfono. `primera` es el indice de la
    /// primera y `t_primera` su tiempo de cancion. Si no continuan el segmento
    /// abierto, empieza uno nuevo.
    pub fn anadir(&mut self, primera: u64, t_primera: f64, muestras: &[f32]) {
        if muestras.is_empty() {
            return;
        }
        let continua = self.siguiente == Some(primera) && !self.segmentos.is_empty();
        if !continua {
            self.segmentos.push(Segmento {
                inicio: t_primera,
                muestras: Vec::with_capacity(self.frecuencia as usize * 30),
            });
        }
        if let Some(s) = self.segmentos.last_mut() {
            s.muestras.extend_from_slice(muestras);
        }
        self.siguiente = Some(primera + muestras.len() as u64);
    }

    /// Cierra el segmento abierto (pausa, salto).
    pub fn cortar(&mut self) {
        self.siguiente = None;
    }

    pub fn frecuencia(&self) -> u32 {
        self.frecuencia
    }

    pub fn segmentos(&self) -> &[Segmento] {
        &self.segmentos
    }

    /// Segundos de voz grabada.
    pub fn duracion(&self) -> f64 {
        let n: usize = self.segmentos.iter().map(|s| s.muestras.len()).sum();
        n as f64 / f64::from(self.frecuencia.max(1))
    }

    /// Tramo de cancion cubierto: del inicio del primer segmento al fin del ultimo.
    pub fn tramo(&self) -> Option<(f64, f64)> {
        let ini = self
            .segmentos
            .iter()
            .map(|s| s.inicio)
            .fold(f64::INFINITY, f64::min);
        let fin = self
            .segmentos
            .iter()
            .map(|s| s.inicio + s.muestras.len() as f64 / f64::from(self.frecuencia.max(1)))
            .fold(f64::NEG_INFINITY, f64::max);
        (ini.is_finite() && fin.is_finite() && fin > ini).then_some((ini, fin))
    }
}

/// Margen de musica antes de la primera voz y despues de la ultima.
const MARGEN_INICIO_S: f64 = 2.0;
const MARGEN_FIN_S: f64 = 3.0;
/// Nivel RMS al que se lleva la voz (solo cuenta donde hay voz). Una pista
/// masterizada ronda 0,2-0,3: con 0,18 la voz quedaba por debajo de la musica
/// (lo noto el usuario el 2026-10-15).
const RMS_VOZ_OBJETIVO: f32 = 0.28;
/// La musica entra algo mas baja en la grabacion que al jugar, para que la voz se entienda.
pub const MUSICA_EN_GRABACION: f32 = 0.75;
/// Filtro de graves: quita golpes y zumbido por debajo de esta frecuencia.
const CORTE_GRAVES_HZ: f32 = 80.0;
/// Compresor: por encima del umbral, la subida se divide por la razon.
const UMBRAL_COMPRESOR: f32 = 0.22;
const RAZON_COMPRESOR: f32 = 3.0;
const ATAQUE_S: f32 = 0.005;
const RELAJACION_S: f32 = 0.12;
/// Por debajo de esto una muestra se considera silencio para medir la voz.
const UMBRAL_SILENCIO: f32 = 0.01;
const PICO_MAXIMO: f32 = 0.98;

/// Ganancia que lleva la voz a `RMS_VOZ_OBJETIVO` (de 1x a 10x): un microfono
/// suele grabar mucho mas bajo que una pista masterizada.
pub fn ganancia_voz(voz: &GrabacionVoz) -> f32 {
    let (suma, n) = voz
        .segmentos()
        .iter()
        .flat_map(|s| s.muestras.iter())
        .filter(|m| m.abs() > UMBRAL_SILENCIO)
        .fold((0.0f64, 0usize), |(s, n), m| (s + f64::from(m * m), n + 1));
    if n == 0 {
        return 1.0;
    }
    let rms = (suma / n as f64).sqrt() as f32;
    (RMS_VOZ_OBJETIVO / rms.max(1e-6)).clamp(1.0, 10.0)
}

/// Filtro paso alto de un polo (quita graves por debajo de `corte_hz`).
pub fn filtrar_graves(muestras: &mut [f32], fs: u32, corte_hz: f32) {
    let rc = 1.0 / (2.0 * std::f32::consts::PI * corte_hz);
    let dt = 1.0 / fs.max(1) as f32;
    let a = rc / (rc + dt);
    let (mut x_prev, mut y_prev) = (0.0f32, 0.0f32);
    for m in muestras.iter_mut() {
        let y = a * (y_prev + *m - x_prev);
        x_prev = *m;
        y_prev = y;
        *m = y;
    }
}

/// Compresor con seguidor de envolvente: iguala silabas fuertes y debiles
/// para que todas las palabras se entiendan en la mezcla.
pub fn comprimir(muestras: &mut [f32], fs: u32, umbral: f32, razon: f32) {
    let fs = fs.max(1) as f32;
    let ataque = (-1.0 / (ATAQUE_S * fs)).exp();
    let relajacion = (-1.0 / (RELAJACION_S * fs)).exp();
    let mut env = 0.0f32;
    for m in muestras.iter_mut() {
        let nivel = m.abs();
        let coef = if nivel > env { ataque } else { relajacion };
        env = coef * env + (1.0 - coef) * nivel;
        if env > umbral {
            let objetivo = umbral + (env - umbral) / razon;
            *m *= objetivo / env;
        }
    }
}

/// Mezcla la musica (y la voz guia, si se da con su volumen) con la voz del
/// usuario. La voz pasa por filtro de graves, ganancia automatica por
/// `volumen_voz` y compresor. Salida estereo a la frecuencia de la musica,
/// recortada al tramo cantado con un margen. Sin voz grabada devuelve error.
pub fn mezclar(
    musica: &AudioPcm,
    volumen_musica: f32,
    guia: Option<(&AudioPcm, f32)>,
    voz: &GrabacionVoz,
    volumen_voz: f32,
) -> Result<AudioPcm, ErrorAudio> {
    let Some((ini_voz, fin_voz)) = voz.tramo() else {
        return Err(ErrorAudio::Conversion("no hay voz grabada".into()));
    };
    let fs = musica.frecuencia;
    let fs_f = f64::from(fs);
    // Sin copia si ya es estereo: una cancion de 5 min son ~100 MB por copia.
    let base: Cow<'_, [f32]> = if musica.canales == 2 {
        Cow::Borrowed(&musica.muestras)
    } else {
        Cow::Owned(adaptar_canales(&musica.muestras, musica.canales, 2))
    };
    let total_cuadros = base.len() / 2;
    let desde = ((ini_voz - MARGEN_INICIO_S).max(0.0) * fs_f) as usize;
    let hasta = (((fin_voz + MARGEN_FIN_S) * fs_f) as usize).clamp(desde, total_cuadros.max(desde));
    let cuadros = hasta - desde;

    let mut salida = vec![0.0f32; cuadros * 2];
    let mut sumar = |pista: &[f32], ganancia: f32| {
        for (k, par) in salida.chunks_exact_mut(2).enumerate() {
            let i = (desde + k) * 2;
            if i + 1 < pista.len() {
                par[0] += pista[i] * ganancia;
                par[1] += pista[i + 1] * ganancia;
            }
        }
    };
    sumar(&base, volumen_musica * MUSICA_EN_GRABACION);
    if let Some((g, volumen)) = guia.filter(|(_, v)| *v > 0.0) {
        let estereo = adaptar_canales(&g.muestras, g.canales, 2);
        let a_fs = remuestrear(estereo, 2, g.frecuencia, fs)?;
        sumar(&a_fs, volumen * MUSICA_EN_GRABACION);
    }

    let ganancia = ganancia_voz(voz) * volumen_voz.max(0.0);
    for s in voz.segmentos() {
        let mut limpia = s.muestras.clone();
        filtrar_graves(&mut limpia, voz.frecuencia(), CORTE_GRAVES_HZ);
        limpia.iter_mut().for_each(|m| *m *= ganancia);
        comprimir(
            &mut limpia,
            voz.frecuencia(),
            UMBRAL_COMPRESOR,
            RAZON_COMPRESOR,
        );
        let mono = remuestrear(limpia, 1, voz.frecuencia(), fs)?;
        let inicio = (s.inicio * fs_f).round() as i64 - desde as i64;
        for (j, m) in mono.iter().enumerate() {
            let k = inicio + j as i64;
            if k < 0 {
                continue;
            }
            let k = k as usize;
            if k >= cuadros {
                break;
            }
            salida[2 * k] += m;
            salida[2 * k + 1] += m;
        }
    }

    let pico = salida.iter().fold(0.0f32, |p, m| p.max(m.abs()));
    if pico > PICO_MAXIMO {
        let escala = PICO_MAXIMO / pico;
        salida.iter_mut().for_each(|m| *m *= escala);
    }
    Ok(AudioPcm {
        muestras: salida,
        canales: 2,
        frecuencia: fs,
    })
}

/// Escribe un WAV PCM de 16 bits.
pub fn escribir_wav(ruta: &Path, audio: &AudioPcm) -> Result<(), ErrorAudio> {
    let error = |e: hound::Error| ErrorAudio::Wav {
        ruta: ruta.to_path_buf(),
        detalle: e.to_string(),
    };
    let spec = hound::WavSpec {
        channels: audio.canales as u16,
        sample_rate: audio.frecuencia,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(ruta, spec).map_err(error)?;
    for m in &audio.muestras {
        w.write_sample((m.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16)
            .map_err(error)?;
    }
    w.finalize().map_err(error)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn silencio(segundos: f64, fs: u32) -> AudioPcm {
        AudioPcm {
            muestras: vec![0.0; (segundos * f64::from(fs)) as usize * 2],
            canales: 2,
            frecuencia: fs,
        }
    }

    #[test]
    fn segmentos_se_cortan_si_no_son_continuos() {
        let mut g = GrabacionVoz::new(1000);
        g.anadir(0, 1.0, &[0.1; 100]);
        g.anadir(100, 1.1, &[0.1; 100]); // continua
        g.anadir(500, 2.0, &[0.1; 100]); // salto: segmento nuevo
        g.cortar();
        g.anadir(600, 2.1, &[0.1; 100]); // tras cortar: nuevo aunque sea contiguo
        assert_eq!(g.segmentos().len(), 3);
        assert_eq!(g.segmentos()[0].muestras.len(), 200);
        assert_eq!(g.tramo(), Some((1.0, 2.2)));
    }

    #[test]
    fn la_voz_cae_en_su_tiempo_de_cancion() {
        let fs = 1000;
        let musica = silencio(10.0, fs);
        let mut g = GrabacionVoz::new(fs);
        let mut voz = vec![0.0f32; 200];
        voz[0] = 0.5; // un pulso en la primera muestra
        g.anadir(0, 5.0, &voz);
        let mezcla = mezclar(&musica, 1.0, None, &g, 1.0).unwrap();
        // Recorte: empieza 2 s antes de la voz (t = 3.0), asi el pulso esta en 2.0 s.
        let primero = mezcla.muestras.iter().position(|m| m.abs() > 0.0).unwrap();
        assert_eq!(primero / 2, 2 * fs as usize);
        assert_eq!(mezcla.canales, 2);
    }

    #[test]
    fn la_mezcla_no_satura() {
        let fs = 1000;
        let mut musica = silencio(4.0, fs);
        musica.muestras.iter_mut().for_each(|m| *m = 0.9);
        let mut g = GrabacionVoz::new(fs);
        g.anadir(0, 1.0, &[0.9; 1000]);
        let mezcla = mezclar(&musica, 1.0, None, &g, 1.0).unwrap();
        assert!(
            mezcla
                .muestras
                .iter()
                .all(|m| m.abs() <= PICO_MAXIMO + 1e-6)
        );
    }

    #[test]
    fn voz_baja_se_sube_y_sin_voz_no() {
        let mut g = GrabacionVoz::new(1000);
        g.anadir(0, 0.0, &[0.02; 1000]);
        assert!(ganancia_voz(&g) > 5.0);
        assert_eq!(ganancia_voz(&GrabacionVoz::new(1000)), 1.0);
    }

    #[test]
    fn el_compresor_acerca_fuertes_y_debiles() {
        let mut fuerte = vec![0.9f32; 2000];
        let mut debil = vec![0.1f32; 2000];
        comprimir(&mut fuerte, 1000, UMBRAL_COMPRESOR, RAZON_COMPRESOR);
        comprimir(&mut debil, 1000, UMBRAL_COMPRESOR, RAZON_COMPRESOR);
        let antes = 0.9 / 0.1;
        let despues = fuerte[1999] / debil[1999];
        assert!(
            despues < antes / 2.0,
            "razon antes {antes}, despues {despues}"
        );
        assert!((debil[1999] - 0.1).abs() < 1e-6, "lo debil no se toca");
    }

    #[test]
    fn el_filtro_quita_la_continua() {
        let mut v = vec![0.5f32; 4000];
        filtrar_graves(&mut v, 1000, 80.0);
        assert!(v[3999].abs() < 1e-3);
    }

    #[test]
    fn sin_voz_es_error() {
        assert!(
            mezclar(
                &silencio(1.0, 1000),
                1.0,
                None,
                &GrabacionVoz::new(1000),
                1.0
            )
            .is_err()
        );
    }
}
