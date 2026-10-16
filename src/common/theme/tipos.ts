// Declaraciones de tema propias: paleta `dorado`, colores del juego y
// variantes de tipografía. Un color nuevo se declara aquí y se define en paleta.ts.

import type { CSSProperties } from "react";
import type { PaletteColor, PaletteColorOptions } from "@mui/material/styles";

export interface ColoresJuego {
  fondo: string;
  lineaSemitono: string;
  lineaOctava: string;
  lineaAhora: string;
  nota: string;
  notaBorde: string;
  notaActiva: string;
  notaDorada: string;
  notaDoradaBorde: string;
  notaLibre: string;
  curvaReferencia: string;
  rastro: string;
  rastroSombra: string;
  perfecta: string;
  bien: string;
  regular: string;
  fallo: string;
  letraPendiente: string;
  letraCantada: string;
  letraSiguiente: string;
  respaldo: string;
  velo: string;
}

declare module "@mui/material/styles" {
  interface Palette {
    dorado: PaletteColor;
    juego: ColoresJuego;
  }
  interface PaletteOptions {
    dorado?: PaletteColorOptions;
    juego?: ColoresJuego;
  }
  interface TypographyVariants {
    letraActual: CSSProperties;
    letraSiguiente: CSSProperties;
    lectura: CSSProperties;
    romaji: CSSProperties;
    cifraHud: CSSProperties;
    cifraGrande: CSSProperties;
  }
  interface TypographyVariantsOptions {
    letraActual?: CSSProperties;
    letraSiguiente?: CSSProperties;
    lectura?: CSSProperties;
    romaji?: CSSProperties;
    cifraHud?: CSSProperties;
    cifraGrande?: CSSProperties;
  }
}

declare module "@mui/material/Typography" {
  interface TypographyPropsVariantOverrides {
    letraActual: true;
    letraSiguiente: true;
    lectura: true;
    romaji: true;
    cifraHud: true;
    cifraGrande: true;
  }
}

declare module "@mui/material/Chip" {
  interface ChipPropsColorOverrides {
    dorado: true;
  }
}

declare module "@mui/material/Button" {
  interface ButtonPropsColorOverrides {
    dorado: true;
  }
}
