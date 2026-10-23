import Box from "@mui/material/Box";
import Checkbox from "@mui/material/Checkbox";
import Chip from "@mui/material/Chip";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import MusicNoteIcon from "@mui/icons-material/MusicNote";
import type { VideoPrevio } from "@/common/interfaces";
import { duracionCorta, motivoNoProcesable } from "../helpers/vistaPrevia";

export interface TarjetaVideoPrevioProps {
  video: VideoPrevio;
  seleccionado: boolean;
  /** Sin casilla: es el único vídeo (no hay nada que elegir). */
  sinCasilla?: boolean;
  /** Ya hay el máximo de elegidos: las casillas sin marcar se bloquean. */
  topeAlcanzado?: boolean;
  onCambiar: (url: string, seleccionado: boolean) => void;
}

/** Miniatura, título, artista y duración de un vídeo (adaptado de youtube-downloader: VideoCard). */
export const TarjetaVideoPrevio = ({
  video,
  seleccionado,
  sinCasilla = false,
  topeAlcanzado = false,
  onCambiar,
}: TarjetaVideoPrevioProps) => {
  const motivo = motivoNoProcesable(video);
  const procesable = motivo === null;
  const bloqueada = !procesable || (topeAlcanzado && !seleccionado);
  return (
    <Stack
      direction="row"
      spacing={1.5}
      sx={{
        alignItems: "center",
        p: 1.25,
        borderRadius: 2.5,
        border: 1,
        borderColor: seleccionado && procesable ? "primary.main" : "divider",
        opacity: procesable ? 1 : 0.55,
      }}
    >
      {!sinCasilla && (
        <Checkbox
          checked={seleccionado && procesable}
          disabled={bloqueada}
          onChange={(e) => onCambiar(video.url, e.target.checked)}
          slotProps={{ input: { "aria-label": `Procesar ${video.titulo}` } }}
        />
      )}
      <Box
        sx={{
          width: 120,
          height: 68,
          flexShrink: 0,
          borderRadius: 1.5,
          overflow: "hidden",
          bgcolor: "background.default",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "text.secondary",
        }}
      >
        {video.miniatura !== null ? (
          <Box component="img" src={video.miniatura} alt="" sx={{ width: "100%", height: "100%", objectFit: "cover" }} />
        ) : (
          <MusicNoteIcon />
        )}
      </Box>
      <Stack sx={{ minWidth: 0, flexGrow: 1 }}>
        <Typography variant="body1" noWrap sx={{ fontWeight: 600 }}>
          {video.titulo}
        </Typography>
        <Typography variant="body2" noWrap sx={{ color: "text.secondary" }}>
          {video.canal}
        </Typography>
        <Stack direction="row" spacing={1} sx={{ mt: 0.5, alignItems: "center" }}>
          {video.duracion !== null && (
            <Typography variant="caption" sx={{ fontFamily: "monospace", color: "text.secondary" }}>
              {duracionCorta(video.duracion)}
            </Typography>
          )}
          {motivo !== null && <Chip size="small" label={motivo} color="warning" variant="outlined" />}
        </Stack>
      </Stack>
    </Stack>
  );
};
