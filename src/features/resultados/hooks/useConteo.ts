import { useEffect, useState } from "react";
import { suavizar } from "../helpers/textos";

/** Cuenta de 0 a `objetivo` en `duracionMs`. Solo se usa una vez por pantalla. */
export const useConteo = (objetivo: number, duracionMs = 1600): number => {
  const [valor, fijarValor] = useState(0);

  useEffect(() => {
    const inicio = performance.now();
    let id = 0;
    const paso = (ahora: number) => {
      const avance = suavizar((ahora - inicio) / duracionMs);
      fijarValor(Math.round(objetivo * avance));
      if (avance < 1) id = requestAnimationFrame(paso);
    };
    id = requestAnimationFrame(paso);
    return () => cancelAnimationFrame(id);
  }, [objetivo, duracionMs]);

  return valor;
};
