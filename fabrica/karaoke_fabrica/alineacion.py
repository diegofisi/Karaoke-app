"""Tiempo por palabra: anclas por reconocimiento + alineacion forzada por linea.

Version 1 (medida el 2026-10-15) alineaba la cancion entera de una vez y se
rompia a partir del minuto 1 en estribillos repetidos. Esta version hace tres
pasos:

1. Anclas. Qwen3-ASR transcribe la voz separada con marcas de tiempo, en
   trozos de 15 a 30 s cortados en silencios de la voz, sin contexto.
2. Emparejamiento. La letra conocida se empareja en orden con lo reconocido
   (difflib sobre claves de texto). Cada palabra emparejada es un ancla con
   tiempo. Con las anclas se situa cada linea aunque falten palabras.
3. Alineacion fina. Cada linea se alinea con Qwen3-ForcedAligner dentro de su
   ventana (de la primera a la ultima ancla, con margen). Un error queda
   limitado a una linea.

Confianza por linea: cobertura de anclas, acuerdo entre la alineacion fina y
las anclas, y plausibilidad del ritmo. Por debajo de CONFIANZA_MINIMA la linea
usa las anclas que tenga y reparte el resto, y queda marcada como "respaldo".
"""

from __future__ import annotations

import difflib
from dataclasses import dataclass
from functools import lru_cache

import numpy as np

from .eventos import aviso, etapa, log
from .letra import Letra, LineaFuente
from .modelo import Linea, Palabra
from .texto import PalabraTexto, clave, palabras, romaji, texto_para_alinear

MODELO = "Qwen/Qwen3-ForcedAligner-0.6B"
MODELO_ASR = "Qwen/Qwen3-ASR-1.7B"
IDIOMA_QWEN = {"es": "Spanish", "en": "English", "ja": "Japanese"}
QWEN_A_IDIOMA = {v: k for k, v in IDIOMA_QWEN.items()}
SR = 16000

DURACION_MINIMA = 0.05      # s por palabra
CONFIANZA_MINIMA = 0.5
MARGEN = 0.6                # s de margen de la ventana fina a cada lado
SEGUNDOS_POR_PALABRA = 0.3  # para extrapolar desde el ancla mas cercana
ACUERDO = 0.35              # s: la alineacion fina y el ancla "coinciden" si difieren menos
LOTE = 12                   # lineas por llamada al alineador
TROZO_MINIMO = 15.0         # s, reconocimiento por trozos cortos
TROZO_MAXIMO = 30.0
GRUPO_MAXIMO = 20.0         # s: lineas situadas se agrupan hasta este largo
GRUPO_LIMITE = 40.0         # s: un tramo sin situar mas largo se parte
RONDAS = 2                  # realineaciones de lo no confiable entre vecinas confiables


@dataclass
class Unidad:
    texto: str
    inicio: float
    fin: float


Intervalo = tuple[float, float]


# --- modelos ---------------------------------------------------------------

def _dispositivo():
    """CUDA (Windows/Linux con NVIDIA) > MPS (Mac con Apple Silicon) > CPU.

    En MPS se usa bfloat16, que exige macOS 14 o posterior; sin verificar en
    Mac (esta maquina es Windows).
    """
    import torch

    if torch.cuda.is_available():
        return "cuda:0", torch.bfloat16
    if getattr(torch.backends, "mps", None) is not None and torch.backends.mps.is_available():
        return "mps", torch.bfloat16
    return "cpu", torch.float32


@lru_cache(maxsize=1)
def _alineador():
    from qwen_asr import Qwen3ForcedAligner

    disp, tipo = _dispositivo()
    log.info("Cargando %s en %s", MODELO, disp)
    return Qwen3ForcedAligner.from_pretrained(MODELO, dtype=tipo, device_map=disp)


@lru_cache(maxsize=1)
def _asr():
    from qwen_asr import Qwen3ASRModel

    disp, tipo = _dispositivo()
    log.info("Cargando %s en %s", MODELO_ASR, disp)
    return Qwen3ASRModel.from_pretrained(
        MODELO_ASR, dtype=tipo, device_map=disp, max_inference_batch_size=8, max_new_tokens=1024,
        forced_aligner=MODELO, forced_aligner_kwargs=dict(dtype=tipo, device_map=disp),
    )


