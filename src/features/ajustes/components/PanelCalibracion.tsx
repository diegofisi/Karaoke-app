import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import Chip from "@mui/material/Chip";
import LinearProgress from "@mui/material/LinearProgress";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import GraphicEqIcon from "@mui/icons-material/GraphicEq";
import type { Calibracion } from "../hooks/useAjustesPantalla";

export interface PanelCalibracionProps {
  calibracion: Calibracion;
  latenciaActual: number;
  onCalibrar: () => void;
  onUsarMedida: () => void;
}

/** Asistente de calibración: suenan pitidos, el micrófono los oye, la mediana es la latencia. */
export const PanelCalibracion = ({ calibracion, latenciaActual, onCalibrar, onUsarMedida }: PanelCalibracionProps) => {
  const { enCurso, pitidoActual, total, medidas, resultado, error } = calibracion;
  return (
    <Stack spacing={1.5}>
      <Typography variant="body2" sx={{ color: "text.secondary" }}>
        Mide el retraso entre lo que suena y lo que oye el micrófono. Usa los altavoces a volumen normal (no
        auriculares) en un sitio silencioso. Sonarán unos pitidos.
      </Typography>
      <Stack direction="row" spacing={1.5} sx={{ alignItems: "center" }}>
        <Button variant="outlined" startIcon={<GraphicEqIcon />} onClick={onCalibrar} disabled={enCurso}>
          {enCurso ? "Calibrando…" : "Calibrar"}
        </Button>
        {enCurso && total > 0 && (
          <Typography variant="body2">
            Pitido {pitidoActual} de {total}
          </Typography>
        )}
      </Stack>
      {enCurso && <LinearProgress variant={total > 0 ? "determinate" : "indeterminate"} value={total > 0 ? (pitidoActual / total) * 100 : 0} />}
      {medidas.length > 0 && (
        <Stack direction="row" spacing={1} sx={{ flexWrap: "wrap", rowGap: 1 }}>
          {medidas.map((m) => (
            <Chip
              key={m.indice}
              size="small"
              variant="outlined"
              color={m.ms === null ? "warning" : "default"}
              label={m.ms === null ? "no se oyó" : `${Math.round(m.ms)} ms`}
            />
          ))}
        </Stack>
      )}
      {error !== null && <Alert severity="error">{error}</Alert>}
      {resultado !== null && (
        <Alert
          severity={resultado.fiable ? "success" : "warning"}
          action={
            <Button color="inherit" size="small" onClick={onUsarMedida}
              disabled={Math.round(resultado.latencia_ms) === Math.round(latenciaActual)}>
              Usar {Math.round(resultado.latencia_ms)} ms
            </Button>
          }
        >
          {resultado.fiable
            ? `Latencia medida: ${Math.round(resultado.latencia_ms)} ms.`
            : `Medida poco fiable (${Math.round(resultado.latencia_ms)} ms): sube el volumen o baja el ruido y repite.`}
        </Alert>
      )}
    </Stack>
  );
};
