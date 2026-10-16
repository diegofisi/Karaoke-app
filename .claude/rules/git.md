---
paths:
  - '**'
---

# Git

## Estado de hoy (2026-10-15)

- La raíz del repo es `Karaoke/`. Remoto `origin` en GitHub
  (`diegofisi/karaoke-gmi2`). Rama por defecto: `master`.
- La rama **`legado-python`** guarda la versión anterior (FastAPI + React en
  el navegador). Es de referencia: no se commitea en ella ni se borra.
- `master` está en plena reescritura: la versión anterior se está quitando y
  entran `docs/`, `fabrica/`, `eval/`, y después `crates/`, `src-tauri/`,
  `src/`.
- No hay hooks de git todavía.

## Qué no entra nunca en el historial

`.claude/settings.local.json`, `fabrica/.venv/`, `venv/`, `target/`,
`node_modules/`, `dist/`, `eval/datos/` (audio de JamendoLyrics), los `.wav`,
`.mp3` y modelos (`*.pt`, `*.pth`, `*.bin`, `*.ckpt`, `*.onnx`), `.env`,
`*.pfx`, `*.pem`. `.gitignore` cubre la mayoría; si un `git status` enseña
uno de estos como nuevo, para y avisa antes de añadirlo.

`eval/resultados/*.json` **sí** entra: es el registro de las medidas.

## Commits

- **Sin commits ni pushes si no se piden.** Terminar la tarea no incluye
  committear. Pedir un commit no incluye un push.
- **Sin atribución a IA.** Ni `Co-Authored-By: Claude`, ni "Generated with
  Claude", ni nada parecido en mensajes, PRs o archivos. Lo aplican
  `.claude/settings.json` (`includeCoAuthoredBy: false`, `attribution`
  vacío) y `~/.claude/CLAUDE.md`. Tras committear: `git log -1 --format=%B`
  y `amend` si se coló.
- **Mensajes en español, `Área: qué cambió`**: `Fábrica: alinear línea a
  línea con desfase global`, `Juego: puntuar por frames de 10 ms`. Áreas
  habituales: `Fábrica`, `Eval`, `Core`, `Tono`, `Puntuación`, `Audio`,
  `Biblioteca`, `Tauri`, `Interfaz`, `Docs`, `Claude`. Máximo 72 caracteres
  el asunto; lo demás al cuerpo tras una línea en blanco.
- **Un cambio lógico por commit.** Un `cargo fmt` del repo entero es su propio
  commit. Un cambio de contrato lleva todos sus sitios en el mismo commit.
- Un cambio en alineación o tono cita en el cuerpo los números de `eval/`
  antes y después.

## Ramas

- **Propón una rama** (`area/que-hace`, en español) para cualquier cambio que
  no sea trivial, y créala **antes** de committear si el usuario pide un
  commit estando en `master`.
- **Nunca reescribas historia compartida**: sin `push --force`
  (`settings.json` lo deniega), sin `reset --hard` sobre trabajo ajeno.
- No toques `legado-python`.

## Hooks de git: pendientes

No existen. No los crees por tu cuenta. Reparto previsto cuando el usuario
los pida:

| Hook | Qué haría |
|---|---|
| `pre-commit` | `node scripts/check-rules.mjs --staged` y `tsc -b` |
| `commit-msg` | rechazar trailers de atribución y asuntos de más de 72 caracteres |
| `pre-push` | `npm run verify` |

`.claude/hooks/block-git-bypass.mjs` ya impide al agente saltárselos con
`--no-verify`, `-n` o `-c core.hooksPath=`.
