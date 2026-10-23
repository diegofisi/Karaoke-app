import type { VideoPrevio, VistaPrevia } from "@/common/interfaces";

/** Máximo de canciones de una lista que se procesan de una vez (cada una tarda minutos). */
export const MAXIMO_POR_LOTE = 25;
/** Una canción más larga que esto no se procesa (igual que la fábrica). */
export const DURACION_MAXIMA_S = 15 * 60;

export const TEXTO_PROBLEMA: Record<string, string> = {
  privado: "Vídeo privado",
  necesita_sesion: "Necesita iniciar sesión",
  no_disponible: "No disponible",
};

/** Por qué un vídeo no se puede procesar, o `null` si sí. */
export const motivoNoProcesable = (v: VideoPrevio): string | null => {
  if (v.problema !== null) return TEXTO_PROBLEMA[v.problema] ?? "No disponible";
  if (v.duracion !== null && v.duracion > DURACION_MAXIMA_S) return "Dura más de 15 minutos";
  return null;
};

export const entradasDe = (vista: VistaPrevia): VideoPrevio[] =>
  vista.tipo === "video" ? [vista] : vista.entradas;

/** Los vídeos procesables de la vista, hasta el máximo por lote. */
export const seleccionInicial = (vista: VistaPrevia): Set<string> =>
  new Set(
    entradasDe(vista)
      .filter((v) => motivoNoProcesable(v) === null)
      .slice(0, MAXIMO_POR_LOTE)
      .map((v) => v.url),
  );

/** "3:58" */
export const duracionCorta = (segundos: number | null): string => {
  if (segundos === null) return "";
  const s = Math.round(segundos);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
};
