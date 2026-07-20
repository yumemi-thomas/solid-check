# Third-party provenance

`solid-checker` is MIT licensed. It contains and depends on third-party software
whose original notices remain applicable.

## DOM Expressions

- Upstream: https://github.com/ryansolid/dom-expressions
- Imported revision: `e717d06cf0eb489375f178a6506247d6b12822ab`
- Location: `third_party/dom-expressions`
- License: MIT; the upstream `LICENSE` is retained in that directory.

Only the JSX compiler sources needed by `solid-checker` are retained. Local
changes add the compiler execution-map interface and `solid-compiler-facts`
sidecar.

## Oxc

- Upstream: https://github.com/oxc-project/oxc
- Version: `0.118`, resolved exactly by the compiler's `Cargo.lock`
- License: MIT

Oxc is consumed as published Rust crates. It is not forked or copied into this
repository.

## tsgolint and TypeScript-Go

- tsgolint revision: `c3269c01a0c894a31330e1b4c3bd4edc6eb7694b`
- TypeScript-Go revision: `2bd066d87f5b`
- Resolution: pinned Go module versions in `go.mod` and `go.sum`
- Licenses: MIT

Only tsgolint's TypeScript-Go shim modules are consumed. Neither repository is
forked or copied into this repository.

## Solid Primitives

- Upstream: https://github.com/solidjs-community/solid-primitives
- Corpus revision: `46e038a1554cdac58b0a2f04cde735f010508061`
- License: MIT

Solid Primitives is fetched only by the optional corpus workflow. Its source is
not redistributed as part of this repository.
