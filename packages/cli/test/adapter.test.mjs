import assert from "node:assert/strict";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  writeFileSync
} from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

const require = createRequire(import.meta.url);
const plugin = require("../eslint.cjs");

function sourceCode(text) {
  return {
    text,
    getLocFromIndex(index) {
      const lines = text.slice(0, index).split("\n");
      return { line: lines.length, column: lines.at(-1).length };
    }
  };
}

function run(snapshot, filename, text) {
  const reports = [];
  const context = {
    settings: { solidCheck: { snapshot } },
    options: [],
    sourceCode: sourceCode(text),
    filename,
    physicalFilename: filename,
    report(descriptor) {
      reports.push(descriptor);
    }
  };
  plugin.rules.certification.create(context).Program({ type: "Program" });
  return reports;
}

test("exports an Oxlint-compatible certification plugin", () => {
  const exported = require("solid-checker/eslint");
  assert.equal(exported.meta.name, "solid-check");
  assert.ok(exported.rules.certification);
  assert.equal(
    exported.configs.recommended.rules["solid-check/certification"],
    "error"
  );
});

test("reports only findings belonging to the linted file", () => {
  const root = mkdtempSync(join(tmpdir(), "solid-check-adapter-"));
  const filename = join(root, "App.tsx");
  const other = join(root, "Other.tsx");
  const findings = [filename, other].map((path, index) => ({
    id: `SC100${index + 1}`,
    rule: "strict-read-untracked",
    kind: "violation",
    severity: "error",
    message: "reactive read outside tracking",
    primaryLocation: {
      path,
      startByte: 6,
      endByte: 11,
      line: 1,
      column: 7
    },
    evidence: [{ message: "proven component prop" }]
  }));

  const reports = run({ status: "violation", findings }, filename, "const value = 1;");
  assert.equal(reports.length, 1);
  assert.equal(
    reports[0].data.message,
    "[SC1001] reactive read outside tracking (proven component prop)"
  );
  assert.deepEqual(reports[0].loc, {
    start: { line: 1, column: 6 },
    end: { line: 1, column: 11 }
  });
});

test("projects safe same-file fixes and UTF-8 byte ranges", () => {
  const filename = join(mkdtempSync(join(tmpdir(), "solid-check-adapter-")), "App.tsx");
  const location = {
    path: filename,
    startByte: 4,
    endByte: 9,
    line: 1,
    column: 3
  };
  const reports = run({
    findings: [{
      id: "SC1003",
      rule: "component-props-destructure",
      kind: "violation",
      severity: "error",
      message: "do not destructure props",
      primaryLocation: location,
      fixes: [{
        message: "Keep props",
        applicability: "safe",
        edits: [{ location, newText: "props" }]
      }]
    }]
  }, filename, "😀value");

  const calls = [];
  const edits = reports[0].fix({
    replaceTextRange(range, newText) {
      calls.push({ range, newText });
      return { range, text: newText };
    }
  });
  assert.deepEqual(calls, [{ range: [2, 7], newText: "props" }]);
  assert.equal(edits.length, 1);
  assert.equal(plugin._testing.byteOffsetToIndex("😀value", 4), 2);
});

test("discovers tsconfig and runs native analysis once per project", () => {
  const root = mkdtempSync(join(tmpdir(), "solid-check-adapter-"));
  const sourceRoot = join(root, "src");
  mkdirSync(sourceRoot);
  writeFileSync(join(root, "tsconfig.json"), "{}\n");
  const counter = join(root, "runs.txt");
  const analyzer = join(root, "analyzer.mjs");
  writeFileSync(analyzer, `import { existsSync, readFileSync, writeFileSync } from "node:fs";
const counter = process.argv[2];
const args = process.argv.slice(3);
if (!args.includes("--project") || !args.includes("--format") || !args.includes("json")) {
  process.stderr.write("missing transparent project analysis arguments");
  process.exit(2);
}
const count = existsSync(counter) ? Number(readFileSync(counter, "utf8")) : 0;
writeFileSync(counter, String(count + 1));
process.stdout.write(JSON.stringify({ status: "certified", findings: [] }));
`);

  plugin._testing.snapshotCache.clear();
  const config = {
    command: process.execPath,
    commandArgs: [analyzer, counter]
  };
  for (const name of ["App.tsx", "Other.tsx"]) {
    const filename = join(sourceRoot, name);
    writeFileSync(filename, "export {};\n");
    const context = {
      filename,
      physicalFilename: filename,
      settings: { solidCheck: config },
      options: []
    };
    const snapshot = plugin._testing.loadSnapshot(context);
    assert.equal(snapshot.status, "certified");
    assert.equal(plugin._testing.configuredProject(context, config), join(root, "tsconfig.json"));
  }
  assert.equal(readFileSync(counter, "utf8"), "1");
});

test("reuses an ESLint parser project before filesystem discovery", () => {
  const root = mkdtempSync(join(tmpdir(), "solid-check-adapter-"));
  const project = join(root, "tsconfig.eslint.json");
  const context = {
    filename: join(root, "src", "App.tsx"),
    languageOptions: { parserOptions: { project: "tsconfig.eslint.json" } }
  };
  assert.equal(
    plugin._testing.configuredProject(context, { cwd: root }),
    project
  );
});
