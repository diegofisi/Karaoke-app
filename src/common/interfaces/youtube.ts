// Sesión de YouTube (CONTRATO.md §5 y §6). Origen: src-tauri/src/youtube.rs y
// src-tauri/src/comandos/youtube.rs.

export const EstadoSesion = { Conectada: "conectada", Caducada: "caducada", Ninguna: "ninguna" } as const;
export type EstadoSesion = (typeof EstadoSesion)[keyof typeof EstadoSesion];

export interface CuentaYoutube {
  nombre: string;
  /** "@usuario". */
  usuario: string | null;
  /** data URL o URL de la foto. */
  foto: string | null;
}

/** Canal de `iniciar_sesion_youtube`. */
export type EventoSesion =
  | { tipo: "conectada" }
  | { tipo: "cerrada" }
  | { tipo: "error"; mensaje: string };

/** `codigo` del error de la fábrica (CONTRATO.md §4). */
export const CodigoErrorFabrica = { BloqueoYoutube: "bloqueo_youtube", SesionRechazada: "sesion_rechazada" } as const;
export type CodigoErrorFabrica = (typeof CodigoErrorFabrica)[keyof typeof CodigoErrorFabrica];
