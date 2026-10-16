import { Calificacion, TipoNota, type Calificacion as TipoCalificacion } from "@/common/interfaces";
import type { ColoresJuego } from "@/common/theme/tipos";
import { primeroDesde } from "../helpers/notas";
import type { EscenaJuego } from "../interfaces/escena";
import type { VistaLienzo } from "./vista";

const ALTO_MINIMO_PX = 6;
const ALTO_MAXIMO_PX = 22;
const BRILLO_DORADA_PX = 14;

export const colorDeCalificacion = (calificacion: TipoCalificacion, colores: ColoresJuego): string => {
  switch (calificacion) {
    case Calificacion.Perfecta:
      return colores.perfecta;
    case Calificacion.Bien:
      return colores.bien;
    case Calificacion.Regular:
      return colores.regular;
    case Calificacion.Fallo:
      return colores.fallo;
  }
};

/**
 * Notas como barras redondeadas en su tono. Las doradas brillan; las libres
 * son tenues; las cerradas toman el color de su calificación; la que suena
 * ahora se resalta.
 */
export const dibujarNotas = (
  ctx: CanvasRenderingContext2D,
  vista: VistaLienzo,
  escena: EscenaJuego,
  calificaciones: ReadonlyMap<number, TipoCalificacion>,
  colores: ColoresJuego,
): void => {
  const alto = Math.min(ALTO_MAXIMO_PX, Math.max(ALTO_MINIMO_PX, vista.semitono * 0.8));
  const radio = alto / 2;
  const notas = escena.notas;
  for (let i = primeroDesde(notas, vista.desde - escena.duracionMaxNota); i < notas.length; i += 1) {
    const nota = notas[i];
    if (nota === undefined || nota.inicio > vista.hasta) break;
    if (nota.fin < vista.desde) continue;

    const x0 = vista.x(nota.inicio);
    const ancho = Math.max(alto, vista.x(nota.fin) - x0);
    const y = vista.y(nota.midi) - radio;
    const calificacion = calificaciones.get(nota.indice);
    const activa = nota.inicio <= vista.tAhora && vista.tAhora <= nota.fin;
    const dorada = nota.tipo === TipoNota.Dorada;

    let relleno = colores.nota;
    if (nota.tipo === TipoNota.Libre) relleno = colores.notaLibre;
    else if (calificacion !== undefined) relleno = colorDeCalificacion(calificacion, colores);
    else if (dorada) relleno = colores.notaDorada;
    else if (activa) relleno = colores.notaActiva;

    ctx.save();
    if (dorada) {
      ctx.shadowColor = colores.notaDoradaBorde;
      ctx.shadowBlur = BRILLO_DORADA_PX;
    }
    ctx.fillStyle = relleno;
    ctx.beginPath();
    ctx.roundRect(x0, y, ancho, alto, radio);
    ctx.fill();
    ctx.restore();

    if (dorada || activa) {
      ctx.strokeStyle = dorada ? colores.notaDoradaBorde : colores.notaBorde;
      ctx.lineWidth = activa ? 2.5 : 1.5;
      ctx.beginPath();
      ctx.roundRect(x0, y, ancho, alto, radio);
      ctx.stroke();
    }
  }
};
