// npm run verify: la puerta antes de dar algo por hecho (.claude/rules/verification.md).
// Encadena los pasos y para en el primero que falla. Windows y macOS: el
// Python de la fabrica esta en Scripts/ o en bin/ segun el sistema.
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

const raiz = path.resolve(import.meta.dirname, "..");

// La misma version en la app, Tauri, el workspace Cargo y la fabrica: la app
// reinstala la fabrica cuando su version cambia (entorno.rs).
const leer = (ruta) => readFileSync(path.join(raiz, ruta), "utf8");
const versiones = {
  "package.json": JSON.parse(leer("package.json")).version,
  "src-tauri/tauri.conf.json": JSON.parse(leer("src-tauri/tauri.conf.json")).version,
  "Cargo.toml": leer("Cargo.toml").match(/^version = "([^"]+)"/m)?.[1],
  "fabrica/pyproject.toml": leer("fabrica/pyproject.toml").match(/^version = "([^"]+)"/m)?.[1],
  "fabrica/karaoke_fabrica/__init__.py": leer("fabrica/karaoke_fabrica/__init__.py").match(/^__version__ = "([^"]+)"/m)?.[1],
};
console.log("== versiones");
if (new Set(Object.values(versiones)).size !== 1) {
  console.error("verify NO LISTO: las versiones no coinciden", versiones);
  process.exit(1);
}
console.log(Object.values(versiones)[0]);
const python = process.platform === "win32"
  ? path.join(raiz, "fabrica", ".venv", "Scripts", "python.exe")
  : path.join(raiz, "fabrica", ".venv", "bin", "python");

const pasos = [
  // tauri-build exige que existan los sidecars (ffmpeg, deno, uv).
  ["binarios", "npm", ["run", "binarios"]],
  ["typecheck + build web", "npm", ["run", "build"]],
  ["cargo clippy", "cargo", ["clippy", "-j", "4", "--workspace", "--all-targets", "--", "-D", "warnings"]],
  ["cargo test", "cargo", ["test", "-j", "4", "--workspace"]],
  ["cargo fmt", "cargo", ["fmt", "--all", "--check"]],
];
if (existsSync(python)) pasos.push(["pytest fabrica", python, ["-m", "pytest", "-q", "fabrica"]]);
else console.warn(`Aviso: sin entorno de la fabrica (${python}); se salta pytest. Ver fabrica/README.md`);

for (const [nombre, comando, args] of pasos) {
  console.log(`\n== ${nombre}`);
  // En Windows npm es un .cmd y Node solo lo lanza a traves de la shell.
  const shell = process.platform === "win32" && comando === "npm";
  const r = spawnSync(comando, args, { cwd: raiz, stdio: "inherit", shell });
  if (r.status !== 0) {
    console.error(`\nverify NO LISTO: fallo "${nombre}" (codigo ${r.status})`);
    process.exit(r.status ?? 1);
  }
}
console.log("\nverify LISTO");
