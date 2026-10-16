import Alert from "@mui/material/Alert";
import AlertTitle from "@mui/material/AlertTitle";
import type { EstadoFabrica } from "@/common/interfaces";

export interface AvisoFabricaProps {
  /** `undefined` mientras se consulta. */
  estado: EstadoFabrica | undefined;
  /** Mensaje de error si la consulta falló. */
  error: string | null;
}

/** Banner cuando la fábrica (Python) no está lista para procesar canciones. */
export const AvisoFabrica = ({ estado, error }: AvisoFabricaProps) => {
  if (error !== null) {
    return (
      <Alert severity="error" sx={{ mb: 3 }}>
        <AlertTitle>No se pudo comprobar la fábrica</AlertTitle>
        {error}
      </Alert>
    );
  }
  if (estado === undefined || estado.lista) return null;
  return (
    <Alert severity="warning" sx={{ mb: 3 }}>
      <AlertTitle>La fábrica de canciones no está lista</AlertTitle>
      {estado.mensaje ?? "No se pueden procesar canciones nuevas hasta que la fábrica esté instalada."}
      {estado.python !== null && ` Python ${estado.python}.`}
      {estado.cuda ? ` GPU: ${estado.gpu ?? "con CUDA"}.` : " Sin GPU con CUDA: el proceso será lento."}
    </Alert>
  );
};
