import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useNavigate } from "react-router-dom";
import { CabeceraPagina } from "@/common/components/CabeceraPagina";
import { RUTAS } from "@/common/routes/rutas";
import { FormularioProceso } from "../components/FormularioProceso";
import { TarjetaTrabajo } from "../components/TarjetaTrabajo";
import { useProcesar } from "../hooks/useProcesar";

export const ProcesarPage = () => {
  const procesar = useProcesar();
  const navegar = useNavigate();
  const irAAjustes = () => void navegar(RUTAS.ajustes);
  const faltan = procesar.estadoFabrica?.requisitos.filter((r) => !r.ok) ?? [];

  return (
    <Box sx={{ maxWidth: 920 }}>
      <CabeceraPagina
        titulo="Procesar"
        subtitulo="La fábrica separa la voz, alinea la letra palabra por palabra y saca las notas. Tarda de 1 a 3 minutos con GPU."
      />
      {/* El detalle de requisitos y la instalación viven en Ajustes; aquí solo se avisa si falta algo. */}
      {procesar.errorFabrica !== null && (
        <Alert
          severity="error"
          sx={{ mb: 3 }}
          action={<Button color="inherit" size="small" onClick={irAAjustes}>Ir a Ajustes</Button>}
        >
          No se pudo comprobar lo necesario para procesar: {procesar.errorFabrica}
        </Alert>
      )}
      {procesar.estadoFabrica !== undefined && !procesar.estadoFabrica.lista && (
        <Alert
          severity="warning"
          sx={{ mb: 3 }}
          action={<Button color="inherit" size="small" onClick={irAAjustes}>Ir a Ajustes</Button>}
        >
          Falta algo para poder procesar: {faltan.map((r) => r.nombre).join(", ")}. En Ajustes → Fábrica se instala
          lo que falte.
        </Alert>
      )}

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
            onAnalizar={procesar.analizar}
            analizando={procesar.analizando}
            errorAnalisis={procesar.errorAnalisis}
            vista={procesar.vista}
            seleccion={procesar.seleccion}
            onCambiarSeleccion={procesar.cambiarSeleccion}
            onSeleccionarTodos={procesar.seleccionarTodos}
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
