import { Route, Routes, useLocation } from "react-router-dom";
import { MarcoApp, SeccionApp } from "@/common/components/MarcoApp";
import { RUTAS } from "@/common/routes/rutas";
import { estaTerminado, useProcesosStore } from "@/common/stores/procesos";
import { AjustesPage } from "@/features/ajustes/pages/AjustesPage";
import { BibliotecaPage } from "@/features/biblioteca/pages/BibliotecaPage";
import { GrabacionesPage } from "@/features/grabaciones/pages/GrabacionesPage";
import { JuegoPage } from "@/features/juego/pages/JuegoPage";
import { ProcesarPage } from "@/features/procesar/pages/ProcesarPage";
import { ResultadosPage } from "@/features/resultados/pages/ResultadosPage";

const seccionDe = (ruta: string): SeccionApp | null => {
  if (ruta === RUTAS.biblioteca) return SeccionApp.Biblioteca;
  if (ruta.startsWith(RUTAS.procesar)) return SeccionApp.Procesar;
  if (ruta.startsWith(RUTAS.ajustes)) return SeccionApp.Ajustes;
  if (ruta.startsWith(RUTAS.grabaciones)) return SeccionApp.Grabaciones;
  return null;
};

export const App = () => {
  const { pathname } = useLocation();
  const trabajosActivos = useProcesosStore((s) => s.trabajos.filter((t) => !estaTerminado(t.estado)).length);

  return (
    <MarcoApp seccionActiva={seccionDe(pathname)} trabajosActivos={trabajosActivos}>
      <Routes>
        <Route path={RUTAS.biblioteca} element={<BibliotecaPage />} />
        <Route path={RUTAS.procesar} element={<ProcesarPage />} />
        <Route path={RUTAS.ajustes} element={<AjustesPage />} />
        <Route path={RUTAS.grabaciones} element={<GrabacionesPage />} />
        <Route path={RUTAS.juego} element={<JuegoPage />} />
        <Route path={RUTAS.resultados} element={<ResultadosPage />} />
        <Route path="*" element={<BibliotecaPage />} />
      </Routes>
    </MarcoApp>
  );
};
