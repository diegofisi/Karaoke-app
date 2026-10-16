import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import Stack from "@mui/material/Stack";
import FolderOpenIcon from "@mui/icons-material/FolderOpen";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import type { Grabacion } from "@/common/interfaces";

export interface AvisoGrabacionProps {
  grabacion: Grabacion | null;
  error: string | null;
  errorAccion: string | null;
  onAbrir: (ruta: string) => void;
  onMostrar: (ruta: string) => void;
}

/** Tras una partida grabada: dónde quedó y botones para abrirla. */
export const AvisoGrabacion = ({ grabacion, error, errorAccion, onAbrir, onMostrar }: AvisoGrabacionProps) => {
  if (grabacion === null && error === null) return null;
  if (grabacion === null) return <Alert severity="warning">{error}</Alert>;
  return (
    <Stack spacing={1}>
      <Alert
        severity="success"
        action={
          <Stack direction="row" spacing={1}>
            <Button color="inherit" size="small" startIcon={<PlayArrowIcon />} onClick={() => onAbrir(grabacion.ruta)}>
              Abrir
            </Button>
            <Button color="inherit" size="small" startIcon={<FolderOpenIcon />} onClick={() => onMostrar(grabacion.ruta)}>
              Mostrar en carpeta
            </Button>
          </Stack>
        }
      >
        Grabación guardada: {grabacion.nombre}
      </Alert>
      {error !== null && <Alert severity="info">{error}</Alert>}
      {errorAccion !== null && <Alert severity="error">{errorAccion}</Alert>}
    </Stack>
  );
};
