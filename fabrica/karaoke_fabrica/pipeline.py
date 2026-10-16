"""Orquesta las etapas y escribe cancion.json.

Cada etapa deja su resultado en la carpeta de la cancion y se salta si ya
existe (descarga, separacion). La alineacion, el tono y las notas son rapidos
con GPU y se rehacen siempre, para que un cambio en ellos se vea al reprocesar.
"""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

import numpy as np

from . import __version__, alineacion, descarga, letra as mod_letra, notas, separacion, tono
from .eventos import ErrorFabrica, aviso, etapa, log
from .modelo import Cancion, CurvaTono, Origen, guardar
from .texto import detectar_idioma


@dataclass
class Peticion:
    salida: Path
    url: str | None = None
    audio: Path | None = None
    titulo: str | None = None
    artista: str | None = None
    idioma: str | None = None
    letra: Path | None = None
    separador: str = separacion.MODELO_POR_DEFECTO
    alinear_sobre: str = "voz"  # "voz" | "mezcla"


def id_cancion(p: Peticion) -> str:
    if p.url:
        return f"yt-{descarga.id_youtube(p.url)}"
    assert p.audio is not None
    h = hashlib.sha1(p.audio.read_bytes()[: 4 * 1024 * 1024]).hexdigest()[:12]
    return f"local-{h}"


def _audio_16k(ruta: Path) -> np.ndarray:
    import soundfile as sf
    from scipy.signal import resample_poly

    audio, sr = sf.read(str(ruta), dtype="float32", always_2d=True)
    mono = audio.mean(axis=1)
    if sr == alineacion.SR:
        return mono
    from math import gcd

    g = gcd(sr, alineacion.SR)
    return resample_poly(mono, alineacion.SR // g, sr // g).astype(np.float32)


def procesar(p: Peticion) -> tuple[str, Path]:
    p.salida.mkdir(parents=True, exist_ok=True)
    cid = id_cancion(p)

    if p.url:
        meta = descarga.descargar(p.url, p.salida)
    else:
        assert p.audio is not None
        meta = descarga.importar_local(p.audio, p.salida, p.titulo or p.audio.stem, p.artista or "")
    if p.titulo:
        meta.titulo = p.titulo
    else:
        # "Light It Up [HD]", "(Official Video)": se quitan tambien para mostrarlo.
        meta.titulo = mod_letra.limpiar_titulo(meta.titulo) or meta.titulo
    if p.artista:
        meta.artista = p.artista
    original = p.salida / "original.wav"

    voz, _ = separacion.separar(original, p.salida, p.separador)

    letra = mod_letra.obtener(meta.artista, meta.titulo, meta.duracion, p.letra, p.salida)
    audio_voz = _audio_16k(voz)
    audio = audio_voz if p.alinear_sobre == "voz" else _audio_16k(original)
    duracion = meta.duracion or len(audio) / alineacion.SR

    unidades = None
    if letra is None:
        etapa("letra", 0.3, "Sin letra disponible: transcribiendo la voz (Qwen3-ASR)")
        detectado, unidades = alineacion.transcribir(audio, p.idioma)
        idioma_t = p.idioma or detectado
        if not unidades or idioma_t not in alineacion.IDIOMA_QWEN:
            raise ErrorFabrica("letra", "No se encontro letra y no se pudo transcribir la voz.")
        letra = alineacion.letra_desde_unidades(unidades, idioma_t)
        aviso("La letra se transcribio automaticamente: puede tener errores. Puedes darla tu como archivo.")
        etapa("letra", 1.0, f"Letra transcrita: {len(letra.lineas)} lineas")
    idioma = p.idioma or detectar_idioma([l.texto for l in letra.lineas])
    if idioma not in alineacion.IDIOMA_QWEN:
        raise ErrorFabrica("letra", f"Idioma no soportado: {idioma}. Soportados: es, en, ja.")
    log.info("Idioma: %s, lineas: %d, origen: %s", idioma, len(letra.lineas), letra.origen)

    lineas = alineacion.alinear(letra, audio, idioma, duracion, unidades, voz=audio_voz)
    if not lineas:
        raise ErrorFabrica("alineacion", "La letra quedo vacia tras alinearla.")

    curva = tono.extraer(voz)

    etapa("notas", 0.0, "Calculando las notas de cada palabra")
    notas.asignar(lineas, curva)
    n_notas = sum(len(w.notas) for l in lineas for w in l.palabras)
    etapa("notas", 1.0, f"{n_notas} notas")

    etapa("guardado", 0.0, "Guardando cancion.json")
    cancion = Cancion(
        id=cid, titulo=meta.titulo, artista=meta.artista, duracion=meta.duracion, idioma=idioma,  # type: ignore[arg-type]
        origen=Origen(
            youtube_id=meta.youtube_id, letra=letra.origen, separador=p.separador,  # type: ignore[arg-type]
            alineador=alineacion.MODELO, tono="swift-f0", fabrica_version=__version__,
            creada=datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        ),
        lineas=lineas,
        tono=CurvaTono(inicio=curva.inicio, paso=curva.paso, midi=tono.a_lista(curva)),
    )
    ruta = p.salida / "cancion.json"
    guardar(cancion, ruta)
    etapa("guardado", 1.0, "Cancion lista")
    return cid, ruta


def diagnostico() -> dict:
    import platform

    info: dict = {"python": platform.python_version(), "sistema": platform.system()}
    try:
        import torch

        info["torch"] = torch.__version__
        info["cuda"] = bool(torch.cuda.is_available())
        mps = getattr(torch.backends, "mps", None) is not None and torch.backends.mps.is_available()
        if info["cuda"]:
            info["aceleracion"], info["gpu"] = "cuda", torch.cuda.get_device_name(0)
        elif mps:
            info["aceleracion"], info["gpu"] = "mps", f"Apple {platform.machine()} (MPS)"
        else:
            info["aceleracion"], info["gpu"] = "cpu", None
    except Exception as e:  # torch roto: se informa, no se lanza
        info["torch"] = None
        info["cuda"] = False
        info["aceleracion"] = "cpu"
        info["gpu"] = None
        info["error_torch"] = str(e)
    info["modelos"] = str(separacion.carpeta_modelos())
    return json.loads(json.dumps(info))
