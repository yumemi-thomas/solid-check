# Compiler facts

Solid compiler facts describe original-source JSX execution semantics. The
production Rust checker loads the controlled `@dom-expressions/jsx-compiler`
implementation in-process, so facts come from the same transform branches as
compilation.

An optional differential sidecar exposes a newline-delimited JSON protocol.
Every request carries
`compilerFactsProtocol: 1` and a SHA-256 hash of the exact UTF-8 source bytes;
every successful `ExecutionMap` repeats both values. The Rust client rejects
stale hashes, incompatible protocol versions, and spans outside the original
source.

The hardened DOM contract covers these compiler decisions:

- Dynamic native JSX children are tracked `jsx-child` regions.
- Dynamic native JSX attributes are tracked `jsx-attribute` regions.
- Expressions the compiler renders exactly once are explicit untracked
  regions: template-inlined and unwrapped-insert children (including
  `staticMarker` holes) as `jsx-child`, one-shot `setAttr` attribute values as
  `jsx-attribute`, and by-value component properties and children as
  `component-getter`.
- `on*` JSX values are deferred `event-handler` callbacks rather than tracked
  reads at element creation.
- Component invocations and dynamic component properties are identified;
  property getters are deferred callbacks.
- Function children of configured control-flow built-ins are render callbacks.
- `hydratable`, `dev`, `effectWrapper`, `wrapConditionals`, `staticMarker`, and
  sorted, unique `builtIns` are forwarded exactly to the compiler.
- Fact arrays are sorted deterministically by original UTF-8 byte spans.

Completeness invariant: every `jsx-expression` operation must be covered by a
tracked region, an untracked region, a callback role, or a
`component-property` operation. The IR builder reports uncovered holes as
`SC9004 execution-map-incomplete` unresolved obligations instead of assuming
untracked rendering, so a fact-recording gap makes the file uncertifiable
rather than silently downgrading reads.

Only DOM generation is supported. Other renderer modes, malformed options,
unknown fact kinds, invalid UTF-8 boundaries, stale hashes, and incompatible
protocol versions fail closed.

Build the sidecar from the in-repository compiler fork with Rust 1.97 (Oxc 0.118
requires Rust 1.92 or newer):

```sh
cargo +1.97 build \
  --manifest-path third_party/dom-expressions/packages/jsx-compiler/Cargo.toml \
  --no-default-features --features sidecar --bin solid-compiler-facts
```

Point the CLI at the resulting process to compare it with the in-process path:

```sh
SOLID_COMPILER_FACTS_BIN=third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts \
  bin/solid-check-rust --project tsconfig.json
```

The compiler-conformance check executes both the controlled DOM Expressions
transform and the sidecar against the same sources:

```sh
node scripts/compiler-conformance.mjs \
  third_party/dom-expressions/packages/jsx-compiler \
  third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts
```
