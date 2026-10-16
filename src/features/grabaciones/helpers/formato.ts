import type { Grabacion } from "@/common/interfaces";
import { formatearTiempo } from "@/common/helpers/tiempo";

/** "15/10/2026 22:41 · 3:58 · archivo.mp3" */
export const describirGrabacion = (g: Grabacion): string => {
  const fecha = new Date(g.fecha);
  const cuando = Number.isNaN(fecha.getTime())
    ? g.fecha
    : fecha.toLocaleString("es", { dateStyle: "short", timeStyle: "short" });
  return `${cuando} · ${formatearTiempo(g.duracion)} · ${g.nombre}`;
};
