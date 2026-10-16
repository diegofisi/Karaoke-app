import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";

export interface DialogoBorrarProps {
  titulo: string | null;
  borrando: boolean;
  error: string | null;
  onConfirmar: () => void;
  onCerrar: () => void;
}

export const DialogoBorrar = ({ titulo, borrando, error, onConfirmar, onCerrar }: DialogoBorrarProps) => (
  <Dialog open={titulo !== null} onClose={borrando ? undefined : onCerrar} maxWidth="xs" fullWidth>
    <DialogTitle>¿Borrar la canción?</DialogTitle>
    <DialogContent>
      <Stack spacing={2}>
        <Typography variant="body1">
          Se borrarán «{titulo}», su audio separado y sus récords. No se puede deshacer.
        </Typography>
        {error !== null && <Alert severity="error">{error}</Alert>}
      </Stack>
    </DialogContent>
    <DialogActions>
      <Button onClick={onCerrar} disabled={borrando}>
        Cancelar
      </Button>
      <Button color="error" variant="contained" onClick={onConfirmar} disabled={borrando}>
        {borrando ? "Borrando…" : "Borrar"}
      </Button>
    </DialogActions>
  </Dialog>
);
