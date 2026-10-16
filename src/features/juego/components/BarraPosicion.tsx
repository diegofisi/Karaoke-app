import { useState } from "react";
import Slider from "@mui/material/Slider";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { formatearTiempo } from "@/common/helpers/tiempo";
import { useMuestreo } from "../hooks/useMuestreo";
import { tiempoDeDibujo, type FuenteFrames } from "../stores/fuenteFrames";

export interface BarraPosicionProps {
  fuente: FuenteFrames;
  duracion: number;
  deshabilitada: boolean;
  onBuscar: (t: number) => void;
}

const mismoSegundoDecimo = (a: number, b: number): boolean => Math.abs(a - b) < 0.1;

/** Barra de posición del modo práctica: muestra el tiempo a 10 Hz y salta al soltar. */
export const BarraPosicion = ({ fuente, duracion, deshabilitada, onBuscar }: BarraPosicionProps) => {
  const posicion = useMuestreo(() => tiempoDeDibujo(fuente, performance.now()), mismoSegundoDecimo);
  const [arrastre, fijarArrastre] = useState<number | null>(null);
  const valor = Math.min(duracion, Math.max(0, arrastre ?? posicion));

  return (
    <Stack direction="row" spacing={2} sx={{ alignItems: "center", flexGrow: 1, minWidth: 220 }}>
      <Typography variant="body2" sx={{ fontVariantNumeric: "tabular-nums", color: "text.secondary" }}>
        {formatearTiempo(valor)}
      </Typography>
      <Slider
        aria-label="Posición en la canción"
        min={0}
        max={Math.max(duracion, 0.1)}
        step={0.1}
        value={valor}
        disabled={deshabilitada}
        valueLabelDisplay="auto"
        valueLabelFormat={formatearTiempo}
        onChange={(_evento, nuevo) => fijarArrastre(Array.isArray(nuevo) ? (nuevo[0] ?? 0) : nuevo)}
        onChangeCommitted={(_evento, nuevo) => {
          fijarArrastre(null);
          onBuscar(Array.isArray(nuevo) ? (nuevo[0] ?? 0) : nuevo);
        }}
      />
      <Typography variant="body2" sx={{ fontVariantNumeric: "tabular-nums", color: "text.secondary" }}>
        {formatearTiempo(duracion)}
      </Typography>
    </Stack>
  );
};
