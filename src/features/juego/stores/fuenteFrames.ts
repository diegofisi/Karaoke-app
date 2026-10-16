// Fuente de frames del juego: el último FrameJuego, su hora de llegada, el
// rastro del micrófono y las calificaciones. Es un objeto mutable que se crea
// una vez por partida (useState perezoso) y que el canal escribe y el bucle de
// dibujo lee en cada frame. Nunca pasa por el estado de React.

import {
  EstadoJuego,
  type Calificacion,
  type EventoLineaCerrada,
  type FrameJuego,
  type TonoMicrofono,
} from "@/common/interfaces";

/** Segundos de rastro del micrófono que se guardan. */
const SEGUNDOS_RASTRO = 3;
/** Tope de la interpolación: si no llega un frame en este tiempo, t se congela. */
const TOPE_INTERPOLACION_MS = 100;
/** Un salto atrás mayor que este (buscar en práctica) vacía el rastro. */
const SALTO_ATRAS_S = 0.25;

export interface UltimaCalificacion {
  calificacion: Calificacion;
  llegadaMs: number;
  contador: number;
}

export interface FuenteFrames {
  ultimo: FrameJuego | null;
  llegadaMs: number;
  rastro: TonoMicrofono[];
  calificaciones: Map<number, Calificacion>;
  lineasCerradas: Map<number, EventoLineaCerrada>;
  ultimaCalificacion: UltimaCalificacion | null;
}

export const crearFuenteFrames = (): FuenteFrames => ({
  ultimo: null,
  llegadaMs: 0,
  rastro: [],
  calificaciones: new Map(),
  lineasCerradas: new Map(),
  ultimaCalificacion: null,
});

export const reiniciarFuente = (fuente: FuenteFrames): void => {
  fuente.ultimo = null;
  fuente.llegadaMs = 0;
  fuente.rastro.length = 0;
  fuente.calificaciones.clear();
  fuente.lineasCerradas.clear();
  fuente.ultimaCalificacion = null;
};

export const vaciarRastro = (fuente: FuenteFrames): void => {
  fuente.rastro.length = 0;
};

export const recibirFrame = (fuente: FuenteFrames, frame: FrameJuego, ahoraMs: number): void => {
  const anterior = fuente.ultimo;
  if (anterior !== null && frame.t < anterior.t - SALTO_ATRAS_S) vaciarRastro(fuente);
  fuente.ultimo = frame;
  fuente.llegadaMs = ahoraMs;

  for (const tono of frame.tonos) fuente.rastro.push(tono);
  const limite = frame.t - SEGUNDOS_RASTRO;
  let viejos = 0;
  while (viejos < fuente.rastro.length && (fuente.rastro[viejos]?.t ?? limite) < limite) viejos += 1;
  if (viejos > 0) fuente.rastro.splice(0, viejos);

  for (const evento of frame.eventos) {
    if (evento.tipo === "nota_cerrada") {
      fuente.calificaciones.set(evento.indice, evento.calificacion);
      fuente.ultimaCalificacion = {
        calificacion: evento.calificacion,
        llegadaMs: ahoraMs,
        contador: (fuente.ultimaCalificacion?.contador ?? 0) + 1,
      };
    } else {
      fuente.lineasCerradas.set(evento.indice_linea, evento);
    }
  }
};

/**
 * Tiempo de canción para dibujar: el `t` del último frame más lo que ha pasado
 * desde que llegó, solo mientras suena y con tope (CONTRATO.md §6: la interfaz
 * interpola y nunca lleva su propio reloj).
 */
export const tiempoInterpolado = (t: number, llegadaMs: number, ahoraMs: number, sonando: boolean): number => {
  if (!sonando) return t;
  const transcurrido = Math.min(Math.max(0, ahoraMs - llegadaMs), TOPE_INTERPOLACION_MS);
  return t + transcurrido / 1000;
};

export const tiempoDeDibujo = (fuente: FuenteFrames, ahoraMs: number): number => {
  const frame = fuente.ultimo;
  if (frame === null) return 0;
  return tiempoInterpolado(frame.t, fuente.llegadaMs, ahoraMs, frame.estado === EstadoJuego.Sonando);
};
