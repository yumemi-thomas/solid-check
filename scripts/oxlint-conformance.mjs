import { mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const fixtureRoot = join(root, "internal/engine/testdata/oxlint-conformance");
const binary = join(root, ".solid-check", "solid-check-oxlint-conformance");
const compiler = process.env.SOLID_COMPILER_FACTS_BIN ?? join(root, "third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts");
const oxlintVersion = "1.67.0";

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, encoding: "utf8", ...options });
  if (result.error) throw result.error;
  return result;
}

mkdirSync(dirname(binary), { recursive: true });
const build = run("go", ["build", "-o", binary, "./cmd/solid-check"]);
if (build.status !== 0) throw new Error(build.stderr || `go build exited ${build.status}`);

let oxlint = process.env.OXLINT_BIN;
if (!oxlint) {
  const resolved = run("npm", ["exec", "--yes", `--package=oxlint@${oxlintVersion}`, "--", "which", "oxlint"]);
  if (resolved.status !== 0) throw new Error(resolved.stderr || `resolve Oxlint exited ${resolved.status}`);
  oxlint = resolved.stdout.trim();
}
const lint = run(binary, [
  "oxlint", "--project", join(fixtureRoot, "tsconfig.json"), "--",
  "--format=default", "-c", join(fixtureRoot, ".oxlintrc.json"), join(fixtureRoot, "failing.tsx")
], { env: { ...process.env, SOLID_COMPILER_FACTS_BIN: compiler, OXLINT_BIN: oxlint } });
const output = `${lint.stdout}${lint.stderr}`;

if (lint.status === 0) throw new Error(`Oxlint unexpectedly accepted the failing fixture:\n${output}`);
for (const expected of ["[SC1003]", "solid-check(certification)", "destructuring component props", "the destructuring pattern is bound to proven component props"]) {
  if (!output.includes(expected)) throw new Error(`Oxlint output is missing ${JSON.stringify(expected)}:\n${output}`);
}

process.stdout.write(`Oxlint ${oxlintVersion} conformance passed. Failing-case output:\n${output}`);
