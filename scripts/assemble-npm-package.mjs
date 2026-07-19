#!/usr/bin/env node

import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";

const [inputArg, outputArg, version] = process.argv.slice(2);
if (!inputArg || !outputArg || !version) {
  throw new Error("usage: assemble-npm-package.mjs INPUT_DIR OUTPUT_DIR VERSION");
}

const input = resolve(inputArg);
const output = resolve(outputArg);
const targets = [
  "linux-x64",
  "linux-arm64",
  "darwin-x64",
  "darwin-arm64",
  "win32-x64"
];

rmSync(output, { recursive: true, force: true });
mkdirSync(output, { recursive: true });

const first = join(input, targets[0]);
cpSync(first, output, {
  recursive: true,
  filter(source) {
    return basename(source) !== "native-manifest.json" && basename(source) !== "native";
  }
});

const packageJsonPath = join(output, "package.json");
const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8"));
packageJson.version = version;
writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`);

const manifests = [];
for (const target of targets) {
  const directory = join(input, target);
  if (!existsSync(directory)) throw new Error(`missing release artifact: ${target}`);
  cpSync(join(directory, "native", target), join(output, "native", target), {
    recursive: true
  });
  manifests.push(JSON.parse(readFileSync(join(directory, "native-manifest.json"), "utf8")));
}

writeFileSync(
  join(output, "native-manifest.json"),
  `${JSON.stringify({ schema: 1, version, targets: manifests }, null, 2)}\n`
);
