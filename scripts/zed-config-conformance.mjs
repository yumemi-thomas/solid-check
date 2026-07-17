import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const settings = JSON.parse(readFileSync(resolve(root, ".zed/settings.json"), "utf8"));
const solidCheck = settings.lsp?.["solid-check"]?.binary;
const oxlint = settings.lsp?.oxlint?.initialization_options?.settings;

assert.equal(solidCheck?.path, "bin/solid-checkd");
assert.deepEqual(solidCheck?.arguments, [
  "--project",
  "examples/solid-2-dev-app/tsconfig.json"
]);
assert.equal(
  solidCheck?.env?.SOLID_COMPILER_FACTS_BIN,
  "third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts"
);
assert.equal(
  oxlint?.configPath,
  "examples/solid-2-dev-app/.oxlintrc.editor.json"
);

const editorConfig = JSON.parse(
  readFileSync(resolve(root, oxlint.configPath), "utf8")
);
assert.equal(editorConfig.jsPlugins, undefined);
assert.equal(editorConfig.plugins, undefined);
assert.equal(editorConfig.rules, undefined);

for (const language of ["JavaScript", "JSX", "TypeScript", "TSX"]) {
  assert.deepEqual(settings.languages?.[language]?.language_servers?.slice(0, 5), [
    "typescript-ls",
    "solid-check",
    "oxlint",
    "!vtsls",
    "!typescript-language-server"
  ]);
}

console.log("Zed root-worktree configuration conformance passed");
