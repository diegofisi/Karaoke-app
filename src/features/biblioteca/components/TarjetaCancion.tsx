import Button from "@mui/material/Button";
import Card from "@mui/material/Card";
import CardActions from "@mui/material/CardActions";
import CardContent from "@mui/material/CardContent";
import Chip from "@mui/material/Chip";
import IconButton from "@mui/material/IconButton";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import DeleteOutlineIcon from "@mui/icons-material/DeleteOutlineOutlined";
import EmojiEventsIcon from "@mui/icons-material/EmojiEvents";
import MicIcon from "@mui/icons-material/Mic";
import ScheduleIcon from "@mui/icons-material/Schedule";
import WarningAmberIcon from "@mui/icons-material/WarningAmber";
import { textoLineasAproximadas } from "../helpers/filas";
import type { FilaCancion } from "../interfaces/filaCancion";

export interface TarjetaCancionProps {
  fila: FilaCancion;
  onCantar: (id: string) => void;
  onBorrar: (id: string) => void;
}

export const TarjetaCancion = ({ fila, onCantar, onBorrar }: TarjetaCancionProps) => (
  <Card sx={{ display: "flex", flexDirection: "column", height: "100%" }}>
    <CardContent sx={{ flexGrow: 1 }}>
      <Stack spacing={1.5}>
        <Stack spacing={0.25}>
          <Typography variant="h6" component="h2" sx={{ lineHeight: 1.3 }}>
            {fila.titulo}
          </Typography>
          <Typography variant="body2" sx={{ color: "text.secondary" }}>
            {fila.artista}
          </Typography>
        </Stack>
        <Stack direction="row" spacing={1} useFlexGap sx={{ flexWrap: "wrap" }}>
          <Chip size="small" label={fila.nombreIdioma} variant="outlined" />
          <Chip size="small" icon={<ScheduleIcon />} label={fila.duracionTexto} variant="outlined" />
          {fila.recordTexto !== null && (
            <Chip size="small" color="dorado" icon={<EmojiEventsIcon />} label={`Récord ${fila.recordTexto}`} />
          )}
          {fila.lineasRespaldo > 0 && (
            <Tooltip title="La alineación de estas líneas falló y sus tiempos son aproximados.">
              <Chip
                size="small"
                color="warning"
                variant="outlined"
                icon={<WarningAmberIcon />}
                label={textoLineasAproximadas(fila.lineasRespaldo)}
              />
            </Tooltip>
          )}
        </Stack>
      </Stack>
    </CardContent>
    <CardActions sx={{ px: 2, pb: 2, justifyContent: "space-between" }}>
      <Button variant="contained" startIcon={<MicIcon />} onClick={() => onCantar(fila.id)}>
        Cantar
      </Button>
      <Tooltip title="Borrar canción">
        <IconButton aria-label={`Borrar ${fila.titulo}`} onClick={() => onBorrar(fila.id)}>
          <DeleteOutlineIcon />
        </IconButton>
      </Tooltip>
    </CardActions>
  </Card>
);
