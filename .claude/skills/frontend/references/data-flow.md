# Flujo de datos — de serde a las props, sin mapper

> **Aplica:** siempre. La tabla de comandos, envoltorios, hooks y claves está
> en `project.md` cuando existan; este archivo no la copia.

## El flujo

```
struct Rust (serde) ──► common/interfaces/X.ts ──► comandos.ts ──► consultas.ts ──► features/{f}/hooks/useY ──► props
   snake_case español      espejo literal          invoke<X>        useQuery/useMutation   cruza, deriva, expone       componente

Channel<T> de Rust ──► comandos.ts (new Channel, onmessage) ──► store o ref ──► hook de feature / bucle de dibujo
```

| Capa | Responsabilidad |
|---|---|
| Tipo (`common/interfaces/`) | Copia a mano del `struct` de Rust y de `cancion.json`: mismos nombres `snake_case`, `null` donde Rust tiene `Option`, comentario con el origen (`// Origen: crates/karaoke-core/src/lib.rs`) |
| Envoltorio (`comandos.ts`) | `invoke` con el nombre literal; o crea el `Channel`, le pone el manejador y lo pasa al comando |
| Hook de consulta (`consultas.ts`) | `useQuery` / `useMutation`. Uno por envoltorio de consulta o mutación |
| Hook de feature | Cruza, deriva view-models, abre y cierra canales, expone handlers y estados |
| Página / container | Llama al hook de feature y reparte props |
| Componente | Props. Cero conocimiento de React Query o de Tauri |

## Por qué aquí NO hay DTO → mapper → Model

CONTRATO §1: "Nombres en español, sin tildes, `snake_case`, en JSON, Rust,
TypeScript y Python. La interfaz no traduce nombres." Backend, fábrica e
interfaz están en el mismo repo y los nombres ya están en el idioma del
código. Un mapper solo cambiaría `_` por mayúsculas.

| Necesidad | Aquí |
|---|---|
| Un derivado para pintar | lo calcula el hook de feature y lo tipa en `features/{f}/interfaces/` |
| Formatear (tiempo `mm:ss`, nota MIDI → nombre) | helper puro en `helpers/` |
| Enviar algo a Rust | con la forma exacta del `struct` |

## Clasificación de un comando

| Arquetipo | Ejemplo previsto | Forma | En la pantalla |
|---|---|---|---|
| **Consulta** sin argumentos | lista de canciones, ajustes | `useQuery({ queryKey: ["canciones"], queryFn })` | `data`, `isLoading`, `isError` |
| **Consulta** con argumento que puede faltar | una canción por `id` | `enabled: id.length > 0` | apagada hasta tener el argumento |
| **Mutación** que cambia disco | guardar récord, guardar ajustes | `useMutation` + `invalidateQueries` | `mutate(args, { onSuccess })` |
| **Canal** de un trabajo largo | procesar canción (fábrica) | envoltorio que abre el `Channel`; mensajes a un store | barra de progreso con la etapa; al `listo`, invalidar `["canciones"]` |
| **Canal** de alta frecuencia | frames del juego (~60 Hz) | envoltorio que abre el `Channel`; mensajes a un ref o store **sin re-render** | el canvas los lee en su bucle |

No hay sondeo (`refetchInterval`): lo que cambia en Rust llega por un canal o
tras una mutación.

## Canales

- El envoltorio de `comandos.ts` crea el `Channel`, asigna `onmessage` y llama
  al comando. Devuelve lo necesario para cerrar (o el comando de parar).
- Mensajes de la fábrica (CONTRATO §4): `etapa` → progreso; `aviso` → lista
  de avisos; `listo` → invalidar la lista de canciones; `error` → mensaje en
  español en pantalla.
- Frames del juego: se escriben en un ref o en un store con `setState` sin
  suscriptores de React. El canvas lee el último en cada frame de dibujo.
  Nunca `useState` por frame.
- El hook que abre el canal lo cierra (comando de parar) al desmontar.

## Claves de caché

- `["nombre-kebab", ...argumentos]`: `["cancion", id]`, `["records", id]`.
  Un argumento opcional va como `null`, nunca se omite.
- La invalidación vive en el `onSuccess` del hook de mutación, en
  `consultas.ts`, porque es intrínseca al comando.

## Hook de feature — la forma

```ts
export const useBiblioteca = (): DatosBiblioteca => {
  const consulta = useCanciones();
  const canciones = consulta.data ?? [];
  return {
    canciones: canciones.map(aFilaCancion),
    cargando: consulta.isLoading,
    error: consulta.isError,
  };
};
```

- Devuelve un objeto con nombre tipado, no la respuesta cruda de React Query.
- Un fallo **no borra** lo que ya había: React Query conserva `data` en un
  refetch fallido.

## Sin modo mock

No hay rama de datos de ejemplo en `comandos.ts`. Fuera de Tauri el `invoke`
rechaza y la pantalla enseña el estado de error. La prueba real del
transporte es `npm run tauri dev`, que lanza el usuario.

## Sí / No

| Sí | No |
|---|---|
| Tipo exacto al `struct` y a `cancion.json` | Inventar un campo "obvio" |
| Un envoltorio por comando, en `common/api/` | `invoke` o `new Channel` dentro de una feature |
| Frames del juego en ref o store leído por el canvas | `useState` con cada frame |
| `enabled` para apagar una consulta sin argumento | Pedir `cancion("")` y tratar el error |
| Invalidar en `consultas.ts` | Invalidar "por si acaso" desde una pantalla |
