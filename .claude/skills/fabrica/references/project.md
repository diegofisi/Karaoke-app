# Vinculación — karaoke, fábrica Python

**Estado (2026-10-15):** la fábrica **se está escribiendo ahora** (fase 1 de
ARQUITECTURA §6). Lo que sigue se comprobó mirando la carpeta ese día; puede
haber cambiado. Donde este archivo y el código discrepen, **manda el código**
y se arregla este archivo (o se re-ejecuta `bind-project`).

## Qué es

Proceso Python aparte. Entrada: URL de YouTube o archivo de audio, idioma
opcional, letra opcional. Salida: `<salida>/original.wav`, `voz.wav`,
`instrumental.wav`, `cancion.json`, `fabrica.log` (CONTRATO §2). Comunicación
con Rust: solo líneas JSON por `stdout` (CONTRATO §4).

## Entorno

| Qué | Valor |
|---|---|
| Python | 3.12 (`requires-python = ">=3.12,<3.13"`); el venv tiene 3.12.8 |
| Venv | `fabrica/.venv` (ignorado por git). Hay otro `venv/` en la raíz, de la versión anterior: no se usa |
| GPU | RTX 5070 Ti (Blackwell). Necesita `torch` con CUDA 12.8: se instala **antes** que el resto, desde el índice `cu128` (`--index-url https://download.pytorch.org/whl/cu128`) |
| Paquete | `fabrica/pyproject.toml`, proyecto `karaoke-fabrica`, instalado en el venv en modo editable |
| Tests | `pytest` (extra `dev`), `testpaths = ["tests"]` |
| Formato / lint | ninguno (sin ruff, sin black). La revisión es manual |

## Módulos (`fabrica/karaoke_fabrica/`, 2026-10-15)

| Módulo | Etapa / papel |
|---|---|
| `__main__.py` | Línea de comandos: `procesar` (`--url` o `--audio`, `--salida`, `--titulo`, `--artista`, `--idioma`, `--letra`, `--separador`, `--alinear-sobre voz\|mezcla`) y `diagnostico` |
| `eventos.py` | El único que escribe en el `stdout` real. `preparar_salida` redirige `sys.stdout` a `stderr` y configura el registro a `stderr` + `fabrica.log`. `emitir`, `ErrorFabrica`, `ETAPAS` |
| `modelo.py` | Tipos de `cancion.json` (dataclasses, `Literal`), `VERSION = 1`. Espejo de `crates/karaoke-core` |
| `pipeline.py` | `Peticion`, `id_cancion`, `procesar` (orden de etapas), `diagnostico` |
| `descarga.py` | yt-dlp |
| `separacion.py` | audio-separator, Mel-RoFormer |
| `letra.py` | LRCLIB, letra del usuario (`.txt`, `.lrc`) |
| `alineacion.py` | Qwen3-ForcedAligner-0.6B (`qwen-asr`) |
| `tono.py` | SwiftF0 |
| `notas.py` | Notas por palabra a partir de la curva de tono |
| `texto.py` | Normalización, tokenización japonesa, romaji |

`fabrica/tests/`: `test_puras.py` (funciones sin modelos).

## Comandos

```bash
# desde fabrica/
.venv\Scripts\python -m pytest -q
.venv\Scripts\python -m karaoke_fabrica diagnostico
.venv\Scripts\python -m karaoke_fabrica procesar --url <url> --salida <carpeta>

# desde la raíz
fabrica/.venv/Scripts/python.exe eval/medir.py --idiomas en es [--n 5] [--alinear-sobre voz|mezcla]
```

`procesar` descarga modelos y usa la GPU: tarda minutos. No lo lances en una
sesión salvo que el usuario lo pida.

## `eval/`

| Qué | Dónde |
|---|---|
| Script | `eval/medir.py` |
| Datos | `eval/datos/jamendolyrics/` (JamendoLyrics; ignorado por git) |
| Separaciones reutilizadas | `eval/datos/salidas/` (ignorado por git) |
| Resultados | `eval/resultados/<fecha>-<etiqueta>.json` (se versiona) |
| Métricas | MAE y mediana del inicio de palabra en ms; % dentro de ±50, ±100, ±300 ms; MAE del fin |
| Objetivo de partida | más del 80 % de inicios dentro de ±100 ms (ARQUITECTURA §5) |

La medida de tono contra referencia está prevista y no existe todavía.

## Licencias (registro; uso personal)

La app es de uso personal (decisión del usuario, 2026-10-15): se elige la
herramienta que mejor mida, aunque su licencia no sea comercial. Esta tabla
registra la licencia de cada una por si algún día se distribuye.

| Herramienta | Licencia | En uso |
|---|---|---|
| `yt-dlp` | Unlicense | sí |
| `audio-separator` + Mel-RoFormer (Kimberley Jensen) | MIT (código y pesos) | sí |
| LRCLIB | código MIT; el texto de las letras tiene copyright | sí |
| Qwen3-ASR / Qwen3-ForcedAligner (`qwen-asr`) | Apache-2.0 | sí |
| SwiftF0 (`swift-f0`) | MIT | sí |
| `fugashi` + `unidic-lite` + `cutlet` | MIT / BSD / MIT | sí |
| MMS_FA (torchaudio), segunda opinión de alineación | CC-BY-NC (no comercial) | sí (`alineacion_mms.py`; +0,8 puntos medido el 2026-10-15) |
| `jonatasgrosman/wav2vec2-large-xlsr-53-*` | Apache-2.0; el japonés entrenado con JSUT (no comercial) | candidato a medir |
| RMVPE | posiblemente solo investigación (no verificado) | candidato a medir (tono) |
| Modelos de separación "karaoke" que conservan coros (aufr33/viperx, becruily) | sin licencia localizable | candidato a medir |
| `pykakasi` | GPL-3 | no hace falta (`cutlet` ya cubre romaji) |
| PESTO | código LGPL | no hace falta (SwiftF0 es mejor en el benchmark) |

Fuentes en `docs/REVISION-2026-10-15.md` §9. Una herramienta nueva entra en
esta tabla en el mismo cambio que la añade a `pyproject.toml`.

## Previsto, no existe todavía

- Marcadores de pytest para tests que necesitan red, GPU o modelos (se saltan
  por defecto).
- Medida del tono de referencia en `eval/`.
- El paso de pytest dentro de `npm run verify` (no hay `package.json`).
- Python embebido y descarga de modelos para la app instalada (fase 5).
- Fase 4 (palabras en directo) no es de la fábrica: es Rust (sherpa-onnx).

## Mantenimiento

Se actualiza en el mismo cambio que el código que describe. Re-ejecuta
`bind-project` cuando cambien los módulos, el entorno o `eval/`.
