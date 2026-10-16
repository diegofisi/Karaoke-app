import type { ResumenCancion } from "@/common/interfaces";
import { NOMBRE_IDIOMA } from "@/common/helpers/textos";
import { formatearPuntos, formatearTiempo } from "@/common/helpers/tiempo";
import type { FilaCancion } from "../interfaces/filaCancion";

export const aFilaCancion = (resumen: ResumenCancion): FilaCancion => ({
  id: resumen.id,
  titulo: resumen.titulo,
  artista: resumen.artista,
  idioma: resumen.idioma,
  nombreIdioma: NOMBRE_IDIOMA[resumen.idioma] ?? resumen.idioma,
  duracionTexto: formatearTiempo(resumen.duracion),
  recordTexto: resumen.record === null ? null : formatearPuntos(resumen.record),
  lineasRespaldo: resumen.lineas_respaldo,
  lineasTotal: resumen.lineas_total,
});

/** "1 línea aproximada" / "N líneas aproximadas". */
export const textoLineasAproximadas = (cantidad: number): string =>
  cantidad === 1 ? "1 línea aproximada" : `${cantidad} líneas aproximadas`;
