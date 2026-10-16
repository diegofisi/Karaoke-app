"""Texto de la letra: palabras por idioma, claves para comparar, japones.

La `clave` de un texto es lo que se compara entre la letra y lo que devuelve el
alineador: NFKC, minusculas, solo letras y numeros de cualquier alfabeto. No
borra kana ni kanji (el error de la version anterior: normalizar a a-z dejaba
vacio el japones).
"""

from __future__ import annotations

import re
import unicodedata
from dataclasses import dataclass
from functools import lru_cache


@dataclass
class PalabraTexto:
    texto: str          # como se muestra
    clave: str          # para casar con el alineador; puede ser "" (signos sueltos)
    lectura: str | None = None  # hiragana, solo japones


def clave(texto: str) -> str:
    t = unicodedata.normalize("NFKC", texto).lower()
    return "".join(c for c in t if c.isalnum())


def detectar_idioma(lineas: list[str]) -> str:
    """es/en/ja por el alfabeto y por palabras frecuentes. Basta para elegir modelo."""
    todo = " ".join(lineas)
    kana = sum(1 for c in todo if "぀" <= c <= "ヿ")
    kanji = sum(1 for c in todo if "一" <= c <= "鿿")
    letras = sum(1 for c in todo if c.isalpha())
    if letras and (kana + kanji) / letras > 0.2:
        return "ja"
    palabras = re.findall(r"[a-záéíóúñü']+", todo.lower())
    es = {"que", "de", "la", "el", "y", "en", "no", "me", "te", "mi", "tu", "lo", "es", "por", "con", "una", "se", "como", "yo", "amor"}
    en = {"the", "you", "i", "and", "to", "a", "me", "my", "it", "in", "of", "your", "is", "that", "love", "on", "we", "be", "all", "for"}
    n_es = sum(1 for p in palabras if p in es) + 3 * sum(1 for c in todo if c in "ñáéíóú¿¡")
    n_en = sum(1 for p in palabras if p in en)
    return "es" if n_es > n_en else "en"


def _a_hiragana(katakana: str) -> str:
    return "".join(chr(ord(c) - 0x60) if "ァ" <= c <= "ヶ" else c for c in katakana)


@lru_cache(maxsize=1)
def _tagger():
    import fugashi

    return fugashi.Tagger()


@lru_cache(maxsize=1)
def _cutlet():
    import cutlet

    k = cutlet.Cutlet()
    k.use_foreign_spelling = False
    return k


def romaji(texto: str) -> str:
    return _cutlet().romaji(texto)


def palabras_ja(linea: str) -> list[PalabraTexto]:
    """Morfemas de fugashi/unidic. Los signos se pegan a la palabra anterior para mostrarlos."""
    salida: list[PalabraTexto] = []
    for m in _tagger()(linea):
        superficie = m.surface
        k = clave(superficie)
        if not k:
            if salida:
                salida[-1].texto += superficie
            continue
        kana = getattr(m.feature, "kana", None) or getattr(m.feature, "pron", None)
        lectura = _a_hiragana(kana) if kana and kana != "*" else None
        # Sin lectura si ya es kana: el furigana solo hace falta sobre kanji.
        if lectura and all(not ("一" <= c <= "鿿") for c in superficie):
            lectura = None
        salida.append(PalabraTexto(superficie, k, lectura))
    return salida


def palabras_espacio(linea: str) -> list[PalabraTexto]:
    """es/en: por espacios. Un signo suelto ("-", "...") se pega a la palabra anterior."""
    salida: list[PalabraTexto] = []
    for trozo in linea.split():
        k = clave(trozo)
        if not k:
            if salida:
                salida[-1].texto += " " + trozo
            continue
        salida.append(PalabraTexto(trozo, k))
    return salida


def palabras(linea: str, idioma: str) -> list[PalabraTexto]:
    return palabras_ja(linea) if idioma == "ja" else palabras_espacio(linea)


def texto_para_alinear(lista: list[PalabraTexto], idioma: str) -> str:
    """Texto que se le da al alineador. En japones sin espacios."""
    if idioma == "ja":
        return "".join(p.texto for p in lista)
    return " ".join(p.texto for p in lista)
