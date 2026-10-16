// Notas planas y curva de referencia. Funciones puras.

import type { Cancion, CurvaTono, Linea, TipoNota } from "@/common/interfaces";

/** Una nota con su posición en la canción. `indice` = el de `nota_cerrada` (CONTRATO.md §6). */
export interface NotaPlana {
  indice: number;
  indice_linea: number;
  inicio: number;
  fin: number;
  midi: number;
  tipo: TipoNota;
}

/**
 * Todas las notas ordenadas por inicio, con el mismo orden que
 * `Cancion::notas_planas` de crates/karaoke-core (orden estable: a igual
 * inicio se conserva el del archivo). Así `indice` casa con el de los eventos.
 */
export const aplanarNotas = (cancion: Cancion): NotaPlana[] => {
  const notas = cancion.lineas.flatMap((linea, indiceLinea) =>
    linea.palabras.flatMap((palabra) =>
      palabra.notas.map((nota) => ({ indice_linea: indiceLinea, inicio: nota.inicio, fin: nota.fin, midi: nota.midi, tipo: nota.tipo })),
    ),
  );
  notas.sort((a, b) => a.inicio - b.inicio);
  return notas.map((nota, indice) => ({ ...nota, indice }));
};

/** Primer índice `i` con `inicio >= t` (búsqueda binaria; `elementos` ordenado por inicio). */
export const primeroDesde = (elementos: readonly { inicio: number }[], t: number): number => {
  let bajo = 0;
  let alto = elementos.length;
  while (bajo < alto) {
    const medio = (bajo + alto) >>> 1;
    const elemento = elementos[medio];
    if (elemento !== undefined && elemento.inicio < t) bajo = medio + 1;
    else alto = medio;
  }
  return bajo;
};

/** La nota que suena en `t`, o `null`. */
export const notaEn = (notas: readonly NotaPlana[], t: number): NotaPlana | null => {
  const candidata = notas[primeroDesde(notas, t + 1e-9) - 1];
  return candidata !== undefined && t <= candidata.fin ? candidata : null;
};

/** Tono de referencia en `t`: el frame más cercano. Mismo cálculo que `CurvaTono::en`. */
export const curvaEn = (curva: CurvaTono, t: number): number | null => {
  if (!(Number.isFinite(curva.paso) && curva.paso > 0)) return null;
  const posicion = Math.round((t - curva.inicio) / curva.paso);
  if (!Number.isFinite(posicion) || posicion < 0 || posicion >= curva.midi.length) return null;
  return curva.midi[posicion] ?? null;
};

/** La duración de la nota más larga, para saber desde dónde buscar las visibles. */
export const duracionMaxima = (notas: readonly NotaPlana[]): number =>
  notas.reduce((maxima, nota) => Math.max(maxima, nota.fin - nota.inicio), 0);

/** Índice de la línea que se canta ahora o, entre líneas, de la siguiente. `lineas.length` si ya no quedan. */
export const indiceLineaActual = (lineas: readonly Linea[], t: number): number => {
  const indice = lineas.findIndex((linea) => linea.fin >= t);
  return indice === -1 ? lineas.length : indice;
};

/** Avance del barrido de una palabra, de 0 a 1. */
export const progresoPalabra = (inicio: number, fin: number, t: number): number => {
  if (t <= inicio) return 0;
  if (t >= fin || fin <= inicio) return 1;
  return (t - inicio) / (fin - inicio);
};
