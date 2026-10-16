import { useParams, useSearchParams } from "react-router-dom";
import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Chip from "@mui/material/Chip";
import CircularProgress from "@mui/material/CircularProgress";
import FormControlLabel from "@mui/material/FormControlLabel";
import Stack from "@mui/material/Stack";
import Switch from "@mui/material/Switch";
import Typography from "@mui/material/Typography";
import PauseIcon from "@mui/icons-material/Pause";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import ReplayIcon from "@mui/icons-material/Replay";
import StopIcon from "@mui/icons-material/Stop";
import FiberManualRecordIcon from "@mui/icons-material/FiberManualRecord";
import { useAjustes, useCancion, useGuardarAjustes } from "@/common/api/consultas";
import { AvisoGrabacion } from "@/common/components/AvisoGrabacion";
import { EstadoContenido } from "@/common/components/EstadoContenido";
import { mensajeDeError } from "@/common/helpers/errores";
import { useAccionesGrabacion } from "@/common/hooks/useAccionesGrabacion";
import { Modo, type Ajustes, type Cancion, type Modo as TipoModo } from "@/common/interfaces";
import { PARAMETRO_MODO } from "@/common/routes/rutas";
import { BarraPosicion } from "../components/BarraPosicion";
import { LetraJuego } from "../components/LetraJuego";
import { LienzoJuego } from "../components/LienzoJuego";
import { MarcadorJuego } from "../components/MarcadorJuego";
import { useJuego } from "../hooks/useJuego";
import { FaseJuego } from "../interfaces/fase";

const leerModo = (valor: string | null): TipoModo => (valor === Modo.Practica ? Modo.Practica : Modo.Puntuacion);

/** Carga la canción y los ajustes; la partida se monta cuando están los dos. */
export const JuegoPage = () => {
  const { id = "" } = useParams();
  const [parametros] = useSearchParams();
  const modo = leerModo(parametros.get(PARAMETRO_MODO));
  const cancion = useCancion(id);
  const ajustes = useAjustes();

  if (cancion.isPending || ajustes.isPending) {
    return <EstadoContenido tipo="cargando" mensaje="Preparando la canción…" />;
  }
  if (cancion.isError || ajustes.isError) {
    return (
      <EstadoContenido
        tipo="error"
        mensaje="No se pudo abrir la canción."
        detalle={mensajeDeError(cancion.error ?? ajustes.error)}
      />
    );
  }
  return <Partida cancion={cancion.data} ajustes={ajustes.data} modo={modo} />;
};

interface PartidaProps {
  cancion: Cancion;
  ajustes: Ajustes;
  modo: TipoModo;
}

