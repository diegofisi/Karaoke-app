// El ÚNICO archivo con colores literales. Un tono nuevo se añade aquí, se
// registra en theme.ts y se declara en tipos.ts. El canvas del juego también
// pinta con estos colores (leídos del tema).

import type { ColoresJuego } from "./tipos";

export const PALETA = {
  fondo: "#0F0B1A",
  papel: "#1A1430",
  papelElevado: "#231B3D",
  borde: "#2E2650",

  textoPrincipal: "#F3F0FF",
  textoSecundario: "#B8B0D9",
  textoDeshabilitado: "#6E668F",

  violeta: "#8B5CF6",
  violetaClaro: "#A78BFA",
  violetaOscuro: "#6D28D9",
  blanco: "#FFFFFF",

  cian: "#22D3EE",
  cianClaro: "#67E8F9",
  cianOscuro: "#0E9FB5",
  textoSobreClaro: "#0F0B1A",

  dorado: "#F5B942",
  doradoClaro: "#FFD27A",
  doradoOscuro: "#C98A12",
  textoSobreDorado: "#1A1206",

  exito: "#34D399",
  aviso: "#FBBF24",
  error: "#F87171",
  info: "#60A5FA",
} as const;

export const COLORES_JUEGO: ColoresJuego = {
  fondo: "#120D22",
  lineaSemitono: "#FFFFFF0A",
  lineaOctava: "#FFFFFF1F",
  lineaAhora: "#F3F0FFCC",
  nota: "#5B3FC4",
  notaBorde: "#A78BFA",
  notaActiva: "#9F7CFF",
  notaDorada: "#E0A21F",
  notaDoradaBorde: "#FFD27A",
  notaLibre: "#6E668F55",
  curvaReferencia: "#C4B5FD38",
  rastro: "#22D3EE",
  rastroSombra: "#22D3EE80",
  perfecta: "#34D399",
  bien: "#60A5FA",
  regular: "#FBBF24",
  fallo: "#F87171",
  letraPendiente: "#F3F0FF",
  letraCantada: "#C4A1FF",
  letraSiguiente: "#8F87B3",
  respaldo: "#FBBF2499",
  velo: "#0F0B1AD9",
};
