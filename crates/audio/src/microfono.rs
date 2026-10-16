//! Captura del microfono en bruto: mono `f32` en un ring `rtrb`.
//!
//! Sin cancelacion de eco, supresion de ruido ni ganancia automatica
//! (`project-pitfalls.md`): `cpal` abre el dispositivo en modo compartido sin
//! efectos propios. El callback solo convierte, mezcla a mono y empuja.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SizedSample};

use crate::ErrorAudio;
use crate::dispositivos::{buscar_entrada, nombre_de};

/// Segundos de audio que caben en el ring.
const SEGUNDOS_RING: usize = 2;

#[derive(Debug, Default)]
struct CompartidoMic {
    /// Muestras mono entregadas por el dispositivo (incluidas las perdidas).
    capturadas: AtomicU64,
    /// Muestras descartadas porque el ring estaba lleno.
    perdidas: AtomicU64,
    perdido: AtomicBool,
}

/// Microfono abierto. El consumidor (hilo del juego) llama a
/// [`Microfono::leer`] con frecuencia (cada ~10-20 ms); el ring guarda
/// [`SEGUNDOS_RING`] s.
pub struct Microfono {
    _stream: cpal::Stream,
    consumidor: rtrb::Consumer<f32>,
    compartido: Arc<CompartidoMic>,
    frecuencia: u32,
    nombre: String,
    /// Indice de la siguiente muestra que devolvera `leer`.
    siguiente: u64,
    perdidas_vistas: u64,
}

impl Microfono {
    /// Abre la entrada `id` (de [`crate::listar_entradas`]; tambien vale el
    /// nombre) o la entrada por defecto si es `None`, con su configuracion
    /// por defecto.
    pub fn abrir(id: Option<&str>) -> Result<Microfono, ErrorAudio> {
        let host = cpal::default_host();
        let dispositivo = match id {
            None => host.default_input_device().ok_or(ErrorAudio::SinEntrada)?,
            Some(id) => buscar_entrada(&host, id)
                .ok_or_else(|| ErrorAudio::EntradaNoEncontrada(id.to_string()))?,
        };
        let nombre = nombre_de(&dispositivo).unwrap_or_default();
        let soportada = dispositivo
            .default_input_config()
            .map_err(|e| ErrorAudio::Microfono(e.to_string()))?;
        let config = soportada.config();
        let canales = usize::from(config.channels).max(1);
        let frecuencia = config.sample_rate;
        let (productor, consumidor) =
            rtrb::RingBuffer::new((frecuencia as usize).max(1) * SEGUNDOS_RING);
        let compartido = Arc::new(CompartidoMic::default());

        use cpal::SampleFormat as F;
        let c = &compartido;
        let stream = match soportada.sample_format() {
            F::F32 => construir::<f32>(&dispositivo, &config, productor, canales, c),
            F::F64 => construir::<f64>(&dispositivo, &config, productor, canales, c),
            F::I16 => construir::<i16>(&dispositivo, &config, productor, canales, c),
            F::I32 => construir::<i32>(&dispositivo, &config, productor, canales, c),
            F::U16 => construir::<u16>(&dispositivo, &config, productor, canales, c),
            F::U8 => construir::<u8>(&dispositivo, &config, productor, canales, c),
            F::I8 => construir::<i8>(&dispositivo, &config, productor, canales, c),
            otro => Err(ErrorAudio::Microfono(format!(
                "formato de muestra no soportado ({otro:?})"
            ))),
        }?;
        stream
            .play()
            .map_err(|e| ErrorAudio::Microfono(e.to_string()))?;
        Ok(Microfono {
            _stream: stream,
            consumidor,
            compartido,
            frecuencia,
            nombre,
            siguiente: 0,
            perdidas_vistas: 0,
        })
    }

    /// Frecuencia de muestreo del microfono (Hz).
    pub fn frecuencia(&self) -> u32 {
        self.frecuencia
    }

    /// Nombre del dispositivo abierto.
    pub fn nombre(&self) -> &str {
        &self.nombre
    }

