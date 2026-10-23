// Hooks de React Query, uno por comando (CONTRATO.md §5). La invalidación de
// la caché vive aquí porque es intrínseca al comando.

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type {
  Ajustes,
  EventoCalibracion,
  EventoFabrica,
  EventoInstalacion,
  EventoSesion,
  FrameJuego,
  Modo,
  PeticionProceso,
} from "@/common/interfaces";
import {
  abrirGrabacion,
  analizarUrl,
  borrarCancion,
  borrarGrabacion,
  cerrarSesionYoutube,
  cuentaYoutube,
  estadoSesionYoutube,
  iniciarSesionYoutube,
  buscarJuego,
  calibrarLatencia,
  cancelarProceso,
  detenerJuego,
  estadoFabrica,
  fijarVozGuia,
  guardarAjustes,
  instalarFabrica,
  iniciarJuego,
  listarCanciones,
  listarGrabaciones,
  listarMicrofonos,
  obtenerAjustes,
  mostrarGrabacion,
  obtenerCancion,
  pausarJuego,
  procesarCancion,
  reanudarJuego,
} from "./comandos";

export const CLAVES = {
  estadoFabrica: ["estado-fabrica"] as const,
  canciones: ["canciones"] as const,
  cancion: (id: string | null) => ["cancion", id] as const,
  microfonos: ["microfonos"] as const,
  ajustes: ["ajustes"] as const,
  grabaciones: ["grabaciones"] as const,
  sesionYoutube: ["sesion-youtube"] as const,
  cuentaYoutube: ["sesion-youtube", "cuenta"] as const,
};

// --- Consultas ---

export const useEstadoFabrica = () => useQuery({ queryKey: CLAVES.estadoFabrica, queryFn: estadoFabrica });

export const useCanciones = () => useQuery({ queryKey: CLAVES.canciones, queryFn: listarCanciones });

export const useCancion = (id: string | null) =>
  useQuery({
    queryKey: CLAVES.cancion(id),
    queryFn: () => obtenerCancion(id ?? ""),
    enabled: id !== null && id.length > 0,
    staleTime: Infinity,
  });

export const useMicrofonos = () => useQuery({ queryKey: CLAVES.microfonos, queryFn: listarMicrofonos });

export const useAjustes = () => useQuery({ queryKey: CLAVES.ajustes, queryFn: obtenerAjustes });

export const useGrabaciones = () => useQuery({ queryKey: CLAVES.grabaciones, queryFn: listarGrabaciones });

/** Vista previa de la URL escrita; se reintenta a mano, no sola. */
export const useAnalizarUrl = () => useMutation({ mutationFn: (url: string) => analizarUrl(url) });

export const useSesionYoutube = () => useQuery({ queryKey: CLAVES.sesionYoutube, queryFn: estadoSesionYoutube });

/** Nombre y foto solo con sesión (consulta a YouTube por red). */
export const useCuentaYoutube = (conSesion: boolean) =>
  useQuery({ queryKey: CLAVES.cuentaYoutube, queryFn: cuentaYoutube, enabled: conSesion, staleTime: 5 * 60_000 });

/** Abre el inicio de sesión. Al conectar o cerrar la ventana se vuelve a leer la sesión. */
export const useIniciarSesionYoutube = () => {
  const cliente = useQueryClient();
  return useMutation({
    mutationFn: (alEvento: (evento: EventoSesion) => void) =>
      iniciarSesionYoutube((evento) => {
        if (evento.tipo !== "error") void cliente.invalidateQueries({ queryKey: CLAVES.sesionYoutube });
        alEvento(evento);
      }),
  });
};

export const useCerrarSesionYoutube = () => {
  const cliente = useQueryClient();
  return useMutation({
    mutationFn: cerrarSesionYoutube,
    onSuccess: () => {
      cliente.removeQueries({ queryKey: CLAVES.cuentaYoutube });
      void cliente.invalidateQueries({ queryKey: CLAVES.sesionYoutube });
    },
  });
};

// --- Mutaciones ---

export const useBorrarCancion = () => {
  const cliente = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => borrarCancion(id),
    onSuccess: (_resultado, id) => {
      cliente.removeQueries({ queryKey: CLAVES.cancion(id) });
      void cliente.invalidateQueries({ queryKey: CLAVES.canciones });
    },
  });
};

