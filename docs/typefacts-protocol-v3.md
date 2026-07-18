# TypeFacts lifecycle protocol v3

TypeFacts v3 adds a retained-project lifecycle without changing frozen v2.
Every deterministic-CBOR frame carries `schema: 3`, a non-zero `requestId`,
the normalized absolute tsconfig `projectId`, and a monotonic `generation`.

Before accepting requests, the service emits one deterministic-CBOR startup
frame containing handshake protocol `1`, the SHA-256 identity of the frozen
v2 schema, and the build ID compiled into both executables. Rust waits at most
five seconds and rejects a missing or mismatched handshake. CLI/LSP launchers
surface this compatibility failure with exit code `3`; there is no fallback
to an unverified service.

Operations:

- `open` validates project identity and the initial generation;
- `update` applies versioned changed or deleted source overlays and advances
  exactly one generation;
- `analyze` accepts separate Oxc `structuralSpans` and Solid compiler
  `compilerSpans`, or semantic entity demands. Stateful clients retain one
  opaque `stateToken`: the first/resynchronizing request sets `resetState`
  and sends a full demand snapshot; later requests send only changed
  path-groups plus `removedDemandPaths`;
- `sources` returns the exact non-declaration source files selected by the
  retained TypeScript-Go project, including overlay bytes. Rust orchestration
  uses this operation instead of approximating tsconfig semantics with a
  filesystem walk;
- `cancel` acknowledges a request identity;
- `close` completes the retained session.

Responses echo request, project, and generation identities and contain either
`ok: true` with a table/affected paths, or a structured `{code, message}`
error. Invalid or stale v3 requests do not terminate the service. Frozen v2
requests remain supported on the same framed stream.

Stateful analyze responses issue the next `stateToken` and use one of three
table modes:

- `full` carries a complete v2-shaped table on first analysis, explicit
  resynchronization, or a non-durable-symbol generation;
- `delta` carries path-grouped source/entity/file replacements and removals,
  plus durable-symbol upserts/removals;
- `reuse` confirms that the retained table is unchanged and carries no table.

The demand set and table advance as one state. A missing/stale token returns
`state-mismatch`; Rust disposes of both retained values and retries once with
a full snapshot. Rust commits the proposed demand set and applies the table
only after a successful response validates. The service checks cancellation
before committing an analyzed generation. A warm token match with no demand
changes is answered directly without rebuilding, converting, or serializing
the unchanged table.

The required differential invariant is:

`apply(delta, retained table) == freshly rebuilt full table`

Tests exercise this across edits, deletion, and demand shrink. Process tests
also cover cancellation and subprocess loss/replay; a restarted sidecar has
no token, so the normal resynchronization path is used.

The service dispatches generation-scoped requests (`open`, `update`,
`analyze`, `sources`, `close`) through a single ordered worker in frame
arrival order, so a client may pipeline an `update` and further requests
against the resulting generation without awaiting intermediate responses.
`cancel` bypasses the ordered queue: the reader fires the target request's
context immediately, and only the acknowledgement is ordered. Responses may
complete out of request order and are correlated exclusively by `requestId`;
response encoding happens off the worker, so a large table encode never
delays the next request's compute.

The Rust client has one reader multiplexer and a synchronized writer, so it can
send `cancel` while `analyze` is outstanding and route out-of-order responses.
Per edit it performs one **edit exchange**: the `update` is written first and
local fact preparation overlaps its round trip; the `analyze` for the new
generation is sent once the update is acknowledged. The update half of an
edit exchange always lands — cancellation applies only to the analyze half.
The LSP serializes state-changing updates, cancels superseded native work, and
propagates cancellation to the active TS-Go analysis. If the subprocess dies,
the retained LSP session restarts it with bounded backoff, repeats the
handshake, reopens the project, and replays the current generation before
retrying.
