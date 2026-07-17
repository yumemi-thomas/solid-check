# Contributing

`solid-check` contains a Go certification engine and a controlled Rust/Oxc DOM
Expressions compiler fork. Keep their internal implementations separate: the
versioned `ExecutionMap` protocol is the seam between them.

Read the [architectural plan](docs/reactivity-checker-plan.md),
[implementation status](docs/implementation-status.md), and
[monorepo policy](docs/monorepo.md) before changing semantic behavior.

## Prerequisites

- Go 1.26 or newer
- Rust 1.93 with `rustfmt` and `clippy`
- Node.js 24 and pnpm 11 for compiler conformance and corpus work
- `jq` for schema and bundled-contract validation

## Common commands

```sh
make build        # Go CLI and Rust sidecar
make test         # focused Go and Rust tests
make benchmark    # native cold, snapshot, update, and editor-path baselines
make profile      # CPU and allocation profiles for the warm editor path
make verify       # race tests, vet, formatting, Clippy, schemas and protocols
make conformance  # complete controlled compiler suite and ExecutionMap parity
make corpus       # fresh pinned Solid Primitives build and 98 contracts
```

Run `make verify` before proposing a change. Changes to compiler execution
semantics must also pass `make conformance`. Changes to package-contract
inference or the bundled Solid contracts must pass `make corpus`.

## Semantic changes

Implement new diagnostics as vertical slices:

1. add positive and negative source fixtures;
2. expose only the required Type Facts or compiler facts;
3. represent the behavior in Reactive IR;
4. add the proof obligation and fail-closed behavior;
5. return evidence sufficient to explain the result; and
6. verify CLI and engine snapshots agree.

Do not infer JSX execution behavior from transformed output in the checker.
Record it in the compiler-owned `ExecutionMap`. Do not expose TypeScript or Oxc
nodes through certification interfaces. Unsupported behavior that can affect a
proof must produce `uncertifiable`, never silent certification.

## Package contracts

`solid-reactivity.json` is generated output. Maintainers should not need to
author its full structure manually. Contract changes must remain deterministic,
validate referenced artifact hashes, and fail when an exported reactive effect
cannot be represented safely.

## Upstream code

DOM Expressions is maintained as a Git subtree under
`third_party/dom-expressions`. Follow [docs/monorepo.md](docs/monorepo.md) when
updating it and retain upstream history and licensing. Oxc, tsgolint, and
TypeScript-Go remain pinned dependencies unless local source changes make a
real fork necessary.
