import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { PanelRequisitos } from "@/common/components/PanelRequisitos";
import { useInstalacionFabrica } from "@/common/hooks/useInstalacionFabrica";
import { CabeceraPagina } from "@/common/components/CabeceraPagina";
import { FormularioProceso } from "../components/FormularioProceso";
import { TarjetaTrabajo } from "../components/TarjetaTrabajo";
import { useProcesar } from "../hooks/useProcesar";

export const ProcesarPage = () => {
  const procesar = useProcesar();
  const instalacion = useInstalacionFabrica();

  return (
    <Box sx={{ maxWidth: 920 }}>
      <CabeceraPagina
        titulo="Procesar"
        subtitulo="La fábrica separa la voz, alinea la letra palabra por palabra y saca las notas. Tarda de 1 a 3 minutos con GPU."
      />
      <PanelRequisitos
        estado={procesar.estadoFabrica}
        error={procesar.errorFabrica}
        instalacion={instalacion}
        mostrarSiListo={true}
      />

      <Card sx={{ mb: 4 }}>
        <CardContent sx={{ p: 3 }}>
          <FormularioProceso
            formulario={procesar.formulario}
            cambiar={procesar.cambiar}
            problema={procesar.problema}
            mostrarProblema={procesar.mostrarProblema}
            onElegirArchivo={procesar.elegirArchivo}
            errorDialogo={procesar.errorDialogo}
            onEnviar={procesar.enviar}
            arrancando={procesar.arrancando}
            fabricaNoLista={procesar.fabricaNoLista}
          />
        </CardContent>
      </Card>

      <Stack direction="row" sx={{ alignItems: "center", justifyContent: "space-between", mb: 2 }}>
        <Typography variant="h5" component="h2">
          Cola
        </Typography>
        {procesar.hayTerminados && <Button onClick={procesar.quitarTerminados}>Quitar los terminados</Button>}
      </Stack>

      {procesar.errorCancelar !== null && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {procesar.errorCancelar}
        </Alert>
      )}

      {procesar.trabajos.length === 0 ? (
        <Typography variant="body2" sx={{ color: "text.secondary" }}>
          No hay canciones en proceso. Lo que empieces aquí sigue en marcha aunque cambies de pantalla.
        </Typography>
      ) : (
        <Stack spacing={2}>
          {procesar.trabajos.map((trabajo) => (
            <TarjetaTrabajo
              key={trabajo.clave}
              trabajo={trabajo}
              cancelando={trabajo.idCancion !== null && procesar.cancelando === trabajo.idCancion}
              onCancelar={procesar.cancelar}
              onQuitar={procesar.quitar}
              onCantar={procesar.cantar}
              onReintentar={procesar.reintentar}
              onIniciarSesion={procesar.iniciarSesionYReintentar}
              errorSesion={procesar.errorSesion}
            />
          ))}
        </Stack>
      )}
    </Box>
  );
};
