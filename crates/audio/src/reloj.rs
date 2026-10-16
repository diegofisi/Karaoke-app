//! Indice de muestra del microfono -> tiempo de cancion (CONTRATO seccion 7).
//!
//! La formula del contrato, para la lectura que acaba de hacerse:
//!
//! `t(k) = posicion_al_leer - (n_leidas - k) / fs_mic - latencia_ms / 1000`
//!
//! Se reescribe como `t(k) = desfase + k / fs_mic - latencia_ms / 1000` con
//! `desfase = posicion_al_leer - n_leidas / fs_mic`. `posicion` avanza a
//! saltos de un bloque de salida y `n_leidas` a saltos de un bloque de
//! entrada, asi que el desfase de cada lectura tiembla unos milisegundos. Se
//! suaviza con una media exponencial; un cambio mayor que
//! [`SALTO_MAXIMO_S`] (busqueda, pausa larga) reinicia la media.
//!
//! La calibracion usa este mismo reloj con latencia 0, asi que el sesgo medio
//! del suavizado queda dentro de `latencia_ms`.

/// Peso de cada lectura nueva en la media exponencial. Con una lectura cada
/// 16 ms, la constante de tiempo es ~0,3 s.
pub const ALFA_POR_DEFECTO: f64 = 0.05;

/// Diferencia (s) entre el desfase de una lectura y la media a partir de la
/// cual se considera un salto del reloj y la media se reinicia.
pub const SALTO_MAXIMO_S: f64 = 0.25;

#[derive(Debug, Clone, PartialEq)]
pub struct Reloj {
    frecuencia: f64,
    alfa: f64,
    salto_maximo: f64,
    desfase: Option<f64>,
}

impl Reloj {
    /// Reloj para un microfono de `frecuencia_mic` Hz con los parametros por
    /// defecto.
    pub fn new(frecuencia_mic: u32) -> Reloj {
        Reloj::con_parametros(frecuencia_mic, ALFA_POR_DEFECTO, SALTO_MAXIMO_S)
    }

    /// `alfa` en `(0, 1]` (1 = sin suavizado); `salto_maximo` en segundos.
    pub fn con_parametros(frecuencia_mic: u32, alfa: f64, salto_maximo: f64) -> Reloj {
        Reloj {
            frecuencia: f64::from(frecuencia_mic.max(1)),
            alfa: if alfa.is_finite() {
                alfa.clamp(1e-6, 1.0)
            } else {
                ALFA_POR_DEFECTO
            },
            salto_maximo,
            desfase: None,
        }
    }

    /// Anota una lectura: `posicion` del reproductor y
    /// [`crate::Microfono::muestras_totales`], tomadas juntas justo despues
    /// de leer el microfono.
    pub fn anotar(&mut self, posicion: f64, muestras_totales: u64) {
        if !posicion.is_finite() {
            return;
        }
        let instantaneo = posicion - muestras_totales as f64 / self.frecuencia;
        self.desfase = Some(match self.desfase {
            Some(d) if (instantaneo - d).abs() <= self.salto_maximo => {
                d + self.alfa * (instantaneo - d)
            }
            _ => instantaneo,
        });
    }

    /// Olvida la media. Llamalo al buscar, pausar o reanudar la cancion (el
    /// reloj de la cancion salta o se para mientras el microfono sigue).
    pub fn reiniciar(&mut self) {
        self.desfase = None;
    }

    /// Tiempo de cancion de la muestra 0 del microfono, suavizado. `None`
    /// antes de la primera lectura.
    pub fn desfase(&self) -> Option<f64> {
        self.desfase
    }

    /// Tiempo de cancion (s) de la muestra `k` del microfono, ya corregido por
    /// `latencia_ms`. Antes de la primera [`Reloj::anotar`] toma desfase 0.
    pub fn tiempo_de_muestra(&self, k: u64, latencia_ms: f64) -> f64 {
        self.desfase.unwrap_or(0.0) + k as f64 / self.frecuencia - latencia_ms / 1000.0
    }

