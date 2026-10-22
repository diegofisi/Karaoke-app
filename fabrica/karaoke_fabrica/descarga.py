"""Descarga de YouTube con yt-dlp y conversion a WAV 44,1 kHz estereo."""

from __future__ import annotations

import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

from .eventos import ErrorFabrica, aviso, etapa, log

DURACION_MAXIMA = 15 * 60

_ID_YOUTUBE = re.compile(r"(?:v=|youtu\.be/|shorts/|music\.youtube\.com/watch\?v=)([\w-]{11})")


@dataclass
class Metadatos:
    youtube_id: str | None
    titulo: str
    artista: str
    duracion: float


def id_youtube(url: str) -> str:
    m = _ID_YOUTUBE.search(url)
    if not m:
        raise ErrorFabrica("descarga", "No se pudo leer el ID del video en la URL.")
    return m.group(1)


def a_wav(origen: Path, destino: Path) -> None:
    """Convierte cualquier audio a WAV PCM 16 bits, 44,1 kHz, estereo."""
    tmp = destino.with_suffix(".tmp.wav")
    resultado = subprocess.run(
        ["ffmpeg", "-y", "-loglevel", "error", "-i", str(origen), "-vn", "-ar", "44100", "-ac", "2",
         "-c:a", "pcm_s16le", str(tmp)],
        capture_output=True, text=True,
    )
    if resultado.returncode != 0:
        raise ErrorFabrica("descarga", f"No se pudo convertir el audio con ffmpeg: {resultado.stderr.strip()[:300]}")
    tmp.replace(destino)


def runtimes_js() -> dict:
    """Runtimes de JavaScript para yt-dlp, los que haya instalados.

    Desde finales de 2025 YouTube exige resolver un desafio en JavaScript; sin
    runtime, yt-dlp recibe HTTP 403 (visto el 2026-10-15). yt-dlp solo usa
    Deno por defecto; aqui se activan tambien Node y Bun si estan en el PATH
    (Node ya hace falta para la app). Mismo comportamiento en Windows y Mac.
    """
    import os
    import shutil

    # Una app abierta desde el Finder de macOS no hereda el PATH de la terminal:
    # se miran tambien las carpetas habituales de Homebrew, Deno y Bun.
    casa = Path.home()
    extra = os.pathsep.join(str(c) for c in (
        Path("/opt/homebrew/bin"), Path("/usr/local/bin"), casa / ".deno" / "bin", casa / ".bun" / "bin",
    ))
    ruta = os.environ.get("PATH", "") + os.pathsep + extra
    disponibles: dict = {}
    for nombre in ("deno", "node", "bun"):
        encontrado = shutil.which(nombre, path=ruta)
        if encontrado:
            disponibles[nombre] = {"path": encontrado}
    if not disponibles:
        aviso("No hay Deno, Node ni Bun en el PATH: YouTube puede rechazar la descarga (HTTP 403).")
    return disponibles


def _artista_y_titulo(info: dict) -> tuple[str, str]:
    """YouTube Music trae `artist` y `track`; un video normal solo trae el titulo."""
    artista = info.get("artist") or info.get("creator") or ""
    titulo = info.get("track") or ""
    if artista and titulo:
        return artista.split(",")[0].strip(), titulo.strip()
    crudo = info.get("title") or ""
    if " - " in crudo:
        a, t = crudo.split(" - ", 1)
        return a.strip(), t.strip()
    canal = (info.get("channel") or info.get("uploader") or "").removesuffix(" - Topic").strip()
    return canal, crudo.strip()


@dataclass
class SesionYoutube:
    """Sesion de YouTube del usuario, para cuando YouTube pide "confirmar que no
    eres un bot". `cookies`: archivo cookies.txt (formato Netscape).
    `navegador`: firefox, chrome, edge, brave, safari... (en Windows solo
    Firefox: Chrome, Edge y Brave cifran o bloquean sus cookies)."""

    cookies: Path | None = None
    navegador: str | None = None

    @property
    def activa(self) -> bool:
        return self.cookies is not None or bool(self.navegador)


# Probado el 2026-10-16: los clientes alternativos de yt-dlp (tv, web_safari,
# mweb) fallaban incluso sin bloqueo, asi que solo se reintenta el normal.
REINTENTOS_BLOQUEO = 1
ESPERA_REINTENTO_S = 5.0

