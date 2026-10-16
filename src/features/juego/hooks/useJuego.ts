import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  useBuscarJuego,
  useDetenerJuego,
  useIniciarJuego,
  usePausarJuego,
  useReanudarJuego,
} from "@/common/api/consultas";
import {
  EstadoJuego,
  type Cancion,
  type EstadoJuego as TipoEstadoJuego,
  type Grabacion,
  type Modo,
} from "@/common/interfaces";
import { mensajeDeError } from "@/common/helpers/errores";
import { RUTAS, rutaResultados, type EstadoNavegacionResultados } from "@/common/routes/rutas";
import { crearEscena } from "../helpers/escena";
import type { EscenaJuego } from "../interfaces/escena";
import { FaseJuego } from "../interfaces/fase";
import {
  crearFuenteFrames,
  recibirFrame,
  reiniciarFuente,
  vaciarRastro,
  type FuenteFrames,
} from "../stores/fuenteFrames";

export interface DatosJuego {
  fase: FaseJuego;
  /** Error que para la partida (no arrancó o no se pudo detener). */
  error: string | null;
  /** Error de un control (pausa, reanudar, buscar); la partida sigue. */
  errorControl: string | null;
  escena: EscenaJuego;
  fuente: FuenteFrames;
  ocupado: boolean;
  /** Grabación de la última práctica (en puntuación va a la pantalla de resultados). */
  grabacion: Grabacion | null;
  errorGrabacion: string | null;
  empezar: () => void;
  pausar: () => void;
  reanudar: () => void;
  detener: () => void;
  buscar: (t: number) => void;
  salir: () => void;
}

type MotivoFin = "cancion_terminada" | "usuario";

/**
 * Orquesta una partida: abre el canal de frames, los escribe en la fuente
 * (sin estado de React) y solo cambia la fase cuando cambia el `estado` del
 * frame. El fin lo anuncia Rust con `estado: "terminado"`.
 */
export const useJuego = (cancion: Cancion, modo: Modo): DatosJuego => {
  const navegar = useNavigate();
  const [fuente] = useState(crearFuenteFrames);
  const escena = useMemo(() => crearEscena(cancion), [cancion]);
  const [fase, fijarFase] = useState<FaseJuego>(FaseJuego.Lista);
  const [error, fijarError] = useState<string | null>(null);
  const [errorControl, fijarErrorControl] = useState<string | null>(null);
  const [grabacion, fijarGrabacion] = useState<Grabacion | null>(null);
  const [errorGrabacion, fijarErrorGrabacion] = useState<string | null>(null);

  const inicio = useIniciarJuego();
  const pausa = usePausarJuego();
  const reanudacion = useReanudarJuego();
  const busqueda = useBuscarJuego();
  const detencion = useDetenerJuego();

  /** Cada partida tiene un número; los frames de una partida anterior se ignoran. */
  const partidaRef = useRef(0);
  const activaRef = useRef(false);
  const alEstadoRef = useRef<(estado: TipoEstadoJuego) => void>(() => undefined);
  const detenerAlSalirRef = useRef(detencion.mutate);

  const fallar = (causa: unknown) => {
    fijarError(mensajeDeError(causa));
    fijarFase(FaseJuego.Error);
  };

  const avisar = (causa: unknown) => fijarErrorControl(mensajeDeError(causa));

  const terminar = (motivo: MotivoFin) => {
    if (!activaRef.current) return;
    activaRef.current = false;
    partidaRef.current += 1;
    fijarFase(FaseJuego.Terminando);
    detencion.mutate(undefined, {
      onSuccess: (fin) => {
        const grabada = fin?.grabacion ?? null;
        const errorGrabada = fin?.error_grabacion ?? null;
        if (fin?.resultado) {
          const estado: EstadoNavegacionResultados = {
            resultado: fin.resultado,
            modo,
            grabacion: grabada,
            error_grabacion: errorGrabada,
          };
          void navegar(rutaResultados(cancion.id), { state: estado, replace: true });
        } else if (motivo === "usuario" && grabada === null && errorGrabada === null) {
          void navegar(RUTAS.biblioteca);
        } else {
          // Practica terminada (o detenida con grabacion): se queda para mostrarla.
          fijarGrabacion(grabada);
          fijarErrorGrabacion(errorGrabada);
          fijarFase(FaseJuego.FinPractica);
        }
      },
      onError: fallar,
    });
  };

  const alEstado = (estado: TipoEstadoJuego) => {
    if (estado === EstadoJuego.Sonando) fijarFase(FaseJuego.Sonando);
    else if (estado === EstadoJuego.Pausado) fijarFase(FaseJuego.Pausada);
    else terminar("cancion_terminada");
  };

  // El canal llama a la versión más reciente de `alEstado`, no a la del render en que se abrió.
  useEffect(() => {
    alEstadoRef.current = alEstado;
    detenerAlSalirRef.current = detencion.mutate;
  });

  // Al salir de la pantalla con la partida en marcha, se para en Rust.
  useEffect(
    () => () => {
      if (!activaRef.current) return;
      activaRef.current = false;
      partidaRef.current += 1;
      detenerAlSalirRef.current();
    },
    [],
  );

  const empezar = () => {
    partidaRef.current += 1;
    const partida = partidaRef.current;
    let estadoAnterior: TipoEstadoJuego | null = null;
    reiniciarFuente(fuente);
    fijarGrabacion(null);
    fijarErrorGrabacion(null);
    fijarError(null);
    fijarErrorControl(null);
    fijarFase(FaseJuego.Arrancando);
    activaRef.current = true;
    inicio.mutate(
      {
        id: cancion.id,
        modo,
        desde: 0,
        alFrame: (frame) => {
          if (partida !== partidaRef.current) return;
          recibirFrame(fuente, frame, performance.now());
          if (frame.estado !== estadoAnterior) {
            estadoAnterior = frame.estado;
            alEstadoRef.current(frame.estado);
          }
        },
      },
      {
        onError: (causa) => {
          activaRef.current = false;
          fallar(causa);
        },
      },
    );
  };

  return {
    fase,
    error,
    errorControl,
    escena,
    fuente,
    ocupado: inicio.isPending || pausa.isPending || reanudacion.isPending || detencion.isPending,
    grabacion,
    errorGrabacion,
    empezar,
    pausar: () => {
      fijarErrorControl(null);
      pausa.mutate(undefined, { onSuccess: () => fijarFase(FaseJuego.Pausada), onError: avisar });
    },
    reanudar: () => {
      fijarErrorControl(null);
      reanudacion.mutate(undefined, { onSuccess: () => fijarFase(FaseJuego.Sonando), onError: avisar });
    },
    detener: () => {
      if (activaRef.current) terminar("usuario");
      else void navegar(RUTAS.biblioteca);
    },
    buscar: (t) => {
      fijarErrorControl(null);
      vaciarRastro(fuente);
      busqueda.mutate(t, { onError: avisar });
    },
    salir: () => void navegar(RUTAS.biblioteca),
  };
};
