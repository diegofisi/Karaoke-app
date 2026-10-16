#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use biblioteca::{Ajustes, Biblioteca, Dificultad, ErrorBiblioteca, ResumenCancion};
use karaoke_core::Idioma;

const FIXTURE: &str = include_str!("../../karaoke-core/tests/fixtures/cancion_minima.json");

fn biblioteca() -> (tempfile::TempDir, Biblioteca) {
    let dir = tempfile::tempdir().unwrap();
    let b = Biblioteca::new(dir.path().to_path_buf());
    (dir, b)
}

/// Crea `<datos>/canciones/<id>/cancion.json` a partir del fixture, con su
/// id y titulo, y fija la fecha de modificacion.
fn crear_cancion(b: &Biblioteca, id: &str, titulo: &str, hace_s: u64) -> PathBuf {
    let carpeta = b.carpeta_canciones().join(id);
    fs::create_dir_all(&carpeta).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    json["id"] = id.into();
    json["titulo"] = titulo.into();
    let ruta = carpeta.join("cancion.json");
    fs::write(&ruta, serde_json::to_string(&json).unwrap()).unwrap();
    let fecha = SystemTime::now() - Duration::from_secs(hace_s);
    fs::File::options()
        .write(true)
        .open(&ruta)
        .unwrap()
        .set_modified(fecha)
        .unwrap();
    carpeta
}

fn ids(lista: &[ResumenCancion]) -> Vec<&str> {
    lista.iter().map(|r| r.cancion.id.as_str()).collect()
}

#[test]
fn sin_carpeta_de_canciones_lista_vacia() {
    let (_dir, b) = biblioteca();
    assert!(b.listar().unwrap().is_empty());
}

#[test]
fn lista_la_mas_reciente_primero_y_salta_las_no_listas() {
    let (_dir, b) = biblioteca();
    crear_cancion(&b, "yt-vieja", "Vieja", 3000);
    crear_cancion(&b, "yt-nueva", "Nueva", 10);
    crear_cancion(&b, "local-media", "Media", 600);
    // En proceso: carpeta sin cancion.json.
    fs::create_dir_all(b.carpeta_canciones().join("yt-en-proceso")).unwrap();
    fs::write(
        b.carpeta_canciones().join("yt-en-proceso").join("voz.wav"),
        b"",
    )
    .unwrap();
    // cancion.json roto.
    let rota = b.carpeta_canciones().join("yt-rota");
    fs::create_dir_all(&rota).unwrap();
    fs::write(rota.join("cancion.json"), "{").unwrap();
    // Un archivo suelto en canciones/.
    fs::write(b.carpeta_canciones().join("suelto.txt"), "x").unwrap();

    let lista = b.listar().unwrap();
    assert_eq!(ids(&lista), ["yt-nueva", "local-media", "yt-vieja"]);
    let nueva = &lista[0];
    assert_eq!(nueva.cancion.titulo, "Nueva");
    assert_eq!(nueva.cancion.idioma, Idioma::Ja);
    assert_eq!(nueva.record, None);
    assert!(nueva.cancion.lineas_total > 0);
}

#[test]
fn resumen_serializa_plano_como_el_contrato() {
    let (_dir, b) = biblioteca();
    crear_cancion(&b, "yt-uno", "Uno", 0);
    b.guardar_resultado("yt-uno", Dificultad::Normal, 5000)
        .unwrap();
    let lista = b.listar().unwrap();
    let json = serde_json::to_value(&lista[0]).unwrap();
    let mut claves: Vec<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    claves.sort_unstable();
    assert_eq!(
        claves,
        [
            "artista",
            "duracion",
            "id",
            "idioma",
            "lineas_respaldo",
            "lineas_total",
            "record",
            "titulo"
        ]
    );
    assert_eq!(json["record"], 5000);
}

#[test]
fn obtener_lee_la_cancion() {
    let (_dir, b) = biblioteca();
    crear_cancion(&b, "yt-uno", "Uno", 0);
    let c = b.obtener("yt-uno").unwrap();
    assert_eq!(c.id, "yt-uno");
    assert!(matches!(
        b.obtener("yt-otra"),
        Err(ErrorBiblioteca::NoExiste(_))
    ));
}

#[test]
fn borrar_quita_la_carpeta_y_rechaza_rutas() {
    let (dir, b) = biblioteca();
    let carpeta = crear_cancion(&b, "yt-uno", "Uno", 0);
    // Algo fuera de canciones/ que no debe tocarse.
    let fuera = dir.path().join("importante");
    fs::create_dir_all(&fuera).unwrap();

    for id in [
        "..",
        "../importante",
        "..\\importante",
        "yt-uno/..",
        "",
        "a/b",
        "C:\\x",
    ] {
        let e = b.borrar(id).unwrap_err();
        assert!(matches!(e, ErrorBiblioteca::IdInvalido(_)), "{id}");
        assert!(e.to_string().starts_with("No se pudo"), "{e}");
    }
    assert!(fuera.is_dir());
    assert!(carpeta.is_dir());

    b.borrar("yt-uno").unwrap();
    assert!(!carpeta.exists());
    assert!(b.listar().unwrap().is_empty());
    // Borrar algo que ya no existe no es un error.
    b.borrar("yt-uno").unwrap();
}

#[test]
fn carpeta_valida_el_id() {
    let (dir, b) = biblioteca();
    assert_eq!(
        b.carpeta("yt-uno").unwrap(),
        dir.path().join("canciones").join("yt-uno")
    );
    assert!(b.carpeta("../x").is_err());
}

