import { useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  useAnalizarUrl,
  useCancelarProceso,
  useEstadoFabrica,
  useIniciarSesionYoutube,
  useProcesarCancion,
} from "@/common/api/consultas";
import { olvidarFalloRenovacion, renovarSesionEnSilencio } from "@/common/api/sesionYoutube";
import { elegirArchivoAudio } from "@/common/api/dialogo";
import {
  CodigoErrorFabrica,
  type EstadoFabrica,
  type EventoFabrica,
  type Modo,
  type PeticionProceso,
  type VistaPrevia,
} from "@/common/interfaces";
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
import { MAXIMO_POR_LOTE, entradasDe, seleccionInicial } from "../helpers/vistaPrevia";
import { OrigenAudio, type CambiarCampo, type FormularioProceso } from "../interfaces/formulario";

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
  /** Vista previa de la URL (solo origen URL). */
  analizar: () => void;
  analizando: boolean;
  errorAnalisis: string | null;
  vista: VistaPrevia | null;
  seleccion: Set<string>;
  cambiarSeleccion: (url: string, seleccionado: boolean) => void;
  seleccionarTodos: (seleccionar: boolean) => void;
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
  /** Vuelve a procesar un trabajo con error. */
  reintentar: (clave: string) => void;
  /** Abre el inicio de sesión de YouTube; al conectar, reintenta el trabajo. */
  iniciarSesionYReintentar: (clave: string) => void;
  errorSesion: string | null;
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
  const reiniciar = useProcesosStore((estado) => estado.reiniciar);
  const inicioSesion = useIniciarSesionYoutube();
  const [errorSesion, fijarErrorSesion] = useState<string | null>(null);

  /** Lanza la fábrica para un trabajo ya creado en la cola. */
  const lanzar = (clave: string, peticion: PeticionProceso) => {
    const alEvento = (evento: EventoFabrica) => {
      aplicarEvento(clave, evento);
      // YouTube rechazó la sesión guardada: se renueva sola y se reintenta UNA vez.
      if (evento.tipo === "error" && evento.codigo === CodigoErrorFabrica.SesionRechazada) {
        const trabajo = useProcesosStore.getState().trabajos.find((t) => t.clave === clave);
        if (trabajo === undefined || trabajo.reintentadoSolo) return;
        void renovarSesionEnSilencio().then((renovada) => {
          if (!renovada) return;
          reiniciar(clave, true);
          lanzar(clave, peticion);
        });
      }
    };
    procesar.mutate({
      peticion,
      alEvento,
      alArrancar: (id) => fijarId(clave, id),
      alFallar: (error) => fallarArranque(clave, mensajeDeError(error)),
    });
  };

  const reintentar = (clave: string) => {
    const trabajo = useProcesosStore.getState().trabajos.find((t) => t.clave === clave);
    if (trabajo?.peticion == null) return;
    reiniciar(clave, false);
    lanzar(clave, trabajo.peticion);
  };

  const iniciarSesionYReintentar = (clave: string) => {
    fijarErrorSesion(null);
    inicioSesion.mutate((evento) => {
      if (evento.tipo === "conectada") {
        olvidarFalloRenovacion();
        reintentar(clave);
      } else if (evento.tipo === "error") {
        fijarErrorSesion(evento.mensaje);
      }
    }, { onError: (e) => fijarErrorSesion(mensajeDeError(e)) });
  };

  const [formulario, fijarFormulario] = useState<FormularioProceso>(FORMULARIO_VACIO);
  const [intentado, fijarIntentado] = useState(false);
  const [errorDialogo, fijarErrorDialogo] = useState<string | null>(null);
  const analisis = useAnalizarUrl();
  const [vista, fijarVista] = useState<VistaPrevia | null>(null);
  const [seleccion, fijarSeleccion] = useState<Set<string>>(new Set());
  /** URL de la vista previa actual: si el usuario la cambia, la vista deja de valer. */
  const [urlAnalizada, fijarUrlAnalizada] = useState("");
  /** La URL escrita ahora mismo, para descartar un análisis que llega tarde. */
  const urlActual = useRef(formulario.url);
  urlActual.current = formulario.url;

  const problema = problemaDeFormulario(formulario);
  // Bloqueado tambien mientras se comprueba o si la comprobacion falla.
  const fabricaNoLista = fabrica.data?.lista !== true;

  const cambiar: CambiarCampo = (campo, valor) => {
    fijarFormulario((anterior) => ({ ...anterior, [campo]: valor }));
    if (campo === "url" && String(valor).trim() !== urlAnalizada) {
      fijarVista(null);
      fijarSeleccion(new Set());
    }
  };

  const analizar = () => {
    fijarIntentado(true);
    const url = formulario.url.trim();
    if (problemaDeFormulario(formulario) !== null) return;
    fijarVista(null);
    analisis.mutate(url, {
      onSuccess: (r) => {
        // El usuario cambió la URL mientras se analizaba: esta vista ya no es la suya.
        if (urlActual.current.trim() !== url) return;
        fijarVista(r.vista);
        fijarSeleccion(seleccionInicial(r.vista));
        fijarUrlAnalizada(url);
        fijarIntentado(false);
        // La sesión guardada ya no vale: se renueva en silencio para la descarga.
        if (r.sesion_rechazada) void renovarSesionEnSilencio();
      },
    });
  };

  const cambiarSeleccion = (url: string, seleccionado: boolean) =>
    fijarSeleccion((s) => {
      const nueva = new Set(s);
      // Tope por lote también a mano: cada canción tarda minutos.
      if (seleccionado && nueva.size < MAXIMO_POR_LOTE) nueva.add(url);
      else if (!seleccionado) nueva.delete(url);
      return nueva;
    });

  const seleccionarTodos = (seleccionar: boolean) =>
    fijarSeleccion(vista !== null && seleccionar ? seleccionInicial(vista) : new Set());

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
    if (formulario.origen === OrigenAudio.Url) {
      // Sin vista previa todavía: el botón Procesar primero analiza.
      if (vista === null) {
        analizar();
        return;
      }
      // Un trabajo por vídeo elegido, con su título ya conocido.
      for (const video of entradasDe(vista)) {
        if (!seleccion.has(video.url)) continue;
        const clave = nuevaClaveTrabajo();
        const peticion = aPeticion(formulario, video.url);
        const nombre = video.canal ? `${video.canal} — ${video.titulo}` : video.titulo;
        crear(clave, nombre, peticion);
        lanzar(clave, peticion);
      }
      fijarVista(null);
      fijarSeleccion(new Set());
      fijarUrlAnalizada("");
    } else {
      const clave = nuevaClaveTrabajo();
      const peticion = aPeticion(formulario);
      crear(clave, nombreDeTrabajo(formulario), peticion);
      lanzar(clave, peticion);
    }
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
    analizar,
    analizando: analisis.isPending,
    errorAnalisis: analisis.isError ? mensajeDeError(analisis.error) : null,
    vista,
    seleccion,
    cambiarSeleccion,
    seleccionarTodos,
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
    reintentar,
    iniciarSesionYReintentar,
    errorSesion,
  };
};
