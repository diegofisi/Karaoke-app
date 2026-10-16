import { altoSemitono, midiAY, tiempoAX, ventanaVisible, type RangoMidi } from "../helpers/geometria";

/** Medidas de un frame de dibujo, en píxeles CSS. */
export interface VistaLienzo {
  ancho: number;
  alto: number;
  tAhora: number;
  rango: RangoMidi;
  desde: number;
  hasta: number;
  semitono: number;
  x: (t: number) => number;
  y: (midi: number) => number;
}

export const crearVista = (ancho: number, alto: number, tAhora: number, rango: RangoMidi): VistaLienzo => {
  const { desde, hasta } = ventanaVisible(tAhora, ancho);
  return {
    ancho,
    alto,
    tAhora,
    rango,
    desde,
    hasta,
    semitono: altoSemitono(rango, alto),
    x: (t) => tiempoAX(t, tAhora, ancho),
    y: (midi) => midiAY(midi, rango, alto),
  };
};
