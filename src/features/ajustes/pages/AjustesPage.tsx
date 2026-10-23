import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import FormControl from "@mui/material/FormControl";
import FormControlLabel from "@mui/material/FormControlLabel";
import InputLabel from "@mui/material/InputLabel";
import MenuItem from "@mui/material/MenuItem";
import Paper from "@mui/material/Paper";
import Select from "@mui/material/Select";
import Slider from "@mui/material/Slider";
import Stack from "@mui/material/Stack";
import Switch from "@mui/material/Switch";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Typography from "@mui/material/Typography";
import type { ReactNode } from "react";
import { elegirCarpeta } from "@/common/api/dialogo";
import { useEstadoFabrica } from "@/common/api/consultas";
import { CabeceraPagina } from "@/common/components/CabeceraPagina";
import { EstadoContenido } from "@/common/components/EstadoContenido";
import { PanelRequisitos } from "@/common/components/PanelRequisitos";
import { mensajeDeError } from "@/common/helpers/errores";
import { useInstalacionFabrica } from "@/common/hooks/useInstalacionFabrica";
import { Dificultad, type Dificultad as TipoDificultad } from "@/common/interfaces";
import { PanelCalibracion } from "../components/PanelCalibracion";
import { SeccionYoutube } from "../components/SeccionYoutube";
import { useAjustesPantalla } from "../hooks/useAjustesPantalla";

const MICROFONO_POR_DEFECTO = "";

