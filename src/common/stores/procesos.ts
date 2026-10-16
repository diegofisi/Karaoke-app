// Trabajos de la fábrica en curso y terminados. Vive fuera de React para que
// el progreso siga al cambiar de pantalla (los mensajes del canal llegan aquí
// aunque la pantalla Procesar no esté montada).

import { create } from "zustand";
import { ETAPAS_ORDENADAS, type Etapa, type EventoFabrica } from "@/common/interfaces";

export const EstadoTrabajo = {
  Arrancando: "arrancando",
  EnCurso: "en_curso",
  Listo: "listo",
  Error: "error",
  Cancelado: "cancelado",
} as const;
export type EstadoTrabajo = (typeof EstadoTrabajo)[keyof typeof EstadoTrabajo];

export interface ProgresoEtapa {
  avance: number;
  mensaje: string;
}

export interface Trabajo {
  /** Clave local; existe antes de que Rust devuelva el id. */
  clave: string;
  nombre: string;
  idCancion: string | null;
  estado: EstadoTrabajo;
  etapaActual: Etapa | null;
  etapas: Partial<Record<Etapa, ProgresoEtapa>>;
  avisos: string[];
  error: string | null;
  etapaError: Etapa | null;
}

interface ProcesosState {
  trabajos: Trabajo[];
}

interface ProcesosActions {
  crear: (clave: string, nombre: string) => void;
  fijarId: (clave: string, idCancion: string) => void;
  aplicarEvento: (clave: string, evento: EventoFabrica) => void;
  fallarArranque: (clave: string, mensaje: string) => void;
  quitar: (clave: string) => void;
  quitarTerminados: () => void;
  reset: () => void;
}

const estadoInicial: ProcesosState = { trabajos: [] };

/** `etapa` llega como texto abierto desde la fábrica: se valida, no se castea. */
const aEtapa = (valor: string): Etapa | null => ETAPAS_ORDENADAS.find((etapa) => etapa === valor) ?? null;

export const estaTerminado = (estado: EstadoTrabajo): boolean =>
  estado === EstadoTrabajo.Listo || estado === EstadoTrabajo.Error || estado === EstadoTrabajo.Cancelado;

const completarHasta = (
  etapas: Partial<Record<Etapa, ProgresoEtapa>>,
  limite: number,
): Partial<Record<Etapa, ProgresoEtapa>> => {
  const resultado = { ...etapas };
  ETAPAS_ORDENADAS.slice(0, limite).forEach((etapa) => {
    resultado[etapa] = { avance: 1, mensaje: resultado[etapa]?.mensaje ?? "" };
  });
  return resultado;
};

const siguienteTrabajo = (trabajo: Trabajo, evento: EventoFabrica): Trabajo => {
  if (estaTerminado(trabajo.estado)) return trabajo;
  switch (evento.tipo) {
    case "etapa": {
      const etapa = aEtapa(evento.etapa);
      if (etapa === null) return { ...trabajo, estado: EstadoTrabajo.EnCurso };
      const etapas = completarHasta(trabajo.etapas, ETAPAS_ORDENADAS.indexOf(etapa));
      etapas[etapa] = { avance: Math.min(1, Math.max(0, evento.avance)), mensaje: evento.mensaje };
      return { ...trabajo, estado: EstadoTrabajo.EnCurso, etapaActual: etapa, etapas };
    }
    case "aviso":
      return { ...trabajo, avisos: [...trabajo.avisos, evento.mensaje] };
    case "listo":
      return {
        ...trabajo,
        estado: EstadoTrabajo.Listo,
        idCancion: evento.id,
        etapaActual: null,
        etapas: completarHasta(trabajo.etapas, ETAPAS_ORDENADAS.length),
      };
    case "error":
      return {
        ...trabajo,
        estado: EstadoTrabajo.Error,
        error: evento.mensaje,
        etapaError: aEtapa(evento.etapa) ?? trabajo.etapaActual,
      };
    case "cancelado":
      return { ...trabajo, estado: EstadoTrabajo.Cancelado };
  }
};

const actualizar = (trabajos: Trabajo[], clave: string, cambio: (trabajo: Trabajo) => Trabajo): Trabajo[] =>
  trabajos.map((trabajo) => (trabajo.clave === clave ? cambio(trabajo) : trabajo));

export const useProcesosStore = create<ProcesosState & ProcesosActions>((set) => ({
  ...estadoInicial,
  crear: (clave, nombre) =>
    set((estado) => ({
      trabajos: [
        {
          clave,
          nombre,
          idCancion: null,
          estado: EstadoTrabajo.Arrancando,
          etapaActual: null,
          etapas: {},
          avisos: [],
          error: null,
          etapaError: null,
        },
        ...estado.trabajos,
      ],
    })),
  fijarId: (clave, idCancion) =>
    set((estado) => ({ trabajos: actualizar(estado.trabajos, clave, (trabajo) => ({ ...trabajo, idCancion })) })),
  aplicarEvento: (clave, evento) =>
    set((estado) => ({ trabajos: actualizar(estado.trabajos, clave, (trabajo) => siguienteTrabajo(trabajo, evento)) })),
  fallarArranque: (clave, mensaje) =>
    set((estado) => ({
      trabajos: actualizar(estado.trabajos, clave, (trabajo) => ({
        ...trabajo,
        estado: EstadoTrabajo.Error,
        error: mensaje,
      })),
    })),
  quitar: (clave) => set((estado) => ({ trabajos: estado.trabajos.filter((trabajo) => trabajo.clave !== clave) })),
  quitarTerminados: () =>
    set((estado) => ({ trabajos: estado.trabajos.filter((trabajo) => !estaTerminado(trabajo.estado)) })),
  reset: () => set(estadoInicial),
}));
