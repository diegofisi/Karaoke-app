import type { TonoMicrofono } from "@/common/interfaces";
import type { ColoresJuego } from "@/common/theme/tipos";
import { limitarAlRango } from "../helpers/geometria";
import { curvaEn, notaEn } from "../helpers/notas";
import { ajustarOctava } from "../helpers/octava";
import type { EscenaJuego } from "../interfaces/escena";
import type { VistaLienzo } from "./vista";

/** Más de este hueco entre dos frames del micrófono corta el trazo. */
const HUECO_MAXIMO_S = 0.05;
/** El punto del tono actual solo se pinta si el último frame es así de reciente. */
const RECIENTE_S = 0.15;
const RADIO_PUNTO_PX = 6;

/** Tono del usuario para dibujar: transportado a la octava de la nota (o de la guía) en ese instante. */
const tonoDibujable = (tono: TonoMicrofono, escena: EscenaJuego): number | null => {
  if (tono.midi === null || !Number.isFinite(tono.midi)) return null;
  const referencia = notaEn(escena.notas, tono.t)?.midi ?? curvaEn(escena.curva, tono.t);
  const ajustado = referencia === null ? tono.midi : ajustarOctava(tono.midi, referencia);
  return limitarAlRango(ajustado, escena.rango);
};

/** Rastro del micrófono hasta la línea de "ahora", con un punto en el tono actual. */
export const dibujarRastro = (
  ctx: CanvasRenderingContext2D,
  vista: VistaLienzo,
  rastro: readonly TonoMicrofono[],
  escena: EscenaJuego,
  colores: ColoresJuego,
): void => {
  if (rastro.length === 0) return;
  ctx.save();
  ctx.strokeStyle = colores.rastro;
  ctx.shadowColor = colores.rastroSombra;
  ctx.shadowBlur = 8;
  ctx.lineWidth = 3;
  ctx.lineJoin = "round";
  ctx.lineCap = "round";
  ctx.beginPath();
  let anterior: TonoMicrofono | null = null;
  let ultimoY: number | null = null;
  let ultimoT = -Infinity;
  for (const tono of rastro) {
    if (tono.t < vista.desde || tono.t > vista.tAhora + HUECO_MAXIMO_S) {
      anterior = null;
      continue;
    }
    const midi = tonoDibujable(tono, escena);
    if (midi === null) {
      anterior = null;
      ultimoY = null;
      continue;
    }
    const x = vista.x(tono.t);
    const y = vista.y(midi);
    const continua = anterior !== null && tono.t - anterior.t <= HUECO_MAXIMO_S;
    if (continua) ctx.lineTo(x, y);
    else ctx.moveTo(x, y);
    anterior = tono;
    ultimoY = y;
    ultimoT = tono.t;
  }
  ctx.stroke();

  if (ultimoY !== null && vista.tAhora - ultimoT <= RECIENTE_S) {
    ctx.fillStyle = colores.rastro;
    ctx.beginPath();
    ctx.arc(vista.x(ultimoT), ultimoY, RADIO_PUNTO_PX, 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.restore();
};
