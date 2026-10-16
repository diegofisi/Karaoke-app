# Trampas universales — React + MUI + React Query + Tauri + canvas

> **Aplica:** siempre. Léelo antes de terminar cualquier cambio, junto con
> `project-pitfalls.md`. Lo específico de este repo va allí.

## Fronteras

1. **Un componente no llama a hooks de datos ni lee el store.** Es un
   container en la carpeta equivocada.
2. **Sin imports entre features.** Lo que comparten dos features sube a
   `common/`.
3. **`invoke` y `new Channel` solo en `comandos.ts`; `useQuery` /
   `useMutation` solo en `consultas.ts`.**
4. **Un container con `useMemo`, `useEffect` o un handler de varios pasos**
   es un hook sin extraer.
5. **No inventes campos del backend.** El tipo de `common/interfaces/` se
   copia del `struct` y de CONTRATO §3, no se completa.

## Tauri

6. **El nombre del comando es literal y `snake_case`.** `tsc` no lo
   comprueba.
7. **`rename_all = "snake_case"` manda sobre los argumentos**, también el
   nombre del canal.
8. **`undefined` no se serializa.** Los `Option` van como `null`.
9. **`Err(String)` llega como `string`, no como `Error`.**
10. **El navegador no tiene Tauri.** Que la pantalla no se rompa en
    `npm run dev` no verifica el transporte.
11. **`onmessage` se asigna antes del `invoke`**, o se pierden los primeros
    mensajes.

## Tiempo real y canvas

12. **`setState` por frame** re-renderiza el árbol 60 veces por segundo y
    produce tirones. Frames a un ref o a un store leído con `getState()`.
13. **Un `useCallback` o un efecto que captura datos que cambian** los ve
    congelados en el valor del render en que se creó. Lo que cambia mientras
    el bucle vive se lee de un ref o de `getState()` dentro del bucle.
14. **Un reloj propio en la interfaz** se separa del audio. `t` viene de Rust
    y se interpola con `performance.now()`.
15. **Puntuar o contar algo por `requestAnimationFrame`** depende de la
    frecuencia de la pantalla. La interfaz no puntúa.
16. **Un bucle de `requestAnimationFrame` sin `cancelAnimationFrame`** sigue
    vivo al cambiar de pantalla; con `StrictMode` se duplica.
17. **Cambiar `canvas.width` en cada frame** borra el canvas y reserva
    memoria. Solo al montar y al cambiar de tamaño.
18. **El canvas sin `devicePixelRatio`** se ve borroso en pantallas con
    escala.

## React Query

19. **Una consulta con argumento vacío se apaga con `enabled`.**
20. **La clave incluye todos los argumentos**, y un opcional va como `null`.
21. **Una mutación invalida todas las claves que cambia en disco.**
22. **`mutate` + callbacks, nunca `mutateAsync` + `try/catch`.**
23. **`data` sobrevive a un refetch fallido.** No lo sustituyas por `[]`.
24. **No hay `onSuccess` en `useQuery` (v5).**
25. **`isLoading` es solo la primera carga.**

## Stores

26. **Selecciona estrecho.** Un componente suscrito al store entero
    re-renderiza con cualquier cambio.
27. **Un store no importa React.** Fuera de React, `getState()` /
    `setState()`.
28. **Un `string` abierto de Rust no se castea a una unión.** Se valida.
29. **Un store singleton guarda estado entre tests y entre partidas.**
    `reset()`.

## Efectos y render

30. **El estado derivado no va en un efecto.**
31. **Reiniciar un hijo = `key`.**
32. **Un efecto con dependencia inestable corre en cada render.**
33. **`setState` con el valor anterior usa actualizador funcional.**
34. **`key` es identidad, no índice.**

## MUI y estilo

35. **Ningún `#RRGGBB` ni `alpha()` en un componente ni en el código de
    dibujo.** Un tono nuevo entra en la paleta primero.
36. **`Typography` con `fontSize` a mano es una variante que falta.**
37. **Un `IconButton` deshabilitado no recibe eventos de ratón**: el
    `Tooltip` necesita un `<Box component="span">`.
38. **Importa MUI por ruta** (`@mui/material/Stack`).

## Texto y accesibilidad

39. **Texto visible en español**, incluido `placeholder`, `aria-label`,
    `Tooltip` y mensajes de estado.
40. **El color solo no comunica nada.** Texto o icono además del color.
41. **Interactivo = `Button` / `IconButton`.** Nunca `<Box onClick>`.

## Proceso

42. **La verificación es un comando, no una impresión.** `tsc -b` y
    `vite build` antes de decir "hecho".
43. **`tsc -b --noEmit` no es `tsc -b`.** Con proyectos referenciados
    `composite`, `--noEmit` falla con TS6310. Usa el comando de `project.md`.
44. **Un comentario que cita una sección del contrato la cita por número.**
45. **Una corrección repetida es una línea en `project-pitfalls.md`.**
46. **El usuario prueba en `tauri dev`; el agente no lo lanza.** Tampoco
    `vite`.
