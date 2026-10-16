// Hook Stop: un `tsc -b` y/o un `cargo check --workspace` al final de la
// respuesta, solo si se tocaron archivos .ts/.tsx de `src/` o .rs/Cargo.toml.
//
// SOLO INFORMA. Siempre sale con 0 y nunca emite `decision: "block"`, así que
// no puede parar la respuesta ni entrar en bucle. `npm run verify` sigue siendo
// la puerta real; esto es un aviso temprano entre verificaciones.
//
// Se ejecuta en la raíz del repo ($CLAUDE_PROJECT_DIR), que es a la vez raíz
// del workspace Cargo y del package.json. Cada paso se salta en silencio si su
// archivo no existe todavía:
//   - tsc -b               solo si existen tsconfig.json y node_modules/.bin/tsc
//   - cargo check          solo si existe Cargo.toml en la raíz
//
// Cómo sabe qué cambió: `git status --porcelain` (working tree, índice y sin
// seguimiento). Si git falla, no informa de nada.
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';

const PRESUPUESTO_TOTAL_MS = 150_000; // por debajo del timeout del hook en settings.json
const CARPETAS_IGNORADAS = new Set(['node_modules', 'target', 'dist', 'gen', '.git', '.venv', 'venv']);

function terminar(mensaje) {
  // systemMessage muestra un aviso al usuario sin bloquear la respuesta.
  if (mensaje) process.stdout.write(JSON.stringify({ systemMessage: mensaje }));
  process.exit(0);
}

function git(args, cwd) {
  try {
    return execFileSync('git', args, {
      cwd, encoding: 'utf8', timeout: 10_000, windowsHide: true, stdio: ['ignore', 'pipe', 'ignore'],
    });
  } catch {
    return null;
  }
}

function ignorado(ruta) {
  return ruta.split(path.sep).some((s) => CARPETAS_IGNORADAS.has(s));
}

// Archivos cambiados según `git status`. Con -z las rutas no llevan comillas;
// en un renombrado (R) o copia (C), la entrada siguiente es la ruta de origen y
// se salta. Las rutas son relativas al toplevel del repo.
function cambiadosSegunGit(raiz) {
  const top = (git(['rev-parse', '--show-toplevel'], raiz) || '').trim();
  if (!top) return [];
  const salida = git(['status', '--porcelain=v1', '-z', '--untracked-files=all'], raiz);
  if (!salida) return [];
  const entradas = salida.split('\0');
  const rutas = [];
  for (let i = 0; i < entradas.length; i++) {
    const e = entradas[i];
    if (e.length < 4) continue;
    const xy = e.slice(0, 2);
    rutas.push(e.slice(3));
    if (/[RC]/.test(xy)) i++; // saltar la ruta de origen
  }
  return [...new Set(rutas)]
    .map((f) => path.resolve(top, f))
    .filter((f) => existsSync(f) && !ignorado(f));
}

// Un solo string de comando: los .cmd de Windows necesitan shell, y las comillas
// protegen rutas con espacios o paréntesis.
function ejecutar(cmd, args, cwd, timeout) {
  const conShell = process.platform === 'win32';
  const q = (s) => (conShell && /[\s"()]/.test(String(s)) ? `"${s}"` : String(s));
  return spawnSync(q(cmd), args.map(q), {
    cwd,
    encoding: 'utf8',
    timeout,
    shell: conShell,
    windowsHide: true,
  });
}

function resumir(etiqueta, resultado, patron) {
  if (resultado.error || resultado.status === null || resultado.status === 0) return []; // timeout, fallo de spawn o verde: silencio
  const lineas = `${resultado.stdout || ''}${resultado.stderr || ''}`.split('\n').filter((l) => patron.test(l));
  const informe = [`${etiqueta}: ${lineas.length} error(es)`, ...lineas.slice(0, 5).map((l) => `  ${l.trim()}`)];
  if (lineas.length > 5) informe.push(`  ... ${lineas.length - 5} más`);
  return informe;
}

let raw = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (c) => (raw += c));
process.stdin.on('end', () => {
  try {
    const input = JSON.parse(raw || '{}');
    if (input?.stop_hook_active) terminar(); // ya estamos dentro de un Stop anterior

    const raiz = process.env.CLAUDE_PROJECT_DIR
      ? path.resolve(process.env.CLAUDE_PROJECT_DIR)
      : process.cwd();

    const tsconfig = path.join(raiz, 'tsconfig.json');
    const tsc = path.join(raiz, 'node_modules', '.bin', process.platform === 'win32' ? 'tsc.cmd' : 'tsc');
    const manifiesto = path.join(raiz, 'Cargo.toml');
    const puedeTs = existsSync(tsconfig) && existsSync(tsc);
    const puedeRs = existsSync(manifiesto);
    if (!puedeTs && !puedeRs) terminar(); // todavía no hay interfaz ni workspace

    const archivos = cambiadosSegunGit(raiz);
    const srcTs = path.join(raiz, 'src') + path.sep;
    const tocoTs = puedeTs && archivos.some((f) => /\.(ts|tsx|mts|cts)$/.test(f) && f.toLowerCase().startsWith(srcTs.toLowerCase()));
    const tocoRs = puedeRs && archivos.some((f) => /\.rs$/.test(f) || path.basename(f) === 'Cargo.toml');
    if (!tocoTs && !tocoRs) terminar();

    const porPaso = Math.floor(PRESUPUESTO_TOTAL_MS / ((tocoTs ? 1 : 0) + (tocoRs ? 1 : 0)));
    const informe = [];

    if (tocoTs) {
      // `tsc -b`, igual que el futuro `npm run typecheck`. Sin `--noEmit`: con
      // proyectos referenciados `composite`, `tsc -b --noEmit` falla con TS6310.
      const r = ejecutar(tsc, ['-b', tsconfig], raiz, porPaso);
      informe.push(...resumir('tsc -b', r, /error TS\d+/));
    }

    if (tocoRs) {
      const r = ejecutar('cargo', ['check', '--workspace', '--manifest-path', manifiesto, '--message-format=short'], raiz, porPaso);
      informe.push(...resumir('cargo check --workspace', r, /error(\[E\d+\])?:/));
    }

    if (informe.length === 0) terminar();
    terminar(`Comprobación al cerrar (solo informa; npm run verify desde la raíz es la puerta)\n${informe.join('\n')}`);
  } catch {
    process.exit(0); // nunca bloquear
  }
});
