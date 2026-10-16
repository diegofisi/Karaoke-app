"""Separacion de voz e instrumental con audio-separator.

Modelo por defecto: Mel-Band RoFormer de Kimberley Jensen (licencia MIT, SDR de
voz ~11). Deja los coros en la pista de voz: para el karaoke eso significa que
el instrumental queda sin coros. Un modelo "karaoke" que los conserve se puede
elegir con --separador, pero su licencia no esta clara (docs/REVISION).
"""

from __future__ import annotations

import os
from pathlib import Path

from .eventos import ErrorFabrica, etapa, log

MODELO_POR_DEFECTO = "vocals_mel_band_roformer.ckpt"  # el real lo elige recursos.detectar()


def carpeta_modelos() -> Path:
    base = os.environ.get("KARAOKE_MODELOS")
    ruta = Path(base) if base else Path.home() / ".karaoke" / "modelos"
    ruta.mkdir(parents=True, exist_ok=True)
    return ruta


def separar(original: Path, carpeta: Path, modelo: str = MODELO_POR_DEFECTO) -> tuple[Path, Path]:
    voz = carpeta / "voz.wav"
    instrumental = carpeta / "instrumental.wav"
    if voz.exists() and instrumental.exists():
        etapa("separacion", 1.0, "Esta cancion ya estaba separada: se reutiliza")
        return voz, instrumental

    etapa("separacion", 0.0, f"Cargando el separador ({modelo})")
    from audio_separator.separator import Separator

    tmp = carpeta / "separacion.tmp"
    tmp.mkdir(exist_ok=True)
    try:
        sep = Separator(output_dir=str(tmp), output_format="WAV", model_file_dir=str(carpeta_modelos()))
        sep.load_model(model_filename=modelo)
        etapa("separacion", 0.1, "Separando la voz del instrumental (GPU)")
        salidas = [tmp / Path(s).name for s in sep.separate(str(original))]
    except Exception as e:
        raise ErrorFabrica("separacion", f"No se pudo separar la voz: {e}") from e

    def buscar(*nombres: str) -> Path:
        for s in salidas:
            if any(f"({n})" in s.name.lower() for n in nombres):
                return s
        raise ErrorFabrica("separacion", f"El separador no devolvio la pista {nombres[0]}: {[s.name for s in salidas]}")

    buscar("vocals").replace(voz)
    buscar("instrumental", "other", "no_vocals").replace(instrumental)
    for s in tmp.iterdir():
        s.unlink()
    tmp.rmdir()
    log.info("voz: %s, instrumental: %s", voz, instrumental)
    etapa("separacion", 1.0, "Voz e instrumental separados")
    return voz, instrumental
