import { mkdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const fixtureRoot = join(root, "internal/engine/testdata/eslint-reactivity-v2");
const binary = join(root, ".solid-check", "solid-check-eslint-conformance");
mkdirSync(dirname(binary), { recursive: true });
const build = spawnSync("go", ["build", "-o", binary, "./cmd/solid-check"], { cwd: root, encoding: "utf8" });
if (build.status !== 0) throw new Error(build.stderr || `go build exited ${build.status}`);

const plugin = require(join(root, "packages/eslint-plugin-solid-check"));
plugin._testing.snapshotCache.clear();
const snapshot = plugin._testing.loadSnapshot({
  settings: { solidCheck: { command: binary, cwd: fixtureRoot, project: "tsconfig.json" } },
  options: [], filename: join(fixtureRoot, "effect-apply-parameter.tsx")
});
const migration = JSON.parse(readFileSync(join(root, "packages/eslint-plugin-solid-check/reactivity-v2-migration.json"), "utf8"));

function sourceCode(text) {
  return { text, getLocFromIndex(index) { const lines = text.slice(0,index).split("\n"); return {line:lines.length,column:lines.at(-1).length}; } };
}
function reportsFor(file, ruleName = "certification") {
  const filename = join(fixtureRoot, file);
  const reports = [];
  const context = { settings:{solidCheck:{snapshot}}, options:[], filename, physicalFilename:filename, sourceCode:sourceCode(readFileSync(filename,"utf8")), report(value){reports.push(value);} };
  plugin.rules[ruleName].create(context).Program({type:"Program"});
  return reports;
}

for (const fixture of migration.sourceFixtures) {
  const findings = snapshot.findings.filter(finding => finding.primaryLocation?.path.endsWith(`/${fixture.file}`));
  const reports = reportsFor(fixture.file, fixture.eslintRule);
  if (fixture.canonicalRule) {
    const finding = findings.find(candidate => candidate.rule === fixture.canonicalRule);
    if (!finding) throw new Error(`${fixture.file}: missing engine rule ${fixture.canonicalRule}`);
    if (!reports.some(report => report.data?.message?.startsWith(`[${finding.id}]`))) throw new Error(`${fixture.file}: ESLint did not project ${finding.id}`);
  }
  if (fixture.absentRule && findings.some(candidate => candidate.rule === fixture.absentRule)) throw new Error(`${fixture.file}: unexpected ${fixture.absentRule}`);
}
console.log(`ESLint migration conformance: ${migration.sourceFixtures.length} source fixtures passed through engine and adapter`);
