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
        return bool(self.lineas) and all(l.inicio is not None for l in self.lineas)


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


def _pedir(ruta: str, params: dict) -> object:
    """GET a LRCLIB. LRCLIB limita por IP (429): se reintenta respetando retry-after."""
    import time

    import requests

    for intento in range(4):
        r = requests.get(f"{LRCLIB}/{ruta}", params=params, headers={"User-Agent": AGENTE}, timeout=15)
        if r.status_code == 404:
            return None
        if r.status_code == 429 and intento < 3:
            try:
                espera = float(r.headers.get("retry-after", "0"))
            except ValueError:
                espera = 0.0
            espera = min(max(espera, 2.0 * (intento + 1)), 15.0)
            log.info("LRCLIB 429: reintento en %.0f s", espera)
            time.sleep(espera)
            continue
        r.raise_for_status()
        return r.json()
    return None


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
    try:
        reg = _pedir("get", {"artist_name": artista, "track_name": titulo_l, "duration": round(duracion)})
        if isinstance(reg, dict):
            letra = _desde_registro(reg)
            if letra:
                log.info("LRCLIB get: %s - %s", reg.get("artistName"), reg.get("trackName"))
                return letra
        candidatos = _pedir("search", {"q": f"{artista} {titulo_l}".strip()}) or []
        if not candidatos:
            candidatos = _pedir("search", {"track_name": titulo_l}) or []
    except Exception as e:  # red caida, LRCLIB caido: no es fatal
        aviso(f"No se pudo consultar LRCLIB: {e}")
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
        etapa("letra", 1.0, f"Letra del usuario: {len(letra.lineas)} lineas")
        return letra
    cache = carpeta / "letra.json" if carpeta else None
    if cache and cache.exists():
        letra = _de_json(json.loads(cache.read_text(encoding="utf-8")))
        etapa("letra", 1.0, f"Letra guardada ({letra.origen}): {len(letra.lineas)} lineas")
        return letra
    letra = buscar_lrclib(artista, titulo, duracion)
    if letra and cache:
        cache.write_text(json.dumps(_a_json(letra), ensure_ascii=False, indent=1), encoding="utf-8")
    if letra:
        tipo = "sincronizada" if letra.sincronizada else "sin tiempos"
        etapa("letra", 1.0, f"Letra de LRCLIB ({tipo}): {len(letra.lineas)} lineas")
    return letra
