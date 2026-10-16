# Estado de la reescritura

Última sesión: 2026-10-15. Se paró a petición del usuario (memoria del equipo
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
| **v5** (en uso) | + rondas que realinean lo no confiable entre vecinas confiables; respaldo sobre tramos con voz | **67,5 %** (es 70,2 %); a ≤300 ms 85,4 %; error medio 353 ms (v4: 654) | 62 ms |
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
