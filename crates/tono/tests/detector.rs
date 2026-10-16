#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::f64::consts::TAU;

use tono::{DetectorTono, FrameTono};

const AMPLITUD: f64 = 0.3;

/// Seno de frecuencia constante.
fn seno(fs: u32, hz: f64, n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| (AMPLITUD * (TAU * hz * i as f64 / f64::from(fs)).sin()) as f32)
        .collect()
}

/// Seno con vibrato: `profundidad` semitonos de pico a `velocidad` Hz.
fn seno_con_vibrato(fs: u32, hz: f64, profundidad: f64, velocidad: f64, n: usize) -> Vec<f32> {
    let mut fase = 0.0;
    (0..n)
        .map(|i| {
            let t = i as f64 / f64::from(fs);
            let f = hz * (profundidad * (TAU * velocidad * t).sin() / 12.0).exp2();
            fase += TAU * f / f64::from(fs);
            (AMPLITUD * fase.sin()) as f32
        })
        .collect()
}

/// Pasa la senal en bloques de `bloque` muestras y junta los frames.
fn procesar(detector: &mut DetectorTono, senal: &[f32], bloque: usize) -> Vec<FrameTono> {
    let mut frames = Vec::new();
    for trozo in senal.chunks(bloque) {
        detector.empujar(trozo, |f| frames.push(f));
    }
    frames
}

#[test]
fn seno_de_220_hz_da_midi_57() {
    for fs in [44_100, 48_000] {
        for bloque in [37, 256, 4096] {
            let mut d = DetectorTono::new(fs);
            let frames = procesar(&mut d, &seno(fs, 220.0, fs as usize), bloque);
            assert!(frames.len() > 50, "fs={fs}: {} frames", frames.len());
            // Todas las ventanas estan llenas de seno: ninguna es calentamiento.
            for f in &frames {
                let midi = f
                    .midi
                    .unwrap_or_else(|| panic!("fs={fs}: frame sin voz {f:?}"));
                assert!((midi - 57.0).abs() < 0.1, "fs={fs}: midi {midi}");
                assert!(f.claridad >= 0.8, "claridad {}", f.claridad);
            }
        }
    }
}

#[test]
fn silencio_no_tiene_voz() {
    let fs = 48_000;
    let mut d = DetectorTono::new(fs);
    let frames = procesar(&mut d, &vec![0.0; fs as usize], 512);
    assert!(!frames.is_empty());
    assert!(frames.iter().all(|f| f.midi.is_none() && f.rms == 0.0));
}

#[test]
fn seno_muy_debil_no_enciende_la_voz() {
    let fs = 48_000;
    let mut d = DetectorTono::new(fs);
    // RMS = 0.001 / sqrt(2), por debajo de UMBRAL_RMS.
    let senal: Vec<f32> = seno(fs, 220.0, fs as usize)
        .iter()
        .map(|m| m / 300.0)
        .collect();
    let frames = procesar(&mut d, &senal, 512);
    assert!(frames.iter().all(|f| f.midi.is_none()));
}

#[test]
fn fuera_de_rango_no_tiene_voz() {
    let fs = 48_000;
    let mut d = DetectorTono::new(fs);
    let frames = procesar(&mut d, &seno(fs, 1500.0, fs as usize), 512);
    assert!(frames.iter().all(|f| f.midi.is_none()));
}

#[test]
fn vibrato_de_6_hz_se_sigue_detectando() {
    for fs in [44_100, 48_000] {
        let mut d = DetectorTono::new(fs);
        let senal = seno_con_vibrato(fs, 220.0, 0.5, 6.0, 2 * fs as usize);
        let frames = procesar(&mut d, &senal, 480);
        let mut minimo = f32::MAX;
        let mut maximo = f32::MIN;
        for f in &frames {
            let midi = f.midi.unwrap_or_else(|| panic!("frame sin voz {f:?}"));
            assert!((midi - 57.0).abs() < 0.6, "midi {midi}");
            minimo = minimo.min(midi);
            maximo = maximo.max(midi);
        }
        // El vibrato se ve en la salida (la ventana de 43 ms lo suaviza algo).
        assert!(maximo - minimo > 0.4, "rango {minimo}..{maximo}");
    }
}

#[test]
fn las_marcas_avanzan_exactamente_un_salto() {
    for fs in [44_100, 48_000] {
        let mut d = DetectorTono::new(fs);
        let frames = procesar(&mut d, &seno(fs, 330.0, fs as usize / 2), 333);
        let salto = d.salto() as u64;
        assert_eq!(frames[0].muestra, d.tamano_ventana() as u64 / 2);
        assert!(
            frames
                .windows(2)
                .all(|p| p[1].muestra - p[0].muestra == salto)
        );
        // Una muestra por debajo de la ventana: aun no sale ningun frame.
        let mut d = DetectorTono::new(fs);
        let ventana = d.tamano_ventana();
        let mut n = 0;
        d.empujar(&vec![0.0; ventana - 1], |_| n += 1);
        assert_eq!(n, 0);
        d.empujar(&[0.0], |_| n += 1);
        assert_eq!(n, 1);
    }
}

/// Latencia de deteccion: un seno que empieza en `s0` da su primer frame sonoro
/// con el centro a menos de `salto + ventana / 2` muestras de `s0`.
#[test]
fn latencia_del_inicio_de_voz() {
    for fs in [44_100, 48_000] {
        let mut d = DetectorTono::new(fs);
        let s0: usize = 10_007;
        let mut senal = vec![0.0; s0];
        senal.extend(seno(fs, 220.0, fs as usize / 2));
        let frames = procesar(&mut d, &senal, 256);
        let primero = frames.iter().find(|f| f.midi.is_some()).unwrap();
        let limite = (d.salto() + d.tamano_ventana() / 2) as i64;
        let diferencia = primero.muestra as i64 - s0 as i64;
        assert!(
            diferencia.abs() <= limite,
            "fs={fs}: primer frame sonoro en {} (s0={s0}, limite {limite})",
            primero.muestra
        );
        // Ningun frame anterior a s0 por mas de media ventana tiene voz.
        assert!(
            frames
                .iter()
                .filter(|f| (f.muestra as i64) < s0 as i64 - limite)
                .all(|f| f.midi.is_none())
        );
    }
}

#[test]
fn la_retencion_cubre_cortes_breves_y_luego_apaga() {
    let fs = 48_000;
    let mut d = DetectorTono::new(fs);
    let mut senal = seno(fs, 220.0, fs as usize / 2);
    senal.extend(vec![0.0; fs as usize / 2]);
    let frames = procesar(&mut d, &senal, 480);
    let ultimo_sonoro = frames.iter().rposition(|f| f.midi.is_some()).unwrap();
    // Tras el ultimo sonoro todo es silencio.
    assert!(frames[ultimo_sonoro + 1..].iter().all(|f| f.midi.is_none()));
    // Los SALTOS_RETENCION ultimos frames sonoros son silencio puro que
    // repite el ultimo tono medido.
    let retenidos = &frames[ultimo_sonoro + 1 - tono::SALTOS_RETENCION as usize..=ultimo_sonoro];
    let medido = frames[ultimo_sonoro - tono::SALTOS_RETENCION as usize];
    assert!(medido.claridad >= tono::UMBRAL_CLARIDAD_SOSTENER);
    for f in retenidos {
        assert_eq!(f.rms, 0.0);
        assert_eq!(f.midi, medido.midi);
    }
}
