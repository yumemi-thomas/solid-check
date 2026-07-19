# Rust analysis foundations

This workspace is the production analysis backend. It includes the checker,
CLI, and language server.

Fact ownership is deliberately split:

- `solid-ast-facts`: parser-derived source structure from one Oxc AST walk;
- `solid-compiler-facts`: Solid compiler execution roles (`ExecutionMap`);
- `solid-ts-facts`: checker-derived facts decoded from the frozen TypeFacts v2
  protocol;
- `solid-facts-core`: source identity, generations, hashes, and byte spans;
- `solid-facts`: validates and joins the three domains without exposing either
  Oxc or TypeScript-Go nodes.
- `solid-facts-backend`: orchestration, retained caches, certification
  snapshots, contracts, and the CLI;
- `solid-reactive-ir` and `solid-reactive-solver`: native analysis and rules;
- `solid-lsp`: asynchronous incremental LSP scheduling and presentation.

The AST package contains no regular expressions. TypeScript facts contain no
syntax-discovery fallback. Both choices are architectural constraints: Oxc owns
structure and TypeScript-Go owns checker semantics.

The production Rust path has one live seam: Rust to the small Go TypeFacts
service. Oxc AST facts and Solid compiler facts run in-process. The old
compiler-facts sidecar remains only as a differential option selected with
`--compiler`.

The TypeFacts service is `cmd/solid-typefacts`. It retains one TS-Go project
and serves lifecycle v3 plus frozen v2 fact tables over deterministic,
length-prefixed CBOR. Startup verifies the protocol, frozen schema hash, and
build ID before analysis. Incremental overlays, affected paths, cancellation,
out-of-order response correlation, crash recovery, and generation replay are
implemented.

The integration tests launch both the real Go service and the controlled
Oxc/Solid compiler sidecar on the tracer fixture, then join their output with
the Rust Oxc AST facts.

The Rust-led path sends Oxc-derived identifier locations as authoritative
closure seeds. When those seeds are present, the Go closure builder bypasses
its legacy regular-expression discovery. Oxc parsing is bounded by
`available_parallelism`; output is restored to source order before joining.
AST facts are cached by path and source hash, and compiler facts by path,
source hash, and compiler options.

`IncrementalSession` retains the compiler and TypeFacts processes, current
source overlays, generation, and both caches. A v3 update invalidates only
changed/deleted paths; unchanged file facts survive and output remains sorted.

`solid-check-rust` is the diagnostic CLI. It accepts `--project`,
loads the Oxc Solid compiler as an in-process Rust crate, and uses
the sibling `solid-typefacts` executable automatically; `SOLID_TYPEFACTS_BIN`
or `--typefacts` can override it. `--compiler` remains available to compare
the legacy compiler-facts process boundary.
TypeScript-Go supplies the configured project source set, so tsconfig
include/exclude and project resolution are authoritative rather than
reimplemented by a directory walk.

The CLI defaults to text output. `--format json` emits the stable certification
snapshot (`status`, findings, package summaries, and metrics), `--certify`
returns exit code 1 unless the status is `certified`, repeatable `--contract`
flags override discovered contracts, and `--validate-contract` validates a
contract and its artifact hashes without opening a project.

Implemented rule slices are:

- `strict-read-untracked`;
- `reactive-write-in-owned-scope`;
- `action-called-in-owned-scope`;
- `cleanup-in-forbidden-scope`;
- `primitive-in-leaf-owner`;
- `flush-in-forbidden-scope`;
- `invalid-cleanup-return` (with `cleanup-return-unresolved` when TS-Go cannot
  prove a returned call or identifier);
- `missing-effect-function`;
- `sync-node-received-async`;
- `invalid-refresh-target` and `invalid-affects-target`, including refresh
  writes flowing through the owned-scope rule;
- `primitive-in-directive-application`, including primitives in direct
  application callbacks and closures returned through forwarded directive
  factories;
- `no-owner-effect`, `no-owner-cleanup`, `no-owner-boundary`, and
  `settled-cleanup-unowned`, using a fixed-point owner-context graph across
  components, roots, helpers, effect phases, events, and leaf owners;
- `pending-async-untracked-read`, `pending-async-forbidden-scope`, and
  `async-outside-loading-boundary`, with TS-Go async-result provenance and Oxc
  JSX dominance through aliased boundaries, components, and boundary wrappers;
- `reactive-read-after-await`, using TS-Go's dominance-proven
  `callsAfterAwait` facts rather than source-order guesses;
- component props reads, aliases, Solid `merge`, and
  `component-props-destructure`, plus `component-returns-conditionally` for
  reactive return-shape guards, with Oxc binding/member shapes and checker
  identities.

Oxc discovers bindings, options, calls, callback nesting, and function graphs;
TS-Go joins canonical symbols across imports; ExecutionMap classifies tracked
JSX and compiler-managed callbacks. Cleanup return shapes come directly from
Oxc expression kinds; TS-Go resolves locally or remotely declared functions
and call return types. Function summaries are instantiated once per owned
root and at rendering call sites. The fixed-point summaries cover cross-file
helpers, callback parameters, generics and overload implementations, recursive
SCCs, returned closures, and store paths while preserving Go solver
multiplicity through cycles.

```sh
SOLID_TYPEFACTS_BIN=bin/solid-typefacts \
cargo +1.97 run --manifest-path rust/Cargo.toml --bin solid-check-rust -- \
  --format json --certify \
  --project internal/reactiveir/testdata/tracer/tsconfig.json
```

Set `SOLID_CHECK_TIMINGS=1` to emit nanosecond stage timings on stderr. Oxc AST
and Solid compiler facts are produced in parallel per source; deterministic
source order is restored before the TS-Go closure is joined.

`solid-checkd-rust` shares the CLI's contract/IR/solver/snapshot module,
rejects stale document versions, publishes UTF-16 diagnostics and quick
fixes, and cancels superseded analysis across the TypeFacts boundary.

`make package` creates one install tree containing `solid-check`,
`solid-checkd`, and the matching `solid-typefacts` helper plus a checksum
manifest. CI builds and smoke-tests that layout on Darwin and Linux
arm64/amd64 and Windows amd64. Tagged releases publish each layout as a
platform-constrained optional npm package alongside the portable launcher.
The `solid-check-wasm` workspace crate exposes the same in-process analysis
pipeline through napi-rs on `wasm32-wasip1-threads`; its host supplies sources
and TypeFacts directly instead of spawning the native TypeFacts service.
