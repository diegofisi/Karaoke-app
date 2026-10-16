# Fábrica

Proceso Python que prepara cada canción (descarga, separación de voz, letra,
alineación por palabra, tono y notas) y escribe `cancion.json`. La lanza la app
(`src-tauri`); también se puede usar sola. Protocolo: `docs/CONTRATO.md` §4.

Requisitos: Python 3.12, `ffmpeg`, y un runtime de JavaScript (Node, Deno o
Bun) en el PATH. YouTube exige resolver un desafío en JavaScript; sin runtime,
la descarga falla con HTTP 403. Node ya hace falta para la app.

## Instalar

Windows con NVIDIA (CUDA 13; el torch de PyPI en Windows es solo CPU):

```powershell
py -3.12 -m venv fabrica\.venv
fabrica\.venv\Scripts\python -m pip install torch==2.14.1 torchaudio==2.11.0 torchvision==0.29.1 --index-url https://download.pytorch.org/whl/cu130
fabrica\.venv\Scripts\python -m pip install -e "fabrica[dev]"
```

Mac con Apple Silicon (usa la GPU de Apple, MPS; necesita macOS 14 o posterior):

```bash
brew install python@3.12 ffmpeg
python3.12 -m venv fabrica/.venv
fabrica/.venv/bin/python -m pip install torch torchaudio torchvision
fabrica/.venv/bin/python -m pip install -e "fabrica[dev]"
```

Los modelos se descargan al primer uso: el separador en `~/.karaoke/modelos`
(o `KARAOKE_MODELOS`), Qwen3 en la caché de Hugging Face. Unos 7 GB en total.

## Usar

```
python -m karaoke_fabrica diagnostico
python -m karaoke_fabrica procesar --url <youtube> --salida <carpeta>
python -m karaoke_fabrica procesar --audio <archivo> --titulo T --artista A --salida <carpeta> [--letra letra.txt]
```

## Probar y medir

```
python -m pytest fabrica                     # pruebas sin modelos ni GPU
python eval/medir.py --idiomas en es --n 5   # tiempo por palabra contra JamendoLyrics
```

Estado en Mac: preparado, sin probar (el desarrollo se hace en Windows).
