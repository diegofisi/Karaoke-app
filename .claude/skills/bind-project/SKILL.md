---
name: bind-project
description: >
  Vincula las skills backend (Rust, Tauri 2, cpal), frontend (React 19, MUI,
  React Query, Zustand, invoke y Channel) y fabrica (Python, modelos) del
  karaoke a la realidad del repo: inspecciona Cargo.toml del workspace y de
  crates/karaoke-core, tono, puntuacion, audio, biblioteca y src-tauri,
  package.json, tsconfig, tauri.conf.json, capabilities, el árbol de src/,
  src-tauri/src/comandos/, fabrica/pyproject.toml, fabrica/karaoke_fabrica/,
  eval/, docs/CONTRATO.md y la cuenta de comandos de Tauri; pregunta solo lo
  que no se puede deducir; y regenera references/project.md de cada skill,
  las reglas por ruta, el script verify y los punteros de CLAUDE.md. Úsala
  cuando aparezca el código previsto (workspace Cargo, package.json, src/),
  cuando entre un crate, un módulo de la fábrica, vitest, eslint o un router,
  cuando cambie la versión de Tauri, o cuando un project.md contradiga el
  código.
---

# Vincular las skills a este repo

Las skills `backend`, `frontend` y `fabrica` de `.claude/skills/` describen
doctrina. El archivo `references/project.md` de cada una es la capa que solo
es verdad en este repo. Esta skill escribe o actualiza esa capa a partir de lo
que hay. La salida son archivos, no conversación.

Nunca inventes un hecho sobre el stack. Todo lo generado sale del repo o de
una respuesta del usuario.

## Rutas de este repo

