"""Pruebas de las partes puras de la fabrica (sin modelos ni GPU)."""

import numpy as np
import pytest

from karaoke_fabrica.alineacion import (
    Unidad, _confianza, _ordenar, _repartir, _rellenar_huecos, anclar, casar, letra_desde_unidades,
)
from karaoke_fabrica.letra import buscar_lrclib, leer_lrc, leer_texto_plano, limpiar_titulo
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


# --- limpieza de la letra -----------------------------------------------------

from karaoke_fabrica.letra import LineaFuente, limpiar_lineas  # noqa: E402


def test_limpiar_lineas_expande_repeticiones_y_quita_marcas():
    entrada = [
        LineaFuente("[Chorus]", 1.0),
        LineaFuente("We came to burn this down x 3", 10.0),
        LineaFuente("Hey! Hey!", 20.0),
        LineaFuente("x 2", 30.0),
        LineaFuente("(x2)", 31.0),
        LineaFuente("Oh oh (3x)", 40.0),
        LineaFuente("Coro:", 50.0),
        LineaFuente("Max 5 veces", 60.0),
    ]
    salida = limpiar_lineas(entrada)
    assert [(l.texto, l.inicio) for l in salida] == [
        ("We came to burn this down", 10.0),
        ("We came to burn this down", None),
        ("We came to burn this down", None),
        ("Hey! Hey!", 20.0),
        ("Oh oh", 40.0),
        ("Oh oh", None),
        ("Oh oh", None),
        ("Max 5 veces", 60.0),
    ]


def test_linea_entre_parentesis_que_se_canta_se_conserva():
    salida = limpiar_lineas([LineaFuente("(Oh, baby)", None)])
    assert [l.texto for l in salida] == ["(Oh, baby)"]


# --- fin de palabra por actividad de la voz --------------------------------------

from karaoke_fabrica.alineacion import extender_fines  # noqa: E402
from karaoke_fabrica.modelo import Linea, Palabra  # noqa: E402


def test_extender_fines_sigue_la_voz_sin_pisar_la_siguiente():
    # Voz de 0,0 a 2,0 s (frames de 20 ms) y silencio despues.
    activa = np.array([True] * 100 + [False] * 100)
    linea = Linea(inicio=0.0, fin=1.4, texto="uno dos", confianza=1.0, metodo="alineada", palabras=[
        Palabra("uno", 0.0, 0.3),   # la voz sigue: se alarga hasta el inicio de "dos"
        Palabra("dos", 1.0, 1.4),   # se alarga hasta que acaba la voz (2,0 s)
    ])
    extender_fines([linea], activa)
    assert linea.palabras[0].fin == pytest.approx(0.98, abs=0.021)
    assert linea.palabras[1].fin == pytest.approx(2.0, abs=0.021)
    assert linea.fin == linea.palabras[1].fin


def test_extender_fines_no_acorta():
    activa = np.zeros(100, dtype=bool)
    linea = Linea(inicio=0.0, fin=0.5, texto="a", confianza=1.0, metodo="alineada", palabras=[Palabra("a", 0.0, 0.5)])
    extender_fines([linea], activa)
    assert linea.palabras[0].fin == 0.5


@pytest.mark.parametrize("linea", [
    "Bridge over troubled water", "Puente roto", "Hook me up", "Bis a bis", "Intro to love",
])
def test_letra_que_empieza_por_palabra_de_seccion_se_conserva(linea):
    assert [l.texto for l in limpiar_lineas([LineaFuente(linea, None)])] == [linea]


@pytest.mark.parametrize("linea", ["[Chorus]", "(Verso 2)", "Coro:", "Chorus 2", "[Hook: Artista]", "{Bridge}", "Pre-Chorus"])
def test_cabeceras_de_seccion_se_quitan(linea):
    assert limpiar_lineas([LineaFuente(linea, None)]) == []


# --- descarga: bloqueo de YouTube y sesion ----------------------------------------------

from pathlib import Path  # noqa: E402

from karaoke_fabrica.descarga import SesionYoutube, es_bloqueo_bot, opciones_base  # noqa: E402


def test_detecta_el_bloqueo_de_bot():
    assert es_bloqueo_bot("ERROR: [youtube] x: Sign in to confirm you're not a bot. Use --cookies")
    assert not es_bloqueo_bot("ERROR: Video unavailable")


