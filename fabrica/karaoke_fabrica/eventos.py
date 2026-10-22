"""Protocolo de salida de la fabrica (docs/CONTRATO.md seccion 4).

stdout solo lleva objetos JSON, uno por linea. El registro va a stderr y a
fabrica.log. Nada mas puede escribir en stdout: por eso el arranque redirige
sys.stdout a stderr y este modulo guarda el stdout real.
"""

from __future__ import annotations

import json
import logging
import sys
from pathlib import Path
from typing import Any, TextIO

ETAPAS = ("descarga", "separacion", "letra", "alineacion", "tono", "notas", "guardado")
# "preparar" (descarga de modelos) usa su propio evento; ver preparar.py.

_salida: TextIO = sys.stdout
etapa_actual = "descarga"
log = logging.getLogger("fabrica")


def preparar_salida(carpeta_log: Path | None) -> None:
    """Reserva el stdout real para los eventos y manda todo lo demas a stderr.

    Las librerias (yt-dlp, audio-separator, transformers) imprimen en stdout;
    si no se redirige, rompen el protocolo.
    """
    global _salida
    _salida = sys.stdout
    # Siempre UTF-8: la consola de Windows (cp1252) no puede con "→", tildes de
    # otros idiomas ni japones, y un error al emitir tumbaria la fabrica.
    try:
        _salida.reconfigure(encoding="utf-8")  # type: ignore[attr-defined]
    except (AttributeError, ValueError):
        pass
    sys.stdout = sys.stderr

    manejadores: list[logging.Handler] = [logging.StreamHandler(sys.stderr)]
    if carpeta_log is not None:
        carpeta_log.mkdir(parents=True, exist_ok=True)
        manejadores.append(logging.FileHandler(carpeta_log / "fabrica.log", mode="w", encoding="utf-8"))
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
        handlers=manejadores,
        force=True,
    )


def emitir(tipo: str, **campos: Any) -> None:
    linea = json.dumps({"tipo": tipo, **campos}, ensure_ascii=False)
    _salida.write(linea + "\n")
    _salida.flush()


def etapa(nombre: str, avance: float, mensaje: str) -> None:
    global etapa_actual
    if nombre not in ETAPAS:
        raise ValueError(f"etapa desconocida: {nombre}")
    etapa_actual = nombre
    log.info("[%s %.0f%%] %s", nombre, avance * 100, mensaje)
    emitir("etapa", etapa=nombre, avance=round(max(0.0, min(1.0, avance)), 3), mensaje=mensaje)


def aviso(mensaje: str) -> None:
    log.warning(mensaje)
    emitir("aviso", mensaje=mensaje)


class ErrorFabrica(Exception):
    """Error con mensaje en espanol para el usuario y la etapa donde paso.
    `codigo` (opcional) dice a la interfaz que puede hacer: "bloqueo_youtube"
    (YouTube pide sesion y no hay) o "sesion_rechazada" (la hay pero la rechaza)."""

    def __init__(self, etapa: str, mensaje: str, codigo: str | None = None) -> None:
        super().__init__(mensaje)
        self.etapa = etapa
        self.mensaje = mensaje
        self.codigo = codigo
