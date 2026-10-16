//! Estado compartido de la app. Ningun lock se sostiene durante I/O ni
//! mientras corre un proceso: se toma, se copia o se mueve lo necesario, y se
//! suelta.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard};

use biblioteca::{Ajustes, Biblioteca};

use crate::juego::Partida;
use crate::tipos::EstadoFabrica;

/// Un proceso de la fabrica. `pid` es `None` mientras arranca: la entrada se
/// reserva antes de lanzar el proceso para que dos peticiones iguales no
/// lancen dos fabricas sobre la misma carpeta.
pub struct Trabajo {
    pub pid: Option<u32>,
    pub cancelado: Arc<AtomicBool>,
}

/// Quien usa el audio (salida + microfono). Juego y calibracion se excluyen:
/// un solo lock decide, sin carreras entre "comprobar" y "ocupar".
pub enum UsoAudio {
    Libre,
    /// `iniciar_juego` esta abriendo el audio fuera del lock.
    Arrancando,
    Jugando(Partida),
    Calibrando,
}

impl UsoAudio {
    /// Libre, o una partida cuyo hilo ya acabo (la interfaz aun no la detuvo).
    pub fn disponible(&self) -> bool {
        match self {
            UsoAudio::Libre => true,
            UsoAudio::Jugando(p) => p.hilo.is_finished(),
            UsoAudio::Arrancando | UsoAudio::Calibrando => false,
        }
    }
}

pub struct Estado {
    pub biblioteca: Biblioteca,
    /// Musica/Karaoke del usuario (CONTRATO seccion 5, grabaciones).
    pub carpeta_grabaciones: PathBuf,
    pub ajustes: Mutex<Ajustes>,
    pub trabajos: Mutex<HashMap<String, Trabajo>>,
    pub audio: Mutex<UsoAudio>,
    /// Resultado de `diagnostico`, una vez por sesion.
    pub diagnostico: Mutex<Option<EstadoFabrica>>,
}

impl Estado {
    pub fn new(biblioteca: Biblioteca, carpeta_grabaciones: PathBuf) -> Estado {
        let ajustes = biblioteca.leer_ajustes();
        Estado {
            biblioteca,
            carpeta_grabaciones,
            ajustes: Mutex::new(ajustes),
            trabajos: Mutex::new(HashMap::new()),
            audio: Mutex::new(UsoAudio::Libre),
            diagnostico: Mutex::new(None),
        }
    }
}

/// Lock con el error ya en espanol para la interfaz.
pub fn candado<T>(m: &Mutex<T>) -> Result<MutexGuard<'_, T>, String> {
    m.lock()
        .map_err(|_| "No se pudo usar el estado interno (un hilo fallo antes)".to_string())
}

/// Lock que, si el mutex quedo envenenado, sigue con el valor que haya. Solo
/// para limpiezas (liberar el audio, cerrar la app) que no deben fallar.
pub fn candado_limpieza<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}
