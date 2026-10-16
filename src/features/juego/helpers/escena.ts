import type { Cancion } from "@/common/interfaces";
import type { EscenaJuego } from "../interfaces/escena";
import { rangoMidiDe } from "./geometria";
import { aplanarNotas, duracionMaxima } from "./notas";

export const crearEscena = (cancion: Cancion): EscenaJuego => {
  const notas = aplanarNotas(cancion);
  return {
    notas,
    duracionMaxNota: duracionMaxima(notas),
    curva: cancion.tono,
    rango: rangoMidiDe(
      notas.map((nota) => nota.midi),
      cancion.tono.midi,
    ),
  };
};
