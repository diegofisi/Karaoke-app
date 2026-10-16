import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import AddIcon from "@mui/icons-material/Add";
import { AvisoFabrica } from "@/common/components/AvisoFabrica";
import { CabeceraPagina } from "@/common/components/CabeceraPagina";
import { EstadoContenido } from "@/common/components/EstadoContenido";
import { DialogoBorrar } from "../components/DialogoBorrar";
import { DialogoModo } from "../components/DialogoModo";
import { TarjetaCancion } from "../components/TarjetaCancion";
import { useBiblioteca } from "../hooks/useBiblioteca";

export const BibliotecaPage = () => {
  const biblioteca = useBiblioteca();
  const { filas, cargando, error } = biblioteca;

  return (
    <Box>
      <CabeceraPagina
        titulo="Biblioteca"
        subtitulo="Tus canciones procesadas, listas para cantar."
        acciones={
          <Button variant="outlined" startIcon={<AddIcon />} onClick={biblioteca.irAProcesar}>
            Procesar canción
          </Button>
        }
      />
      <AvisoFabrica estado={biblioteca.estadoFabrica} error={biblioteca.errorFabrica} />

      {cargando && <EstadoContenido tipo="cargando" mensaje="Leyendo la biblioteca…" />}
      {!cargando && error !== null && filas.length === 0 && (
        <EstadoContenido
          tipo="error"
          mensaje="No se pudo leer la biblioteca."
          detalle={error}
          accion={<Button onClick={biblioteca.reintentar}>Reintentar</Button>}
        />
      )}
      {!cargando && error === null && filas.length === 0 && (
        <EstadoContenido
          tipo="vacio"
          mensaje="No hay canciones todavía."
          detalle="Procesa la primera a partir de una URL de YouTube o de un archivo de audio."
          accion={
            <Button variant="contained" startIcon={<AddIcon />} onClick={biblioteca.irAProcesar}>
              Ir a Procesar
            </Button>
          }
        />
      )}
      {filas.length > 0 && (
        <Box sx={{ display: "grid", gap: 2.5, gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))" }}>
          {filas.map((fila) => (
            <TarjetaCancion
              key={fila.id}
              fila={fila}
              onCantar={biblioteca.abrirCantar}
              onBorrar={biblioteca.abrirBorrar}
            />
          ))}
        </Box>
      )}

      <DialogoModo
        titulo={biblioteca.paraCantar?.titulo ?? null}
        onElegir={biblioteca.cantar}
        onCerrar={biblioteca.cerrarCantar}
      />
      <DialogoBorrar
        titulo={biblioteca.paraBorrar?.titulo ?? null}
        borrando={biblioteca.borrando}
        error={biblioteca.errorBorrar}
        onConfirmar={biblioteca.confirmarBorrar}
        onCerrar={biblioteca.cerrarBorrar}
      />
    </Box>
  );
};
