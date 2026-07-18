# Monorepo and upstream policy

This repository is the build and review home for the complete `solid-check`
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

DOM Expressions is imported as a Git subtree so its upstream ancestry remains
available without requiring Git submodules. Synchronization should be done in a
clean worktree:

```sh
git subtree pull --prefix third_party/dom-expressions \
  https://github.com/ryansolid/dom-expressions.git next
```

Resolve conflicts inside the subtree, then run `make conformance` and
`make verify`. Record the new upstream revision in `THIRD_PARTY_NOTICES.md`.

Oxc, tsgolint, and TypeScript-Go stay pinned dependencies until local source
changes are required. If one becomes a fork, import it under `third_party/`
with its license and provenance rather than maintaining a hidden sibling
checkout.

## Corpus policy

Solid Primitives is a pinned compatibility corpus, not shipped code. Run
`make corpus` to clone the reviewed revision into a temporary directory, build
it, generate contracts to a fixed point, and validate all published artifacts.
Set `SOLID_PRIMITIVES_CORPUS` to reuse an existing clean checkout.
