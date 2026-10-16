//! Enumeracion de entradas y busqueda de un dispositivo por id.

use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};

/// Un microfono seleccionable (`Microfono` en CONTRATO seccion 5).
///
/// `id` es el `DeviceId` estable de `cpal` 0.17 en texto (en WASAPI,
/// `wasapi:<id del endpoint>`): sobrevive a reinicios y no cambia si el
/// usuario renombra el dispositivo. Si `cpal` no da id, se usa el nombre.
/// [`crate::Microfono::abrir`] acepta tambien el nombre, para ajustes
/// guardados con un nombre.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DispositivoEntrada {
    pub id: String,
    pub nombre: String,
    pub por_defecto: bool,
}

/// Lista las entradas del host por defecto. Nunca falla: sin dispositivos (o
/// si la enumeracion falla) devuelve una lista vacia.
pub fn listar_entradas() -> Vec<DispositivoEntrada> {
    let host = cpal::default_host();
    let id_defecto = host.default_input_device().map(|d| id_de(&d));
    let Ok(dispositivos) = host.input_devices() else {
        return Vec::new();
    };
    dispositivos
        .map(|d| {
            let id = id_de(&d);
            DispositivoEntrada {
                nombre: nombre_de(&d).unwrap_or_else(|| id.clone()),
                por_defecto: id_defecto.as_deref() == Some(id.as_str()),
                id,
            }
        })
        .collect()
}

pub(crate) fn nombre_de(dispositivo: &cpal::Device) -> Option<String> {
    dispositivo.description().ok().map(|d| d.name().to_string())
}

pub(crate) fn id_de(dispositivo: &cpal::Device) -> String {
    match dispositivo.id() {
        Ok(id) => id.to_string(),
        Err(_) => nombre_de(dispositivo).unwrap_or_default(),
    }
}

/// Busca una entrada por id (o, si no hay coincidencia, por nombre).
pub(crate) fn buscar_entrada(host: &cpal::Host, id: &str) -> Option<cpal::Device> {
    let lista: Vec<cpal::Device> = host.input_devices().ok()?.collect();
    let por_id = lista.iter().position(|d| id_de(d) == id);
    let indice = por_id.or_else(|| {
        lista
            .iter()
            .position(|d| nombre_de(d).as_deref() == Some(id))
    })?;
    lista.into_iter().nth(indice)
}
