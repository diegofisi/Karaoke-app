import type { FormEvent } from "react";
import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import MenuItem from "@mui/material/MenuItem";
import Select from "@mui/material/Select";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Typography from "@mui/material/Typography";
import AudioFileIcon from "@mui/icons-material/AudioFile";
import AutoFixHighIcon from "@mui/icons-material/AutoFixHigh";
import FolderOpenIcon from "@mui/icons-material/FolderOpen";
import YouTubeIcon from "@mui/icons-material/YouTube";
import { Idioma } from "@/common/interfaces";
import { NOMBRE_IDIOMA, aIdioma } from "@/common/helpers/textos";
import { nombreDeArchivo } from "../helpers/formulario";
import {
  IDIOMA_AUTOMATICO,
  OrigenAudio,
  type CambiarCampo,
  type FormularioProceso as Formulario,
} from "../interfaces/formulario";

export interface FormularioProcesoProps {
  formulario: Formulario;
  cambiar: CambiarCampo;
  problema: string | null;
  mostrarProblema: boolean;
  onElegirArchivo: () => void;
  errorDialogo: string | null;
  onEnviar: () => void;
  arrancando: boolean;
  fabricaNoLista: boolean;
}

export const FormularioProceso = ({
  formulario,
  cambiar,
  problema,
  mostrarProblema,
  onElegirArchivo,
  errorDialogo,
  onEnviar,
  arrancando,
  fabricaNoLista,
}: FormularioProcesoProps) => {
  const alEnviar = (evento: FormEvent<HTMLFormElement>) => {
    evento.preventDefault();
    onEnviar();
  };
  const esUrl = formulario.origen === OrigenAudio.Url;
  const errorVisible = mostrarProblema && problema !== null;

  return (
    <Box component="form" onSubmit={alEnviar} noValidate>
      <Stack spacing={2.5}>
        <ToggleButtonGroup
          exclusive
          color="primary"
          value={formulario.origen}
          onChange={(_evento, valor: string | null) => {
            if (valor === OrigenAudio.Url || valor === OrigenAudio.Archivo) cambiar("origen", valor);
          }}
          aria-label="Origen del audio"
        >
          <ToggleButton value={OrigenAudio.Url} sx={{ px: 2.5, gap: 1 }}>
            <YouTubeIcon fontSize="small" />
            URL de YouTube
          </ToggleButton>
          <ToggleButton value={OrigenAudio.Archivo} sx={{ px: 2.5, gap: 1 }}>
            <AudioFileIcon fontSize="small" />
            Archivo local
          </ToggleButton>
        </ToggleButtonGroup>

        {esUrl ? (
          <TextField
            label="URL de YouTube"
            placeholder="https://www.youtube.com/watch?v=…"
            value={formulario.url}
            onChange={(evento) => cambiar("url", evento.target.value)}
            error={errorVisible}
            helperText={errorVisible ? problema : "El título y el artista se leen del vídeo."}
            fullWidth
            autoFocus
          />
        ) : (
          <Stack spacing={2}>
            <Stack direction="row" spacing={2} sx={{ alignItems: "center" }}>
              <Button type="button" variant="outlined" startIcon={<FolderOpenIcon />} onClick={onElegirArchivo}>
                Elegir archivo
              </Button>
              <Typography
                variant="body2"
                sx={{ color: errorVisible ? "error.main" : "text.secondary", wordBreak: "break-all" }}
              >
                {formulario.archivo !== null
                  ? nombreDeArchivo(formulario.archivo)
                  : errorVisible
                    ? problema
                    : "Ningún archivo elegido."}
              </Typography>
            </Stack>
            {errorDialogo !== null && <Alert severity="error">{errorDialogo}</Alert>}
            <Stack direction={{ xs: "column", sm: "row" }} spacing={2}>
              <TextField
                label="Título (opcional)"
                value={formulario.titulo}
                onChange={(evento) => cambiar("titulo", evento.target.value)}
                fullWidth
              />
              <TextField
                label="Artista (opcional)"
                value={formulario.artista}
                onChange={(evento) => cambiar("artista", evento.target.value)}
                fullWidth
              />
            </Stack>
          </Stack>
        )}

        <FormControl sx={{ maxWidth: 280 }}>
          <InputLabel id="idioma-proceso">Idioma de la letra</InputLabel>
          <Select
            labelId="idioma-proceso"
            label="Idioma de la letra"
            value={formulario.idioma}
            onChange={(evento) => cambiar("idioma", aIdioma(evento.target.value) ?? IDIOMA_AUTOMATICO)}
          >
            <MenuItem value={IDIOMA_AUTOMATICO}>Detectar automáticamente</MenuItem>
            <MenuItem value={Idioma.Es}>{NOMBRE_IDIOMA[Idioma.Es]}</MenuItem>
            <MenuItem value={Idioma.En}>{NOMBRE_IDIOMA[Idioma.En]}</MenuItem>
            <MenuItem value={Idioma.Ja}>{NOMBRE_IDIOMA[Idioma.Ja]}</MenuItem>
          </Select>
        </FormControl>

        <TextField
          label="Letra (opcional)"
          placeholder="Pega aquí la letra en texto plano o en formato LRC."
          value={formulario.letra}
          onChange={(evento) => cambiar("letra", evento.target.value)}
          helperText="Si no la pegas, se busca en LRCLIB; si tampoco está, se transcribe del audio."
          multiline
          minRows={4}
          maxRows={14}
          fullWidth
        />

        <Box>
          <Button
            type="submit"
            variant="contained"
            size="large"
            startIcon={<AutoFixHighIcon />}
            disabled={arrancando || fabricaNoLista || (mostrarProblema && problema !== null)}
          >
            {arrancando ? "Arrancando…" : "Procesar"}
          </Button>
        </Box>
      </Stack>
    </Box>
  );
};