    /// Vacia `destino` y copia en el todas las muestras disponibles. Devuelve
    /// el indice (contado desde que se abrio) de la primera muestra copiada.
    ///
    /// Si el ring se desbordo desde la lectura anterior, las muestras perdidas
    /// se cuentan antes de las leidas ahora, para que los indices sigan al
    /// tiempo real (aproximado solo en esa lectura).
    pub fn leer(&mut self, destino: &mut Vec<f32>) -> u64 {
        destino.clear();
        let perdidas = self.compartido.perdidas.load(Ordering::Acquire);
        if perdidas > self.perdidas_vistas {
            self.siguiente += perdidas - self.perdidas_vistas;
            self.perdidas_vistas = perdidas;
        }
        let disponibles = self.consumidor.slots();
        if let Ok(trozo) = self.consumidor.read_chunk(disponibles) {
            let (a, b) = trozo.as_slices();
            destino.extend_from_slice(a);
            destino.extend_from_slice(b);
            trozo.commit_all();
        }
        let inicio = self.siguiente;
        self.siguiente += destino.len() as u64;
        inicio
    }

    /// Indice de la siguiente muestra: muestras leidas mas perdidas. Es el
    /// `n_leidas` de CONTRATO seccion 7.
    pub fn muestras_totales(&self) -> u64 {
        self.siguiente
    }

    /// Muestras entregadas por el dispositivo hasta ahora.
    pub fn capturadas(&self) -> u64 {
        self.compartido.capturadas.load(Ordering::Acquire)
    }

    /// Muestras descartadas porque el ring estaba lleno (el lector se retraso
    /// mas de [`SEGUNDOS_RING`] s).
    pub fn perdidas(&self) -> u64 {
        self.compartido.perdidas.load(Ordering::Acquire)
    }

    /// El dispositivo de entrada se desconecto o el stream fallo.
    pub fn dispositivo_perdido(&self) -> bool {
        self.compartido.perdido.load(Ordering::Acquire)
    }
}

/// Cuerpo del callback de entrada: mezcla a mono y empuja. Sin reservas.
fn empujar_mono<T>(
    productor: &mut rtrb::Producer<f32>,
    datos: &[T],
    canales: usize,
    compartido: &CompartidoMic,
) where
    T: Sample,
    f32: FromSample<T>,
{
    let inverso = 1.0 / canales as f32;
    let mut cuadros = 0u64;
    let mut perdidas = 0u64;
    for cuadro in datos.chunks_exact(canales) {
        let suma: f32 = cuadro.iter().map(|&s| s.to_sample::<f32>()).sum();
        if productor.push(suma * inverso).is_err() {
            perdidas += 1;
        }
        cuadros += 1;
    }
    compartido.capturadas.fetch_add(cuadros, Ordering::Release);
    if perdidas > 0 {
        compartido.perdidas.fetch_add(perdidas, Ordering::Release);
    }
}

fn construir<T>(
    dispositivo: &cpal::Device,
    config: &cpal::StreamConfig,
    mut productor: rtrb::Producer<f32>,
    canales: usize,
    compartido: &Arc<CompartidoMic>,
) -> Result<cpal::Stream, ErrorAudio>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let en_callback = Arc::clone(compartido);
    let para_error = Arc::clone(compartido);
    dispositivo
        .build_input_stream(
            config,
            move |datos: &[T], _| empujar_mono(&mut productor, datos, canales, &en_callback),
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
        .map_err(|e| ErrorAudio::Microfono(e.to_string()))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn mezcla_a_mono_y_cuenta_perdidas() {
        let (mut productor, mut consumidor) = rtrb::RingBuffer::new(3);
        let compartido = CompartidoMic::default();
        let datos: [i16; 8] = [16_384, 0, 0, 16_384, -16_384, -16_384, 100, 100];
        empujar_mono(&mut productor, &datos, 2, &compartido);
        assert_eq!(compartido.capturadas.load(Ordering::Acquire), 4);
        assert_eq!(compartido.perdidas.load(Ordering::Acquire), 1);
        assert_eq!(consumidor.pop().unwrap(), 0.25);
        assert_eq!(consumidor.pop().unwrap(), 0.25);
        assert_eq!(consumidor.pop().unwrap(), -0.5);
        assert!(consumidor.pop().is_err());
    }
}
