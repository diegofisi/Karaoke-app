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
  /** Carpeta de la fábrica (la del repositorio o `fabrica/`); `null` = la de desarrollo. */
  carpeta_fabrica: string | null;
  /** cookies.txt con tu sesión de YouTube (si pide confirmar que no eres un bot). */
  youtube_cookies: string | null;
  /** Navegador del que leer la sesión: firefox, safari, chrome, edge, brave. */
  youtube_navegador: string | null;
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
