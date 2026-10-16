# Contrato

Fuente de verdad de todo lo que cruza una frontera: fábrica → disco → Rust →
interfaz. Si el código y este archivo discrepan, se anota en
`.claude/skills/backend/references/project.md` → Discrepancias y decide el
usuario. Los cambios de contrato se hacen en todos sus sitios en el mismo
cambio (struct Rust, interfaz TypeScript, modelo Python, esta tabla).

## 1. Convenciones

- Nombres en español, sin tildes, `snake_case`, en JSON, Rust (serde con
  `rename_all = "snake_case"`), TypeScript y Python. La interfaz no traduce
  nombres.
- Tiempos en **segundos** (`f64`), desde el inicio del audio original.
- Tono en **MIDI con decimales** (`69.0` = La 440 Hz). `null` = sin voz.
- Errores hacia la interfaz: `String` en español que empieza por "No se pudo…".
- Texto visible de la interfaz: español.

## 2. Carpeta de una canción

`<datos>/canciones/<id>/`, donde `<datos>` es `app_data_dir` de Tauri y `<id>`
es `yt-<id de YouTube>` o `local-<hash>`.

| Archivo | Quién lo escribe | Contenido |
|---|---|---|
| `original.wav` | fábrica | Audio descargado, 44,1 kHz estéreo |
| `voz.wav` | fábrica | Voz separada, 44,1 kHz estéreo |
| `instrumental.wav` | fábrica | Instrumental, 44,1 kHz estéreo |
| `cancion.json` | fábrica | Sección 3. Se escribe **al final**, de forma atómica (tmp + rename). Si no existe, la canción no está lista |
| `fabrica.log` | fábrica | Registro de la última ejecución |
| `records.json` | Rust | Récords del jugador |

## 3. `cancion.json` (versión 1)

```json
{
  "version": 1,
  "id": "yt-dQw4w9WgXcQ",
  "titulo": "Never Gonna Give You Up",
  "artista": "Rick Astley",
  "duracion": 213.1,
  "idioma": "en",
  "origen": {
    "youtube_id": "dQw4w9WgXcQ",
    "letra": "lrclib",
    "separador": "vocals_mel_band_roformer.ckpt",
    "alineador": "Qwen/Qwen3-ForcedAligner-0.6B",
    "tono": "swift-f0",
    "fabrica_version": "0.1.0",
    "creada": "2026-10-15T18:30:00Z"
  },
  "lineas": [
    {
      "inicio": 18.92,
      "fin": 21.40,
      "texto": "We're no strangers to love",
      "romaji": null,
      "confianza": 0.93,
      "metodo": "alineada",
      "palabras": [
        {
          "texto": "We're",
          "lectura": null,
          "inicio": 18.92,
          "fin": 19.18,
          "notas": [
            { "inicio": 18.92, "fin": 19.18, "midi": 64.1, "tipo": "normal" }
          ]
        }
      ]
    }
  ],
  "tono": {
    "inicio": 0.0,
    "paso": 0.016,
    "midi": [null, null, 64.08, 64.12]
  }
}
```

| Campo | Tipo | Regla |
|---|---|---|
| `idioma` | `"es" \| "en" \| "ja"` | Otros idiomas: no soportados en la versión 1 |
| `origen.letra` | `"lrclib" \| "usuario" \| "transcrita"` | De dónde salió el texto |
| `lineas[].metodo` | `"alineada" \| "respaldo"` | `respaldo` = la alineación falló o dio confianza baja y los tiempos se repartieron dentro de la línea. La interfaz puede marcarla |
| `lineas[].confianza` | `0.0 – 1.0` | Calidad de la alineación de la línea |
| `lineas[].romaji` | `string \| null` | Solo japonés |
| `palabras[].lectura` | `string \| null` | Solo japonés: lectura en hiragana (furigana) |
| `palabras[].notas` | lista, puede estar vacía | Vacía = palabra sin tono fiable (hablada, susurro). Se canta pero no puntúa tono |
| `notas[].midi` | `f64` | Mediana del tono de referencia en la nota |
| `notas[].tipo` | `"normal" \| "dorada" \| "libre"` | `dorada` vale doble; `libre` no puntúa |
| `tono` | curva | Tono de referencia frame a frame (SwiftF0, suavizado). Para dibujar la guía y para el modo de puntuación por contorno |

Orden garantizado: `lineas` por `inicio`; `palabras` por `inicio`; `notas` por
`inicio`; sin solapes dentro de una línea.

## 4. Protocolo de la fábrica

Ejecución:

```
python -m karaoke_fabrica procesar --url <url> --salida <carpeta> [--idioma es|en|ja] [--letra <archivo.txt|.lrc>]
                                   [--cookies <cookies.txt> | --navegador <firefox|safari|...>]
python -m karaoke_fabrica procesar --audio <archivo> --titulo <t> --artista <a> --salida <carpeta> [...]
python -m karaoke_fabrica diagnostico
python -m karaoke_fabrica preparar     # descarga todos los modelos de antemano
```

Cada línea de **stdout** es un objeto JSON. Nada más va a stdout (el registro va
a stderr y a `fabrica.log`).

