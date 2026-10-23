import { Etapa, type Etapa as TipoEtapa, type PeticionProceso } from "@/common/interfaces";
import { IDIOMA_AUTOMATICO, OrigenAudio, type FormularioProceso } from "../interfaces/formulario";

export const FORMULARIO_VACIO: FormularioProceso = {
  origen: OrigenAudio.Url,
  url: "",
  archivo: null,
  titulo: "",
  artista: "",
  idioma: IDIOMA_AUTOMATICO,
  letra: "",
};

// Vídeo, short, live o lista (playlist?list=…).
const PATRON_YOUTUBE =
  /^(https?:\/\/)?((www|m|music)\.)?(youtube\.com\/(watch\?(.*&)?v=|shorts\/|live\/|embed\/|playlist\?(.*&)?list=)|youtu\.be\/)[\w-]{6,}/i;

export const problemaDeUrl = (url: string): string | null => {
  const limpia = url.trim();
  if (limpia.length === 0) return "Pega la URL de un vídeo de YouTube.";
  if (!PATRON_YOUTUBE.test(limpia)) return "No parece una URL de YouTube (youtube.com/watch?v=… o youtu.be/…).";
  return null;
};

export const problemaDeFormulario = (formulario: FormularioProceso): string | null => {
  if (formulario.origen === OrigenAudio.Url) return problemaDeUrl(formulario.url);
  if (formulario.archivo === null) return "Elige un archivo de audio.";
  return null;
};

const textoONulo = (texto: string): string | null => {
  const limpio = texto.trim();
  return limpio.length > 0 ? limpio : null;
};

/** Petición con la forma exacta de `PeticionProceso` (CONTRATO.md §5): `null` en lo que no aplica.
 * Con URL, `urlVideo` es la del vídeo concreto elegido en la vista previa. */
export const aPeticion = (formulario: FormularioProceso, urlVideo: string | null = null): PeticionProceso => {
  const esUrl = formulario.origen === OrigenAudio.Url;
  return {
    url: esUrl ? (urlVideo ?? formulario.url.trim()) : null,
    archivo: esUrl ? null : formulario.archivo,
    titulo: esUrl ? null : textoONulo(formulario.titulo),
    artista: esUrl ? null : textoONulo(formulario.artista),
    idioma: formulario.idioma === IDIOMA_AUTOMATICO ? null : formulario.idioma,
    letra: formulario.letra.trim().length > 0 ? formulario.letra : null,
  };
};

/** Último trozo de una ruta de Windows o POSIX. */
export const nombreDeArchivo = (ruta: string): string => ruta.split(/[\\/]/).pop() ?? ruta;

export const nombreDeTrabajo = (formulario: FormularioProceso): string => {
  if (formulario.origen === OrigenAudio.Url) return formulario.url.trim();
  const titulo = textoONulo(formulario.titulo);
  if (titulo !== null) return titulo;
  return formulario.archivo === null ? "Archivo local" : nombreDeArchivo(formulario.archivo);
};

export const NOMBRE_ETAPA: Record<TipoEtapa, string> = {
  [Etapa.Descarga]: "Descarga",
  [Etapa.Separacion]: "Separación de la voz",
  [Etapa.Letra]: "Letra",
  [Etapa.Alineacion]: "Alineación por palabra",
  [Etapa.Tono]: "Tono de referencia",
  [Etapa.Notas]: "Notas",
  [Etapa.Guardado]: "Guardado",
};

let contadorTrabajos = 0;

/** Clave local del trabajo, única en la sesión. */
export const nuevaClaveTrabajo = (): string => {
  contadorTrabajos += 1;
  return `trabajo-${Date.now()}-${contadorTrabajos}`;
};
