// Tipos de la fábrica: estado, petición y eventos del canal (CONTRATO.md §4, §5 y §6).

import type { Idioma } from "./cancion";

export interface EstadoFabrica {
  /** Todos los requisitos obligatorios en verde: se puede procesar. */
  lista: boolean;
  python: string | null;
  cuda: boolean;
  gpu: string | null;
  mensaje: string | null;
  requisitos: Requisito[];
  /** La app puede instalar la fábrica sola. */
  puede_instalar: boolean;
}

/** Una comprobación previa a procesar (CONTRATO.md §5). */
export interface Requisito {
  id: "fabrica" | "ffmpeg" | "javascript" | "gpu";
  nombre: string;
  ok: boolean;
  /** `false` = solo avisa (la GPU). */
  obligatorio: boolean;
  detalle: string;
}

/** Progreso de `instalar_fabrica` (CONTRATO.md §6). */
export type EventoInstalacion =
  | { tipo: "paso"; indice: number; total: number; nombre: string; mensaje: string }
  | { tipo: "linea"; texto: string };

/** Argumento `peticion` de `procesar_cancion`. `letra`: texto pegado (.txt o LRC). */
export interface PeticionProceso {
  url: string | null;
  archivo: string | null;
  titulo: string | null;
  artista: string | null;
  idioma: Idioma | null;
  letra: string | null;
}

export const Etapa = {
  Descarga: "descarga",
  Separacion: "separacion",
  Letra: "letra",
  Alineacion: "alineacion",
  Tono: "tono",
  Notas: "notas",
  Guardado: "guardado",
} as const;
export type Etapa = (typeof Etapa)[keyof typeof Etapa];

/** Orden en que la fábrica recorre las etapas (CONTRATO.md §4). */
export const ETAPAS_ORDENADAS: readonly Etapa[] = [
  Etapa.Descarga,
  Etapa.Separacion,
  Etapa.Letra,
  Etapa.Alineacion,
  Etapa.Tono,
  Etapa.Notas,
  Etapa.Guardado,
];

export interface EventoFabricaEtapa {
  tipo: "etapa";
  etapa: string;
  /** 0 a 1. */
  avance: number;
  mensaje: string;
}

export interface EventoFabricaAviso {
  tipo: "aviso";
  mensaje: string;
}

export interface EventoFabricaListo {
  tipo: "listo";
  id: string;
  /** Ruta de `cancion.json`. */
  ruta: string;
}

export interface EventoFabricaError {
  tipo: "error";
  mensaje: string;
  etapa: string;
}

export interface EventoFabricaCancelado {
  tipo: "cancelado";
}

/** Mensajes del canal de `procesar_cancion`. Tras listo, error o cancelado no llega nada más. */
export type EventoFabrica =
  | EventoFabricaEtapa
  | EventoFabricaAviso
  | EventoFabricaListo
  | EventoFabricaError
  | EventoFabricaCancelado;