| `tipo` | Campos | Cuándo |
|---|---|---|
| `"etapa"` | `etapa`, `avance` (0–1), `mensaje` | Al empezar cada etapa y al avanzar |
| `"aviso"` | `mensaje` | Algo degradado pero no fatal (sin letra en LRCLIB, línea en respaldo) |
| `"listo"` | `id`, `ruta` (de `cancion.json`) | Fin con éxito |
| `"error"` | `mensaje` (español), `etapa` | Fin con fallo. Código de salida ≠ 0 |
| `"preparar"` | `paso` (`separador`, `reconocedor`, `alineador`, `mms`), `avance`, `mensaje` | Solo en `preparar`; al final, `listo` con `id` y `ruta` vacíos |

`etapa` ∈ `descarga`, `separacion`, `letra`, `alineacion`, `tono`, `notas`,
`guardado`.

`diagnostico` escribe un único objeto
`{"tipo":"diagnostico","python":"3.12.x","sistema":"Windows"|"Darwin","torch":"2.x","cuda":true,"aceleracion":"cuda"|"mps"|"cpu","gpu":"NVIDIA ..."|"Apple arm64 (MPS)"|null,"modelos":"<carpeta>"}`.
`aceleracion` = `mps` es la GPU de Apple Silicon en Mac.

## 5. Comandos de Tauri

Cada comando tiene su envoltorio tipado en `src/common/api/comandos.ts` (el
único archivo con `invoke`) y su registro en `generate_handler!`. Los
argumentos se escriben en `snake_case` también en TypeScript
(`#[tauri::command(rename_all = "snake_case")]`). Todos devuelven
`Result<T, String>`; el `String` es el mensaje en español para la interfaz.

| Comando | Argumentos | Devuelve |
|---|---|---|
| `estado_fabrica` | — | `EstadoFabrica` |
| `listar_canciones` | — | `ResumenCancion[]` (orden: más reciente primero) |
| `obtener_cancion` | `id: string` | `Cancion` (sección 3) |
| `borrar_cancion` | `id: string` | `null` |
| `procesar_cancion` | `peticion: PeticionProceso`, `canal: Channel<EventoFabrica>` | `string` (id de la canción). Vuelve en cuanto el proceso arranca; el resto llega por el canal |
| `cancelar_proceso` | `id: string` | `null` |
| `instalar_fabrica` | `canal: Channel<EventoInstalacion>` | `null`. Instala con `uv` (empaquetado) Python 3.12, PyTorch (CUDA, CPU o MPS), la fábrica y los modelos en `<datos>/entorno/`. Tarda minutos |
| `listar_microfonos` | — | `Microfono[]` |
| `obtener_ajustes` | — | `Ajustes` |
| `guardar_ajustes` | `ajustes: Ajustes` | `Ajustes` |
| `calibrar_latencia` | `canal: Channel<EventoCalibracion>` | `ResultadoCalibracion` |
| `iniciar_juego` | `id: string`, `modo: Modo`, `desde: number`, `canal: Channel<FrameJuego>` | `null` |
| `pausar_juego` | — | `null` |
| `reanudar_juego` | — | `null` |
| `buscar_juego` | `t: number` | `null` (solo en modo `practica`) |
| `fijar_voz_guia` | `volumen: number` | `null`. Pone (0–1) o quita (0) la voz original durante la partida |
| `detener_juego` | — | `FinPartida \| null` (`null` si no había partida). Si se grababa, espera a que la grabación se guarde |
| `listar_grabaciones` | — | `Grabacion[]` (más reciente primero) |
| `abrir_grabacion` | `ruta: string` | `null`. La abre con el reproductor del sistema |
| `mostrar_grabacion` | `ruta: string` | `null`. La muestra en el Explorador (Windows) o el Finder (Mac) |
| `borrar_grabacion` | `ruta: string` | `null`. Solo acepta rutas dentro de la carpeta de grabaciones |

### Tipos

