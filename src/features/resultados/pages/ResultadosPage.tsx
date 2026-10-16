import { useLocation, useNavigate, useParams } from "react-router-dom";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Chip from "@mui/material/Chip";
import LinearProgress from "@mui/material/LinearProgress";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import EmojiEventsIcon from "@mui/icons-material/EmojiEvents";
import ReplayIcon from "@mui/icons-material/Replay";
import { useCancion } from "@/common/api/consultas";
import { AvisoGrabacion } from "@/common/components/AvisoGrabacion";
import { EstadoContenido } from "@/common/components/EstadoContenido";
import { useAccionesGrabacion } from "@/common/hooks/useAccionesGrabacion";
import type { Calificacion, ResultadoJuego } from "@/common/interfaces";
import { RUTAS, rutaJuego, type EstadoNavegacionResultados } from "@/common/routes/rutas";
import { NOMBRE_CALIFICACION, NOMBRE_NIVEL } from "../helpers/textos";
import { useConteo } from "../hooks/useConteo";

const ORDEN: Calificacion[] = ["perfecta", "bien", "regular", "fallo"];

const esEstadoResultados = (valor: unknown): valor is EstadoNavegacionResultados =>
  typeof valor === "object" && valor !== null && "resultado" in valor && "modo" in valor;

/** El resultado llega por el estado de navegación desde el juego; no se vuelve a pedir a Rust. */
export const ResultadosPage = () => {
  const { id = "" } = useParams();
  const { state } = useLocation();
  const navegar = useNavigate();
  const cancion = useCancion(id);
  const acciones = useAccionesGrabacion();

  if (!esEstadoResultados(state)) {
    return (
      <EstadoContenido
        tipo="vacio"
        mensaje="No hay resultados que mostrar."
        detalle="Los resultados aparecen al terminar una partida en modo puntuación."
        accion={<Button onClick={() => void navegar(RUTAS.biblioteca)}>Ir a la biblioteca</Button>}
      />
    );
  }

  return (
    <Stack spacing={3} sx={{ maxWidth: 720, mx: "auto", py: 4 }}>
      <Stack spacing={0.5} sx={{ textAlign: "center" }}>
        <Typography variant="h5" component="h1">
          {cancion.data?.titulo ?? "Resultado"}
        </Typography>
        {cancion.data !== undefined && (
          <Typography variant="body2" sx={{ color: "text.secondary" }}>
            {cancion.data.artista}
          </Typography>
        )}
      </Stack>
      <Marcador resultado={state.resultado} />
      <AvisoGrabacion
        grabacion={state.grabacion ?? null}
        error={state.error_grabacion ?? null}
        errorAccion={acciones.error}
        onAbrir={acciones.abrir}
        onMostrar={acciones.mostrar}
      />
      <Desglose resultado={state.resultado} />
      <Stack direction="row" spacing={1.5} sx={{ justifyContent: "center" }}>
        <Button
          variant="contained"
          startIcon={<ReplayIcon />}
          onClick={() => void navegar(rutaJuego(id, state.modo), { replace: true })}
        >
          Repetir
        </Button>
        <Button onClick={() => void navegar(RUTAS.biblioteca)}>Biblioteca</Button>
      </Stack>
    </Stack>
  );
};

const Marcador = ({ resultado }: { resultado: ResultadoJuego }) => {
  const puntos = useConteo(resultado.puntos);
  return (
    <Paper sx={{ p: 4, textAlign: "center", borderRadius: 4 }}>
      <Typography variant="h2" component="p" sx={{ fontWeight: 700 }}>
        {puntos.toLocaleString("es")}
      </Typography>
      <Typography variant="h6" sx={{ color: "primary.main" }}>
        {NOMBRE_NIVEL[resultado.nivel]}
      </Typography>
      <Stack direction="row" spacing={1} sx={{ justifyContent: "center", mt: 2 }}>
        {resultado.es_record && <Chip icon={<EmojiEventsIcon />} label="¡Nuevo récord!" color="secondary" />}
        {!resultado.es_record && resultado.record_anterior !== null && (
          <Chip label={`Récord: ${resultado.record_anterior.toLocaleString("es")}`} variant="outlined" />
        )}
        <Chip label={`Mejor combo: ${resultado.mejor_combo}`} variant="outlined" />
        <Chip label={`Precisión: ${Math.round(resultado.precision_media * 100)} %`} variant="outlined" />
      </Stack>
    </Paper>
  );
};

const Desglose = ({ resultado }: { resultado: ResultadoJuego }) => {
  const total = ORDEN.reduce((suma, c) => suma + resultado.por_calificacion[c], 0);
  return (
    <Paper sx={{ p: 3, borderRadius: 4 }}>
      <Stack spacing={1.5}>
        {ORDEN.map((c) => {
          const n = resultado.por_calificacion[c];
          return (
            <Box key={c}>
              <Stack direction="row" sx={{ justifyContent: "space-between", mb: 0.5 }}>
                <Typography variant="body2">{NOMBRE_CALIFICACION[c]}</Typography>
                <Typography variant="body2" sx={{ color: "text.secondary" }}>
                  {n}
                </Typography>
              </Stack>
              <LinearProgress
                variant="determinate"
                value={total > 0 ? (n / total) * 100 : 0}
                sx={{ height: 8, borderRadius: 4, "& .MuiLinearProgress-bar": { bgcolor: `juego.${c}` } }}
              />
            </Box>
          );
        })}
      </Stack>
    </Paper>
  );
};
