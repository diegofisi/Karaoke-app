import Button from "@mui/material/Button";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import EmojiEventsIcon from "@mui/icons-material/EmojiEvents";
import SchoolIcon from "@mui/icons-material/School";
import { Modo } from "@/common/interfaces";

export interface DialogoModoProps {
  titulo: string | null;
  onElegir: (modo: Modo) => void;
  onCerrar: () => void;
}

export const DialogoModo = ({ titulo, onElegir, onCerrar }: DialogoModoProps) => (
  <Dialog open={titulo !== null} onClose={onCerrar} maxWidth="xs" fullWidth>
    <DialogTitle>¿Cómo quieres cantar?</DialogTitle>
    <DialogContent>
      <Stack spacing={2}>
        <Typography variant="body2" sx={{ color: "text.secondary" }}>
          {titulo}
        </Typography>
        <Button
          variant="outlined"
          size="large"
          startIcon={<SchoolIcon />}
          onClick={() => onElegir(Modo.Practica)}
          sx={{ justifyContent: "flex-start", py: 1.5 }}
        >
          <Stack sx={{ alignItems: "flex-start", textAlign: "left" }}>
            <Typography variant="subtitle1" component="span">
              Práctica
            </Typography>
            <Typography variant="caption" component="span" sx={{ color: "text.secondary" }}>
              Sin puntuación. Puedes saltar a cualquier parte.
            </Typography>
          </Stack>
        </Button>
        <Button
          variant="contained"
          size="large"
          startIcon={<EmojiEventsIcon />}
          onClick={() => onElegir(Modo.Puntuacion)}
          sx={{ justifyContent: "flex-start", py: 1.5 }}
        >
          <Stack sx={{ alignItems: "flex-start", textAlign: "left" }}>
            <Typography variant="subtitle1" component="span">
              Puntuación
            </Typography>
            <Typography variant="caption" component="span">
              Canción entera, con puntos y récord.
            </Typography>
          </Stack>
        </Button>
      </Stack>
    </DialogContent>
    <DialogActions>
      <Button onClick={onCerrar}>Cancelar</Button>
    </DialogActions>
  </Dialog>
);
