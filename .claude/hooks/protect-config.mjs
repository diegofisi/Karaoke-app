// Hook PreToolUse (Edit|Write): bloquea la edición de archivos de configuración
// de compilación, lint y pruebas. Un agente que no consigue que pase `tsc`,
// `clippy` o `pytest` tiende a relajar la configuración en vez de arreglar el
// código; este hook lo devuelve al código. Copiado del de InglesApp (que adapta
// el de MonitorHub y `scripts/hooks/config-protection.js` de ECC).
//
// Códigos de salida:
//   0 = permitir (no es un archivo protegido, o se está creando por primera vez)
//   2 = bloquear (edición de un archivo protegido que ya existe)
//
// `tauri.conf.json`, `Cargo.toml`, `package.json` y `pyproject.toml` NO están en
// la lista: son configuración de la app y dependencias, no comprobaciones que se
// puedan debilitar.
//
// Escape legítimo (por ejemplo, añadir un alias de ruta a tsconfig.json): el
// USUARIO exporta KARAOKE_ALLOW_CONFIG=1 en la sesión, o edita él el archivo.
import { lstatSync } from 'node:fs';
import path from 'node:path';

const PROTEGIDOS = new Set([
  'tsconfig.json',
  'tsconfig.app.json',
  'tsconfig.node.json',
  'vite.config.ts',
  'vite.config.js',
  'vite.config.mts',
  'vitest.config.ts',
  'rustfmt.toml',
  '.rustfmt.toml',
  'clippy.toml',
  '.clippy.toml',
  'eslint.config.js',
  'eslint.config.mjs',
  'eslint.config.ts',
  '.eslintrc',
  '.eslintrc.json',
  '.eslintrc.cjs',
  '.prettierrc',
  '.prettierrc.json',
  'prettier.config.js',
  'prettier.config.mjs',
  'pytest.ini',
  'ruff.toml',
  '.ruff.toml',
]);

let raw = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (c) => (raw += c));
process.stdin.on('end', () => {
  try {
    if (/^(1|true|yes)$/i.test(String(process.env.KARAOKE_ALLOW_CONFIG || ''))) process.exit(0);

    const input = JSON.parse(raw || '{}');
    const archivo = input?.tool_input?.file_path;
    if (!archivo) process.exit(0);

    const base = path.basename(String(archivo)).toLowerCase();
    if (!PROTEGIDOS.has(base)) process.exit(0);

    // Crear una configuración por primera vez es legítimo (por ejemplo,
    // tsconfig.json cuando nazca la interfaz, o vitest.config.ts cuando entren
    // las pruebas). Lo que se bloquea es debilitar una que ya existe.
    let existe = true;
    try {
      lstatSync(archivo);
    } catch (err) {
      if (err && err.code === 'ENOENT') existe = false;
      // Cualquier otro error (permisos, enlace roto) deja existe=true: fallar cerrado.
    }
    if (!existe) process.exit(0);

    process.stderr.write(
      `BLOQUEADO: no se edita ${base} para que pase una comprobación. ` +
        'Arregla el código. Si el cambio de configuración es legítimo (nueva ' +
        'herramienta, nuevo alias), pide al usuario que lo haga él o que exporte ' +
        'KARAOKE_ALLOW_CONFIG=1 en esta sesión.\n',
    );
    process.exit(2);
  } catch {
    process.exit(0); // ante cualquier duda, no bloquear
  }
});
