# Karaoke

Karaoke de escritorio para Windows y macOS. Toma una canción de YouTube (o un
archivo de audio), separa la voz del instrumental, coloca la letra palabra por
palabra sobre la voz original y puntúa tu canto en tiempo real.

## Qué hace

- **Procesar** una URL de YouTube o un archivo local: descarga, separa la voz
  (Mel-RoFormer), busca la letra en LRCLIB (o usa la tuya, o la transcribe), la
  alinea palabra por palabra con la voz del cantante (Qwen3-ForcedAligner, con
  MMS como segunda opinión) y saca el tono de referencia (SwiftF0).
- **Cantar** en modo práctica o puntuación: notas en pantalla, tu tono en
  directo, letra que se colorea al ritmo del cantante, furigana y romaji en
  japonés. En práctica puedes poner o quitar la voz original al vuelo.
- **Grabar** tu voz mezclada con la canción: MP3 en `Música/Karaoke`.
- **Calibrar** la latencia del micrófono con pitidos.

Idiomas de la letra: español, inglés y japonés.

## Arquitectura

| Parte | Lenguaje | Carpeta |
|---|---|---|
| Fábrica (prepara cada canción con modelos de IA, antes de jugar) | Python | `fabrica/` |
| Juego en tiempo real (audio, micrófono, tono, puntuación) | Rust | `crates/`, `src-tauri/` |
| Interfaz | React + TypeScript (Tauri 2) | `src/` |
| Medida de la alineación | Python | `eval/` |

Detalle en `docs/ARQUITECTURA.md`; el contrato entre las partes en
`docs/CONTRATO.md`; el estado y las medidas en `docs/ESTADO.md`.

Precisión medida del tiempo por palabra (JamendoLyrics, canciones no usadas al
ajustar): el 81,7 % de las palabras empieza a 100 ms o menos de la voz real.

## Instalar (usuario)

Descarga el instalador de la última versión en
[Releases](https://github.com/diegofisi/Karaoke-app/releases): el `.exe` de
Windows o el `.dmg` de Mac (Apple Silicon). **No hace falta instalar nada más**:

- ffmpeg, Deno y uv van dentro de la app.
- La primera vez, la app comprueba lo necesario y ofrece **«Instalar la
  fábrica»**: descarga sola Python 3.12, PyTorch (con CUDA si hay NVIDIA; MPS
  en Mac) y los modelos de IA, unos 13 GB en total.
- Requisitos del equipo: Windows 10/11 o macOS 14+ con Apple Silicon. Una GPU
  NVIDIA (o un Mac M1-M4) hace que procesar una canción tarde 1-3 minutos; sin
  GPU funciona, pero mucho más lento.

## Instalar y ejecutar (desarrollo)

1. Entorno de la fábrica: ver `fabrica/README.md` (CUDA en Windows, MPS en Mac).
2. Interfaz y app:

```bash
npm install
npm run tauri dev
```

Comprobar todo (tipos, build, clippy, tests de Rust y de la fábrica):

```bash
npm run verify
```

Generar el ejecutable y el instalador de Windows: `npm run tauri build`
(en macOS: `npm run tauri build -- --bundles app`).

## Limitaciones

- Uso personal: algunos modelos tienen licencia no comercial (registro en
  `.claude/skills/fabrica/references/project.md`). El texto de las letras
  tiene copyright de sus autores.
- La versión anterior (Python + navegador) está en la rama `legado-python`.
