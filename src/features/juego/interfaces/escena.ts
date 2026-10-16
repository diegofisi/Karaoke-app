import type { CurvaTono } from "@/common/interfaces";
import type { RangoMidi } from "../helpers/geometria";
import type { NotaPlana } from "../helpers/notas";

/** Lo que el lienzo necesita de la canción, calculado una vez. */
export interface EscenaJuego {
  notas: NotaPlana[];
  duracionMaxNota: number;
  curva: CurvaTono;
  rango: RangoMidi;
}
