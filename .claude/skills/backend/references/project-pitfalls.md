# Libro de errores — karaoke, backend Rust

Este es el libro de errores de **este repositorio**: una línea fechada por cada
error ya pagado aquí. `pitfalls.md` tiene lo que es verdad de cualquier backend
Rust + Tauri + audio; este archivo, lo que solo es verdad aquí.

La skill lee los dos antes de terminar cualquier cambio. Un cambio que
reintroduce una línea de este archivo no está terminado.

## Cómo crece

Añade una línea cuando un bug llegó al código y hubo que arreglarlo, o cuando
el usuario corrigió lo mismo dos veces. No añadas opiniones de estilo,
repeticiones de `pitfalls.md`, ni nada que un tipo, un lint o un test ya
impidan: si se puede hacer ejecutable, hazlo ejecutable.

Formato: `AAAA-MM-DD — la regla, en imperativo — por qué: qué se rompió — dónde`.

Las entradas del 2026-10-15 vienen de las dos versiones anteriores (rama
`legado-python` y `TranslatorMnga/karaoke-app`), revisadas en
`docs/REVISION-2026-10-15.md`. El código nuevo todavía no existe; son reglas
para no repetirlas.

## Entradas

- 2026-10-15 — No asignes tiempos de palabra en Rust ni los deduzcas del tono:
  los tiempos vienen de `cancion.json`, alineados por la fábrica — por qué: en
  `karaoke-app`, `crates/chartgen` repartía las palabras por tramos con voz y,
  si faltaban, a partes iguales; con legato salía 9/9/8/9/9 beats en "Cool
  wind in my hair" — dónde: `crates/karaoke-core`, `crates/puntuacion`.
- 2026-10-15 — Puntúa por frames de audio de 10 ms con su marca de tiempo,
  nunca por frame de pantalla ni por mensaje del `Channel` — por qué: la
  versión Python sumaba un punto por `requestAnimationFrame` y una pantalla de
  144 Hz puntuaba distinto que una de 60 Hz — dónde: `crates/puntuacion`,
  hilo del juego en `src-tauri`.
- 2026-10-15 — Captura el micrófono en Rust con entrada en bruto, sin
  cancelación de eco, supresión de ruido ni ganancia automática — por qué:
  `getUserMedia` con `echoCancellation`, `noiseSuppression` y
  `autoGainControl` distorsionaba el tono y añadía retraso — dónde:
  `crates/audio`.
- 2026-10-15 — No normalices texto quitando lo que no sea `a-z0-9`; compara
  con normalización Unicode (NFKC + minúsculas) y conserva kana y kanji — por
  qué: la normalización `[^a-z0-9]` dejaba vacías todas las palabras japonesas
  y no puntuaban — dónde: cualquier comparación de palabras (fase 4,
  `crates/puntuacion`).
- 2026-10-15 — Juego y calibración usan el mismo audio: compruébalo y ocúpalo
  bajo un solo lock (`estado.audio: UsoAudio`), nunca con dos banderas
  separadas — por qué: el revisor encontró que iniciar_juego y
  calibrar_latencia podían abrir el audio a la vez, y que una partida
  descartada guardaba 0 puntos como récord — dónde: `src-tauri/src/estado.rs`,
  `comandos/juego.rs`, `comandos/ajustes.rs`.
- 2026-10-15 — Todo proceso hijo (la fábrica) se mata al cerrar la app
  (`RunEvent::Exit` → `cerrar_todo`) y si falla su hilo vigilante — por qué:
  Python con torch cargado seguía vivo tras cerrar la ventana — dónde:
  `src-tauri/src/lib.rs`, `comandos/fabrica.rs`.
- 2026-10-16 — El id de una canción local se calcula sobre el archivo
  COMPLETO — por qué: con solo los primeros 4 MiB, dos MP3 del mismo disco
  (carátula grande al inicio) podían compartir id y reutilizar el audio de
  la otra canción — dónde: `src-tauri/src/fabrica.rs` (`id_de_archivo`) y
  `fabrica/karaoke_fabrica/pipeline.py` (`id_cancion`), que deben coincidir.
