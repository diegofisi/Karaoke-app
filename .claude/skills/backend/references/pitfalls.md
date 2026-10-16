# Trampas universales — Rust + Tauri + audio en tiempo real

**Aplica siempre.** Léelo antes de terminar cualquier cambio. Lo específico de
este repo está en `project-pitfalls.md`, fechado.

La forma recurrente: **un camino lo hizo bien y su hermano no.** Al arreglar
algo, busca los otros llamadores de la misma idea.

## Comandos y contrato

1. **Un comando sin registrar en `generate_handler!` compila y falla en
   runtime** con "command not found".
2. **Un struct cambiado en Rust sin su copia en TypeScript (y en Python si es
   de `cancion.json`) compila en todos los lados y falla en runtime.**
3. **Argumentos multi-palabra**: con `rename_all = "snake_case"` la interfaz
   manda `latencia_ms`; sin el atributo, Tauri espera `latenciaMs`.
4. **`unwrap()` en un comando mata la app.** Devuelve un `String`.
5. **`anyhow::Error` no es `Serialize`**: un comando que lo devuelva no
   compila.
6. **`to_string()` de `anyhow::Error` es solo el contexto más externo.**
7. **Nunca `std::thread::sleep` en un comando.**

## Audio en tiempo real

8. **Reservar memoria en el callback de `cpal`** produce cortes que no se ven
   en desarrollo y sí en un portátil cargado. Buffers reservados antes.
9. **Un `Mutex` en el callback** bloquea el audio mientras otro hilo lo tiene.
   Ring buffer sin bloqueos y atómicos.
10. **El reloj del sistema no es el reloj de la canción.** El reloj es el
    contador de muestras consumidas por la salida. `Instant::now()` deriva del
    audio en pocos minutos.
11. **Entrada y salida tienen frecuencias y relojes distintos.** Cada bloque
    del micrófono se convierte con la frecuencia de **su** dispositivo.
12. **El dispositivo puede pedir otro formato** (`i16`, `u16`, 44,1 o 48 kHz,
    1 o 2 canales). Convierte a `f32` mono a la entrada; no asumas.
13. **El usuario oye tarde y el micrófono llega tarde.** Sin restar
    `latencia_ms`, cada nota pierde puntos al empezar.
14. **Puntuar por frame de pantalla** hace que una pantalla de 144 Hz puntúe
    distinto que una de 60 Hz. Se puntúa por tiempo de audio.
15. **Un stream de `cpal` que se suelta deja de sonar.** Guarda el `Stream`
    mientras el juego dure; en Windows (WASAPI) no es `Send`: créalo y
    suéltalo en el mismo hilo.

## Procesos externos

16. **Un `stderr` que nadie lee** llena el pipe y el proceso hijo se bloquea.
17. **Un texto largo por argumento** supera el límite de la línea de comandos
    de Windows. Por archivo o por stdin.
18. **Un proceso hijo sin `kill` al cerrar** sigue usando la GPU después de
    cerrar la app.
19. **Una ventana de consola que se abre** al lanzar Python desde la app:
    `CREATE_NO_WINDOW`.

## Estado e hilos

20. **Lock sostenido durante una llamada lenta** bloquea todos los comandos
    que lo usan.
21. **`MutexGuard` de `std::sync` a través de un `.await`** no compila (no es
    `Send`).
22. **Dos locks en orden distinto en dos sitios** es un interbloqueo.
23. **Un `Mutex` envenenado** hace que todos los `lock()` fallen para siempre.
24. **Capturar `State<'_>` en un hilo no compila; capturar `AppHandle` y
    hacer `app.state()` dentro, sí.**

## Archivos

25. **Escribir un JSON sin tmp + rename** deja un archivo a medias si el
    proceso muere.
26. **Una ruta relativa a `CARGO_MANIFEST_DIR` solo vale en desarrollo y en
    tests.** La app instalada usa `app.path()`.

## Workspace y build

27. **Un binario y una librería con el mismo nombre en el mismo paquete chocan
    en Windows.** La librería lleva sufijo (`_lib`).
28. **Que `cargo check` y `cargo clippy` pasen no garantiza que `cargo test`
    compile.** La verificación corre todos.
29. **`target/` guarda rutas absolutas.** Mover el repo de carpeta exige
    `cargo clean` de los paquetes afectados.
30. **Una dependencia del workspace se declara una vez** en
    `[workspace.dependencies]` y se usa con `workspace = true`.

## Errores

31. **`let _ =` o `.ok()` sin comentario** esconde un fallo real.
32. **Traducir el mismo error en dos capas** produce "No se pudo X: No se pudo
    X".

## Proceso

33. **No aflojes `clippy`, `tsconfig` ni un test para que pase la
    verificación.** Arregla el código.
34. **No lances `npm run tauri dev` ni `npm run dev` desde el agente.** No
    terminan.
35. **`cargo fmt` del repo entero es una tarea propia**, nunca mezclada con un
    arreglo.
36. **No actualices `tauri` ni `cpal` como parte de otra tarea.**
37. **Reporta solo lo que ejecutaste**, con el comando y su salida.
