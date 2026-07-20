# solid-check

`solid-check` finds reactivity bugs in [Solid 2](https://docs.solidjs.com) projects:
memos that go stale after an `await`, destructured props that silently lose
reactivity, cleanups registered in the wrong scope, and more. It analyzes your
whole TypeScript project — not one file at a time — and either **certifies** the
project's reactive behavior or tells you exactly why it can't.

## Quick start

```sh
npm install --save-dev solid-checker
npx solid-check --project tsconfig.json
```

Diagnostics print as framed source excerpts with severity markers, evidence
labels, and a fix hint — the same style Oxlint uses. Use `--format json` for
machine-readable findings or `--format text` for compact output.

In CI, add `--certify` to fail the build unless the project is fully certified:

```sh
npx solid-check --project tsconfig.json --certify
```

Linux (x64, arm64), macOS (x64, arm64), and Windows (x64) are supported; npm
downloads only the binary matching your platform.

## What it catches

Every finding carries a stable code (`SCxxxx`) and comes in one of two kinds:

- **violation** — the analyzer proved the code misbehaves at runtime.
- **uncertifiable** — the analyzer could not prove the code correct, and the
  rule page explains how to make it provable.

For example, `SC1002` [reactive-read-after-await](docs/rules/reactive-read-after-await.md)
catches async computations that stop reacting:

```tsx
const profile = createMemo(async () => {
  const posts = await fetchPosts();
  // Tracking ended at the await: changing userId() never re-runs this memo.
  return posts.filter((post) => post.author === userId());
});
```

The rules cover tracking and component semantics, writes and actions, cleanup
and ownership, async boundaries, directives, and API shapes. See the
[full rule index](docs/rules/README.md) for every code with examples and fixes.

## Editor integration

The package also ships `solid-checkd`, an incremental language server that
publishes diagnostics for the whole project (including unopened files), keeps
them in sync as you type, and offers preferred quick fixes — for example,
**Fix: Keep component props reactive** on a props-destructuring diagnostic.

For Zed, use the extension in [`packages/zed-solid-check`](packages/zed-solid-check)
and follow the [Zed setup guide](docs/zed.md). Any editor with LSP support can
run `solid-checkd --project tsconfig.json` directly.

## Oxlint integration

To surface project findings through your existing Oxlint run, load the bundled
adapter in `.oxlintrc.json`:

```json
{
  "jsPlugins": ["solid-checker/eslint"],
  "rules": {
    "solid-check/certification": "error"
  }
}
```

The adapter finds the nearest `tsconfig.json`, runs the project analysis once,
and projects the findings into Oxlint. Set `settings.solidCheck.project` if
your config has a nonstandard name or is a solution-style root config.

## StackBlitz, WebContainers, and browser workers

Where spawning a native process isn't possible, import the process-free WASM
API from the same package:

```js
import { checkSync } from "solid-checker";
```

## Publishing a Solid library?

Libraries can ship a `solid-reactivity.json` contract describing the reactive
behavior of their exports, so downstream projects stay certifiable without
analyzing your source. Generate one with `solid-check --emit-contract` and see
[package contracts](docs/package-contracts.md) for details.

## Documentation

- [Rule index](docs/rules/README.md) — every diagnostic, with examples and fixes
- [Documentation index](docs/README.md) — architecture, protocols, glossary
- [Zed integration](docs/zed.md) — editor setup

## Development

The checker, CLI, and language server are written in Rust; a small
TypeScript-Go service (`solid-typefacts`) supplies type-checker facts. Go 1.26,
Rust 1.97, and Node.js 24 are required.

```sh
make build
make test
make verify
```

Run the checker from a source build:

```sh
SOLID_TYPEFACTS_BIN=bin/solid-typefacts \
  bin/solid-check-rust \
  --project internal/reactiveir/testdata/tracer/tsconfig.json
```

`make package` builds the distributable native npm package. See the
[contribution guide](CONTRIBUTING.md) and the
[Rust architecture notes](rust/README.md).
