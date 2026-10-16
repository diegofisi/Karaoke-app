---
paths:
  - '**'
---

# Verificación

Qué tiene que estar verde antes de reportar una tarea como hecha. Siempre
activa; los detalles están en `references/project.md` de cada skill.

- **Un comando, desde la raíz: `npm run verify`.** Encadenará con `&&`:
  `tsc -b` → `check:rules` → `vite build` →
  `cargo clippy --workspace --all-targets` → `cargo test --workspace` →
  pytest de la fábrica. Nunca con `;`: un paso que falla seguido de uno que
  pasa se lee verde.
- **Hoy `verify` no existe** (no hay `package.json`, ni `Cargo.toml`, ni
  `src/`). Mientras tanto, se ejecuta lo que exista:
  - fábrica: `.venv\Scripts\python -m pytest -q` desde `fabrica/`;
  - Rust, cuando haya workspace: `cargo clippy --workspace --all-targets` y
    `cargo test --workspace`;
  - interfaz, cuando haya `tsconfig.json`: `npx tsc -b` y `npx vite build`.
  Di en el reporte qué pasos no existían.
- **Ejecútalo, no lo razones.** Un comando que no se ejecutó no pasó. Cita el
  comando, el directorio y su código de salida.
- **Lo que `verify` no comprueba lo compruebas tú y lo dices**: el contrato
  cambiado en todos sus sitios (`rules/contrato.md`), el callback de audio
  sin reservas ni locks, la puntuación en tiempo de audio, los datos del
  juego fuera del estado de React, `stdout` de la fábrica limpio, y la prueba
  manual en `npm run tauri dev`, que hace el usuario.
- **Cambio en alineación o tono de la fábrica = número de `eval/` antes y
  después.** Sin número, no está verificado.
- **Código nuevo lleva test o motivo.** Lógica pura en Rust: `#[cfg(test)]`.
  Fábrica: `fabrica/tests/`. Interfaz sin runner: una línea diciendo cómo se
  verificó.
- **Nunca hagas pasar la comprobación debilitándola.** Ni `#[allow(...)]`
  nuevo, ni `// @ts-expect-error`, ni relajar `tsconfig` (el hook
  no se toca aunque el hook `protect-config` esté desactivado), ni saltar un test, ni `pytest -k` para
  esquivar uno. Arregla el código; si la comprobación está mal, dilo y para.
- **El implementador no firma la implementación.** Más de un archivo, o
  cualquier toque al contrato: pásalo al agente `verifier` con la lista de
  archivos (o `git diff`) y reporta su veredicto.
- **Los hooks de Claude están desactivados** (2026-10-16, decisión del usuario):
  nada comprueba solo al terminar; `npm run verify` es la única puerta.
- **Tres intentos.** Si el mismo fallo sobrevive a tres arreglos, para y
  escala con el historial.
- **No lances `npm run dev`, `npm run tauri dev` ni `vite`**: no terminan.
  `settings.json` los deniega. Tampoco `python -m karaoke_fabrica procesar`
  sin que se pida: descarga modelos y ocupa la GPU minutos.
