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
import { elegirArchivoCookies, elegirCarpeta } from "@/common/api/dialogo";
import { CabeceraPagina } from "@/common/components/CabeceraPagina";
import { EstadoContenido } from "@/common/components/EstadoContenido";
import { Dificultad, type Dificultad as TipoDificultad } from "@/common/interfaces";
import { PanelCalibracion } from "../components/PanelCalibracion";
import { useAjustesPantalla } from "../hooks/useAjustesPantalla";

const MICROFONO_POR_DEFECTO = "";

export const AjustesPage = () => {
  const a = useAjustesPantalla();
  const b = a.borrador;

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
          <Typography variant="body2" sx={{ mb: 1 }}>
            Si al procesar aparece «confirmar que no eres un bot», YouTube está bloqueando las descargas desde tu
            conexión. Dale tu sesión de YouTube de una de estas dos formas:
          </Typography>
          <Typography variant="body2" sx={{ fontWeight: 600, mt: 1.5 }}>
            1. Archivo cookies.txt (funciona siempre)
          </Typography>
          <Typography variant="caption" component="p" sx={{ color: "text.secondary", mb: 1 }}>
            En tu navegador instala la extensión «Get cookies.txt LOCALLY», abre youtube.com con tu sesión iniciada,
            exporta las cookies y elige aquí ese archivo. Si deja de funcionar, vuelve a exportarlo.
          </Typography>
          <Typography variant="body2" sx={{ color: "text.secondary", mb: 1, wordBreak: "break-all" }}>
            {b.youtube_cookies ?? "Ningún archivo elegido"}
          </Typography>
          <Stack direction="row" spacing={1}>
            <Button
              variant="outlined"
              onClick={() => {
                void elegirArchivoCookies().then((c) => {
                  if (c !== null) a.cambiar("youtube_cookies", c);
                });
              }}
            >
              Elegir cookies.txt…
            </Button>
            <Button onClick={() => a.cambiar("youtube_cookies", null)} disabled={b.youtube_cookies === null}>
              Quitar
            </Button>
          </Stack>
          <Typography variant="body2" sx={{ fontWeight: 600, mt: 2.5 }}>
            2. Leer la sesión del navegador
          </Typography>
          <Typography variant="caption" component="p" sx={{ color: "text.secondary", mb: 1 }}>
            En Mac funciona con Safari, Chrome, Firefox, Brave y Edge. En Windows solo con Firefox: Chrome, Edge y
            Brave no dejan leer sus cookies. Si hay un cookies.txt elegido, se usa ese.
          </Typography>
          <FormControl size="small" sx={{ minWidth: 220 }}>
            <InputLabel id="navegador">Navegador</InputLabel>
            <Select
              labelId="navegador"
              label="Navegador"
              value={b.youtube_navegador ?? ""}
              onChange={(e) => a.cambiar("youtube_navegador", e.target.value === "" ? null : e.target.value)}
            >
              <MenuItem value="">Ninguno</MenuItem>
              <MenuItem value="firefox">Firefox</MenuItem>
              <MenuItem value="safari">Safari (Mac)</MenuItem>
              <MenuItem value="chrome">Chrome (Mac)</MenuItem>
              <MenuItem value="edge">Edge (Mac)</MenuItem>
              <MenuItem value="brave">Brave (Mac)</MenuItem>
            </Select>
          </FormControl>
        </Seccion>

        <Seccion titulo="Fábrica">
          <Typography variant="body2" sx={{ mb: 1 }}>
            Carpeta del repositorio (o su carpeta <code>fabrica</code>) con el entorno de Python instalado.
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
