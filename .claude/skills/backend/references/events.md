# Canales y eventos Rust → interfaz

**Aplica** en cuanto haya un comando que produce datos continuos: el progreso
de la fábrica y los frames del juego (CONTRATO §6). Hasta entonces, todo es
`invoke` con valor de retorno.

## Qué mecanismo

| Situación | Mecanismo |
|---|---|
| La interfaz pidió algo y el resultado llega en la misma llamada | Valor de retorno del comando |
| Un comando arrancó un trabajo y manda avance o datos a quien lo pidió (fábrica, juego) | **`tauri::ipc::Channel<T>`** pasado como argumento del comando |
| Algo pasó sin que nadie lo pidiera (un dispositivo de audio desapareció, otra ventana) | Evento (`app.emit`) |
| Un cambio que varias pantallas deben ver | Primero React Query invalidando la consulta; evento solo si no basta |

`Channel` es el mecanismo para flujos: va ordenado, va solo a quien lo pidió,
y Tauri lo documenta para mandar datos con frecuencia alta. `emit` pasa por el
sistema de eventos (todas las ventanas, serialización a JSON por evento) y no
se usa para frames.

## Forma

```rust
use tauri::ipc::Channel;

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FrameJuego { t: f64, midi: Option<f64>, claridad: f32, puntos: u32 }

// dentro del hilo del juego
if canal.send(frame).is_err() {
    break; // la ventana ya no escucha: el juego para
}
```

- **Payload = struct `Serialize`** con campos `snake_case` en español,
  documentado en `CONTRATO.md` §6 y espejado en `src/common/interfaces/` en el
  mismo cambio. Nunca una tupla ni un valor suelto.
- **Mensajes de la fábrica**: los objetos de CONTRATO §4 tal cual (`tipo`:
  `etapa`, `aviso`, `listo`, `error`). Rust los parsea para validar, y los
  reenvía con la misma forma: un `enum` con `#[serde(tag = "tipo",
  rename_all = "snake_case")]`.
- **El `Result` de `send`**: un fallo significa que la interfaz ya no escucha
  (ventana cerrada, pantalla desmontada). Para el trabajo o ignóralo con un
  comentario; nunca `unwrap`.
- Un canal se cierra cuando el hilo suelta el `Channel`. Antes de soltarlo, el
  último mensaje dice cómo terminó (`listo`, `error`, `fin`).

## Frecuencia

| Flujo | Frecuencia | Por qué |
|---|---|---|
| Frames del juego | uno cada ~16 ms (~60 Hz) | La interfaz pinta a la frecuencia de la pantalla e interpola `t` con `performance.now()` entre frames (ARQUITECTURA §4) |
| Cálculo de tono y puntuación | cada 10 ms de audio | Va en Rust, no depende de la pantalla. Un frame de pantalla puede llevar el último valor o un resumen de varios |
| Progreso de la fábrica | al empezar cada etapa y al avanzar | Lo decide la fábrica; Rust no inventa avance |

Nunca un mensaje por bloque de audio ni por muestra. Un frame es pequeño:
números, no listas que crecen.

## Quién manda

Siempre `src-tauri`. Los crates de dominio no conocen `tauri`: si una
operación larga del crate necesita informar, recibe un callback
`&mut dyn FnMut(Avance)` o devuelve un iterador, y `src-tauri` manda.

## Eventos (`emit`)

Si alguna vez hace falta uno: nombre en `kebab-case` en español
(`dispositivo-perdido`), payload struct `Serialize`, documentado en
`CONTRATO.md`, escuchado desde un único archivo del frontend. El `Result` de
`emit` se ignora con `let _ =` y comentario.
