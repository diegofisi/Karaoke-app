import type { Calificacion, TonoMicrofono } from "@/common/interfaces";
import type { ColoresJuego } from "@/common/theme/tipos";
import type { EscenaJuego } from "../interfaces/escena";
import { dibujarCurvaReferencia } from "./curva";
import { dibujarFondo, dibujarLineaAhora } from "./fondo";
import { dibujarNotas } from "./notas";
import { dibujarRastro } from "./rastro";
import { crearVista } from "./vista";

export interface DatosFrame {
  tAhora: number;
  rastro: readonly TonoMicrofono[];
  calificaciones: ReadonlyMap<number, Calificacion>;
}

/** Pinta un frame completo del carril de notas. Medidas en píxeles CSS. */
export const dibujarFrame = (
  ctx: CanvasRenderingContext2D,
  ancho: number,
  alto: number,
  escena: EscenaJuego,
  datos: DatosFrame,
  colores: ColoresJuego,
): void => {
  if (ancho <= 0 || alto <= 0) return;
  const vista = crearVista(ancho, alto, datos.tAhora, escena.rango);
  dibujarFondo(ctx, vista, colores);
  dibujarCurvaReferencia(ctx, vista, escena.curva, colores);
  dibujarNotas(ctx, vista, escena, datos.calificaciones, colores);
  dibujarLineaAhora(ctx, vista, colores);
  dibujarRastro(ctx, vista, datos.rastro, escena, colores);
};
