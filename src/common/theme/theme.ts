import { createTheme } from "@mui/material/styles";
import { COLORES_JUEGO, PALETA } from "./paleta";
import "./tipos";

const FUENTE =
  '"Inter", "Segoe UI", system-ui, -apple-system, "Yu Gothic UI", "Meiryo", "Hiragino Sans", "Noto Sans JP", sans-serif';

export const tema = createTheme({
  palette: {
    mode: "dark",
    primary: {
      main: PALETA.violeta,
      light: PALETA.violetaClaro,
      dark: PALETA.violetaOscuro,
      contrastText: PALETA.blanco,
    },
    secondary: {
      main: PALETA.cian,
      light: PALETA.cianClaro,
      dark: PALETA.cianOscuro,
      contrastText: PALETA.textoSobreClaro,
    },
    dorado: {
      main: PALETA.dorado,
      light: PALETA.doradoClaro,
      dark: PALETA.doradoOscuro,
      contrastText: PALETA.textoSobreDorado,
    },
    success: { main: PALETA.exito, contrastText: PALETA.textoSobreClaro },
    warning: { main: PALETA.aviso, contrastText: PALETA.textoSobreClaro },
    error: { main: PALETA.error, contrastText: PALETA.textoSobreClaro },
    info: { main: PALETA.info, contrastText: PALETA.textoSobreClaro },
    background: { default: PALETA.fondo, paper: PALETA.papel },
    text: {
      primary: PALETA.textoPrincipal,
      secondary: PALETA.textoSecundario,
      disabled: PALETA.textoDeshabilitado,
    },
    divider: PALETA.borde,
    juego: COLORES_JUEGO,
  },
  shape: { borderRadius: 12 },
  typography: {
    fontFamily: FUENTE,
    h4: { fontWeight: 800, letterSpacing: "-0.02em" },
    h5: { fontWeight: 700 },
    h6: { fontWeight: 700 },
    button: { textTransform: "none", fontWeight: 600 },
    letraActual: { fontFamily: FUENTE, fontSize: "2.25rem", fontWeight: 800, lineHeight: 1.6 },
    letraSiguiente: { fontFamily: FUENTE, fontSize: "1.5rem", fontWeight: 600, lineHeight: 1.6 },
    lectura: { fontFamily: FUENTE, fontSize: "0.85rem", fontWeight: 600 },
    romaji: { fontFamily: FUENTE, fontSize: "1.1rem", fontWeight: 500, fontStyle: "italic" },
    cifraHud: { fontFamily: FUENTE, fontSize: "1.75rem", fontWeight: 800, fontVariantNumeric: "tabular-nums" },
    cifraGrande: { fontFamily: FUENTE, fontSize: "4.5rem", fontWeight: 900, fontVariantNumeric: "tabular-nums" },
  },
  components: {
    MuiButton: { defaultProps: { disableElevation: true } },
    MuiPaper: { styleOverrides: { root: { backgroundImage: "none" } } },
    MuiCard: { defaultProps: { variant: "outlined" } },
    MuiTypography: {
      defaultProps: {
        variantMapping: {
          letraActual: "div",
          letraSiguiente: "div",
          lectura: "span",
          romaji: "div",
          cifraHud: "span",
          cifraGrande: "span",
        },
      },
    },
  },
});