def liberar_modelos() -> None:
    import gc

    import torch

    _asr.cache_clear()
    _alineador.cache_clear()
    gc.collect()
    if torch.cuda.is_available():
        torch.cuda.empty_cache()
    elif getattr(torch.backends, "mps", None) is not None and torch.backends.mps.is_available():
        torch.mps.empty_cache()


# --- paso 1: anclas por reconocimiento ----------------------------------

def trozos_por_silencio(audio: np.ndarray, minimo: float = TROZO_MINIMO, maximo: float = TROZO_MAXIMO) -> list[tuple[int, int]]:
    """Corta el audio en trozos de `minimo` a `maximo` segundos por el punto de menos energia.

    Medido el 2026-10-15: el alineador pierde precision al final de trozos
    largos (con 180 s, los ultimos 40 s se desviaban segundos); con ~20-30 s
    no. Devuelve pares [ini, fin) en muestras que cubren todo el audio.
    """
    n = len(audio)
    ventana = int(0.2 * SR)
    if n <= maximo * SR:
        return [(0, n)]
    energia = np.convolve(np.abs(audio), np.ones(ventana, dtype=np.float32), mode="same")
    trozos: list[tuple[int, int]] = []
    ini = 0
    while n - ini > maximo * SR:
        a, b = ini + int(minimo * SR), min(n, ini + int(maximo * SR))
        corte = a + int(np.argmin(energia[a:b]))
        trozos.append((ini, corte))
        ini = corte
    trozos.append((ini, n))
    return trozos


def transcribir(audio: np.ndarray, idioma: str | None) -> tuple[str | None, list[Unidad]]:
    """Reconoce la voz con marcas por unidad, en trozos cortos. Devuelve (idioma detectado, unidades).

    Sin contexto: con la letra como contexto el modelo la copia entera y le
    pone tiempos inventados (medido el 2026-10-15: 629 unidades para 322 palabras).
    """
    trozos = trozos_por_silencio(audio)
    resultados = _asr().transcribe(
        audio=[(audio[a:b], SR) for a, b in trozos],
        language=IDIOMA_QWEN.get(idioma or "", None), return_time_stamps=True,
    )
    unidades: list[Unidad] = []
    idiomas: list[str] = []
    for (a, _), res in zip(trozos, resultados):
        desplazamiento = a / SR
        unidades.extend(Unidad(str(u.text), float(u.start_time) + desplazamiento, float(u.end_time) + desplazamiento)
                        for u in (res.time_stamps or []))
        if res.language:
            idiomas.append(str(res.language).strip().capitalize())
    detectado = QWEN_A_IDIOMA.get(max(set(idiomas), key=idiomas.count)) if idiomas else None
    log.info("ASR: %d unidades en %d trozos, idioma %s", len(unidades), len(trozos), detectado)
    return detectado, unidades


def _tokens(lista: list[PalabraTexto], idioma: str) -> tuple[list[str], list[int]]:
    """Tokens de comparacion y la palabra a la que pertenece cada uno.

    es/en: una palabra = un token. ja: un caracter = un token (el reconocedor y
    la letra no parten las palabras igual).
    """
    tokens: list[str] = []
    duenos: list[int] = []
    for i, p in enumerate(lista):
        if idioma == "ja":
            tokens.extend(p.clave)
            duenos.extend([i] * len(p.clave))
        elif p.clave:
            tokens.append(p.clave)
            duenos.append(i)
    return tokens, duenos


def anclar(planas: list[PalabraTexto], unidades: list[Unidad], idioma: str) -> list[Intervalo | None]:
    """Empareja en orden la letra con lo reconocido. Devuelve un ancla (o None) por palabra."""
    tok_letra, duenos = _tokens(planas, idioma)
    tok_asr: list[str] = []
    tiempos_asr: list[Intervalo] = []
    for u in unidades:
        k = clave(u.texto)
        if not k:
            continue
        if idioma == "ja":
            tok_asr.extend(k)
            tiempos_asr.extend([(u.inicio, u.fin)] * len(k))
        else:
            tok_asr.append(k)
            tiempos_asr.append((u.inicio, u.fin))

    por_palabra: list[list[float]] = [[] for _ in planas]
    emparejador = difflib.SequenceMatcher(a=tok_letra, b=tok_asr, autojunk=False)
    for bloque in emparejador.get_matching_blocks():
        # En es/en un emparejamiento aislado de una palabra corta ("a", "i") es
        # poco fiable: se exige un bloque de 2 o una palabra de 3+ letras.
        if idioma != "ja" and bloque.size == 1 and len(tok_letra[bloque.a]) < 3:
            continue
        for k in range(bloque.size):
            ini, fin = tiempos_asr[bloque.b + k]
            por_palabra[duenos[bloque.a + k]].extend((ini, fin))
    return [(min(t), max(t)) if t else None for t in por_palabra]


