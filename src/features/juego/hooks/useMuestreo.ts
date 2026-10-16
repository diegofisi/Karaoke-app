import { useEffect, useRef, useState } from "react";

/** 10 Hz: lo máximo a lo que se re-renderiza algo de la pantalla del juego. */
export const INTERVALO_MUESTREO_MS = 100;

/**
 * Lee un valor que cambia sin parar (la fuente de frames) a baja frecuencia y
 * solo re-renderiza cuando el valor leído cambia según `igual`.
 */
export const useMuestreo = <T>(leer: () => T, igual: (a: T, b: T) => boolean = Object.is): T => {
  const [valor, fijarValor] = useState<T>(leer);
  const leerRef = useRef(leer);
  const igualRef = useRef(igual);

  useEffect(() => {
    leerRef.current = leer;
    igualRef.current = igual;
  });

  useEffect(() => {
    const id = window.setInterval(() => {
      const nuevo = leerRef.current();
      fijarValor((anterior) => (igualRef.current(anterior, nuevo) ? anterior : nuevo));
    }, INTERVALO_MUESTREO_MS);
    return () => window.clearInterval(id);
  }, []);

  return valor;
};
