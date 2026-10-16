# Errores

**Aplica siempre.** Dos capas, dos tipos: `anyhow::Error` dentro de los
crates, `String` en español en la frontera con la interfaz. Sin `thiserror`
salvo que un crate necesite que el llamador distinga casos (y entonces se
escribe en `project.md`).

## Dentro de los crates: `anyhow` con contexto legible

```rust
pub fn leer_cancion(carpeta: &Path, id: &str) -> Result<Cancion> {
    let ruta = carpeta.join(id).join("cancion.json");
    let texto = fs::read_to_string(&ruta)
        .with_context(|| format!("No se pudo leer la canción {id}: falta cancion.json"))?;
    serde_json::from_str(&texto)
        .with_context(|| format!("No se pudo leer la canción {id}: cancion.json no es válido"))
}
```

- Toda función pública que puede fallar devuelve `anyhow::Result<T>`.
- El `with_context` es **la frase que verá el usuario**, en español,
  empezando por "No se pudo…" (CONTRATO §1). La causa técnica queda detrás.
- Pon contexto donde el fallo depende de la entrada (un id, una ruta, un
  dispositivo). Un `?` sin contexto vale cuando el fallo es un bug interno.
- `anyhow::bail!` y `anyhow::ensure!` para validaciones (orden de `lineas`,
  `version` de `cancion.json` desconocida).

## En la frontera: un `String`, una sola vez

```rust
biblioteca::leer_cancion(&carpeta, &id).map_err(|e| e.to_string())
```

`anyhow::Error::to_string()` devuelve **solo el contexto más externo**. Por eso
el contexto tiene que ser una frase completa. La cadena entera
(`format!("{e:#}")`) va al registro, no a la interfaz.

## Errores que viajan por un canal

Un fallo durante un trabajo largo (fábrica, juego) no es el `Err` del comando:
el comando ya devolvió. Viaja como un mensaje del canal con `tipo: "error"` y
`mensaje` en español (CONTRATO §4 para la fábrica). Después de ese mensaje, el
hilo termina; no manda nada más por ese canal.

## Errores dentro del hilo de audio

El callback de `cpal` no puede devolver error ni reservar memoria para
formatear uno. Si algo falla ahí (bloque vacío, desbordamiento del buffer), se
cuenta en un contador atómico y el hilo del juego lo lee y lo informa fuera
del callback. El error de stream de `cpal` (dispositivo desconectado) llega
por su callback de error: se marca una bandera atómica y el juego para con
"No se pudo seguir: se desconectó el dispositivo de audio".

## `Option` no es error

- Un frame sin voz tiene tono `None` (`null`): estado normal.
- Una palabra con `notas` vacía se canta pero no puntúa tono: estado normal.
- Un id de canción sin `cancion.json` es "la canción no está lista": decide
  en la API si es `Option` o error y escríbelo en el doc-comment.

## Lo que no se hace

- `unwrap()` / `expect()` fuera de tests y del `expect` único que cierra
  `tauri::Builder::run(...)` (es el arranque; no hay a quién devolver el
  error).
- `Result<T, anyhow::Error>` en la firma de un comando: no es `Serialize`; no
  compila.
- Mensajes con jerga sin la parte humana delante.
- Traducir el mismo error en dos capas.
- Silenciar con `.ok()` o `let _ =` sin comentario. Excepción documentada: el
  `Result` de `channel.send(...)` cuando la ventana ya se cerró.
