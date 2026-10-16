//! Prueba la instalacion de la fabrica fuera de la interfaz:
//! `cargo run -p karaoke --example instalar_entorno -- <carpeta-datos> <carpeta-fabrica>`
//! Necesita `uv` en el PATH (en la app va empaquetado junto al ejecutable).
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(datos), Some(fuente)) = (args.next(), args.next()) else {
        eprintln!("uso: instalar_entorno <carpeta-datos> <carpeta-fabrica>");
        std::process::exit(2);
    };
    let datos = PathBuf::from(datos);
    let r = karaoke_lib::entorno::instalar(&datos, &PathBuf::from(fuente), |e| match e {
        karaoke_lib::entorno::EventoInstalacion::Paso {
            indice,
            total,
            mensaje,
            ..
        } => {
            println!("== [{indice}/{total}] {mensaje}")
        }
        karaoke_lib::entorno::EventoInstalacion::Linea { texto } => println!("   {texto}"),
    });
    match r {
        Ok(()) => println!(
            "INSTALADO: {}",
            karaoke_lib::entorno::python(&datos).display()
        ),
        Err(e) => {
            eprintln!("ERROR: {e}");
            std::process::exit(1);
        }
    }
}
