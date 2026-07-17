"use strict";

const { readFileSync } = require("node:fs");
const { dirname, isAbsolute, resolve } = require("node:path");
const { spawnSync } = require("node:child_process");

const snapshotCache = new Map();

function contextFilename(context) {
  return context.physicalFilename ?? context.filename ?? context.getPhysicalFilename?.() ?? context.getFilename?.() ?? "<input>";
}

function configuration(context) {
  const settings = context.settings?.solidCheck ?? {};
  const options = context.options?.[0] ?? {};
  return { ...settings, ...options };
}

function loadSnapshot(context) {
  const config = configuration(context);
  if (config.snapshot != null) return config.snapshot;
  const injectedSnapshotPath = process.env.SOLID_CHECK_SNAPSHOT_PATH;
  if (config.snapshotPath != null || injectedSnapshotPath) {
    const path = injectedSnapshotPath ? resolve(injectedSnapshotPath) : resolve(config.cwd ?? process.cwd(), config.snapshotPath);
    const key = `file:${path}`;
    if (!snapshotCache.has(key)) snapshotCache.set(key, JSON.parse(readFileSync(path, "utf8")));
    return snapshotCache.get(key);
  }

  const filename = contextFilename(context);
  const cwd = config.cwd ?? (filename === "<input>" ? process.cwd() : dirname(filename));
  const project = config.project ?? "tsconfig.json";
  const command = config.command ?? process.env.SOLID_CHECK_BIN ?? "solid-check";
  const contracts = Array.isArray(config.contracts) ? config.contracts : [];
  const key = JSON.stringify({ command, cwd, project, contracts });
  if (snapshotCache.has(key)) return snapshotCache.get(key);

  const args = ["--project", project, "--format", "json"];
  for (const contract of contracts) args.push("--contract", contract);
  const result = spawnSync(command, args, { cwd, encoding: "utf8", env: process.env });
  if (result.error) throw new Error(`solid-check adapter could not start ${command}: ${result.error.message}`);
  if (result.status !== 0) throw new Error(`solid-check adapter failed (${result.status}): ${result.stderr.trim()}`);
  const snapshot = JSON.parse(result.stdout);
  snapshotCache.set(key, snapshot);
  return snapshot;
}

function samePath(left, right) {
  const normalize = value => resolve(value).replaceAll("\\", "/");
  return normalize(left) === normalize(right);
}

function byteOffsetToIndex(text, byteOffset) {
  if (byteOffset <= 0) return 0;
  let bytes = 0;
  let index = 0;
  for (const character of text) {
    const width = Buffer.byteLength(character);
    if (bytes + width > byteOffset) break;
    bytes += width;
    index += character.length;
  }
  return index;
}

function findingRange(sourceCode, location) {
  const text = sourceCode.text;
  return [byteOffsetToIndex(text, location.startByte), byteOffsetToIndex(text, location.endByte)];
}

function evidenceSuffix(finding) {
  const messages = [];
  for (const step of finding.evidence ?? []) if (step.message && !messages.includes(step.message)) messages.push(step.message);
  const related = finding.relatedLocations ?? [];
  for (const location of related) {
    const summary = `related: ${location.path}:${location.line}:${location.column}`;
    if (!messages.includes(summary)) messages.push(summary);
  }
  return messages.length === 0 ? "" : ` (${messages.join("; ")})`;
}

function fixForFinding(fixer, finding, sourceCode, filename) {
  if (!Array.isArray(finding.fixes) || finding.fixes.length === 0) return null;
  const fix = finding.fixes.find(candidate => candidate.applicability === "safe" && candidate.edits?.every(edit => samePath(edit.location.path, filename)));
  if (!fix) return null;
  return fix.edits.map(edit => fixer.replaceTextRange(findingRange(sourceCode, edit.location), edit.newText));
}

const adapterSchema = [{ type: "object", additionalProperties: false, properties: {
  command: { type: "string" }, project: { type: "string" }, cwd: { type: "string" },
  contracts: { type: "array", items: { type: "string" } }, snapshotPath: { type: "string" }
}}];

