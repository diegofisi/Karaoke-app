"""Pruebas de las partes puras de la fabrica (sin modelos ni GPU)."""

import numpy as np
import pytest

from karaoke_fabrica.alineacion import (
    Unidad, _confianza, _ordenar, _repartir, _rellenar_huecos, anclar, casar, letra_desde_unidades,
)
from karaoke_fabrica.letra import leer_lrc, leer_texto_plano, limpiar_titulo
from karaoke_fabrica.notas import notas_de_palabra
from karaoke_fabrica.texto import clave, detectar_idioma, palabras, palabras_espacio
from karaoke_fabrica.tono import PASO, Curva, limpiar


# --- texto ---------------------------------------------------------------

def test_clave_conserva_japones_y_acentos():
    # El error de la version anterior: normalizar a a-z borraba el japones.
    assert clave("夢を見た") == "夢を見た"
    assert clave("¿Corazón?") == "corazón"
    assert clave("Ｆｕｌｌ") == "full"  # NFKC: anchos completos


def test_signos_sueltos_se_pegan_a_la_palabra_anterior():
    ps = palabras_espacio("hola - que tal ...")
    assert [p.texto for p in ps] == ["hola -", "que", "tal ..."]
    assert [p.clave for p in ps] == ["hola", "que", "tal"]


def test_palabras_japones_con_lectura():
    ps = palabras("夢を見た", "ja")
    assert "".join(p.clave for p in ps) == "夢を見た"
    yume = next(p for p in ps if p.texto.startswith("夢"))
    assert yume.lectura == "ゆめ"
    # Las palabras ya en kana no llevan furigana.
    assert all(p.lectura is None for p in ps if all("぀" <= c <= "ゟ" for c in p.texto))


def test_detectar_idioma():
    assert detectar_idioma(["君の名前を呼んだ", "夜空に"]) == "ja"
    assert detectar_idioma(["Te quiero más que a mi vida", "y no me olvides"]) == "es"
    assert detectar_idioma(["I want you to know that I love you", "all the time"]) == "en"


# --- letra ---------------------------------------------------------------

def test_leer_lrc_con_varias_marcas_y_enhanced():
    lrc = "[ar:x]\n[00:12.50]Hola mundo\n[00:20.00][01:05.25]Coro <00:20.10>aqui\n[00:30.00]\n"
    lineas = leer_lrc(lrc)
    assert [(round(l.inicio, 2), l.texto) for l in lineas] == [
        (12.5, "Hola mundo"), (20.0, "Coro aqui"), (65.25, "Coro aqui")]


def test_texto_plano_ignora_vacias():
    assert [l.texto for l in leer_texto_plano("uno\n\n dos \n")] == ["uno", "dos"]


@pytest.mark.parametrize("crudo,limpio", [
    ("Despacito (Official Video)", "Despacito"),
    ("Song [Lyric Video] ft. Someone", "Song"),
    ("夜に駆ける【公式】", "夜に駆ける"),
])
def test_limpiar_titulo(crudo, limpio):
    assert limpiar_titulo(crudo) == limpio


# --- casar unidades del alineador con nuestras palabras -----------------

def test_casar_por_palabras():
    ps = palabras_espacio("We're no strangers")
    us = [Unidad("We're", 1.0, 1.2), Unidad("no", 1.3, 1.5), Unidad("strangers", 1.6, 2.4)]
    assert casar(ps, us) == [(1.0, 1.2), (1.3, 1.5), (1.6, 2.4)]


def test_casar_por_caracteres_japones():
    ps = palabras("夢を見た", "ja")
    us = [Unidad(c, 0.1 * i, 0.1 * i + 0.1) for i, c in enumerate("夢を見た")]
    resultado = casar(ps, us)
    assert all(iv is not None for iv in resultado)
    assert resultado[0][0] == 0.0 and resultado[-1][1] == pytest.approx(0.4)


def test_casar_ignora_unidades_que_no_estan_y_deja_none():
    ps = palabras_espacio("uno dos tres")
    us = [Unidad("uno", 0, 1), Unidad("ruido", 1, 2), Unidad("tres", 2, 3)]
    assert casar(ps, us) == [(0, 1), None, (2, 3)]


def test_ordenar_quita_solapes_y_da_duracion_minima():
    salida = _ordenar([(1.0, 1.5), (1.4, 1.41), None, (1.3, 2.0)])
    assert salida[1][0] == 1.5 and salida[1][1] >= 1.55
    assert salida[2] is None
    assert salida[3][0] >= salida[1][1]


