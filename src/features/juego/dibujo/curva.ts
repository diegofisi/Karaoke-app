import type { CurvaTono } from "@/common/interfaces";
import type { ColoresJuego } from "@/common/theme/tipos";
import type { VistaLienzo } from "./vista";

/** Curva de referencia `cancion.tono`, tenue, cortada donde no hay voz. */
export const dibujarCurvaReferencia = (
  ctx: CanvasRenderingContext2D,
  vista: VistaLienzo,
  curva: CurvaTono,
  colores: ColoresJuego,
): void => {
  if (!(curva.paso > 0) || curva.midi.length === 0) return;
  const primero = Math.max(0, Math.floor((vista.desde - curva.inicio) / curva.paso));
  const ultimo = Math.min(curva.midi.length - 1, Math.ceil((vista.hasta - curva.inicio) / curva.paso));
  ctx.strokeStyle = colores.curvaReferencia;
  ctx.lineWidth = 2;
  ctx.lineJoin = "round";
  ctx.beginPath();
  let enTrazo = false;
  for (let i = primero; i <= ultimo; i += 1) {
    const midi = curva.midi[i] ?? null;
    if (midi === null || midi < vista.rango.min || midi > vista.rango.max) {
      enTrazo = false;
      continue;
    }
    const x = vista.x(curva.inicio + i * curva.paso);
    const y = vista.y(midi);
    if (enTrazo) ctx.lineTo(x, y);
    else ctx.moveTo(x, y);
    enTrazo = true;
  }
  ctx.stroke();
};