# --- paso 2: ventana de cada linea ---------------------------------------

def _ventanas(anclas: list[list[Intervalo | None]], fuentes: list[LineaFuente], desfase: float,
              duracion: float) -> list[tuple[float, float] | None]:
    """Inicio y fin aproximados de las lineas que se pueden situar; None en las demas.

    Una linea se situa por sus anclas si tiene al menos 2 (o el 40 % de sus
    palabras): un ancla suelta puede ser una palabra comun emparejada con otra
    repeticion. Si no, por su marca LRC (+ desfase) si la hay.
    """
    aprox: list[tuple[float, float] | None] = []
    for i, a in enumerate(anclas):
        idx = [k for k, iv in enumerate(a) if iv is not None]
        if idx and (len(idx) >= 2 or len(idx) / len(a) >= 0.4):
            primero, ultimo = idx[0], idx[-1]
            ini = a[primero][0] - primero * SEGUNDOS_POR_PALABRA  # type: ignore[index]
            fin = a[ultimo][1] + (len(a) - 1 - ultimo) * SEGUNDOS_POR_PALABRA  # type: ignore[index]
            aprox.append((max(0.0, ini), min(duracion, fin)))
        elif fuentes[i].inicio is not None:
            ini = float(fuentes[i].inicio) + desfase  # type: ignore[arg-type]
            aprox.append((ini, ini + SEGUNDOS_POR_PALABRA * max(1, len(a)) + 0.5))
        else:
            aprox.append(None)
    return aprox


def _grupos(aprox: list[tuple[float, float] | None], largos: list[int],
            duracion: float) -> list[tuple[float, float, list[int]]]:
    """Agrupa lineas consecutivas en tramos de como mucho GRUPO_MAXIMO segundos.

    Las lineas sin situar van en el grupo de sus vecinas, y el grupo se acota
    con la ultima linea situada antes y la primera despues. Asi el alineador
    busca un texto conocido en un tramo acotado, que es donde acierta (medido el
    2026-10-15: 70 % de inicios a <=100 ms con tramos de ~20 s).
    """
    n = len(aprox)

    def fin_anterior(i: int) -> float:
        return next((aprox[k][1] for k in range(i - 1, -1, -1) if aprox[k] is not None), 0.0)  # type: ignore[index]

    def inicio_siguiente(i: int) -> float:
        return next((aprox[k][0] for k in range(i + 1, n) if aprox[k] is not None), duracion)  # type: ignore[index]

    grupos: list[tuple[float, float, list[int]]] = []
    i = 0
    while i < n:
        ini = aprox[i][0] if aprox[i] is not None else fin_anterior(i)  # type: ignore[index]
        j = i
        while j + 1 < n:
            sig = aprox[j + 1]
            if sig is None or sig[1] - ini <= GRUPO_MAXIMO:
                j += 1
                continue
            break
        fin = aprox[j][1] if aprox[j] is not None else inicio_siguiente(j)  # type: ignore[index]
        indices = list(range(i, j + 1))
        # Tramo sin situar demasiado largo: se parte en proporcion al texto.
        partes = max(1, int(np.ceil((fin - ini) / GRUPO_LIMITE)))
        if partes > 1 and len(indices) > 1:
            pesos = np.cumsum([largos[k] for k in indices], dtype=float)
            pesos /= pesos[-1]
            previo, t_previo = 0, ini
            for p in range(1, partes + 1):
                hasta = len(indices) if p == partes else max(previo + 1, int(np.searchsorted(pesos, p / partes)) + 1)
                hasta = min(hasta, len(indices))
                if hasta <= previo:
                    continue
                t_fin = fin if hasta == len(indices) else ini + (fin - ini) * float(pesos[hasta - 1])
                grupos.append((t_previo, t_fin, indices[previo:hasta]))
                previo, t_previo = hasta, t_fin
        else:
            grupos.append((ini, fin, indices))
        i = j + 1
    return grupos


