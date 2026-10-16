# Arquitectura — `common/` y `features/`, reglas de import, shell

> **Aplica:** siempre. Los nombres reales de features y comandos están en
> `project.md`; este archivo da la forma.

## Filosofía

- **Lógica ≠ interfaz.** Los hooks piensan; los componentes pintan.
- **La interfaz solo pinta.** No toca audio, no detecta tono, no puntúa
  (ARQUITECTURA §1). Eso es Rust.
- **Un tipo por `struct`.** Lo que devuelve Rust llega con la forma exacta de
  serde (español, `snake_case`). No hay mapper (`data-flow.md`).
- **Slices verticales**: una carpeta por pantalla, autocontenida.
- **Lo compartido sube** a `common/`. Nunca un import entre features.
- **Sin etiquetas HTML crudas** salvo `<canvas>`: `Stack`, `Box`,
  `Typography` de MUI; el color sale del tema.

## Estructura

```text
src/
 ├── common/
 │    ├── api/
 │    │    ├── comandos.ts      # el ÚNICO archivo con `invoke` y `new Channel`; un envoltorio por comando
 │    │    ├── consultas.ts     # un hook de React Query por consulta o mutación
 │    │    └── queryClient.ts   # la instancia única de QueryClient
 │    ├── components/           # primitivas de la casa (las que existan; lista en project.md)
 │    ├── interfaces/           # espejo literal de los struct de Rust y de cancion.json + index.ts
 │    ├── stores/               # Zustand: lo que leen varias pantallas
 │    └── theme/                # paleta.ts (el único #RRGGBB), theme.ts, tipos.ts
 ├── features/
 │    └── {feature}/
 │         ├── pages/           # {Nombre}Page.tsx: raíz de composición de la pantalla
 │         ├── containers/      # solo si la página coloca 2+ unidades conectadas
 │         ├── components/      # presentacionales
 │         ├── hooks/           # la lógica
 │         ├── helpers/         # funciones puras (geometría del canvas, formato de tiempo)
 │         ├── dibujo/          # solo en `juego`: funciones que pintan en el canvas
 │         └── interfaces/      # view-models propios de la pantalla
 ├── App.tsx
 └── main.tsx                   # StrictMode + QueryClientProvider + ThemeProvider
```

No toda feature necesita todas las carpetas.

### Propósito de cada carpeta de una feature

| Carpeta | Qué contiene | Qué no contiene |
|---|---|---|
| `pages/` | Un componente por pantalla. Compone hooks, containers y componentes | lógica de datos de más de dos líneas |
| `containers/` | Unidad conectada y fina: llama a un hook, pinta componentes | layout, `useMemo`, `useEffect`, handlers de varios pasos |
| `components/` | Props → JSX; estado efímero propio | hooks de `consultas.ts`, store, `invoke` |
| `hooks/` | Cruza hooks de `consultas.ts`, deriva, abre canales, expone handlers | JSX |
| `helpers/` | Funciones puras, testeables sin React | hooks, imports de MUI |
| `dibujo/` | `(ctx, estado, t) => void`: pintan un frame | React, stores, `invoke` |
| `interfaces/` | View-models, uniones, objetos const | espejos de Rust (van en `common/interfaces/`) |

## Por qué `api/` no está dentro de cada feature

Varios comandos los usan varias pantallas (la lista de canciones la leen
biblioteca, resultados y juego). Un envoltorio por feature sería la misma
función copiada con claves de caché distintas. Por eso `comandos.ts` y
`consultas.ts` viven en `common/api/` y son la única capa que conoce React
Query y Tauri.

## Reglas de dependencia

| Desde | Puede importar | Nunca importa |
|---|---|---|
| `common/api/` | `common/interfaces/`, `@tauri-apps/api/core` (solo `comandos.ts`), React Query | `features/` |
| `common/components/` | `common/**` | `features/` |
| `common/stores/`, `common/theme/` | `common/**` | `features/`, React Query |
| `features/{f}` | `@/common/**`, su propia carpeta | otra feature |
| `App.tsx` | `features/*/pages/*Page.tsx`, `common/` | el interior de una feature |

Containers nunca renderizan containers.

## Acceso entre features — sin imports cruzados

1. Lo que las dos necesitan está en `common/api/consultas.ts`: cada una lo
   llama. Se comparte la caché, no el código.
2. Un componente o helper que las dos pintan igual → sube a `common/`.
3. Un cruce de datos que solo una necesita → vive en el `hooks/` de esa,
   aunque se parezca al de la otra.

## Shell

La navegación entre pantallas (router o estado) está **por decidir**
(`project.md`). Sea cual sea:

- Ninguna ruta ni nombre de pantalla se escribe como literal en un
  componente: constantes en `common/`.
- Lo que una pantalla pasa a otra (el `id` de la canción) va en la ruta o en
  el estado de navegación, no en un store global.
- El juego ocupa la ventana: el shell no pinta barras encima del canvas.

## Encapsulación del transporte

`invoke` y `Channel` se importan **solo** en `common/api/comandos.ts`. React
Query solo en `common/api/consultas.ts` y `queryClient.ts`. Páginas,
containers y componentes consumen hooks que devuelven los tipos de
`common/interfaces/`. Lo comprobará `check:rules` (previsto); mientras, es
revisión manual.
