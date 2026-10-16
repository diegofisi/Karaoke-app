// Espejo de `ResumenCancion` (CONTRATO.md §5).
// Origen: crates/karaoke-core/src/lib.rs (ResumenCancion) + `record`, que añade src-tauri.

import type { Idioma } from "./cancion";

export interface ResumenCancion {
  id: string;
  titulo: string;
  artista: string;
  duracion: number;
  idioma: Idioma;
  lineas_respaldo: number;
  lineas_total: number;
  record: number | null;
}
