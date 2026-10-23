// Vista previa de una URL de YouTube antes de procesar (CONTRATO.md §5).
// Origen: src-tauri/src/vista_previa.rs.

export interface VideoPrevio {
  id: string;
  url: string;
  titulo: string;
  /** Artista (del título "Artista - Título") o, si no, el canal. */
  canal: string;
  /** Segundos. */
  duracion: number | null;
  miniatura: string | null;
  /** `null` = disponible; si no: "privado", "necesita_sesion", "no_disponible"… */
  problema: string | null;
}

export type VistaPrevia =
  | ({ tipo: "video" } & VideoPrevio)
  | { tipo: "lista"; id: string; titulo: string; canal: string; entradas: VideoPrevio[] };

export interface VistaPreviaRespuesta {
  vista: VistaPrevia;
  /** YouTube rechazó la sesión guardada; la vista salió sin ella. */
  sesion_rechazada: boolean;
}
