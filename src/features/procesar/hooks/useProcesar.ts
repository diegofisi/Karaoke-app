import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useCancelarProceso, useEstadoFabrica, useProcesarCancion } from "@/common/api/consultas";
import { elegirArchivoAudio } from "@/common/api/dialogo";
import type { EstadoFabrica, Modo } from "@/common/interfaces";
import { mensajeDeError } from "@/common/helpers/errores";
import { rutaJuego } from "@/common/routes/rutas";
import { estaTerminado, useProcesosStore, type Trabajo } from "@/common/stores/procesos";
import {
  FORMULARIO_VACIO,
  aPeticion,
  nombreDeTrabajo,
  nuevaClaveTrabajo,
  problemaDeFormulario,
} from "../helpers/formulario";
import type { CambiarCampo, FormularioProceso } from "../interfaces/formulario";

export interface DatosProcesar {
  formulario: FormularioProceso;
  cambiar: CambiarCampo;
  problema: string | null;
  /** El problema se enseña cuando el usuario ya escribió algo o intentó enviar. */
  mostrarProblema: boolean;
  elegirArchivo: () => void;
  errorDialogo: string | null;
  enviar: () => void;
  arrancando: boolean;
  fabricaNoLista: boolean;
  estadoFabrica: EstadoFabrica | undefined;
  errorFabrica: string | null;
  trabajos: Trabajo[];
  hayTerminados: boolean;
  cancelar: (idCancion: string) => void;
  cancelando: string | null;
  errorCancelar: string | null;
  quitar: (clave: string) => void;
  quitarTerminados: () => void;
  cantar: (idCancion: string, modo: Modo) => void;
}

export const useProcesar = (): DatosProcesar => {
  const navegar = useNavigate();
  const fabrica = useEstadoFabrica();
  const procesar = useProcesarCancion();
  const cancelacion = useCancelarProceso();

  const trabajos = useProcesosStore((estado) => estado.trabajos);
  const crear = useProcesosStore((estado) => estado.crear);
  const fijarId = useProcesosStore((estado) => estado.fijarId);
  const aplicarEvento = useProcesosStore((estado) => estado.aplicarEvento);
  const fallarArranque = useProcesosStore((estado) => estado.fallarArranque);
  const quitar = useProcesosStore((estado) => estado.quitar);
  const quitarTerminados = useProcesosStore((estado) => estado.quitarTerminados);

  const [formulario, fijarFormulario] = useState<FormularioProceso>(FORMULARIO_VACIO);
  const [intentado, fijarIntentado] = useState(false);
  const [errorDialogo, fijarErrorDialogo] = useState<string | null>(null);

  const problema = problemaDeFormulario(formulario);
  // Bloqueado tambien mientras se comprueba o si la comprobacion falla.
  const fabricaNoLista = fabrica.data?.lista !== true;

  const cambiar: CambiarCampo = (campo, valor) =>
    fijarFormulario((anterior) => ({ ...anterior, [campo]: valor }));

  const elegirArchivo = () => {
    fijarErrorDialogo(null);
    elegirArchivoAudio()
      .then((ruta) => {
        if (ruta !== null) fijarFormulario((anterior) => ({ ...anterior, archivo: ruta }));
      })
      .catch((error: unknown) => fijarErrorDialogo(mensajeDeError(error)));
  };

  const enviar = () => {
    fijarIntentado(true);
    if (problema !== null || fabricaNoLista) return;
    const clave = nuevaClaveTrabajo();
    crear(clave, nombreDeTrabajo(formulario));
    procesar.mutate({
      peticion: aPeticion(formulario),
      alEvento: (evento) => aplicarEvento(clave, evento),
      alArrancar: (id) => fijarId(clave, id),
      alFallar: (error) => fallarArranque(clave, mensajeDeError(error)),
    });
    fijarIntentado(false);
    fijarFormulario((anterior) => ({ ...FORMULARIO_VACIO, origen: anterior.origen, idioma: anterior.idioma }));
  };

  const datoUsuario =
    formulario.url.trim().length > 0 || formulario.archivo !== null || formulario.titulo.trim().length > 0;

  return {
    formulario,
    cambiar,
    problema,
    mostrarProblema: intentado || datoUsuario,
    elegirArchivo,
    errorDialogo,
    enviar,
    arrancando: procesar.isPending,
    fabricaNoLista,
    estadoFabrica: fabrica.data,
    errorFabrica: fabrica.isError ? mensajeDeError(fabrica.error) : null,
    trabajos,
    hayTerminados: trabajos.some((trabajo) => estaTerminado(trabajo.estado)),
    cancelar: (idCancion) => cancelacion.mutate(idCancion),
    cancelando: cancelacion.isPending ? (cancelacion.variables ?? null) : null,
    errorCancelar: cancelacion.isError ? mensajeDeError(cancelacion.error) : null,
    quitar,
    quitarTerminados,
    cantar: (idCancion, modo) => void navegar(rutaJuego(idCancion, modo)),
  };
};
