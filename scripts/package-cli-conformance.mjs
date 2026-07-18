import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const example = resolve(root, "examples/solid-2-dev-app");
const packageJson = JSON.parse(readFileSync(resolve(example, "package.json"), "utf8"));

assert.equal(packageJson.devDependencies?.["solid-check"], "file:../../packages/cli");
assert.equal(
  packageJson.scripts?.lint,
  "oxlint"
);
assert.equal(packageJson.scripts?.["lint:fix"], "oxlint --fix");
assert.equal(existsSync(resolve(example, "scripts/lint.mjs")), false);

console.log("Packaged CLI example conformance passed");
