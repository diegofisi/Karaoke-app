import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useBorrarCancion, useCanciones, useEstadoFabrica } from "@/common/api/consultas";
import type { EstadoFabrica, Modo } from "@/common/interfaces";
import { mensajeDeError } from "@/common/helpers/errores";
import { RUTAS, rutaJuego } from "@/common/routes/rutas";
import { aFilaCancion } from "../helpers/filas";
import type { FilaCancion } from "../interfaces/filaCancion";

export interface DatosBiblioteca {
  filas: FilaCancion[];
  cargando: boolean;
  error: string | null;
  estadoFabrica: EstadoFabrica | undefined;
  errorFabrica: string | null;
  paraCantar: FilaCancion | null;
  abrirCantar: (id: string) => void;
  cerrarCantar: () => void;
  cantar: (modo: Modo) => void;
  paraBorrar: FilaCancion | null;
  abrirBorrar: (id: string) => void;
  cerrarBorrar: () => void;
  confirmarBorrar: () => void;
  borrando: boolean;
  errorBorrar: string | null;
  irAProcesar: () => void;
  reintentar: () => void;
}

export const useBiblioteca = (): DatosBiblioteca => {
  const navegar = useNavigate();
  const consulta = useCanciones();
  const fabrica = useEstadoFabrica();
  const borrado = useBorrarCancion();
  const [idCantar, fijarIdCantar] = useState<string | null>(null);
  const [idBorrar, fijarIdBorrar] = useState<string | null>(null);

  const filas = (consulta.data ?? []).map(aFilaCancion);
  const buscar = (id: string | null) => (id === null ? null : (filas.find((fila) => fila.id === id) ?? null));

  const paraCantar = buscar(idCantar);
  const paraBorrar = buscar(idBorrar);

  return {
    filas,
    cargando: consulta.isLoading,
    error: consulta.isError ? mensajeDeError(consulta.error) : null,
    estadoFabrica: fabrica.data,
    errorFabrica: fabrica.isError ? mensajeDeError(fabrica.error) : null,
    paraCantar,
    abrirCantar: fijarIdCantar,
    cerrarCantar: () => fijarIdCantar(null),
    cantar: (modo) => {
      if (paraCantar === null) return;
      fijarIdCantar(null);
      void navegar(rutaJuego(paraCantar.id, modo));
    },
    paraBorrar,
    abrirBorrar: (id) => {
      borrado.reset();
      fijarIdBorrar(id);
    },
    cerrarBorrar: () => fijarIdBorrar(null),
    confirmarBorrar: () => {
      if (paraBorrar === null) return;
      borrado.mutate(paraBorrar.id, { onSuccess: () => fijarIdBorrar(null) });
    },
    borrando: borrado.isPending,
    errorBorrar: borrado.isError ? mensajeDeError(borrado.error) : null,
    irAProcesar: () => void navegar(RUTAS.procesar),
    reintentar: () => void consulta.refetch(),
  };
};
