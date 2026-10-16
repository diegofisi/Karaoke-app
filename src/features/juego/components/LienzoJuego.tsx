import { useEffect, useRef } from "react";
import Box from "@mui/material/Box";
import { useTheme } from "@mui/material/styles";
import { dibujarFrame } from "../dibujo/frame";
import type { EscenaJuego } from "../interfaces/escena";
import { tiempoDeDibujo, type FuenteFrames } from "../stores/fuenteFrames";

export interface LienzoJuegoProps {
  escena: EscenaJuego;
  fuente: FuenteFrames;
}

/**
 * Carril de notas. Un solo `<canvas>` con su bucle de requestAnimationFrame,
 * montado una vez: lee la fuente de frames dentro de cada frame de dibujo y
 * nunca toca el estado de React.
 */
export const LienzoJuego = ({ escena, fuente }: LienzoJuegoProps) => {
  const lienzo = useRef<HTMLCanvasElement | null>(null);
  const colores = useTheme().palette.juego;

  useEffect(() => {
    const canvas = lienzo.current;
    const ctx = canvas?.getContext("2d") ?? null;
    if (canvas === null || ctx === null) return;

    let ancho = 0;
    let alto = 0;
    // El tamaño en píxeles reales solo se toca al montar y al cambiar de tamaño.
    const medir = () => {
      const escala = window.devicePixelRatio || 1;
      ancho = canvas.clientWidth;
      alto = canvas.clientHeight;
      canvas.width = Math.max(1, Math.round(ancho * escala));
      canvas.height = Math.max(1, Math.round(alto * escala));
      ctx.setTransform(escala, 0, 0, escala, 0, 0);
    };
    medir();
    const observador = new ResizeObserver(medir);
    observador.observe(canvas);

    let id = 0;
    const pintar = (ahora: number) => {
      dibujarFrame(
        ctx,
        ancho,
        alto,
        escena,
        { tAhora: tiempoDeDibujo(fuente, ahora), rastro: fuente.rastro, calificaciones: fuente.calificaciones },
        colores,
      );
      id = requestAnimationFrame(pintar);
    };
    id = requestAnimationFrame(pintar);

    return () => {
      cancelAnimationFrame(id);
      observador.disconnect();
    };
  }, [escena, fuente, colores]);

  return (
    <Box
      component="canvas"
      ref={lienzo}
      role="img"
      aria-label="Carril de notas: las barras son las notas de la canción y la línea brillante es tu voz."
      sx={{ display: "block", width: "100%", height: "100%" }}
    />
  );
};
