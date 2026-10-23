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
- 2026-10-15 — Las letras de LRCLIB traen marcas que no se cantan ("x 3",
  "x 2" sueltas, "[Chorus]"); límpialas antes de alinear (`limpiar_lineas`)
  — por qué: el alineador buscaba "x" y "3" en el audio y la frase repetida
  escrita una sola vez desplazaba las líneas siguientes — dónde:
  `fabrica/karaoke_fabrica/letra.py`.
- 2026-10-15 — No subas la confianza mínima de la alineación para "ser más
  exigente": con 0,65 empeoró (66,1 % frente a 67,5 %) porque mandaba al
  respaldo líneas buenas. Para las líneas dudosas funciona mejor una segunda
  opinión (MMS) — dónde: `alineacion.py` (`CONFIANZA_MINIMA`, `USAR_MMS`).
- 2026-10-16 — YouTube a veces responde "Sign in to confirm you're not a bot"
  (bloqueo por conexión, tras muchas descargas). Lo fiable es la sesión del
  usuario: `--cookies <cookies.txt>` (la app lo guarda al iniciar sesión).
  `--navegador` se quitó el 2026-10-22: en Windows yt-dlp no puede leer
  cookies de Chrome, Brave (cifrado DPAPI) ni Edge. Los clientes alternativos (tv,
  web_safari, mweb) fallaban incluso sin bloqueo: no se usan — dónde:
  `fabrica/karaoke_fabrica/descarga.py`.
- 2026-10-16 — En CPU, los modelos van en bfloat16: en float32 el
  reconocedor ocupaba 12 GB de RAM y en equipos de 8-16 GB parecía colgado
  (mismo resultado y velocidad en bf16). Dónde corre cada modelo lo decide la
  memoria LIBRE al cargarlo (`recursos.py`), no el tipo de equipo.
- 2026-10-16 — Al reconocer por grupos (para mostrar el avance), el grupo
  debe tener el tamaño del lote interno del reconocedor (`LOTE_ASR`): con
  grupos de 2 cambiaban los lotes y una canción bajó de 73,7 % a 39,7 %.
- 2026-10-22 — yt-dlp (≥ 2026) solo da formatos de YouTube si resuelve su
  desafío: necesita un runtime de JavaScript (Deno) **y** los scripts
  `yt-dlp-ejs`, que vienen con `yt-dlp[default]`, no con `yt-dlp` a secas.
  Síntoma: "Requested format is not available" en la descarga (Mac), con
  WARNING "Signature solving failed… challenge solver script distribution".
  Captura siempre los WARNING de yt-dlp (`RegistroYtDlp`) y ponlos en el
  error. Las rutas de `deno` y `ffmpeg` llegan de la app por `KARAOKE_DENO`
  y `KARAOKE_FFMPEG`; no busques por nombre en el PATH (en desarrollo los
  sidecars llevan la tripleta en el nombre) — dónde:
  `fabrica/karaoke_fabrica/descarga.py`, `fabrica/pyproject.toml`,
  `src-tauri/src/fabrica.rs`.
