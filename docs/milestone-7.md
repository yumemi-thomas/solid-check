# Milestone 7: incremental LSP

Milestone 7 delivers `solid-checkd`, a stdio Language Server Protocol adapter
over the same `engine.ProjectSession` used by `solid-check`. It does not contain
checker rules or construct a second diagnostic model.

## Editor contract

Start one daemon per TypeScript project:

```sh
SOLID_COMPILER_FACTS_BIN=third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts \
  bin/solid-checkd --project path/to/tsconfig.json
```

The daemon uses standard JSON-RPC `Content-Length` framing and supports:

- `initialize`, `initialized`, `shutdown`, and `exit`;
- full-document `textDocument/didOpen`, `didChange`, and `didClose` sync;
- pushed `textDocument/publishDiagnostics` and pulled
  `textDocument/diagnostic` results;
- `textDocument/codeAction` for proof-backed safe fixes;
- `solid/checkSnapshot` for the canonical certification snapshot; and
- `solid/explainFinding` for the complete finding, evidence chain, related
  locations, and fixes.

Locations are converted from stable UTF-8 byte spans to zero-based UTF-16 LSP
positions. Closing a document removes its overlay and restores the current disk
contents (or deletes the overlay file when it no longer exists). Editor
versions are translated to monotonically increasing engine versions, so a
close/reopen cycle cannot accidentally submit a stale Type Facts update.

## Incremental architecture

Every accepted document change calls `ProjectSession.Update`; the daemon then
requests a new immutable `certification.Snapshot`. Diagnostics, explanations,
and code actions are projections of that snapshot. The LSP never invokes the
solver, Type Facts, or compiler sidecar directly.

This keeps the equivalence guarantee explicit:

- `solid/checkSnapshot` is the exact snapshot returned by the engine, not an
  LSP-specific reconstruction;
- a native integration test alternates the tracer project through multiple
  certified and violating edits and compares the persistent session after each
  edit with a freshly opened clean session over the same disk state; and
- adapter tests cover arbitrary edit sequences, overlay restoration, JSON-RPC
  framing, related information, explanations, safe workspace edits, and UTF-16
  coordinates.

## Limits

The initial protocol intentionally advertises full-document sync. Range-based
edit composition, cancellation scheduling, workspace-folder multiplexing, and
performance budgets are production hardening work rather than alternate
analysis paths. Large-project latency and memory baselines remain Milestone 9.

## Verification

The standard acceptance gate is:

```sh
make verify
```

The LSP-specific suite is also available as `go test -race ./internal/lsp`.
