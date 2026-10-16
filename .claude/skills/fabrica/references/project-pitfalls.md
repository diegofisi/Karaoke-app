# Libro de errores — karaoke, fábrica Python

Una línea fechada por cada error ya pagado. La skill lo lee antes de terminar
cualquier cambio. Un cambio que reintroduce una línea no está terminado.

Formato: `AAAA-MM-DD — la regla, en imperativo — por qué: qué se rompió — dónde`.

Las entradas del 2026-10-15 vienen de las dos versiones anteriores (rama
`legado-python` y `TranslatorMnga/karaoke-app`), revisadas en
`docs/REVISION-2026-10-15.md`.

## Entradas

- 2026-10-15 — No asignes tiempos de palabra sin alineación forzada; si la
  alineación falla, reparte dentro de la línea y márcala `metodo: "respaldo"`
  con un `aviso` — por qué: `karaoke-app` deducía el tiempo de cada palabra de
  los tramos con voz del tono y, cuando faltaban (legato), repartía a partes
  iguales sin decirlo: "Cool wind in my hair" salía 9/9/8/9/9 beats; la
  versión Python transcribía con Whisper en vez de colocar la letra conocida —
  dónde: `alineacion.py`, `notas.py`.
- 2026-10-15 — No normalices texto quitando lo que no sea `a-z0-9`; usa NFKC
  y conserva kana y kanji — por qué: esa normalización dejaba vacías todas las
  palabras japonesas y no puntuaban — dónde: `texto.py` y cualquier
  comparación de palabras en `eval/`.
- 2026-10-15 — yt-dlp necesita un runtime de JavaScript para YouTube; pásaselo
  con `js_runtimes` (Deno, Node o Bun, buscados también en las carpetas de
  Homebrew para el Finder de macOS) — por qué: sin runtime, YouTube devolvía
  HTTP 403 en la primera prueba real de la app — dónde:
  `fabrica/karaoke_fabrica/descarga.py` (`runtimes_js`).
