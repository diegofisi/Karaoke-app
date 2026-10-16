// Ajustes, micrófonos y calibración (CONTRATO.md §5 y §6).
// Origen de Dificultad: crates/puntuacion/src/lib.rs.

export const Dificultad = { Facil: "facil", Normal: "normal", Dificil: "dificil" } as const;
export type Dificultad = (typeof Dificultad)[keyof typeof Dificultad];

export interface Microfono {
  id: string;
  nombre: string;
  por_defecto: boolean;
}

/** Volúmenes de 0 a 1; `volumen_guia` 0 = sin voz guía. */
export interface Ajustes {
  microfono: string | null;
  latencia_ms: number;
  dificultad: Dificultad;
  volumen_musica: number;
  volumen_guia: number;
  /** Volumen de tu voz en la grabación: 1 = automático, de 0 a 3. */
  volumen_voz: number;
  mostrar_romaji: boolean;
  /** Grabar la voz durante la partida y guardarla mezclada con la música. */
  grabar: boolean;
}

export interface ResultadoCalibracion {
  latencia_ms: number;
  medidas_ms: number[];
  fiable: boolean;
}

export interface EventoCalibracionPitido {
  tipo: "pitido";
  indice: number;
  total: number;
}

export interface EventoCalibracionMedida {
  tipo: "medida";
  indice: number;
  /** `null` = el pitido no se detectó. */
  ms: number | null;
}

export type EventoCalibracion = EventoCalibracionPitido | EventoCalibracionMedida;
