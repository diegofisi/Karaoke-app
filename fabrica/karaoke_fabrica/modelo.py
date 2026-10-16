"""Tipos de cancion.json (docs/CONTRATO.md seccion 3).

Espejo de crates/karaoke-core. Un cambio aqui es un cambio de contrato: struct
Rust, interfaz TypeScript y CONTRATO.md cambian en el mismo commit.
"""

from __future__ import annotations

import json
import os
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Literal

VERSION = 1

Idioma = Literal["es", "en", "ja"]
OrigenLetra = Literal["lrclib", "usuario", "transcrita"]
Metodo = Literal["alineada", "respaldo"]
TipoNota = Literal["normal", "dorada", "libre"]


@dataclass
class Nota:
    inicio: float
    fin: float
    midi: float
    tipo: TipoNota = "normal"


@dataclass
class Palabra:
    texto: str
    inicio: float
    fin: float
    lectura: str | None = None
    notas: list[Nota] = field(default_factory=list)


@dataclass
class Linea:
    inicio: float
    fin: float
    texto: str
    palabras: list[Palabra]
    confianza: float
    metodo: Metodo
    romaji: str | None = None


@dataclass
class CurvaTono:
    inicio: float
    paso: float
    midi: list[float | None]


@dataclass
class Origen:
    youtube_id: str | None
    letra: OrigenLetra
    separador: str
    alineador: str
    tono: str
    fabrica_version: str
    creada: str


@dataclass
class Cancion:
    id: str
    titulo: str
    artista: str
    duracion: float
    idioma: Idioma
    origen: Origen
    lineas: list[Linea]
    tono: CurvaTono
    version: int = VERSION


def _redondear(valor: object) -> object:
    if isinstance(valor, float):
        return round(valor, 3)
    if isinstance(valor, list):
        return [_redondear(v) for v in valor]
    if isinstance(valor, dict):
        return {k: _redondear(v) for k, v in valor.items()}
    return valor


def a_dict(cancion: Cancion) -> dict:
    datos = asdict(cancion)
    # El orden de las claves sigue al contrato: version primero.
    ordenado = {"version": datos.pop("version")}
    ordenado.update(datos)
    return _redondear(ordenado)  # type: ignore[return-value]


def guardar(cancion: Cancion, ruta: Path) -> None:
    """Escribe cancion.json de forma atomica: si no existe, la cancion no esta lista."""
    tmp = ruta.with_suffix(".json.tmp")
    tmp.write_text(json.dumps(a_dict(cancion), ensure_ascii=False, indent=1), encoding="utf-8")
    os.replace(tmp, ruta)
