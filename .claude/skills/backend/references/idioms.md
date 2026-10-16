# Idiomas de Rust que importan aquí

**Aplica siempre.** Lo que hay en este repo: structs serde que cruzan a la
interfaz y a la fábrica, DSP sobre `&[f32]`, audio con `cpal`, hilos con
canales, `anyhow`. Lo que no hay: HTTP, traits de repositorio, SQL (de
momento).

## Propiedad y préstamo

- **Presta por defecto** en los crates: `&Path`, `&str`, `&[f32]` para
  muestras, `&Cancion` para leer.
- **Excepción obligada**: un `#[tauri::command]` recibe argumentos **por
  valor** porque Tauri los deserializa. Presta hacia abajo.
- **`.clone()` es una decisión.** Clonar un `Cancion` entero por frame es un
  hallazgo; clonar un `Arc` para pasarlo a un hilo no.
- Datos de solo lectura compartidos entre hilos (la canción cargada, la curva
  de tono): `Arc<T>`, sin `Mutex`.

## Números: tiempo, tono, muestras

- Tiempo de la canción: **segundos en `f64`** (CONTRATO §1). Muestras:
  `u64`. La conversión es una función con nombre (`muestras_a_segundos(n,
  frecuencia)`), no una división suelta repetida.
- Tono: **MIDI `f64`** (`69.0` = La 440). `Option<f64>`: `None` = sin voz.
  Nunca `0.0` ni `-1.0` como "sin voz".
- Muestras de audio: `f32`. No mezcles `f32` y `f64` en una misma fórmula sin
  `as` explícito.
- Comparar `f64` con `==` es un bug salvo contra un valor que tú escribiste;
  usa tolerancia.
- Constantes con unidad en el nombre: `PASO_MS: f64 = 10.0`,
  `VENTANA_MUESTRAS: usize = 2048`.

## `Option` y `Result` con combinadores

`and_then`, `map`, `unwrap_or`, `let ... else` para la salida temprana, `?`
antes que `match Err`.

## Enums que cruzan

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoNota { Normal, Dorada, Libre }
```

Representación serde **explícita** para que la unión de TypeScript y el
`Literal` de Python sean exactos. `match` exhaustivo sin `_ =>` sobre un enum
del dominio: añadir una variante debe romper la compilación donde se decide
algo.

## Iteradores antes que bucles con acumulador

Un `for` con `push` es un `map().collect()`. **Excepción**: en el hilo de
audio y en el bucle del juego, un `for` sobre un buffer reservado de antemano
es lo correcto; un `collect()` ahí reserva memoria.

## Tiempo real

- Nada de `Vec::new()`, `format!`, `String`, `Box::new` ni `println!` dentro
  del callback de `cpal`. Los buffers se reservan antes de abrir el stream.
- Comunicación con el callback: ring buffer sin bloqueos (`rtrb`, `ringbuf`)
  o atómicos (`AtomicU64` para el reloj, `AtomicBool` para parar).
- Ver `state-and-threads.md` → Hilo de audio.

## Visibilidad mínima

`pub` solo en lo que `src-tauri` o los tests llaman. Ayudas internas privadas;
los tests unitarios las alcanzan por `use super::*`.

## Atributos y derives

- Tipos de `cancion.json`: `Debug, Clone, Serialize, Deserialize`, con
  `#[serde(rename_all = "snake_case")]` aunque los campos ya lo sean (deja
  clara la regla).
- `#[serde(default)]` en un campo nuevo para que las canciones viejas abran.
- `#[allow(clippy::...)]` solo con un comentario de por qué, nunca para que
  pase la verificación.

## Lo que NO aplica aquí

HTTP, auth, traits de repositorio, pools de conexiones, `async` en el camino
del audio. Si un cambio "necesita" uno de estos, es una decisión de
arquitectura que se escribe en `project.md` antes de codificar.