| Qué | Ruta |
|---|---|
| Raíz (git, workspace Cargo, `package.json`, Tauri) | `C:\Users\USER\Desktop\Karaoke\` |
| Crates Rust | `crates\{karaoke-core,tono,puntuacion,audio,biblioteca}\` |
| Pegamento Tauri | `src-tauri\` |
| Interfaz | `src\` |
| Fábrica Python | `fabrica\` (venv en `fabrica\.venv`) |
| Medida | `eval\` |
| Configuración de Claude | `.claude\` (`skills/`, `rules/`, `hooks/`, `agents/`, `settings.json`, `corrections.md`) |
| Punteros | `CLAUDE.md` |
| Contrato y arquitectura | `docs\CONTRATO.md`, `docs\ARQUITECTURA.md` |

Todos los comandos se ejecutan desde la raíz, salvo pytest (desde `fabrica\`).
`venv\` en la raíz es de la versión anterior: no se inspecciona.

## A. Inspecciona antes de preguntar

Una pasada, no más de 25 lecturas:

- `Cargo.toml` raíz: `members`, `[workspace.dependencies]`, `resolver`.
- `crates/*/Cargo.toml` y `src-tauri/Cargo.toml`: versión de `tauri`, `cpal`,
  ring buffer, `[lib] name`, plugins de Tauri.
- `src-tauri/tauri.conf.json` y `src-tauri/capabilities/*.json`.
- `package.json`: dependencias (`react`, `@mui/material`, `zustand`,
  `@tanstack/react-query`, `@tauri-apps/api`, ¿router?), devDependencies
  (¿`vitest`? ¿`eslint`?), scripts (¿`verify`?).
- `tsconfig.json` (y referenciados), `vite.config.ts`.
- Árbol de `src/`: `common/{api,components,interfaces,stores,theme}`,
  `features/*`.
- Árbol de `src-tauri/src/` y `crates/*/src/`, `crates/*/tests/`.
- Cuenta de comandos. El atributo lleva argumentos, así que se busca el
  prefijo: `grep -rn "#\[tauri::command" src-tauri/src | wc -l`. Comparar con
  `generate_handler!` en `lib.rs`, con los envoltorios de
  `src/common/api/comandos.ts` y con `CONTRATO.md` §5. Las cuatro deben
  coincidir.
- `fabrica/pyproject.toml`, `fabrica/karaoke_fabrica/*.py`,
  `fabrica/tests/`, `eval/`.
- `docs/CONTRATO.md` (§3 a §6) y `docs/ARQUITECTURA.md` §6 (fase actual).
- `CLAUDE.md`, `.claude/` existente.
- Si existen: `npx tsc -b`, `cargo clippy --workspace --all-targets`,
  `cargo fmt --check`, pytest: cuántos errores, avisos y diffs hay hoy.

Resume la detección en no más de 12 líneas antes del paso B.

## B. Pregunta solo lo que no se deduce

Una ronda, máximo cuatro preguntas, elegidas de esta lista:

1. ¿Entra `vitest` ahora? Si entra, `verify` lo incluye.
2. ¿`cargo clippy -- -D warnings` desde ya? Por defecto: sí si A contó cero
   avisos.
3. Router: ¿react-router o navegación por estado?
4. Dónde se guardan los ajustes (`latencia_ms`, dispositivos), si el código
   ya los necesita y `CONTRATO.md` no lo dice.

No preguntar el idioma: `CONTRATO.md` §1 lo fija (español en nombres, en
interfaz y en documentación).

## C. Genera, en este orden

1. `references/project.md` de `backend`, `frontend` y `fabrica`. Tabla
   **Aplica / no aplica** entera; cada "no" con motivo. Lo que ARQUITECTURA
   prevé y no existe va en "Previsto, no existe todavía", no en la tabla
   principal. Borra de "Previsto" lo que ya existe.
2. `project-pitfalls.md` de cada skill: **no se toca** si existe.
3. La `description` del frontmatter de cada `SKILL.md` con las palabras
   reales del repo (crates, módulos, features, comandos).
4. `.claude/rules/*.md`: cada `paths:` apunta a globs que existan o que
   ARQUITECTURA prevé de forma explícita.
5. `.claude/settings.json`: fusiona, nunca sobrescribe.
   `includeCoAuthoredBy: false` siempre; `deny` de `npm run dev`,
   `tauri dev`, `vite`, `push --force`, `commit --no-verify`.
6. `package.json`: script `verify` que encadena con `&&`: `tsc -b`,
   `check:rules` (si existe `scripts/check-rules.mjs`), `vite build`,
   `cargo clippy --workspace --all-targets`, `cargo test --workspace`, pytest
   de la fábrica. Nunca con `;`.
7. `CLAUDE.md`: actualiza la sección que cambie; no reescribas el resto.

## D. Puerta de aprobación

Tras C.1, **para**. Muestra los `project.md` completos y pide aprobación.
Nada de C.2 a C.7 se escribe hasta recibirla.

## E. Verifica

`npm run verify` desde la raíz (si existe). Reporta el comando, el código de
salida y el primer paso que falló. No arregles fallos preexistentes como
parte de la vinculación. Comprueba también:

- Cada `paths:` de `.claude/rules/*.md` casa con al menos un archivo real.
- `.claude/settings.json` parsea como JSON.
- Las cuatro cuentas de comandos del paso A coinciden.

## Re-vinculación

Cuándo: aparece el workspace o `package.json`; entra un crate, una feature o
un módulo de la fábrica; entra vitest, eslint o un router; cambia la versión
mayor de `tauri` o `cpal`; un `project.md` contradice el código.

Qué: repite A, muestra solo las filas que cambian, reescribe `project.md` y
las reglas cuyos `paths:` ya no casan. **Nunca toques `project-pitfalls.md`
ni `corrections.md`.** Son historia.

## Hecho cuando

- Los tres `project.md` existen, aprobados, sin celdas vacías.
- Cada `paths:` de `.claude/rules/` casa con al menos un archivo real o
  previsto.
- `settings.json` parsea y lleva `includeCoAuthoredBy: false`.
- `package.json` tiene `verify` (cuando exista `package.json`).
- `CLAUDE.md` tiene los punteros y nada de lo que ya tenía se borró.
