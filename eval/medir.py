"""Mide el tiempo por palabra de la fabrica contra marcas hechas a mano.

Conjunto: JamendoLyrics (eval/datos/jamendolyrics, de Hugging Face
`jamendolyrics/jamendolyrics`). 20 canciones en ingles y 20 en espanol con el
inicio y el fin de cada palabra marcados a mano.

A la fabrica se le da la letra SIN tiempos (como letra del usuario), para no
filtrarle la respuesta. La separacion se guarda en eval/datos/salidas y se
reutiliza entre ejecuciones; la alineacion se rehace siempre.

Uso (desde la raiz del repo):
    fabrica/.venv/Scripts/python.exe eval/medir.py --idiomas en es [--n 5] [--desde 5] [--alinear-sobre voz|mezcla]

Las 5 primeras canciones de cada idioma se usaron para ajustar la alineacion
(2026-10-15); `--desde 5` mide en canciones que no se miraron al ajustar.

Salida: tabla por cancion y total, y un JSON en eval/resultados/<fecha>-<etiqueta>.json.
Metricas (las de doc 11, seccion 6):
  - error absoluto medio (MAE) y mediano del INICIO de palabra, en ms
  - % de inicios dentro de +-50, +-100 y +-300 ms
  - MAE del FIN de palabra (menos fiable en canto: vocales sostenidas)
"""

from __future__ import annotations

import argparse
import ast
import difflib
import json
import sys
import time
from datetime import datetime
from pathlib import Path

import numpy as np

RAIZ = Path(__file__).resolve().parents[1]
DATOS = RAIZ / "eval" / "datos" / "jamendolyrics"
# KARAOKE_EVAL_SALIDAS: otra carpeta, para medir otro separador sin mezclar sus voz.wav.
import os  # noqa: E402

SALIDAS = Path(os.environ["KARAOKE_EVAL_SALIDAS"]) if os.environ.get("KARAOKE_EVAL_SALIDAS") else RAIZ / "eval" / "datos" / "salidas"
RESULTADOS = RAIZ / "eval" / "resultados"

sys.path.insert(0, str(RAIZ / "fabrica"))


def cargar_canciones(idiomas: list[str]) -> list[dict]:
    canciones = []
    for linea in (DATOS / "metadata.jsonl").read_text(encoding="utf-8").splitlines():
        d = json.loads(linea)
        if d["language"] not in idiomas:
            continue
        palabras = d["words"]
        if isinstance(palabras, str):
            palabras = ast.literal_eval(palabras)
        d["palabras_ref"] = palabras
        canciones.append(d)
    return canciones


def emparejar(nuestras: list[dict], referencia: list[dict]) -> list[tuple[dict, dict]]:
    """Empareja palabras por su clave de texto en orden (difflib), por si el reparto difiere."""
    from karaoke_fabrica.texto import clave

    a = [clave(p["texto"]) for p in nuestras]
    b = [clave(p["text"]) for p in referencia]
    pares = []
    for bloque in difflib.SequenceMatcher(a=a, b=b, autojunk=False).get_matching_blocks():
        for k in range(bloque.size):
            pares.append((nuestras[bloque.a + k], referencia[bloque.b + k]))
    return pares


