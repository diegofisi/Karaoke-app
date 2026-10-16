# Transporte Tauri — `invoke`, `Channel`, argumentos, errores, recursos

> **Aplica:** siempre. Los nombres exactos de comandos están en
> `src-tauri/src/lib.rs` y en CONTRATO §5 y §6; `project.md` los tabula.

## `invoke` y `Channel` — solo en `common/api/comandos.ts`

```ts
import { invoke, Channel } from "@tauri-apps/api/core";

export const cancion = (id: string): Promise<Cancion> => invoke("cancion", { id });

export const procesarCancion = (url: string, alMensaje: (m: MensajeFabrica) => void): Promise<void> => {
  const canal = new Channel<MensajeFabrica>();
  canal.onmessage = alMensaje;
  return invoke("procesar_cancion", { url, canal });
};
```

(Nombres ilustrativos; los reales salen de CONTRATO §5 y §6.)

- **Nombre del comando** = el de `generate_handler!`, literal, `snake_case`.
  Un typo es "command not found" en tiempo de ejecución, no en `tsc`.
- **Nombres de argumentos**: todos los comandos llevan
  `rename_all = "snake_case"`, así que se envían con el nombre exacto del
  parámetro de Rust (`{ latencia_ms }`, no `{ latenciaMs }`). También el
  canal: si en Rust se llama `canal`, aquí es `{ canal }`.
- **Payloads de objeto** con la forma exacta del `struct`: `null` explícito
  donde Rust tiene `Option`, todos los campos obligatorios presentes.
- El envoltorio **no captura errores**: devuelve la promesa.
- `invoke<T>` no comprueba nada en tiempo de ejecución. Si el `struct` cambia
  y el `.ts` no, el error aparece al pintar.

## Canales

- `new Channel<T>()` solo en `comandos.ts`. El tipo `T` es el espejo del
  mensaje de Rust en `common/interfaces/`.
- `onmessage` se asigna **antes** de llamar a `invoke`.
- Un canal de alta frecuencia (frames del juego) escribe en un ref o en un
  store sin suscriptores de React (`state.md`). Nunca `setState` de React en
  `onmessage` a 60 Hz.
- El canal no se cierra desde JS: se para con el comando de parar de Rust
  (`detener_juego`, `cancelar_proceso`; nombres por fijar en CONTRATO §5).
- Mensajes de la fábrica (CONTRATO §4): `{ tipo: "etapa" | "aviso" | "listo"
  | "error", ... }`. Unión discriminada en TypeScript; `switch` exhaustivo.

## Errores — `Err(String)` en español

Todos los comandos devuelven `Result<T, String>`. Cuando Rust devuelve `Err`,
`invoke` rechaza con **ese `string`**, en español, que empieza por "No se
pudo…" (CONTRATO §1).

```ts
const mensajeDeError = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);
```

- La interfaz está en español, así que el mensaje de Rust se pinta tal cual.
- Nunca `alert()`, nunca solo `console.error`, nunca un aviso que desaparezca
  solo antes de poder leerlo.
- Un hook no interpreta el texto del error. Si hace falta distinguir casos, el
  cambio es en Rust (un `enum` serializado).
- Un error **durante** un trabajo largo llega por el canal (`tipo: "error"`),
  no como rechazo de la promesa.

## Recursos

- El audio lo reproduce **Rust**. La interfaz no usa `<audio>` ni Web Audio
  para la canción ni para el micrófono. No hay `getUserMedia`
  (`project-pitfalls.md`).
- Si la interfaz necesita un archivo de la carpeta de la canción (una
  portada), se pide a Rust o se usa `convertFileSrc` con el permiso de
  `assetProtocol` en `tauri.conf.json`. Por decidir.
- Ningún recurso se pide a internet desde la interfaz: fuentes locales.

## Ventana y permisos

Los comandos propios no necesitan permiso. Lo que use una API de plugin o de
`core:*` (diálogo de archivo, ventana) necesita su permiso en
`src-tauri/capabilities/`. Un "not allowed" en consola se arregla ahí, no con
`try/catch`. Esas llamadas van en un archivo de `common/api/`, no en un
componente.

## Sin modo demo

`npm run dev` abre la interfaz sin Tauri: todos los `invoke` fallan y cada
pantalla enseña su estado de error. Sirve para maquetar. **No** verifica el
transporte: lo prueba el usuario en `npm run tauri dev`. El agente no lanza
ninguno de los dos.

## Sí / No

| Sí | No |
|---|---|
| `invoke` y `new Channel` en `comandos.ts` | `invoke` en un hook de feature o un componente |
| Argumentos con el nombre del parámetro de Rust | Adivinar camelCase |
| Frames del canal a un ref o store | `setState` en `onmessage` |
| Mensaje de Rust en pantalla tal cual | `alert(error)` |
| Audio y micrófono en Rust | `<audio>`, `getUserMedia` |
