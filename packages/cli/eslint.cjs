"use strict";

const { existsSync, readFileSync } = require("node:fs");
const { dirname, isAbsolute, join, parse, resolve } = require("node:path");
const { spawnSync } = require("node:child_process");

const packageVersion = require("./package.json").version;
const snapshotCache = new Map();

function contextFilename(context) {
  return (
    context.physicalFilename ??
    context.filename ??
    context.getPhysicalFilename?.() ??
    context.getFilename?.() ??
    "<input>"
  );
}

function configuration(context) {
  const settings = context.settings?.solidCheck ?? {};
  const options = context.options?.[0] ?? {};
  return { ...settings, ...options };
}

function findProject(start) {
  let directory = resolve(start);
  for (;;) {
    const candidate = join(directory, "tsconfig.json");
    if (existsSync(candidate)) return candidate;
    const parent = dirname(directory);
    if (parent === directory || directory === parse(directory).root) return undefined;
    directory = parent;
  }
}

function configuredProject(context, config) {
  const filename = contextFilename(context);
  const cwd = resolve(config.cwd ?? process.cwd());
  const parserProject = context.languageOptions?.parserOptions?.project;
  const selected = config.project ?? (
    typeof parserProject === "string"
      ? parserProject
      : Array.isArray(parserProject)
        ? parserProject[0]
        : undefined
  );
  if (selected) return isAbsolute(selected) ? selected : resolve(cwd, selected);
  const start = filename === "<input>" ? cwd : dirname(resolve(filename));
  const discovered = findProject(start);
  if (!discovered) {
    throw new Error(
      `solid-check adapter could not find tsconfig.json from ${start}; ` +
      "set settings.solidCheck.project"
    );
  }
  return discovered;
}

function loadSnapshot(context) {
  const config = configuration(context);
  if (config.snapshot != null) return config.snapshot;
  if (config.snapshotPath != null) {
    const path = resolve(config.cwd ?? process.cwd(), config.snapshotPath);
    const key = `file:${path}`;
    if (!snapshotCache.has(key)) {
      snapshotCache.set(key, JSON.parse(readFileSync(path, "utf8")));
    }
    return snapshotCache.get(key);
  }

  const project = configuredProject(context, config);
  const command = config.command ?? process.env.SOLID_CHECK_BIN ?? process.execPath;
  const commandArgs = config.command || process.env.SOLID_CHECK_BIN
    ? [...(config.commandArgs ?? [])]
    : [join(__dirname, "bin", "solid-check.mjs")];
  const contracts = Array.isArray(config.contracts) ? config.contracts : [];
  const key = JSON.stringify({ command, commandArgs, project, contracts });
  if (snapshotCache.has(key)) return snapshotCache.get(key);

  const args = [
    ...commandArgs,
    "--project",
    project,
    "--format",
    "json"
  ];
  for (const contract of contracts) args.push("--contract", contract);
  const result = spawnSync(command, args, {
    cwd: dirname(project),
    encoding: "utf8",
    env: process.env
  });
  if (result.error) {
    throw new Error(`solid-check adapter could not start analysis: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(
      `solid-check adapter analysis failed (${result.status}): ${result.stderr.trim()}`
    );
  }
  let snapshot;
  try {
    snapshot = JSON.parse(result.stdout);
  } catch (error) {
    throw new Error(`solid-check adapter received invalid JSON: ${error.message}`);
  }
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
  return [
    byteOffsetToIndex(sourceCode.text, location.startByte),
    byteOffsetToIndex(sourceCode.text, location.endByte)
  ];
}

function evidenceSuffix(finding) {
  const messages = [];
  for (const step of finding.evidence ?? []) {
    if (step.message && !messages.includes(step.message)) messages.push(step.message);
  }
  for (const location of finding.relatedLocations ?? []) {
    const summary = `related: ${location.path}:${location.line}:${location.column}`;
    if (!messages.includes(summary)) messages.push(summary);
  }
  return messages.length === 0 ? "" : ` (${messages.join("; ")})`;
}

function fixForFinding(fixer, finding, sourceCode, filename) {
  const fix = finding.fixes?.find(candidate =>
    candidate.applicability === "safe" &&
    candidate.edits?.every(edit => samePath(edit.location.path, filename))
  );
  if (!fix) return null;
  return fix.edits.map(edit =>
    fixer.replaceTextRange(
      findingRange(sourceCode, edit.location),
      edit.newText
    )
  );
}

const adapterSchema = [{
  type: "object",
  additionalProperties: false,
  properties: {
    command: { type: "string" },
    commandArgs: { type: "array", items: { type: "string" } },
    project: { type: "string" },
    cwd: { type: "string" },
    contracts: { type: "array", items: { type: "string" } },
    snapshotPath: { type: "string" }
  }
}];

const certification = {
  meta: {
    type: "problem",
    docs: {
      description: "Report canonical solid-check project findings",
      recommended: true
    },
    fixable: "code",
    schema: adapterSchema,
    messages: { finding: "{{message}}" }
  },
  create(context) {
    return {
      Program(program) {
        const snapshot = loadSnapshot(context);
        const sourceCode = context.sourceCode ?? context.getSourceCode();
        const filename = contextFilename(context);
        for (const finding of snapshot.findings ?? []) {
          const location = finding.primaryLocation;
          if (location?.path && !samePath(location.path, filename)) continue;
          const range = location ? findingRange(sourceCode, location) : [0, 0];
          context.report({
            node: program,
            loc: {
              start: sourceCode.getLocFromIndex(range[0]),
              end: sourceCode.getLocFromIndex(range[1])
            },
            messageId: "finding",
            data: {
              message: `[${finding.id}] ${finding.message}${evidenceSuffix(finding)}`
            },
            fix: finding.fixes?.length
              ? fixer => fixForFinding(fixer, finding, sourceCode, filename)
              : undefined
          });
        }
      }
    };
  }
};

const plugin = {
  meta: { name: "solid-check", version: packageVersion },
  rules: { certification },
  configs: {}
};
plugin.configs.recommended = {
  plugins: { "solid-check": plugin },
  rules: { "solid-check/certification": "error" }
};

module.exports = plugin;
module.exports._testing = {
  byteOffsetToIndex,
  configuredProject,
  findProject,
  loadSnapshot,
  snapshotCache
};
