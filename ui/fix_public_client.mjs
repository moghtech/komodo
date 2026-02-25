import { readFileSync, writeFileSync } from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

console.log("Fixing public client...");

function fix_mogh_auth_import(path) {
  const file_path = __dirname + "/public/client/" + path;
  const contents = readFileSync(file_path);
  const fixed = contents
    .toString()
    .replaceAll('from "mogh_auth_client"', 'from \"npm:mogh_auth_client\"');
  writeFileSync(file_path, fixed);
}

for (const path of ["lib.d.ts", "lib.js"]) {
  fix_mogh_auth_import(path)
}
