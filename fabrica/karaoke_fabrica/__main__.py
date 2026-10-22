"""Linea de comandos de la fabrica (docs/CONTRATO.md seccion 4)."""

from __future__ import annotations

import argparse
import sys
import traceback
from pathlib import Path

from . import eventos
from .eventos import ErrorFabrica, emitir


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(prog="karaoke_fabrica")
    sub = ap.add_subparsers(dest="orden", required=True)

    pr = sub.add_parser("procesar", help="De URL o archivo a cancion.json")
    fuente = pr.add_mutually_exclusive_group(required=True)
    fuente.add_argument("--url")
    fuente.add_argument("--audio", type=Path)
    pr.add_argument("--salida", type=Path, required=True)
    pr.add_argument("--titulo")
    pr.add_argument("--artista")
    pr.add_argument("--idioma", choices=["es", "en", "ja"])
    pr.add_argument("--letra", type=Path, help="Archivo .txt o .lrc con la letra")
    pr.add_argument("--separador", default=None)
    pr.add_argument("--alinear-sobre", choices=["voz", "mezcla"], default="voz")
    pr.add_argument("--cookies", type=Path, help="cookies.txt de YouTube (si pide confirmar que no eres un bot)")
    pr.add_argument("--navegador", help="Leer la sesion de YouTube de este navegador (firefox, safari, chrome...)")

    sub.add_parser("diagnostico", help="Python, GPU y carpeta de modelos")
    sub.add_parser("preparar", help="Descarga todos los modelos de antemano")

    args = ap.parse_args(argv)

    if args.orden == "preparar":
        eventos.preparar_salida(None)
        from .preparar import preparar

        try:
            preparar()
        except Exception as e:  # sin red, sin disco: sale como evento, no como traza en stdout
            traceback.print_exc(file=sys.stderr)
            emitir("error", etapa="preparar", mensaje=f"No se pudieron descargar los modelos: {e}")
            return 1
        emitir("listo", id="", ruta="")
        return 0

    if args.orden == "diagnostico":
        eventos.preparar_salida(None)
        from .pipeline import diagnostico

        emitir("diagnostico", **diagnostico())
        return 0

    eventos.preparar_salida(args.salida)
    from .pipeline import Peticion, procesar

    peticion = Peticion(
        salida=args.salida, url=args.url, audio=args.audio, titulo=args.titulo, artista=args.artista,
        idioma=args.idioma, letra=args.letra, separador=args.separador,
        alinear_sobre=args.alinear_sobre, cookies=args.cookies, navegador=args.navegador,
    )
    try:
        cid, ruta = procesar(peticion)
    except ErrorFabrica as e:
        emitir("error", etapa=e.etapa, mensaje=e.mensaje, **({"codigo": e.codigo} if e.codigo else {}))
        return 1
    except Exception as e:  # cualquier fallo inesperado sale como evento, nunca como traza en stdout
        traceback.print_exc(file=sys.stderr)
        emitir("error", etapa=eventos.etapa_actual, mensaje=f"No se pudo procesar la cancion: {e}")
        return 1
    emitir("listo", id=cid, ruta=str(ruta))
    return 0


if __name__ == "__main__":
    sys.exit(main())
