#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use karaoke_core::{NotaPlana, TipoNota};
use puntuacion::{Calificacion, Dificultad, Entrada, Evento, Nivel, Puntuador, Resultado};

fn nota(indice_linea: usize, inicio: f64, fin: f64, midi: f64, tipo: TipoNota) -> NotaPlana {
    NotaPlana {
        indice_linea,
        indice_palabra: 0,
        inicio,
        fin,
        midi,
        tipo,
    }
}

/// Cancion de prueba: dos lineas, cuatro notas normales.
fn cancion() -> Vec<NotaPlana> {
    vec![
        nota(0, 0.5, 1.0, 60.0, TipoNota::Normal),
        nota(0, 1.0, 1.8, 62.0, TipoNota::Normal),
        nota(1, 2.0, 2.6, 64.0, TipoNota::Normal),
        nota(1, 2.6, 3.4, 65.5, TipoNota::Normal),
    ]
}

/// La nota que suena en `t`, si hay alguna.
fn nota_en(notas: &[NotaPlana], t: f64) -> Option<&NotaPlana> {
    notas.iter().find(|n| n.inicio <= t && t < n.fin)
}

/// Canta de 0 a `fin` a `fps` frames por segundo. `voz` recibe la nota que
/// suena (si hay) y el tiempo, y devuelve el tono del usuario.
fn cantar(
    notas: &[NotaPlana],
    dificultad: Dificultad,
    fps: u32,
    fin: f64,
    voz: impl Fn(Option<&NotaPlana>, f64) -> Option<f32>,
) -> (Resultado, Vec<Evento>) {
    let mut p = Puntuador::new(notas, dificultad);
    let mut eventos = Vec::new();
    let frames = (fin * f64::from(fps)).ceil() as u32;
    for k in 0..=frames {
        let t = f64::from(k) / f64::from(fps);
        eventos.extend(p.frame(Entrada {
            t,
            midi: voz(nota_en(notas, t), t),
        }));
    }
    (p.terminar(), eventos)
}

fn afinado(n: Option<&NotaPlana>, _t: f64) -> Option<f32> {
    n.map(|n| n.midi as f32)
}

fn creditos(eventos: &[Evento]) -> Vec<(usize, f64)> {
    eventos
        .iter()
        .filter_map(|e| match e {
            Evento::NotaCerrada {
                indice, credito, ..
            } => Some((*indice, *credito)),
            Evento::LineaCerrada { .. } => None,
        })
        .collect()
}

fn bonos(eventos: &[Evento]) -> Vec<(usize, f64, f64)> {
    eventos
        .iter()
        .filter_map(|e| match e {
            Evento::LineaCerrada {
                indice_linea,
                ratio,
                bono,
            } => Some((*indice_linea, *ratio, *bono)),
            Evento::NotaCerrada { .. } => None,
        })
        .collect()
}

#[test]
fn cantar_perfecto_da_10000() {
    let notas = cancion();
    let (r, eventos) = cantar(&notas, Dificultad::Dificil, 100, 4.0, afinado);
    assert_eq!(r.puntos, 10_000);
    assert_eq!(r.nivel, Nivel::Leyenda);
    assert_eq!(r.por_calificacion.perfecta, 4);
    assert_eq!(r.mejor_combo, 4);
    assert!((r.precision_media - 1.0).abs() < 1e-12);
    assert_eq!(creditos(&eventos).len(), 4);
    assert_eq!(bonos(&eventos).len(), 2);
}

#[test]
fn silencio_da_0() {
    let notas = cancion();
    let (r, _) = cantar(&notas, Dificultad::Facil, 100, 4.0, |_, _| None);
    assert_eq!(r.puntos, 0);
    assert_eq!(r.nivel, Nivel::Desafinado);
    assert_eq!(r.por_calificacion.fallo, 4);
    assert_eq!(r.mejor_combo, 0);
    assert_eq!(r.precision_media, 0.0);
}

#[test]
fn un_semitono_de_error_segun_dificultad() {
    let notas = cancion();
    let semitono_arriba = |n: Option<&NotaPlana>, _| n.map(|n| n.midi as f32 + 1.0);

    let (r, eventos) = cantar(&notas, Dificultad::Normal, 100, 4.0, semitono_arriba);
    assert_eq!(r.puntos, 10_000);
    assert!(creditos(&eventos).iter().all(|(_, c)| *c == 1.0));

    let (r, eventos) = cantar(&notas, Dificultad::Dificil, 100, 4.0, semitono_arriba);
    for (_, c) in creditos(&eventos) {
        assert!((0.45..=0.55).contains(&c), "credito {c}");
    }
    assert_eq!(r.por_calificacion.regular, 4);
    // 9000 * 0.5 de notas y ningun bono (ratio 0.5).
    assert_eq!(r.puntos, 4_500);
}

