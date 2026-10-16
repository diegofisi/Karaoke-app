/** Segundos a `m:ss` (o `h:mm:ss` desde una hora). Valores no finitos o negativos cuentan como 0. */
export const formatearTiempo = (segundos: number): string => {
  const total = Number.isFinite(segundos) && segundos > 0 ? Math.floor(segundos) : 0;
  const horas = Math.floor(total / 3600);
  const minutos = Math.floor((total % 3600) / 60);
  const resto = total % 60;
  const ss = String(resto).padStart(2, "0");
  if (horas > 0) return `${horas}:${String(minutos).padStart(2, "0")}:${ss}`;
  return `${minutos}:${ss}`;
};

/** Número entero con separador de miles en español (12.345). */
export const formatearPuntos = (puntos: number): string =>
  Math.round(Number.isFinite(puntos) ? puntos : 0).toLocaleString("es-ES", { useGrouping: true });
