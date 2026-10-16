import { useEffect, useRef, useState } from "react";
import Box from "@mui/material/Box";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import HistoryToggleOffIcon from "@mui/icons-material/HistoryToggleOff";
import { Idioma, Metodo, type Idioma as TipoIdioma, type Linea, type Palabra } from "@/common/interfaces";
import { indiceLineaActual, progresoPalabra } from "../helpers/notas";
import { tiempoDeDibujo, type FuenteFrames } from "../stores/fuenteFrames";
import { PalabraLetra } from "./PalabraLetra";

export interface LetraJuegoProps {
  lineas: Linea[];
  idioma: TipoIdioma;
  mostrarRomaji: boolean;
  fuente: FuenteFrames;
}

/**
 * Línea actual y siguiente. React solo re-renderiza al cambiar de línea; el
 * barrido de cada palabra lo escribe el bucle de requestAnimationFrame como
 * variable CSS (`--avance`) directamente en el elemento.
 */
export const LetraJuego = ({ lineas, idioma, mostrarRomaji, fuente }: LetraJuegoProps) => {
  const [indice, fijarIndice] = useState(0);
  const palabrasActuales = useRef(new Map<HTMLElement, Palabra>());

  useEffect(() => {
    let id = 0;
    let indiceDibujado = -1;
    const pintar = (ahora: number) => {
      const t = tiempoDeDibujo(fuente, ahora);
      const nuevo = indiceLineaActual(lineas, t);
      if (nuevo !== indiceDibujado) {
        indiceDibujado = nuevo;
        fijarIndice(nuevo);
      }
      palabrasActuales.current.forEach((palabra, elemento) => {
        const avance = progresoPalabra(palabra.inicio, palabra.fin, t);
        elemento.style.setProperty("--avance", `${(avance * 100).toFixed(1)}%`);
      });
      id = requestAnimationFrame(pintar);
    };
    id = requestAnimationFrame(pintar);
    return () => cancelAnimationFrame(id);
  }, [lineas, fuente]);

  const registrar = (palabra: Palabra) => (elemento: HTMLElement | null) => {
    if (elemento === null) return undefined;
    const registro = palabrasActuales.current;
    registro.set(elemento, palabra);
    return () => {
      registro.delete(elemento);
    };
  };

  const japones = idioma === Idioma.Ja;
  const actual = lineas[indice];
  const siguiente = lineas[indice + 1];

  if (lineas.length === 0) {
    return (
      <Typography variant="body1" sx={{ color: "text.secondary", textAlign: "center", py: 3 }}>
        Esta canción no tiene letra.
      </Typography>
    );
  }

  return (
    <Stack spacing={1} sx={{ alignItems: "center", textAlign: "center", minHeight: 170, py: 2, px: 3 }}>
      {actual === undefined ? (
        <Typography variant="letraSiguiente" sx={{ color: "juego.letraSiguiente" }}>
          Fin de la letra
        </Typography>
      ) : (
        // `key` por línea: al cambiar de línea las palabras se crean de nuevo con `--avance` a 0.
        <Stack key={indice} spacing={0.5} sx={{ alignItems: "center" }}>
          <Stack direction="row" spacing={1.5} sx={{ alignItems: "center" }}>
            {actual.metodo === Metodo.Respaldo && (
              <Tooltip title="Tiempos aproximados: la alineación de esta línea falló.">
                <HistoryToggleOffIcon
                  aria-label="Línea con tiempos aproximados"
                  sx={{ color: "juego.respaldo", fontSize: 22 }}
                />
              </Tooltip>
            )}
            <Typography variant="letraActual" sx={{ color: "juego.letraPendiente" }}>
              {actual.palabras.length === 0
                ? actual.texto
                : actual.palabras.map((palabra, posicion) => (
                    <Box component="span" key={palabra.inicio}>
                      {posicion > 0 && !japones && " "}
                      <PalabraLetra palabra={palabra} conLectura={japones} refBarrido={registrar(palabra)} />
                    </Box>
                  ))}
            </Typography>
          </Stack>
          {mostrarRomaji && actual.romaji !== null && (
            <Typography variant="romaji" sx={{ color: "text.secondary" }}>
              {actual.romaji}
            </Typography>
          )}
        </Stack>
      )}
      {siguiente !== undefined && (
        <Typography variant="letraSiguiente" sx={{ color: "juego.letraSiguiente" }}>
          {siguiente.palabras.length === 0
            ? siguiente.texto
            : siguiente.palabras.map((palabra, posicion) => (
                <Box component="span" key={palabra.inicio}>
                  {posicion > 0 && !japones && " "}
                  <PalabraLetra palabra={palabra} conLectura={japones} />
                </Box>
              ))}
        </Typography>
      )}
    </Stack>
  );
};