#[test]
fn una_octava_por_debajo_puntua_completo() {
    let notas = cancion();
    let (r, _) = cantar(&notas, Dificultad::Dificil, 100, 4.0, |n, _| {
        n.map(|n| n.midi as f32 - 12.0)
    });
    assert_eq!(r.puntos, 10_000);
}

#[test]
fn la_dorada_vale_doble() {
    // Dos lineas, una nota de 1 s cada una; la segunda es dorada.
    let notas = vec![
        nota(0, 0.0, 1.0, 60.0, TipoNota::Normal),
        nota(1, 1.0, 2.0, 60.0, TipoNota::Dorada),
    ];
    let (solo_normal, _) = cantar(&notas, Dificultad::Normal, 100, 2.5, |n, _| {
        n.filter(|n| n.tipo == TipoNota::Normal)
            .map(|n| n.midi as f32)
    });
    let (solo_dorada, _) = cantar(&notas, Dificultad::Normal, 100, 2.5, |n, _| {
        n.filter(|n| n.tipo == TipoNota::Dorada)
            .map(|n| n.midi as f32)
    });
    // 9000 * 1/3 + bono de su linea (500); 9000 * 2/3 + 500.
    assert_eq!(solo_normal.puntos, 3_500);
    assert_eq!(solo_dorada.puntos, 6_500);
}

#[test]
fn las_notas_libres_se_ignoran() {
    let notas = vec![
        nota(0, 0.0, 1.0, 60.0, TipoNota::Normal),
        nota(0, 1.0, 2.0, 70.0, TipoNota::Libre),
        nota(1, 2.0, 3.0, 70.0, TipoNota::Libre),
    ];
    let p = Puntuador::new(&notas, Dificultad::Normal);
    assert_eq!(p.estado().notas_total, 1);
    // Solo se canta la normal; la libre queda en silencio.
    let (r, eventos) = cantar(&notas, Dificultad::Normal, 100, 3.5, |n, _| {
        n.filter(|n| n.tipo == TipoNota::Normal)
            .map(|n| n.midi as f32)
    });
    assert_eq!(r.puntos, 10_000);
    assert_eq!(creditos(&eventos), vec![(0, 1.0)]);
    // La linea 1 solo tiene una libre: no da bono ni evento.
    assert_eq!(bonos(&eventos).len(), 1);
}

#[test]
fn no_depende_de_los_frames_por_segundo() {
    let notas = cancion();
    // Error que sube y baja dentro de cada nota: 0 en los extremos, 2,5
    // semitonos en el centro.
    let voz = |n: Option<&NotaPlana>, t: f64| {
        n.map(|n| {
            let s = (t - n.inicio) / (n.fin - n.inicio);
            (n.midi + 2.5 * (std::f64::consts::PI * s).sin()) as f32
        })
    };
    let (a100, _) = cantar(&notas, Dificultad::Normal, 100, 4.0, voz);
    let (a50, _) = cantar(&notas, Dificultad::Normal, 50, 4.0, voz);
    assert!(
        a100.puntos > 2_000 && a100.puntos < 9_000,
        "{}",
        a100.puntos
    );
    let diferencia = (f64::from(a100.puntos) - f64::from(a50.puntos)).abs();
    assert!(
        diferencia <= 0.01 * f64::from(a100.puntos),
        "100 Hz: {}, 50 Hz: {}",
        a100.puntos,
        a50.puntos
    );
}

#[test]
fn bono_de_linea() {
    let notas = vec![
        nota(0, 0.0, 1.0, 60.0, TipoNota::Normal),
        nota(1, 1.0, 2.0, 60.0, TipoNota::Normal),
    ];
    // Linea 0 perfecta; linea 1 a 1,3 semitonos: credito 0,7 en Normal.
    let (r, eventos) = cantar(&notas, Dificultad::Normal, 100, 2.5, |n, t| {
        n.map(|n| n.midi as f32 + if t < 1.0 { 0.0 } else { 1.3 })
    });
    let b = bonos(&eventos);
    assert_eq!(b.len(), 2);
    assert_eq!(b[0].0, 0);
    assert!((b[0].1 - 1.0).abs() < 1e-9 && (b[0].2 - 500.0).abs() < 1e-9);
    assert_eq!(b[1].0, 1);
    assert!((b[1].1 - 0.7).abs() < 1e-4, "ratio {}", b[1].1);
    // 500 * (0,7 - 0,5) / 0,4 = 250.
    assert!((b[1].2 - 250.0).abs() < 0.1, "bono {}", b[1].2);
    // 4500 + 4500 * 0,7 + 500 + 250.
    assert_eq!(r.puntos, 8_400);
    assert_eq!(r.por_calificacion.perfecta, 1);
    assert_eq!(r.por_calificacion.bien, 1);
}

