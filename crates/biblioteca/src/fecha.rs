//! Fecha UTC en RFC 3339 sin dependencias.

use std::time::{SystemTime, UNIX_EPOCH};

/// Ahora, en UTC: `AAAA-MM-DDTHH:MM:SSZ`.
pub fn ahora_rfc3339() -> String {
    let segundos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    rfc3339(segundos)
}

/// Segundos desde 1970-01-01 a texto RFC 3339 (UTC).
pub(crate) fn rfc3339(segundos: u64) -> String {
    let dias = (segundos / 86_400) as i64;
    let resto = segundos % 86_400;
    let (anio, mes, dia) = civil_desde_dias(dias);
    format!(
        "{anio:04}-{mes:02}-{dia:02}T{:02}:{:02}:{:02}Z",
        resto / 3600,
        resto / 60 % 60,
        resto % 60
    )
}

/// Dias desde 1970-01-01 a (anio, mes, dia) del calendario gregoriano
/// (algoritmo `civil_from_days` de Howard Hinnant).
fn civil_desde_dias(dias: i64) -> (i64, u32, u32) {
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dia = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let anio = yoe + era * 400 + i64::from(mes <= 2);
    (anio, mes, dia)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn fechas_conocidas() {
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(rfc3339(1_791_995_400), "2026-10-14T16:30:00Z");
        assert_eq!(rfc3339(4_102_444_799), "2099-12-31T23:59:59Z");
    }
}
