import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import Badge from "@mui/material/Badge";
import Box from "@mui/material/Box";
import List from "@mui/material/List";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import LibraryMusicIcon from "@mui/icons-material/LibraryMusic";
import AutoFixHighIcon from "@mui/icons-material/AutoFixHigh";
import TuneIcon from "@mui/icons-material/Tune";
import QueueMusicIcon from "@mui/icons-material/QueueMusic";
import MicIcon from "@mui/icons-material/Mic";
import { RUTAS } from "@/common/routes/rutas";

export const SeccionApp = {
  Biblioteca: "biblioteca",
  Procesar: "procesar",
  Grabaciones: "grabaciones",
  Ajustes: "ajustes",
} as const;
export type SeccionApp = (typeof SeccionApp)[keyof typeof SeccionApp];

export interface MarcoAppProps {
  seccionActiva: SeccionApp | null;
  trabajosActivos: number;
  children: ReactNode;
}

const ANCHO_BARRA_PX = 232;

/** Barra lateral con las tres secciones y el área de contenido. */
export const MarcoApp = ({ seccionActiva, trabajosActivos, children }: MarcoAppProps) => (
  <Box sx={{ display: "flex", minHeight: "100vh", bgcolor: "background.default" }}>
    <Box
      component="nav"
      aria-label="Secciones"
      sx={{
        width: ANCHO_BARRA_PX,
        flexShrink: 0,
        borderRight: 1,
        borderColor: "divider",
        bgcolor: "background.paper",
        position: "sticky",
        top: 0,
        height: "100vh",
      }}
    >
      <Stack direction="row" spacing={1.5} sx={{ alignItems: "center", px: 3, py: 3 }}>
        <MicIcon sx={{ color: "primary.main" }} />
        <Typography variant="h6" component="span">
          Karaoke
        </Typography>
      </Stack>
      <List sx={{ px: 1.5 }}>
        <ListItemButton
          component={Link}
          to={RUTAS.biblioteca}
          selected={seccionActiva === SeccionApp.Biblioteca}
          sx={{ borderRadius: 2, mb: 0.5 }}
        >
          <ListItemIcon>
            <LibraryMusicIcon />
          </ListItemIcon>
          <ListItemText primary="Biblioteca" />
        </ListItemButton>
        <ListItemButton
          component={Link}
          to={RUTAS.procesar}
          selected={seccionActiva === SeccionApp.Procesar}
          sx={{ borderRadius: 2, mb: 0.5 }}
        >
          <ListItemIcon>
            <Badge badgeContent={trabajosActivos} color="secondary" aria-label={`${trabajosActivos} en proceso`}>
              <AutoFixHighIcon />
            </Badge>
          </ListItemIcon>
          <ListItemText primary="Procesar" />
        </ListItemButton>
        <ListItemButton
          component={Link}
          to={RUTAS.grabaciones}
          selected={seccionActiva === SeccionApp.Grabaciones}
          sx={{ borderRadius: 2, mb: 0.5 }}
        >
          <ListItemIcon>
            <QueueMusicIcon />
          </ListItemIcon>
          <ListItemText primary="Grabaciones" />
        </ListItemButton>
        <ListItemButton
          component={Link}
          to={RUTAS.ajustes}
          selected={seccionActiva === SeccionApp.Ajustes}
          sx={{ borderRadius: 2 }}
        >
          <ListItemIcon>
            <TuneIcon />
          </ListItemIcon>
          <ListItemText primary="Ajustes" />
        </ListItemButton>
      </List>
    </Box>
    <Box component="main" sx={{ flexGrow: 1, minWidth: 0, px: { xs: 3, md: 5 }, py: 4 }}>
      {children}
    </Box>
  </Box>
);
