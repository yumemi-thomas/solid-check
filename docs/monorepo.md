# Monorepo and upstream policy

This repository is the build and review home for the complete `solid-checker`
analysis path. A contributor needs one clone for the Rust checker, the Go
TypeFacts service, compiler integration, schemas, tests, and corpus automation.

## Module seams

Physical colocation does not merge the module interfaces:

- `rust/solid-facts-backend` orchestrates certification.
- `rust/solid-compiler-facts` owns compiler-fact integration.
- `cmd/solid-typefacts` and `internal/typefacts` own TypeScript-Go facts.
- `third_party/dom-expressions/packages/jsx-compiler` owns JSX execution
  semantics.
- `rust/solid-facts-backend` owns package contracts.

Oxc and compiler facts stay in-process. The versioned `ExecutionMap` protocol
remains available for differential testing.

## Fork policy

`third_party/dom-expressions` is a deliberately narrow source import, not a
complete upstream checkout. Only `packages/jsx-compiler` and the minimal
workspace files required to build its N-API wrapper are retained.

To update it, compare against the recorded upstream revision, import compiler
changes selectively, and review any upstream changes outside the compiler for
dependencies that must also move. Do not restore unrelated runtime packages,
release infrastructure, generated artifacts, or repository tooling. Then run
`make conformance` and `make verify`, and record the new revision in
`THIRD_PARTY_NOTICES.md`.

Oxc, tsgolint, and TypeScript-Go stay pinned dependencies until local source
changes are required. If one becomes a fork, import it under `third_party/`
with its license and provenance rather than maintaining a hidden sibling
checkout.

## Corpus policy

Solid Primitives is a pinned compatibility corpus, not shipped code. Run
`make corpus` to clone the reviewed revision into a temporary directory, build
it, generate contracts to a fixed point, and validate all published artifacts.
Set `SOLID_PRIMITIVES_CORPUS` to reuse an existing clean checkout.