export const AjustesPage = () => {
  const a = useAjustesPantalla();
  const b = a.borrador;
  const fabrica = useEstadoFabrica();
  const instalacion = useInstalacionFabrica();

  if (a.cargando || (b === null && a.errorCarga === null)) {
    return <EstadoContenido tipo="cargando" mensaje="Leyendo los ajustes…" />;
  }
  if (b === null) {
    return <EstadoContenido tipo="error" mensaje="No se pudieron leer los ajustes." detalle={a.errorCarga} />;
  }

  return (
    <Box sx={{ maxWidth: 760 }}>
      <CabeceraPagina
        titulo="Ajustes"
        acciones={
          <Stack direction="row" spacing={1}>
            <Button onClick={a.descartar} disabled={!a.cambiado || a.guardando}>
              Descartar
            </Button>
            <Button variant="contained" onClick={a.guardar} disabled={!a.cambiado || a.guardando}>
              {a.guardando ? "Guardando…" : "Guardar"}
            </Button>
          </Stack>
        }
      />
      {a.errorGuardar !== null && <Alert severity="error" sx={{ mb: 2 }}>{a.errorGuardar}</Alert>}
      {a.guardadoOk && <Alert severity="success" sx={{ mb: 2 }}>Ajustes guardados.</Alert>}

      <Stack spacing={2.5}>
        <Seccion titulo="Micrófono">
          <FormControl fullWidth size="small">
            <InputLabel id="microfono">Micrófono</InputLabel>
            <Select
              labelId="microfono"
              label="Micrófono"
              value={b.microfono ?? MICROFONO_POR_DEFECTO}
              onChange={(e) => a.cambiar("microfono", e.target.value === MICROFONO_POR_DEFECTO ? null : e.target.value)}
            >
              <MenuItem value={MICROFONO_POR_DEFECTO}>El del sistema</MenuItem>
              {a.microfonos.map((m) => (
                <MenuItem key={m.id} value={m.id}>
                  {m.nombre}
                  {m.por_defecto ? " (predeterminado)" : ""}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
        </Seccion>

        <Seccion titulo="Latencia">
          <PanelCalibracion
            calibracion={a.calibracion}
            latenciaActual={b.latencia_ms}
            onCalibrar={a.calibrar}
            onUsarMedida={a.usarLatenciaMedida}
          />
          <Typography variant="body2" sx={{ mt: 2 }}>
            Latencia: {Math.round(b.latencia_ms)} ms
          </Typography>
          <Slider
            min={0}
            max={400}
            step={5}
            value={b.latencia_ms}
            onChange={(_, v) => a.cambiar("latencia_ms", v as number)}
            valueLabelDisplay="auto"
            aria-label="Latencia en milisegundos"
          />
        </Seccion>

        <Seccion titulo="Juego">
          <Typography variant="body2" sx={{ mb: 1 }}>
            Dificultad
          </Typography>
          <ToggleButtonGroup
            exclusive
            size="small"
            value={b.dificultad}
            onChange={(_, v: TipoDificultad | null) => {
              if (v !== null) a.cambiar("dificultad", v);
            }}
          >
            <ToggleButton value={Dificultad.Facil}>Fácil (±2 semitonos)</ToggleButton>
            <ToggleButton value={Dificultad.Normal}>Normal (±1)</ToggleButton>
            <ToggleButton value={Dificultad.Dificil}>Difícil (±½)</ToggleButton>
          </ToggleButtonGroup>
          <FormControlLabel
            sx={{ display: "flex", mt: 2 }}
            control={<Switch checked={b.mostrar_romaji} onChange={(e) => a.cambiar("mostrar_romaji", e.target.checked)} />}
            label="Mostrar romaji en canciones japonesas"
          />
          <FormControlLabel
            sx={{ display: "flex" }}
            control={<Switch checked={b.grabar} onChange={(e) => a.cambiar("grabar", e.target.checked)} />}
            label="Grabar mi voz en cada partida (se guarda en Música/Karaoke)"
          />
        </Seccion>

        <Seccion titulo="YouTube">
          <SeccionYoutube />
        </Seccion>

        <Seccion titulo="Fábrica">
          <PanelRequisitos
            estado={fabrica.data}
            error={fabrica.isError ? mensajeDeError(fabrica.error) : null}
            instalacion={instalacion}
            mostrarSiListo={true}
          />
          <Typography variant="body2" sx={{ mb: 1 }}>
            Carpeta del repositorio (o su carpeta <code>fabrica</code>) con el entorno de Python instalado. Déjala por
            defecto salvo que desarrolles la fábrica.
          </Typography>
          <Typography variant="body2" sx={{ color: "text.secondary", mb: 1.5, wordBreak: "break-all" }}>
            {b.carpeta_fabrica ?? "La de desarrollo (por defecto)"}
          </Typography>
          <Stack direction="row" spacing={1}>
            <Button
              variant="outlined"
              onClick={() => {
                void elegirCarpeta("Elige la carpeta del Karaoke o de su fábrica").then((c) => {
                  if (c !== null) a.cambiar("carpeta_fabrica", c);
                });
              }}
            >
              Elegir carpeta…
            </Button>
            <Button onClick={() => a.cambiar("carpeta_fabrica", null)} disabled={b.carpeta_fabrica === null}>
              Usar la de por defecto
            </Button>
          </Stack>
        </Seccion>

        <Seccion titulo="Volumen">
          <Typography variant="body2">Música: {Math.round(b.volumen_musica * 100)} %</Typography>
          <Slider min={0} max={1} step={0.05} value={b.volumen_musica}
            onChange={(_, v) => a.cambiar("volumen_musica", v as number)} aria-label="Volumen de la música" />
          <Typography variant="body2">
            Voz guía: {b.volumen_guia === 0 ? "apagada" : `${Math.round(b.volumen_guia * 100)} %`}
          </Typography>
          <Slider min={0} max={1} step={0.05} value={b.volumen_guia}
            onChange={(_, v) => a.cambiar("volumen_guia", v as number)} aria-label="Volumen de la voz guía" />
          <Typography variant="body2">
            Tu voz en la grabación: {Math.round(b.volumen_voz * 100)} %
            {b.volumen_voz === 1 ? " (automático)" : ""}
          </Typography>
          <Slider min={0} max={3} step={0.1} value={b.volumen_voz}
            onChange={(_, v) => a.cambiar("volumen_voz", v as number)} aria-label="Volumen de tu voz en la grabación"
            marks={[{ value: 1, label: "auto" }]} />
          <Typography variant="caption" sx={{ color: "text.secondary" }}>
            La voz ya se nivela sola y se comprime para que se entiendan las palabras; súbelo si aún se oye baja.
          </Typography>
        </Seccion>
      </Stack>
    </Box>
  );
};

const Seccion = ({ titulo, children }: { titulo: string; children: ReactNode }) => (
  <Paper sx={{ p: 3, borderRadius: 3 }}>
    <Typography variant="h6" component="h2" sx={{ mb: 2 }}>
      {titulo}
    </Typography>
    {children}
  </Paper>
);
