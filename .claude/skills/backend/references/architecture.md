# Arquitectura — crates de dominio / pegamento Tauri

**Aplica siempre.** Todas las demás referencias asumen estas reglas. Los
nombres reales de crates y módulos están en `project.md`.

## Regla de dependencia

```
src-tauri/   (pegamento: Tauri, estado, comandos, canales, lanza la fábrica, bucle del juego)
   │ usa
   ▼
crates/audio        crates/biblioteca      crates/puntuacion      crates/tono
 (cpal)              (std::fs, serde)       (puro)                 (puro)
                         │                      │
                         └──────────┬───────────┘
                                    ▼
                     crates/karaoke-core (tipos de cancion.json, serde)
```

Qué crate usa a `karaoke-core` lo fija `project.md`; el dibujo muestra los
que previsiblemente lo necesitan.

- **Los crates de dominio** no conocen `tauri`, no mandan nada a la interfaz y
  no leen el estado de `src-tauri`. Reciben datos o una ruta y devuelven
  `anyhow::Result<T>` o un valor.
- **`tono` y `puntuacion` son puros**: entran muestras o números, salen
  números. Sin `cpal`, sin archivos, sin hilos, sin reloj del sistema. Así se
  prueban con datos sintéticos.
- **`audio`** es el único crate que conoce `cpal`. Expone el reloj por
  muestras y los bloques del micrófono con su marca; no puntúa.
- **`biblioteca`** es el único crate que lee y escribe en la carpeta de
  canciones. No sabe dónde está esa carpeta: la recibe.
- **`karaoke-core`** posee los tipos de `cancion.json`. Lo usan los demás; él
  no usa a nadie del workspace.
- **`src-tauri`** conoce todo. Es el único que expone `#[tauri::command]`,
  guarda el estado, decide rutas (`app_data_dir`), abre `Channel`, lanza la
  fábrica y convierte errores a `String`. No contiene lógica de dominio.

Si un comando "necesita" lógica nueva, la lógica va en un crate y el comando
la llama. Si un crate "necesita" avisar a la interfaz, devuelve el dato o
recibe un callback, y `src-tauri` decide cómo mandarlo.

## Un crate de dominio

```
crates/{nombre}/
├── Cargo.toml        ← dependencias por `workspace = true`
├── src/lib.rs        ← API pública: tipos + funciones
├── src/{modulo}.rs   ← piezas con archivo propio
└── tests/*.rs        ← integración (carpetas temporales, fixtures)
```

La lógica que se prueba sin I/O lleva `#[cfg(test)] mod pruebas` al pie de su
módulo.

## Dónde va cada cosa

| Necesito… | Va en |
|---|---|
| Un tipo de `cancion.json` | `crates/karaoke-core` (+ espejo en `src/common/interfaces/` + `fabrica/karaoke_fabrica/modelo.py`) |
| Detectar el tono de un bloque de muestras | `crates/tono` |
| Puntuar una nota o una línea | `crates/puntuacion` |
| Abrir dispositivos, reproducir, capturar, medir latencia | `crates/audio` |
| Listar canciones, leer `cancion.json`, guardar récords | `crates/biblioteca` |
| Un tipo que solo existe en la frontera (frame del juego, argumentos) | `src-tauri/src/`, junto al comando o al bucle que lo usa |
| Dónde está la carpeta de datos | `src-tauri` (`app_data_dir`) |
| Lanzar la fábrica y leer su `stdout` | `src-tauri` |
| Convertir `anyhow::Error` a `String` | el comando, una vez (`errors.md`) |

## Nombres

| Cosa | Convención | Ejemplo |
|---|---|---|
| Comando Tauri | `snake_case`, en español, EXACTO al de `CONTRATO.md` y `comandos.ts` | `procesar_cancion` |
| Struct que cruza | `PascalCase` en español, campos `snake_case` en español | `Linea { inicio, fin, texto }` |
| Enum que cruza | representación serde explícita | `#[serde(rename_all = "snake_case")]` |
| Crate | `kebab-case` en el `Cargo.toml`, `snake_case` en el `use` | `karaoke-core` → `karaoke_core` |
| Mensaje de error | español, empieza por "No se pudo…" | `"No se pudo leer cancion.json de yt-abc"` |

Comentarios y nombres en **español**, sin tildes en los identificadores
(`latencia_ms`, `inicio`). Un comentario dice por qué, no qué. Sigue al vecino
en un archivo existente; no renombres de paso.
