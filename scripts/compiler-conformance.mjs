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

async function analyze(source) {
  const sourceHash = `sha256:${createHash("sha256").update(source).digest("hex")}`;
  sidecar.stdin.write(`${JSON.stringify({
    compilerFactsProtocol: 1,
    path: "/workspace/App.tsx",
    source,
    sourceHash,
    compilerOptions: { moduleName: "dom", generate: "dom" },
  })}\n`);
  const line = await lines.next();
  assert.equal(line.done, false);
  const response = JSON.parse(line.value);
  assert.equal(response.ok, true, response.error?.message);
  return response.executionMap;
}

const childSource = "const view = <div>{count()}</div>;";
const childTransform = transform(childSource, {
  filename: "App.tsx",
  moduleName: "dom",
  generate: "dom",
  compilerFacts: true,
});
assert.match(childTransform.code, /_\$insert\(_el\$, count\)/);
const childFacts = await analyze(childSource);
assert.deepEqual(childFacts, childTransform.executionMap);

const eventSource = "const view = <button onClick={() => setCount(count() + 1)}>Go</button>;";
const eventTransform = transform(eventSource, {
  filename: "App.tsx",
  moduleName: "dom",
  generate: "dom",
  compilerFacts: true,
});
assert.match(eventTransform.code, /\.\$\$click = \(\) => setCount/);
const eventFacts = await analyze(eventSource);
assert.deepEqual(eventFacts, eventTransform.executionMap);

sidecar.stdin.end();
assert.equal(await new Promise((resolve) => sidecar.on("exit", resolve)), 0);
