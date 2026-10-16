# Persistencia — carpetas de canción y `records.json`

**Aplica** al tocar `crates/biblioteca`, `crates/karaoke-core` o cualquier
escritura en disco. Hoy no hay base de datos. Si algún día entra `rusqlite`
(por ejemplo, para muchos récords), se reescribe este archivo antes.

## Dónde está cada cosa (CONTRATO §2)

`<datos>/canciones/<id>/`, con `<datos>` = `app_data_dir` de Tauri e `<id>` =
`yt-<id de YouTube>` o `local-<hash>`.

| Archivo | Lo escribe | Rust |
|---|---|---|
| `original.wav`, `voz.wav`, `instrumental.wav` | la fábrica | solo lee (`instrumental.wav` para reproducir) |
| `cancion.json` | la fábrica, al final, atómico | **solo lee**. Si no existe, la canción no está lista |
| `fabrica.log` | la fábrica | solo lee, si la interfaz enseña el registro |
| `records.json` | **Rust** | lee y escribe |

## Reglas

- **Rust nunca escribe `cancion.json` ni los `.wav`.** Si un dato de la
  canción está mal, se arregla en la fábrica y se vuelve a procesar.
- **`biblioteca` recibe la carpeta**, no la calcula. `src-tauri` la saca de
  `app.path().app_data_dir()`. Una ruta relativa a `CARGO_MANIFEST_DIR` solo
  vale en tests.
- **Una carpeta sin `cancion.json` no es una canción lista.** Puede ser una
  canción a medio procesar o un fallo. El listado la marca así, no la omite
  en silencio y no la borra.
- **`version` de `cancion.json`**: la lectura comprueba `version`. Una versión
  mayor que la conocida es "No se pudo abrir la canción: la creó una versión
  más nueva". Un campo nuevo opcional lleva `#[serde(default)]`.

## Escribir `records.json`

- Escritura **atómica**: escribir `records.json.tmp` en la misma carpeta,
  `sync_all`, y `fs::rename` encima del bueno. Un corte a mitad nunca deja un
  archivo a medias.
- Leer, modificar y escribir el archivo entero. Es pequeño.
- Un `records.json` que no parsea no se sobrescribe en silencio: se renombra a
  `records.json.roto` y se avisa.
- Las fechas en el archivo van en RFC 3339 con zona (como `origen.creada`).

## Ajustes del jugador

`latencia_ms`, dispositivos de entrada y salida y similares. **Dónde se
guardan no está fijado en CONTRATO.md.** Antes de escribirlos, se añade al
contrato (archivo y forma) y lo aprueba el usuario.

## Qué se persiste y qué no

| Persistido | No persistido |
|---|---|
| Récords, ajustes | Frames del juego, tono del micrófono, puntuación en curso |
| Lo que produce la fábrica (lo escribe ella) | Cachés que se pueden reconstruir desde la carpeta |

## Prueba manual (la hace el usuario con `npm run tauri dev`)

1. Carpeta de datos vacía: la biblioteca abre vacía, sin error.
2. Una canción a medio procesar (sin `cancion.json`): se lista como no lista.
3. Jugar, cerrar la app, abrir: el récord sigue.

El agente la pide y la anota en el reporte; no la ejecuta.
