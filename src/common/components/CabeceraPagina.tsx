import type { ReactNode } from "react";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";

export interface CabeceraPaginaProps {
  titulo: string;
  subtitulo?: string;
  acciones?: ReactNode;
}

export const CabeceraPagina = ({ titulo, subtitulo, acciones }: CabeceraPaginaProps) => (
  <Stack direction="row" spacing={2} sx={{ alignItems: "flex-end", justifyContent: "space-between", mb: 3 }}>
    <Stack spacing={0.5}>
      <Typography variant="h4" component="h1">
        {titulo}
      </Typography>
      {subtitulo !== undefined && (
        <Typography variant="body1" sx={{ color: "text.secondary" }}>
          {subtitulo}
        </Typography>
      )}
    </Stack>
    {acciones}
  </Stack>
);
