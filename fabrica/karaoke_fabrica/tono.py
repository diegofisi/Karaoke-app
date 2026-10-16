"""Tono de referencia de la voz separada con SwiftF0, y su limpieza.

SwiftF0 da un frame cada 16 ms con tono y confianza. Sobre una voz separada
quedan tres tipos de basura que se limpian aqui:
1. islas sonoras muy cortas (restos de la separacion, consonantes);
2. saltos de octava (el detector elige el armonico o el subarmonico);
3. temblor frame a frame (mediana de 3 frames dentro de cada tramo sonoro).
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import numpy as np

from .eventos import etapa

PASO = 0.016            # periodo de frame de SwiftF0
UMBRAL_CONFIANZA = 0.5  # el umbral por defecto de SwiftF0
FMIN, FMAX = 60.0, 1100.0
ISLA_MINIMA = 3         # frames: menos de 48 ms con voz se descarta
VENTANA_OCTAVA = 47     # frames a cada lado (~0,75 s) para la mediana de referencia


@dataclass
class Curva:
    inicio: float
    paso: float
    midi: np.ndarray      # NaN = sin voz
    confianza: np.ndarray
    volumen_db: np.ndarray


def hz_a_midi(hz: np.ndarray) -> np.ndarray:
    with np.errstate(divide="ignore", invalid="ignore"):
        return 69.0 + 12.0 * np.log2(hz / 440.0)


def _tramos(sonoro: np.ndarray) -> list[tuple[int, int]]:
    """Pares [ini, fin) de frames consecutivos con voz."""
    tramos: list[tuple[int, int]] = []
    i, n = 0, len(sonoro)
    while i < n:
        if sonoro[i]:
            j = i
            while j < n and sonoro[j]:
                j += 1
            tramos.append((i, j))
            i = j
        else:
            i += 1
    return tramos


def limpiar(midi: np.ndarray) -> np.ndarray:
    m = midi.copy()
    sonoro = ~np.isnan(m)

    for a, b in _tramos(sonoro):
        if b - a < ISLA_MINIMA:
            m[a:b] = np.nan
    sonoro = ~np.isnan(m)

    # Octava: cada frame se compara con la mediana de la voz cercana. Si esta a
    # mas de 9 semitonos, se traslada las octavas que haga falta. Dos pasadas:
    # la segunda usa la referencia ya corregida.
    idx = np.flatnonzero(sonoro)
    for _ in range(2):
        if len(idx) == 0:
            break
        valores = m[idx]
        corregidos = valores.copy()
        for k, i in enumerate(idx):
            lo = np.searchsorted(idx, i - VENTANA_OCTAVA)
            hi = np.searchsorted(idx, i + VENTANA_OCTAVA, side="right")
            ref = float(np.median(valores[lo:hi]))
            dif = ref - valores[k]
            if abs(dif) > 9:
                corregidos[k] = valores[k] + 12 * round(dif / 12)
        m[idx] = corregidos

    # Mediana de 3 dentro de cada tramo (no mezcla voz con silencio).
    salida = m.copy()
    for a, b in _tramos(sonoro):
        if b - a >= 3:
            seg = m[a:b]
            ventana = np.stack([np.r_[seg[0], seg[:-1]], seg, np.r_[seg[1:], seg[-1]]])
            salida[a:b] = np.median(ventana, axis=0)
    return salida


def leer_mono(ruta: Path) -> tuple[np.ndarray, int]:
    import soundfile as sf

    audio, sr = sf.read(str(ruta), dtype="float32", always_2d=True)
    return audio.mean(axis=1), sr


def extraer(voz: Path) -> Curva:
    from swift_f0 import SwiftF0

    etapa("tono", 0.0, "Detectando el tono de la voz (SwiftF0)")
    audio, sr = leer_mono(voz)
    resultado = SwiftF0().detect(audio, sr, fmin=FMIN, fmax=FMAX)
    hz = np.asarray(resultado.pitch_hz, dtype=np.float64)
    conf = np.asarray(resultado.confidence, dtype=np.float64)
    vol = np.asarray(resultado.loudness_db, dtype=np.float64)
    tiempos = np.asarray(resultado.timestamps, dtype=np.float64)

    midi = hz_a_midi(hz)
    midi[(conf < UMBRAL_CONFIANZA) | ~np.isfinite(midi)] = np.nan
    etapa("tono", 0.7, "Limpiando octavas y restos de la separacion")
    midi = limpiar(midi)
    inicio = float(tiempos[0]) if len(tiempos) else 0.0
    etapa("tono", 1.0, f"Tono listo: {int((~np.isnan(midi)).sum() * PASO)} s con voz")
    return Curva(inicio=inicio, paso=PASO, midi=midi, confianza=conf, volumen_db=vol)


def a_lista(curva: Curva) -> list[float | None]:
    return [None if np.isnan(v) else round(float(v), 2) for v in curva.midi]


def tramo(curva: Curva, t0: float, t1: float) -> np.ndarray:
    """Frames de la curva entre t0 y t1 (segundos)."""
    a = max(0, int(round((t0 - curva.inicio) / curva.paso)))
    b = max(a, int(round((t1 - curva.inicio) / curva.paso)))
    return curva.midi[a:b]
