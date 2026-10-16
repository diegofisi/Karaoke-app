---
name: frontend
description: >
  Interfaz del karaoke (src/ en la raíz del repo: Tauri 2 + React 19 +
  TypeScript + Vite + MUI + React Query + Zustand). Úsala al crear o cambiar
  cualquier archivo de src/: una pantalla de las features biblioteca,
  procesar, juego, resultados o ajustes (src/features/{biblioteca,procesar,
  juego,resultados,ajustes}/), el envoltorio de invoke o de Channel en
  src/common/api/comandos.ts, un hook de React Query en
  src/common/api/consultas.ts, un tipo de src/common/interfaces/ (espejo de
  serde y de cancion.json: Cancion, Linea, Palabra, Nota, frames del juego,
  mensajes de la fábrica), un store de Zustand, el tema de MUI, el canvas del
  juego (dibujo fuera de los re-renders de React con requestAnimationFrame,
  interpolación de t con performance.now()), la barra de progreso de la
  fábrica, la calibración de latencia, o cualquier regla de docs/CONTRATO.md
  que afecte a la interfaz. Texto visible en español. Lo que aplica a este
  repo está en references/project.md, que se lee primero y manda.
---

# Frontend — React + MUI sobre Tauri

Doctrina portable para la interfaz de una app de escritorio Tauri escrita en
React con MUI: slices verticales, transporte por `invoke` y `Channel`
encapsulado, container y presentacional, React Query para lo que devuelve
Rust, Zustand para lo que no, y un canvas fuera de React para lo que cambia
60 veces por segundo. La doctrina no sabe qué comandos existen ni qué
pantallas hay: eso lo dice `references/project.md`.

## Precedencia

**Código > `docs/CONTRATO.md` > `references/project.md` > doctrina.**

- Si el código y `project.md` discrepan, manda el código y arreglar
  `project.md` es parte del mismo cambio.
- Si `CONTRATO.md` y el código discrepan, no se "arregla" ninguno de los dos
  de paso: se anota y se avisa al usuario.

## Cómo usar esta skill

1. Lee **`references/project.md`** entero: stack, mapa de features, tabla
   *Aplica / no aplica*, lo previsto y la verificación.
2. Lee **`references/pitfalls.md`** y **`references/project-pitfalls.md`**
   antes de dar por terminado cualquier cambio.
3. Abre solo los archivos de doctrina que nombre el workflow.
4. Para tipos y nombres, abre `docs/CONTRATO.md`. Los nombres se copian, no
   se traducen.

## Índice de referencias

| Referencia | Cubre | Léela cuando |
|---|---|---|
| `references/project.md` | Stack, features, comandos ↔ hooks, convenciones de la casa, previsto, verificación | Siempre, primero |
| `references/pitfalls.md` | Trampas universales de React + MUI + React Query + Tauri + canvas | Antes de terminar |
| `references/project-pitfalls.md` | El libro de errores fechado de ESTE repo | Antes de terminar; al corregir algo por segunda vez |
| `references/architecture.md` | `common/` y `features/`, reglas de import, el shell | Crear una feature, colocar un archivo |
| `references/data-flow.md` | serde → `interfaces/` → `comandos.ts` → `consultas.ts` → hook → props; canales; claves de caché | Cualquier hook que toque un comando |
| `references/tauri-transport.md` | `invoke` y `Channel` solo en `comandos.ts`, `snake_case`, `Err(String)` en español | Tocar `comandos.ts`, un error de Rust, un canal |
| `references/containers-pages.md` | Frontera container / componente, patrones de página, extraer hooks | Escribir páginas, containers o hooks |
| `references/components.md` | Contrato presentacional, MUI `sx` con la paleta, estados, el canvas del juego | Escribir JSX o dibujar |
| `references/state.md` | Zustand, React Query, `useState`, refs; datos de juego leídos en el frame, no por clausura | Tocar un store, el bucle de dibujo, un temporizador |
| `references/forms.md` | Formularios sin librería con campos de MUI | URL de YouTube, letra, ajustes |
| `references/conventions.md` | Nombres, idioma, estilo React, tema, efectos | Nombrar algo; revisión final |
| `references/testing.md` | Vitest + Testing Library + `mockIPC` | Cuando entre Vitest |

## Workflow A — Nueva feature

1. Crea `src/features/{nombre}/` con solo las carpetas que vaya a usar —
   `architecture.md`.
2. Orden: tipo en `common/interfaces/` → envoltorio en `comandos.ts` → hook
   en `consultas.ts` → hook de feature → componentes → página.
3. Ruta y enlace en el shell — `architecture.md` → Shell.
4. Ninguna feature importa de otra. Lo compartido sube a `common/`.

## Workflow B — Nuevo hook sobre un comando

1. Confirma que el comando existe en `src-tauri/src/lib.rs`
   (`generate_handler!`) y en `CONTRATO.md` §5. Nunca inventes un nombre ni
   un campo.
2. Si el tipo es nuevo, su espejo en `common/interfaces/`: mismos nombres
   `snake_case`, `null` donde Rust tiene `Option` — `data-flow.md`.
3. Envoltorio tipado en `comandos.ts` — `tauri-transport.md`.
4. Clasifícalo (`data-flow.md`): consulta → `useQuery`; mutación →
   `useMutation` que invalida; canal → función que abre el `Channel` y un
   store o ref que recibe los mensajes.

## Workflow C — Pantalla del juego (canvas)

1. El dibujo vive en un `<canvas>` con su propio bucle de
   `requestAnimationFrame`, montado una vez — `components.md` → Canvas.
2. Los frames que llegan por el `Channel` se guardan en un ref o en un store
   leído con `getState()` **dentro** del frame de dibujo, nunca en estado de
   React ni capturados por un `useCallback` — `state.md`,
   `project-pitfalls.md`.
3. `t` se interpola con `performance.now()` desde el último frame recibido.
   La interfaz no tiene reloj propio de la canción.
4. React solo re-renderiza lo que cambia poco (título, pausa, resultado
   final).
5. La interfaz no puntúa ni detecta tono: lo hace Rust.

## Workflow D — Nuevo componente

1. Presentacional: props + estado efímero propio. Sin hooks de
   `consultas.ts`, sin store, sin `invoke` — `containers-pages.md`.
2. MUI (`Stack`, `Box`, `Typography`, `Button`); nunca `<div>`, `<span>`,
   `<p>`. Colores por ruta de la paleta en `sx`.
3. Texto visible **en español**, inline.
4. Cargando / error / vacío dentro del área de contenido — `components.md`.

## Workflow E — Nuevo store

1. Si un comando lo devuelve, es React Query, no un store — `state.md`.
2. Store para estado de interfaz que leen varias pantallas, y para datos de
   alta frecuencia que el canvas lee con `getState()`.
3. State y Actions tipados por separado, `estadoInicial`, `reset()`.

## Workflow F — Vincular a este proyecto

Cuando cambie el stack o aparezca el código: skill hermana **`bind-project`**.
`project-pitfalls.md` nunca se reescribe: solo se añaden líneas.

## Validación

Desde la raíz: `npm run verify` cuando exista; mientras tanto, `npx tsc -b` y
`npx vite build`. No ejecutes `vite` (dev) ni `tauri dev`: los lanza el
usuario.

Lo que ningún comando comprueba y queda en tus manos: `invoke` o `Channel`
fuera de `comandos.ts`, hooks de `consultas.ts` dentro de un componente,
imports entre features, un `#RRGGBB` fuera del tema, datos del juego en
estado de React o en una clausura, texto visible que no esté en español.
