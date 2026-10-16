import type { ColoresJuego } from "@/common/theme/tipos";
import { FRACCION_AHORA } from "../helpers/geometria";
import type { VistaLienzo } from "./vista";

/** Fondo con una línea por semitono; las de do (múltiplo de 12) más marcadas. */
export const dibujarFondo = (ctx: CanvasRenderingContext2D, vista: VistaLienzo, colores: ColoresJuego): void => {
  ctx.fillStyle = colores.fondo;
  ctx.fillRect(0, 0, vista.ancho, vista.alto);
  ctx.lineWidth = 1;
  for (let midi = Math.ceil(vista.rango.min); midi <= Math.floor(vista.rango.max); midi += 1) {
    const y = Math.round(vista.y(midi)) + 0.5;
    ctx.strokeStyle = midi % 12 === 0 ? colores.lineaOctava : colores.lineaSemitono;
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(vista.ancho, y);
    ctx.stroke();
  }
};

/** Línea vertical de "ahora", al 25 % del ancho. */
export const dibujarLineaAhora = (ctx: CanvasRenderingContext2D, vista: VistaLienzo, colores: ColoresJuego): void => {
  const x = Math.round(vista.ancho * FRACCION_AHORA) + 0.5;
  ctx.strokeStyle = colores.lineaAhora;
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(x, 0);
  ctx.lineTo(x, vista.alto);
  ctx.stroke();
};
