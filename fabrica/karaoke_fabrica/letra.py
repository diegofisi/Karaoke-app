"""Fuentes de la letra: archivo del usuario, LRCLIB, o transcripcion.

Devuelve lineas con texto y, si la fuente lo trae, el inicio aproximado de cada
linea (LRC). Los tiempos de LRC solo sirven para trocear el audio y para
comprobar la alineacion: el tiempo por palabra siempre sale del alineador.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

from .eventos import aviso, etapa, log

LRCLIB = "https://lrclib.net/api"
AGENTE = "karaoke-fabrica/0.1 (https://github.com/diegofisi/karaoke-gmi2)"

_MARCA_LRC = re.compile(r"\[(\d+):(\d+(?:[.:]\d+)?)\]")
_RUIDO_TITULO = re.compile(
    r"\s*[\(\[【](?:[^)\]】]*?(?:official|oficial|video|lyric|letra|audio|mv|m/v|hd|4k|remaster|live|visualizer|公式)[^)\]】]*)[\)\]】]",
    re.IGNORECASE,
)


@dataclass
class LineaFuente:
    texto: str
    inicio: float | None  # segundos, si la fuente es LRC


@dataclass
class Letra:
    lineas: list[LineaFuente]
    origen: str  # "lrclib" | "usuario" | "transcrita"

    @property
    def sincronizada(self) -> bool:
        """Trae tiempos LRC. Las repeticiones expandidas ("x3") no los tienen."""
        return any(l.inicio is not None for l in self.lineas)


# Cabecera de seccion que no se canta: "[Chorus]", "(Verso 2)", "Coro:", "Chorus 2".
# Solo entre corchetes/parentesis, o la palabra sola (con numero o ":"
# opcionales): "Bridge over troubled water" o "Puente roto" SI se cantan.
_PALABRAS_SECCION = (
    r"(?:pre-?\s*)?(?:chorus|verse|bridge|intro|outro|hook|refrain|interlude|instrumental|"
    r"coro|estribillo|verso|estrofa|puente|repeat|repite|repetir|bis)"
)
_SECCION = re.compile(
    rf"^\s*(?:[\[\(\{{]\s*{_PALABRAS_SECCION}\b[^\]\)\}}]*[\]\)\}}]|{_PALABRAS_SECCION}\s*\d*)\s*:?\s*$",
    re.IGNORECASE,
)
# Repeticion: "x3", "x 3", "(x3)", "[3x]", "×2". Sola en la linea o al final.
_REPETICION = r"[\(\[]?\s*(?:[x×]\s*(\d{1,2})|(\d{1,2})\s*[x×])\s*[\)\]]?"
_REPETICION_SOLA = re.compile(rf"^\s*{_REPETICION}\s*$", re.IGNORECASE)
_REPETICION_FINAL = re.compile(rf"^(.*\S)\s+{_REPETICION}\s*$", re.IGNORECASE)
MAX_REPETICIONES = 8


def limpiar_lineas(lineas: list[LineaFuente]) -> list[LineaFuente]:
    """Quita lo que no se canta y expande las repeticiones.

    Visto el 2026-10-15 en una letra real de LRCLIB: una linea "x 2" y otra
    terminada en "x 3". El alineador buscaba "x" y "3" en el audio, fallaba, y
    como la frase se canta 3 veces pero estaba escrita una, desplazaba las
    lineas siguientes. Ahora: "frase x3" -> la frase 3 veces (solo la primera
    conserva su tiempo LRC); una "x 2" suelta o una cabecera de seccion se
    quitan.
    """
    salida: list[LineaFuente] = []
    for l in lineas:
        texto = l.texto.strip()
        if not texto or _SECCION.match(texto) or _REPETICION_SOLA.match(texto):
            continue
        m = _REPETICION_FINAL.match(texto)
        if m:
            veces = int(m.group(2) or m.group(3))
            frase = m.group(1).strip()
            if 2 <= veces <= MAX_REPETICIONES:
                salida.append(LineaFuente(frase, l.inicio))
                salida.extend(LineaFuente(frase, None) for _ in range(veces - 1))
                continue
            texto = frase
        salida.append(LineaFuente(texto, l.inicio))
    return salida


def limpiar_titulo(titulo: str) -> str:
    t = _RUIDO_TITULO.sub("", titulo)
    t = re.sub(r"\s*(?:ft\.|feat\.|featuring)\s.*$", "", t, flags=re.IGNORECASE)
    return t.strip(" -|")


def leer_lrc(texto: str) -> list[LineaFuente]:
    """Lee LRC. Una linea con varias marcas se repite en cada una. Ignora lineas vacias."""
    lineas: list[LineaFuente] = []
    for cruda in texto.splitlines():
        marcas = list(_MARCA_LRC.finditer(cruda))
        if not marcas:
            continue
        contenido = _MARCA_LRC.sub("", cruda).strip()
        # Enhanced LRC trae <mm:ss.xx> por palabra: se quitan; el alineador manda.
        contenido = re.sub(r"<\d+:\d+(?:\.\d+)?>", "", contenido).strip()
        if not contenido:
            continue
        for m in marcas:
            seg = m.group(2).replace(":", ".")
            lineas.append(LineaFuente(contenido, int(m.group(1)) * 60 + float(seg)))
    lineas.sort(key=lambda l: l.inicio or 0.0)
    return lineas


def leer_texto_plano(texto: str) -> list[LineaFuente]:
    return [LineaFuente(l.strip(), None) for l in texto.splitlines() if l.strip()]


def desde_archivo(ruta: Path) -> Letra:
    texto = ruta.read_text(encoding="utf-8-sig")
    lineas = leer_lrc(texto)
    if not lineas:
        lineas = leer_texto_plano(texto)
    return Letra(lineas, "usuario")


# Esperas entre reintentos a LRCLIB (429 o 5xx): ~1 min en total, que es lo
# que suelen durar sus caidas; el procesado entero tarda minutos de todos modos.
ESPERAS_LRCLIB = (2.0, 4.0, 8.0, 15.0, 15.0, 20.0)


def _pedir(ruta: str, params: dict) -> object:
    """GET a LRCLIB. 429 (limite por IP) y 5xx se reintentan respetando retry-after."""
    import time

    import requests

    ultimo: Exception | None = None
    for intento in range(len(ESPERAS_LRCLIB) + 1):
        try:
            r = requests.get(f"{LRCLIB}/{ruta}", params=params, headers={"User-Agent": AGENTE}, timeout=15)
        except requests.RequestException as e:  # red caida un momento: tambien se reintenta
            ultimo = e
            if intento < len(ESPERAS_LRCLIB):
                log.info("LRCLIB sin respuesta (%s): reintento en %.0f s", e, ESPERAS_LRCLIB[intento])
                time.sleep(ESPERAS_LRCLIB[intento])
                continue
            raise
        if r.status_code == 404:
            return None
        # 429: limite por IP; 5xx: servidor caido (503 durante minutos, visto el 2026-10-22).
        if (r.status_code == 429 or 500 <= r.status_code < 600) and intento < len(ESPERAS_LRCLIB):
            try:
                espera = float(r.headers.get("retry-after", "0"))
            except ValueError:
                espera = 0.0
            espera = min(max(espera, ESPERAS_LRCLIB[intento]), 30.0)
            log.info("LRCLIB %d: reintento en %.0f s", r.status_code, espera)
            time.sleep(espera)
            continue
        r.raise_for_status()
        return r.json()
    raise RuntimeError(f"LRCLIB no respondio: {ultimo}")


def _desde_registro(reg: dict) -> Letra | None:
    if reg.get("instrumental"):
        return None
    if reg.get("syncedLyrics"):
        lineas = leer_lrc(reg["syncedLyrics"])
        if lineas:
            return Letra(lineas, "lrclib")
    if reg.get("plainLyrics"):
        return Letra(leer_texto_plano(reg["plainLyrics"]), "lrclib")
    return None


def buscar_lrclib(artista: str, titulo: str, duracion: float) -> Letra | None:
    titulo_l = limpiar_titulo(titulo)
    # /get exige artista: sin el (archivo local sin " - " en el nombre) da 400.
    # Un fallo de /get no impide probar /search (visto el 2026-10-16).
    if artista.strip():
        try:
            reg = _pedir("get", {"artist_name": artista, "track_name": titulo_l, "duration": round(duracion)})
            if isinstance(reg, dict):
                letra = _desde_registro(reg)
                if letra:
                    log.info("LRCLIB get: %s - %s", reg.get("artistName"), reg.get("trackName"))
                    return letra
        except Exception as e:  # sigue con la busqueda general
            log.warning("LRCLIB get fallo (%s); se prueba la busqueda", e)
    try:
        candidatos = _pedir("search", {"q": f"{artista} {titulo_l}".strip()}) or []
        if not candidatos:
            candidatos = _pedir("search", {"track_name": titulo_l}) or []
    except Exception as e:
        # LRCLIB caido (503 durante minutos, visto el 2026-10-22) o sin red: no
        # es fatal, se transcribe la voz. Sin letra.json en la carpeta, el
        # siguiente procesado vuelve a preguntar a LRCLIB.
        aviso(f"LRCLIB (la base de letras) no responde ({e}): la letra se transcribe de la voz y puede tener "
              "errores. Vuelve a procesar la cancion mas tarde para buscarla de nuevo, o pega la letra.")
        return None

    # Preferir sincronizada y con duracion parecida (la version de YouTube puede tener intro).
    def puntaje(c: dict) -> float:
        dif = abs(float(c.get("duration") or 0) - duracion)
        return (0 if c.get("syncedLyrics") else 100) + dif

    validos = [c for c in candidatos if isinstance(c, dict) and abs(float(c.get("duration") or 0) - duracion) <= 15]
    for c in sorted(validos, key=puntaje):
        letra = _desde_registro(c)
        if letra:
            log.info("LRCLIB search: %s - %s (%.0fs)", c.get("artistName"), c.get("trackName"), c.get("duration") or 0)
            return letra
    return None


def _a_json(letra: Letra) -> dict:
    return {"origen": letra.origen, "lineas": [{"texto": l.texto, "inicio": l.inicio} for l in letra.lineas]}


def _de_json(d: dict) -> Letra:
    return Letra([LineaFuente(l["texto"], l["inicio"]) for l in d["lineas"]], d["origen"])


def obtener(artista: str, titulo: str, duracion: float, archivo_usuario: Path | None,
            carpeta: Path | None = None) -> Letra | None:
    """Usuario > cache > LRCLIB. None si no hay letra; la transcripcion la decide el pipeline.

    La respuesta de LRCLIB se guarda en <carpeta>/letra.json para no volver a
    pedirla al reprocesar (LRCLIB limita peticiones por IP).
    """
    import json

    etapa("letra", 0.0, "Buscando la letra")
    if archivo_usuario is not None:
        letra = desde_archivo(archivo_usuario)
        letra.lineas = limpiar_lineas(letra.lineas)
        etapa("letra", 1.0, f"Letra del usuario: {len(letra.lineas)} lineas")
        return letra
    cache = carpeta / "letra.json" if carpeta else None
    if cache and cache.exists():
        letra = _de_json(json.loads(cache.read_text(encoding="utf-8")))
        letra.lineas = limpiar_lineas(letra.lineas)
        etapa("letra", 1.0, f"Letra guardada ({letra.origen}): {len(letra.lineas)} lineas")
        return letra
    letra = buscar_lrclib(artista, titulo, duracion)
    if letra and cache:
        # La cache guarda la letra tal cual llego; la limpieza se aplica al leerla.
        cache.write_text(json.dumps(_a_json(letra), ensure_ascii=False, indent=1), encoding="utf-8")
    if letra:
        letra.lineas = limpiar_lineas(letra.lineas)
    if letra:
        tipo = "sincronizada" if letra.sincronizada else "sin tiempos"
        etapa("letra", 1.0, f"Letra de LRCLIB ({tipo}): {len(letra.lineas)} lineas")
    return letra