# --- paso 3: alineacion fina por linea ------------------------------------

def _alinear_lote(trozos: list[np.ndarray], textos: list[str], idioma: str) -> list[list[Unidad]]:
    res = _alineador().align(audio=[(t, SR) for t in trozos], text=textos, language=IDIOMA_QWEN[idioma])
    return [[Unidad(str(u.text), float(u.start_time), float(u.end_time)) for u in r] for r in res]


def casar(lista: list[PalabraTexto], unidades: list[Unidad]) -> list[Intervalo | None]:
    """Asigna a cada palabra el intervalo de las unidades que caen en su texto.

    Trabaja sobre la cadena de claves concatenadas, asi sirve igual si el
    alineador devuelve palabras o caracteres.
    """
    claves = [p.clave for p in lista]
    cadena = "".join(claves)
    limites = np.cumsum([0] + [len(k) for k in claves])
    tramos: list[list[float]] = [[] for _ in lista]
    cursor = 0
    for u in unidades:
        k = clave(u.texto)
        if not k:
            continue
        pos = cadena.find(k, cursor)
        if pos < 0 or pos - cursor > 40:
            continue
        fin_pos = pos + len(k)
        cursor = fin_pos
        primera = int(np.searchsorted(limites, pos, side="right") - 1)
        ultima = int(np.searchsorted(limites, fin_pos - 1, side="right") - 1)
        for i in range(primera, ultima + 1):
            tramos[i].extend((u.inicio, u.fin))
    return [(min(t), max(t)) if t else None for t in tramos]


def _ordenar(intervalos: list[Intervalo | None], desde: float = -1.0) -> list[Intervalo | None]:
    """Sin solapes, duracion minima, orden creciente."""
    salida: list[Intervalo | None] = []
    ultimo_fin = desde
    for iv in intervalos:
        if iv is None:
            salida.append(None)
            continue
        ini = max(iv[0], ultimo_fin)
        fin = max(iv[1], ini + DURACION_MINIMA)
        salida.append((ini, fin))
        ultimo_fin = fin
    return salida


def _repartir(lista: list[PalabraTexto], ini: float, fin: float) -> list[Intervalo]:
    """Reparte [ini, fin) en proporcion a la longitud de cada clave."""
    pesos = np.array([max(1, len(p.clave)) for p in lista], dtype=float)
    bordes = ini + (fin - ini) * np.concatenate([[0.0], np.cumsum(pesos) / pesos.sum()])
    return [(float(bordes[i]), float(bordes[i + 1])) for i in range(len(lista))]


def _rellenar_huecos(lista: list[PalabraTexto], ivs: list[Intervalo | None], ini: float, fin: float) -> list[Intervalo]:
    """Palabras sin tiempo: se reparten entre la vecina anterior y la siguiente."""
    salida = list(ivs)
    n = len(salida)
    k = 0
    while k < n:
        if salida[k] is not None:
            k += 1
            continue
        j = k
        while j < n and salida[j] is None:
            j += 1
        a = salida[k - 1][1] if k > 0 else ini  # type: ignore[index]
        b = salida[j][0] if j < n else max(fin, a)  # type: ignore[index]
        b = max(b, a + DURACION_MINIMA * (j - k))
        for m, iv in enumerate(_repartir(lista[k:j], a, b)):
            salida[k + m] = iv
        k = j
    return salida  # type: ignore[return-value]


def _confianza(fina: list[Intervalo | None], anclas: list[Intervalo | None]) -> float:
    n = len(fina)
    if n == 0:
        return 0.0
    validos = [iv for iv in fina if iv is not None]
    if not validos:
        return 0.0
    cubiertas = len(validos) / n
    duracion = validos[-1][1] - validos[0][0]
    ritmo = n / max(duracion, 1e-3)
    plausible = 1.0 if ritmo <= 8 and duracion <= 20 else 0.3
    colapsadas = sum(1 for a, b in validos if b - a <= DURACION_MINIMA + 1e-6) / len(validos)
    pares = [(f, a) for f, a in zip(fina, anclas) if f is not None and a is not None]
    if pares:
        acuerdo = sum(1 for f, a in pares if abs(f[0] - a[0]) <= ACUERDO) / len(pares)
        factor_anclas = 0.4 + 0.6 * acuerdo
    else:
        factor_anclas = 0.6  # sin anclas no hay con que comprobar
    return float(cubiertas * plausible * (1.0 - 0.7 * colapsadas) * factor_anclas)


