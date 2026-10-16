# Vinculación — karaoke, backend Rust + Tauri 2

**Estado (2026-10-15):** el backend Rust **no existe todavía**. No hay
`Cargo.toml`, ni `crates/`, ni código en `src-tauri/` (solo una carpeta
`target/` vieja). Este archivo describe lo **previsto** en
`docs/ARQUITECTURA.md` (fase 2: crates; fase 3: Tauri) y en
`docs/CONTRATO.md`. Cuando el código exista, se re-ejecuta `bind-project` y
este archivo pasa a describir lo que hay. Donde este archivo y el código
discrepen, **manda el código**.

Todos los comandos y rutas se dan desde la raíz del repo,
`C:\Users\USER\Desktop\Karaoke\`, que es a la vez raíz del workspace Cargo,
del `package.json` y del proyecto Tauri (no hay subcarpeta de app).

## Qué es

Karaoke de escritorio. Dos mitades (ARQUITECTURA §1):

- **Fábrica** (Python, proceso aparte, skill `fabrica`): antes de jugar,
  descarga, separa la voz, consigue la letra, la alinea palabra por palabra,
  saca el tono de referencia y las notas. Escribe `cancion.json`.
- **Juego** (Rust, esta skill): reproduce el instrumental, captura el
  micrófono, detecta el tono cada 10 ms, puntúa y manda frames a la interfaz.

Regla: todo lo lento (modelos de IA) va en la fábrica. Durante el juego solo
corre DSP ligero en Rust.

## Stack previsto

- Rust estable, workspace en la raíz con miembros `crates/*` y `src-tauri`.
  Dependencias compartidas en `[workspace.dependencies]`.
- Tauri 2. `src-tauri` con `[lib]` de nombre distinto del binario (sufijo
  `_lib`).
- `cpal` para audio. Ring buffer sin bloqueos (`rtrb` o `ringbuf`; por
  decidir).
- `serde`, `serde_json`, `anyhow`.
- Sin base de datos (`records.json`); `rusqlite` quizá más adelante.
- Frontend en `src/` (skill `frontend`).

Ninguna versión está fijada todavía. Se anotan aquí cuando exista
`Cargo.lock`.

## Aplica / no aplica

| Tema de la doctrina | ¿Aplica? | Dónde / por qué |
|---|---|---|
| Crates de dominio puros + pegamento Tauri (`architecture.md`) | **Sí** | `karaoke-core`, `tono`, `puntuacion`, `audio`, `biblioteca` sin `tauri`; `src-tauri` los une |
| Comandos finos (`commands.md`) | **Sí** | `src-tauri/src/comandos/`; lista en CONTRATO §5 cuando exista |
| Error como `String` en español (`errors.md`) | **Sí** | Empieza por "No se pudo…" (CONTRATO §1) |
| Estado con `Mutex` corto (`state-and-threads.md`) | **Sí** | Juego en curso y proceso de la fábrica en el estado |
| Hilo de audio sin reservas ni locks | **Sí** | `crates/audio`, callback de `cpal` |
| Hilo del juego + `Channel` a ~60 Hz (`events.md`) | **Sí** | CONTRATO §6 |
| Proceso externo con líneas JSON por `stdout` | **Sí** | La fábrica, CONTRATO §4 |
| Eventos `emit` | **Casi no** | Solo para algo que nadie pidió (dispositivo perdido). Los flujos van por `Channel` |
| Persistencia (`persistence.md`) | **Sí, archivos** | Carpetas de canción + `records.json` atómico. Sin SQL |
| Base de datos / migraciones | **No** | No hay SQLite. Si entra, se reescribe `persistence.md` antes |
| Tests con `cargo test` (`testing.md`) | **Sí** | Lógica pura con señales sintéticas; biblioteca con carpetas temporales |
| Capabilities / permisos Tauri | **Mínimo** | Los comandos propios no necesitan permiso |
| Red, HTTP, auth | **No** | La red la usa la fábrica (YouTube, LRCLIB), no Rust |
| Win32 / `unsafe` | **No previsto** | Salvo lo que `cpal` haga por dentro |

## Mapa de crates previsto (ARQUITECTURA §3)

| Crate / módulo | Posee | Conoce |
|---|---|---|
| `crates/karaoke-core` | Tipos serde de `cancion.json` versión 1 (`Cancion`, `Origen`, `Linea`, `Palabra`, `Nota`, `TipoNota`, `CurvaTono`) y su lectura y validación (orden, `version`) | `serde`, `serde_json`, `anyhow` |
| `crates/tono` | Detector MPM para el micrófono: bloque `&[f32]` → `Option<f64>` MIDI + claridad | solo `std` |
| `crates/puntuacion` | Puntuación por nota y por línea a partir de tono del micrófono con marca de tiempo y las notas de `cancion.json` | `karaoke-core` |
| `crates/audio` | `cpal`: reproducción con reloj por muestras, captura del micrófono con marca, calibración por pitidos (`latencia_ms`) | `cpal`, ring buffer |
| `crates/biblioteca` | Listar `<datos>/canciones/`, leer `cancion.json`, leer y escribir `records.json` | `karaoke-core`, `std::fs` |
| `src-tauri` | Estado, comandos, `generate_handler!`, lanzar la fábrica y reenviar sus líneas por `Channel`, bucle del juego | todo lo anterior + `tauri` |

## Contrato / fuente de verdad

- `docs/CONTRATO.md` manda. Los nombres en español, sin tildes,
  `snake_case`, iguales en JSON, Rust, TypeScript y Python. La interfaz no
  traduce nombres.
- Tiempos en segundos `f64` desde el inicio del audio original. Tono en MIDI
  `f64`; `null` = sin voz.
- Cambiar un tipo de `cancion.json` = `crates/karaoke-core` +
  `src/common/interfaces/` + `fabrica/karaoke_fabrica/modelo.py` +
  `CONTRATO.md` en el mismo cambio (`.claude/rules/contrato.md`).

Invariantes que el tipo no expresa:

| Invariante | Se cumplirá en |
|---|---|
| `cancion.json` solo lo escribe la fábrica | `biblioteca` solo lee; ningún comando escribe en la carpeta salvo `records.json` |
| Sin `cancion.json`, la canción no está lista | listado de `biblioteca` |
| El reloj es el contador de muestras de la salida | `crates/audio` |
| `latencia_ms` se resta al tiempo del micrófono | hilo del juego |
| Se puntúa cada 10 ms de audio, no por frame de pantalla | `crates/puntuacion` + hilo del juego |
| `notas` vacía = no puntúa tono; `libre` no puntúa; `dorada` vale doble | `crates/puntuacion` |
| `stdout` de la fábrica: solo líneas JSON | lector en `src-tauri` (una línea que no parsea es error) |

## Comandos previstos

```bash
npm run tauri dev          # app completa. NO desde una sesión de Claude: no termina
npm run dev                # solo la interfaz. Tampoco termina
```

## Verificación prevista

Desde la raíz, `npm run verify` encadenará con `&&`: `tsc -b` →
`check:rules` → `vite build` → `cargo clippy --workspace --all-targets` →
`cargo test --workspace` → pytest de la fábrica. **Hoy no existe** (no hay
`package.json`). Cuando exista el workspace:

```bash
cargo clippy --workspace --all-targets
cargo test --workspace
```

Decisión pendiente para cuando exista: activar `-D warnings` desde el primer
commit (no hay avisos heredados que lo impidan).

## Previsto, no existe todavía

- `Cargo.toml` del workspace, los cinco crates, `src-tauri/src/`,
  `tauri.conf.json`, `capabilities/`.
- `docs/CONTRATO.md` §5 (comandos) y §6 (canales) están vacías: se rellenan al
  implementar.
- Dónde se guardan los ajustes (`latencia_ms`, dispositivos): sin decidir.
- Ruta del Python de la fábrica en la app instalada (fase 5). En desarrollo:
  `fabrica/.venv/Scripts/python.exe -m karaoke_fabrica`.
- `scripts/check-rules.mjs` (reglas ejecutables: `unwrap` en runtime, comando
  registrado y con envoltorio).
- Fase 4: verificación de palabras en directo (sherpa-onnx + Nemotron 3.5);
  la palabra da un bono, nunca resta.

## Discrepancias abiertas

Ninguna (no hay código). Se anotan aquí cuando el código y `CONTRATO.md`
discrepen; decide el usuario.

## Mantenimiento

Este archivo se actualiza en el mismo cambio que el código que describe.
Re-ejecuta `bind-project` cuando aparezca el workspace, entre un crate, cambie
el número de comandos o los comandos de verificación.
