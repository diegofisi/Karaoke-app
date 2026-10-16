# Arquitectura

Fecha: 2026-10-15. Reescritura completa. La versión anterior (Python + React en
el navegador) está en la rama `legado-python`. El porqué de cada decisión está
en `docs/REVISION-2026-10-15.md`.

## 1. Dos mitades

| Mitad | Cuándo corre | Lenguaje | Qué hace |
|---|---|---|---|
| **Fábrica** | Antes de jugar, una vez por canción (1 a 3 minutos con GPU) | Python, proceso aparte | Descarga, separa la voz, consigue la letra, la **alinea** palabra por palabra, saca el tono de referencia y las notas. Escribe `cancion.json` |
| **Juego** | Mientras se canta, en tiempo real | Rust | Reproduce el instrumental, captura el micrófono, detecta el tono cada 10 ms, puntúa. Manda frames a la interfaz |
| Interfaz | Siempre | React | Solo pinta. No toca audio |

Regla: todo lo lento (modelos de IA) va en la fábrica. Durante el juego solo
corre DSP ligero en Rust.

## 2. Por qué la fábrica es Python

Lo que falló en las dos versiones anteriores fue la fábrica: el tiempo por
palabra y el tono de referencia. Los mejores modelos para eso existen en Python
y con GPU:

| Etapa | Herramienta | Licencia |
|---|---|---|
| Descarga | `yt-dlp` | Unlicense |
| Separación de voz | `audio-separator` con Mel-RoFormer de Kimberley Jensen | MIT (código y pesos) |
| Letra | LRCLIB; si no hay, la letra la da el usuario; si tampoco, Qwen3-ASR transcribe | — / Apache-2.0 |
| Alineación por palabra | **Qwen3-ForcedAligner-0.6B** (`qwen-asr`) | Apache-2.0 |
| Tono de referencia | **SwiftF0** | MIT |
| Japonés: palabras, lecturas y romaji | `fugashi` + `unidic-lite` + `cutlet` (no `pykakasi`, que es GPL-3) | MIT / BSD / MIT |

Reescribir esos modelos en Rust fue la causa de la versión débil anterior. Si
algún día hace falta quitar Python, se exportan a ONNX y se cambia **solo** la
fábrica: el contrato `cancion.json` no cambia.

## 3. Carpetas

```
Karaoke/
├── CLAUDE.md                 punteros para Claude Code
├── .claude/                  skills, reglas, hooks, agentes
├── docs/                     CONTRATO.md (fuente de verdad), ARQUITECTURA.md, REVISION-*.md
├── fabrica/                  Python: karaoke_fabrica/ + tests/ + pyproject.toml
├── eval/                     medida del tiempo por palabra y del tono
├── crates/
│   ├── karaoke-core/         tipos de cancion.json (serde) y su lectura
│   ├── tono/                 detector MPM para el micrófono (puro)
│   ├── puntuacion/           puntuación por nota y por línea (puro)
│   ├── audio/                cpal: reproducción con reloj por muestras, micrófono, calibración
│   └── biblioteca/           carpeta de canciones y récords
├── src-tauri/                pegamento: comandos, estado, lanza la fábrica, bucle del juego
└── src/                      React 19 + MUI + React Query + Zustand
```

## 4. Tiempo: una sola fuente de verdad

- El reloj es el **número de muestras que el dispositivo de salida ya
  consumió** (`crates/audio`). No `currentTime` de un `<audio>`.
- Cada bloque del micrófono lleva su marca en muestras del dispositivo de
  entrada.
- `latencia_ms` (calibrada con pitidos) se resta al tiempo del micrófono antes
  de compararlo con la canción. Así el usuario no pierde puntos al inicio de
  cada nota.
- La interfaz recibe `t` en cada frame y lo interpola con `performance.now()`
  entre frames. Nunca lleva su propio reloj.

## 5. Medir antes de decidir

`eval/` mide el error del inicio de palabra en milisegundos contra marcas
hechas a mano (JamendoLyrics y canciones propias). Un cambio en la fábrica que
no mejora el número no entra. Objetivo de partida: más del 80 % de los inicios
de palabra dentro de ±100 ms.

## 6. Fases

| Fase | Contenido | Estado |
|---|---|---|
| 1 | Fábrica completa + medida | en curso |
| 2 | Crates Rust: core, tono, puntuación, audio, biblioteca | pendiente |
| 3 | Tauri + interfaz: biblioteca, procesar, juego, resultados, ajustes | pendiente |
| 4 | Palabras en directo: sherpa-onnx + Nemotron 3.5, bono por palabra | pendiente |
| 5 | Empaquetado: fábrica con su Python embebido, descarga de modelos | pendiente |
