import { useState } from "react";
import { useInstalarFabrica } from "@/common/api/consultas";
import { mensajeDeError } from "@/common/helpers/errores";

const LINEAS_VISIBLES = 6;

export interface Instalacion {
  enCurso: boolean;
  /** Paso actual (desde 1) y total; 0 antes de empezar. */
  paso: number;
  total: number;
  mensaje: string | null;
  /** Últimas líneas de salida, para que se vea que avanza. */
  lineas: string[];
  error: string | null;
  terminada: boolean;
  instalar: () => void;
}

/** Instalación de la fábrica desde la app (Procesar y Biblioteca la comparten). */
export const useInstalacionFabrica = (): Instalacion => {
  const instalacion = useInstalarFabrica();
  const [paso, fijarPaso] = useState(0);
  const [total, fijarTotal] = useState(0);
  const [mensaje, fijarMensaje] = useState<string | null>(null);
  const [lineas, fijarLineas] = useState<string[]>([]);

  return {
    enCurso: instalacion.isPending,
    paso,
    total,
    mensaje,
    lineas,
    error: instalacion.isError ? mensajeDeError(instalacion.error) : null,
    terminada: instalacion.isSuccess,
    instalar: () => {
      fijarPaso(0);
      fijarLineas([]);
      fijarMensaje("Preparando…");
      instalacion.mutate((evento) => {
        if (evento.tipo === "paso") {
          fijarPaso(evento.indice);
          fijarTotal(evento.total);
          fijarMensaje(evento.mensaje);
        } else {
          fijarLineas((actuales) => [...actuales, evento.texto].slice(-LINEAS_VISIBLES));
        }
      });
    },
  };
};
