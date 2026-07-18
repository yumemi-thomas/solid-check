#!/usr/bin/env node

import { spawn } from "node:child_process";
import process from "node:process";

const [server, project] = process.argv.slice(2);
if (!server || !project) {
  throw new Error("usage: lsp-smoke.mjs <solid-checkd> <tsconfig.json>");
}

function frame(message) {
  const payload = Buffer.from(JSON.stringify(message));
  return Buffer.concat([
    Buffer.from(`Content-Length: ${payload.length}\r\n\r\n`),
    payload
  ]);
}

function decodeFrames(bytes) {
  const messages = [];
  let offset = 0;
  while (offset < bytes.length) {
    const boundary = bytes.indexOf("\r\n\r\n", offset);
    if (boundary < 0) throw new Error("truncated LSP header");
    const header = bytes.subarray(offset, boundary).toString("utf8");
    const match = /^Content-Length:\s*(\d+)$/im.exec(header);
    if (!match) throw new Error("missing Content-Length");
    const length = Number(match[1]);
    const start = boundary + 4;
    const end = start + length;
    messages.push(JSON.parse(bytes.subarray(start, end).toString("utf8")));
    offset = end;
  }
  return messages;
}

const childEnvironment = { ...process.env };
delete childEnvironment.SOLID_TYPEFACTS_BIN;
const child = spawn(server, ["--project", project], {
  env: childEnvironment,
  stdio: ["pipe", "pipe", "pipe"]
});
const stdout = [];
const stderr = [];
child.stdout.on("data", chunk => stdout.push(chunk));
child.stderr.on("data", chunk => stderr.push(chunk));
for (const message of [
  { jsonrpc: "2.0", id: 1, method: "initialize", params: { capabilities: {} } },
  { jsonrpc: "2.0", method: "initialized", params: {} },
  { jsonrpc: "2.0", id: 2, method: "shutdown" },
  { jsonrpc: "2.0", method: "exit" }
]) {
  child.stdin.write(frame(message));
}
child.stdin.end();

const timeout = setTimeout(() => child.kill("SIGKILL"), 15_000);
const result = await new Promise((resolve, reject) => {
  child.once("error", reject);
  child.once("exit", (code, signal) => resolve({ code, signal }));
});
clearTimeout(timeout);
if (result.code !== 0 || result.signal) {
  throw new Error(
    `LSP exited code=${result.code} signal=${result.signal}: ${Buffer.concat(stderr)}`
  );
}
const messages = decodeFrames(Buffer.concat(stdout));
const initialize = messages.find(message => message.id === 1);
const shutdown = messages.find(message => message.id === 2);
if (initialize?.result?.capabilities?.positionEncoding !== "utf-16") {
  throw new Error(`invalid initialize response: ${JSON.stringify(initialize)}`);
}
if (!shutdown || shutdown.result !== null) {
  throw new Error(`invalid shutdown response: ${JSON.stringify(shutdown)}`);
}
