/** Fases de la pantalla del juego. Las cambia el estado de los frames de Rust, no un reloj propio. */
export const FaseJuego = {
  Lista: "lista",
  Arrancando: "arrancando",
  Sonando: "sonando",
  Pausada: "pausada",
  Terminando: "terminando",
  FinPractica: "fin_practica",
  Error: "error",
} as const;
export type FaseJuego = (typeof FaseJuego)[keyof typeof FaseJuego];