# --- conjunto ----------------------------------------------------------------

def letra_desde_unidades(unidades: list[Unidad], idioma: str) -> Letra:
    """Sin letra: las lineas salen de las pausas del reconocimiento (>= 0,8 s) o cada 10 palabras."""
    lineas: list[LineaFuente] = []
    actual: list[str] = []
    ini: float | None = None
    previo_fin = -1.0
    sep = "" if idioma == "ja" else " "
    for u in unidades:
        if actual and (u.inicio - previo_fin >= 0.8 or len(actual) >= 10):
            lineas.append(LineaFuente(sep.join(actual), ini))
            actual, ini = [], None
        if ini is None:
            ini = u.inicio
        actual.append(u.texto)
        previo_fin = u.fin
    if actual:
        lineas.append(LineaFuente(sep.join(actual), ini))
    return Letra(lineas, "transcrita")


def actividad_voz(voz: np.ndarray) -> np.ndarray:
    """Frames de 20 ms con voz en la pista separada (energia sobre un umbral relativo)."""
    paso = int(0.02 * SR)
    n = len(voz) // paso
    if n == 0:
        return np.zeros(0, dtype=bool)
    rms = np.sqrt(np.mean(voz[: n * paso].reshape(n, paso) ** 2, axis=1))
    referencia = float(np.percentile(rms, 95)) or 1e-6
    return rms > 0.08 * referencia


def _repartir_en_voz(lista: list[PalabraTexto], ini: float, fin: float, activa: np.ndarray | None) -> list[Intervalo]:
    """Respaldo: reparte las palabras sobre los tramos con voz de [ini, fin).

    Sin tramos con voz (o sin pista de voz), reparte la ventana entera.
    """
    if activa is None or fin - ini < 0.1:
        return _repartir(lista, ini, max(fin, ini + DURACION_MINIMA * len(lista)))
    a, b = int(ini / 0.02), int(np.ceil(fin / 0.02))
    frames = np.flatnonzero(activa[a:b]) + a
    if len(frames) < len(lista):
        return _repartir(lista, ini, fin)
    # Tiempo con voz "comprimido": se reparte ahi y se vuelve a tiempo real.
    pesos = np.array([max(1, len(p.clave)) for p in lista], dtype=float)
    cortes = np.concatenate([[0.0], np.cumsum(pesos) / pesos.sum()]) * (len(frames) - 1)
    salida: list[Intervalo] = []
    for k in range(len(lista)):
        f0 = frames[int(round(cortes[k]))]
        f1 = frames[int(round(cortes[k + 1]))]
        salida.append((f0 * 0.02, max(f1 * 0.02, f0 * 0.02 + DURACION_MINIMA)))
    return salida


def _huecos(confiable: list[bool], finas: list[list[Intervalo | None]], largos: list[int],
            duracion: float) -> list[tuple[float, float, list[int]]]:
    """Tramos de lineas no confiables, acotados por las lineas confiables vecinas."""
    n = len(confiable)

    def fin_linea(k: int) -> float:
        return max(iv[1] for iv in finas[k] if iv is not None)

    def ini_linea(k: int) -> float:
        return min(iv[0] for iv in finas[k] if iv is not None)

    tramos: list[tuple[float, float, list[int]]] = []
    i = 0
    while i < n:
        if confiable[i]:
            i += 1
            continue
        j = i
        while j + 1 < n and not confiable[j + 1]:
            j += 1
        t0 = fin_linea(i - 1) if i > 0 else 0.0
        t1 = ini_linea(j + 1) if j + 1 < n else duracion
        indices = list(range(i, j + 1))
        partes = max(1, int(np.ceil((t1 - t0) / GRUPO_LIMITE)))
        if partes > 1 and len(indices) > 1:
            pesos = np.cumsum([largos[k] for k in indices], dtype=float)
            pesos /= pesos[-1]
            previo, t_previo = 0, t0
            for p in range(1, partes + 1):
                hasta = len(indices) if p == partes else min(len(indices), max(previo + 1, int(np.searchsorted(pesos, p / partes)) + 1))
                if hasta <= previo:
                    continue
                t_fin = t1 if hasta == len(indices) else t0 + (t1 - t0) * float(pesos[hasta - 1])
                tramos.append((t_previo, t_fin, indices[previo:hasta]))
                previo, t_previo = hasta, t_fin
        else:
            tramos.append((t0, t1, indices))
        i = j + 1
    return tramos


