//! Puntuacion del juego: pura, sin audio ni hilos.
//!
//! # Reglas
//!
//! - **Total 10 000 puntos**: 9 000 para las notas y 1 000 de bono por linea.
//! - **Peso de una nota** = su duracion en segundos; `dorada` vale el doble;
//!   `libre` pesa 0 y nunca puntua. Una nota de duracion cero tampoco puntua.
//!   Puntos maximos de una nota = `9000 * peso / suma de pesos`.
//! - **Entrada**: un frame de audio cada 10 ms con `t` en tiempo de cancion (ya
//!   corregido por la latencia) y el tono del usuario en MIDI o `None`.
//! - **Frame dentro de una nota** (`inicio <= t < fin`): distancia `d` en
//!   semitonos entre el usuario y la nota, plegada a la octava
//!   (`d = |((u - n) mod 12 + 6) mod 12 - 6|`, en `[0, 6]`; cantar una octava
//!   por debajo cuenta como afinado). Con la tolerancia `tol` de la dificultad
//!   (Facil 2,0; Normal 1,0; Dificil 0,5), el credito del frame es 1 si
//!   `d <= tol`, baja en linea recta de 1 a 0 entre `tol` y `tol + 1`, y es 0
//!   mas alla o si no hay voz.
//! - **Credito de la nota** = media del credito de los frames que cayeron
//!   dentro de ella. Se cuentan frames, no tiempo de pared: el resultado no
//!   depende de cuantos frames por segundo lleguen. Una nota sin frames (por un
//!   salto en la reproduccion) tiene credito 0.
//! - **Cierre de una nota**: cuando llega un frame con `t >= fin`, o al
//!   `terminar()`. Suma `puntos_max * credito` y se califica: Perfecta
//!   `>= 0,9`, Bien `>= 0,6`, Regular `>= 0,3`, si no Fallo. Perfecta y Bien
//!   suben el combo; Regular y Fallo lo ponen a cero.
//! - **Bono de linea**: cuando todas las notas puntuables de una linea se han
//!   cerrado, `ratio` = media de sus creditos ponderada por peso y
//!   `bono = 1000 / lineas_con_notas_puntuables * clamp((ratio - 0,5) / 0,4, 0, 1)`.
//! - **Nivel** por puntos: menos de 2000 Desafinado, 4000 Aficionado,
//!   6000 Prometedor, 7500 Cantante, 9000 Estrella; si no, Leyenda.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use karaoke_core::{NotaPlana, TipoNota};
use serde::{Deserialize, Serialize};

/// Puntos de todas las notas juntas.
pub const PUNTOS_NOTAS: f64 = 9_000.0;
/// Puntos de todos los bonos de linea juntos.
pub const PUNTOS_LINEAS: f64 = 1_000.0;
/// Maximo de la partida.
pub const PUNTOS_MAXIMOS: u32 = 10_000;

/// Ancho, en semitonos, de la rampa de credito tras la tolerancia.
const RAMPA: f64 = 1.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dificultad {
    Facil,
    #[default]
    Normal,
    Dificil,
}

