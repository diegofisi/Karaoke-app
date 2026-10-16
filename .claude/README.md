# `.claude/` del karaoke

Todo lo que Claude Code lee para trabajar en este repo. Abre Claude Code
**desde la raíz del repo** (`Karaoke/`): los hooks y los permisos se resuelven
contra `$CLAUDE_PROJECT_DIR`, y la raíz es a la vez la de git, Cargo, npm y
Tauri. Copiado y adaptado de `InglesApp/.claude` el 2026-10-15.

## Qué hay

```
CLAUDE.md (raíz)          punteros: carpetas, fuentes de verdad, tabla glob → skill, verify, git
.claude/
  settings.json           sin atribución a IA; permisos; deny de dev/vite/push --force/--no-verify; hooks
  settings.local.json     permisos locales del usuario; no se versiona
  corrections.md          memoria entre sesiones: una línea fechada por corrección repetida
  hooks/                  DESACTIVADOS por el usuario (2026-10-16): codigo comentado y sin conexion en settings.json
    protect-config.mjs    PreToolUse Edit|Write: bloquea editar tsconfig/vite/clippy/rustfmt/eslint/prettier/pytest.ini/ruff existentes (escape: KARAOKE_ALLOW_CONFIG=1)
    block-git-bypass.mjs  PreToolUse Bash: bloquea --no-verify / -n / core.hooksPath en git
    check-edited.mjs      PostToolUse Edit|Write: pasa scripts/check-rules.mjs por el archivo escrito y BLOQUEA si rompe una regla (hoy el script no existe: no hace nada)
    stop-check.mjs        Stop: tsc -b y cargo check sobre lo cambiado según git status, si existen tsconfig.json y Cargo.toml; solo informa
  agents/
    verifier.md           revisor sin Write/Edit; postura RECHAZAR; cero hallazgos es válido; carriles Rust, TS y Python
    build-fixer.md        arregla errores de tsc/cargo/pytest con cambios mínimos; para a los 3 intentos
  rules/
    git.md, verification.md   siempre activas
    contrato.md               al tocar src-tauri/src, crates/*/src/lib.rs, src/common/api, src/common/interfaces, modelo.py, docs/CONTRATO.md
  skills/
    backend/    SKILL.md + references/   Rust + Tauri 2 + cpal (crates y src-tauri)
    frontend/   SKILL.md + references/   React 19 + MUI + React Query + Zustand + canvas sobre Tauri
    fabrica/    SKILL.md + references/   Python: modelos, cancion.json, protocolo stdout, eval/
    bind-project/SKILL.md              re-vincula las skills cuando aparece o cambia el código
```

## Las dos capas de cada skill

| Capa | Archivos | Cambia |
|---|---|---|
| Doctrina | `SKILL.md`, `references/*.md` salvo los dos de abajo | rara vez; es verdad para cualquier app de este stack |
| Vinculación | `references/project.md`, `references/project-pitfalls.md` | con el repo: `project.md` cuando cambia el stack, `project-pitfalls.md` con cada bug pagado |

Precedencia: **código > `docs/CONTRATO.md` > `project.md` > doctrina**. Si el
código y `project.md` discrepan, manda el código y se arregla el archivo en
el mismo cambio. Si `CONTRATO.md` choca con el código, decide el usuario.

Hoy (2026-10-15) solo existe la fábrica. Los `project.md` de `backend` y
`frontend` describen lo previsto en `docs/ARQUITECTURA.md`, con una sección
"Previsto, no existe todavía". Cuando aparezca el código, se ejecuta
`bind-project`.

## Skill, regla, hook o agente

| Si lo que quieres es… | Es un… |
|---|---|
| Explicar cómo se hace algo en este stack | Skill (o una referencia nueva dentro de una skill) |
| Una regla corta que debe estar presente al tocar ciertas rutas | `rules/*.md` con `paths:` |
| Algo que un script puede comprobar | Un paso de `npm run verify` o una regla de `scripts/check-rules.mjs` |
| Algo que un script puede **impedir** | Un hook `PreToolUse` que sale con 2 |
| Un trabajo con otra postura y otras herramientas | Un agente en `agents/` |
| Una corrección que ya has hecho dos veces | Una línea en `corrections.md` |
| Un bug ya pagado en el código | Una línea en el `project-pitfalls.md` de la skill |

## Lo que todavía no existe

- `package.json` y `npm run verify` (encadenará `tsc -b`, `check:rules`,
  `vite build`, `cargo clippy`, `cargo test` y pytest de la fábrica).
- `scripts/check-rules.mjs` con el escape `// permitido: <razón>`.
- Hooks de git (`rules/git.md` dice el reparto previsto).

## Higiene

- El implementador no valida su propio trabajo: `verifier`.
- Tope de tres intentos sobre el mismo fallo; después, escalar.
- Un arreglo por tarea; sin refactors ni formateos de paso.
- `npm run dev`, `npm run tauri dev` y `vite` los lanza el usuario.
