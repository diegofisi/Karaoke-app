# Karaoke

Reescritura completa (2026-10-15). La versión anterior está en la rama
`legado-python`. El porqué de cada decisión: `docs/REVISION-2026-10-15.md`.

## Fuentes de verdad

- `docs/CONTRATO.md` — manda sobre todo: nombres, `cancion.json`, protocolo de
  la fábrica, comandos y canales. Un cambio de contrato lo aprueba el usuario y
  se hace en todos sus sitios a la vez (`.claude/rules/contrato.md`).
- `docs/ARQUITECTURA.md` — las dos mitades, carpetas, tiempo, fases.
- `.claude/corrections.md` — correcciones repetidas. Léelo al empezar.

## Carpetas

| Carpeta | Qué es | Estado |
|---|---|---|
| `fabrica/` | Python: descarga, separa, alinea, saca el tono, escribe `cancion.json`. Venv en `fabrica/.venv` | en curso |
| `eval/` | Medida del tiempo por palabra contra marcas a mano | en curso |
| `crates/` | Rust: `karaoke-core`, `tono`, `puntuacion`, `audio`, `biblioteca` | previsto |
| `src-tauri/` | Pegamento Tauri: comandos, estado, lanza la fábrica, bucle del juego | previsto |
| `src/` | React 19 + MUI + React Query + Zustand. Solo pinta | previsto |
| `docs/` | Contrato, arquitectura, revisiones | — |
| `venv/` | Python de la versión anterior. No se usa | — |

La raíz es a la vez raíz de git, del workspace Cargo, del `package.json` y de
Tauri. Abre Claude Code desde aquí.

## Qué skill leer

| Si tocas… | Skill |
|---|---|
| `crates/**`, `src-tauri/**`, `Cargo.toml` | `.claude/skills/backend/` |
| `src/**`, `package.json`, `vite.config.ts`, `tsconfig*.json` | `.claude/skills/frontend/` |
| `fabrica/**`, `eval/**` | `.claude/skills/fabrica/` |
| El stack cambió o aparece código nuevo | `.claude/skills/bind-project/` |

Cada skill: lee `references/project.md` primero; manda sobre la doctrina.

## Verificación

`npm run verify` desde la raíz: `tsc -b && check:rules && vite build && cargo
clippy --workspace --all-targets && cargo test --workspace && pytest de la
fábrica`. **Todavía no existe.** Mientras tanto, lo que exista
(`.claude/rules/verification.md`); hoy, desde `fabrica/`:
`.venv\Scripts\python -m pytest -q`. Un cambio en alineación o tono se mide
con `eval/` antes y después. No lances `npm run dev`, `tauri dev` ni `vite`.

## Git

- Sin commits ni pushes si no se piden.
- Sin atribución a IA en commits, PRs ni archivos.
- Mensajes en español: `Área: qué cambió`, asunto de 72 caracteres como máximo.
- Sin `push --force` ni `--no-verify`. No tocar `legado-python`.
- Detalle: `.claude/rules/git.md`.

## Estilo

Español en código, comentarios, documentación e interfaz; identificadores sin
tildes. Lenguaje directo, sin metáforas.