const Partida = ({ cancion, ajustes, modo }: PartidaProps) => {
  const juego = useJuego(cancion, modo);
  const guardado = useGuardarAjustes();
  const acciones = useAccionesGrabacion();
  const { fase } = juego;
  const fijarGrabar = (grabar: boolean) => guardado.mutate({ ...ajustes, grabar });
  const enCurso = fase === FaseJuego.Sonando || fase === FaseJuego.Pausada;

  if (fase === FaseJuego.Error) {
    return (
      <EstadoContenido
        tipo="error"
        mensaje="La partida se detuvo."
        detalle={juego.error}
        accion={
          <Stack direction="row" spacing={1.5}>
            <Button variant="contained" startIcon={<ReplayIcon />} onClick={juego.empezar}>
              Reintentar
            </Button>
            <Button onClick={juego.salir}>Volver a la biblioteca</Button>
          </Stack>
        }
      />
    );
  }

  return (
    <Stack spacing={2} sx={{ height: "calc(100vh - 48px)", minHeight: 560 }}>
      <Stack direction="row" spacing={2} sx={{ alignItems: "center" }}>
        <Stack sx={{ flexGrow: 1, minWidth: 0 }}>
          <Typography variant="h5" component="h1" noWrap>
            {cancion.titulo}
          </Typography>
          <Typography variant="body2" sx={{ color: "text.secondary" }} noWrap>
            {cancion.artista}
          </Typography>
        </Stack>
        {ajustes.grabar && enCurso && (
          <Chip icon={<FiberManualRecordIcon />} label="Grabando" color="error" variant="outlined" />
        )}
        <Chip label={modo === Modo.Practica ? "Práctica" : "Puntuación"} variant="outlined" />
        {fase === FaseJuego.Sonando && (
          <Button variant="outlined" startIcon={<PauseIcon />} onClick={juego.pausar} disabled={juego.ocupado}>
            Pausar
          </Button>
        )}
        {fase === FaseJuego.Pausada && (
          <Button variant="contained" startIcon={<PlayArrowIcon />} onClick={juego.reanudar} disabled={juego.ocupado}>
            Reanudar
          </Button>
        )}
        <Button color="inherit" startIcon={<StopIcon />} onClick={juego.detener} disabled={fase === FaseJuego.Terminando}>
          {enCurso ? "Terminar" : "Salir"}
        </Button>
      </Stack>

      {juego.errorControl !== null && <Alert severity="warning">{juego.errorControl}</Alert>}
      {fase === FaseJuego.FinPractica && (
        <AvisoGrabacion
          grabacion={juego.grabacion}
          error={juego.errorGrabacion}
          errorAccion={acciones.error}
          onAbrir={acciones.abrir}
          onMostrar={acciones.mostrar}
        />
      )}

      {modo === Modo.Puntuacion && <MarcadorJuego fuente={juego.fuente} />}

      <Box sx={{ position: "relative", flexGrow: 1, minHeight: 240, borderRadius: 3, overflow: "hidden" }}>
        <LienzoJuego escena={juego.escena} fuente={juego.fuente} />
        <VeloFase
          fase={fase}
          ocupado={juego.ocupado || guardado.isPending}
          grabar={ajustes.grabar}
          onGrabar={fijarGrabar}
          onEmpezar={juego.empezar}
          onSalir={juego.salir}
        />
      </Box>

      <LetraJuego
        lineas={cancion.lineas}
        idioma={cancion.idioma}
        mostrarRomaji={ajustes.mostrar_romaji}
        fuente={juego.fuente}
      />

      {modo === Modo.Practica && (
        <BarraPosicion
          fuente={juego.fuente}
          duracion={cancion.duracion}
          deshabilitada={!enCurso}
          onBuscar={juego.buscar}
        />
      )}
    </Stack>
  );
};

interface VeloFaseProps {
  fase: FaseJuego;
  ocupado: boolean;
  grabar: boolean;
  onGrabar: (grabar: boolean) => void;
  onEmpezar: () => void;
  onSalir: () => void;
}

/** Capa sobre el lienzo antes de empezar, al arrancar y al terminar una práctica. */
const VeloFase = ({ fase, ocupado, grabar, onGrabar, onEmpezar, onSalir }: VeloFaseProps) => {
  if (fase !== FaseJuego.Lista && fase !== FaseJuego.Arrancando && fase !== FaseJuego.Terminando
    && fase !== FaseJuego.FinPractica) {
    return null;
  }
  return (
    <Stack
      spacing={2}
      sx={{
        position: "absolute",
        inset: 0,
        alignItems: "center",
        justifyContent: "center",
        bgcolor: "juego.velo",
      }}
    >
      {fase === FaseJuego.Lista && (
        <>
          <Typography variant="body1" sx={{ color: "text.secondary", maxWidth: 420, textAlign: "center" }}>
            Usa auriculares: si la música entra al micrófono, se mezcla con tu voz.
          </Typography>
          <FormControlLabel
            control={<Switch checked={grabar} onChange={(e) => onGrabar(e.target.checked)} disabled={ocupado} />}
            label="Grabar mi voz (se guarda en Música/Karaoke)"
          />
          <Button size="large" variant="contained" startIcon={<PlayArrowIcon />} onClick={onEmpezar} disabled={ocupado}>
            Empezar
          </Button>
        </>
      )}
      {fase === FaseJuego.Arrancando && <CircularProgress />}
      {fase === FaseJuego.Terminando && (
        <>
          <CircularProgress />
          {grabar && <Typography variant="body2">Guardando la grabación…</Typography>}
        </>
      )}
      {fase === FaseJuego.FinPractica && (
        <>
          <Typography variant="h6">Fin de la práctica</Typography>
          <Stack direction="row" spacing={1.5}>
            <Button variant="contained" startIcon={<ReplayIcon />} onClick={onEmpezar}>
              Otra vez
            </Button>
            <Button onClick={onSalir}>Volver a la biblioteca</Button>
          </Stack>
        </>
      )}
    </Stack>
  );
};