```ts
EstadoFabrica   { lista: boolean,   // todos los requisitos obligatorios en verde
                  python: string | null, cuda: boolean, gpu: string | null, mensaje: string | null,
                  requisitos: Requisito[], puede_instalar: boolean }
Requisito       { id: "fabrica" | "ffmpeg" | "javascript" | "gpu", nombre: string, ok: boolean,
                  obligatorio: boolean,   // gpu: false (solo avisa)
                  detalle: string }
ResumenCancion  { id, titulo, artista, duracion: number, idioma: "es"|"en"|"ja",
                  lineas_respaldo: number, lineas_total: number, record: number | null }
PeticionProceso { url: string | null, archivo: string | null, titulo: string | null, artista: string | null,
                  idioma: "es"|"en"|"ja" | null, letra: string | null }   // letra: texto pegado (.txt o LRC)
Microfono       { id: string, nombre: string, por_defecto: boolean }
Dificultad      = "facil" | "normal" | "dificil"
Modo            = "practica" | "puntuacion"
Ajustes         { microfono: string | null, latencia_ms: number, dificultad: Dificultad,
                  volumen_musica: number, volumen_guia: number, volumen_voz: number,
                  mostrar_romaji: boolean, grabar: boolean, carpeta_fabrica: string | null,
                  youtube_cookies: string | null,   // cookies.txt; prioridad sobre el navegador
                  youtube_navegador: string | null } // firefox | safari | chrome | edge | brave
                  // volúmenes 0–1; volumen_guia 0 = sin voz guía. volumen_voz 0–3 (1 = automático).
                  // carpeta_fabrica: la del repositorio o `fabrica/`; null = la de desarrollo.
                  // Por defecto: null, 150, "normal", 0.9, 0, 1, true, false, null, null, null
ResultadoCalibracion { latencia_ms: number, medidas_ms: number[], fiable: boolean }
ResultadoJuego  { puntos: number, nivel: Nivel, mejor_combo: number, precision_media: number,
                  por_calificacion: { perfecta, bien, regular, fallo }, record_anterior: number | null, es_record: boolean }
Nivel           = "desafinado" | "aficionado" | "prometedor" | "cantante" | "estrella" | "leyenda"
FinPartida      { resultado: ResultadoJuego | null,   // null en practica
                  grabacion: Grabacion | null, error_grabacion: string | null }
Grabacion       { ruta: string, nombre: string, id_cancion: string, titulo: string, artista: string,
                  fecha: string /* RFC 3339 */, duracion: number }
```

**Grabaciones** (`ajustes.grabar`): durante la partida se graba el micrófono;
al terminar se mezcla con lo que sonaba (instrumental y, si estaba activa, la
voz guía a su volumen), con la voz colocada por la fórmula de la sección 7. Se
guarda como MP3 (ffmpeg; WAV si ffmpeg falla) en Música/Karaoke
(`audio_dir` del sistema), con un `.json` de datos al lado.

Ajustes y récords se guardan en `<datos>/ajustes.json` y
`<datos>/canciones/<id>/records.json`.

## 6. Canales (Rust → interfaz)

Se usan `tauri::ipc::Channel<T>`, no eventos globales: cada canal es de una
sola llamada y se cierra con ella.

**`EventoFabrica`**: los objetos de la sección 4 tal cual (`etapa`, `aviso`,
`listo`, `error`), más `{ tipo: "cancelado" }` si se canceló. Tras `listo`,
`error` o `cancelado` no llega nada más.

**`EventoInstalacion`**: `{ tipo: "paso", indice, total, nombre, mensaje }` al
empezar cada paso (`python`, `entorno`, `pytorch`, `fabrica`, `modelos`) y
`{ tipo: "linea", texto }` con la salida de cada paso.

**`EventoCalibracion`**: `{ tipo: "pitido", indice: number, total: number }`
cuando suena cada pitido y `{ tipo: "medida", indice: number, ms: number | null }`
al detectarlo (o no).

**`FrameJuego`**: uno cada ~16 ms mientras el juego corre.

```ts
FrameJuego {
  t: number,                          // tiempo de canción que SE OYE ahora (s)
  tonos: { t: number, midi: number | null }[],  // frames del micrófono desde el anterior,
                                      // con t ya corregido por latencia_ms (cada 10 ms)
  puntos: number, combo: number,
  eventos: EventoPuntuacion[],        // notas y líneas cerradas desde el anterior
  estado: "sonando" | "pausado" | "terminado"
}
EventoPuntuacion = { tipo: "nota_cerrada", indice: number, calificacion: "perfecta"|"bien"|"regular"|"fallo", credito: number }
                 | { tipo: "linea_cerrada", indice_linea: number, ratio: number, bono: number }
```

La interfaz interpola `t` con `performance.now()` entre frames y nunca lleva
su propio reloj. En modo `practica` no hay puntuación: `puntos` y `combo` van a
0 y `eventos` vacío.

## 7. Tiempo y latencia

- El reloj de la canción es el número de frames que el callback de salida de
  cpal ya entregó, dividido por su frecuencia (`crates/audio`).
- Un frame del micrófono con índice `k` tiene tiempo de canción
  `t = posicion_al_leer − (n_leidas − k) / fs_mic − latencia_ms / 1000`.
- La calibración mide exactamente esa ida y vuelta (pitido por la misma salida,
  detección en la misma entrada, misma fórmula), por eso corrige todo el
  camino: búfer de salida, altavoz, aire, micrófono, búfer de entrada.

## 8. Herramientas y entorno de la fábrica

- **Empaquetadas con la app** (sidecars de Tauri, junto al ejecutable):
  ffmpeg, deno (runtime de JavaScript que exige YouTube) y uv. Las baja
  `scripts/descargar-binarios.mjs` con versión fijada y SHA-256 verificado.
  Si no están, se buscan en el PATH (y en Homebrew en macOS).
- **Python de la fábrica**, en este orden: `KARAOKE_PYTHON` > `ajustes.carpeta_fabrica`
  > entorno instalado por la app (`<datos>/entorno/venv`, con `listo.json`)
  > repositorio de desarrollo.
- Los procesos de la fábrica reciben un PATH con la carpeta de las
  herramientas empaquetadas primero.
