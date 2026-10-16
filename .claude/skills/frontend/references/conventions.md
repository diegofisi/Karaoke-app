# Convenciones — nombres, idioma, estilo React, tema, enumeraciones, efectos

> **Aplica:** siempre. Revisión final de cualquier cambio. `project.md` →
> Convenciones de la casa lo concreta para este repo.

## Idioma

1. **El código habla español sin tildes**: identificadores, nombres de
   archivo, comentarios, claves de caché, props (`cancionId`, `onFin`,
   `LienzoJuego.tsx`, `"canciones"`). Igual que Rust, Python y CONTRATO §1.
2. **La pantalla habla español**: toda cadena visible en español, inline.
3. Los datos que vienen de Rust (`snake_case` en español) no se traducen ni
   se renombran: `linea.inicio`, `nota.midi`, `origen.fabrica_version`.

## Nombres

| Elemento | Convención | Ejemplo |
|---|---|---|
| Carpeta de feature | nombre de la pantalla en español | `biblioteca/`, `juego/` |
| Página | `{Nombre}Page.tsx` | `BibliotecaPage.tsx`, `JuegoPage.tsx` |
| Container | `{Nombre}Container.tsx` | `ProcesoContainer.tsx` |
| Componente | `PascalCase.tsx`, sustantivo en español | `FilaCancion.tsx`, `LienzoJuego.tsx` |
| Hook de feature | `use{Que}.ts` | `useBiblioteca.ts`, `useFramesJuego.ts` |
| Hook de datos | `use{Comando}` en `consultas.ts` | `useCanciones`, `useProcesarCancion` |
| Envoltorio de comando | camelCase del comando en `comandos.ts` | `procesarCancion` |
| Tipo espejo de Rust | el nombre del `struct` | `Cancion`, `Linea`, `Nota` |
| Función de dibujo | verbo `dibujar{Que}` en `dibujo/` | `dibujarGuiaTono` |
| Store | `{dominio}.ts` exporta `use{Dominio}Store` | `useJuegoStore` |
| Constante | `UPPER_SNAKE_CASE` con unidad | `ALTO_SEMITONO_PX`, `VENTANA_VISIBLE_S` |
| Objeto const + tipo | `PascalCase`, mismo nombre | `TipoNota` |
| Nombre de comando | exacto al de `lib.rs`, `snake_case` | `"procesar_cancion"` |
| Clave de caché | `kebab-case` | `"canciones"`, `"records"` |

Identificadores descriptivos: `.map((linea) => …)`, no `(l)`. Setters de
`useState` en español: `const [fase, fijarFase] = useState(…)`.

Imports con alias `@/` para todo lo que cruce de carpeta; relativos dentro de
la misma feature.

## Estilo React

- `.tsx` solo para archivos con JSX.
- Props: `export interface {Nombre}Props { … }`, desestructuradas en la
  firma. Nunca `React.FC`.
- `export const Nombre = (…) => (…)`; sin `export default`.
- Orden de imports: `react` → MUI → `@tanstack/react-query` → `@/common/...`
  → relativos. `import type` para lo que solo es tipo.
- `key` es identidad (`cancion.id`), no índice. Las palabras de una línea no
  tienen id: `key` por `inicio` (es único y ordenado en una línea, CONTRATO
  §3).
- React 19: `ref` es una prop normal; nada de `forwardRef`. `useMemo` /
  `useCallback` solo con motivo.

## Comentarios

En español, pocos, y solo cuando dicen el porqué. Cita CONTRATO.md y
ARQUITECTURA.md **por número de sección** (`CONTRATO.md §3`).

## Tema y tokens

- `common/theme/paleta.ts` es el **único** archivo con `#RRGGBB`. Un color
  nuevo se añade allí, se registra en `theme.ts` y se declara en `tipos.ts`.
- `sx` para todo estilo; nunca `style={{}}` ni clases CSS para lo que `sx`
  resuelve. Excepción: el contenido del canvas, que se pinta con la API 2D y
  los colores del tema.
- Tipografía por `variant`.

## Objeto const + tipo, nunca `enum`

```ts
export const TipoNota = { Normal: "normal", Dorada: "dorada", Libre: "libre" } as const;
export type TipoNota = (typeof TipoNota)[keyof typeof TipoNota];
```

Los valores son los que viajan desde Rust (CONTRATO §3). Compara siempre
contra la constante.

## Efectos — probablemente no lo necesitas

`useEffect` es solo para sincronizar con algo externo: el bucle de
`requestAnimationFrame`, un `ResizeObserver`, abrir y cerrar un `Channel`.

| Anti-patrón | Correcto |
|---|---|
| Derivar estado en un efecto | Calcular en el render o en `useMemo` |
| Vaciar un formulario cuando cambia el dato | `key={dato.id}` |
| `setState` por cada frame del juego | Ref o store leído en el bucle de dibujo |
| Un `useCallback` que lee datos del juego desde su clausura | Leer de un ref o de `getState()` dentro del frame |

Reglas de hooks:

- **Actualizador funcional** cuando el nuevo estado depende del anterior.
- **`useRef` no se lee ni se escribe en el render**, solo en handlers,
  efectos y el bucle de dibujo.
- **Dependencias completas** en `useEffect` / `useCallback`. Si un callback
  necesita "siempre el último valor", ese valor va en un ref, no se omite de
  las dependencias.

## Sí / No

**Sí**: español en el código y en pantalla · rutas de la paleta en `sx` ·
objeto const + tipo · `key` por id · lógica en hooks · canvas para lo que
cambia por frame · `npx tsc -b` (o `npm run verify`) antes de decir hecho.

**No**: `#RRGGBB` fuera de `paleta.ts` · `enum` · `any` · `React.FC` ·
`forwardRef` · `<div>`/`<span>`/`<p>` · `console.log` que quede · `alert` ·
`invoke` fuera de `comandos.ts` · `useQuery` fuera de `consultas.ts` ·
imports entre features · `setState` por frame · reloj propio de la canción.
