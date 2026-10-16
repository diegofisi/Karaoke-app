// // Hook PreToolUse (Bash): bloquea `--no-verify` / `-n` en git commit|push|merge|
// // rebase|cherry-pick|am, y cualquier `-c core.hooksPath=` que saltaría los hooks
// // de git. Hoy el repo no tiene hooks de git; este hook ya está activo para que,
// // cuando existan, el agente no pueda saltárselos desde el primer commit.
// //
// // Copiado del de InglesApp (versión mínima de ECC
// // `scripts/hooks/block-no-verify.js`): una expresión por segmento de comando
// // basta para un repo de una persona.
// //
// // Códigos de salida: 0 = permitir, 2 = bloquear (el motivo va por stderr).
// const COMANDOS_CON_HOOKS = new Set(['commit', 'push', 'merge', 'rebase', 'cherry-pick', 'am']);

// // Divide en palabras respetando comillas simples y dobles, para que un `-n`
// // dentro de un mensaje de commit no cuente como flag.
// function tokenizar(segmento) {
//   const tokens = [];
//   let actual = '';
//   let comilla = null;
//   let enToken = false;
//   for (const ch of segmento) {
//     if (comilla) {
//       if (ch === comilla) comilla = null;
//       else actual += ch;
//       continue;
//     }
//     if (ch === '"' || ch === "'") { comilla = ch; enToken = true; continue; }
//     if (/\s/.test(ch)) {
//       if (enToken) { tokens.push(actual); actual = ''; enToken = false; }
//       continue;
//     }
//     actual += ch;
//     enToken = true;
//   }
//   if (enToken) tokens.push(actual);
//   return tokens;
// }

// function esSalto(segmento) {
//   const tokens = tokenizar(segmento);
//   const iGit = tokens.findIndex((t) => /^(.*[\\/])?git(\.exe)?$/i.test(t));
//   if (iGit === -1) return false;

//   // Opciones globales entre `git` y el subcomando: aquí vive `-c core.hooksPath=`.
//   let i = iGit + 1;
//   let cmd = null;
//   for (; i < tokens.length; i++) {
//     const t = tokens[i];
//     if (/^-c$/i.test(t) && /^core\.hookspath=/i.test(tokens[i + 1] || '')) return true;
//     if (/^-c\s*core\.hookspath=/i.test(t)) return true;
//     if (t === '-c' || t === '-C' || t === '--git-dir' || t === '--work-tree') { i++; continue; }
//     if (t.startsWith('-')) continue;
//     cmd = t;
//     break;
//   }
//   if (!cmd || !COMANDOS_CON_HOOKS.has(cmd)) return false;

//   // Flags del subcomando. Las opciones que llevan valor consumen el token siguiente.
//   for (let j = i + 1; j < tokens.length; j++) {
//     const t = tokens[j];
//     if (t === '--') break;
//     // --no-verify y sus prefijos no ambiguos (git acepta --no-veri, --no-verif...)
//     if (t.length >= '--no-v'.length && '--no-verify'.startsWith(t)) return true;
//     if (cmd === 'commit') {
//       if (/^-(m|F|C|c|t)$/.test(t) || /^--(message|file|template|author|date)$/.test(t)) { j++; continue; }
//       if (/^-(m|F|C|c|t)./.test(t) || /^--(message|file|template|author|date)=/.test(t)) continue;
//       if (/^-[a-zA-Z]*n[a-zA-Z]*$/.test(t)) return true; // -n suelto o en grupo (-an, -sn)
//     }
//   }
//   return false;
// }

// let raw = '';
// process.stdin.setEncoding('utf8');
// process.stdin.on('data', (c) => (raw += c));
// process.stdin.on('end', () => {
//   try {
//     const input = JSON.parse(raw || '{}');
//     const cmd = String(input?.tool_input?.command || '');
//     if (!cmd) process.exit(0);

//     // Un segmento por comando encadenado; se ignoran los comentarios.
//     const segmentos = cmd
//       .split(/\n|;|&&|\|\||\|/)
//       .map((s) => s.replace(/(^|\s)#.*$/, '').trim())
//       .filter(Boolean);

//     if (segmentos.some(esSalto)) {
//       process.stderr.write(
//         'BLOQUEADO: no se saltan los hooks de git (--no-verify, -n, core.hooksPath). ' +
//           'Si un hook falla, arregla la causa; si el hook está mal, dilo y para.\n',
//       );
//       process.exit(2);
//     }
//     process.exit(0);
//   } catch {
//     process.exit(0); // ante cualquier duda, no bloquear
//   }
// });
