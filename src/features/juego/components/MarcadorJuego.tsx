import { keyframes } from "@emotion/react";
import Box from "@mui/material/Box";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { Calificacion, type Calificacion as TipoCalificacion } from "@/common/interfaces";
import { formatearPuntos } from "@/common/helpers/tiempo";
import { useMuestreo } from "../hooks/useMuestreo";
import type { FuenteFrames } from "../stores/fuenteFrames";

export interface MarcadorJuegoProps {
  fuente: FuenteFrames;
}

/** Cuánto se ve el aviso de la última calificación. */
const VISIBLE_MS = 900;

interface LecturaMarcador {
  puntos: number;
  combo: number;
  calificacion: TipoCalificacion | null;
  contador: number;
}

const leerMarcador = (fuente: FuenteFrames): LecturaMarcador => {
  const ultima = fuente.ultimaCalificacion;
  const visible = ultima !== null && performance.now() - ultima.llegadaMs < VISIBLE_MS;
  return {
    puntos: fuente.ultimo?.puntos ?? 0,
    combo: fuente.ultimo?.combo ?? 0,
    calificacion: visible ? ultima.calificacion : null,
    contador: visible ? ultima.contador : 0,
  };
};

const mismaLectura = (a: LecturaMarcador, b: LecturaMarcador): boolean =>
  a.puntos === b.puntos && a.combo === b.combo && a.calificacion === b.calificacion && a.contador === b.contador;

const TEXTO_CALIFICACION: Record<TipoCalificacion, string> = {
  [Calificacion.Perfecta]: "¡Perfecta!",
  [Calificacion.Bien]: "¡Bien!",
  [Calificacion.Regular]: "Regular",
  [Calificacion.Fallo]: "Fallo",
};

const COLOR_CALIFICACION: Record<TipoCalificacion, string> = {
  [Calificacion.Perfecta]: "juego.perfecta",
  [Calificacion.Bien]: "juego.bien",
  [Calificacion.Regular]: "juego.regular",
  [Calificacion.Fallo]: "juego.fallo",
};

const aparecer = keyframes`
  from { opacity: 0; transform: translateY(8px) scale(0.9); }
  to { opacity: 1; transform: translateY(0) scale(1); }
`;

/** Puntos, combo y la última calificación. Se actualiza a 10 Hz como mucho. */
export const MarcadorJuego = ({ fuente }: MarcadorJuegoProps) => {
  const lectura = useMuestreo(() => leerMarcador(fuente), mismaLectura);

  return (
    <Stack direction="row" spacing={3} sx={{ alignItems: "center" }}>
      <Box sx={{ minWidth: 120, textAlign: "center" }} aria-live="polite">
        {lectura.calificacion !== null && (
          <Typography
            key={lectura.contador}
            variant="h6"
            sx={{ color: COLOR_CALIFICACION[lectura.calificacion], animation: `${aparecer} 180ms ease-out` }}
          >
            {TEXTO_CALIFICACION[lectura.calificacion]}
          </Typography>
        )}
      </Box>
      <Stack sx={{ alignItems: "flex-end" }}>
        <Typography variant="caption" sx={{ color: "text.secondary" }}>
          Combo
        </Typography>
        <Typography variant="cifraHud" sx={{ color: lectura.combo > 0 ? "secondary.main" : "text.secondary" }}>
          ×{lectura.combo}
        </Typography>
      </Stack>
      <Stack sx={{ alignItems: "flex-end", minWidth: 110 }}>
        <Typography variant="caption" sx={{ color: "text.secondary" }}>
          Puntos
        </Typography>
        <Typography variant="cifraHud">{formatearPuntos(lectura.puntos)}</Typography>
      </Stack>
    </Stack>
  );
};
