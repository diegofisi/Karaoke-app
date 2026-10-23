# Karaoke — libro de errores del frontend

> **Este archivo crece.** Cada corrección que ha habido que hacer dos veces, y
> cada cosa rota que se encuentra en el código, se convierte en una línea
> fechada. Léelo antes de terminar cualquier cambio de frontend, junto con
> `pitfalls.md`. Una línea se borra cuando el código la hace imposible (un
> tipo, un test, una regla de `verify`), y el commit dice por qué.
>
> Formato: `- **AAAA-MM-DD** — la regla, en imperativo. Síntoma y dónde.`
>
> Las líneas del 2026-10-15 vienen de la versión anterior (rama
> `legado-python`, `frontend/src/`), revisada en
> `docs/REVISION-2026-10-15.md`. El código nuevo todavía no existe; son
> reglas para no repetirlas.

## Entradas

- **2026-10-15** — Pasa los datos del juego por refs o stores leídos en el
  momento del frame, nunca por la clausura de un `useCallback`. Síntoma: el
  bucle del juego antiguo se creaba con `useCallback(..., [songData,
  updateLyrics])` y leía `speech.fullText`, que no estaba en las
  dependencias; leía siempre el texto vacío del primer render y la
  puntuación de letra comparaba contra nada
  (`legado-python:frontend/src/pages/KaraokePage.tsx:175-191`).
- **2026-10-15** — No puntúes ni cuentes nada por `requestAnimationFrame`:
  la puntuación se hace en Rust por frames de audio de 10 ms. Síntoma: se
  sumaba un punto por frame de pantalla y una pantalla de 144 Hz puntuaba
  distinto que una de 60 Hz (`legado-python:frontend/src/audio/scorer.ts:71`).
- **2026-10-15** — No captures el micrófono en la interfaz: nada de
  `getUserMedia`. La captura está en Rust con entrada en bruto. Síntoma:
  `echoCancellation`, `noiseSuppression` y `autoGainControl` distorsionaban
  el tono y añadían retraso
  (`legado-python:frontend/src/audio/audioEngine.ts:43-49`).
- **2026-10-15** — No normalices texto quitando lo que no sea `a-z0-9`.
  Síntoma: la normalización borraba todas las palabras japonesas y no
  puntuaban (`legado-python:frontend/src/audio/scorer.ts:151-157`). La
  interfaz no compara palabras; si alguna vez tiene que normalizar para
  mostrar, NFKC y conservar kana y kanji.
- **2026-10-15** — No hagas `setState` por frame en la página del juego.
  Síntoma: `setCurrentTime`, `setUserPitch` y `setScoreState` en cada
  `requestAnimationFrame` repintaban la página entera y producían tirones
  (`legado-python:frontend/src/pages/KaraokePage.tsx:127-186`).
- **2026-10-22** — En MUI 9 no existe `inputProps` en `Checkbox`, `Switch`
  ni `Radio`: usa `slotProps={{ input: { "aria-label": … } }}`. Síntoma:
  `tsc` falla con "Property 'inputProps' does not exist on type
  CheckboxProps" (`src/features/procesar/components/TarjetaVideoPrevio.tsx`).
