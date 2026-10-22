// Renovación silenciosa de la sesión de YouTube (adaptado de
// youtube-downloader: attemptSilentReconnect). Función normal, no un hook:
// la llama la cola de procesos fuera de React.

import { EstadoSesion } from "@/common/interfaces";
import { estadoSesionYoutube, obtenerAjustes, renovarSesionYoutube } from "./comandos";
import { CLAVES } from "./consultas";
import { queryClient } from "./queryClient";

/** Tras un intento fallido, durante este tiempo no se abre otra ventana oculta de 20 s. */
const ESPERA_TRAS_FALLO_MS = 60_000;

let enCurso: Promise<boolean> | null = null;
let ultimoFallo = 0;

/** Al iniciar sesión a mano, se puede volver a intentar enseguida. */
export const olvidarFalloRenovacion = (): void => {
  ultimoFallo = 0;
};

/**
 * Renueva la sesión sin que el usuario haga nada. Una sola a la vez (las
 * llamadas simultáneas comparten la promesa). Sin sesión guardada no hay nada
 * que renovar (tras «Cerrar sesión», por ejemplo): devuelve `false` sin abrir ventana.
 */
export const renovarSesionEnSilencio = (): Promise<boolean> => {
  if (enCurso !== null) return enCurso;
  if (Date.now() - ultimoFallo < ESPERA_TRAS_FALLO_MS) return Promise.resolve(false);
  enCurso = (async () => {
    const estado = await estadoSesionYoutube().catch(() => EstadoSesion.Ninguna);
    if (estado === EstadoSesion.Ninguna) return false;
    // Con un cookies.txt elegido a mano, ese tiene prioridad: renovar no cambiaria nada.
    const ajustes = await obtenerAjustes().catch(() => null);
    if (ajustes?.youtube_cookies != null) return false;
    const ok = await renovarSesionYoutube().catch(() => false);
    if (!ok) ultimoFallo = Date.now();
    void queryClient.invalidateQueries({ queryKey: CLAVES.sesionYoutube });
    return ok;
  })().finally(() => {
    enCurso = null;
  });
  return enCurso;
};
