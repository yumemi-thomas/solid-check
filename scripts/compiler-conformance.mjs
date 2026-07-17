import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import path from "node:path";
import { createInterface } from "node:readline";
import { spawn } from "node:child_process";

const [, , compilerPackage, sidecarBinary] = process.argv;
if (!compilerPackage || !sidecarBinary) {
  throw new Error("usage: node compiler-conformance.mjs <jsx-compiler-package> <sidecar-binary>");
}

const require = createRequire(import.meta.url);
const { transform } = require(path.resolve(compilerPackage));
const sidecar = spawn(path.resolve(sidecarBinary), [], { stdio: ["pipe", "pipe", "inherit"] });
const lines = createInterface({ input: sidecar.stdout })[Symbol.asyncIterator]();

async function analyze(source, compilerOptions = {}) {
  const sourceHash = `sha256:${createHash("sha256").update(source).digest("hex")}`;
  sidecar.stdin.write(`${JSON.stringify({
    compilerFactsProtocol: 1,
    path: "/workspace/App.tsx",
    source,
    sourceHash,
    compilerOptions: {
      moduleName: "dom",
      generate: "dom",
      ...compilerOptions,
    },
  })}\n`);
  const line = await lines.next();
  assert.equal(line.done, false);
  const response = JSON.parse(line.value);
  assert.equal(response.ok, true, response.error?.message);
  return response.executionMap;
}

function transformOptions(compilerOptions = {}) {
  const options = {
    filename: "App.tsx",
    moduleName: "dom",
    generate: "dom",
    compilerFacts: true,
    ...compilerOptions,
  };
  if (options.effectWrapper === "") options.effectWrapper = false;
  return options;
}

async function assertParity(source, compilerOptions = {}) {
  const direct = transform(source, transformOptions(compilerOptions));
  const sidecarFacts = await analyze(source, compilerOptions);
  assert.deepEqual(sidecarFacts, direct.executionMap);
  return direct;
}

const childSource = "const view = <div>{count()}</div>;";
const childTransform = await assertParity(childSource);
assert.match(childTransform.code, /_\$insert\(_el\$, count\)/);

const eventSource = "const view = <button onClick={() => setCount(count() + 1)}>Go</button>;";
const eventTransform = await assertParity(eventSource);
assert.match(eventTransform.code, /\.\$\$click = \(\) => setCount/);

await assertParity("const 東京 = () => 1;\r\nconst view = <div>{東京()}</div>;", {
  hydratable: true,
  dev: true,
});
await assertParity("const view = <Comp value={count()} />;");
await assertParity(
  "const view = <For each={items()}>{item => <span>{item()}</span>}</For>;",
  { builtIns: ["For"] },
);
await assertParity("const view = <div title={count()} />;", {
  effectWrapper: "",
});
await assertParity("const view = <div>{/*@once*/ count()}</div>;", {
  staticMarker: "@once",
});
await assertParity("const view = <div>{ready() && value()}</div>;", {
  wrapConditionals: false,
});
await assertParity("const view = <button ref={tooltip(options())}>Save</button>;");

// Untracked-region facts: holes the compiler renders once must be classified
// explicitly, and every jsx-expression operation must stay covered by a
// tracked region, untracked region, callback role, or component-property.
function assertJsxExpressionsCovered(executionMap) {
  const covers = (outer, inner) => outer.start <= inner.start && inner.end <= outer.end;
  for (const operation of executionMap.jsxOperations) {
    if (operation.kind !== "jsx-expression") continue;
    const covered =
      executionMap.trackedRegions.some((region) => covers(region.span, operation.span)) ||
      executionMap.untrackedRegions.some((region) => covers(region.span, operation.span)) ||
      executionMap.callbackRoles.some((role) => covers(role.span, operation.span)) ||
      executionMap.jsxOperations.some(
        (other) => other.kind === "component-property" && covers(other.span, operation.span),
      );
    assert.ok(covered, `jsx-expression at ${JSON.stringify(operation.span)} is uncovered`);
  }
}

const staticHole = await assertParity("const view = <div>{/*@static*/ count()}{label}</div>;");
assert.deepEqual(
  staticHole.executionMap.untrackedRegions.map((region) => region.reason),
  ["jsx-child", "jsx-child"],
);
assertJsxExpressionsCovered(staticHole.executionMap);

const staticAttribute = await assertParity("const view = <div title={label}>{'inline'}</div>;");
assert.deepEqual(
  staticAttribute.executionMap.untrackedRegions.map((region) => region.reason),
  ["jsx-attribute", "jsx-child"],
);
assertJsxExpressionsCovered(staticAttribute.executionMap);

const staticComponent = await assertParity("const view = <Comp note={label}>{0}</Comp>;");
assert.deepEqual(
  staticComponent.executionMap.untrackedRegions.map((region) => region.reason),
  ["component-getter", "component-getter"],
);
assertJsxExpressionsCovered(staticComponent.executionMap);

for (const covered of [
  childTransform,
  eventTransform,
  await assertParity("const view = <div>{first}{second()}</div>;"),
  await assertParity("const view = <div>{/*@once*/ count()}</div>;", { staticMarker: "@once" }),
]) {
  assertJsxExpressionsCovered(covered.executionMap);
}

sidecar.stdin.end();
assert.equal(await new Promise((resolve) => sidecar.on("exit", resolve)), 0);