AYUDA_BLOQUEO = (
    "YouTube pide confirmar que no eres un bot y bloquea la descarga desde esta conexion. "
    "Suele ser temporal (pasa tras muchas descargas seguidas). Soluciones: "
    "1) espera un rato y vuelve a intentarlo; "
    "2) en Ajustes > YouTube, inicia sesion con tu cuenta de YouTube; "
    "3) procesa la cancion desde un archivo de audio (pestana \"Archivo local\")."
)


def es_bloqueo_bot(error: Exception | str) -> bool:
    texto = str(error).lower()
    return "not a bot" in texto or "sign in to confirm" in texto


def es_sesion_rechazada(error: Exception | str) -> bool:
    return "cookies are no longer valid" in str(error).lower()


CODIGO_BLOQUEO = "bloqueo_youtube"
CODIGO_RECHAZO = "sesion_rechazada"
MENSAJE_RECHAZO = ("YouTube rechazo tu sesion (caducada o cerrada en otro sitio). "
                   "Vuelve a iniciar sesion en Ajustes > YouTube, o procesa la cancion desde un archivo de audio.")


def opciones_base(js: dict, sesion: SesionYoutube) -> dict:
    opciones: dict = {
        "quiet": True, "no_warnings": True, "noplaylist": True, "js_runtimes": js,
        # Solo audio, tambien al leer los datos: un cliente sin video no falla por eso.
        "format": "bestaudio/best",
    }
    if sesion.cookies is not None:
        opciones["cookiefile"] = str(sesion.cookies)
    elif sesion.navegador:
        opciones["cookiesfrombrowser"] = (sesion.navegador,)
    return opciones


def _error_de_sesion(e: Exception, sesion: SesionYoutube) -> str | None:
    """Mensaje en espanol si el fallo es leer la sesion (cookies) y no el video."""
    texto = str(e)
    # El aviso de bloqueo tambien dice "--cookies": no es un fallo de la sesion.
    if es_bloqueo_bot(texto):
        return None
    if sesion.navegador and ("cookie" in texto.lower() or "dpapi" in texto.lower()):
        return (f"No se pudo leer la sesion de YouTube de {sesion.navegador}: {texto}. "
                "En Windows, Chrome, Edge y Brave no dejan leer sus cookies; usa Firefox o un archivo "
                "cookies.txt (Ajustes > YouTube).")
    if sesion.cookies is not None and not sesion.cookies.is_file():
        return f"No se encontro el archivo de cookies {sesion.cookies}. Revisalo en Ajustes > YouTube."
    return None


def descargar(url: str, carpeta: Path, sesion: SesionYoutube | None = None) -> Metadatos:
    """yt-dlp reescribe el archivo de cookies al terminar: recibe una copia
    propia de esta ejecucion, que se borra al acabar (asi nunca pisa una sesion
    recien renovada; idea de youtube-downloader)."""
    sesion = sesion or SesionYoutube()
    if sesion.cookies is None or not sesion.cookies.is_file():
        return _descargar(url, carpeta, sesion)
    import shutil

    copia = carpeta / "cookies-ejecucion.txt"  # mismo nombre que youtube::COPIA_EJECUCION (Rust la limpia al arrancar)
    shutil.copyfile(sesion.cookies, copia)
    try:
        return _descargar(url, carpeta, SesionYoutube(cookies=copia, navegador=sesion.navegador))
    finally:
        copia.unlink(missing_ok=True)