def metricas(errores_ini: np.ndarray, errores_fin: np.ndarray) -> dict:
    ab = np.abs(errores_ini)
    return {
        "palabras": int(len(ab)),
        "mae_inicio_ms": round(float(ab.mean() * 1000), 1) if len(ab) else None,
        "mediana_inicio_ms": round(float(np.median(ab) * 1000), 1) if len(ab) else None,
        "dentro_50ms": round(float((ab <= 0.05).mean() * 100), 1) if len(ab) else None,
        "dentro_100ms": round(float((ab <= 0.10).mean() * 100), 1) if len(ab) else None,
        "dentro_300ms": round(float((ab <= 0.30).mean() * 100), 1) if len(ab) else None,
        "mae_fin_ms": round(float(np.abs(errores_fin).mean() * 1000), 1) if len(errores_fin) else None,
        "sesgo_inicio_ms": round(float(np.median(errores_ini) * 1000), 1) if len(errores_ini) else None,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--idiomas", nargs="+", default=["en", "es"])
    ap.add_argument("--n", type=int, default=0, help="como mucho n canciones por idioma (0 = todas)")
    ap.add_argument("--desde", type=int, default=0,
                    help="saltar las primeras canciones de cada idioma (para medir en canciones no usadas al ajustar)")
    ap.add_argument("--alinear-sobre", choices=["voz", "mezcla"], default="voz")
    ap.add_argument("--etiqueta", default="")
    args = ap.parse_args()

    from karaoke_fabrica import eventos
    from karaoke_fabrica.pipeline import Peticion, procesar

    eventos.preparar_salida(None)

    canciones = cargar_canciones(args.idiomas)
    if args.n or args.desde:
        # Por idioma: las canciones de la posicion `desde` a `desde + n`.
        por_idioma: dict[str, int] = {}
        filtradas = []
        for c in canciones:
            k = por_idioma.get(c["language"], 0)
            if k >= args.desde and (not args.n or k < args.desde + args.n):
                filtradas.append(c)
            por_idioma[c["language"]] = k + 1
        canciones = filtradas

    filas = []
    todos_ini: dict[str, list[float]] = {}
    todos_fin: dict[str, list[float]] = {}
    for i, c in enumerate(canciones):
        carpeta = SALIDAS / c["name"]
        carpeta.mkdir(parents=True, exist_ok=True)
        letra = carpeta / "letra.txt"
        letra.write_text(c["text"], encoding="utf-8")
        t0 = time.time()
        try:
            _, ruta = procesar(Peticion(
                salida=carpeta, audio=DATOS / c["file_name"], titulo=c["title"], artista=c["artist"],
                idioma=c["language"], letra=letra, alinear_sobre=args.alinear_sobre,
            ))
        except Exception as e:  # una cancion que falla no para la medida
            print(f"[{i + 1}/{len(canciones)}] {c['name']}: FALLO {e}", file=sys.stderr)
            filas.append({"cancion": c["name"], "idioma": c["language"], "fallo": str(e)})
            continue
        cancion = json.loads(ruta.read_text(encoding="utf-8"))
        nuestras = [p for l in cancion["lineas"] for p in l["palabras"]]
        pares = emparejar(nuestras, c["palabras_ref"])
        ini = np.array([n["inicio"] - r["start"] for n, r in pares])
        fin = np.array([n["fin"] - r["end"] for n, r in pares])
        todos_ini.setdefault(c["language"], []).extend(ini.tolist())
        todos_fin.setdefault(c["language"], []).extend(fin.tolist())
        respaldo = sum(1 for l in cancion["lineas"] if l["metodo"] == "respaldo")
        fila = {"cancion": c["name"], "idioma": c["language"], "segundos": round(time.time() - t0, 1),
                "emparejadas": f"{len(pares)}/{len(c['palabras_ref'])}",
                "lineas_respaldo": f"{respaldo}/{len(cancion['lineas'])}", **metricas(ini, fin)}
        filas.append(fila)
        print(f"[{i + 1}/{len(canciones)}] {c['name']}: MAE {fila['mae_inicio_ms']} ms, "
              f"<=100ms {fila['dentro_100ms']}%, respaldo {fila['lineas_respaldo']}", file=sys.stderr)

    total = {idioma: metricas(np.array(todos_ini[idioma]), np.array(todos_fin[idioma])) for idioma in todos_ini}
    total["todos"] = metricas(np.array(sum(todos_ini.values(), [])), np.array(sum(todos_fin.values(), [])))

    RESULTADOS.mkdir(parents=True, exist_ok=True)
    nombre = datetime.now().strftime("%Y-%m-%d-%H%M") + (f"-{args.etiqueta}" if args.etiqueta else "")
    salida = {"fecha": datetime.now().isoformat(timespec="seconds"), "alinear_sobre": args.alinear_sobre,
              "total": total, "canciones": filas}
    (RESULTADOS / f"{nombre}.json").write_text(json.dumps(salida, ensure_ascii=False, indent=1), encoding="utf-8")

    print("\nidioma | palabras | MAE inicio | mediana | <=50ms | <=100ms | <=300ms | MAE fin | sesgo", file=sys.stderr)
    for idioma, m in total.items():
        print(f"{idioma:6} | {m['palabras']:8} | {m['mae_inicio_ms']:7} ms | {m['mediana_inicio_ms']:5} ms | "
              f"{m['dentro_50ms']:5}% | {m['dentro_100ms']:6}% | {m['dentro_300ms']:6}% | {m['mae_fin_ms']:5} ms | "
              f"{m['sesgo_inicio_ms']} ms", file=sys.stderr)
    print(f"\nGuardado en eval/resultados/{nombre}.json", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
