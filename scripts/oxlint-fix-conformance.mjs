import assert from "node:assert/strict";
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const sourceFixture = join(root, "internal/engine/testdata/eslint-reactivity-v2");
const binary = join(root, ".solid-check", "solid-check-oxlint-fix-conformance");
const compiler = process.env.SOLID_COMPILER_FACTS_BIN ?? join(
  root,
  "third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts"
);
const adapter = join(root, "packages/eslint-plugin-solid-check/index.cjs");
const temporary = mkdtempSync(join(tmpdir(), "solid-check-oxlint-fix-"));

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, encoding: "utf8", ...options });
  if (result.error) throw result.error;
  return result;
}

try {
  cpSync(sourceFixture, temporary, { recursive: true });
  const target = join(temporary, "component-props-parameter-destructure.tsx");
  const config = join(temporary, ".oxlintrc.json");
  writeFileSync(config, JSON.stringify({
    jsPlugins: [{ name: "solid-check", specifier: adapter }],
    rules: { "solid-check/certification": "error" }
  }));

  mkdirSync(dirname(binary), { recursive: true });
  const build = run("go", ["build", "-o", binary, "./cmd/solid-check"]);
  if (build.status !== 0) throw new Error(build.stderr || `go build exited ${build.status}`);

  let oxlint = process.env.OXLINT_BIN;
  if (!oxlint) {
    const resolved = run("npm", ["exec", "--yes", "--package=oxlint@1.67.0", "--", "which", "oxlint"]);
    if (resolved.status !== 0) throw new Error(resolved.stderr || `resolve Oxlint exited ${resolved.status}`);
    oxlint = resolved.stdout.trim();
  }
  const env = { ...process.env, SOLID_COMPILER_FACTS_BIN: compiler, OXLINT_BIN: oxlint };
  const fixed = run(binary, [
    "oxlint",
    "--project", join(temporary, "tsconfig.json"),
    "--fix",
    "--config", config,
    target
  ], { env });
  if (fixed.status !== 0) {
    throw new Error(`Oxlint fix exited ${fixed.status}:\n${fixed.stdout}${fixed.stderr}`);
  }

  assert.equal(readFileSync(target, "utf8"), `function Card(props: { title: string }) {
  return <h1>{props.title}</h1>;
}

export { Card };
`);

  const clean = run(binary, [
    "oxlint",
    "--project", join(temporary, "tsconfig.json"),
    "--config", config,
    target
  ], { env });
  if (clean.status !== 0) {
    throw new Error(`fixed source did not lint cleanly:\n${clean.stdout}${clean.stderr}`);
  }
  console.log("Oxlint safe-fix conformance passed");
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
