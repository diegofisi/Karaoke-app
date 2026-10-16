# Comandos Tauri (`#[tauri::command]`)

**Aplica siempre.** Viven en `src-tauri/src/comandos/`, se registran en
`src-tauri/src/lib.rs`, y la interfaz los llama por nombre exacto desde
`src/common/api/comandos.ts`.

## Forma canónica

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn cancion(estado: State<Estado>, id: String) -> Result<Cancion, String> {
    let carpeta = estado.carpeta_canciones();
    biblioteca::leer_cancion(&carpeta, &id).map_err(|e| e.to_string())
}
```

Tres líneas: tomar lo que hace falta del estado, llamar al crate, convertir el
error. Si un comando tiene más, pregunta qué parte debería estar en un crate.

Reglas:

- **Fino.** Decodifica argumentos → UNA llamada a un crate → devuelve. Si hay
  que combinar dos llamadas, la combinación va en el crate.
- **Siempre `Result<T, String>`** con el mensaje en español que empieza por
  "No se pudo…" (CONTRATO §1, `errors.md`). Un `panic!`, `unwrap()` o
  `expect()` en un comando mata el proceso.
- **`State<Estado>`** solo si se usa.
- **`rename_all = "snake_case"`** en todos: la interfaz manda `latencia_ms`
  tal cual. Sin ese atributo, Tauri esperaría `latenciaMs`. No mezcles.
- **Argumentos por valor** (`id: String`): Tauri los deserializa. El comando
  presta hacia abajo.
- **Registro obligatorio** en `tauri::generate_handler![...]`. Sin registro,
  el `invoke` falla en tiempo de ejecución con "command not found".
- **Permisos**: un comando propio no necesita entrada en `capabilities/`.
  Solo las APIs de plugin o de `core:*` que la interfaz llame directamente.

## Comandos que abren un canal

Para datos continuos (progreso de la fábrica, frames del juego) el comando
recibe un `tauri::ipc::Channel<T>` y devuelve en cuanto el trabajo arranca:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn procesar_cancion(app: AppHandle, url: String, canal: Channel<EventoFabrica>) -> Result<(), String> {
    fabrica::lanzar(&app, url, canal).map_err(|e| e.to_string())
}
```

- El trabajo corre en un hilo propio; el comando no espera a que termine.
- El `Channel` se mueve al hilo. Cuando el hilo termina, el canal se suelta.
- Un error al **arrancar** (no se encuentra Python) es el `Err` del comando.
  Un error **durante** el trabajo viaja por el canal (`tipo: "error"`).
- Para parar: un comando aparte (`detener_juego`, `cancelar_proceso`) que
  marca una bandera atómica o mata el proceso hijo. Detalle en
  `state-and-threads.md` y `events.md`.

## Duración

| Duración | Qué hacer |
|---|---|
| < ~100 ms (leer una carpeta, un `cancion.json`) | Síncrono, como arriba |
| Largo con un solo resultado | `async fn` + `tauri::async_runtime::spawn_blocking` |
| Largo con avance o datos continuos (fábrica, juego) | Hilo + `Channel` (`events.md`) |

Nunca `std::thread::sleep` en un comando.

## Un comando, un lugar donde se decide el error

El crate devuelve `anyhow::Result` con el contexto ya en español; el comando
solo hace `.map_err(|e| e.to_string())`. No añadas un segundo `with_context`
en el comando: produce "No se pudo X: No se pudo X".

## Añadir un comando: los sitios

1. La función en el crate, con su test.
2. `src-tauri/src/comandos/{modulo}.rs`: el comando.
3. `lib.rs`: la línea en `generate_handler!`.
4. `docs/CONTRATO.md` §5 (o §6 si abre canal): la fila.
5. `src/common/api/comandos.ts`: el envoltorio; hook en `consultas.ts` si es
   consulta o mutación.
6. `src/common/interfaces/`: la interfaz, si el tipo es nuevo.

Mismo cambio. Cuenta los sitios; `check-rules` (previsto) detectará parte.
