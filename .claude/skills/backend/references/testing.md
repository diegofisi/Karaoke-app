# Testing

**Aplica siempre.** `cargo test --workspace` está en la verificación. Código
nuevo en un crate lleva su test o una línea que diga por qué no.

## Clases de test

| Clase | Dónde | Contra qué | Ejemplo |
|---|---|---|---|
| Unitario de lógica pura | `#[cfg(test)] mod pruebas` al pie del módulo | señales sintéticas y números | `tono`: un seno de 440 Hz da `69.0 ± 0.05`; ruido blanco da `None` |
| Puntuación | `crates/puntuacion` | notas y curvas construidas en el test | cantar la nota exacta da el máximo; un semitono abajo da menos; una nota `libre` no puntúa |
| Lectura de `cancion.json` | `crates/karaoke-core/tests/` | fixtures JSON en `tests/datos/` | el ejemplo de CONTRATO §3 parsea; una `version` desconocida falla con mensaje en español |
| Carpetas y récords | `crates/biblioteca/tests/` | una carpeta temporal **por prueba** | carpeta sin `cancion.json` = no lista; escribir y releer `records.json` |
| Audio con dispositivo | — | — | No en `cargo test`: depende del hardware. Lo prueba el usuario |

`src-tauri` no tiene tests mientras los comandos sean finos: lo que se prueba
es la función del crate.

## Forma

```rust
#[test]
fn seno_de_440_hz_es_la_69() {
    let muestras = seno(440.0, 48_000, 2048);
    let midi = detectar(&muestras, 48_000).expect("un seno limpio tiene tono");
    assert!((midi - 69.0).abs() < 0.05, "esperaba 69.0, salió {midi}");
}
```

- Nombre del test = el comportamiento, en español, sin tildes.
- Cada `assert!` con su mensaje, en términos del producto.
- Señales generadas en el test (seno, silencio, ruido con semilla fija), no
  archivos de audio grandes en el repo.
- Sin `sleep`, sin leer `%APPDATA%`, sin tocar los datos reales del usuario.

## Carpetas temporales

Nombre único por prueba: `temp_dir()/{nombre}-{pid}-{contador}` con un
`AtomicU64` del proceso (el reloj de Windows puede dar el mismo instante a dos
tests en paralelo). Un guardián que borra la carpeta en `Drop`. Si entra
`tempfile` como dependencia de desarrollo, se usa ese y se anota en
`project.md`.

## Qué prueba cada cambio

| Cambio | Test |
|---|---|
| Detector de tono | casos de seno, octava (110, 220, 880 Hz), silencio, ruido, voz sintética con vibrato |
| Puntuación | nota exacta, desviada, sin voz, `dorada` vale doble, `libre` no puntúa, `notas` vacía |
| Tipo de `cancion.json` | el ejemplo del contrato parsea; campo nuevo con `default` abre un JSON viejo |
| Biblioteca | listado con canción lista, a medio hacer y carpeta ajena; récord atómico |
| Comando nuevo | ninguno en Rust; el test está en la función del crate |

## Lo que `cargo test` no cubre

- Que el comando esté registrado y que la interfaz lo llame bien: se cuenta a
  mano (`commands.md`).
- Latencia, cortes de audio, dispositivos reales: los prueba el usuario en
  `npm run tauri dev`.
- La fábrica: tiene sus propios tests en `fabrica/tests` (skill `fabrica`).

## Clippy

`cargo clippy --workspace --all-targets` corre en la verificación. Empieza en
cero avisos; cuando el workspace exista, se decide si va con `-D warnings`
desde el primer día (`project.md`). No añadas avisos.
