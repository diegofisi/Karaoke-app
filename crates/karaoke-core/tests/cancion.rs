#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use karaoke_core::{Cancion, ErrorCancion, Idioma, Metodo, OrigenLetra, TipoNota};

fn ruta_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cancion_minima.json")
}

fn texto_fixture() -> String {
    std::fs::read_to_string(ruta_fixture()).unwrap()
}

#[test]
fn lee_todos_los_campos_del_fixture() {
    let c = Cancion::leer(&ruta_fixture()).unwrap();
    assert_eq!(c.version, 1);
    assert_eq!(c.idioma, Idioma::Ja);
    assert_eq!(c.origen.letra, OrigenLetra::Lrclib);
    assert_eq!(c.origen.youtube_id.as_deref(), Some("abc123XYZ_-"));
    assert_eq!(c.lineas.len(), 2);
    assert_eq!(c.lineas[0].metodo, Metodo::Alineada);
    assert_eq!(c.lineas[0].romaji.as_deref(), Some("konnichiwa sekai"));
    assert_eq!(c.lineas[1].metodo, Metodo::Respaldo);
    assert_eq!(c.lineas[1].romaji, None);
    assert_eq!(c.lineas[1].palabras[0].lectura, None);
    assert!(c.lineas[1].palabras[0].notas.is_empty());
    assert_eq!(c.lineas[0].palabras[0].notas[1].tipo, TipoNota::Dorada);
    assert_eq!(c.lineas[0].palabras[1].notas[0].tipo, TipoNota::Libre);
    assert_eq!(c.tono.midi[0], None);
    assert_eq!(c.tono.midi[2], Some(64.08));
}

#[test]
fn ida_y_vuelta_conserva_el_json() {
    let original: serde_json::Value = serde_json::from_str(&texto_fixture()).unwrap();
    let cancion = Cancion::desde_json(&texto_fixture()).unwrap();
    let escrito = serde_json::to_value(&cancion).unwrap();
    assert_eq!(original, escrito);
    let de_nuevo = Cancion::desde_json(&serde_json::to_string(&cancion).unwrap()).unwrap();
    assert_eq!(cancion, de_nuevo);
}

#[test]
fn enums_se_escriben_en_minusculas() {
    let cancion = Cancion::desde_json(&texto_fixture()).unwrap();
    let json = serde_json::to_string(&cancion).unwrap();
    for esperado in [
        "\"idioma\":\"ja\"",
        "\"letra\":\"lrclib\"",
        "\"metodo\":\"alineada\"",
        "\"metodo\":\"respaldo\"",
        "\"tipo\":\"dorada\"",
        "\"tipo\":\"libre\"",
        "\"tipo\":\"normal\"",
        "\"romaji\":null",
    ] {
        assert!(json.contains(esperado), "falta {esperado}");
    }
}

#[test]
fn notas_planas_y_resumen() {
    let c = Cancion::desde_json(&texto_fixture()).unwrap();
    let notas = c.notas_planas();
    assert_eq!(notas.len(), 3);
    assert!(notas.windows(2).all(|p| p[0].inicio <= p[1].inicio));
    assert_eq!((notas[2].indice_linea, notas[2].indice_palabra), (0, 1));
    assert_eq!(notas[1].tipo, TipoNota::Dorada);

    let r = c.resumen();
    assert_eq!(r.lineas_total, 2);
    assert_eq!(r.lineas_respaldo, 1);
    assert_eq!(r.id, "yt-abc123XYZ_-");
    assert_eq!(r.idioma, Idioma::Ja);
}

#[test]
fn curva_de_tono_del_fixture() {
    let c = Cancion::desde_json(&texto_fixture()).unwrap();
    assert_eq!(c.tono.en(0.0), None);
    assert_eq!(c.tono.en(0.032), Some(64.08));
    assert_eq!(c.tono.en(0.050), Some(64.12));
    assert_eq!(c.tono.en(10.0), None);
}

#[test]
fn rechaza_lineas_desordenadas() {
    let mut v: serde_json::Value = serde_json::from_str(&texto_fixture()).unwrap();
    v["lineas"].as_array_mut().unwrap().reverse();
    let err = Cancion::desde_json(&v.to_string()).unwrap_err();
    assert!(matches!(err, ErrorCancion::Invalida(_)), "{err:?}");
    assert!(err.to_string().starts_with("No se pudo leer la cancion: "));
    assert!(err.to_string().contains("ordenadas"));
}

#[test]
fn rechaza_notas_solapadas_dentro_de_la_linea() {
    let mut v: serde_json::Value = serde_json::from_str(&texto_fixture()).unwrap();
    v["lineas"][0]["palabras"][0]["notas"][0]["fin"] = serde_json::json!(1.6);
    let err = Cancion::desde_json(&v.to_string()).unwrap_err();
    assert!(err.to_string().contains("se solapan"), "{err}");
}

#[test]
fn acepta_solape_dentro_de_la_tolerancia() {
    let mut v: serde_json::Value = serde_json::from_str(&texto_fixture()).unwrap();
    v["lineas"][0]["palabras"][0]["notas"][0]["fin"] = serde_json::json!(1.5000005);
    assert!(Cancion::desde_json(&v.to_string()).is_ok());
}

#[test]
fn rechaza_version_distinta_de_1() {
    for version in [0, 2] {
        let mut v: serde_json::Value = serde_json::from_str(&texto_fixture()).unwrap();
        v["version"] = serde_json::json!(version);
        let err = Cancion::desde_json(&v.to_string()).unwrap_err();
        assert!(
            matches!(err, ErrorCancion::Version { encontrada } if encontrada == version),
            "{err:?}"
        );
        assert!(err.to_string().starts_with("No se pudo leer la cancion: "));
    }
}

#[test]
fn archivo_inexistente_da_error_en_espanol() {
    let err = Cancion::leer(Path::new("no/existe/cancion.json")).unwrap_err();
    assert!(matches!(err, ErrorCancion::Archivo { .. }));
    assert!(err.to_string().starts_with("No se pudo leer la cancion: "));
}
