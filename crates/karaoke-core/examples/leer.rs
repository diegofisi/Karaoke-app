//! Lee un cancion.json real y resume su contenido: `cargo run -p karaoke-core --example leer -- <ruta>`.
use karaoke_core::Cancion;
use std::path::PathBuf;

fn main() {
    let Some(ruta) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("uso: leer <cancion.json>");
        std::process::exit(2);
    };
    match Cancion::leer(&ruta) {
        Ok(c) => println!(
            "{} - {}: {} lineas, {} notas, {} frames de tono",
            c.artista,
            c.titulo,
            c.lineas.len(),
            c.notas_planas().len(),
            c.tono.midi.len()
        ),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