def test_opciones_con_cookies_y_cliente():
    o = opciones_base({}, SesionYoutube(cookies=Path("c.txt")))
    assert o["cookiefile"] == "c.txt" and o["format"] == "bestaudio/best"
    o = opciones_base({}, SesionYoutube())
    assert "cookiefile" not in o and o["noplaylist"] is True


from karaoke_fabrica.descarga import artista_y_titulo_de_nombre  # noqa: E402


def test_artista_y_titulo_del_nombre_del_archivo():
    assert artista_y_titulo_de_nombre("Campo De Almas - Cósmica") == ("Campo De Almas", "Cósmica")
    assert artista_y_titulo_de_nombre("solo titulo") == ("", "solo titulo")


def test_bloqueo_no_se_confunde_con_fallo_de_cookies(tmp_path):
    from karaoke_fabrica.descarga import _error_de_sesion

    bloqueo = Exception("ERROR: [youtube] x: Sign in to confirm you're not a bot. Use --cookies-from-browser or --cookies")
    assert _error_de_sesion(bloqueo, SesionYoutube(cookies=tmp_path / "no-existe.txt")) is None
    otro = Exception("ERROR: Video unavailable")
    assert "No se encontro" in _error_de_sesion(otro, SesionYoutube(cookies=tmp_path / "no-existe.txt"))
    assert _error_de_sesion(otro, SesionYoutube()) is None


def test_id_local_es_sha1_del_archivo_completo(tmp_path):
    from karaoke_fabrica.pipeline import Peticion, id_cancion

    archivo = tmp_path / "a.bin"
    archivo.write_bytes(b"hola")
    # Mismo valor que la prueba de Rust (src-tauri/src/fabrica.rs): los dos lados deben coincidir.
    assert id_cancion(Peticion(salida=tmp_path, audio=archivo)) == "local-99800b85d338"


def test_descarga_usa_una_copia_de_las_cookies_y_la_borra(tmp_path, monkeypatch):
    import karaoke_fabrica.descarga as d

    original = tmp_path / "cookies.txt"
    CONTENIDO = "# Netscape HTTP Cookie File" + chr(10)
    original.write_text(CONTENIDO, encoding="utf-8")
    vistas = {}

    def falso(url, carpeta, sesion):
        vistas["ruta"] = sesion.cookies
        vistas["existia"] = sesion.cookies.is_file()
        raise d.ErrorFabrica("descarga", "parar aqui")

    monkeypatch.setattr(d, "_descargar", falso)
    with pytest.raises(d.ErrorFabrica):
        d.descargar("https://youtu.be/dQw4w9WgXcQ", tmp_path, SesionYoutube(cookies=original))
    assert vistas["ruta"] != original and vistas["existia"]
    assert not vistas["ruta"].exists(), "la copia se borra al terminar"
    assert original.read_text(encoding="utf-8") == CONTENIDO


def test_sesion_rechazada_se_detecta():
    from karaoke_fabrica.descarga import es_sesion_rechazada

    assert es_sesion_rechazada("ERROR: The provided YouTube account cookies are no longer valid.")
    assert not es_sesion_rechazada("Video unavailable")


def test_lrclib_caido_no_es_fatal(monkeypatch):
    """Sin LRCLIB se sigue (se transcribe la voz): nunca se cuelga el procesado."""
    import karaoke_fabrica.letra as m

    avisos: list[str] = []
    monkeypatch.setattr(m, "aviso", avisos.append)

    def caido(ruta, params):
        raise RuntimeError("503 Service Unavailable")

    monkeypatch.setattr(m, "_pedir", caido)
    assert buscar_lrclib("Artista", "Titulo", 200.0) is None
    assert avisos and "LRCLIB" in avisos[0] and "transcribe" in avisos[0]

    monkeypatch.setattr(m, "_pedir", lambda ruta, params: None)
    assert buscar_lrclib("Artista", "Titulo", 200.0) is None


