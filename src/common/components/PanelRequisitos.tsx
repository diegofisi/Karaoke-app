import Alert from "@mui/material/Alert";
import AlertTitle from "@mui/material/AlertTitle";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import LinearProgress from "@mui/material/LinearProgress";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import CancelIcon from "@mui/icons-material/Cancel";
import WarningAmberIcon from "@mui/icons-material/WarningAmber";
import DownloadIcon from "@mui/icons-material/Download";
import type { EstadoFabrica } from "@/common/interfaces";
import type { Instalacion } from "@/common/hooks/useInstalacionFabrica";

export interface PanelRequisitosProps {
  /** `undefined` mientras se comprueba. */
  estado: EstadoFabrica | undefined;
  error: string | null;
  instalacion: Instalacion;
  /** Con todo en verde: `true` lo muestra igual (Procesar); `false` lo oculta (Biblioteca). */
  mostrarSiListo: boolean;
}

/** Lo que hace falta para procesar canciones, comprobado antes de dejar empezar. */
export const PanelRequisitos = ({ estado, error, instalacion, mostrarSiListo }: PanelRequisitosProps) => {
  if (error !== null) {
    return (
      <Alert severity="error" sx={{ mb: 3 }}>
        <AlertTitle>No se pudo comprobar lo necesario para procesar</AlertTitle>
        {error}
      </Alert>
    );
  }
  if (estado === undefined) {
    return (
      <Paper sx={{ p: 2.5, mb: 3, borderRadius: 3 }}>
        <Typography variant="body2" sx={{ mb: 1 }}>
          Comprobando lo necesario para procesar canciones…
        </Typography>
        <LinearProgress />
      </Paper>
    );
  }
  if (estado.lista && !mostrarSiListo && !instalacion.enCurso) return null;

  const falta = estado.requisitos.find((r) => r.id === "fabrica" && !r.ok);
  return (
    <Paper sx={{ p: 2.5, mb: 3, borderRadius: 3 }}>
      <Typography variant="subtitle1" sx={{ mb: 1.5 }}>
        {estado.lista ? "Todo listo para procesar" : "Falta algo para poder procesar canciones"}
      </Typography>
      <Stack spacing={1}>
        {estado.requisitos.map((r) => (
          <Stack key={r.id} direction="row" spacing={1.5} sx={{ alignItems: "flex-start" }}>
            <Box sx={{ pt: 0.25, color: r.ok ? "success.main" : r.obligatorio ? "error.main" : "warning.main" }}>
              {r.ok ? <CheckCircleIcon fontSize="small" /> : r.obligatorio ? <CancelIcon fontSize="small" /> : <WarningAmberIcon fontSize="small" />}
            </Box>
            <Box>
              <Typography variant="body2">{r.nombre}</Typography>
              <Typography variant="caption" sx={{ color: "text.secondary", wordBreak: "break-all" }}>
                {r.detalle}
              </Typography>
            </Box>
          </Stack>
        ))}
      </Stack>

      {(falta !== undefined || instalacion.enCurso) && estado.puede_instalar && (
        <Box sx={{ mt: 2.5 }}>
          {!instalacion.enCurso && (
            <>
              <Typography variant="body2" sx={{ color: "text.secondary", mb: 1.5 }}>
                La primera vez, la app descarga e instala sola Python, PyTorch y los modelos de IA (unos 13 GB) y
                puede tardar bastante según tu conexión. Actualizar la fábrica a una versión nueva tarda menos de un
                minuto. No hace falta instalar nada más.
              </Typography>
              <Button variant="contained" startIcon={<DownloadIcon />} onClick={instalacion.instalar}>
                Instalar la fábrica
              </Button>
            </>
          )}
          {instalacion.enCurso && (
            <Stack spacing={1}>
              <Typography variant="body2">
                {instalacion.total > 0 ? `Paso ${instalacion.paso} de ${instalacion.total}: ` : ""}
                {instalacion.mensaje}
              </Typography>
              <LinearProgress
                variant={instalacion.total > 0 ? "determinate" : "indeterminate"}
                value={instalacion.total > 0 ? ((instalacion.paso - 1) / instalacion.total) * 100 : 0}
              />
              <Box
                component="pre"
                sx={{ m: 0, p: 1, borderRadius: 1, bgcolor: "background.default", fontSize: 12, color: "text.secondary", whiteSpace: "pre-wrap", wordBreak: "break-all", minHeight: 60 }}
              >
                {instalacion.lineas.join("\n")}
              </Box>
            </Stack>
          )}
        </Box>
      )}
      {instalacion.error !== null && (
        <Alert severity="error" sx={{ mt: 2, whiteSpace: "pre-wrap" }}>
          {instalacion.error}
        </Alert>
      )}
      {instalacion.terminada && estado.lista && (
        <Alert severity="success" sx={{ mt: 2 }}>
          Fábrica instalada. Ya puedes procesar canciones.
        </Alert>
      )}
    </Paper>
  );
};