#[test]
fn records_por_dificultad() {
    let (_dir, b) = biblioteca();
    let carpeta = crear_cancion(&b, "yt-uno", "Uno", 0);

    assert_eq!(
        b.guardar_resultado("yt-uno", Dificultad::Normal, 5000)
            .unwrap(),
        (None, true)
    );
    assert_eq!(
        b.guardar_resultado("yt-uno", Dificultad::Normal, 4000)
            .unwrap(),
        (Some(5000), false)
    );
    // Empatar no es record.
    assert_eq!(
        b.guardar_resultado("yt-uno", Dificultad::Normal, 5000)
            .unwrap(),
        (Some(5000), false)
    );
    assert_eq!(
        b.guardar_resultado("yt-uno", Dificultad::Normal, 7000)
            .unwrap(),
        (Some(7000 - 2000), true)
    );
    // Otra dificultad lleva su propio record.
    assert_eq!(
        b.guardar_resultado("yt-uno", Dificultad::Dificil, 3000)
            .unwrap(),
        (None, true)
    );

    let records = b.records("yt-uno").unwrap();
    assert_eq!(records.normal.as_ref().unwrap().puntos, 7000);
    assert_eq!(records.dificil.as_ref().unwrap().puntos, 3000);
    assert_eq!(records.facil, None);
    let fecha = &records.normal.as_ref().unwrap().fecha;
    assert!(fecha.len() == 20 && fecha.ends_with('Z'), "{fecha}");

    // En disco, con nombres del contrato y sin tmp sobrante.
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(carpeta.join("records.json")).unwrap()).unwrap();
    assert_eq!(json["normal"]["puntos"], 7000);
    assert!(json["facil"].is_null());
    assert!(!carpeta.join("records.json.tmp").exists());

    // El resumen muestra el mejor de todas las dificultades.
    assert_eq!(b.listar().unwrap()[0].record, Some(7000));
}

#[test]
fn records_rotos_cuentan_como_vacios() {
    let (_dir, b) = biblioteca();
    let carpeta = crear_cancion(&b, "yt-uno", "Uno", 0);
    fs::write(carpeta.join("records.json"), "no es json").unwrap();
    assert_eq!(b.listar().unwrap()[0].record, None);
    assert_eq!(
        b.guardar_resultado("yt-uno", Dificultad::Facil, 10)
            .unwrap(),
        (None, true)
    );
}

#[test]
fn guardar_resultado_de_cancion_inexistente_falla() {
    let (_dir, b) = biblioteca();
    assert!(matches!(
        b.guardar_resultado("yt-nada", Dificultad::Normal, 1),
        Err(ErrorBiblioteca::NoExiste(_))
    ));
    assert!(matches!(
        b.guardar_resultado("../x", Dificultad::Normal, 1),
        Err(ErrorBiblioteca::IdInvalido(_))
    ));
}

fn escribir(ruta: &Path, texto: &str) {
    fs::create_dir_all(ruta.parent().unwrap()).unwrap();
    fs::write(ruta, texto).unwrap();
}

#[test]
fn ajustes_por_defecto_sin_archivo_o_corrupto() {
    let (_dir, b) = biblioteca();
    let defecto = Ajustes {
        microfono: None,
        latencia_ms: 150.0,
        dificultad: Dificultad::Normal,
        volumen_musica: 0.9,
        volumen_guia: 0.0,
        volumen_voz: 1.0,
        mostrar_romaji: true,
        grabar: false,
    };
    assert_eq!(Ajustes::default(), defecto);
    assert_eq!(b.leer_ajustes(), defecto);
    escribir(&b.ruta_ajustes(), "{ roto");
    assert_eq!(b.leer_ajustes(), defecto);
}

#[test]
fn ajustes_con_campos_que_faltan() {
    let (_dir, b) = biblioteca();
    escribir(
        &b.ruta_ajustes(),
        r#"{ "latencia_ms": 95.5, "dificultad": "dificil" }"#,
    );
    let a = b.leer_ajustes();
    assert_eq!(a.latencia_ms, 95.5);
    assert_eq!(a.dificultad, Dificultad::Dificil);
    assert_eq!(a.volumen_musica, 0.9);
    assert!(a.mostrar_romaji);
    assert_eq!(a.microfono, None);
}

#[test]
fn ajustes_ida_y_vuelta() {
    let (_dir, b) = biblioteca();
    let a = Ajustes {
        microfono: Some("wasapi:{0.0.1.00000000}.{abc}".to_string()),
        latencia_ms: 120.0,
        dificultad: Dificultad::Facil,
        volumen_musica: 0.5,
        volumen_guia: 0.3,
        volumen_voz: 1.5,
        mostrar_romaji: false,
        grabar: false,
    };
    assert_eq!(b.guardar_ajustes(&a).unwrap(), a);
    assert_eq!(b.leer_ajustes(), a);
    assert!(!b.datos().join("ajustes.json.tmp").exists());

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(b.ruta_ajustes()).unwrap()).unwrap();
    assert_eq!(json["dificultad"], "facil");
    assert_eq!(json["volumen_guia"], 0.3);
}

#[test]
fn ajustes_fuera_de_rango_se_corrigen() {
    let (_dir, b) = biblioteca();
    let a = Ajustes {
        microfono: Some("  ".to_string()),
        latencia_ms: -20.0,
        volumen_musica: 3.0,
        volumen_guia: f64::NAN,
        ..Ajustes::default()
    };
    let guardados = b.guardar_ajustes(&a).unwrap();
    assert_eq!(guardados.microfono, None);
    assert_eq!(guardados.latencia_ms, 0.0);
    assert_eq!(guardados.volumen_musica, 1.0);
    assert_eq!(guardados.volumen_guia, 0.0);
}