def test_repartir_proporcional_a_la_longitud():
    ps = palabras_espacio("a bbb")
    assert _repartir(ps, 0.0, 4.0) == [(0.0, 1.0), (1.0, 4.0)]


def test_confianza():
    buena = [(0.0, 0.3), (0.3, 0.7), (0.7, 1.2)]
    mala = [(0.0, 0.05), (0.05, 0.10), (0.10, 0.15)]
    assert _confianza(buena, buena) > 0.9                      # coincide con las anclas
    assert _confianza(buena, [(5.0, 5.3), None, None]) < 0.5   # contradice a su ancla
    assert _confianza(mala, [None] * 3) < 0.5                  # colapsada
    assert _confianza([None, None], [None, None]) == 0.0


def test_anclar_en_orden_con_palabras_de_mas_y_de_menos():
    letra = palabras_espacio("never gonna give you up never gonna let you down")
    # El reconocedor se salto "give" y metio "oh".
    asr = [Unidad(t, i, i + 0.5) for i, t in enumerate(
        ["never", "gonna", "you", "up", "oh", "never", "gonna", "let", "you", "down"])]
    anclas = anclar(letra, asr, "en")
    assert anclas[0] == (0, 0.5)
    assert anclas[2] is None                      # "give" no se reconocio
    assert anclas[5] == (5, 5.5)                  # segundo "never", tras el "oh"
    assert anclas[9] == (9, 9.5)


def test_anclar_japones_por_caracteres():
    letra = palabras("夢を見た", "ja")
    asr = [Unidad("夢を", 1.0, 1.4), Unidad("見た", 1.4, 2.0)]
    anclas = anclar(letra, asr, "ja")
    assert all(a is not None for a in anclas)
    assert anclas[0][0] == 1.0 and anclas[-1][1] == 2.0


def test_rellenar_huecos_entre_vecinas():
    ps = palabras_espacio("uno dos tres cuatro")
    salida = _rellenar_huecos(ps, [(0.0, 1.0), None, None, (3.0, 4.0)], 0.0, 4.0)
    assert salida[1][0] == 1.0 and salida[2][1] == 3.0


def test_letra_desde_unidades_corta_en_pausas():
    us = [Unidad("hola", 0, 0.5), Unidad("mundo", 0.5, 1.0), Unidad("otra", 2.5, 3.0), Unidad("linea", 3.0, 3.4)]
    letra = letra_desde_unidades(us, "es")
    assert [l.texto for l in letra.lineas] == ["hola mundo", "otra linea"]
    assert letra.lineas[1].inicio == 2.5


# --- tono ----------------------------------------------------------------

def test_limpiar_corrige_salto_de_octava_y_quita_islas():
    m = np.full(200, 60.0)
    m[100:110] = 72.0         # 10 frames una octava arriba: error del detector
    m[150:152] = np.nan
    m[180:200] = np.nan
    m[190:192] = 65.0         # isla de 2 frames (32 ms): resto de separacion
    salida = limpiar(m)
    assert np.allclose(salida[100:110], 60.0)
    assert np.isnan(salida[190:192]).all()


def _curva(valores):
    return Curva(inicio=0.0, paso=PASO, midi=np.array(valores, dtype=float),
                 confianza=np.ones(len(valores)), volumen_db=np.zeros(len(valores)))


def test_vibrato_no_parte_la_nota():
    t = np.arange(60) * PASO
    vibrato = 64 + 0.6 * np.sin(2 * np.pi * 6 * t)  # +-0,6 semitonos a 6 Hz
    notas = notas_de_palabra(_curva(vibrato), 0.0, 60 * PASO)
    assert len(notas) == 1
    assert notas[0].midi == pytest.approx(64, abs=0.2)


def test_melisma_da_dos_notas():
    valores = [60.0] * 30 + [64.0] * 30
    notas = notas_de_palabra(_curva(valores), 0.0, 60 * PASO)
    assert [round(n.midi) for n in notas] == [60, 64]
    assert notas[0].inicio == 0.0 and notas[-1].fin == pytest.approx(60 * PASO)
    assert notas[0].fin == pytest.approx(notas[1].inicio)


def test_palabra_sin_voz_no_tiene_notas():
    assert notas_de_palabra(_curva([np.nan] * 40), 0.0, 40 * PASO) == []
