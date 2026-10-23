# Estado de la reescritura

Última sesión: 2026-10-22. Se paró a petición del usuario (memoria del equipo
llena). Este archivo dice dónde retomar.

## Hecho y verificado

| Pieza | Estado | Comprobación |
|---|---|---|
| Código viejo | Guardado en la rama local `legado-python` (sin push) | `git log legado-python` |
| `docs/` | `REVISION-2026-10-15.md`, `ARQUITECTURA.md`, `CONTRATO.md` (§1-§7 completos) | — |
| `.claude/` | Skills `backend`, `frontend`, `fabrica`, `bind-project`; reglas, hooks, agentes, `settings.json` | hooks pasan `node --check` |
| `fabrica/` | Pipeline completo: descarga, separación (Mel-RoFormer), letra (LRCLIB con reintento y caché), alineación, tono (SwiftF0), notas, `cancion.json` | `pytest`: 23 pruebas en verde; probado de punta a punta con una canción real |
| `crates/karaoke-core`, `tono`, `puntuacion`, `audio`, `biblioteca` | Completos | `cargo test --workspace` en verde (~75 pruebas), `clippy` 0 avisos, `fmt` limpio; Rust lee el `cancion.json` real |
| `eval/medir.py` | Medida contra JamendoLyrics (20 en + 20 es, marcas a mano) | resultados en `eval/resultados/` |

Entorno de la fábrica: `fabrica/.venv` (Python 3.12, torch 2.14.1+cu130 para la
RTX 5070 Ti; el torch de PyPI es solo CPU en Windows y hay que instalar desde
el índice cu130). Modelos en `~/.karaoke/modelos` y en la caché de Hugging Face.

## Tiempo por palabra: medidas (10 canciones, 2 197 palabras)

| Versión | Qué hace | Inicios a ≤100 ms | Mediana |
|---|---|---|---|
| v1 | Alinear la canción entera de una vez | 26,8 % | 2 100 ms |
| v2 | Anclas con el reconocedor (con la letra como contexto) | 30,6 % | 3 391 ms |
| v3 | Reconocedor por trozos de 15-30 s, sin contexto | 64,1 % | 68 ms |
| v4 | + alineación por grupos acotados por anclas | 66,4 % (es 70,9 %) | 64 ms |
| v5 | + rondas que realinean lo no confiable entre vecinas confiables; respaldo sobre tramos con voz | **67,5 %** (es 70,2 %); a ≤300 ms 85,4 %; error medio 353 ms (v4: 654) | 62 ms |
| v6a | + fin de palabra alargado mientras suena la voz (coloreado) | 67,5 %; error del fin 380 → 356 ms | 62 ms |
| v6b | confianza mínima 0,65 (descartada: empeora) | 66,1 % | 65 ms |
| **v6f** (en uso) | + segunda opinión MMS (CTC) en las líneas que Qwen no alinea, juzgada sin anclas, umbral 0,05 | **70,6 %**; líneas aproximadas 44 → 16; error medio 311 ms | 59 ms |

Validación en 10 canciones **no usadas para ajustar** (`--desde 5`): v5 79,2 %
→ v6f **81,7 %** a ≤100 ms; error medio 285 → 173 ms; líneas aproximadas 27 → 8.

Limpieza de letra (2026-10-15): marcas de repetición ("x 3") expandidas y
marcas sueltas o de sección quitadas. En "Light It Up": 4 → 2 líneas
aproximadas.
| techo medido | Alinear con tramos de ~20 s bien colocados | 70,5 % (1 canción) | 56 ms |

Lecciones (ya en el código y en sus comentarios):
- Con la letra como contexto, Qwen3-ASR la copia entera con tiempos inventados.
- El alineador pierde precisión al final de tramos largos: trozos de 15-30 s.

## A medias (se paró en mitad del trabajo)

1. ~~v5 de la alineación~~: medida, mejora, se queda (tabla de arriba).
2. **Medida sobre la mezcla** (`--alinear-sobre mezcla`): sin hacer.
3. ~~Interfaz React~~: completa (biblioteca, procesar, juego, resultados,
   ajustes con calibración); `npm run build` pasa.
4. ~~`src-tauri/`~~: completo (15 comandos, lanzador de la fábrica para
   Windows y Mac, hilo del juego, calibración). Revisado por el agente
   `verifier`; sus 6 hallazgos (procesos huérfanos al cerrar, carreras entre
   juego y calibración, récord falso de una partida descartada) corregidos.
   `npm run verify` → LISTO.
5. ~~Primera prueba real~~: la app arranca. Primer fallo encontrado y
   corregido: YouTube devolvía 403 porque yt-dlp no tenía runtime de JavaScript.
6. ~~Grabar mi voz~~ (pedido del usuario): interruptor en el inicio de la
   partida y en Ajustes; MP3 con lo que sonaba + tu voz en Música/Karaoke;
   sección Grabaciones (abrir, mostrar en carpeta, borrar). Revisado por
   `verifier`, hallazgos corregidos. Falta probarla con micrófono real.

## Windows y Mac

- Fábrica: CUDA en Windows, MPS (GPU de Apple) en Mac; instalación por sistema
  en `fabrica/README.md`.
- `src-tauri`: ruta del Python del venv por sistema; cancelar mata el árbol de
  procesos (`taskkill /T` en Windows, grupo de procesos en Mac);
  `Info.plist` con el permiso de micrófono para macOS; el audio de cpal se
  crea dentro de su hilo (en macOS no se puede mover de hilo).