export interface ArgumentosProceso {
  peticion: PeticionProceso;
  alEvento: (evento: EventoFabrica) => void;
  /** Recibe el id de la canción cuando el proceso arranca. */
  alArrancar: (id: string) => void;
  /** El proceso no llegó a arrancar. */
  alFallar: (error: unknown) => void;
}

/**
 * Arranca la fábrica. Al llegar `listo` por el canal se invalida la lista de
 * canciones. `alArrancar` y `alFallar` van en las opciones de `useMutation`,
 * no en `mutate`, para que se ejecuten aunque la pantalla se desmonte.
 */
export const useProcesarCancion = () => {
  const cliente = useQueryClient();
  return useMutation({
    mutationFn: ({ peticion, alEvento }: ArgumentosProceso) =>
      procesarCancion(peticion, (evento) => {
        if (evento.tipo === "listo") {
          void cliente.invalidateQueries({ queryKey: CLAVES.canciones });
          cliente.removeQueries({ queryKey: CLAVES.cancion(evento.id) });
        }
        alEvento(evento);
      }),
    onSuccess: (id, { alArrancar }) => alArrancar(id),
    onError: (error, { alFallar }) => alFallar(error),
  });
};

/** Al terminar (bien o mal) se vuelve a comprobar la fábrica. */
export const useInstalarFabrica = () => {
  const cliente = useQueryClient();
  return useMutation({
    mutationFn: (alEvento: (evento: EventoInstalacion) => void) => instalarFabrica(alEvento),
    onSettled: () => void cliente.invalidateQueries({ queryKey: CLAVES.estadoFabrica }),
  });
};

export const useCancelarProceso = () => useMutation({ mutationFn: (id: string) => cancelarProceso(id) });

export const useGuardarAjustes = () => {
  const cliente = useQueryClient();
  return useMutation({
    mutationFn: (ajustes: Ajustes) => guardarAjustes(ajustes),
    onSuccess: (guardados) => {
      cliente.setQueryData(CLAVES.ajustes, guardados);
      void cliente.invalidateQueries({ queryKey: CLAVES.ajustes });
      // Con otra carpeta de fábrica, Rust vuelve a hacer el diagnóstico.
      void cliente.invalidateQueries({ queryKey: CLAVES.estadoFabrica });
    },
  });
};

export const useCalibrarLatencia = () =>
  useMutation({ mutationFn: (alEvento: (evento: EventoCalibracion) => void) => calibrarLatencia(alEvento) });

export interface ArgumentosInicioJuego {
  id: string;
  modo: Modo;
  desde: number;
  alFrame: (frame: FrameJuego) => void;
}

export const useIniciarJuego = () =>
  useMutation({
    mutationFn: ({ id, modo, desde, alFrame }: ArgumentosInicioJuego) => iniciarJuego(id, modo, desde, alFrame),
  });

export const usePausarJuego = () => useMutation({ mutationFn: pausarJuego });

export const useReanudarJuego = () => useMutation({ mutationFn: reanudarJuego });

export const useFijarVozGuia = () => useMutation({ mutationFn: (volumen: number) => fijarVozGuia(volumen) });

export const useBuscarJuego = () => useMutation({ mutationFn: (t: number) => buscarJuego(t) });

/** Al terminar puede cambiar el récord y aparecer una grabación: se invalidan las dos listas. */
export const useDetenerJuego = () => {
  const cliente = useQueryClient();
  return useMutation({
    mutationFn: detenerJuego,
    onSuccess: (fin) => {
      if (fin?.resultado) void cliente.invalidateQueries({ queryKey: CLAVES.canciones });
      if (fin?.grabacion) void cliente.invalidateQueries({ queryKey: CLAVES.grabaciones });
    },
  });
};

export const useAbrirGrabacion = () => useMutation({ mutationFn: (ruta: string) => abrirGrabacion(ruta) });

export const useMostrarGrabacion = () => useMutation({ mutationFn: (ruta: string) => mostrarGrabacion(ruta) });

export const useBorrarGrabacion = () => {
  const cliente = useQueryClient();
  return useMutation({
    mutationFn: (ruta: string) => borrarGrabacion(ruta),
    onSuccess: () => void cliente.invalidateQueries({ queryKey: CLAVES.grabaciones }),
  });
};
