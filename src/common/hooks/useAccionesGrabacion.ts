import { useAbrirGrabacion, useMostrarGrabacion } from "@/common/api/consultas";
import { mensajeDeError } from "@/common/helpers/errores";

export interface AccionesGrabacion {
  abrir: (ruta: string) => void;
  mostrar: (ruta: string) => void;
  error: string | null;
}

/** Abrir y mostrar en carpeta: lo usan el juego, los resultados y Grabaciones. */
export const useAccionesGrabacion = (): AccionesGrabacion => {
  const abrir = useAbrirGrabacion();
  const mostrar = useMostrarGrabacion();
  const fallo = abrir.error ?? mostrar.error;
  return {
    abrir: (ruta) => abrir.mutate(ruta),
    mostrar: (ruta) => mostrar.mutate(ruta),
    error: fallo ? mensajeDeError(fallo) : null,
  };
};
