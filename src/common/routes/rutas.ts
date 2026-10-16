// Rutas de la app. Ningún componente escribe una ruta como literal.

import type { Grabacion, Modo, ResultadoJuego } from "@/common/interfaces";

export const RUTAS = {
  biblioteca: "/",
  procesar: "/procesar",
  ajustes: "/ajustes",
  grabaciones: "/grabaciones",
  juego: "/juego/:id",
  resultados: "/resultados/:id",
} as const;

export const PARAMETRO_MODO = "modo";

export const rutaJuego = (id: string, modo: Modo): string =>
  `/juego/${encodeURIComponent(id)}?${PARAMETRO_MODO}=${modo}`;

export const rutaResultados = (id: string): string => `/resultados/${encodeURIComponent(id)}`;

/** Lo que el juego pasa a resultados por el estado de navegación. */
export interface EstadoNavegacionResultados {
  resultado: ResultadoJuego;
  modo: Modo;
  grabacion: Grabacion | null;
  error_grabacion: string | null;
}
