import { useState } from "react";
import Alert from "@mui/material/Alert";
import Avatar from "@mui/material/Avatar";
import Button from "@mui/material/Button";
import CircularProgress from "@mui/material/CircularProgress";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import LoginIcon from "@mui/icons-material/Login";
import LogoutIcon from "@mui/icons-material/Logout";
import {
  useCerrarSesionYoutube,
  useCuentaYoutube,
  useIniciarSesionYoutube,
  useSesionYoutube,
} from "@/common/api/consultas";
import { olvidarFalloRenovacion } from "@/common/api/sesionYoutube";
import { mensajeDeError } from "@/common/helpers/errores";
import { EstadoSesion } from "@/common/interfaces";

/** Sesión de YouTube: para cuando YouTube pide "confirmar que no eres un bot". */
export const SeccionYoutube = () => {
  const sesion = useSesionYoutube();
  const conSesion = sesion.data === EstadoSesion.Conectada;
  const cuenta = useCuentaYoutube(conSesion);
  const inicio = useIniciarSesionYoutube();
  const cierre = useCerrarSesionYoutube();
  const [ventanaAbierta, fijarVentanaAbierta] = useState(false);
  const [error, fijarError] = useState<string | null>(null);

  const iniciar = () => {
    fijarError(null);
    fijarVentanaAbierta(true);
    inicio.mutate(
      (evento) => {
        if (evento.tipo === "conectada") olvidarFalloRenovacion();
        if (evento.tipo === "error") fijarError(evento.mensaje);
        if (evento.tipo !== "error") fijarVentanaAbierta(false);
      },
      {
        onError: (e) => {
          fijarError(mensajeDeError(e));
          fijarVentanaAbierta(false);
        },
      },
    );
  };

  return (
    <Stack spacing={2}>
      <Typography variant="body2" sx={{ color: "text.secondary" }}>
        A veces YouTube bloquea las descargas con «confirma que no eres un bot». Con tu sesión iniciada, la app
        descarga como tú y el bloqueo desaparece. La sesión se guarda solo en este equipo.
      </Typography>

      {sesion.isPending && <CircularProgress size={24} />}

      {conSesion && (
        <Stack direction="row" spacing={2} sx={{ alignItems: "center" }}>
          <Avatar src={cuenta.data?.foto ?? undefined} alt="" sx={{ width: 48, height: 48 }}>
            {cuenta.data?.nombre.slice(0, 1) ?? "Y"}
          </Avatar>
          <Stack sx={{ flexGrow: 1, minWidth: 0 }}>
            <Typography variant="subtitle1" noWrap>
              {cuenta.isPending ? "Sesión iniciada" : (cuenta.data?.nombre ?? "Sesión iniciada")}
            </Typography>
            <Typography variant="body2" sx={{ color: "text.secondary" }} noWrap>
              {cuenta.data?.usuario ?? "Conectada a YouTube"}
            </Typography>
          </Stack>
          <Button color="inherit" startIcon={<LogoutIcon />} onClick={() => cierre.mutate()} disabled={cierre.isPending}>
            Cerrar sesión
          </Button>
        </Stack>
      )}

      {!sesion.isPending && !conSesion && (
        <Stack spacing={1.5} sx={{ alignItems: "flex-start" }}>
          {sesion.data === EstadoSesion.Caducada && (
            <Alert severity="warning">Tu sesión de YouTube caducó. Vuelve a iniciarla.</Alert>
          )}
          <Button variant="contained" startIcon={<LoginIcon />} onClick={iniciar} disabled={ventanaAbierta}>
            {ventanaAbierta ? "Esperando el inicio de sesión…" : "Iniciar sesión con Google"}
          </Button>
          {ventanaAbierta && (
            <Typography variant="caption" sx={{ color: "text.secondary" }}>
              Inicia sesión en la ventana que se abrió. Se cerrará sola cuando llegues a YouTube.
            </Typography>
          )}
        </Stack>
      )}

      {error !== null && <Alert severity="error">{error}</Alert>}
      {cierre.isError && <Alert severity="error">{mensajeDeError(cierre.error)}</Alert>}
    </Stack>
  );
};
