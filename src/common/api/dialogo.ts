// Diálogo de archivo del sistema (plugin `dialog` de Tauri). Necesita el
// permiso `dialog:allow-open` en src-tauri/capabilities/.

import { open } from "@tauri-apps/plugin-dialog";

const EXTENSIONES_AUDIO = ["mp3", "wav", "flac", "m4a", "aac", "ogg", "opus", "webm"];

/** Ruta absoluta del archivo elegido, o `null` si el usuario cancela. */
export const elegirArchivoAudio = async (): Promise<string | null> => {
  const eleccion = await open({
    title: "Elige un archivo de audio",
    multiple: false,
    directory: false,
    filters: [{ name: "Audio", extensions: EXTENSIONES_AUDIO }],
  });
  return typeof eleccion === "string" ? eleccion : null;
};
