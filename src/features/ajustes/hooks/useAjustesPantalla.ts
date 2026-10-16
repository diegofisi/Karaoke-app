import { useEffect, useState } from "react";
import { useAjustes, useCalibrarLatencia, useGuardarAjustes, useMicrofonos } from "@/common/api/consultas";
import { mensajeDeError } from "@/common/helpers/errores";
import type { Ajustes, Microfono, ResultadoCalibracion } from "@/common/interfaces";

export interface MedidaPitido {
  indice: number;
  ms: number | null;
}

export interface Calibracion {
  enCurso: boolean;
  pitidoActual: number;
  total: number;
  medidas: MedidaPitido[];
  resultado: ResultadoCalibracion | null;
  error: string | null;
}

const CALIBRACION_INICIAL: Calibracion = {
  enCurso: false, pitidoActual: 0, total: 0, medidas: [], resultado: null, error: null,
};

export interface DatosAjustes {
  cargando: boolean;
  errorCarga: string | null;
  borrador: Ajustes | null;
  cambiado: boolean;
  microfonos: Microfono[];
  guardando: boolean;
  errorGuardar: string | null;
  guardadoOk: boolean;
  calibracion: Calibracion;
  cambiar: <K extends keyof Ajustes>(clave: K, valor: Ajustes[K]) => void;
  guardar: () => void;
  descartar: () => void;
  calibrar: () => void;
  usarLatenciaMedida: () => void;
}

/** Borrador local de los ajustes: se edita en pantalla y se guarda con un botón. */
export const useAjustesPantalla = (): DatosAjustes => {
  const ajustes = useAjustes();
  const microfonos = useMicrofonos();
  const guardado = useGuardarAjustes();
  const calibracionMut = useCalibrarLatencia();
  const [borrador, fijarBorrador] = useState<Ajustes | null>(null);
  const [calibracion, fijarCalibracion] = useState<Calibracion>(CALIBRACION_INICIAL);

  useEffect(() => {
    if (ajustes.data !== undefined && borrador === null) fijarBorrador(ajustes.data);
  }, [ajustes.data, borrador]);

  const cambiar = <K extends keyof Ajustes>(clave: K, valor: Ajustes[K]) => {
    guardado.reset();
    fijarBorrador((actual) => (actual === null ? actual : { ...actual, [clave]: valor }));
  };

  const calibrar = () => {
    fijarCalibracion({ ...CALIBRACION_INICIAL, enCurso: true });
    calibracionMut.mutate(
      (evento) => {
        fijarCalibracion((actual) =>
          evento.tipo === "pitido"
            ? { ...actual, pitidoActual: evento.indice + 1, total: evento.total }
            : { ...actual, medidas: [...actual.medidas, { indice: evento.indice, ms: evento.ms }] },
        );
      },
      {
        onSuccess: (resultado) => fijarCalibracion((actual) => ({ ...actual, enCurso: false, resultado })),
        onError: (error) =>
          fijarCalibracion((actual) => ({ ...actual, enCurso: false, error: mensajeDeError(error) })),
      },
    );
  };

  const cambiado = borrador !== null && ajustes.data !== undefined
    && JSON.stringify(borrador) !== JSON.stringify(ajustes.data);

  return {
    cargando: ajustes.isPending,
    errorCarga: ajustes.isError ? mensajeDeError(ajustes.error) : null,
    borrador,
    cambiado,
    microfonos: microfonos.data ?? [],
    guardando: guardado.isPending,
    errorGuardar: guardado.isError ? mensajeDeError(guardado.error) : null,
    guardadoOk: guardado.isSuccess && !cambiado,
    calibracion,
    cambiar,
    guardar: () => {
      if (borrador !== null) guardado.mutate(borrador);
    },
    descartar: () => {
      guardado.reset();
      fijarBorrador(ajustes.data ?? null);
    },
    calibrar,
    usarLatenciaMedida: () => {
      if (calibracion.resultado !== null) cambiar("latencia_ms", Math.round(calibracion.resultado.latencia_ms));
    },
  };
};
