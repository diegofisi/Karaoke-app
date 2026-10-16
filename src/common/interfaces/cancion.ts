// Espejo de `cancion.json` versión 1 (CONTRATO.md §3).
// Origen: crates/karaoke-core/src/lib.rs (Cancion, Origen, Linea, Palabra, Nota, CurvaTono).

export const Idioma = { Es: "es", En: "en", Ja: "ja" } as const;
export type Idioma = (typeof Idioma)[keyof typeof Idioma];

export const OrigenLetra = { Lrclib: "lrclib", Usuario: "usuario", Transcrita: "transcrita" } as const;
export type OrigenLetra = (typeof OrigenLetra)[keyof typeof OrigenLetra];

export const Metodo = { Alineada: "alineada", Respaldo: "respaldo" } as const;
export type Metodo = (typeof Metodo)[keyof typeof Metodo];

export const TipoNota = { Normal: "normal", Dorada: "dorada", Libre: "libre" } as const;
export type TipoNota = (typeof TipoNota)[keyof typeof TipoNota];

export interface Nota {
  inicio: number;
  fin: number;
  /** Mediana del tono de referencia en la nota, MIDI con decimales. */
  midi: number;
  tipo: TipoNota;
}

export interface Palabra {
  texto: string;
  /** Solo japonés: lectura en hiragana (furigana). */
  lectura: string | null;
  inicio: number;
  fin: number;
  /** Vacía = palabra sin tono fiable: se canta pero no puntúa tono. */
  notas: Nota[];
}

export interface Linea {
  inicio: number;
  fin: number;
  texto: string;
  /** Solo japonés. */
  romaji: string | null;
  /** Calidad de la alineación de la línea, 0 a 1. */
  confianza: number;
  metodo: Metodo;
  palabras: Palabra[];
}

export interface Origen {
  youtube_id: string | null;
  letra: OrigenLetra;
  separador: string;
  alineador: string;
  tono: string;
  fabrica_version: string;
  creada: string;
}

/** Tono de referencia frame a frame: el frame `i` está en `inicio + i * paso`. */
export interface CurvaTono {
  inicio: number;
  paso: number;
  midi: (number | null)[];
}

export interface Cancion {
  version: number;
  id: string;
  titulo: string;
  artista: string;
  duracion: number;
  idioma: Idioma;
  origen: Origen;
  lineas: Linea[];
  tono: CurvaTono;
}
