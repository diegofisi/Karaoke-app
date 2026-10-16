// Juego: modo, frames del canal y resultado (CONTRATO.md §5 y §6).
// Origen de Calificacion, Nivel y EventoPuntuacion: crates/puntuacion/src/lib.rs.

export const Modo = { Practica: "practica", Puntuacion: "puntuacion" } as const;
export type Modo = (typeof Modo)[keyof typeof Modo];

export const Calificacion = { Perfecta: "perfecta", Bien: "bien", Regular: "regular", Fallo: "fallo" } as const;
export type Calificacion = (typeof Calificacion)[keyof typeof Calificacion];

export const Nivel = {
  Desafinado: "desafinado",
  Aficionado: "aficionado",
  Prometedor: "prometedor",
  Cantante: "cantante",
  Estrella: "estrella",
  Leyenda: "leyenda",
} as const;
export type Nivel = (typeof Nivel)[keyof typeof Nivel];

export const EstadoJuego = { Sonando: "sonando", Pausado: "pausado", Terminado: "terminado" } as const;
export type EstadoJuego = (typeof EstadoJuego)[keyof typeof EstadoJuego];

/** Un frame del micrófono (cada 10 ms), con `t` ya corregido por `latencia_ms`. */
export interface TonoMicrofono {
  t: number;
  midi: number | null;
}

export interface EventoNotaCerrada {
  tipo: "nota_cerrada";
  /** Índice en las notas planas de la canción (todas, ordenadas por inicio). */
  indice: number;
  calificacion: Calificacion;
  credito: number;
}

export interface EventoLineaCerrada {
  tipo: "linea_cerrada";
  indice_linea: number;
  ratio: number;
  bono: number;
}

export type EventoPuntuacion = EventoNotaCerrada | EventoLineaCerrada;

/** Uno cada ~16 ms mientras el juego corre. */
export interface FrameJuego {
  /** Tiempo de canción que se oye ahora (s). */
  t: number;
  /** Frames del micrófono desde el anterior. */
  tonos: TonoMicrofono[];
  puntos: number;
  combo: number;
  /** Notas y líneas cerradas desde el anterior. */
  eventos: EventoPuntuacion[];
  estado: EstadoJuego;
}

export interface PorCalificacion {
  perfecta: number;
  bien: number;
  regular: number;
  fallo: number;
}

export interface ResultadoJuego {
  puntos: number;
  nivel: Nivel;
  mejor_combo: number;
  precision_media: number;
  por_calificacion: PorCalificacion;
  record_anterior: number | null;
  es_record: boolean;
}
