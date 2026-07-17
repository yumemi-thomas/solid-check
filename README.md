# solid-check

`solid-check` is a project-level Solid 2 reactivity certification engine. This
monorepo contains the Go checker, the controlled DOM Expressions compiler fork,
and its Rust/Oxc compiler-facts sidecar so a single checkout can build and test
the complete analysis path.

Start with the [architectural plan](docs/reactivity-checker-plan.md), then read
the [implementation status and roadmap](docs/implementation-status.md). The
[documentation index](docs/README.md) maps the remaining design and verification
documents.

This repository has completed the initial bootstrap, Type Facts slice, first
compiler `ExecutionMap` slice, Reactive IR tracer bullet, interprocedural
solver, package contracts, core Solid 2 coverage, and the incremental language
server, and ESLint compatibility adapter. Milestones 0 through 8 are complete. It currently
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
- the static Solid 2 read, reactive-write/refresh, action, effect-phase,
  ownership, cleanup, async, control-flow, directive, and API-shape rules;
- proof evidence for each finding and a safe terminal leaf-cleanup fix;
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
  and certifies the corrected tracer scenario;
- `solid-checkd`, with open-document overlays, incremental diagnostics,
  related proof locations, explanations, safe fixes, and canonical snapshots;
- a snapshot-only ESLint/Oxlint adapter with Reactivity v2 semantic-rule
  migration and an ephemeral terminal wrapper; and
- a native Zed extension that launches `solid-checkd` alongside Oxlint.

See [docs/compiler-facts.md](docs/compiler-facts.md) for the controlled DOM
Expressions compiler baseline and sidecar setup. See
[docs/tracer-bullet.md](docs/tracer-bullet.md) for the first supported proof.
See [docs/interprocedural-solver.md](docs/interprocedural-solver.md) for the
Milestone 4 summary model and its explicit coverage limits.
See [docs/package-contracts.md](docs/package-contracts.md) for the Milestone 5
contract format, trust boundary, and CLI workflow.
See [docs/milestone-6.md](docs/milestone-6.md) for the completed static catalog,
runtime-only boundary, and acceptance evidence.
See [docs/milestone-7.md](docs/milestone-7.md) for the LSP protocol, incremental
architecture, and clean-versus-incremental equivalence evidence.
See [docs/milestone-8.md](docs/milestone-8.md) for ESLint configuration and the
Reactivity v2 semantic migration map.

The fail-closed behavior is deliberate: an unimplemented or unsupported proof
obligation must never be reported as certified.

## Repository layout

```text
cmd/solid-check/       CLI adapter
cmd/solid-checkd/      stdio LSP adapter
internal/engine/       project session orchestration
internal/lsp/          LSP transport and snapshot presentation
internal/typefacts/    minimal typescript-go integration (Milestone 1)
internal/compilerfacts/Go protocol and persistent sidecar client (Milestone 2)
third_party/
  dom-expressions/     history-preserving compiler fork and Rust/Oxc sidecar
internal/reactiveir/   AST-independent reactive IR
internal/solver/       whole-project effect solver
packages/eslint-plugin-solid-check/ ESLint snapshot adapter
packages/cli/       npm CLI launcher and bundled adapter entry point
packages/zed-solid-check/ Zed language-server extension
examples/solid-2-dev-app/ Runnable Solid 2 + Oxlint development example
pkg/contracts/         package contract model, validator, and file I/O
pkg/certification/     stable external result model
schema/                serialized public contracts
docs/                  semantic inventory and decisions
```

The engine remains a deep module even though the compiler fork is physically in
the same repository. See [docs/monorepo.md](docs/monorepo.md) for upstream
provenance and synchronization policy.

See [CONTRIBUTING.md](CONTRIBUTING.md) for prerequisites, verification commands,
and change guidelines.

For a runnable Solid 2 application with clean and intentionally failing Oxlint
commands, see the [development app example](examples/solid-2-dev-app/README.md).
For live Zed diagnostics, see the [Zed integration guide](docs/zed.md).

## Development

Go 1.26 and Rust 1.93 or newer are required. The root `Makefile` is the common
developer interface:

```sh
make build
make test
make verify

go run ./cmd/solid-check \
  --project internal/typefacts/testdata/aliased-import/tsconfig.json \
  --format json
```

Run Oxlint with one native analysis pass and no persistent snapshot:

```sh
SOLID_COMPILER_FACTS_BIN=third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts \
  bin/solid-check oxlint --project path/to/tsconfig.json -- --format=default
```

Applications can consume the workspace package through the same command shape
planned for the platform-bundled npm distribution:

```json
{
  "devDependencies": {
    "solid-check": "file:../../packages/cli"
  },
  "scripts": {
    "lint": "solid-check oxlint"
  }
}
```

The package exports `solid-check/eslint` for both ESLint and Oxlint adapters,
locates the compiler sidecar automatically, and forwards native output and exit
codes. It remains private and unpublished in this repository.

For Zed, build the tools and extension, then install
`packages/zed-solid-check` once through **zed: extensions → Install Dev
Extension**:

```sh
make zed-setup
```

Build and start the language server for an editor integration:

```sh
make build
SOLID_COMPILER_FACTS_BIN=third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts \
  bin/solid-checkd --project path/to/tsconfig.json
```

Run the end-to-end strict-read tracer fixture with the Rust sidecar:

```sh
cargo +1.93 build \
  --manifest-path third_party/dom-expressions/packages/jsx-compiler/Cargo.toml \
  --no-default-features --features sidecar --bin solid-compiler-facts
SOLID_COMPILER_FACTS_BIN=third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts \
  go run ./cmd/solid-check \
  --project internal/reactiveir/testdata/tracer/tsconfig.json \
  --format json

SOLID_COMPILER_FACTS_BIN=third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts \
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
