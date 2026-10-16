---
paths:
  - 'src-tauri/src/**'
  - 'crates/*/src/lib.rs'
  - 'src/common/api/**'
  - 'src/common/interfaces/**'
  - 'fabrica/karaoke_fabrica/modelo.py'
  - 'docs/CONTRATO.md'
---

# Contrato fábrica → disco → Rust → interfaz

`docs/CONTRATO.md` manda. Si algo aquí choca con él, gana él. Estos archivos
son una sola cosa en varios trozos: tocar uno casi siempre implica tocar los
demás **en el mismo cambio**.

| Cambio | Archivos que cambian juntos |
|---|---|
| Campo de `cancion.json` (nuevo, renombrado, tipo) | `docs/CONTRATO.md` §3 + `fabrica/karaoke_fabrica/modelo.py` + `crates/karaoke-core/src/lib.rs` + `src/common/interfaces/` + fixture de test en `crates/karaoke-core/tests/` y en `fabrica/tests/`. `version` sube si las canciones ya procesadas dejan de leerse |
| Mensaje del protocolo de la fábrica | `docs/CONTRATO.md` §4 + `fabrica/karaoke_fabrica/eventos.py` + el lector en `src-tauri` + el tipo del mensaje en `src/common/interfaces/` |
| Comando nuevo | fn con `#[tauri::command(rename_all = "snake_case")]` en `src-tauri/src/comandos/` + `generate_handler!` en `src-tauri/src/lib.rs` + `CONTRATO.md` §5 + envoltorio en `src/common/api/comandos.ts` + hook en `consultas.ts` + `interfaces/` si el tipo es nuevo |
| Canal nuevo o frame cambiado | struct `Serialize` en `src-tauri` + `CONTRATO.md` §6 + envoltorio con `Channel` en `comandos.ts` + tipo en `interfaces/` |
| Argumento renombrado | la fn del comando + `comandos.ts` + `CONTRATO.md` |

Reglas:

- **Los cambios de contrato los aprueba el usuario** antes de tocar código.
  Si el código y `CONTRATO.md` discrepan, se anota en
  `.claude/skills/backend/references/project.md` → Discrepancias y decide él.
- **Nombres en español, sin tildes, `snake_case`**, idénticos en JSON, Rust
  (`rename_all = "snake_case"`), TypeScript y Python. Nadie traduce nombres.
- **Tiempos en segundos `f64`; tono en MIDI `f64`; `null` = sin voz**
  (CONTRATO §1). `Option<T>` en Rust = `T | null` en TypeScript =
  `T | None` en Python.
- **El struct de `crates/karaoke-core` es la fuente de verdad en Rust**; los
  demás crates y `src-tauri` lo importan, no lo copian.
- **Todo comando devuelve `Result<T, String>`** con un mensaje en español que
  empieza por "No se pudo…".
- **`comandos.ts` es el único archivo con `invoke` y `new Channel`.**
- **`cancion.json` solo lo escribe la fábrica**, al final y de forma
  atómica. Rust y la interfaz solo lo leen.
- **`stdout` de la fábrica = solo líneas JSON.**
- Sin `unwrap()` en un comando; sin `std::thread::sleep`; ningún lock
  sostenido durante I/O.
