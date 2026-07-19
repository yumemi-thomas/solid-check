# solid-check

`solid-check` is a project-level Solid 2 reactivity checker. The production
checker, CLI, and language server are implemented in Rust. TypeScript-Go runs
as the small `solid-typefacts` service and supplies checker-derived facts over
the versioned TypeFacts protocol.

## Components

- `rust/solid-facts-backend`: checker orchestration and `solid-check` CLI
- `rust/solid-lsp`: incremental `solid-checkd` language server
- `rust/solid-reactive-ir` and `rust/solid-reactive-solver`: analysis and rules
- `rust/solid-ast-facts` and `rust/solid-compiler-facts`: Oxc and Solid compiler facts
- `cmd/solid-typefacts` and `internal/typefacts`: retained TypeScript-Go service
- `packages/cli`: npm launcher and native package layout
- `packages/zed-solid-check`: Zed extension
- `third_party/dom-expressions`: controlled Solid compiler fork

The only production language boundary is Rust to the Go TypeFacts service.
Oxc AST facts and Solid compiler facts run in-process.

## Development

Go 1.26, Rust 1.97, and Node.js 24 are required.

```sh
make build
make test
make verify
```

Run the checker:

```sh
SOLID_TYPEFACTS_BIN=bin/solid-typefacts \
  bin/solid-check-rust \
  --project internal/reactiveir/testdata/tracer/tsconfig.json
```

Install the published CLI:

```sh
npm install --save-dev solid-checker
```

Build a distributable native package:

```sh
make package
```

The package exposes the commands as `solid-check` and `solid-checkd` and ships
the matching `solid-typefacts` helper.

Maintainers publish a release by pushing a semantic-version tag such as
`v0.1.0`. For the first publish, add an `NPM_TOKEN` secret to the `npm` GitHub
environment. After the package exists, configure npm trusted publishing for
this repository and `.github/workflows/publish-npm.yml`; subsequent releases
do not need the token.

See [the documentation index](docs/README.md), [Rust architecture](rust/README.md),
and [contribution guide](CONTRIBUTING.md).
