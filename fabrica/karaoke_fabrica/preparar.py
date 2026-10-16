"""Descarga todos los modelos de antemano (CONTRATO seccion 4, orden `preparar`).

La app lo ejecuta al instalar la fabrica, para que la primera cancion no tenga
que esperar ~8 GB de descargas. Cada paso emite
{"tipo": "preparar", "paso": ..., "avance": 0-1, "mensaje": ...}. Un modelo ya
descargado no se vuelve a bajar (caches de Hugging Face, torch y audio-separator).
"""

from __future__ import annotations

from .eventos import emitir, log

PASOS = ("separador", "reconocedor", "alineador", "mms")


def _paso(paso: str, avance: float, mensaje: str) -> None:
    log.info("[preparar %s %.0f%%] %s", paso, avance * 100, mensaje)
    emitir("preparar", paso=paso, avance=round(avance, 3), mensaje=mensaje)


def preparar() -> None:
    """Descarga todos los modelos, incluidos los dos separadores: cual se usa
    depende de la memoria libre en cada momento (recursos.py)."""
    from huggingface_hub import snapshot_download

    from . import alineacion, separacion
    from .recursos import ASR_GRANDE, SEPARADOR_LIGERO, SEPARADOR_PRECISO

    from audio_separator.separator import Separator

    for i, modelo in enumerate((SEPARADOR_PRECISO, SEPARADOR_LIGERO)):
        _paso("separador", i / 2, f"Descargando el separador de voz {modelo}")
        separador = Separator(output_dir=str(separacion.carpeta_modelos()), model_file_dir=str(separacion.carpeta_modelos()))
        separador.load_model(model_filename=modelo)
        del separador
    _paso("separador", 1.0, "Separadores listos")

    _paso("reconocedor", 0.0, f"Descargando {ASR_GRANDE} (~4,4 GB)")
    snapshot_download(ASR_GRANDE)
    _paso("reconocedor", 1.0, "Reconocedor listo")

    _paso("alineador", 0.0, f"Descargando {alineacion.MODELO} (~1,8 GB)")
    snapshot_download(alineacion.MODELO)
    _paso("alineador", 1.0, "Alineador listo")

    _paso("mms", 0.0, "Descargando el alineador MMS (~1,2 GB)")
    import torchaudio

    torchaudio.pipelines.MMS_FA.get_model(with_star=False)
    _paso("mms", 1.0, "Alineador MMS listo")
