# Correcciones

Memoria entre sesiones. Una conversación termina y se lleva sus lecciones; este
archivo no.

**La regla:** cada vez que el usuario tiene que corregir lo mismo dos veces, se
convierte en una línea fechada aquí. Ni un párrafo ni un documento: una línea,
así:

```
- AAAA-MM-DD — qué salió mal · por qué importa · qué hacer en su lugar
```

Detalles que sostienen el mecanismo:

- `CLAUDE.md` apunta a este archivo, así que se lee al empezar cada sesión.
  Mantenlo corto: cuando pase de ~40 líneas, promueve las entradas estables a
  la regla de `.claude/rules/*.md` que toque o al `project-pitfalls.md` de la
  skill, y bórralas de aquí.
- Una línea se queda hasta que sea falsa. No "limpies" entradas viejas porque
  parezcan obvias: parecen obvias porque están escritas.
- Lo que describe *cómo funciona el proyecto* va en `project.md`. Lo que
  describe *un error ya pagado* va aquí o en `project-pitfalls.md`. La
  diferencia: una corrección tiene fecha y víctima.
- Lo que un script pueda comprobar debe ser un hook o un paso de `verify`, no
  una línea aquí. Una línea aquí es el recurso para lo que solo atrapa el
  criterio.

Las entradas de antes del 2026-10-15 vienen de `InglesApp/.claude/corrections.md`
y son sobre el usuario y esta máquina, no sobre aquel proyecto.

## Entradas

- 2026-08-23 — Se usaron metáforas y modismos al explicar · el usuario lee de
  forma literal y una figura retórica le llega como un dato falso o como una
  instrucción · lenguaje directo siempre: di la cosa, no su imagen; una idea
  por frase.
- 2026-09-02 — Scripts escritos con heredocs de Bash de más de ~2 KB llegaban
  cortados ("unexpected EOF") · en esta máquina la capa de shell trunca el
  heredoc largo y el archivo queda a medias sin aviso · escribe archivos con la
  herramienta Write; `sed` y heredocs cortos sí funcionan; la shell es para
  ejecutar, no para redactar.
- 2026-09-17 — Los commits llevaban `Co-Authored-By: Claude` / "Generated with
  Claude" · el usuario no quiere atribución a IA en el historial, y pedirlo en
  prosa se olvida cada sesión · lo aplica `settings.json`
  (`includeCoAuthoredBy: false`, `attribution` vacío) y `~/.claude/CLAUDE.md`;
  nunca escribas un trailer a mano, y tras committear comprueba
  `git log -1 --format=%B` y haz `amend` si se coló uno.
- 2026-10-04 — `Move-Item` de una carpeta con `.git` falló con "permisos
  insuficientes" · un archivo de `.git\` estaba bloqueado por otro programa y
  `Move-Item` aborta a medias · usa `robocopy /E /MOVE origen destino`; termina
  con 0 errores y deja el origen eliminado.

<!-- Entradas nuevas debajo de esta línea. -->
- 2026-10-15 — Editar archivos con `pathlib.write_text` de Python en Windows los
  guarda con CRLF y el diff parece reescribir el archivo entero · el revisor
  rechazó el commit por eso · edita con la herramienta Edit/Write; si hace
  falta un script, usa `write_bytes` con `\n`; `.gitattributes` ya fuerza LF.
