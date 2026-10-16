//! Calibracion de `latencia_ms` con pitidos (CONTRATO secciones 6 y 7).
//!
//! Cada pitido sale por el [`Reproductor`] en modo reloj libre, que da el
//! tiempo de reloj de su primera muestra. El microfono lo graba; el inicio se
//! busca en la grabacion y se convierte a tiempo de reloj con el mismo
//! [`Reloj`] que usa el juego (latencia 0). La diferencia es la ida y vuelta
//! completa: bufer de salida, altavoz, aire, microfono, bufer de entrada.

use std::ops::Range;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::{ErrorAudio, Microfono, Reloj, Reproductor};

pub const PITIDOS_POR_DEFECTO: usize = 6;
pub const FRECUENCIA_PITIDO_HZ: f64 = 1000.0;
pub const DURACION_PITIDO_S: f64 = 0.060;
pub const SEPARACION_PITIDOS_S: f64 = 0.8;

/// Ventana de busqueda del inicio, en segundos despues del pitido.
const BUSQUEDA_DESDE_S: f64 = 0.030;
const BUSQUEDA_HASTA_S: f64 = 0.600;
/// Tramo antes del pitido con el que se estima el ruido de fondo.
const RUIDO_S: f64 = 0.200;
/// Escucha previa para tener ruido de fondo y desfase del reloj.
const ESCUCHA_INICIAL: Duration = Duration::from_millis(400);
/// Espera extra maxima a que llegue el final de una ventana.
const ESPERA_EXTRA: Duration = Duration::from_millis(500);
const PAUSA_LECTURA: Duration = Duration::from_millis(5);

/// Bloque de energia del detector (1 ms).
const BLOQUE_S: f64 = 0.001;
/// Un bloque es sonido si su energia supera el ruido por este factor (10 dB).
const FACTOR_UMBRAL: f32 = 10.0;
/// Energia minima de un bloque con sonido (RMS ~ -60 dBFS).
const ENERGIA_MINIMA: f32 = 1e-6;
/// Bloques seguidos por encima del umbral para aceptar un inicio (descarta
/// chasquidos).
const BLOQUES_SEGUIDOS: usize = 3;
/// Peso de cada bloque silencioso en el ruido de fondo adaptativo.
const ALFA_RUIDO: f32 = 0.05;
/// Un bloque actualiza el ruido de fondo solo si su energia es menor que el
/// ruido por este factor (5 dB).
const FACTOR_ADAPTAR: f32 = 3.0;

/// Medidas minimas y dispersion maxima (MAD, ms) para un resultado fiable.
const MEDIDAS_FIABLES: usize = 4;
const MAD_FIABLE_MS: f64 = 15.0;