def _alinear_tramos(audio: np.ndarray, tramos: list[tuple[float, float, list[int]]],
                    por_linea: list[list[PalabraTexto]], idioma: str, duracion: float,
                    avance: tuple[float, float]) -> dict[int, list[Intervalo | None]]:
    """Alinea el texto de cada tramo dentro de su ventana (con margen). Devuelve intervalos por linea."""
    salida: dict[int, list[Intervalo | None]] = {}
    for b in range(0, len(tramos), LOTE):
        lote = tramos[b:b + LOTE]
        etapa("alineacion", avance[0] + (avance[1] - avance[0]) * b / max(1, len(tramos)), "Alineando la letra por tramos")
        ventanas = []
        for ini, fin, indices in lote:
            t0, t1 = max(0.0, ini - MARGEN), min(duracion, fin + MARGEN)
            minimo = 0.5 + 0.15 * sum(len(por_linea[k]) for k in indices)
            if t1 - t0 < minimo:
                t1 = min(duracion, t0 + minimo)
            ventanas.append((t0, t1))
        planas_t = [[p for k in indices for p in por_linea[k]] for _, _, indices in lote]
        try:
            resultados = _alinear_lote([audio[int(t0 * SR):int(t1 * SR)] for t0, t1 in ventanas],
                                       [texto_para_alinear(pt, idioma) for pt in planas_t], idioma)
        except Exception as e:  # un lote que falla no tumba la cancion
            aviso(f"El alineador fallo en un tramo: {e}")
            continue
        for (t0, _), (_, _, indices), pt, unidades in zip(ventanas, lote, planas_t, resultados):
            casados = [None if iv is None else (iv[0] + t0, iv[1] + t0) for iv in casar(pt, unidades)]
            k = 0
            for linea in indices:
                salida[linea] = casados[k:k + len(por_linea[linea])]
                k += len(por_linea[linea])
    return salida


