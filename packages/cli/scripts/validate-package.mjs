import { existsSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));

if (packageJson.name !== "solid-checker") {
  throw new Error(`unexpected package name: ${packageJson.name}`);
}
if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(packageJson.version)) {
  throw new Error(`invalid publish version: ${packageJson.version}`);
}
if (!existsSync(join(root, "native-manifest.json"))) {
  throw new Error("native-manifest.json is missing; run `make package` before packing");
}
for (const command of ["solid-check", "solid-checkd"]) {
  if (!existsSync(join(root, packageJson.bin[command]))) {
    throw new Error(`launcher for ${command} is missing`);
  }
}
