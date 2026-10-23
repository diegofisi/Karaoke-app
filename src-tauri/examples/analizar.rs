//! `cargo run -p karaoke --example analizar -- <url>`: vista previa por consola.
fn main() {
    let url = std::env::args().nth(1).unwrap_or_default();
    let u = karaoke_lib::fabrica::ubicar(None, std::path::Path::new("."));
    match karaoke_lib::vista_previa::analizar(
        &u,
        &url,
        &karaoke_lib::fabrica::SesionYoutube::default(),
    ) {
        Ok(a) => println!(
            "{}",
            serde_json::to_string_pretty(&a.vista).unwrap_or_default()
        ),
        Err(e) => {
            eprintln!("ERROR: {e}");
            std::process::exit(1);
        }
    }
}
