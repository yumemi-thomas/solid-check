#!/usr/bin/env node

import { existsSync } from "node:fs";
import { dirname, join, parse, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";

const packageRoot = resolve(import.meta.dirname, "..");
const executableName = process.platform === "win32" ? "solid-check.exe" : "solid-check";
const sidecarName = process.platform === "win32" ? "solid-compiler-facts.exe" : "solid-compiler-facts";

function findRepository(start) {
  let directory = resolve(start);
  for (;;) {
    if (
      existsSync(join(directory, "go.mod")) &&
      existsSync(join(directory, "cmd", "solid-check"))
    ) {
      return directory;
    }
    const parent = dirname(directory);
    if (parent === directory || directory === parse(directory).root) return undefined;
    directory = parent;
  }
}

function packagedBinary(name) {
  return join(packageRoot, "native", `${process.platform}-${process.arch}`, name);
}

const repository = findRepository(packageRoot) ?? findRepository(process.cwd());
let executable = process.env.SOLID_CHECK_NATIVE_BIN || packagedBinary(executableName);
let repositorySidecar;

if (!existsSync(executable) && repository) {
  executable = join(repository, "bin", executableName);
  repositorySidecar = join(
    repository,
    "third_party",
    "dom-expressions",
    "packages",
    "jsx-compiler",
    "target",
    "debug",
    sidecarName
  );
  if (
    !existsSync(executable) ||
    (!process.env.SOLID_COMPILER_FACTS_BIN && !existsSync(repositorySidecar))
  ) {
    const build = spawnSync("make", ["build"], {
      cwd: repository,
      env: process.env,
      stdio: "inherit"
    });
    if (build.error) {
      console.error(`solid-check: could not build development binaries: ${build.error.message}`);
      process.exit(2);
    }
    if (build.status !== 0) process.exit(build.status ?? 2);
  }
}

if (!existsSync(executable)) {
  console.error(
    `solid-check: no native binary for ${process.platform}-${process.arch}; ` +
    "set SOLID_CHECK_NATIVE_BIN or install a supported package"
  );
  process.exit(2);
}

const env = { ...process.env };
if (!env.SOLID_COMPILER_FACTS_BIN) {
  const packagedSidecar = packagedBinary(sidecarName);
  if (existsSync(packagedSidecar)) {
    env.SOLID_COMPILER_FACTS_BIN = packagedSidecar;
  } else if (repositorySidecar && existsSync(repositorySidecar)) {
    env.SOLID_COMPILER_FACTS_BIN = repositorySidecar;
  }
}

const child = spawnSync(executable, process.argv.slice(2), {
  cwd: process.cwd(),
  env,
  stdio: "inherit"
});
if (child.error) {
  console.error(`solid-check: could not start ${executable}: ${child.error.message}`);
  process.exit(2);
}
if (child.signal) {
  process.kill(process.pid, child.signal);
}
process.exit(child.status ?? 2);
