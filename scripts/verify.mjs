// npm run verify: la puerta antes de dar algo por hecho (.claude/rules/verification.md).
// Encadena los pasos y para en el primero que falla. Windows y macOS: el
// Python de la fabrica esta en Scripts/ o en bin/ segun el sistema.
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";

const raiz = path.resolve(import.meta.dirname, "..");
const python = process.platform === "win32"
  ? path.join(raiz, "fabrica", ".venv", "Scripts", "python.exe")
  : path.join(raiz, "fabrica", ".venv", "bin", "python");

const pasos = [
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