#[test]
fn ratio_de_linea_ponderado_por_peso() {
    // Una linea: nota de 1 s cantada a medias (0,5) y dorada de 0,25 s
    // (peso 0,5) perfecta: ratio = (0,5 * 1 + 1 * 0,5) / 1,5 = 2/3.
    let notas = vec![
        nota(3, 0.0, 1.0, 60.0, TipoNota::Normal),
        nota(3, 1.0, 1.25, 62.0, TipoNota::Dorada),
    ];
    let (_, eventos) = cantar(&notas, Dificultad::Normal, 100, 2.0, |n, t| {
        n.map(|n| n.midi as f32 + if t < 1.0 { 1.5 } else { 0.0 })
    });
    let b = bonos(&eventos);
    assert_eq!(b.len(), 1);
    assert_eq!(b[0].0, 3);
    assert!((b[0].1 - 2.0 / 3.0).abs() < 1e-4, "ratio {}", b[0].1);
    let esperado = 1000.0 * ((2.0 / 3.0 - 0.5) / 0.4);
    assert!((b[0].2 - esperado).abs() < 0.5, "bono {}", b[0].2);
}

#[test]
fn combo_y_calificaciones() {
    let notas: Vec<NotaPlana> = (0..5)
        .map(|i| {
            nota(
                i,
                f64::from(i as u32),
                f64::from(i as u32) + 1.0,
                60.0,
                TipoNota::Normal,
            )
        })
        .collect();
    // Bien, bien, fallo, perfecta, perfecta (y la ultima por terminar()).
    let p_voz = |n: Option<&NotaPlana>, t: f64| {
        let desvio = match t as u32 {
            0 | 1 => 1.3,
            2 => 5.0,
            _ => 0.0,
        };
        n.map(|n| n.midi as f32 + desvio)
    };
    let mut p = Puntuador::new(&notas, Dificultad::Normal);
    let mut calificaciones = Vec::new();
    for k in 0..480 {
        let t = f64::from(k) / 100.0;
        for e in p.frame(Entrada {
            t,
            midi: p_voz(nota_en(&notas, t), t),
        }) {
            if let Evento::NotaCerrada { calificacion, .. } = e {
                calificaciones.push(calificacion);
            }
        }
    }
    assert_eq!(p.estado().combo, 1);
    assert_eq!(p.estado().notas_cerradas, 4);
    let r = p.terminar();
    assert_eq!(
        calificaciones,
        vec![
            Calificacion::Bien,
            Calificacion::Bien,
            Calificacion::Fallo,
            Calificacion::Perfecta
        ]
    );
    // La quinta nota la cierra terminar() con 80 frames afinados.
    assert_eq!(r.por_calificacion.perfecta, 2);
    assert_eq!(r.mejor_combo, 2);
}

#[test]
fn un_salto_cierra_las_notas_saltadas_con_credito_0() {
    let notas = cancion();
    let mut p = Puntuador::new(&notas, Dificultad::Normal);
    assert!(p.frame(Entrada { t: 0.0, midi: None }).is_empty());
    let eventos = p.frame(Entrada {
        t: 10.0,
        midi: Some(60.0),
    });
    assert_eq!(creditos(&eventos).len(), 4);
    assert!(creditos(&eventos).iter().all(|(_, c)| *c == 0.0));
    assert_eq!(p.estado().notas_cerradas, 4);
    assert_eq!(p.terminar().puntos, 0);
}

#[test]
fn sin_notas_puntuables_no_divide_por_cero() {
    let notas = vec![nota(0, 0.0, 1.0, 60.0, TipoNota::Libre)];
    let (r, eventos) = cantar(&notas, Dificultad::Normal, 100, 1.5, afinado);
    assert_eq!(r.puntos, 0);
    assert!(eventos.is_empty());
}
