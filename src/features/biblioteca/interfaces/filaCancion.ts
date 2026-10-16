import type { Idioma } from "@/common/interfaces";

/** Lo que pinta una tarjeta de la biblioteca, ya formateado. */
export interface FilaCancion {
  id: string;
  titulo: string;
  artista: string;
  idioma: Idioma;
  nombreIdioma: string;
  duracionTexto: string;
  recordTexto: string | null;
  lineasRespaldo: number;
  lineasTotal: number;
}
