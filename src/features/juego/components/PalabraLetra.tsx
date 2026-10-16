import type { Ref } from "react";
import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";
import type { Theme } from "@mui/material/styles";
import type { Palabra } from "@/common/interfaces";

export interface PalabraLetraProps {
  palabra: Palabra;
  /** Japonés: lectura en <ruby> sobre la palabra si la trae. */
  conLectura: boolean;
  /** Si llega, la palabra se barre: el bucle de la letra escribe `--avance` en este elemento. */
  refBarrido?: Ref<HTMLElement>;
}

/** Barrido de color con background-clip: la parte cantada hasta `--avance`, el resto pendiente. */
const estiloBarrido = (tema: Theme) => ({
  backgroundImage: `linear-gradient(90deg, ${tema.palette.juego.letraCantada} 0 var(--avance, 0%), ${tema.palette.juego.letraPendiente} var(--avance, 0%) 100%)`,
  backgroundClip: "text",
  WebkitBackgroundClip: "text",
  color: "transparent",
  WebkitTextFillColor: "transparent",
});

export const PalabraLetra = ({ palabra, conLectura, refBarrido }: PalabraLetraProps) => {
  const texto =
    refBarrido === undefined ? (
      <Box component="span">{palabra.texto}</Box>
    ) : (
      <Box component="span" ref={refBarrido} sx={estiloBarrido}>
        {palabra.texto}
      </Box>
    );

  const lectura = palabra.lectura;
  if (!conLectura || lectura === null || lectura === palabra.texto) return texto;

  return (
    <Box component="ruby" sx={{ rubyPosition: "over" }}>
      {texto}
      <Typography component="rt" variant="lectura" sx={{ color: "text.secondary" }}>
        {lectura}
      </Typography>
    </Box>
  );
};