function createSnapshotRule(description, accepts) {
  return { meta: {
    type: "problem",
    docs: { description, recommended: true },
    fixable: "code",
    schema: adapterSchema,
    messages: { finding: "{{message}}" }
  },
  create(context) {
    return { Program(program) {
      const snapshot = loadSnapshot(context);
      const sourceCode = context.sourceCode ?? context.getSourceCode();
      const filename = contextFilename(context);
      for (const finding of snapshot.findings ?? []) {
        if (!accepts(finding)) continue;
        if (finding.primaryLocation && !samePath(finding.primaryLocation.path, filename)) continue;
        const range = finding.primaryLocation ? findingRange(sourceCode, finding.primaryLocation) : [0, 0];
        context.report({
          node: program,
          loc: { start: sourceCode.getLocFromIndex(range[0]), end: sourceCode.getLocFromIndex(range[1]) },
          messageId: "finding",
          data: { message: `[${finding.id}] ${finding.message}${evidenceSuffix(finding)}` },
          fix: finding.fixes?.length ? fixer => fixForFinding(fixer, finding, sourceCode, filename) : undefined
        });
      }
    }};
  }
  };
}

const findingGroups = {
  "no-owned-scope-writes": new Set(["reactive-write-in-owned-scope", "action-called-in-owned-scope"]),
  "no-leaf-owner-operations": new Set(["cleanup-in-forbidden-scope", "primitive-in-leaf-owner", "flush-in-forbidden-scope"])
};

const certificationRule = createSnapshotRule("Report canonical solid-check certification findings", () => true);
const namedRules = Object.fromEntries(Object.entries(findingGroups).map(([name, rules]) => [name, createSnapshotRule(`Report canonical ${name} findings`, finding => rules.has(finding.rule))]));
namedRules["no-untracked-read-in-effect-apply"] = createSnapshotRule("Disallow reactive reads in an untracked createEffect apply callback", finding => finding.rule === "strict-read-untracked" && finding.analysisContext === "createEffect apply callback");
namedRules["no-reactive-read-after-await"] = createSnapshotRule("Disallow reactive reads after dependency tracking ends at await", finding => finding.rule === "reactive-read-after-await");
namedRules["no-destructure"] = createSnapshotRule("Disallow destructuring proven Solid component props", finding => finding.rule === "component-props-destructure");
namedRules["no-stale-props-alias"] = createSnapshotRule("Disallow proven untracked component props reads", finding => finding.rule === "strict-read-untracked" && finding.subjectKind === "component-props");
namedRules["components-return-once"] = createSnapshotRule("Disallow reactive conditions that choose a component return shape", finding => finding.rule === "strict-read-untracked" && finding.analysisContext?.endsWith(" conditional return"));
namedRules["no-untracked-reactivity"] = createSnapshotRule("Disallow proven reactive reads outside tracking", finding => finding.rule === "strict-read-untracked" || finding.rule === "reactive-read-after-await");
namedRules["no-reactive-value-misuse"] = createSnapshotRule("Disallow proven invalid reactive writes and targets", finding => new Set(["reactive-write-in-owned-scope", "invalid-refresh-target", "invalid-affects-target"]).has(finding.rule));
namedRules["no-derived-signal-in-effect"] = createSnapshotRule("Disallow effects that write derived reactive state", finding => finding.rule === "reactive-write-in-owned-scope" && finding.analysisContext === "createEffect compute");

const plugin = {
  meta: { name: "eslint-plugin-solid-check", version: "0.0.0" },
  rules: { certification: certificationRule, ...namedRules },
  configs: {}
};
plugin.configs.recommended = { plugins: { "solid-check": plugin }, rules: { "solid-check/certification": "error" } };
plugin.configs["flat/recommended"] = plugin.configs.recommended;
plugin.configs.compatibility = { plugins:{"solid-check":plugin}, rules:{
  "solid-check/no-owned-scope-writes":"error",
  "solid-check/no-leaf-owner-operations":"error",
  "solid-check/no-untracked-read-in-effect-apply":"warn",
  "solid-check/no-reactive-read-after-await":"warn",
  "solid-check/no-destructure":"error",
  "solid-check/no-stale-props-alias":"warn"
}};
plugin.configs["flat/compatibility"] = plugin.configs.compatibility;

module.exports = plugin;
module.exports._testing = { byteOffsetToIndex, findingRange, loadSnapshot, snapshotCache, findingGroups };