- Interfaz: WebView2 en Windows, WebKit en Mac (`vite.config.ts` compila para
  los dos).
- Sin probar en Mac: el desarrollo se hace en Windows.

## Decisión del usuario (2026-10-15): uso personal, solo lo mejor

La app no se venderá. Se elige siempre lo que mejor mida, sin importar el
tiempo invertido ni que la licencia sea no comercial. Python se queda en la
fábrica: es donde están los mejores modelos y donde más rápido se mide. Esto
abre candidatos que antes estaban descartados, y cada uno se mide con `eval/`:

| Etapa | Candidato | Por qué podría ser mejor |
|---|---|---|
| Alineación | MMS (`ctc-forced-aligner`), y wav2vec2 por idioma | Otro tipo de alineador (CTC + Viterbi); comparar o combinar con Qwen3 en las líneas que fallan |
| Separación | Modelos "karaoke" que conservan los coros en el instrumental | El instrumental suena como un karaoke real; la voz guía queda solo con la voz principal |
| Separación | Ensamble de varios RoFormer | Más limpieza de voz → mejor tono de referencia |
| Tono | RMVPE frente a SwiftF0 | Diseñado para voz con música; hay que medir el tono contra una referencia (falta ese script) |

## Después

- Objetivo de la alineación: más del 80 % de inicios a ≤100 ms. Ideas sin
  probar: confianza real a partir de las probabilidades del alineador (hoy usa
  `argmax`), medir las 40 canciones y no solo 10.
- Japonés: sin medir (JamendoLyrics no trae japonés); hace falta un conjunto
  propio marcado a mano.
- Fase 4 (palabras en directo con sherpa-onnx + Nemotron 3.5) y fase 5
  (empaquetar la fábrica con su Python).
- Nada de esto está commiteado. El usuario decide cuándo.

## Sin requisitos previos (2026-10-16, versión 0.2.0)

- **Comprobación antes de procesar**: la fábrica (Python, PyTorch, modelos),
  ffmpeg, el runtime de JavaScript y la GPU, con ✓/✗ y qué hacer. El botón
  Procesar queda bloqueado hasta que lo obligatorio esté en verde (también
  mientras se comprueba).
- **Herramientas dentro de la app**: ffmpeg 9.0.2, Deno 2.9.7 y uv 0.12.19
  (sidecars; `scripts/descargar-binarios.mjs`, SHA-256 fijados).
- **Fábrica autoinstalable**: «Instalar la fábrica» descarga con uv Python
  3.12, PyTorch 2.14.1 (CUDA / CPU / MPS), la fábrica y los modelos en
  `<datos>/entorno/`. Probado desde cero en Windows (5/5 pasos) y procesando
  una canción de YouTube con un PATH limpio (sin ffmpeg ni Node del sistema).
- Sin probar en un Mac real.

## Memoria según el equipo (2026-10-16, versión 0.2.2)

Sin perfiles fijos: antes de cargar cada modelo se mira la memoria LIBRE en
ese momento (`fabrica/karaoke_fabrica/recursos.py`) y se compara con lo que
necesita (medido en GPU, bfloat16): separador 2,0 GB, reconocedor 5,8 GB,
alineador 1,9 GB, MMS 1,8 GB. Si cabe, a la GPU; si no, a la CPU (bfloat16:
el reconocedor en float32 ocupaba 12 GB de RAM y colgaba equipos de 8-16 GB).

| Prueba | Resultado (inicios a ≤100 ms) |
|---|---|
| Validación, 10 canciones, todo en GPU | 81,7 % (igual que antes) |
| GPU con 4 GB libres: reconocedor en CPU, resto en GPU (1 canción) | 86,5 % (igual que en GPU) |
| Todo en CPU con bfloat16 (1 canción) | 86,5 % |
| Sin GPU: separador Kim_Vocal_2 (validación) | 80,5 % |
| Reconocedor 0,6B (descartado) | 67,1 % (español 43,5 %) |
| Avance por grupos de 2 tramos (descartado) | 79,3 %: cambiaba los lotes; grupos de 8 = idéntico |

## YouTube como en youtube-downloader (2026-10-22, versión 0.2.4)

- **Sesión**: solo la ventana de inicio de sesión de la app
  (`<datos>/youtube/cookies.txt`). Se quitaron `Ajustes.youtube_cookies`,
  `youtube_navegador` y `--navegador` de la fábrica: en Windows yt-dlp no
  lee Chrome/Edge/Brave y el archivo a mano confundía.
- **Vista previa** (`src-tauri/src/vista_previa.rs`, comando `analizar_url`):
  `yt-dlp -J --flat-playlist` antes de procesar. Un vídeo con `&list=` es un
  vídeo; solo `/playlist?list=` es lista. En la lista, casillas por vídeo
  (máximo 25 por lote, los privados o de más de 15 min deshabilitados) y un
  trabajo de la cola por vídeo elegido. Si la sesión guardada falla, se
  repite sin ella y la interfaz la renueva en silencio.
- **Requisitos e instalación** de la fábrica viven en Ajustes → Fábrica;
  Procesar solo avisa con "Ir a Ajustes" si falta algo.
- Pendiente: medir en PC de gama baja (sin GPU) con `eval/` el tiempo total
  por canción, además del porcentaje a ≤100 ms, para decidir si hace falta
  un separador aún más ligero.
