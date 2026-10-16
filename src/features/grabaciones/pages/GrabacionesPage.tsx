import { useState } from "react";
import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import IconButton from "@mui/material/IconButton";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import DeleteIcon from "@mui/icons-material/Delete";
import FolderOpenIcon from "@mui/icons-material/FolderOpen";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import { useBorrarGrabacion, useGrabaciones } from "@/common/api/consultas";
import { CabeceraPagina } from "@/common/components/CabeceraPagina";
import { EstadoContenido } from "@/common/components/EstadoContenido";
import { mensajeDeError } from "@/common/helpers/errores";
import { useAccionesGrabacion } from "@/common/hooks/useAccionesGrabacion";
import type { Grabacion } from "@/common/interfaces";
import { describirGrabacion } from "../helpers/formato";

export const GrabacionesPage = () => {
  const lista = useGrabaciones();
  const acciones = useAccionesGrabacion();
  const borrado = useBorrarGrabacion();
  const [paraBorrar, fijarParaBorrar] = useState<Grabacion | null>(null);

  const confirmarBorrar = () => {
    if (paraBorrar === null) return;
    borrado.mutate(paraBorrar.ruta, { onSuccess: () => fijarParaBorrar(null) });
  };

  return (
    <Box sx={{ maxWidth: 900 }}>
      <CabeceraPagina titulo="Grabaciones" subtitulo="Tus partidas grabadas. Se guardan en la carpeta Música/Karaoke." />
      {acciones.error !== null && <Alert severity="error" sx={{ mb: 2 }}>{acciones.error}</Alert>}
      {lista.isPending && <EstadoContenido tipo="cargando" mensaje="Buscando grabaciones…" />}
      {lista.isError && (
        <EstadoContenido tipo="error" mensaje="No se pudieron leer las grabaciones." detalle={mensajeDeError(lista.error)} />
      )}
      {lista.data?.length === 0 && (
        <EstadoContenido
          tipo="vacio"
          mensaje="Todavía no hay grabaciones."
          detalle="Activa «Grabar mi voz» antes de empezar una canción."
        />
      )}
      <Stack spacing={1.5}>
        {lista.data?.map((g) => (
          <Paper key={g.ruta} sx={{ p: 2, borderRadius: 3 }}>
            <Stack direction="row" spacing={2} sx={{ alignItems: "center" }}>
              <Stack sx={{ flexGrow: 1, minWidth: 0 }}>
                <Typography variant="subtitle1" noWrap>
                  {g.artista ? `${g.artista} — ${g.titulo}` : g.titulo}
                </Typography>
                <Typography variant="body2" sx={{ color: "text.secondary" }} noWrap>
                  {describirGrabacion(g)}
                </Typography>
              </Stack>
              <Button variant="outlined" startIcon={<PlayArrowIcon />} onClick={() => acciones.abrir(g.ruta)}>
                Abrir
              </Button>
              <Tooltip title="Mostrar en carpeta">
                <IconButton aria-label="Mostrar en carpeta" onClick={() => acciones.mostrar(g.ruta)}>
                  <FolderOpenIcon />
                </IconButton>
              </Tooltip>
              <Tooltip title="Borrar">
                <IconButton aria-label="Borrar" onClick={() => fijarParaBorrar(g)}>
                  <DeleteIcon />
                </IconButton>
              </Tooltip>
            </Stack>
          </Paper>
        ))}
      </Stack>

      <Dialog open={paraBorrar !== null} onClose={() => fijarParaBorrar(null)}>
        <DialogTitle>¿Borrar la grabación?</DialogTitle>
        <DialogContent>
          <Typography variant="body2">{paraBorrar?.nombre}</Typography>
          {borrado.isError && <Alert severity="error" sx={{ mt: 2 }}>{mensajeDeError(borrado.error)}</Alert>}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => fijarParaBorrar(null)}>Cancelar</Button>
          <Button color="error" onClick={confirmarBorrar} disabled={borrado.isPending}>
            Borrar
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};