def alinear(letra: Letra, audio: np.ndarray, idioma: str, duracion: float,
            unidades_asr: list[Unidad] | None = None, voz: np.ndarray | None = None) -> list[Linea]:
    """Devuelve las lineas con el tiempo de cada palabra.

    `audio`: lo que se alinea (voz separada o mezcla), mono 16 kHz. `voz`: la
    voz separada a 16 kHz para saber donde hay canto (respaldo); si falta, se
    usa `audio`.
    """
    fuentes = [l for l in letra.lineas if clave(l.texto)]
    por_linea = [palabras(l.texto, idioma) for l in fuentes]
    planas = [p for lista in por_linea for p in lista]
    largos = [len(clave(f.texto)) for f in fuentes]
    n = len(fuentes)

    if unidades_asr is None:
        etapa("alineacion", 0.05, "Reconociendo la voz para situar cada linea")
        _, unidades_asr = transcribir(audio, idioma)

    etapa("alineacion", 0.3, "Emparejando la letra con lo reconocido")
    anclas_planas = anclar(planas, unidades_asr, idioma)
    anclas: list[list[Intervalo | None]] = []
    k = 0
    for lista in por_linea:
        anclas.append(anclas_planas[k:k + len(lista)])
        k += len(lista)
    con_ancla = sum(1 for a in anclas_planas if a is not None)
    log.info("Anclas: %d de %d palabras", con_ancla, len(planas))
    if planas and con_ancla / len(planas) < 0.3:
        aviso(f"Solo {con_ancla} de {len(planas)} palabras de la letra se reconocieron en el audio: "
              "puede que la letra no sea de esta version de la cancion.")

    desfase = 0.0
    if letra.sincronizada:
        difs = [a[0][0] - float(f.inicio) for f, a in zip(fuentes, anclas)  # type: ignore[arg-type]
                if a and a[0] is not None]
        if difs:
            desfase = float(np.median(difs))
            log.info("Desfase LRC -> audio: %.2f s", desfase)

    # Ronda 0: grupos de lineas situadas por sus anclas.
    aprox = _ventanas(anclas, fuentes, desfase, duracion)
    finas: list[list[Intervalo | None]] = [[None] * len(lista) for lista in por_linea]
    confianza = [0.0] * n
    for linea, ivs in _alinear_tramos(audio, _grupos(aprox, largos, duracion), por_linea, idioma, duracion,
                                      (0.35, 0.65)).items():
        finas[linea] = ivs
        confianza[linea] = _confianza(ivs, anclas[linea])

    # Rondas siguientes: lo no confiable se vuelve a alinear entre vecinas confiables.
    for ronda in range(RONDAS):
        confiable = [c >= CONFIANZA_MINIMA and all(iv is not None for iv in f) for c, f in zip(confianza, finas)]
        if all(confiable) or not any(confiable):
            break
        tramos = _huecos(confiable, finas, largos, duracion)
        log.info("Ronda %d: %d lineas por realinear en %d tramos", ronda + 1, n - sum(confiable), len(tramos))
        nuevas = _alinear_tramos(audio, tramos, por_linea, idioma, duracion,
                                 (0.65 + 0.1 * ronda, 0.75 + 0.1 * ronda))
        mejoro = False
        for linea, ivs in nuevas.items():
            c = _confianza(ivs, anclas[linea])
            if c > confianza[linea]:
                finas[linea], confianza[linea], mejoro = ivs, c, True
        if not mejoro:
            break

    # Respaldo: las lineas que siguen sin confianza se reparten sobre la voz entre sus vecinas.
    activa = actividad_voz(voz if voz is not None else audio)
    confiable = [c >= CONFIANZA_MINIMA and all(iv is not None for iv in f) for c, f in zip(confianza, finas)]
    respaldo: dict[int, list[Intervalo]] = {}
    if any(confiable):
        for t0, t1, indices in _huecos(confiable, finas, largos, duracion):
            todas = [p for k in indices for p in por_linea[k]]
            repartidas = _repartir_en_voz(todas, t0, t1, activa)
            k = 0
            for linea in indices:
                respaldo[linea] = repartidas[k:k + len(por_linea[linea])]
                k += len(por_linea[linea])

    lineas: list[Linea] = []
    ultimo_fin = -1.0
    for i, (fuente, lista) in enumerate(zip(fuentes, por_linea)):
        if confiable[i]:
            ivs, metodo = finas[i], "alineada"
        elif i in respaldo:
            ivs, metodo = respaldo[i], "respaldo"  # type: ignore[assignment]
        else:  # nada confiable en toda la cancion: lo que haya, completado
            ivs, metodo = (finas[i] if any(finas[i]) else anclas[i]), "respaldo"
        ini_v = max(0.0, ultimo_fin)
        completos = _rellenar_huecos(lista, _ordenar(ivs, ultimo_fin), ini_v, ini_v + 0.6 * len(lista) + 1.0)
        completos = _ordenar(completos, ultimo_fin)  # type: ignore[assignment]
        ultimo_fin = completos[-1][1]  # type: ignore[index]
        palabras_linea = [Palabra(texto=p.texto, inicio=iv[0], fin=iv[1], lectura=p.lectura)  # type: ignore[index]
                          for p, iv in zip(lista, completos)]
        lineas.append(Linea(
            inicio=palabras_linea[0].inicio, fin=palabras_linea[-1].fin, texto=fuente.texto,
            palabras=palabras_linea, confianza=round(confianza[i], 3), metodo=metodo,  # type: ignore[arg-type]
            romaji=romaji(fuente.texto) if idioma == "ja" else None,
        ))
    en_respaldo = sum(1 for l in lineas if l.metodo == "respaldo")
    if en_respaldo:
        aviso(f"{en_respaldo} de {len(lineas)} lineas tienen tiempos aproximados.")
    etapa("alineacion", 1.0, f"Letra alineada: {len(lineas) - en_respaldo}/{len(lineas)} lineas con tiempo exacto")
    return lineas
