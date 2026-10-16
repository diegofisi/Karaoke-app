# Componentes — contrato presentacional, MUI, estados, canvas

> **Aplica:** siempre. El kit es MUI con el tema de `common/theme/`. La paleta
> y las primitivas de la casa se fijan en `project.md` cuando existan.

## El contrato presentacional

Props (tipos de `common/interfaces/` o un view-model de la feature) → JSX.
Puede tener estado efímero propio (abierto, pestaña, `useRef`). No puede:
llamar a hooks de `consultas.ts`, leer o escribir el store, navegar, ni
derivar negocio.

```tsx
export interface FilaCancionProps {
  cancion: ResumenCancion;
  onJugar: (id: string) => void;
}

export const FilaCancion = ({ cancion, onJugar }: FilaCancionProps) => (
  <Stack direction="row" spacing={1.5} alignItems="center">
    <Typography variant="body1" sx={{ fontWeight: 700 }}>{cancion.titulo}</Typography>
    <Typography variant="body2" sx={{ color: "text.secondary" }}>{cancion.artista}</Typography>
    <Button variant="contained" onClick={() => onJugar(cancion.id)} sx={{ ml: "auto" }}>
      Cantar
    </Button>
  </Stack>
);
```

(Los nombres de tipos y campos del ejemplo son ilustrativos; los reales salen
de `CONTRATO.md`.)

## Primitivas de la casa

Una primitiva nueva va a `common/components/` cuando el mismo patrón aparece
por **segunda** vez en features distintas. La lista de las que existen está
en `project.md`. Úsalas antes que MUI crudo.

## MUI: qué se usa y cómo

- Layout: `Stack` y `Box` (`display: "grid"` para rejillas). No mezcles con
  `Grid` de MUI.
- Texto: `Typography` con `variant` del tema. Nunca `<p>`, `<span>`, `<h1>`.
- Controles: `Button`, `IconButton`, `TextField`, `Select` + `MenuItem`,
  `Slider`, `Switch`, `LinearProgress` (progreso de la fábrica), `Tooltip`.
- Imports por ruta: `import Stack from "@mui/material/Stack"`.

## Color y medida: solo desde el tema

- Color en `sx` por **ruta de la paleta** (`"primary.main"`,
  `"text.secondary"` o los tonos propios que declare `paleta.ts`).
- **Ningún `#RRGGBB`** fuera de `common/theme/paleta.ts`. Ningún `alpha()`,
  `lighten()`, `darken()` en un componente: si falta un tono, se añade a la
  paleta y a `tipos.ts`.
- **El canvas también**: los colores con los que dibuja se leen del tema
  (`useTheme()` una vez al montar, pasados a las funciones de `dibujo/`), no
  se escriben como literales en el código de dibujo.

## Canvas del juego

Lo que cambia ~60 veces por segundo (la guía de tono, la aguja del
micrófono, el resaltado de la palabra, la puntuación en curso) se dibuja en un
`<canvas>`, no con componentes de React.

```tsx
export const LienzoJuego = ({ fuente }: { fuente: FuenteFrames }) => {
  const lienzo = useRef<HTMLCanvasElement | null>(null);
  useEffect(() => {
    const ctx = lienzo.current?.getContext("2d");
    if (!ctx) return;
    let id = 0;
    const pintar = (ahora: number) => {
      const estado = fuente.leer();            // lee el último frame EN ESTE momento
      const t = fuente.tiempoEn(ahora);         // t interpolado con performance.now()
      dibujarFrame(ctx, estado, t);
      id = requestAnimationFrame(pintar);
    };
    id = requestAnimationFrame(pintar);
    return () => cancelAnimationFrame(id);
  }, [fuente]);
  return <Box component="canvas" ref={lienzo} sx={{ width: "100%", height: "100%" }} />;
};
```

- El bucle se monta **una vez**. Sus dependencias son estables (una fuente
  creada una vez), no datos que cambian.
- Los datos del frame se **leen dentro de `pintar`** (ref o
  `store.getState()`), nunca desde una variable capturada al crear el
  callback — `state.md`, `project-pitfalls.md`.
- Ningún `setState` por frame. Si una parte de React necesita un valor que
  cambia (la puntuación final), se actualiza cuando cambia de verdad, no 60
  veces por segundo.
- Tamaño en píxeles reales: `canvas.width = clientWidth * devicePixelRatio`
  al montar y al cambiar el tamaño (`ResizeObserver`), no en cada frame.
- `cancelAnimationFrame` en la limpieza; con `StrictMode` el efecto se monta
  dos veces en desarrollo.

## Tamaño y extracción

| Métrica | Umbral | Acción |
|---|---|---|
| Líneas de JSX | > ~200 | Extraer sub-componentes |
| Props | > 10 | Agrupar en un view-model o dividir |
| Función de `dibujo/` | > ~60 líneas | Dividir por capa (fondo, guía, letra, aguja) |
| Archivos en `components/` | > ~8 | Subcarpetas por vista |

## Estados: cargando, error, vacío

**Dentro del área de contenido**: la cabecera no se desmonta porque la lista
esté cargando. Mensajes en español, concretos: "No hay canciones todavía.
Pega una URL de YouTube para procesar la primera."

- Dato ausente ≠ error: una canción sin `cancion.json` se lista como "sin
  terminar", no se oculta.
- Una línea con `metodo: "respaldo"` se puede marcar (CONTRATO §3); no se
  oculta.

## Texto visible en español

- Toda cadena visible va **en español, inline** en el JSX. No hay
  diccionario ni librería de i18n.
- Lo que viene de Rust o de la fábrica en español (mensajes de error, de
  etapa) es dato: se pinta tal cual.
- La letra de la canción es dato en su idioma (`es`, `en`, `ja`), con
  `romaji` y `lectura` cuando existan.

## Accesibilidad

- Todo estado comunicado por color también por texto o icono.
- Interactivo = `Button` o `IconButton` con `aria-label` si no tiene texto.
  Nunca `<Box onClick>`.
- El canvas lleva `aria-label` con lo que representa; la información que
  importa (puntuación final) también se da en texto fuera del canvas.
