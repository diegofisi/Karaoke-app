import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Checkbox from "@mui/material/Checkbox";
import Chip from "@mui/material/Chip";
import LinearProgress from "@mui/material/LinearProgress";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import PlaylistPlayIcon from "@mui/icons-material/PlaylistPlay";
import type { VistaPrevia } from "@/common/interfaces";
import { MAXIMO_POR_LOTE, entradasDe, motivoNoProcesable } from "../helpers/vistaPrevia";
import { TarjetaVideoPrevio } from "./TarjetaVideoPrevio";

export interface VistaPreviaUrlProps {
  analizando: boolean;
  error: string | null;
  vista: VistaPrevia | null;
  seleccion: Set<string>;
  onCambiar: (url: string, seleccionado: boolean) => void;
  onTodos: (seleccionar: boolean) => void;
  onReintentar: () => void;
}

/** Lo que se va a procesar: un vídeo, o una lista con casillas (adaptado de youtube-downloader: PlaylistGroup). */
export const VistaPreviaUrl = ({ analizando, error, vista, seleccion, onCambiar, onTodos, onReintentar }: VistaPreviaUrlProps) => {
  if (analizando) {
    return (
      <Box>
        <Typography variant="body2" sx={{ mb: 1 }}>
          Leyendo el vídeo en YouTube…
        </Typography>
        <LinearProgress />
      </Box>
    );
  }
  if (error !== null) {
    return (
      <Alert severity="error" action={<Button color="inherit" size="small" onClick={onReintentar}>Reintentar</Button>}>
        {error}
      </Alert>
    );
  }
  if (vista === null) return null;

  if (vista.tipo === "video") {
    return <TarjetaVideoPrevio video={vista} seleccionado sinCasilla onCambiar={onCambiar} />;
  }

  const entradas = entradasDe(vista);
  const procesables = entradas.filter((v) => motivoNoProcesable(v) === null);
  const todas = procesables.length > 0 && procesables.every((v) => seleccion.has(v.url));
  const topeAlcanzado = seleccion.size >= MAXIMO_POR_LOTE;
  return (
    <Stack spacing={1.5}>
      <Stack direction="row" spacing={1.5} sx={{ alignItems: "center" }}>
        <Checkbox
          checked={todas}
          indeterminate={!todas && seleccion.size > 0}
          onChange={(e) => onTodos(e.target.checked)}
          slotProps={{ input: { "aria-label": "Seleccionar todas" } }}
        />
        <PlaylistPlayIcon color="primary" />
        <Stack sx={{ minWidth: 0, flexGrow: 1 }}>
          <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}>
            <Chip size="small" label="LISTA" color="primary" />
            <Typography variant="body1" noWrap sx={{ fontWeight: 600 }}>
              {vista.titulo}
            </Typography>
          </Stack>
          <Typography variant="body2" sx={{ color: "text.secondary" }} noWrap>
            {vista.canal ? `${vista.canal} · ` : ""}
            {entradas.length} vídeos · {seleccion.size} elegidos
          </Typography>
        </Stack>
      </Stack>
      {procesables.length > MAXIMO_POR_LOTE && (
        <Alert severity="info">
          Cada canción tarda minutos en procesarse: como mucho {MAXIMO_POR_LOTE} a la vez. Desmarca una para poder
          marcar otra; el resto lo procesas después.
        </Alert>
      )}
      <Stack spacing={1} sx={{ maxHeight: 420, overflowY: "auto", pr: 0.5 }}>
        {entradas.map((v) => (
          <TarjetaVideoPrevio
            key={v.url}
            video={v}
            seleccionado={seleccion.has(v.url)}
            topeAlcanzado={topeAlcanzado}
            onCambiar={onCambiar}
          />
        ))}
      </Stack>
    </Stack>
  );
};
