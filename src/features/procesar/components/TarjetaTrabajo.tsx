import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Chip from "@mui/material/Chip";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import CloseIcon from "@mui/icons-material/Close";
import EmojiEventsIcon from "@mui/icons-material/EmojiEvents";
import SchoolIcon from "@mui/icons-material/School";
import StopCircleIcon from "@mui/icons-material/StopCircle";
import { Modo } from "@/common/interfaces";
import { EstadoTrabajo, estaTerminado, type Trabajo } from "@/common/stores/procesos";
import { NOMBRE_ETAPA } from "../helpers/formulario";
import { LineaEtapas } from "./LineaEtapas";

export interface TarjetaTrabajoProps {
  trabajo: Trabajo;
  cancelando: boolean;
  onCancelar: (idCancion: string) => void;
  onQuitar: (clave: string) => void;
  onCantar: (idCancion: string, modo: Modo) => void;
}

const ETIQUETA_ESTADO: Record<EstadoTrabajo, { texto: string; color: "default" | "primary" | "success" | "error" }> = {
  [EstadoTrabajo.Arrancando]: { texto: "Arrancando", color: "default" },
  [EstadoTrabajo.EnCurso]: { texto: "En curso", color: "primary" },
  [EstadoTrabajo.Listo]: { texto: "Lista", color: "success" },
  [EstadoTrabajo.Error]: { texto: "Error", color: "error" },
  [EstadoTrabajo.Cancelado]: { texto: "Cancelada", color: "default" },
};

export const TarjetaTrabajo = ({ trabajo, cancelando, onCancelar, onQuitar, onCantar }: TarjetaTrabajoProps) => {
  const etiqueta = ETIQUETA_ESTADO[trabajo.estado];
  const terminado = estaTerminado(trabajo.estado);
  const idCancion = trabajo.idCancion;

  return (
    <Card>
      <CardContent>
        <Stack spacing={2}>
          <Stack direction="row" spacing={1.5} sx={{ alignItems: "center" }}>
            <Typography variant="subtitle1" sx={{ fontWeight: 700, flexGrow: 1, minWidth: 0, wordBreak: "break-all" }}>
              {trabajo.nombre}
            </Typography>
            <Chip size="small" label={etiqueta.texto} color={etiqueta.color} />
          </Stack>

          <LineaEtapas
            etapas={trabajo.etapas}
            etapaActual={trabajo.etapaActual}
            etapaError={trabajo.etapaError}
            activo={!terminado}
          />

          {trabajo.avisos.length > 0 && (
            <Alert severity="warning" variant="outlined">
              <Stack component="ul" spacing={0.5} sx={{ m: 0, pl: 2 }}>
                {trabajo.avisos.map((aviso, posicion) => (
                  // Los avisos no tienen id y solo se añaden al final: la posición es estable.
                  <Typography component="li" variant="body2" key={`${posicion}-${aviso}`}>
                    {aviso}
                  </Typography>
                ))}
              </Stack>
            </Alert>
          )}

          {trabajo.estado === EstadoTrabajo.Error && (
            <Alert severity="error">
              {trabajo.etapaError !== null && `${NOMBRE_ETAPA[trabajo.etapaError]}: `}
              {trabajo.error}
            </Alert>
          )}

          <Stack direction="row" spacing={1.5} useFlexGap sx={{ flexWrap: "wrap" }}>
            {trabajo.estado === EstadoTrabajo.Listo && idCancion !== null && (
              <>
                <Button
                  variant="contained"
                  startIcon={<EmojiEventsIcon />}
                  onClick={() => onCantar(idCancion, Modo.Puntuacion)}
                >
                  Cantar
                </Button>
                <Button variant="outlined" startIcon={<SchoolIcon />} onClick={() => onCantar(idCancion, Modo.Practica)}>
                  Practicar
                </Button>
              </>
            )}
            {!terminado && (
              <Button
                color="error"
                variant="outlined"
                startIcon={<StopCircleIcon />}
                disabled={idCancion === null || cancelando}
                onClick={() => {
                  if (idCancion !== null) onCancelar(idCancion);
                }}
              >
                {cancelando ? "Cancelando…" : "Cancelar"}
              </Button>
            )}
            {terminado && (
              <Button startIcon={<CloseIcon />} onClick={() => onQuitar(trabajo.clave)}>
                Quitar de la lista
              </Button>
            )}
          </Stack>
        </Stack>
      </CardContent>
    </Card>
  );
};
