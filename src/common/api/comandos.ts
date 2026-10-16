// Envoltorios tipados de los comandos de Tauri (CONTRATO.md §5 y §6).
// Es el ÚNICO archivo con `invoke` y `new Channel`. Los argumentos van con el
// nombre exacto del parámetro de Rust (`rename_all = "snake_case"`). Los
// envoltorios no capturan errores: el rechazo es el `String` en español de Rust.

import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  Ajustes,
  Cancion,
  EstadoFabrica,
  EventoCalibracion,
  EventoFabrica,
  FinPartida,
  FrameJuego,
  Grabacion,
  Microfono,
  Modo,
  PeticionProceso,
  ResultadoCalibracion,
  ResumenCancion,
} from "@/common/interfaces";

export const estadoFabrica = (): Promise<EstadoFabrica> => invoke<EstadoFabrica>("estado_fabrica");

export const listarCanciones = (): Promise<ResumenCancion[]> => invoke<ResumenCancion[]>("listar_canciones");

export const obtenerCancion = (id: string): Promise<Cancion> => invoke<Cancion>("obtener_cancion", { id });

export const borrarCancion = (id: string): Promise<null> => invoke<null>("borrar_cancion", { id });

/** Devuelve el id de la canción en cuanto el proceso arranca; el resto llega por `alEvento`. */
export const procesarCancion = (
  peticion: PeticionProceso,
  alEvento: (evento: EventoFabrica) => void,
): Promise<string> => {
  const canal = new Channel<EventoFabrica>();
  canal.onmessage = alEvento;
  return invoke<string>("procesar_cancion", { peticion, canal });
};

export const cancelarProceso = (id: string): Promise<null> => invoke<null>("cancelar_proceso", { id });

export const listarMicrofonos = (): Promise<Microfono[]> => invoke<Microfono[]>("listar_microfonos");

export const obtenerAjustes = (): Promise<Ajustes> => invoke<Ajustes>("obtener_ajustes");

export const guardarAjustes = (ajustes: Ajustes): Promise<Ajustes> => invoke<Ajustes>("guardar_ajustes", { ajustes });

export const calibrarLatencia = (alEvento: (evento: EventoCalibracion) => void): Promise<ResultadoCalibracion> => {
  const canal = new Channel<EventoCalibracion>();
  canal.onmessage = alEvento;
  return invoke<ResultadoCalibracion>("calibrar_latencia", { canal });
};

export const iniciarJuego = (
  id: string,
  modo: Modo,
  desde: number,
  alFrame: (frame: FrameJuego) => void,
): Promise<null> => {
  const canal = new Channel<FrameJuego>();
  canal.onmessage = alFrame;
  return invoke<null>("iniciar_juego", { id, modo, desde, canal });
};

export const pausarJuego = (): Promise<null> => invoke<null>("pausar_juego");

export const reanudarJuego = (): Promise<null> => invoke<null>("reanudar_juego");

/** Solo en modo `practica`. */
export const buscarJuego = (t: number): Promise<null> => invoke<null>("buscar_juego", { t });

/** Espera también a que se guarde la grabación. `null` si no había partida. */
export const detenerJuego = (): Promise<FinPartida | null> => invoke<FinPartida | null>("detener_juego");

export const listarGrabaciones = (): Promise<Grabacion[]> => invoke<Grabacion[]>("listar_grabaciones");

/** La abre con el reproductor del sistema. */
export const abrirGrabacion = (ruta: string): Promise<null> => invoke<null>("abrir_grabacion", { ruta });

/** La muestra en el Explorador (Windows) o el Finder (Mac). */
export const mostrarGrabacion = (ruta: string): Promise<null> => invoke<null>("mostrar_grabacion", { ruta });

export const borrarGrabacion = (ruta: string): Promise<null> => invoke<null>("borrar_grabacion", { ruta });
