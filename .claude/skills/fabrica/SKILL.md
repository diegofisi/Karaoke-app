---
name: fabrica
description: >
  Fábrica Python del karaoke (fabrica/, proceso aparte que lanza src-tauri):
  descarga con yt-dlp, separación de voz con audio-separator (Mel-RoFormer),
  letra de LRCLIB o del usuario, alineación palabra por palabra con
  Qwen3-ForcedAligner, tono de referencia con SwiftF0, notas, japonés con
  fugashi + unidic-lite + cutlet, y el escritor de cancion.json. Úsala al
  crear o cambiar fabrica/karaoke_fabrica/*.py (descarga, separacion, letra,
  alineacion, tono, notas, texto, pipeline, eventos, modelo, __main__), el
  protocolo de líneas JSON por stdout (CONTRATO §4: etapa, aviso, listo,
  error, diagnostico), la caché de etapas, fabrica/tests (pytest),
  fabrica/pyproject.toml, el venv fabrica/.venv (Python 3.12, torch cu128),
  eval/ (medida del error de inicio de palabra y del tono contra marcas a
  mano, JamendoLyrics), o al elegir un modelo o una librería nueva (se elige
  por calidad medida; uso personal). Qué existe hoy está en references/project.md, que se lee
  primero.
---

# Fábrica — Python, modelos de IA, `cancion.json`

La fábrica corre **antes** de jugar, una vez por canción (1 a 3 minutos con
GPU). Recibe una URL o un archivo, y deja en la carpeta de la canción los
`.wav` y `cancion.json` (CONTRATO §2 y §3). Rust la lanza como proceso aparte
y lee su `stdout`. Todo lo lento va aquí; el juego no espera a la fábrica.

## Precedencia

`código > docs/CONTRATO.md > references/project.md > esta doctrina`.
`CONTRATO.md` §3 y §4 son la interfaz con Rust: un cambio ahí lo aprueba el
usuario y se hace en todos sus sitios a la vez (`.claude/rules/contrato.md`).

## Reglas duras

1. **`stdout` solo lleva líneas JSON** (CONTRATO §4). Un objeto por línea:
   `etapa`, `aviso`, `listo`, `error`, o `diagnostico`. Solo `eventos.py`
   escribe en el `stdout` real. El registro va a `stderr` y a `fabrica.log`.
   Las librerías que imprimen (yt-dlp, audio-separator, transformers, barras
   de progreso) se redirigen; un subproceso (`ffmpeg`) recibe
   `stdout=subprocess.DEVNULL` o el `stderr`, porque hereda el descriptor 1.
2. **Cada etapa es cacheable.** Si su salida existe y sus entradas no
   cambiaron, se salta. Volver a procesar tras un fallo en `notas` no vuelve a
   descargar ni a separar. La clave de caché incluye la versión de la etapa.
3. **`cancion.json` se escribe al final, atómico** (tmp + `os.replace`). Si no
   existe, la canción no está lista (CONTRATO §2).
4. **Medir antes de decidir** (ARQUITECTURA §5). Un cambio en alineación o en
   tono se mide con `eval/` antes y después, y los números se anotan. Un
   cambio que no mejora el número no entra.
5. **Uso personal: manda la calidad medida, no la licencia** (decisión del
   usuario, 2026-10-15: la app no se venderá). Se permiten modelos no
   comerciales (MMS, RMVPE, modelos "karaoke" de la comunidad, el wav2vec2
   japonés de jonatasgrosman). La licencia de cada herramienta se **anota**
   en la tabla de `references/project.md`, para saber qué cambiar si algún
   día se distribuye la app.
6. **Los tiempos de palabra salen de la alineación forzada**, nunca del tono
   ni de repartir a partes iguales sin marcarlo. Si la alineación falla o da
   confianza baja, la línea va con `metodo: "respaldo"` y se avisa
   (`references/project-pitfalls.md`).
7. **El texto no pierde letras.** Normalización Unicode (NFKC); nunca filtrar
   a `a-z0-9`. Japonés: palabras, `lectura` en hiragana y `romaji` por línea.

## Índice de referencias

| Referencia | Cubre | Léela cuando |
|---|---|---|
| `references/project.md` | Módulos que existen, entorno, comandos, licencias, `eval/`, previsto | Siempre, primero |
| `references/project-pitfalls.md` | Errores ya pagados en las versiones anteriores | Antes de terminar cualquier cambio |

## Workflow A — Cambiar una etapa

1. Lee el módulo de la etapa y `pipeline.py` (orden y caché).
2. Si cambia el resultado de alineación o de tono: ejecuta `eval/` **antes**
   y guarda el número.
3. Haz el cambio. Sube la versión de la etapa para invalidar su caché.
4. Test en `fabrica/tests/` con datos pequeños (señales sintéticas, letras de
   pocas líneas). Sin red ni GPU en los tests por defecto.
5. `eval/` **después**. Los dos JSON de `eval/resultados/` se versionan, y el
   resumen del cambio cita antes, después, conjunto de canciones y opciones
   (`--idiomas`, `--n`, `--alinear-sobre`). Si no mejora, no entra.

## Workflow B — Cambiar `cancion.json` o el protocolo

1. Primero `docs/CONTRATO.md`, aprobado por el usuario.
2. Mismo cambio: `modelo.py`, `crates/karaoke-core`, `src/common/interfaces/`.
3. Si rompe la lectura de canciones ya procesadas, sube `version`.

## Workflow C — Modelo o librería nueva

1. Licencia del código y de los pesos: se anota (no bloquea; uso personal).
2. Añádelo a `pyproject.toml` (lo instala el usuario; `settings.json` no
   bloquea `pip`, pero no instales sin que se pida).
3. Fila en la tabla de licencias de `project.md` y en ARQUITECTURA §2 si
   sustituye a una herramienta.
4. Medida con `eval/` (Workflow A).

## Workflow D — Mensajes al usuario

`etapa` con `avance` 0–1 y `mensaje` en español; `aviso` para lo degradado
no fatal (sin letra en LRCLIB, línea en respaldo); `error` con `mensaje` en
español y código de salida ≠ 0. El `mensaje` lo lee el usuario en la
interfaz: frase completa, sin trazas.

## Validación

Desde `fabrica/`: `.venv\Scripts\python -m pytest -q`. Desde la raíz,
`npm run verify` lo incluirá. Reporta el comando y su salida. Lo que pytest
no comprueba: que `stdout` sigue limpio con los modelos reales (prueba:
`python -m karaoke_fabrica procesar ... 1>salida.jsonl` y que cada línea
parsea), y el número de `eval/`.
