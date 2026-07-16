# solid-check

`solid-check` is the project-level certification engine proposed by Solid's
[Solid 2 Reactivity Checker Plan](../solid/documentation/solid-2.0/reactivity-checker-plan.md).
It is intentionally separate from the Solid compiler and the ESLint adapter.

This repository has completed the initial bootstrap, Type Facts slice, first
compiler `ExecutionMap` slice, Reactive IR tracer bullet, and first
interprocedural solver. Milestones 0 through 5 are complete. It currently
provides:

- the stable, AST-independent certification result model;
- a fail-closed project session and immutable snapshots;
- a machine-readable snapshot schema;
- a canonical glossary and initial Solid 2 diagnostic inventory;
- a real typescript-go Type Facts adapter for project loading, aliases,
  declarations, cross-file references, resolved calls, and editor overlays;
- a persistent Rust/Oxc sidecar covering tracked JSX children/attributes and
  deferred DOM event handlers;
- cross-file provenance for direct `createSignal` accessor declarations;
- the first `STRICT_READ_UNTRACKED` solver rule, including declaration and
  compiler-region evidence, with certification of its corrected fixture;
- function summaries for cross-file calls, generics, overloads, forwarded
  callbacks, returned closures, recursive SCCs, and direct store-property flow;
- validated `solid-reactivity.json` contracts that can be emitted for a built
  package and loaded by a source-independent consumer analysis;
- automatic dependency-contract discovery, reviewed bundled Solid 2 contracts,
  subpath and re-export support, returned accessor/store summaries, callback
  execution roles, and async metadata;
- generated and artifact-validated contracts for all 98 packages in the Solid
  Primitives `next` branch;
- a minimal `solid-check` CLI that reports the proven tracer-bullet violation
  and certifies the corrected tracer scenario.

See [docs/compiler-facts.md](docs/compiler-facts.md) for the controlled DOM
Expressions compiler baseline and sidecar setup. See
[docs/tracer-bullet.md](docs/tracer-bullet.md) for the first supported proof.
See [docs/interprocedural-solver.md](docs/interprocedural-solver.md) for the
Milestone 4 summary model and its explicit coverage limits.
See [docs/package-contracts.md](docs/package-contracts.md) for the Milestone 5
contract format, trust boundary, and CLI workflow.

The fail-closed behavior is deliberate: an unimplemented or unsupported proof
obligation must never be reported as certified.

## Planned repository boundaries

```text
cmd/solid-check/       CLI adapter
internal/engine/       project session orchestration
internal/typefacts/    minimal typescript-go integration (Milestone 1)
internal/compilerfacts/Go protocol and persistent sidecar client (Milestone 2)
../dom-expressions/    compiler-owned Rust/Oxc sidecar (Milestone 2)
internal/reactiveir/   AST-independent reactive IR
internal/solver/       whole-project effect solver
pkg/contracts/          package contract model, validator, and file I/O
pkg/certification/     stable external result model
schema/                serialized public contracts
docs/                  semantic inventory and decisions
```

## Development

Go 1.26 or newer is required.

```sh
go test ./...
go run ./cmd/solid-check \
  --project internal/typefacts/testdata/aliased-import/tsconfig.json \
  --format json
```

Run the end-to-end strict-read tracer fixture with the Rust sidecar:

```sh
cargo +1.93 build \
  --manifest-path ../dom-expressions/packages/jsx-compiler/Cargo.toml \
  --no-default-features --features sidecar --bin solid-compiler-facts
SOLID_COMPILER_FACTS_BIN=../dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts \
  go run ./cmd/solid-check \
  --project internal/reactiveir/testdata/tracer/tsconfig.json \
  --format json

SOLID_COMPILER_FACTS_BIN=../dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts \
  go run ./cmd/solid-check \
  --project internal/reactiveir/testdata/tracer-corrected/tsconfig.json \
  --format json \
  --certify
```

Certification mode exits non-zero for both violations and uncertifiable
projects:

```sh
go run ./cmd/solid-check --certify
```

Emit and consume a package contract:

```sh
solid-check --project package/tsconfig.json \
  --emit-contract package/solid-reactivity.json \
  --package-name reactive-package \
  --package-version 1.0.0 \
  --declaration-artifact package/index.d.ts

solid-check --project app/tsconfig.json \
  --contract package/solid-reactivity.json \
  --certify
```

Pass `--contract` more than once when a project consumes multiple contracted
packages. Published contracts are normally discovered automatically.

Validate a published contract without analyzing a project:

```sh
solid-check --validate-contract package/solid-reactivity.json
```
