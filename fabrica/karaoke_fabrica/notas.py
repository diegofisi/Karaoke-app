"""Notas por palabra a partir de la curva de tono.

El alineador da donde empieza y acaba cada palabra. Aqui el tono decide cuantas
notas tiene: una palabra cantada sobre dos alturas (melisma) da dos notas. El
vibrato (+-0,5 a 1 semitono, 5-7 Hz) no parte una nota: un cambio cuenta solo
si se aleja mas de UMBRAL_CAMBIO durante al menos FRAMES_CAMBIO frames.
"""

from __future__ import annotations

import numpy as np

from .modelo import Linea, Nota
from .tono import Curva, PASO, tramo

UMBRAL_CAMBIO = 1.0      # semitonos respecto a la mediana de la nota en curso
FRAMES_CAMBIO = 5        # 80 ms sostenidos
NOTA_MINIMA = 5          # frames: notas de menos de 80 ms se funden con la vecina
VOZ_MINIMA = 0.35        # fraccion de frames con voz para que la palabra tenga tono
DORADA_DURACION = 0.5    # s
DORADA_ESTABILIDAD = 0.6 # desviacion tipica maxima (semitonos)
DORADA_FRACCION = 0.06   # como mucho este porcentaje de las notas


def _grupos(seg: np.ndarray) -> list[tuple[int, int]]:
    """Divide los frames con voz en grupos de altura estable. Devuelve [ini, fin)."""
    idx = np.flatnonzero(~np.isnan(seg))
    if len(idx) == 0:
        return []
    grupos: list[tuple[int, int]] = []
    ini = idx[0]
    actuales: list[float] = [seg[ini]]
    fuera = 0
    for k in range(1, len(idx)):
        i = idx[k]
        v = seg[i]
        if abs(v - float(np.median(actuales))) > UMBRAL_CAMBIO:
            fuera += 1
            if fuera >= FRAMES_CAMBIO:
                corte = idx[k - FRAMES_CAMBIO + 1]
                grupos.append((ini, corte))
                ini = corte
                actuales = list(seg[corte:i + 1][~np.isnan(seg[corte:i + 1])])
                fuera = 0
                continue
        else:
            fuera = 0
        actuales.append(v)
    grupos.append((ini, idx[-1] + 1))

    # Fundir grupos cortos con el vecino de altura mas parecida.
    cambiado = True
    while cambiado and len(grupos) > 1:
        cambiado = False
        for g, (a, b) in enumerate(grupos):
            if b - a >= NOTA_MINIMA:
                continue
            med = np.nanmedian(seg[a:b])
            vecinos = [v for v in (g - 1, g + 1) if 0 <= v < len(grupos)]
            mejor = min(vecinos, key=lambda v: abs(np.nanmedian(seg[grupos[v][0]:grupos[v][1]]) - med))
            lo, hi = min(g, mejor), max(g, mejor)
            grupos[lo] = (grupos[lo][0], grupos[hi][1])
            del grupos[hi]
            cambiado = True
            break
    return grupos


def notas_de_palabra(curva: Curva, inicio: float, fin: float) -> list[Nota]:
    seg = tramo(curva, inicio, fin)
    if len(seg) == 0:
        return []
    sonoro = ~np.isnan(seg)
    if sonoro.mean() < VOZ_MINIMA or sonoro.sum() < NOTA_MINIMA:
        return []
    grupos = _grupos(seg)
    t0 = curva.inicio + round((inicio - curva.inicio) / curva.paso) * curva.paso
    notas: list[Nota] = []
    for n, (a, b) in enumerate(grupos):
        n_ini = inicio if n == 0 else t0 + a * PASO
        n_fin = fin if n == len(grupos) - 1 else t0 + grupos[n + 1][0] * PASO
        if n_fin - n_ini <= 0:
            continue
        notas.append(Nota(inicio=n_ini, fin=n_fin, midi=float(np.nanmedian(seg[a:b]))))
    return notas


def asignar(lineas: list[Linea], curva: Curva) -> None:
    """Rellena palabras[].notas y marca las doradas. Modifica las lineas."""
    candidatas: list[tuple[float, Nota]] = []
    total = 0
    for linea in lineas:
        for p in linea.palabras:
            p.notas = notas_de_palabra(curva, p.inicio, p.fin)
            total += len(p.notas)
            for nota in p.notas:
                dur = nota.fin - nota.inicio
                if dur < DORADA_DURACION:
                    continue
                valores = tramo(curva, nota.inicio, nota.fin)
                valores = valores[~np.isnan(valores)]
                if len(valores) >= NOTA_MINIMA and float(np.std(valores)) <= DORADA_ESTABILIDAD:
                    candidatas.append((dur, nota))
    cupo = int(total * DORADA_FRACCION)
    for _, nota in sorted(candidatas, key=lambda c: -c[0])[:cupo]:
        nota.tipo = "dorada"
