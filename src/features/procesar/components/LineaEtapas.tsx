import Box from "@mui/material/Box";
import CircularProgress from "@mui/material/CircularProgress";
import LinearProgress from "@mui/material/LinearProgress";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import ErrorIcon from "@mui/icons-material/Error";
import RadioButtonUncheckedIcon from "@mui/icons-material/RadioButtonUnchecked";
import { ETAPAS_ORDENADAS, type Etapa } from "@/common/interfaces";
import type { ProgresoEtapa } from "@/common/stores/procesos";
import { NOMBRE_ETAPA } from "../helpers/formulario";

export interface LineaEtapasProps {
  etapas: Partial<Record<Etapa, ProgresoEtapa>>;
  etapaActual: Etapa | null;
  etapaError: Etapa | null;
  activo: boolean;
}

type EstadoEtapa = "pendiente" | "en_curso" | "hecha" | "fallida";

const estadoDe = (etapa: Etapa, props: LineaEtapasProps): EstadoEtapa => {
  if (props.etapaError === etapa) return "fallida";
  const progreso = props.etapas[etapa];
  if (props.activo && props.etapaActual === etapa) return "en_curso";
  if (progreso !== undefined && progreso.avance >= 1) return "hecha";
  return "pendiente";
};

const IconoEtapa = ({ estado }: { estado: EstadoEtapa }) => {
  if (estado === "hecha") return <CheckCircleIcon fontSize="small" sx={{ color: "success.main" }} />;
  if (estado === "fallida") return <ErrorIcon fontSize="small" sx={{ color: "error.main" }} />;
  if (estado === "en_curso") return <CircularProgress size={16} sx={{ m: "2px" }} />;
  return <RadioButtonUncheckedIcon fontSize="small" sx={{ color: "text.disabled" }} />;
};

const TEXTO_ESTADO: Record<EstadoEtapa, string> = {
  pendiente: "pendiente",
  en_curso: "en curso",
  hecha: "hecha",
  fallida: "falló",
};

/** Las siete etapas de la fábrica (CONTRATO.md §4) con su avance. */
export const LineaEtapas = (props: LineaEtapasProps) => (
  <Stack spacing={1.25} component="ol" sx={{ listStyle: "none", m: 0, p: 0 }}>
    {ETAPAS_ORDENADAS.map((etapa) => {
      const estado = estadoDe(etapa, props);
      const progreso = props.etapas[etapa];
      const avance = Math.round((progreso?.avance ?? 0) * 100);
      return (
        <Box component="li" key={etapa} aria-label={`${NOMBRE_ETAPA[etapa]}: ${TEXTO_ESTADO[estado]}`}>
          <Stack direction="row" spacing={1.5} sx={{ alignItems: "center" }}>
            <IconoEtapa estado={estado} />
            <Typography
              variant="body2"
              sx={{
                width: 190,
                flexShrink: 0,
                fontWeight: estado === "en_curso" ? 700 : 400,
                color: estado === "pendiente" ? "text.disabled" : "text.primary",
              }}
            >
              {NOMBRE_ETAPA[etapa]}
            </Typography>
            <LinearProgress
              variant="determinate"
              value={estado === "hecha" ? 100 : avance}
              color={estado === "fallida" ? "error" : estado === "hecha" ? "success" : "primary"}
              sx={{ flexGrow: 1, height: 6, borderRadius: 3 }}
            />
            <Typography variant="caption" sx={{ width: 40, textAlign: "right", color: "text.secondary" }}>
              {estado === "pendiente" ? "" : `${estado === "hecha" ? 100 : avance} %`}
            </Typography>
          </Stack>
          {estado === "en_curso" && progreso !== undefined && progreso.mensaje.length > 0 && (
            <Typography variant="caption" sx={{ color: "text.secondary", pl: 4.5, display: "block" }}>
              {progreso.mensaje}
            </Typography>
          )}
        </Box>
      );
    })}
  </Stack>
);
