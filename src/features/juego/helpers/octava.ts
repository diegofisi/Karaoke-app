/**
 * Mueve un tono por octavas enteras para dejarlo lo más cerca posible de
 * `referencia`. Ambos en MIDI con decimales; solo sirve para dibujar el tono
 * del usuario junto a la nota. Mismo cálculo que `ajustar_octava` de
 * crates/puntuacion (redondeo de la mitad alejándose de cero, como Rust).
 */
export const ajustarOctava = (tono: number, referencia: number): number => {
  const octavas = (referencia - tono) / 12;
  const redondeo = Math.sign(octavas) * Math.round(Math.abs(octavas));
  return tono + 12 * redondeo;
};