    /// Inverso de [`Reloj::tiempo_de_muestra`]: la muestra cuyo tiempo es el
    /// mas cercano a `t`. `None` antes de la primera lectura o si `t` cae
    /// antes de la muestra 0.
    pub fn muestra_de_tiempo(&self, t: f64, latencia_ms: f64) -> Option<u64> {
        let desfase = self.desfase?;
        let k = ((t + latencia_ms / 1000.0 - desfase) * self.frecuencia).round();
        (k.is_finite() && k >= 0.0).then_some(k as u64)
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Generador congruencial lineal: ruido reproducible sin dependencias.
    struct Azar(u64);

    impl Azar {
        fn uniforme(&mut self) -> f64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    #[test]
    fn sin_suavizado_es_la_formula_del_contrato() {
        let mut r = Reloj::con_parametros(48_000, 1.0, SALTO_MAXIMO_S);
        let posicion = 12.345;
        let n_leidas = 480_000;
        r.anotar(posicion, n_leidas);
        for k in [n_leidas - 4800, n_leidas - 1, n_leidas] {
            let contrato = posicion - (n_leidas - k) as f64 / 48_000.0 - 150.0 / 1000.0;
            assert!((r.tiempo_de_muestra(k, 150.0) - contrato).abs() < 1e-9);
        }
    }

    #[test]
    fn primera_lectura_no_suaviza_e_inverso_cuadra() {
        let mut r = Reloj::new(44_100);
        assert_eq!(r.muestra_de_tiempo(1.0, 0.0), None);
        r.anotar(2.0, 44_100);
        assert_eq!(r.desfase(), Some(1.0));
        assert_eq!(r.muestra_de_tiempo(1.5, 0.0), Some(22_050));
        assert_eq!(r.muestra_de_tiempo(1.5, 100.0), Some(26_460));
        assert_eq!(r.muestra_de_tiempo(0.5, 0.0), None);
        let t = r.tiempo_de_muestra(26_460, 100.0);
        assert!((t - 1.5).abs() < 1e-9);
    }

    /// Simula el juego: salida a 48 kHz en bloques de 480 cuadros, entrada a
    /// 44,1 kHz en bloques de 441 muestras (los dos de 10 ms), lecturas cada
    /// 16 ms con +-4 ms de temblor. La muestra `k` del microfono se capto en
    /// el tiempo de cancion `k / fs + DESFASE`.
    #[test]
    fn suaviza_el_temblor_de_los_bloques() {
        // No multiplo de 10 ms: los bloques de entrada y salida no van en fase.
        const DESFASE: f64 = 0.7037;
        let mut azar = Azar(7);
        let mut r = Reloj::new(44_100);
        let mut ahora = 1.0;
        let mut error_bruto_max: f64 = 0.0;
        let mut errores = Vec::new();
        for lectura in 0..2000 {
            ahora += 0.016 + (azar.uniforme() - 0.5) * 0.008;
            let posicion = (ahora * 48_000.0 / 480.0).floor() * 480.0 / 48_000.0;
            let muestras = ((ahora - DESFASE) * 44_100.0 / 441.0).floor() as u64 * 441;
            let bruto = posicion - muestras as f64 / 44_100.0;
            error_bruto_max = error_bruto_max.max((bruto - DESFASE).abs());
            r.anotar(posicion, muestras);
            if lectura > 200 {
                errores.push(r.desfase().unwrap() - DESFASE);
            }
        }
        let media = errores.iter().sum::<f64>() / errores.len() as f64;
        let max = errores.iter().fold(0.0f64, |m, e| m.max(e.abs()));
        assert!(error_bruto_max > 0.006, "{error_bruto_max}");
        // Bloques iguales en duracion: el sesgo medio se cancela.
        assert!(media.abs() < 0.001, "media {media}");
        assert!(max < 0.004, "maximo {max}");
    }

    #[test]
    fn un_salto_del_reloj_reinicia_la_media() {
        let mut r = Reloj::new(1000);
        r.anotar(10.0, 5000);
        r.anotar(10.01, 5000);
        assert!((r.desfase().unwrap() - 5.0005).abs() < 1e-9);
        // Busqueda a 60 s: la media no arrastra el valor viejo.
        r.anotar(60.0, 5000);
        assert_eq!(r.desfase(), Some(55.0));
        r.reiniciar();
        assert_eq!(r.desfase(), None);
    }
}
