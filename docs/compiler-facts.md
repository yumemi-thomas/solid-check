# Compiler facts

`solid-compiler-facts` is the persistent Rust/Oxc sidecar for original-source
JSX execution semantics. It is built by the controlled
`@dom-expressions/jsx-compiler` fork, and therefore executes the same transform
implementation and fact-recording branches as the compiler itself.

The protocol is newline-delimited JSON. Every request carries
`compilerFactsProtocol: 1` and a SHA-256 hash of the exact UTF-8 source bytes;
every successful `ExecutionMap` repeats both values. The Go client rejects
stale hashes, incompatible protocol versions, and spans outside the original
source.

The current vertical slice covers these DOM compiler decisions:

- Dynamic native JSX children are tracked `jsx-child` regions.
- Dynamic native JSX attributes are tracked `jsx-attribute` regions.
- `on*` JSX values are deferred `event-handler` callbacks rather than tracked
  reads at element creation.

Other renderer modes and semantic roles fail closed or remain unresolved until
their compiler-conformance fixtures are implemented.

Build the sidecar from the sibling compiler fork with Rust 1.93 (Oxc 0.118
requires Rust 1.92 or newer):

```sh
cargo +1.93 build \
  --manifest-path ../dom-expressions/packages/jsx-compiler/Cargo.toml \
  --no-default-features --features sidecar --bin solid-compiler-facts
```

Point the CLI at the resulting persistent process:

```sh
SOLID_COMPILER_FACTS_BIN=../dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts \
  go run ./cmd/solid-check --project tsconfig.json
```

The compiler-conformance check executes both the controlled DOM Expressions
transform and the sidecar against the same sources:

```sh
node scripts/compiler-conformance.mjs \
  ../dom-expressions/packages/jsx-compiler \
  ../dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts
```