def _descargar(url: str, carpeta: Path, sesion: SesionYoutube) -> Metadatos:
    import yt_dlp

    vid = id_youtube(url)
    destino = carpeta / "original.wav"
    url_limpia = f"https://www.youtube.com/watch?v={vid}"

    etapa("descarga", 0.0, "Leyendo datos del video" + (" (con tu sesion de YouTube)" if sesion.activa else ""))
    js = runtimes_js()
    info = None
    ultimo: Exception | None = None
    for intento in range(REINTENTOS_BLOQUEO + 1):
        try:
            with yt_dlp.YoutubeDL({**opciones_base(js, sesion), "skip_download": True}) as ydl:
                info = ydl.extract_info(url_limpia, download=False)
            break
        except Exception as e:  # yt-dlp lanza DownloadError y otros
            ultimo = e
            mensaje_sesion = _error_de_sesion(e, sesion)
            if mensaje_sesion:
                raise ErrorFabrica("descarga", mensaje_sesion) from e
            if sesion.activa and es_sesion_rechazada(e):
                raise ErrorFabrica("descarga", MENSAJE_RECHAZO, CODIGO_RECHAZO) from e
            if not es_bloqueo_bot(e):
                raise ErrorFabrica("descarga", f"No se pudo leer el video: {e}") from e
            if intento < REINTENTOS_BLOQUEO:
                aviso("YouTube pidio confirmar que no eres un bot; reintentando en unos segundos.")
                import time

                time.sleep(ESPERA_REINTENTO_S)
    if info is None:
        if sesion.activa:
            raise ErrorFabrica("descarga", "YouTube sigue bloqueando la descarga aun con tu sesion. "
                               "Vuelve a iniciar sesion en Ajustes > YouTube, o procesa la cancion "
                               "desde un archivo de audio.", CODIGO_RECHAZO) from ultimo
        raise ErrorFabrica("descarga", AYUDA_BLOQUEO, CODIGO_BLOQUEO) from ultimo

    duracion = float(info.get("duration") or 0)
    if duracion > DURACION_MAXIMA:
        raise ErrorFabrica("descarga", f"El video dura {duracion / 60:.1f} min; el maximo es 15 min.")
    artista, titulo = _artista_y_titulo(info)
    meta = Metadatos(youtube_id=vid, titulo=titulo, artista=artista, duracion=duracion)

    if destino.exists():
        log.info("original.wav ya existe; se reutiliza")
        etapa("descarga", 1.0, "Esta cancion ya se descargo antes: se reutiliza")
        return meta

    def progreso(d: dict) -> None:
        if d.get("status") == "downloading":
            total = d.get("total_bytes") or d.get("total_bytes_estimate") or 0
            if total:
                etapa("descarga", 0.05 + 0.85 * d.get("downloaded_bytes", 0) / total, "Descargando audio")

    plantilla = str(carpeta / "descarga.%(ext)s")
    opciones = {**opciones_base(js, sesion), "outtmpl": plantilla, "progress_hooks": [progreso]}
    try:
        with yt_dlp.YoutubeDL(opciones) as ydl:
            ydl.download([url_limpia])
    except Exception as e:
        if es_bloqueo_bot(e):
            codigo = CODIGO_RECHAZO if sesion.activa else CODIGO_BLOQUEO
            raise ErrorFabrica("descarga", MENSAJE_RECHAZO if sesion.activa else AYUDA_BLOQUEO, codigo) from e
        raise ErrorFabrica("descarga", f"No se pudo descargar el audio: {e}") from e

    bajados = [p for p in carpeta.glob("descarga.*") if p.suffix not in (".part", ".ytdl")]
    if not bajados:
        raise ErrorFabrica("descarga", "yt-dlp termino sin dejar ningun archivo de audio.")
    etapa("descarga", 0.92, "Convirtiendo a WAV")
    a_wav(bajados[0], destino)
    for p in bajados:
        p.unlink(missing_ok=True)
    etapa("descarga", 1.0, "Audio descargado")
    return meta


def artista_y_titulo_de_nombre(nombre: str) -> tuple[str, str]:
    """"Campo De Almas - Cosmica" -> ("Campo De Almas", "Cosmica"). Sin " - ": ("", nombre)."""
    if " - " in nombre:
        a, t = nombre.split(" - ", 1)
        return a.strip(), t.strip()
    return "", nombre.strip()


def importar_local(archivo: Path, carpeta: Path, titulo: str, artista: str) -> Metadatos:
    import soundfile as sf

    destino = carpeta / "original.wav"
    etapa("descarga", 0.0, "Importando audio local")
    reutilizado = destino.exists()
    if not reutilizado:
        a_wav(archivo, destino)
    duracion = sf.info(str(destino)).duration
    etapa("descarga", 1.0, "Este archivo ya se importo antes: se reutiliza" if reutilizado else "Audio importado")
    return Metadatos(youtube_id=None, titulo=titulo, artista=artista, duracion=float(duracion))
