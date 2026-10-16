//! Prueba manual del audio con dispositivos reales (no corre en CI).
//!
//! ```text
//! cargo run -p audio --example prueba_audio -- [cancion.wav] [--calibrar] [--microfono <id>]
//! ```
//!
//! - Lista las entradas.
//! - Con un WAV: lo reproduce 5 s e imprime `posicion`, `posicion_oida` y el
//!   RMS del microfono cada 250 ms.
//! - Con `--calibrar`: mide `latencia_ms` con pitidos.

use std::error::Error;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use audio::{Microfono, PITIDOS_POR_DEFECTO, Reproductor, calibrar, listar_entradas};

fn main() -> Result<(), Box<dyn Error>> {
    let mut wav: Option<PathBuf> = None;
    let mut calibrar_latencia = false;
    let mut id_microfono: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--calibrar" => calibrar_latencia = true,
            "--microfono" => id_microfono = args.next(),
            _ => wav = Some(PathBuf::from(arg)),
        }
    }

    println!("Entradas:");
    for e in listar_entradas() {
        let marca = if e.por_defecto { " (por defecto)" } else { "" };
        println!("  {}{marca}\n    id: {}", e.nombre, e.id);
    }

    let mut reproductor = Reproductor::abrir()?;
    println!(
        "Salida: {} Hz, {} canales",
        reproductor.frecuencia(),
        reproductor.canales()
    );
    let mut microfono = Microfono::abrir(id_microfono.as_deref())?;
    println!(
        "Microfono: {} a {} Hz",
        microfono.nombre(),
        microfono.frecuencia()
    );

    if let Some(ruta) = wav {
        reproductor.cargar(&ruta, None)?;
        reproductor.fijar_volumenes(0.9, 0.0);
        println!("Duracion: {:.2} s", reproductor.duracion());
        reproductor.reproducir();
        let mut bloque = Vec::new();
        let inicio = Instant::now();
        while inicio.elapsed() < Duration::from_secs(5) && !reproductor.termino() {
            std::thread::sleep(Duration::from_millis(250));
            microfono.leer(&mut bloque);
            let rms = if bloque.is_empty() {
                0.0
            } else {
                (bloque.iter().map(|x| x * x).sum::<f32>() / bloque.len() as f32).sqrt()
            };
            println!(
                "posicion {:7.3} s  oida {:7.3} s  rms mic {:.4}  perdidas {}",
                reproductor.posicion(),
                reproductor.posicion_oida(),
                rms,
                microfono.perdidas()
            );
        }
        reproductor.pausar();
    }

    if calibrar_latencia {
        println!("Calibrando: sube el volumen y deja el microfono cerca del altavoz.");
        let resultado = calibrar(
            &mut reproductor,
            &mut microfono,
            PITIDOS_POR_DEFECTO,
            |evento| println!("  {evento:?}"),
        )?;
        println!(
            "latencia_ms = {:.1} (fiable: {}), medidas: {:?}",
            resultado.latencia_ms, resultado.fiable, resultado.medidas_ms
        );
    }
    Ok(())
}
