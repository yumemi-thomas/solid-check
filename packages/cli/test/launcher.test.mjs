import assert from "node:assert/strict";
import { chmodSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { mkdtempSync } from "node:fs";
import test from "node:test";

test("forwards arguments, output, environment, and the native exit code", () => {
  const directory = mkdtempSync(join(tmpdir(), "solid-check-cli-"));
  const native = join(directory, "solid-check-native");
  const capture = join(directory, "arguments");
  writeFileSync(native, `#!/bin/sh
printf '%s\n' "$@" > "$SOLID_CHECK_TEST_CAPTURE"
printf 'native stdout\n'
printf 'native stderr\n' >&2
exit 7
`);
  chmodSync(native, 0o700);

  const result = spawnSync(process.execPath, [
    new URL("../bin/solid-check.mjs", import.meta.url).pathname,
    "--project",
    "tsconfig.json",
    "--format",
    "json"
  ], {
    encoding: "utf8",
    env: {
      ...process.env,
      SOLID_CHECK_NATIVE_BIN: native,
      SOLID_CHECK_TEST_CAPTURE: capture
    }
  });

  assert.equal(result.status, 7);
  assert.equal(result.stdout, "native stdout\n");
  assert.equal(result.stderr, "native stderr\n");
  assert.deepEqual(readFileSync(capture, "utf8").trim().split("\n"), [
    "--project",
    "tsconfig.json",
    "--format",
    "json"
  ]);
});
