---
name: backend
description: >
  Backend Rust del karaoke (workspace Cargo en la raíz del repo: crates/ y
  src-tauri/, Tauri 2, cpal). Úsala al crear o cambiar los crates
  karaoke-core (tipos serde de cancion.json y su lectura), tono (detector MPM
  del micrófono), puntuacion (puntuación por nota y por línea), audio (cpal:
  reproducción con reloj por muestras, captura del micrófono, calibración de
  latencia_ms) o biblioteca (carpetas de canciones en app_data_dir,
  records.json); un comando #[tauri::command] de src-tauri/src/comandos, el
  registro en generate_handler!, el estado de src-tauri, el lanzamiento de la
  fábrica Python (proceso aparte, protocolo de líneas JSON por stdout), el
  bucle del juego que manda frames por tauri::ipc::Channel a ~60 Hz, un error
  devuelto como String "No se pudo…", los tests de crates/*/tests, Cargo.toml
  del workspace, tauri.conf.json o capabilities; o cuando docs/CONTRATO.md y
  el código discrepen. Qué aplica a este repo está en references/project.md,
  que se lee primero y manda sobre esta doctrina.
---

# Backend — Rust + Tauri 2 (workspace de crates, audio en tiempo real)

Doctrina portable para un backend de escritorio en Rust dividido en crates:
crates de dominio puros (sin `tauri`) y un crate Tauri que solo los expone
como comandos y canales. No sabe cuáles de esas reglas usa este repo: eso lo
dice `references/project.md`.

## Precedencia

`código > docs/CONTRATO.md > references/project.md > doctrina`. Si la doctrina
y `project.md` discrepan, manda `project.md`. Si `project.md` y el código
discrepan, manda el código y arreglas `project.md` en el mismo cambio. Si
`docs/CONTRATO.md` y el código discrepan, se anota en `project.md` →
Discrepancias y decide el usuario.

## Cómo usar esta skill

1. **Lee `references/project.md` primero.** Trae el stack, el mapa de crates,
   la tabla **Aplica / no aplica**, lo que está previsto y todavía no existe, y
   los comandos de verificación. Si contradice el código, Workflow G antes.
2. Abre solo los temas que la tabla marca como aplicables. Una sección cuya
   condición es falsa se salta, no se "adapta".
3. Elige un workflow; abre solo las referencias que nombra.
4. Antes de terminar, lee `references/pitfalls.md` y `references/project-pitfalls.md`.
   Un cambio que reintroduce un error listado no está terminado.
5. Valida con los comandos que declara `project.md` — ver **Validación**.

## Índice de referencias

| Referencia | Cubre | Léela cuando |
|---|---|---|
| `references/project.md` | Stack, mapa de crates, aplica/no aplica, previsto, invariantes, verificación | Siempre, primero |
| `references/architecture.md` | Capas: crates de dominio → `src-tauri` como pegamento; regla de dependencia; dónde va cada cosa; nombres | Siempre |
| `references/idioms.md` | Préstamo, `Option` con combinadores, enums serde, `f64` de tiempo y MIDI, visibilidad | Escribir cualquier función nueva; revisión final |
| `references/commands.md` | Forma de un `#[tauri::command]`, `Result<T, String>`, registro, `rename_all`, comandos que abren un `Channel` | Tocar `src-tauri/src/comandos/` o `lib.rs` |
| `references/errors.md` | `anyhow` dentro de los crates, `String` "No se pudo…" en la frontera, sin `unwrap` | Cualquier función que devuelva `Result` |
| `references/state-and-threads.md` | Estado de `src-tauri`, alcance de los locks, hilo de audio sin bloqueos, hilo del juego, proceso de la fábrica | Estado compartido, audio, trabajo de más de ~100 ms |
| `references/events.md` | `tauri::ipc::Channel` para el progreso de la fábrica y los frames del juego; cuándo un evento | Mandar datos de Rust a la interfaz sin que los pida |
| `references/persistence.md` | Carpeta de canción, `cancion.json` solo lectura en Rust, `records.json` con escritura atómica | Tocar `crates/biblioteca` o `crates/karaoke-core` |
| `references/testing.md` | Unitarios de lógica pura (tono, puntuación), integración con carpetas temporales, fixtures de `cancion.json` | Cualquier cambio en un crate |
| `references/pitfalls.md` | Trampas universales de Rust + Tauri + audio en tiempo real | Revisión final de cualquier cambio |
| `references/project-pitfalls.md` | El libro de errores fechado de ESTE repo | Revisión final de cualquier cambio |

## Workflow A — Nuevo comando Tauri

1. Confirma que el crate de dominio ya tiene la función. Si no, Workflow B
   primero: un comando no contiene lógica de dominio.
2. Escribe el comando en su módulo de `src-tauri/src/comandos/`, fino: tomar
   el estado → una llamada al crate → `.map_err(|e| e.to_string())` —
   `commands.md`.
3. Si tarda más de ~100 ms o produce datos continuos, decide el hilo y el
   canal — `state-and-threads.md`, `events.md`.
4. Regístralo en `tauri::generate_handler![...]` de `src-tauri/src/lib.rs`.
5. **Mismo cambio**: fila en `docs/CONTRATO.md` §5 (o §6 si abre un canal),
   envoltorio en `src/common/api/comandos.ts`, interfaz en
   `src/common/interfaces/` con los MISMOS nombres `snake_case` en español —
   `.claude/rules/contrato.md`.
6. Test de la función del crate — `testing.md`. Verifica.

## Workflow B — Lógica nueva en un crate de dominio

1. Elige el crate por lo que posee (`architecture.md` → Dónde va cada cosa).
   `tono` y `puntuacion` son puros: sin `cpal`, sin archivos, sin hilos.
2. Tipos que cruzan a la interfaz: `#[derive(Serialize)]`, campos en español
   `snake_case`, tiempos en segundos `f64`, tono en MIDI `f64`,
   `Option` = `null` (CONTRATO §1).
3. Test unitario en `#[cfg(test)] mod pruebas` o integración en
   `crates/{crate}/tests/` — `testing.md`.

## Workflow C — Audio en tiempo real (`crates/audio`, bucle del juego)

1. Lee `state-and-threads.md` → Hilo de audio. El callback de `cpal` no
   reserva memoria, no toma `Mutex`, no escribe en consola ni en disco.
2. El reloj es el número de muestras que el dispositivo de salida ya consumió.
   Cada bloque del micrófono lleva su marca en muestras del dispositivo de
   entrada. `latencia_ms` se resta al tiempo del micrófono (ARQUITECTURA §4).
3. Tono y puntuación se calculan por **frame de audio de 10 ms**, nunca por
   frame de pantalla — `project-pitfalls.md`.
4. A la interfaz va un frame cada ~16 ms por `Channel` con `t`, tono,
   claridad y puntuación — `events.md`.

## Workflow D — Lanzar la fábrica

1. `std::process::Command` con `stdout` y `stderr` en `piped`, sin ventana de
   consola (`CREATE_NO_WINDOW` en Windows).
2. Un hilo lee `stdout` línea a línea; cada línea es un objeto JSON de
   CONTRATO §4 (`etapa`, `aviso`, `listo`, `error`) y se reenvía tal cual por
   el `Channel` del comando. `stderr` va al registro, no a la interfaz.
3. Una línea que no parsea es un error de la fábrica, no se ignora.
   Código de salida ≠ 0 sin línea `error` = "No se pudo procesar la canción".
4. La canción está lista solo cuando existe `cancion.json` (CONTRATO §2).

## Workflow E — Cambio del contrato (`cancion.json`, comando, canal)

*El cambio más caro del repo: cruza fábrica, crates e interfaz.*

1. El cambio empieza en `docs/CONTRATO.md` y lo aprueba el usuario.
2. Mismo cambio: `crates/karaoke-core` (struct serde), `src/common/interfaces/`,
   `fabrica/karaoke_fabrica/modelo.py`, y `version` de `cancion.json` si se
   rompe la lectura de canciones ya procesadas.
3. Campo nuevo opcional en lectura: `#[serde(default)]` para que las canciones
   viejas sigan abriendo.

## Workflow F — Persistencia (`crates/biblioteca`)

`persistence.md`. Rust nunca escribe `cancion.json` ni los `.wav`; escribe
`records.json` con tmp + rename.

## Workflow G — Vincular a este proyecto

Cuando `references/project.md` contradice el código o entra un crate nuevo:
skill hermana **`bind-project`** (`../bind-project/`). No programes contra una
doctrina sin vincular.

## Validación

Los comandos que declara `project.md`, desde la raíz del repo. Hoy no existen
(no hay `Cargo.toml`); cuando existan: `cargo clippy --workspace --all-targets`,
`cargo test --workspace`, y `npm run verify`, que lo encadena todo. Nunca
sustituyas un comando por una checklist en prosa ni reportes una comprobación
que no ejecutaste. Reporta `clippy / cargo test / verify → LISTO o NO LISTO`
con el comando que falló. Lo que ningún comando comprueba: que el contrato
cambió en todos sus sitios a la vez, que el callback de audio sigue sin
reservas ni locks, y que la puntuación sigue en tiempo de audio.