impl Dificultad {
    /// Semitonos de distancia con credito completo.
    pub fn tolerancia(self) -> f64 {
        match self {
            Dificultad::Facil => 2.0,
            Dificultad::Normal => 1.0,
            Dificultad::Dificil => 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Calificacion {
    Perfecta,
    Bien,
    Regular,
    Fallo,
}

impl Calificacion {
    pub fn desde_credito(credito: f64) -> Calificacion {
        if credito >= 0.9 {
            Calificacion::Perfecta
        } else if credito >= 0.6 {
            Calificacion::Bien
        } else if credito >= 0.3 {
            Calificacion::Regular
        } else {
            Calificacion::Fallo
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Nivel {
    Desafinado,
    Aficionado,
    Prometedor,
    Cantante,
    Estrella,
    Leyenda,
}

impl Nivel {
    pub fn desde_puntos(puntos: u32) -> Nivel {
        match puntos {
            0..2_000 => Nivel::Desafinado,
            2_000..4_000 => Nivel::Aficionado,
            4_000..6_000 => Nivel::Prometedor,
            6_000..7_500 => Nivel::Cantante,
            7_500..9_000 => Nivel::Estrella,
            _ => Nivel::Leyenda,
        }
    }
}

/// Un frame de audio de 10 ms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entrada {
    /// Tiempo de cancion en segundos, ya corregido por la latencia.
    pub t: f64,
    /// Tono del usuario en MIDI; `None` = sin voz.
    pub midi: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Evento {
    NotaCerrada {
        /// Indice de la nota en el slice que recibio `Puntuador::new`.
        indice: usize,
        calificacion: Calificacion,
        credito: f64,
    },
    LineaCerrada {
        indice_linea: usize,
        ratio: f64,
        bono: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Estado {
    pub puntos: f64,
    pub combo: u32,
    pub mejor_combo: u32,
    pub notas_cerradas: u32,
    pub notas_total: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PorCalificacion {
    pub perfecta: u32,
    pub bien: u32,
    pub regular: u32,
    pub fallo: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Resultado {
    /// Redondeados, como mucho 10 000.
    pub puntos: u32,
    pub por_calificacion: PorCalificacion,
    pub mejor_combo: u32,
    /// Media simple del credito (0 a 1) de las notas puntuables; 0 si no hay.
    pub precision_media: f64,
    pub nivel: Nivel,
}

#[derive(Debug, Clone)]
struct NotaInterna {
    indice: usize,
    inicio: f64,
    fin: f64,
    midi: f64,
    peso: f64,
    puntos_max: f64,
    /// Posicion en `Puntuador::lineas`.
    linea: usize,
    suma_credito: f64,
    frames: u32,
    cerrada: bool,
}

#[derive(Debug, Clone)]
struct LineaInterna {
    indice_linea: usize,
    pendientes: u32,
    suma_peso: f64,
    suma_credito_peso: f64,
}

/// Puntua una partida frame a frame.
#[derive(Debug, Clone)]
pub struct Puntuador {
    tolerancia: f64,
    /// Solo las notas puntuables, ordenadas por `inicio`.
    notas: Vec<NotaInterna>,
    lineas: Vec<LineaInterna>,
    /// Ninguna nota antes de esta sigue abierta.
    primera_abierta: usize,
    bono_por_linea: f64,
    puntos: f64,
    combo: u32,
    mejor_combo: u32,
    notas_cerradas: u32,
    por_calificacion: PorCalificacion,
    suma_creditos: f64,
}

impl Puntuador {
    /// `notas` suele ser `Cancion::notas_planas()`; no hace falta que este
    /// ordenado. Las notas `libre` y las de duracion cero se ignoran.
    pub fn new(notas: &[NotaPlana], dificultad: Dificultad) -> Puntuador {
        let mut internas: Vec<NotaInterna> = Vec::new();
        let mut lineas: Vec<LineaInterna> = Vec::new();
        for (indice, nota) in notas.iter().enumerate() {
            let peso = peso_de(nota);
            if !(peso > 0.0 && peso.is_finite() && nota.midi.is_finite()) {
                continue;
            }
            let linea = match lineas
                .iter()
                .position(|l| l.indice_linea == nota.indice_linea)
            {
                Some(posicion) => posicion,
                None => {
                    lineas.push(LineaInterna {
                        indice_linea: nota.indice_linea,
                        pendientes: 0,
                        suma_peso: 0.0,
                        suma_credito_peso: 0.0,
                    });
                    lineas.len() - 1
                }
            };
            if let Some(l) = lineas.get_mut(linea) {
                l.pendientes += 1;
                l.suma_peso += peso;
            }
            internas.push(NotaInterna {
                indice,
                inicio: nota.inicio,
                fin: nota.fin,
                midi: nota.midi,
                peso,
                puntos_max: 0.0,
                linea,
                suma_credito: 0.0,
                frames: 0,
                cerrada: false,
            });
        }
        let suma_pesos: f64 = internas.iter().map(|n| n.peso).sum();
        for n in &mut internas {
            n.puntos_max = PUNTOS_NOTAS * n.peso / suma_pesos;
        }
        internas.sort_by(|a, b| a.inicio.total_cmp(&b.inicio));
        let bono_por_linea = if lineas.is_empty() {
            0.0
        } else {
            PUNTOS_LINEAS / lineas.len() as f64
        };
        Puntuador {
            tolerancia: dificultad.tolerancia(),
            notas: internas,
            lineas,
            primera_abierta: 0,
            bono_por_linea,
            puntos: 0.0,
            combo: 0,
            mejor_combo: 0,
            notas_cerradas: 0,
            por_calificacion: PorCalificacion::default(),
            suma_creditos: 0.0,
        }
    }

    /// Procesa un frame de audio. Devuelve las notas y lineas que cierra.
    pub fn frame(&mut self, e: Entrada) -> Vec<Evento> {
        let mut eventos = Vec::new();
        if !e.t.is_finite() {
            return eventos;
        }
        let mut i = self.primera_abierta;
        while let Some(nota) = self.notas.get_mut(i) {
            if nota.inicio > e.t {
                break;
            }
            if !nota.cerrada {
                if e.t >= nota.fin {
                    self.cerrar(i, &mut eventos);
                } else {
                    nota.suma_credito += credito_frame(e.midi, nota.midi, self.tolerancia);
                    nota.frames += 1;
                }
            }
            i += 1;
        }
        self.avanzar_primera_abierta();
        eventos
    }

    pub fn estado(&self) -> Estado {
        Estado {
            puntos: self.puntos,
            combo: self.combo,
            mejor_combo: self.mejor_combo,
            notas_cerradas: self.notas_cerradas,
            notas_total: self.notas.len() as u32,
        }
    }

    /// Cierra las notas que sigan abiertas (con los frames que recibieron) y
    /// da el resultado final.
    pub fn terminar(mut self) -> Resultado {
        let mut eventos = Vec::new();
        for i in self.primera_abierta..self.notas.len() {
            self.cerrar(i, &mut eventos);
        }
        let puntos = self.puntos.round().clamp(0.0, f64::from(PUNTOS_MAXIMOS)) as u32;
        let precision_media = if self.notas.is_empty() {
            0.0
        } else {
            self.suma_creditos / self.notas.len() as f64
        };
        Resultado {
            puntos,
            por_calificacion: self.por_calificacion,
            mejor_combo: self.mejor_combo,
            precision_media,
            nivel: Nivel::desde_puntos(puntos),
        }
    }

    fn avanzar_primera_abierta(&mut self) {
        while self
            .notas
            .get(self.primera_abierta)
            .is_some_and(|n| n.cerrada)
        {
            self.primera_abierta += 1;
        }
    }

    /// Cierra la nota `i` si sigue abierta: puntos, calificacion, combo y, si
    /// era la ultima pendiente de su linea, el bono de la linea.
    fn cerrar(&mut self, i: usize, eventos: &mut Vec<Evento>) {
        let Some(nota) = self.notas.get_mut(i) else {
            return;
        };
        if nota.cerrada {
            return;
        }
        nota.cerrada = true;
        let credito = if nota.frames == 0 {
            0.0
        } else {
            nota.suma_credito / f64::from(nota.frames)
        };
        let (indice, peso, puntos_max, linea) =
            (nota.indice, nota.peso, nota.puntos_max, nota.linea);

        self.puntos += puntos_max * credito;
        self.suma_creditos += credito;
        self.notas_cerradas += 1;
        let calificacion = Calificacion::desde_credito(credito);
        let contador = match calificacion {
            Calificacion::Perfecta => &mut self.por_calificacion.perfecta,
            Calificacion::Bien => &mut self.por_calificacion.bien,
            Calificacion::Regular => &mut self.por_calificacion.regular,
            Calificacion::Fallo => &mut self.por_calificacion.fallo,
        };
        *contador += 1;
        match calificacion {
            Calificacion::Perfecta | Calificacion::Bien => {
                self.combo += 1;
                self.mejor_combo = self.mejor_combo.max(self.combo);
            }
            Calificacion::Regular | Calificacion::Fallo => self.combo = 0,
        }
        eventos.push(Evento::NotaCerrada {
            indice,
            calificacion,
            credito,
        });

        let Some(l) = self.lineas.get_mut(linea) else {
            return;
        };
        l.suma_credito_peso += credito * peso;
        l.pendientes = l.pendientes.saturating_sub(1);
        if l.pendientes == 0 {
            let ratio = l.suma_credito_peso / l.suma_peso;
            let bono = self.bono_por_linea * ((ratio - 0.5) / 0.4).clamp(0.0, 1.0);
            self.puntos += bono;
            eventos.push(Evento::LineaCerrada {
                indice_linea: l.indice_linea,
                ratio,
                bono,
            });
        }
    }
}

fn peso_de(nota: &NotaPlana) -> f64 {
    let duracion = nota.fin - nota.inicio;
    match nota.tipo {
        TipoNota::Normal => duracion,
        TipoNota::Dorada => 2.0 * duracion,
        TipoNota::Libre => 0.0,
    }
}

/// Distancia en semitonos plegada a la octava, en `[0, 6]`.
pub fn distancia_plegada(usuario: f64, nota: f64) -> f64 {
    (((usuario - nota).rem_euclid(12.0) + 6.0).rem_euclid(12.0) - 6.0).abs()
}

/// Credito de un frame (0 a 1) con la tolerancia `tolerancia` en semitonos.
pub fn credito_frame(usuario: Option<f32>, nota: f64, tolerancia: f64) -> f64 {
    let Some(u) = usuario else {
        return 0.0;
    };
    let d = distancia_plegada(f64::from(u), nota);
    if !d.is_finite() {
        0.0
    } else if d <= tolerancia {
        1.0
    } else {
        (1.0 - (d - tolerancia) / RAMPA).max(0.0)
    }
}

/// Mueve el tono del usuario por octavas enteras para dejarlo lo mas cerca
/// posible de `referencia` (para dibujarlo junto a la nota).
pub fn ajustar_octava(usuario: f32, referencia: f32) -> f32 {
    usuario + 12.0 * ((referencia - usuario) / 12.0).round()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn distancia_plegada_en_rango() {
        assert_eq!(distancia_plegada(60.0, 60.0), 0.0);
        assert_eq!(distancia_plegada(48.0, 60.0), 0.0);
        assert!((distancia_plegada(61.0, 60.0) - 1.0).abs() < 1e-12);
        assert!((distancia_plegada(59.0, 60.0) - 1.0).abs() < 1e-12);
        assert!((distancia_plegada(66.0, 60.0) - 6.0).abs() < 1e-12);
        assert!((distancia_plegada(71.0, 60.0) - 1.0).abs() < 1e-12);
        for i in 0..240 {
            let d = distancia_plegada(40.0 + f64::from(i) * 0.1, 60.0);
            assert!((0.0..=6.0).contains(&d));
        }
    }

    #[test]
    fn credito_frame_por_tramos() {
        assert_eq!(credito_frame(None, 60.0, 1.0), 0.0);
        assert_eq!(credito_frame(Some(61.0), 60.0, 1.0), 1.0);
        assert!((credito_frame(Some(61.5), 60.0, 1.0) - 0.5).abs() < 1e-9);
        assert_eq!(credito_frame(Some(62.5), 60.0, 1.0), 0.0);
    }

    #[test]
    fn ajustar_octava_acerca_a_la_referencia() {
        assert_eq!(ajustar_octava(48.0, 60.0), 60.0);
        assert_eq!(ajustar_octava(73.0, 60.0), 61.0);
        assert_eq!(ajustar_octava(54.5, 60.0), 54.5);
        assert_eq!(ajustar_octava(42.0, 60.0), 66.0);
    }

    #[test]
    fn niveles() {
        assert_eq!(Nivel::desde_puntos(0), Nivel::Desafinado);
        assert_eq!(Nivel::desde_puntos(1_999), Nivel::Desafinado);
        assert_eq!(Nivel::desde_puntos(2_000), Nivel::Aficionado);
        assert_eq!(Nivel::desde_puntos(5_999), Nivel::Prometedor);
        assert_eq!(Nivel::desde_puntos(7_499), Nivel::Cantante);
        assert_eq!(Nivel::desde_puntos(8_999), Nivel::Estrella);
        assert_eq!(Nivel::desde_puntos(9_000), Nivel::Leyenda);
        assert_eq!(Nivel::desde_puntos(10_000), Nivel::Leyenda);
    }
}
