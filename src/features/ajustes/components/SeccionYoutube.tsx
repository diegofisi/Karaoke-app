import { useState } from "react";
import Accordion from "@mui/material/Accordion";
import AccordionDetails from "@mui/material/AccordionDetails";
import AccordionSummary from "@mui/material/AccordionSummary";
import Alert from "@mui/material/Alert";
import Avatar from "@mui/material/Avatar";
import Button from "@mui/material/Button";
import CircularProgress from "@mui/material/CircularProgress";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import MenuItem from "@mui/material/MenuItem";
import Select from "@mui/material/Select";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import LoginIcon from "@mui/icons-material/Login";
import LogoutIcon from "@mui/icons-material/Logout";
import {
  useCerrarSesionYoutube,
  useCuentaYoutube,
  useIniciarSesionYoutube,
  useSesionYoutube,
} from "@/common/api/consultas";
import { elegirArchivoCookies } from "@/common/api/dialogo";
import { olvidarFalloRenovacion } from "@/common/api/sesionYoutube";
import { mensajeDeError } from "@/common/helpers/errores";
import { EstadoSesion, type Ajustes } from "@/common/interfaces";

export interface SeccionYoutubeProps {
  ajustes: Ajustes;
  cambiar: <K extends keyof Ajustes>(clave: K, valor: Ajustes[K]) => void;
}

/** Sesión de YouTube: para cuando YouTube pide "confirmar que no eres un bot". */
export const SeccionYoutube = ({ ajustes, cambiar }: SeccionYoutubeProps) => {
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

      <Accordion disableGutters elevation={0} sx={{ bgcolor: "transparent", "&:before": { display: "none" } }}>
        <AccordionSummary expandIcon={<ExpandMoreIcon />} sx={{ px: 0 }}>
          <Typography variant="body2">Opciones avanzadas</Typography>
        </AccordionSummary>
        <AccordionDetails sx={{ px: 0 }}>
          <Stack spacing={1.5}>
            <Typography variant="caption" sx={{ color: "text.secondary" }}>
              Un archivo cookies.txt elegido aquí tiene prioridad sobre la sesión de la app. Se exporta con la
              extensión «Get cookies.txt LOCALLY» desde youtube.com con tu sesión iniciada.
            </Typography>
            <Typography variant="body2" sx={{ wordBreak: "break-all" }}>
              {ajustes.youtube_cookies ?? "Sin archivo cookies.txt"}
            </Typography>
            <Stack direction="row" spacing={1}>
              <Button
                variant="outlined"
                size="small"
                onClick={() => {
                  void elegirArchivoCookies().then((c) => {
                    if (c !== null) cambiar("youtube_cookies", c);
                  });
                }}
              >
                Elegir cookies.txt…
              </Button>
              <Button size="small" onClick={() => cambiar("youtube_cookies", null)} disabled={ajustes.youtube_cookies === null}>
                Quitar
              </Button>
            </Stack>
            <Typography variant="caption" sx={{ color: "text.secondary" }}>
              Leer la sesión de un navegador (solo si no hay sesión en la app ni archivo). En Windows solo funciona con
              Firefox.
            </Typography>
            <FormControl size="small" sx={{ maxWidth: 240 }}>
              <InputLabel id="navegador">Navegador</InputLabel>
              <Select
                labelId="navegador"
                label="Navegador"
                value={ajustes.youtube_navegador ?? ""}
                onChange={(e) => cambiar("youtube_navegador", e.target.value === "" ? null : e.target.value)}
              >
                <MenuItem value="">Ninguno</MenuItem>
                <MenuItem value="firefox">Firefox</MenuItem>
                <MenuItem value="safari">Safari (Mac)</MenuItem>
                <MenuItem value="chrome">Chrome (Mac)</MenuItem>
                <MenuItem value="edge">Edge (Mac)</MenuItem>
                <MenuItem value="brave">Brave (Mac)</MenuItem>
              </Select>
            </FormControl>
          </Stack>
        </AccordionDetails>
      </Accordion>
    </Stack>
  );
};
