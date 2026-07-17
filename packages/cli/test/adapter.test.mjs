import assert from "node:assert/strict";
import { createRequire } from "node:module";
import test from "node:test";

const require = createRequire(import.meta.url);

test("exports the bundled ESLint and Oxlint adapter", () => {
  const plugin = require("solid-check/eslint");
  assert.ok(plugin.rules.certification);
  assert.ok(plugin.configs.recommended);
});
