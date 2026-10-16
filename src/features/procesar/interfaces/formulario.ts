import type { Idioma } from "@/common/interfaces";

export const OrigenAudio = { Url: "url", Archivo: "archivo" } as const;
export type OrigenAudio = (typeof OrigenAudio)[keyof typeof OrigenAudio];

/** Valor del selector de idioma que deja decidir a la fábrica (`idioma: null`). */
export const IDIOMA_AUTOMATICO = "auto";
export type IdiomaFormulario = Idioma | typeof IDIOMA_AUTOMATICO;

export interface FormularioProceso {
  origen: OrigenAudio;
  url: string;
  archivo: string | null;
  titulo: string;
  artista: string;
  idioma: IdiomaFormulario;
  letra: string;
}

export type CambiarCampo = <K extends keyof FormularioProceso>(campo: K, valor: FormularioProceso[K]) => void;
