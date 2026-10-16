// Hook PostToolUse (Edit|Write): pasa las reglas de arquitectura por el archivo
// que se acaba de escribir y, si las rompe, sale con 2 para que el agente lo
// lea y lo arregle en el acto, en vez de descubrirlo al final en `npm run verify`.
//
// La raíz del repo ($CLAUDE_PROJECT_DIR) es a la vez la raíz de la app: ahí
// estarán `scripts/check-rules.mjs`, `src/`, `src-tauri/` y `crates/`.
//
// HOY `scripts/check-rules.mjs` NO EXISTE: el hook sale con 0 sin hacer nada.
// En cuanto exista, empieza a funcionar sin tocar este archivo. Es el eslabón
// rápido de la cadena; los otros usarán el mismo comprobador:
//   este hook                  -> node scripts/check-rules.mjs --file <archivo>
//   pre-commit (cuando exista) -> node scripts/check-rules.mjs --staged
//   npm run verify             -> node scripts/check-rules.mjs
//
// Nunca bloquea por un fallo propio: si el comprobador no existe o revienta,
// sale con 0 y se calla.
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';

const EXTENSIONES = /\.(rs|ts|tsx|md)$/;
const CARPETAS_IGNORADAS = new Set([
  'node_modules', 'target', 'dist', 'gen', '.git', '.claude',
  '.venv', 'venv', '__pycache__', 'fabrica', 'eval',
]);

let raw = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (c) => (raw += c));
process.stdin.on('end', () => {
  try {
    const input = JSON.parse(raw || '{}');
    const archivo = input?.tool_input?.file_path;
    if (!archivo) process.exit(0);

    const raiz = process.env.CLAUDE_PROJECT_DIR
      ? path.resolve(process.env.CLAUDE_PROJECT_DIR)
      : process.cwd();

    const script = path.join(raiz, 'scripts', 'check-rules.mjs');
    if (!existsSync(script)) process.exit(0);

    const abs = path.resolve(String(archivo));
    const relativo = path.relative(raiz, abs);
    if (!relativo || relativo.startsWith('..') || path.isAbsolute(relativo)) process.exit(0);
    if (!EXTENSIONES.test(abs)) process.exit(0);
    if (relativo.split(path.sep).some((s) => CARPETAS_IGNORADAS.has(s))) process.exit(0);

    const r = spawnSync(process.execPath, [script, '--file', abs], {
      cwd: raiz,
      encoding: 'utf8',
      timeout: 20_000,
      windowsHide: true,
    });

    // status 1 = hay errores de regla. Cualquier otra cosa (timeout, crash) se ignora.
    if (r.status === 1) {
      const detalle = `${r.stderr || ''}${r.stdout || ''}`.trim();
      process.stderr.write(
        'Este archivo rompe una regla de arquitectura del proyecto:\n\n' +
          detalle +
          '\n\nArregla la causa antes de seguir. Las reglas están en .claude/rules/ y\n' +
          'en docs/CONTRATO.md; si la excepción es legítima, documéntala en esa línea\n' +
          'con `// permitido: <razón>`.\n',
      );
      process.exit(2);
    }
    process.exit(0);
  } catch {
    process.exit(0);
  }
});
