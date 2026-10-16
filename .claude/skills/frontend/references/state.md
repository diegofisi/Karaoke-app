# Estado — React Query, Zustand, `useState`, refs y el juego

> **Aplica:** siempre. Lo que existe de verdad está en `project.md`.

## Dónde vive cada cosa

| Tipo de estado | Dónde | Ejemplo previsto |
|---|---|---|
| Lo que devuelve un comando de Rust | **React Query** (`consultas.ts`) | lista de canciones, una canción, récords, ajustes en disco |
| Preferencias de interfaz que leen varias pantallas | **Zustand** (`common/stores/`) | mostrar romaji, mostrar furigana |
| Progreso de la fábrica (canal) | **Zustand** con suscriptores normales | etapa, avance, avisos, error |
| Frames del juego (canal, ~60 Hz) | **ref** o **store leído con `getState()`** desde el bucle de dibujo, sin suscriptores de React | `t`, tono del micrófono, claridad, puntuación |
| Lo que solo una pantalla necesita mientras está montada | **`useState`** | fase del juego, texto del formulario |

Nunca se copia a un store lo que un comando devuelve. Nunca se modela en
`useState` lo que viene de Rust.

## Datos del juego: se leen en el frame, nunca por clausura

El bucle de dibujo y cualquier callback que corra muchas veces leen el dato
**en el momento de usarlo**:

```ts
// bien: el bucle lee el último frame cada vez
const pintar = () => {
  const frame = useJuegoStore.getState().ultimo;
  dibujar(ctx, frame);
  id = requestAnimationFrame(pintar);
};

// mal: el callback captura `frame` del render en que se creó y lo usa para siempre
const pintar = useCallback(() => dibujar(ctx, frame), [ctx]);
```

La versión anterior capturó así el texto reconocido dentro del bucle del
juego (`useCallback` sin la dependencia) y puntuaba contra un texto vacío
(`project-pitfalls.md`). Regla: un valor que cambia mientras el bucle vive va
en un ref o en un store, y se lee dentro del bucle.

## Tiempo en la interfaz

- La interfaz **no tiene reloj propio de la canción**. Cada frame de Rust trae
  `t` (segundos, del contador de muestras de la salida).
- Entre frames, `t` se interpola: `t_frame + (performance.now() -
  llegada_frame) / 1000`, con tope (si no llega un frame en ~100 ms, no se
  extrapola más: se congela).
- `Date.now()` no se usa para el juego: no es monótono.
- La interfaz no puntúa. Si una cifra de puntuación aparece en pantalla, vino
  de Rust.

## Store — forma canónica

```ts
interface JuegoState { ultimo: FrameJuego | null; llegada_ms: number }
interface JuegoActions { recibir: (frame: FrameJuego) => void; reset: () => void }
const estadoInicial: JuegoState = { ultimo: null, llegada_ms: 0 };

export const useJuegoStore = create<JuegoState & JuegoActions>((set) => ({
  ...estadoInicial,
  recibir: (frame) => set({ ultimo: frame, llegada_ms: performance.now() }),
  reset: () => set(estadoInicial),
}));
```

- State y Actions tipados por separado; `estadoInicial`; `reset()`.
- Nombres en español, también las acciones.
- Un store no importa React ni hooks de `consultas.ts`.
- **Selectores estrechos** en componentes: `useStore((estado) => estado.x)`.
  Ningún componente se suscribe a `ultimo` del store del juego: re-renderizaría
  60 veces por segundo. Solo el canvas lo lee con `getState()`.

## Procesos vivos de la interfaz

- **Bucle de dibujo**: un `requestAnimationFrame` montado una vez en un
  efecto, con `cancelAnimationFrame` en la limpieza.
- **Canal del juego**: lo abre un hook de feature al empezar y lo cierra
  (comando de parar) al desmontar. El fin del juego lo decide Rust (mensaje
  final), no un `setTimeout` de la interfaz.
- **Canal de la fábrica**: igual. La interfaz no decide que la fábrica
  terminó; espera `listo` o `error`.

## Reiniciar estado local cuando cambia el dato

- Una canción nueva = componente nuevo: `key={cancion.id}`.
- `reset()` del store del juego al empezar cada partida.

## Anti-carrera

- Un proceso o mutación en curso deshabilita su control.
- Un `setState` que depende del anterior usa actualizador funcional.
- Un mensaje de canal que llega después de cerrar la pantalla se ignora (el
  hook comprueba que la partida sigue siendo la suya).

## Sí / No

| Sí | No |
|---|---|
| `getState()` dentro del bucle de dibujo | `useCallback` que captura datos del juego |
| `t` de Rust interpolado con `performance.now()` | Reloj propio con `setInterval` |
| Selector estrecho en componentes | `const store = useJuegoStore()` |
| `key={cancion.id}` para reiniciar | Un efecto que limpia al cambiar la canción |
