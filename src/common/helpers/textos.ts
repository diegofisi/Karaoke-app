import { Idioma, Modo, type Idioma as TipoIdioma, type Modo as TipoModo } from "@/common/interfaces";

export const NOMBRE_IDIOMA: Record<TipoIdioma, string> = {
  [Idioma.Es]: "Español",
  [Idioma.En]: "Inglés",
  [Idioma.Ja]: "Japonés",
};

export const NOMBRE_MODO: Record<TipoModo, string> = {
  [Modo.Practica]: "Práctica",
  [Modo.Puntuacion]: "Puntuación",
};

/** Valida un texto abierto (de la URL o de un formulario) contra los modos del contrato. */
export const aModo = (valor: string | null): TipoModo | null =>
  valor === Modo.Practica || valor === Modo.Puntuacion ? valor : null;

export const aIdioma = (valor: string | null): TipoIdioma | null =>
  valor === Idioma.Es || valor === Idioma.En || valor === Idioma.Ja ? valor : null;