def test_lrclib_reintenta_los_5xx_y_devuelve_al_recuperarse(monkeypatch):
    import karaoke_fabrica.letra as m

    class Resp:
        def __init__(self, codigo, cuerpo=None):
            self.status_code, self.headers, self._cuerpo = codigo, {}, cuerpo

        def json(self):
            return self._cuerpo

        def raise_for_status(self):
            if self.status_code >= 400:
                raise RuntimeError(self.status_code)

    respuestas = [Resp(503), Resp(503), Resp(200, {"id": 1})]
    import requests

    monkeypatch.setattr(requests, "get", lambda *a, **k: respuestas.pop(0))
    esperas: list[float] = []
    import time

    monkeypatch.setattr(time, "sleep", esperas.append)
    assert m._pedir("get", {}) == {"id": 1}
    assert esperas == [2.0, 4.0]


def test_runtimes_js_usa_el_deno_que_pasa_la_app(tmp_path, monkeypatch):
    from karaoke_fabrica.descarga import runtimes_js

    deno = tmp_path / "deno-x86_64-pc-windows-msvc.exe"
    deno.write_bytes(b"")
    monkeypatch.setenv("KARAOKE_DENO", str(deno))
    monkeypatch.setenv("PATH", "")
    assert runtimes_js()["deno"] == {"path": str(deno)}
    # Una ruta que no existe se ignora, sin romper.
    monkeypatch.setenv("KARAOKE_DENO", str(tmp_path / "no-existe"))
    assert "deno" not in runtimes_js() or runtimes_js()["deno"]["path"] != str(tmp_path / "no-existe")


def test_sin_formatos_explica_runtime_y_version():
    from karaoke_fabrica.descarga import explicar_lectura

    e = Exception("ERROR: [youtube] x: Requested format is not available. Use --list-formats")
    m = explicar_lectura(e, {})
    assert "runtime de JavaScript" in m and "ninguno" in m and "yt-dlp 20" in m
    m = explicar_lectura(e, {"deno": {"path": "/app/deno"}}, ["[youtube] x: Some web client https formats have been skipped as they are missing a PO token"])
    assert "deno (/app/deno)" in m and "yt-dlp dijo: [youtube] x: Some web client" in m
    assert explicar_lectura(Exception("Video unavailable"), {}) == "No se pudo leer el video: Video unavailable"


def test_sin_formatos_con_sesion_reintenta_anonimo(tmp_path, monkeypatch):
    """Con sesion y "Requested format is not available", se vuelve a leer sin cookies."""
    import karaoke_fabrica.descarga as d

    vistas: list[bool] = []

    class FalsoYdl:
        def __init__(self, opciones):
            vistas.append("cookiefile" in opciones)

        def __enter__(self):
            return self

        def __exit__(self, *a):
            return False

        def extract_info(self, url, download=False):
            if vistas[-1]:
                raise Exception("ERROR: [youtube] x: Requested format is not available")
            return {"duration": 100.0, "title": "A - B"}

        def download(self, urls):
            (tmp_path / "descarga.webm").write_bytes(b"x")

    import sys

    # `_descargar` hace `import yt_dlp` dentro: se sustituye el modulo entero.
    monkeypatch.setitem(sys.modules, "yt_dlp", type("M", (), {"YoutubeDL": FalsoYdl}))
    monkeypatch.setattr(d, "runtimes_js", lambda: {})
    monkeypatch.setattr(d, "a_wav", lambda origen, destino: destino.write_bytes(b"wav"))
    monkeypatch.setattr(d, "aviso", lambda m: None)
    cookies = tmp_path / "c.txt"
    cookies.write_text("# Netscape HTTP Cookie File\n")
    meta = d._descargar("https://youtu.be/ppqGf02Pii8", tmp_path, d.SesionYoutube(cookies=cookies))
    assert vistas[:2] == [True, False] and meta.titulo == "B" and (tmp_path / "original.wav").exists()


def test_ffmpeg_usa_la_ruta_que_pasa_la_app(tmp_path, monkeypatch):
    from karaoke_fabrica.descarga import ffmpeg

    exe = tmp_path / "ffmpeg-x86_64-pc-windows-msvc.exe"
    exe.write_bytes(b"")
    monkeypatch.setenv("KARAOKE_FFMPEG", str(exe))
    assert ffmpeg() == str(exe)
    monkeypatch.setenv("KARAOKE_FFMPEG", str(tmp_path / "no-existe"))
    assert ffmpeg() == "ffmpeg"
