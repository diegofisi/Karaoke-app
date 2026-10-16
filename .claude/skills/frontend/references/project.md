# Vinculación — karaoke, interfaz (src/)

> Este archivo es la vinculación de la doctrina portable a ESTE repo. Se lee
> **primero** y **manda** donde discrepe de la doctrina. Donde discrepe del
> código, manda el código y arreglar este archivo es parte del cambio. Donde
> discrepe de `docs/CONTRATO.md`, se anota y se avisa al usuario.

**Estado (2026-10-15):** la interfaz **no existe todavía**. No hay
`package.json`, ni `src/`, ni `tsconfig.json`. La versión anterior (React en
el navegador, `frontend/`) está en la rama `legado-python` y no se reutiliza.
Este archivo describe lo **previsto** en `docs/ARQUITECTURA.md` (fase 3).
Cuando el código exista, se re-ejecuta `bind-project`.

Todo se da desde la raíz del repo, `C:\Users\USER\Desktop\Karaoke\`: ahí
estarán `package.json`, `tsconfig.json`, `vite.config.ts`, `index.html` y
`src/`. No hay subcarpeta de app.

## Qué es

La interfaz del karaoke. **Solo pinta** (ARQUITECTURA §1): no toca audio, no
detecta tono, no puntúa. Recibe de Rust la lista de canciones, `cancion.json`,
el progreso de la fábrica y, durante el juego, un frame cada ~16 ms con `t`,
tono del micrófono, claridad y puntuación.

Fuentes de verdad que este archivo **no** reescribe:

- `docs/CONTRATO.md` — nombres, tipos, `cancion.json`, protocolo de la
  fábrica, comandos (§5) y canales (§6). Manda sobre este archivo.
- `src-tauri/src/lib.rs` (`generate_handler!`) — los comandos que existen.
- `crates/karaoke-core` — los `struct` de `cancion.json`;
  `src/common/interfaces/` es su espejo.

## Stack previsto

| Capa | Previsto | Notas |
|---|---|---|
| Framework | React 19, `StrictMode` | — |
| Lenguaje | TypeScript `strict` | alias `@/` → `src/` |
| Build | Vite | `tsc -b && vite build` |
| Kit de UI | MUI + Emotion | sin Tailwind, sin librería de iconos salvo decisión |
| Datos | React Query (`@tanstack/react-query`) | `common/api/queryClient.ts` |
| Estado de interfaz | Zustand | stores en `common/stores/` y en la feature `juego` |
| Router | **por decidir** | ARQUITECTURA no lo nombra |
| Transporte | `@tauri-apps/api` 2: `invoke` y `Channel` | solo en `common/api/comandos.ts` |
| Dibujo del juego | `<canvas>` 2D con `requestAnimationFrame` | fuera de los re-renders de React |
| Formularios | `useState` + campos de MUI | sin react-hook-form, sin zod |
| i18n | ninguna | texto visible en español inline |
| Tests | por decidir (Vitest) | `testing.md` |

Ninguna versión está fijada. Se anotan aquí cuando exista `package-lock.json`.

## Aplica / no aplica

| Tema | Archivo | ¿Aplica? | Motivo |
|---|---|---|---|
| Slices verticales | `architecture.md` | **Sí** | `src/features/{biblioteca,procesar,juego,resultados,ajustes}/` |
| Container / presentacional | `containers-pages.md` | **Sí** | `juego` y `procesar` orquestan por fases |
| Adaptador DTO → mapper → Model | `data-flow.md` | **No** | CONTRATO §1: la interfaz no traduce nombres |
| React Query | `data-flow.md` | **Sí** | lista de canciones, canción, récords, ajustes |
| `api/` por feature | `architecture.md` | **No** | centralizado en `common/api/` |
| `invoke` | `tauri-transport.md` | **Sí** | solo en `comandos.ts` |
| `Channel` (flujos) | `tauri-transport.md`, `state.md` | **Sí** | progreso de la fábrica y frames del juego (CONTRATO §6) |
| Eventos `listen` | `tauri-transport.md` | **Casi no** | los flujos van por `Channel` |
| Canvas fuera de React | `components.md`, `state.md` | **Sí** | feature `juego` |
| Modo mock | `tauri-transport.md` | **No** | sin rama mock |
| Kit de UI | `components.md` | **Sí (MUI)** | paleta por definir |
| Librería de formularios | `forms.md` | **No** | `useState` + MUI |
| i18n / ayuda en otro idioma | `conventions.md` | **No** | toda la interfaz en español; no hay `<Ayuda>` |
| Audio en la interfaz (`<audio>`, Web Audio, `getUserMedia`) | — | **No** | el audio es de Rust (`crates/audio`) |
| Auth / HTTP | — | **No** | app local |
| Tests | `testing.md` | **No todavía** | sin runner |

## Mapa de features previsto (ARQUITECTURA §6, fase 3)

| Feature | Qué hace | Datos de (previsto) |
|---|---|---|
| `biblioteca` | Lista las canciones de `<datos>/canciones/`, marca las no terminadas, enseña el récord | comando de listado, récords |
| `procesar` | Formulario (URL de YouTube o archivo, idioma, letra opcional) y progreso de la fábrica por etapas | comando que abre el canal de la fábrica (CONTRATO §4) |
| `juego` | Canvas: guía de tono, notas, letra con palabra actual, aguja del micrófono, puntuación. Fases: preparando → cantando → fin | `cancion.json`, canal de frames (~60 Hz) |
| `resultados` | Puntuación final por línea, récord nuevo | resultado final del juego, récords |
| `ajustes` | Dispositivos de entrada y salida, calibración de `latencia_ms`, mostrar romaji y furigana | comandos de ajustes y de calibración |

`src/common/` previsto: `api/` (`comandos.ts`, `consultas.ts`,
`queryClient.ts`), `interfaces/` (espejo de `cancion.json` y de los mensajes
de canal), `stores/`, `theme/`, `components/`.

## Comandos ↔ hooks

**Vacío.** CONTRATO §5 y §6 se rellenan al implementar (fase 3). Cada fila
nueva aquí: comando, argumentos, envoltorio, hook o canal, clave de caché.

| Comando (Rust) | Argumentos | Envoltorio | Hook / canal | Clave |
|---|---|---|---|---|
| — | — | — | — | — |

## Convenciones de la casa

| Pregunta | Este repo |
|---|---|
| Idioma de identificadores, comentarios, archivos | español sin tildes |
| Idioma de la interfaz | **español**, inline en el JSX |
| Campos de datos | `snake_case` en español, exactos a CONTRATO y a los `struct` |
| Props y estado local | `camelCase` en español |
| Claves de React Query | `kebab-case` en español |
| Enumeraciones | objeto `as const` + tipo del mismo nombre (`TipoNota`) |
| Colores | solo `common/theme/paleta.ts` tiene `#RRGGBB`; también para el canvas |
| Tiempo | `t` en segundos de Rust, interpolado con `performance.now()`; sin reloj propio |
| Datos del juego | ref o store leído con `getState()` en el frame; nunca `useState` ni clausura |
| Letra japonesa | `romaji` por línea y `lectura` (hiragana) por palabra, si vienen (CONTRATO §3) |
| Líneas en `respaldo` | se pueden marcar en pantalla; no se ocultan |

## Previsto, no existe todavía

- `package.json` con los scripts `dev`, `build`, `tauri`, `typecheck`
  (`tsc -b`), `check:rules`, `verify`.
- `scripts/check-rules.mjs`: `invoke`/`Channel` solo en `comandos.ts`,
  ningún `#RRGGBB` fuera del tema, comando con envoltorio. El hook
  `check-edited.mjs` lo llamará en cuanto exista.
- La paleta y la tipografía (no hay diseño fijado).
- El router o la forma de navegar entre pantallas.
- Cómo se dibuja el texto japonés con furigana en el canvas (o en una capa
  de DOM encima).

## Verificación prevista

Desde la raíz:

```bash
npm run verify      # tsc -b && check:rules && vite build && cargo clippy --workspace --all-targets && cargo test --workspace && pytest de la fábrica
npm run typecheck   # tsc -b
```

Hoy no existe ninguno. No ejecutes `npm run dev` ni `npm run tauri dev`: no
terminan y la prueba en la app real la hace el usuario.
