# Contributing

The checker, CLI, and LSP are Rust. Go is limited to the TypeScript-Go
`solid-typefacts` service. Keep that boundary explicit: Oxc owns syntax, the
Solid compiler owns execution semantics, and TypeScript-Go owns checker facts.

## Prerequisites

- Go 1.26 or newer
- Rust 1.97 with `rustfmt` and `clippy`
- Node.js 24 and pnpm 11
- `jq`

## Common commands

```sh
make build       # Rust CLI/LSP and Go TypeFacts helper
make test        # TypeFacts, Rust workspace, adapters, compiler, and Zed
make verify      # formatting, vet, Clippy, tests, and schema validation
make package     # native npm package layout
make conformance # controlled compiler conformance
```

Run `make verify` before proposing a change. Changes to compiler execution
semantics must also pass `make conformance`.

## Semantic changes

Add positive and negative fixtures, expose only the required facts, represent
the behavior in Reactive IR, add a fail-closed proof obligation, and return
evidence sufficient to explain each finding. Unsupported behavior that can
affect a proof must produce `uncertifiable`.

Do not infer JSX execution behavior from transformed output. Do not expose
TypeScript-Go or Oxc nodes across fact-domain interfaces.

## Upstream code

The required DOM Expressions compiler sources are maintained as a selective
import under `third_party/dom-expressions`. Follow
[the monorepo policy](docs/monorepo.md) when updating them. Oxc, tsgolint, and
TypeScript-Go remain pinned dependencies.