/// Progreso de la calibracion (`EventoCalibracion`, CONTRATO seccion 6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum EventoCalibracion {
    /// Suena el pitido `indice` (desde 0) de `total`.
    Pitido { indice: usize, total: usize },
    /// Retraso medido del pitido `indice`; `None` si no se detecto.
    Medida { indice: usize, ms: Option<f64> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResultadoCalibracion {
    /// Mediana de las medidas.
    pub latencia_ms: f64,
    pub medidas_ms: Vec<f64>,
    /// Al menos 4 medidas y desviacion absoluta mediana <= 15 ms.
    pub fiable: bool,
}

/// Mide `latencia_ms`. Descarga la cancion del reproductor (modo reloj libre),
/// suena `n_pitidos` de 1 kHz y 60 ms cada 0,8 s y busca cada uno en el
/// microfono. Bloquea unos `0,4 + 0,8 * n_pitidos` s: llamalo desde un hilo
/// propio. `al_progreso` recibe un [`EventoCalibracion`] por pitido y por
/// medida.
///
/// Error si el microfono no envia audio o si no se detecta ningun pitido.
pub fn calibrar(
    reproductor: &mut Reproductor,
    microfono: &mut Microfono,
    n_pitidos: usize,
    mut al_progreso: impl FnMut(EventoCalibracion),
) -> Result<ResultadoCalibracion, ErrorAudio> {
    reproductor.descargar()?;
    let mut grabadora = Grabadora::new(microfono);
    grabadora.escuchar(reproductor, microfono, ESCUCHA_INICIAL);
    if grabadora.grabacion.is_empty() {
        return Err(ErrorAudio::Calibracion(
            "el microfono no envia audio".to_string(),
        ));
    }

    let mut medidas = Vec::with_capacity(n_pitidos);
    for indice in 0..n_pitidos {
        al_progreso(EventoCalibracion::Pitido {
            indice,
            total: n_pitidos,
        });
        let t_pitido = reproductor.sonar_pitido(FRECUENCIA_PITIDO_HZ, DURACION_PITIDO_S)?;
        grabadora.escuchar(
            reproductor,
            microfono,
            Duration::from_secs_f64(SEPARACION_PITIDOS_S),
        );
        let ms = grabadora.medir(reproductor, microfono, t_pitido);
        medidas.extend(ms);
        al_progreso(EventoCalibracion::Medida { indice, ms });
    }

    resumir_medidas(&medidas).ok_or_else(|| {
        ErrorAudio::Calibracion(
            "no se oyo ningun pitido; sube el volumen de los altavoces y acerca el microfono"
                .to_string(),
        )
    })
}

/// Grabacion del microfono durante la calibracion, con su reloj.
struct Grabadora {
    grabacion: Vec<f32>,
    /// Indice de microfono de `grabacion[0]`.
    base: Option<u64>,
    bloque: Vec<f32>,
    reloj: Reloj,
    frecuencia: u32,
}

impl Grabadora {
    fn new(microfono: &Microfono) -> Grabadora {
        let frecuencia = microfono.frecuencia();
        Grabadora {
            grabacion: Vec::with_capacity(frecuencia as usize * 8),
            base: None,
            bloque: Vec::with_capacity(frecuencia as usize),
            reloj: Reloj::new(frecuencia),
            frecuencia,
        }
    }

    fn leer(&mut self, reproductor: &Reproductor, microfono: &mut Microfono) {
        let inicio = microfono.leer(&mut self.bloque);
        self.reloj
            .anotar(reproductor.posicion(), microfono.muestras_totales());
        if self.bloque.is_empty() {
            return;
        }
        let base = *self.base.get_or_insert(inicio);
        // Muestras perdidas (desbordamiento): se rellenan con silencio para
        // que el indice local siga al del microfono.
        let esperado = base + self.grabacion.len() as u64;
        if inicio > esperado {
            let hueco = usize::try_from(inicio - esperado).unwrap_or(0);
            self.grabacion.extend(std::iter::repeat_n(0.0, hueco));
        }
        self.grabacion.extend_from_slice(&self.bloque);
    }

    fn escuchar(
        &mut self,
        reproductor: &Reproductor,
        microfono: &mut Microfono,
        duracion: Duration,
    ) {
        let fin = Instant::now() + duracion;
        while Instant::now() < fin {
            self.leer(reproductor, microfono);
            std::thread::sleep(PAUSA_LECTURA);
        }
        self.leer(reproductor, microfono);
    }

    /// Retraso en ms del pitido que empezo en `t_pitido` (tiempo de reloj).
    fn medir(
        &mut self,
        reproductor: &Reproductor,
        microfono: &mut Microfono,
        t_pitido: f64,
    ) -> Option<f64> {
        let indice = |reloj: &Reloj, base: u64, t: f64| -> Option<usize> {
            let k = reloj.muestra_de_tiempo(t, 0.0)?;
            usize::try_from(k.saturating_sub(base)).ok()
        };
        // Espera a que la grabacion cubra la ventana entera.
        let limite = Instant::now() + ESPERA_EXTRA;
        loop {
            let base = self.base?;
            let hasta = indice(&self.reloj, base, t_pitido + BUSQUEDA_HASTA_S)?;
            if hasta <= self.grabacion.len() || Instant::now() >= limite {
                break;
            }
            std::thread::sleep(PAUSA_LECTURA);
            self.leer(reproductor, microfono);
        }
        let base = self.base?;
        let r = &self.reloj;
        let ruido = indice(r, base, t_pitido - RUIDO_S)?..indice(r, base, t_pitido)?;
        let busqueda = indice(r, base, t_pitido + BUSQUEDA_DESDE_S)?
            ..indice(r, base, t_pitido + BUSQUEDA_HASTA_S)?;
        let inicio = detectar_inicio(&self.grabacion, self.frecuencia, ruido, busqueda)?;
        let t_detectado = self.reloj.tiempo_de_muestra(base + inicio as u64, 0.0);
        Some((t_detectado - t_pitido) * 1000.0)
    }
}

/// Busca el inicio de un sonido en `senal[busqueda]`.
///
/// - Energia media por bloques de 1 ms.
/// - Ruido de fondo inicial: mediana de la energia de los bloques de
///   `senal[ruido]` (tramo anterior a que el sonido pueda llegar; si esta
///   vacio, los 3 primeros bloques de la busqueda). Despues se adapta con los
///   bloques de la busqueda que no pasan de 3 veces el ruido.
/// - Un bloque es sonido si su energia supera 10 veces (10 dB) el ruido y un
///   minimo absoluto. Hacen falta 3 bloques seguidos (descarta chasquidos).
/// - El inicio es la primera muestra del primer bloque de la racha (o del
///   bloque anterior) cuyo cuadrado supera el umbral.
///
/// Devuelve el indice en `senal` o `None` si no hay sonido. Funcion pura.
pub fn detectar_inicio(
    senal: &[f32],
    frecuencia: u32,
    ruido: Range<usize>,
    busqueda: Range<usize>,
) -> Option<usize> {
    let bloque = ((f64::from(frecuencia) * BLOQUE_S).round() as usize).max(1);
    let energia = |r: Range<usize>| -> f32 {
        let trozo = senal.get(r).unwrap_or(&[]);
        if trozo.is_empty() {
            0.0
        } else {
            trozo.iter().map(|x| x * x).sum::<f32>() / trozo.len() as f32
        }
    };

    let ruido_fin = ruido.end.min(senal.len());
    let mut energias_ruido: Vec<f32> = (ruido.start..ruido_fin)
        .step_by(bloque)
        .map(|i| energia(i..(i + bloque).min(ruido_fin)))
        .collect();
    energias_ruido.sort_by(f32::total_cmp);
    let fin = busqueda.end.min(senal.len());
    let mut i = busqueda.start;
    // Sin tramo de ruido, los primeros bloques de la busqueda hacen de ruido.
    if energias_ruido.is_empty() {
        while energias_ruido.len() < BLOQUES_SEGUIDOS && i + bloque <= fin {
            energias_ruido.push(energia(i..i + bloque));
            i += bloque;
        }
        energias_ruido.sort_by(f32::total_cmp);
    }
    let mut piso = energias_ruido
        .get(energias_ruido.len() / 2)
        .copied()
        .unwrap_or(0.0);

    let mut racha: Option<(usize, usize)> = None; // (inicio del primer bloque, bloques)
    while i + bloque <= fin {
        let umbral = (piso * FACTOR_UMBRAL).max(ENERGIA_MINIMA);
        let e = energia(i..i + bloque);
        if e > umbral {
            let (primero, n) = racha.unwrap_or((i, 0));
            if n + 1 >= BLOQUES_SEGUIDOS {
                return Some(refinar(senal, primero, bloque, busqueda.start, umbral));
            }
            racha = Some((primero, n + 1));
        } else {
            racha = None;
            // Solo los bloques claramente de ruido mueven el piso: los de la
            // rampa de un sonido lo subirian y el sonido no se detectaria.
            if e < piso.max(ENERGIA_MINIMA) * FACTOR_ADAPTAR {
                piso += ALFA_RUIDO * (e - piso);
            }
        }
        i += bloque;
    }
    None
}

/// Primera muestra con `x^2 > umbral` desde el bloque anterior a `primero`
/// (sin salir de la ventana de busqueda).
fn refinar(senal: &[f32], primero: usize, bloque: usize, minimo: usize, umbral: f32) -> usize {
    let desde = primero.saturating_sub(bloque).max(minimo);
    senal
        .get(desde..primero + bloque)
        .and_then(|trozo| trozo.iter().position(|x| x * x > umbral))
        .map_or(primero, |p| desde + p)
}

fn mediana(valores: &[f64]) -> Option<f64> {
    let mut v = valores.to_vec();
    v.sort_by(f64::total_cmp);
    let n = v.len();
    match n {
        0 => None,
        _ if n % 2 == 1 => v.get(n / 2).copied(),
        _ => Some((v.get(n / 2 - 1)? + v.get(n / 2)?) / 2.0),
    }
}

/// Resume las medidas: mediana, y fiable si hay al menos 4 y su desviacion
/// absoluta mediana es <= 15 ms. `None` sin medidas. Funcion pura.
pub fn resumir_medidas(medidas_ms: &[f64]) -> Option<ResultadoCalibracion> {
    let latencia_ms = mediana(medidas_ms)?;
    let desviaciones: Vec<f64> = medidas_ms.iter().map(|m| (m - latencia_ms).abs()).collect();
    let mad = mediana(&desviaciones)?;
    Some(ResultadoCalibracion {
        latencia_ms,
        medidas_ms: medidas_ms.to_vec(),
        fiable: medidas_ms.len() >= MEDIDAS_FIABLES && mad <= MAD_FIABLE_MS,
    })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    const FS: u32 = 48_000;

    struct Azar(u64);

    impl Azar {
        /// Uniforme en [-1, 1).
        fn muestra(&mut self) -> f32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
        }
    }

    /// Ruido de `amplitud_ruido` durante `segundos` con un pitido de 1 kHz y
    /// 60 ms (rampa de 2 ms, como lo deforma un altavoz) en `inicio_s`.
    fn senal(segundos: f64, amplitud_ruido: f32, inicio_s: f64, amplitud: f32) -> Vec<f32> {
        let mut azar = Azar(42);
        let n = (segundos * f64::from(FS)) as usize;
        let inicio = (inicio_s * f64::from(FS)) as usize;
        let largo = (DURACION_PITIDO_S * f64::from(FS)) as usize;
        let rampa = 0.002 * f64::from(FS);
        (0..n)
            .map(|i| {
                let mut v = azar.muestra() * amplitud_ruido;
                if i >= inicio && i < inicio + largo {
                    let k = (i - inicio) as f64;
                    let env = (k / rampa).min(1.0);
                    v += ((2.0 * std::f64::consts::PI * 1000.0 * k / f64::from(FS)).sin() * env)
                        as f32
                        * amplitud;
                }
                v
            })
            .collect()
    }

    fn s(t: f64) -> usize {
        (t * f64::from(FS)) as usize
    }

    /// Ventanas como en `calibrar`, para un pitido emitido en `t_pitido`.
    fn detectar(senal: &[f32], t_pitido: f64) -> Option<usize> {
        detectar_inicio(
            senal,
            FS,
            s(t_pitido - RUIDO_S)..s(t_pitido),
            s(t_pitido + BUSQUEDA_DESDE_S)..s(t_pitido + BUSQUEDA_HASTA_S),
        )
    }

    #[test]
    fn detecta_el_retraso_con_ruido() {
        for retraso in [0.045, 0.137, 0.320, 0.550] {
            let x = senal(1.5, 0.01, 0.5 + retraso, 0.2);
            let inicio = detectar(&x, 0.5).unwrap();
            let medido = inicio as f64 / f64::from(FS) - 0.5;
            assert!(
                (medido - retraso).abs() < 0.002,
                "retraso {retraso}: medido {medido}"
            );
        }
    }

    #[test]
    fn detecta_un_pitido_debil_sobre_ruido_fuerte() {
        // Energia del ruido uniforme de amplitud 0,05: 0,05^2/3 = 8,3e-4. Del
        // seno de 0,15: 0,15^2/2 = 1,1e-2. Unos 11 dB por encima.
        let x = senal(1.5, 0.05, 0.7, 0.15);
        let inicio = detectar(&x, 0.5).unwrap();
        let medido = inicio as f64 / f64::from(FS) - 0.5;
        assert!((medido - 0.2).abs() < 0.003, "{medido}");
    }

    #[test]
    fn sin_pitido_no_detecta_nada() {
        let x = senal(1.5, 0.02, 10.0, 0.0);
        assert_eq!(detectar(&x, 0.5), None);
    }

    #[test]
    fn ignora_un_chasquido_antes_del_pitido() {
        let mut x = senal(1.5, 0.01, 0.8, 0.2);
        for v in &mut x[s(0.6)..s(0.6) + 48] {
            *v = 0.9;
        }
        let inicio = detectar(&x, 0.5).unwrap();
        let medido = inicio as f64 / f64::from(FS) - 0.5;
        assert!((medido - 0.3).abs() < 0.002, "{medido}");
    }

    #[test]
    fn pitido_fuera_de_la_ventana_no_cuenta() {
        let x = senal(1.5, 0.01, 0.5 + 0.7, 0.2);
        assert_eq!(detectar(&x, 0.5), None);
    }

    #[test]
    fn ventanas_fuera_de_rango_no_fallan() {
        let x = senal(0.1, 0.01, 10.0, 0.0);
        assert_eq!(detectar_inicio(&x, FS, 0..0, 4000..90_000), None);
        assert_eq!(detectar_inicio(&[], FS, 0..10, 0..10), None);
    }

    #[test]
    fn resume_mediana_y_fiabilidad() {
        let r = resumir_medidas(&[100.0, 104.0, 98.0, 102.0, 101.0]).unwrap();
        assert_eq!(r.latencia_ms, 101.0);
        assert!(r.fiable);

        let pocas = resumir_medidas(&[100.0, 101.0, 102.0]).unwrap();
        assert!(!pocas.fiable);

        let dispersas = resumir_medidas(&[50.0, 100.0, 150.0, 200.0, 250.0]).unwrap();
        assert_eq!(dispersas.latencia_ms, 150.0);
        assert!(!dispersas.fiable);

        let par = resumir_medidas(&[100.0, 110.0, 120.0, 130.0]).unwrap();
        assert_eq!(par.latencia_ms, 115.0);
        assert!(par.fiable);

        assert_eq!(resumir_medidas(&[]), None);
    }

    #[test]
    fn eventos_con_la_forma_del_contrato() {
        let pitido = serde_json::to_value(EventoCalibracion::Pitido {
            indice: 0,
            total: 6,
        })
        .unwrap();
        assert_eq!(
            pitido,
            serde_json::json!({"tipo": "pitido", "indice": 0, "total": 6})
        );
        let medida = serde_json::to_value(EventoCalibracion::Medida {
            indice: 2,
            ms: None,
        })
        .unwrap();
        assert_eq!(
            medida,
            serde_json::json!({"tipo": "medida", "indice": 2, "ms": null})
        );
    }
}
