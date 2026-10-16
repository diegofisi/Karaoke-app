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

## Requisitos

- Windows 10/11 o macOS 14+ (Apple Silicon).
- GPU recomendada: NVIDIA con CUDA en Windows; la GPU de Apple (MPS) en Mac.
- Python 3.12, Node.js 20+, Rust 1.85+, `ffmpeg`.
- Unos 7 GB para los modelos (se descargan la primera vez).

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

- El ejecutable usa la fábrica Python de esta carpeta (`fabrica/.venv`), así
  que hoy funciona en el equipo donde se instaló el entorno. Empaquetar Python
  dentro de la app es la fase 5 (`docs/ARQUITECTURA.md`).
- Uso personal: algunos modelos tienen licencia no comercial (registro en
  `.claude/skills/fabrica/references/project.md`). El texto de las letras
  tiene copyright de sus autores.
- La versión anterior (Python + navegador) está en la rama `legado-python`.
