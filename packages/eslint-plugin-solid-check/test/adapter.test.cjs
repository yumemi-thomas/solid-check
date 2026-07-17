"use strict";

const assert = require("node:assert/strict");
const { mkdtempSync, writeFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const test = require("node:test");
const plugin = require("../index.cjs");

function sourceCode(text) {
  return { text, getLocFromIndex(index) { const prefix = text.slice(0, index); const lines = prefix.split("\n"); return { line: lines.length, column: lines.at(-1).length }; } };
}

function run(snapshot, filename, text, ruleName = "certification") {
  const reports = [];
  const context = {
    settings: { solidCheck: { snapshot } }, options: [], sourceCode: sourceCode(text),
    filename, physicalFilename: filename,
    report(descriptor) { reports.push(descriptor); }
  };
  plugin.rules[ruleName].create(context).Program({ type:"Program" });
  return reports;
}

test("named compatibility rules filter canonical findings exactly", () => {
  const filename = join(mkdtempSync(join(tmpdir(), "solid-check-eslint-")), "App.tsx");
  const location = {path:filename,startByte:0,endByte:5,line:1,column:1};
  const findings = [
    ["SC2001","reactive-write-in-owned-scope"], ["SC2002","action-called-in-owned-scope"],
    ["SC3001","cleanup-in-forbidden-scope"], ["SC3002","primitive-in-leaf-owner"],
    ["SC3003","flush-in-forbidden-scope"], ["SC1001","strict-read-untracked","createEffect apply callback"],
    ["SC1001","strict-read-untracked","Component","accessor"], ["SC1002","reactive-read-after-await","createMemo async computation","accessor"],
    ["SC1001","strict-read-untracked","Card conditional return","component-props"], ["SC1003","component-props-destructure","Card"],
    ["SC2001","reactive-write-in-owned-scope","createEffect compute"], ["SC2001","reactive-write-in-owned-scope","createMemo compute"],
    ["SC6001","invalid-refresh-target"]
  ].map(([id,rule,analysisContext,subjectKind]) => ({id,rule,analysisContext,subjectKind,kind:"violation",severity:"error",message:rule,primaryLocation:location}));
  assert.equal(run({findings},filename,"value", "no-owned-scope-writes").length, 4);
  assert.equal(run({findings},filename,"value", "no-leaf-owner-operations").length, 3);
  assert.equal(run({findings},filename,"value", "no-untracked-read-in-effect-apply").length, 1);
  assert.equal(run({findings},filename,"value", "no-reactive-read-after-await").length, 1);
  assert.equal(run({findings},filename,"value", "no-destructure").length, 1);
  assert.equal(run({findings},filename,"value", "no-stale-props-alias").length, 1);
  assert.equal(run({findings},filename,"value", "components-return-once").length, 1);
  assert.equal(run({findings},filename,"value", "no-untracked-reactivity").length, 4);
  assert.equal(run({findings},filename,"value", "no-reactive-value-misuse").length, 4);
  assert.equal(run({findings},filename,"value", "no-derived-signal-in-effect").length, 1);
});

test("presets never enable aggregate and named semantic rules together", () => {
  assert.equal(plugin.configs.recommended.rules["solid-check/certification"], "error");
  assert.equal(plugin.configs.compatibility.rules["solid-check/certification"], undefined);
  assert.equal(plugin.configs.compatibility.rules["solid-check/no-owned-scope-writes"], "error");
  assert.equal(plugin.configs.compatibility.rules["solid-check/no-leaf-owner-operations"], "error");
  assert.equal(plugin.configs.compatibility.rules["solid-check/no-untracked-read-in-effect-apply"], "warn");
  assert.equal(plugin.configs.compatibility.rules["solid-check/no-reactive-read-after-await"], "warn");
  assert.equal(plugin.configs.compatibility.rules["solid-check/no-destructure"], "error");
  assert.equal(plugin.configs.compatibility.rules["solid-check/no-stale-props-alias"], "warn");
  assert.equal(plugin.configs.compatibility.rules["solid-check/components-return-once"], undefined);
  assert.equal(plugin.configs.compatibility.rules["solid-check/no-untracked-reactivity"], undefined);
  assert.equal(plugin.configs.compatibility.rules["solid-check/no-reactive-value-misuse"], undefined);
  assert.equal(plugin.configs.compatibility.rules["solid-check/no-derived-signal-in-effect"], undefined);
});

test("reports only canonical findings belonging to the linted file", () => {
  const root = mkdtempSync(join(tmpdir(), "solid-check-eslint-"));
  const filename = join(root, "App.tsx");
  const other = join(root, "Other.tsx");
  const findings = [filename, other].map((path, index) => ({ id:`SC200${index+1}`, rule:"reactive-write-in-owned-scope", kind:"violation", severity:"error", message:"write is forbidden", primaryLocation:{path,startByte:6,endByte:11,line:1,column:7}, evidence:[{message:"owned scope"}] }));
  const reports = run({ status:"violation", findings }, filename, "const write = 1;");
  assert.equal(reports.length, 1);
  assert.equal(reports[0].data.message, "[SC2001] write is forbidden (owned scope)");
  assert.deepEqual(reports[0].loc, { start:{line:1,column:6}, end:{line:1,column:11} });
});

test("projects proof-backed same-file safe fixes into ESLint fixers", () => {
  const filename = join(mkdtempSync(join(tmpdir(), "solid-check-eslint-")), "App.tsx");
  const location = {path:filename,startByte:0,endByte:6,line:1,column:1};
  const reports = run({findings:[{id:"SC3001",rule:"cleanup-in-forbidden-scope",kind:"violation",severity:"error",message:"return cleanup",primaryLocation:location,fixes:[{message:"Return cleanup",applicability:"safe",edits:[{location,newText:"return"}]}]}]}, filename, "unsafe();");
  const calls = [];
  const edits = reports[0].fix({ replaceTextRange(range,newText){ calls.push({range,newText}); return {range,text:newText}; } });
  assert.deepEqual(calls, [{range:[0,6],newText:"return"}]);
  assert.equal(edits.length, 1);
});

test("fails closed by reporting project-level uncertifiable findings", () => {
  const filename = join(mkdtempSync(join(tmpdir(), "solid-check-eslint-")), "App.tsx");
  const reports = run({status:"uncertifiable",findings:[{id:"SC0002",rule:"execution-map-unavailable",kind:"uncertifiable",severity:"error",message:"compiler facts unavailable"}]}, filename, "export {};\n");
  assert.equal(reports.length, 1);
  assert.equal(reports[0].data.message, "[SC0002] compiler facts unavailable");
  assert.deepEqual(reports[0].loc, {start:{line:1,column:0},end:{line:1,column:0}});
});

test("converts UTF-8 snapshot bytes to JavaScript UTF-16 indices", () => {
  assert.equal(plugin._testing.byteOffsetToIndex("😀value", 4), 2);
  assert.equal(plugin._testing.byteOffsetToIndex("😀value", 9), 7);
});

test("loads a serialized snapshot without invoking analysis", () => {
  const root = mkdtempSync(join(tmpdir(), "solid-check-eslint-"));
  const path = join(root, "snapshot.json");
  writeFileSync(path, JSON.stringify({status:"certified",findings:[]}));
  const context = { settings:{solidCheck:{snapshotPath:path}}, options:[], filename:join(root,"App.tsx") };
  assert.equal(plugin._testing.loadSnapshot(context).status, "certified");
});

test("loads an orchestrator-injected snapshot without project configuration", () => {
  const root = mkdtempSync(join(tmpdir(), "solid-check-eslint-"));
  const path = join(root, "snapshot.json");
  writeFileSync(path, JSON.stringify({status:"violation",findings:[]}));
  const previous = process.env.SOLID_CHECK_SNAPSHOT_PATH;
  process.env.SOLID_CHECK_SNAPSHOT_PATH = path;
  try {
    plugin._testing.snapshotCache.clear();
    const context = { settings:{}, options:[], filename:join(root,"App.tsx") };
    assert.equal(plugin._testing.loadSnapshot(context).status, "violation");
  } finally {
    if (previous === undefined) delete process.env.SOLID_CHECK_SNAPSHOT_PATH;
    else process.env.SOLID_CHECK_SNAPSHOT_PATH = previous;
  }
});
