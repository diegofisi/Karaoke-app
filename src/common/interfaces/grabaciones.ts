// Grabaciones de partidas (CONTRATO.md §5). Origen: crates/biblioteca/src/grabaciones.rs
// y src-tauri/src/tipos.rs (FinPartida).

import type { ResultadoJuego } from "./juego";

export interface Grabacion {
  /** Ruta absoluta del audio (mp3, o wav si no hubo ffmpeg). */
  ruta: string;
  nombre: string;
  id_cancion: string;
  titulo: string;
  artista: string;
  /** RFC 3339. */
  fecha: string;
  /** Segundos. */
  duracion: number;
}

/** Lo que devuelve `detener_juego`. */
export interface FinPartida {
  /** `null` en práctica. */
  resultado: ResultadoJuego | null;
  /** `null` si no se grababa o si falló (ver `error_grabacion`). */
  grabacion: Grabacion | null;
  error_grabacion: string | null;
}
