# Formularios — sin librería, con campos de MUI

> **Aplica:** siempre. No hay react-hook-form ni zod y no se añaden: los
> formularios de esta app tienen de uno a cuatro campos y una acción.

## Principios

1. **El componente de formulario recibe valores y callbacks por props** y no
   llama a Rust, no lee el store y no navega.
2. **El estado del formulario vive en el hook de feature o en la página**,
   que es quien llama a `mutate` o abre el canal.
3. **Una fuente para la forma**: el tipo del formulario deriva de un tipo que
   ya existe (los argumentos del comando).
4. **Validación en un helper puro** de `helpers/` que devuelve `string | null`
   (el mensaje en español, o `null`).
5. **Un envío**: el botón se deshabilita mientras hay un envío o un proceso
   en curso y cuando el formulario no es válido.
6. **Errores junto al campo** (`helperText` + `error` de `TextField`).
7. **Siembra con `key`**: un formulario que edita un dato cargado se remonta
   con `key={dato.id}`.

## Formularios previstos

| Entrada | Feature | Herramienta |
|---|---|---|
| URL de YouTube + idioma (`es`, `en`, `ja`) | `procesar` | `TextField` + `Select`; valida que sea una URL de YouTube antes de enviar |
| Archivo de audio local + título + artista | `procesar` | botón que abre el diálogo de archivo (plugin de Tauri, por decidir) + `TextField` |
| Letra aportada por el usuario (`.txt` o `.lrc`) | `procesar` | archivo, o `TextField multiline` que se guarda a un archivo para `--letra` (CONTRATO §4) |
| Dispositivos de entrada y salida, `latencia_ms` | `ajustes` | `Select`; la latencia la mide la calibración, con `Slider` para ajuste fino |

## La forma

```tsx
const [url, fijarUrl] = useState("");
const proceso = useProcesarCancion();
const problema = problemaDeUrl(url);

<TextField label="URL de YouTube" value={url} onChange={(evento) => fijarUrl(evento.target.value)}
  error={url.length > 0 && problema !== null} helperText={url.length > 0 ? problema : null} fullWidth />
<Button variant="contained" type="submit" disabled={problema !== null || proceso.enCurso}>
  {proceso.enCurso ? "Procesando…" : "Procesar"}
</Button>
```

Con más de un campo, `<Box component="form" onSubmit={…} noValidate>` con
`event.preventDefault()`. El botón de envío lleva `type="submit"`; los demás,
`type="button"`.

## Validación

- El helper devuelve el texto en español listo para pintar, o `null`.
- La interfaz solo impide enviar algo vacío o mal formado. Lo demás (si el
  vídeo existe, si hay letra) lo decide la fábrica y llega como `aviso` o
  `error` por el canal.

## Campos de MUI

| Campo | Valor | Error |
|---|---|---|
| `TextField` | `value` + `onChange` | `error` + `helperText` |
| `Select` | `value` + `onChange`; opciones con `MenuItem` | `FormControl error` |
| `Slider` | `value` + `onChange={(_, valor) => …}` | no aplica |
| `Switch` | `checked` + `onChange` | no aplica |

Valores de un `Select` que son enumeraciones se comparan contra el objeto
const; el cast de `evento.target.value` es el único permitido.

## Accesibilidad

- Todo campo tiene etiqueta (`label`, `aria-label`, `FormControlLabel`).
- El error se enlaza al campo con `helperText`.
- El botón deshabilitado durante el envío cambia su texto, no solo su color.

## Sí / No

| Sí | No |
|---|---|
| Estado en el hook; componente con `value` + `onChange` | Un componente que llama a `mutate` |
| Helper `problemaDe…(): string \| null` | Validación con ternarios en el JSX |
| `disabled` mientras el proceso sigue | Dos procesos de la misma canción |
