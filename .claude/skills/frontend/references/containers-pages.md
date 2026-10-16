# Páginas, containers y extracción de lógica

> **Aplica:** siempre. "Página" es el componente raíz de una pantalla;
> "vista interna" es una parte de la página elegida por estado.

## La frontera container / componente

La regla no es "componentes sin hooks"; es **datos y negocio** frente a
**presentación**.

**Regla A — dónde vive el estado.** Si el estado desapareciera al remontar y
nada fuera del componente lo notaría, es del componente; si algo externo
(Rust, la caché de React Query, el store, otra unidad de la página) depende
de él, es del hook o del container.

| En el componente | En el hook / container |
|---|---|
| `useState` de abierto, pestaña, borrador del campo | Datos de `consultas.ts` |
| `useRef`, `useId`, el canvas y su bucle de dibujo | Mutaciones y canales (procesar, empezar juego) |
| Animación CSS de su propia interfaz | Lectura o escritura del store |
| | Derivados de negocio, navegación, estado entre unidades |

**Regla B — qué JSX puede tener un container.** Solo otros componentes,
condicionales, fragmentos y props. Layout, tipografía o un botón son un
componente y se extraen.

**Regla C — el container es fino; la lógica vive en un hook.** Un container
ideal se lee `const x = useX(); return <Vista {...x} />;`.

### Las cuatro categorías

| Capa | Carpeta | Qué es | Posee |
|---|---|---|---|
| Página | `pages/` | Raíz de composición | el estado entre unidades (vista interna) |
| Container | `containers/` | Unidad conectada y fina | el cableado de su asunto |
| Componente | `components/` | Presentacional | props + estado efímero |
| Hook | `hooks/` | La lógica | consultas cruzadas, derivados, mutaciones, canales |

### ¿Cuándo existe un container?

Solo cuando una página coloca **2+ unidades conectadas** que no comparten
lógica. Una página que solo envuelve un container es un anti-patrón: o la
página absorbe el hook, o el container desaparece.

## Patrones de página

| Situación | Patrón | Ejemplo previsto |
|---|---|---|
| Una vista, uno o dos hooks | **A — la página absorbe** | `ajustes` |
| Una vista con lógica que cruza consultas | **C — página + hook** | `biblioteca` + `useBiblioteca`; `resultados` + `useResultados` |
| Varias vistas internas o fases | **B — la página orquesta** | `juego` (cuenta atrás → cantando → pausa → fin); `procesar` (formulario → en curso → listo o error) |

### Patrón C — página + hook

```tsx
export const BibliotecaPage = () => {
  const { cargando, error, canciones, jugar } = useBiblioteca();
  return (
    <Stack spacing={2.5}>
      <CabeceraBiblioteca />
      {cargando && <Estado tipo="cargando" />}
      {error && <Estado tipo="error" mensaje="No se pudo leer la carpeta de canciones." />}
      {!cargando && !error && canciones.map((c) => <FilaCancion key={c.id} cancion={c} onJugar={jugar} />)}
    </Stack>
  );
};
```

(`Estado` es ilustrativo: la primitiva real de estados se fija en
`project.md`.)

### Patrón B — la página orquesta por estado

```tsx
type FaseJuego = { tipo: "preparando" } | { tipo: "cantando" } | { tipo: "fin"; resultado: Resultado };

export const JuegoPage = ({ id }: { id: string }) => {
  const [fase, fijarFase] = useState<FaseJuego>({ tipo: "preparando" });
  if (fase.tipo === "fin") return <ResumenFinal resultado={fase.resultado} />;
  if (fase.tipo === "cantando") return <PantallaCanto id={id} onFin={(resultado) => fijarFase({ tipo: "fin", resultado })} />;
  return <CuentaAtras onListo={() => fijarFase({ tipo: "cantando" })} />;
};
```

- La fase es un **tipo unión con discriminante**, no varios booleanos.
- `if` secuenciales con retorno temprano; la vista por defecto es el último
  `return`.
- Las unidades reciben callbacks (`onFin`, `onListo`) y no saben qué otras
  fases hay.
- El cambio de fase no pasa por el bucle de dibujo: el canvas no llama a
  `setState`; el fin lo anuncia el canal (mensaje final de Rust) y el hook lo
  convierte en una llamada a `onFin`.

### Filas como view-models

Los ítems de una lista son tontos. Si una fila necesita derivados (récord,
estado de procesado), el hook precomputa las filas y el componente las recibe
por props.

## Container de escritura

- Llama a `mutate` y decide en **un** sitio qué pasa después (`onSuccess`).
  Nunca `mutateAsync` + `try/catch`.
- Deshabilita el control mientras `isPending` o mientras un canal está
  abierto: dos clics en "Procesar" son dos procesos de la fábrica.

## Extraer un hook

| Señal | Acción |
|---|---|
| 3+ `useMemo` / `useCallback` / `useEffect` en una página o container | Extraer a `hooks/use{Pantalla}.ts` |
| Dos hooks de `consultas.ts` cuyos datos se cruzan | Extraer |
| Abrir y cerrar un `Channel` | Siempre en un hook |
| Un `useState` + un handler | Dejarlo inline |
| Lógica sin React (formatear tiempo, geometría) | `helpers/` |

`useReducer` solo para una máquina de estados real (las fases del juego si
crecen).
