"""Segunda opinion para las lineas que Qwen3-ForcedAligner no consigue alinear.

MMS_FA (torchaudio) es otro tipo de alineador: un modelo CTC por caracteres y
el algoritmo de Viterbi. Falla en otros sitios que Qwen y da una probabilidad
por caracter, que sirve de confianza real. Pesos CC-BY-NC (uso personal: ver
skills/fabrica/references/project.md).

Solo conoce el alfabeto latino sin tildes: es/en se normalizan; el japones se
pasa a romaji palabra por palabra.
"""

from __future__ import annotations

import unicodedata
from functools import lru_cache

import numpy as np

from .eventos import log
from .texto import PalabraTexto, romaji

SR = 16000


@lru_cache(maxsize=1)
def _modelo():
    import torch
    import torchaudio

    paquete = torchaudio.pipelines.MMS_FA
    dispositivo = "cuda" if torch.cuda.is_available() else "cpu"
    log.info("Cargando MMS_FA en %s", dispositivo)
    modelo = paquete.get_model(with_star=False).to(dispositivo)
    return paquete, modelo, dispositivo, set(paquete.get_labels())


def normalizar(texto: str, idioma: str, letras: set[str]) -> str:
    """Solo letras que MMS conoce: sin tildes, minusculas; japones en romaji."""
    if idioma == "ja":
        texto = romaji(texto)
    t = unicodedata.normalize("NFKD", texto.lower())
    return "".join(c for c in t if c in letras and c != "-")


def alinear(audio: np.ndarray, palabras: list[PalabraTexto], idioma: str,
            desplazamiento: float) -> tuple[list[tuple[float, float] | None], list[float | None]]:
    """Alinea `palabras` en `audio` (mono, 16 kHz). Devuelve por palabra su
    intervalo (en tiempo de cancion: + `desplazamiento`) y su confianza
    (probabilidad media por caracter, 0-1). Palabras sin letras validas: None.
    """
    import torch

    paquete, modelo, dispositivo, letras = _modelo()
    normalizadas = [normalizar(p.texto, idioma, letras) for p in palabras]
    con_texto = [i for i, n in enumerate(normalizadas) if n]
    vacio: tuple[list[tuple[float, float] | None], list[float | None]] = ([None] * len(palabras), [None] * len(palabras))
    if not con_texto or len(audio) < SR // 4:
        return vacio

    onda = torch.from_numpy(np.ascontiguousarray(audio, dtype=np.float32)).unsqueeze(0).to(dispositivo)
    with torch.inference_mode():
        emision, _ = modelo(onda)
    tokens = paquete.get_tokenizer()([normalizadas[i] for i in con_texto])
    # Mas caracteres que frames: no cabe; el texto no es de este tramo.
    if sum(len(t) for t in tokens) >= emision.size(1):
        return vacio
    tramos = paquete.get_aligner()(emision[0], tokens)
    proporcion = onda.size(1) / emision.size(1) / SR

    salida: list[tuple[float, float] | None] = [None] * len(palabras)
    confianzas: list[float | None] = [None] * len(palabras)
    for i, tramo in zip(con_texto, tramos):
        if not tramo:
            continue
        salida[i] = (tramo[0].start * proporcion + desplazamiento, tramo[-1].end * proporcion + desplazamiento)
        confianzas[i] = float(np.mean([float(t.score) for t in tramo]))
    return salida, confianzas
