import { Nivel, type Calificacion, type Nivel as TipoNivel } from "@/common/interfaces";

export const NOMBRE_NIVEL: Record<TipoNivel, string> = {
  [Nivel.Desafinado]: "Desafinado",
  [Nivel.Aficionado]: "Aficionado",
  [Nivel.Prometedor]: "Prometedor",
  [Nivel.Cantante]: "Cantante",
  [Nivel.Estrella]: "Estrella",
  [Nivel.Leyenda]: "Leyenda",
};

export const NOMBRE_CALIFICACION: Record<Calificacion, string> = {
  perfecta: "Perfectas",
  bien: "Bien",
  regular: "Regulares",
  fallo: "Fallos",
};

/** Avance suavizado (ease-out cúbico) de 0 a 1. */
export const suavizar = (x: number): number => 1 - (1 - Math.min(1, Math.max(0, x))) ** 3;
