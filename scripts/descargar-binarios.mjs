// Descarga las herramientas que van DENTRO de la app (sidecars de Tauri):
// ffmpeg, deno (runtime de JavaScript para que YouTube deje descargar) y uv
// (instala el Python de la fabrica). Las deja en src-tauri/binaries/ con el
// nombre que pide Tauri: <nombre>-<triple>[.exe]. Verifica cada descarga contra
// el SHA-256 fijado abajo. Si un binario ya esta, no lo vuelve a bajar.
//
// Uso: node scripts/descargar-binarios.mjs [triple]   (por defecto, el del equipo)
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readdirSync, renameSync, rmSync, statSync, chmodSync, writeFileSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

const VERSIONES = { deno: "v2.9.7", uv: "0.12.19", ffmpeg: "9.0.2" };

// sha256: copiado del .sha256 oficial al elegir cada version. Fijado aqui para
// que un servidor comprometido no pueda cambiar a la vez el binario y su hash.
const FUENTES = {
  "x86_64-pc-windows-msvc": {
    ffmpeg: {
      url: `https://www.gyan.dev/ffmpeg/builds/packages/ffmpeg-${VERSIONES.ffmpeg}-essentials_build.zip`,
      sha256: "60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba",
      dentro: "ffmpeg.exe",
    },
    deno: {
      url: `https://github.com/denoland/deno/releases/download/${VERSIONES.deno}/deno-x86_64-pc-windows-msvc.zip`,
      sha256: "a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238",
      dentro: "deno.exe",
    },
    uv: {
      url: `https://github.com/astral-sh/uv/releases/download/${VERSIONES.uv}/uv-x86_64-pc-windows-msvc.zip`,
      sha256: "6dbb02d79e419522f1c500f0adb1cddcff0cda7d59b0d66ea7f5e3b4a1b2f5f0",
      dentro: "uv.exe",
    },
  },
  "aarch64-apple-darwin": {
    ffmpeg: {
      url: "https://ffmpeg.martin-riedl.de/download/macos/arm64/1789931890_9.0.2/ffmpeg.zip",
      sha256: "c8ed4c4e6978a03c485edbfe4e0a5dc2380f8a30bba5150531b31b094492d924",
      dentro: "ffmpeg",
    },
    deno: {
      url: `https://github.com/denoland/deno/releases/download/${VERSIONES.deno}/deno-aarch64-apple-darwin.zip`,
      sha256: "5cd46d6268f6f78f5d88bdc7159d20bd44cdaa4b3303474839f87ec6fe7ae25c",
      dentro: "deno",
    },
    uv: {
      url: `https://github.com/astral-sh/uv/releases/download/${VERSIONES.uv}/uv-aarch64-apple-darwin.tar.gz`,
      sha256: "a9a8df1eedeb192f2e47e40e2faabfb387db4b850209118786d42f89dde3e0ba",
      dentro: "uv",
    },
  },
};

const raiz = path.resolve(import.meta.dirname, "..");
const destino = path.join(raiz, "src-tauri", "binaries");

const tripleDelEquipo = () => {
  const salida = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  const linea = salida.split("\n").find((l) => l.startsWith("host:"));
  if (!linea) throw new Error("No se pudo leer el triple de rustc -vV");
  return linea.slice(5).trim();
};

const bajar = async (url) => {
  const r = await fetch(url, { redirect: "follow" });
  if (!r.ok) throw new Error(`No se pudo descargar ${url}: HTTP ${r.status}`);
  return Buffer.from(await r.arrayBuffer());
};

const buscar = (dir, nombre) => {
  for (const e of readdirSync(dir)) {
    const ruta = path.join(dir, e);
    if (statSync(ruta).isDirectory()) {
      const encontrado = buscar(ruta, nombre);
      if (encontrado) return encontrado;
    } else if (e === nombre) {
      return ruta;
    }
  }
  return null;
};

const triple = process.argv[2] ?? tripleDelEquipo();
const fuentes = FUENTES[triple];
if (!fuentes) {
  console.error(`Triple sin binarios configurados: ${triple}. Soportados: ${Object.keys(FUENTES).join(", ")}`);
  process.exit(1);
}
mkdirSync(destino, { recursive: true });
const exe = triple.includes("windows") ? ".exe" : "";

for (const [nombre, f] of Object.entries(fuentes)) {
  const final = path.join(destino, `${nombre}-${triple}${exe}`);
  const marca = `${final}.version`;
  const version = `${f.url}\n`;
  if (existsSync(final) && existsSync(marca) && readFileSync(marca, "utf8") === version) {
    console.log(`${nombre}: ya esta (${path.basename(final)})`);
    continue;
  }
  console.log(`${nombre}: descargando ${f.url}`);
  const datos = await bajar(f.url);
  const esperado = f.sha256;
  const real = createHash("sha256").update(datos).digest("hex");
  if (real !== esperado) {
    console.error(`${nombre}: el SHA-256 no coincide (esperado ${esperado}, obtenido ${real})`);
    process.exit(1);
  }
  const tmp = mkdtempSync(path.join(tmpdir(), `karaoke-${nombre}-`));
  const archivo = path.join(tmp, path.basename(new URL(f.url).pathname));
  writeFileSync(archivo, datos);
  // bsdtar abre .zip y .tar.gz en Windows 10+ y en macOS. En Windows se usa el
  // del sistema: el `tar` de Git Bash (GNU) lee "C:" como un servidor remoto.
  const tar = process.platform === "win32"
    ? path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "tar.exe")
    : "tar";
  execFileSync(tar, ["-xf", archivo, "-C", tmp]);
  const binario = buscar(tmp, f.dentro);
  if (!binario) {
    console.error(`${nombre}: no se encontro ${f.dentro} dentro de ${f.url}`);
    process.exit(1);
  }
  rmSync(final, { force: true });
  renameSync(binario, final);
  if (!exe) chmodSync(final, 0o755);
  writeFileSync(marca, version);
  rmSync(tmp, { recursive: true, force: true });
  console.log(`${nombre}: listo (${path.basename(final)}, SHA-256 verificado)`);
}
