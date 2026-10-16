# Testing — Vitest + Testing Library + `mockIPC` de Tauri

> **Aplica:** cuando entre Vitest (hoy **no aplica**: no hay `package.json`).
> Este archivo dice cómo se hará para no improvisar. Los helpers puros de
> `features/*/helpers/` y las funciones de `dibujo/` geométricas se prueban
> sin React; por ahí se empieza.

## Principio

Se prueba lo que el usuario ve y hace: render con los providers reales,
interacción por rol y texto con `userEvent`, aserción sobre lo visible y
sobre qué comando se invocó con qué argumentos.

No se prueba: estado interno de un hook, props pasadas a hijos, el DOM exacto
(sin snapshots), píxeles del canvas.

## Herramientas (cuando se instalen, en un commit propio)

| Qué | Con |
|---|---|
| Runner | Vitest (`vitest run`, `environment: "jsdom"`); entra en `npm run verify` el mismo día |
| Componentes y hooks | `@testing-library/react`, `@testing-library/user-event`, `@testing-library/jest-dom` |
| Backend Tauri | `@tauri-apps/api/mocks`: `mockIPC`, `clearMocks` (vienen con `@tauri-apps/api` 2.x) |

No se instala MSW: no hay red.

## Mock del backend

```ts
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
afterEach(() => clearMocks());

mockIPC((comando, argumentos) => {
  if (comando === "canciones") return cancionesDeEjemplo;   // forma exacta de interfaces/
  throw new Error(`comando no mockeado: ${comando}`);
});
```

- El handler recibe el **nombre exacto** del comando y los argumentos tal
  cual los manda `comandos.ts`. Lanza en el caso por defecto.
- Datos de ejemplo tipados con los tipos de `common/interfaces/` para que
  `tsc` los compruebe. Para `cancion.json`, el ejemplo de CONTRATO §3.
- Un error de Rust se simula lanzando un `string` en español.
- **Canales**: el argumento del canal llega al handler; el test lo usa para
  mandar mensajes (`etapa`, `listo`) y comprueba lo que pinta la pantalla. Si
  `mockIPC` no expone el canal de forma cómoda, se mockea el envoltorio de
  `comandos.ts` con `vi.mock` y se llama al `alMensaje` capturado.

## Render con los providers reales

`QueryClientProvider` (con `retry: false`), `ThemeProvider` y el router si
existe. `reset()` de cada store en `afterEach`.

## Prioridad de consultas

1. `getByRole("button", { name: /procesar/i })`, `getByLabelText`,
   `getByText` con la cadena **en español** que pinta el componente.
2. `getByTestId`: último recurso.

`findBy*` para lo que aparece tras un `invoke`. Nunca `setTimeout` +
aserción.

## Qué probar primero

1. **Helpers puros**: formato de tiempo, MIDI → altura en el canvas,
   interpolación de `t` (con tope), validación de URL.
2. **Hooks de feature** con `renderHook` + `mockIPC`.
3. **Stores** sin React: `recibir` + `getState()`.
4. **Pantallas**: procesar enseña la etapa que llega por el canal y, al
   `listo`, la canción aparece en la biblioteca.

## Qué no cubre jsdom

Canvas real (jsdom no tiene contexto 2D sin un paquete aparte), layout,
audio, Tauri de verdad. Lo prueba el usuario en `npm run tauri dev`.

## Sí / No

| Sí | No |
|---|---|
| `mockIPC` con nombres de `lib.rs` | Un mock que acepta cualquier comando |
| Helpers primero | Empezar por la pantalla del juego entera |
| Consultas por rol y texto en español | `querySelector(".MuiButton-root")` |
| `vitest run` dentro de `npm run verify` | Un runner que nadie ejecuta |
