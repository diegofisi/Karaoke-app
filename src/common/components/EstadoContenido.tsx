import type { ReactNode } from "react";
import Box from "@mui/material/Box";
import CircularProgress from "@mui/material/CircularProgress";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import ErrorOutlineIcon from "@mui/icons-material/ErrorOutlineOutlined";
import InboxOutlinedIcon from "@mui/icons-material/InboxOutlined";

export interface EstadoContenidoProps {
  tipo: "cargando" | "error" | "vacio";
  mensaje: string;
  detalle?: string | null;
  accion?: ReactNode;
}

/** Cargando, error o vacío dentro del área de contenido. */
export const EstadoContenido = ({ tipo, mensaje, detalle = null, accion }: EstadoContenidoProps) => (
  <Stack
    spacing={2}
    role={tipo === "error" ? "alert" : "status"}
    sx={{ alignItems: "center", justifyContent: "center", textAlign: "center", py: 8, px: 2 }}
  >
    <Box sx={{ color: tipo === "error" ? "error.main" : "text.secondary" }}>
      {tipo === "cargando" && <CircularProgress color="inherit" size={36} />}
      {tipo === "error" && <ErrorOutlineIcon sx={{ fontSize: 44 }} />}
      {tipo === "vacio" && <InboxOutlinedIcon sx={{ fontSize: 44 }} />}
    </Box>
    <Typography variant="h6">{mensaje}</Typography>
    {detalle !== null && (
      <Typography variant="body2" sx={{ color: "text.secondary", maxWidth: 560 }}>
        {detalle}
      </Typography>
    )}
    {accion}
  </Stack>
);
