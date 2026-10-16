// Geometría del carril de notas: tiempo en x, tono en y. Funciones puras.

/** Segundos visibles por delante de la línea de "ahora". */
export const VENTANA_ADELANTE_S = 4;
/** Posición de la línea de "ahora", como fracción del ancho. */
export const FRACCION_AHORA = 0.25;
/** Semitonos de margen por encima y por debajo de las notas de la canción. */
export const MARGEN_SEMITONOS = 3;
/** Margen vertical del lienzo en píxeles. */
export const MARGEN_VERTICAL_PX = 16;

export interface RangoMidi {
  min: number;
  max: number;
}

export const pixelesPorSegundo = (ancho: number): number => (ancho * (1 - FRACCION_AHORA)) / VENTANA_ADELANTE_S;

/** x en píxeles del instante `t` cuando la canción está en `tAhora`. */
export const tiempoAX = (t: number, tAhora: number, ancho: number): number =>
  ancho * FRACCION_AHORA + (t - tAhora) * pixelesPorSegundo(ancho);

/** Intervalo de tiempo que cabe en el lienzo. */
export const ventanaVisible = (tAhora: number, ancho: number): { desde: number; hasta: number } => {
  const pps = pixelesPorSegundo(ancho);
  if (pps <= 0) return { desde: tAhora, hasta: tAhora };
  return { desde: tAhora - (ancho * FRACCION_AHORA) / pps, hasta: tAhora + VENTANA_ADELANTE_S };
};

/** y en píxeles del tono `midi` (más agudo arriba). */
export const midiAY = (midi: number, rango: RangoMidi, alto: number): number => {
  const util = Math.max(1, alto - 2 * MARGEN_VERTICAL_PX);
  const span = Math.max(1, rango.max - rango.min);
  return MARGEN_VERTICAL_PX + ((rango.max - midi) / span) * util;
};

/** Alto en píxeles de un semitono. */
export const altoSemitono = (rango: RangoMidi, alto: number): number =>
  Math.max(1, alto - 2 * MARGEN_VERTICAL_PX) / Math.max(1, rango.max - rango.min);

/** Limita un tono al rango dibujable. */
export const limitarAlRango = (midi: number, rango: RangoMidi): number =>
  Math.min(rango.max, Math.max(rango.min, midi));

const RANGO_POR_DEFECTO: RangoMidi = { min: 55, max: 79 };
const SPAN_MINIMO = 12;

/**
 * Rango vertical: de la nota más grave a la más aguda, ±3 semitonos; si no hay
 * notas, el de la curva de referencia; si tampoco, un rango de voz media.
 * Nunca menos de una octava.
 */
export const rangoMidiDe = (tonosNotas: number[], tonosCurva: (number | null)[]): RangoMidi => {
  let min = Infinity;
  let max = -Infinity;
  const considerar = (midi: number | null) => {
    if (midi === null || !Number.isFinite(midi)) return;
    if (midi < min) min = midi;
    if (midi > max) max = midi;
  };
  tonosNotas.forEach(considerar);
  if (!Number.isFinite(min)) tonosCurva.forEach(considerar);
  if (!Number.isFinite(min) || !Number.isFinite(max)) return RANGO_POR_DEFECTO;
  let rango = { min: Math.floor(min) - MARGEN_SEMITONOS, max: Math.ceil(max) + MARGEN_SEMITONOS };
  const falta = SPAN_MINIMO - (rango.max - rango.min);
  if (falta > 0) {
    const abajo = Math.floor(falta / 2);
    rango = { min: rango.min - abajo, max: rango.max + (falta - abajo) };
  }
  return rango;
};
