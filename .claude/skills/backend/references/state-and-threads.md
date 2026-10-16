# Estado compartido e hilos

**Aplica siempre.** Este backend tiene tres clases de trabajo con reglas
distintas: comandos cortos, el hilo de audio de `cpal`, y trabajos largos en
hilos propios (el bucle del juego, la lectura de la fábrica).

## Estado de `src-tauri`

Un struct `Estado` creado en `setup` y registrado con `app.manage(estado)`.
Guarda lo que vive más que un comando: la carpeta de datos, el juego en curso
(su bandera de parada y su hilo), el proceso de la fábrica en curso.

- Lo que se lee y no cambia (rutas): campos normales.
- Lo que cambia (juego en curso, proceso hijo): `Mutex<Option<...>>`.
- Nada de `RwLock`, pools ni `Arc` extra sin un motivo escrito en
  `project.md`. Un usuario, una ventana.

## Locks

- **Alcance corto.** Toma el lock, lee o cambia, suéltalo. Nunca sostengas un
  lock durante I/O, una espera o un `join`.
- **Nunca dos locks a la vez.** Si hacen falta dos datos, toma uno, copia lo
  necesario, suéltalo, toma el otro.
- **`PoisonError`** → `String` "No se pudo acceder al estado: inténtalo de
  nuevo". Nunca `.lock().unwrap()` fuera de un test.
- Nada que pueda hacer `panic!` se ejecuta con un lock tomado.

## Hilo de audio (`cpal`)

El callback de `cpal` corre en un hilo del sistema con prioridad alta. Si
tarda o se bloquea, el audio se corta.

**Dentro del callback, prohibido:**

- reservar memoria (`Vec::new`, `push` que crece, `String`, `format!`, `Box`),
- tomar un `Mutex` o cualquier lock,
- `println!`, `log`, escribir en disco,
- mandar por un `Channel` de Tauri o por un `std::sync::mpsc` que reserve,
- llamar a la detección de tono o a la puntuación.

**Dentro del callback, permitido:**

- copiar muestras a/desde un ring buffer sin bloqueos reservado antes
  (`rtrb`, `ringbuf`),
- sumar al contador de muestras (`AtomicU64`, `Ordering::Release`),
- leer una bandera (`AtomicBool`) para silenciar o parar.

Salida: el callback del dispositivo de salida copia del buffer del
instrumental y suma las muestras consumidas al reloj. **Ese contador es el
reloj del juego** (ARQUITECTURA §4).

Entrada: el callback del micrófono copia el bloque a un ring buffer junto con
su marca (muestras del dispositivo de entrada contadas hasta ese bloque).
Entrada en bruto: sin filtros del sistema (cancelación de eco, supresión de
ruido, ganancia automática) — `project-pitfalls.md`.

## Hilo del juego

Un hilo propio, creado por el comando que empieza el juego:

1. Saca bloques del ring buffer del micrófono.
2. Convierte su marca a segundos de la canción y resta `latencia_ms`.
3. Cada 10 ms de audio: `tono` → MIDI; `puntuacion` → puntos de la nota que
   toca en ese instante.
4. Cada ~16 ms de reloj de pared: manda un frame por el `Channel`
   (`events.md`) con `t` leído del contador de salida.
5. Sale cuando la bandera de parada se activa, la canción termina o `send`
   falla. El último mensaje dice cómo terminó.

Este hilo sí puede reservar memoria, pero al arrancar, no por frame.

## Proceso de la fábrica

- `std::process::Command` con `stdout(Stdio::piped())` y
  `stderr(Stdio::piped())`. En Windows, `creation_flags(CREATE_NO_WINDOW)`.
- Un hilo lee `stdout` con `BufReader::lines()`; otro vacía `stderr` al
  registro. Si nadie lee `stderr`, el buffer del pipe se llena y la fábrica
  se queda bloqueada.
- El `Child` se guarda en el estado para poder cancelarlo (`kill` + `wait`).
  Al cerrar la app, se mata.
- Los argumentos van por argumento (CONTRATO §4 los fija); un texto largo
  (letra) va en un archivo (`--letra <archivo>`), no en la línea de comandos.

## Comandos largos con un solo resultado

`async fn` + `tauri::async_runtime::spawn_blocking`. **No sostengas un
`MutexGuard` a través de un `.await`**: no es `Send` y, aunque compilara,
bloquearía los demás comandos.

## Cancelación

Todo hilo largo tiene una forma de parar: `Arc<AtomicBool>` guardado en el
estado, que el comando de parar activa y el hilo comprueba en cada vuelta. El
comando de parar no hace `join` con un lock tomado.
