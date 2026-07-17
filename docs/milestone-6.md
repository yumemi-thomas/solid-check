# Milestone 6: core Solid 2 coverage

Milestone 6 completes the static portion of the current Solid 2 reactive
diagnostic catalog. Each rule is implemented as a vertical slice through
compiler or type facts, Reactive IR, whole-project solving, evidence, and
positive/negative conformance fixtures.

## Delivered coverage

- Reactive reads distinguish JSX, computation, tracked-effect, rendering,
  effect-apply, event, action, untracked, leaf-owner, and directive execution.
- Signal/store setters and `refresh()` propagate through helper call graphs.
  Writes are rejected under owners and allowed in event, action, explicit
  untracked, `onSettled`, tracked-effect, effect-apply, and directive-apply
  phases. Signal `{ ownedWrite: true }` is honored narrowly.
- Invocations returned by `action()` are rejected in owned compute/component
  scopes and allowed in imperative phases, including named event callbacks.
- Leaf-owner rules reject cleanup registration, nested primitives, and
  `flush()` in `onSettled` and `createTrackedEffect`.
- Cleanup return analysis handles block and concise arrows, async callbacks,
  local callback identifiers, local cleanup identifiers, and resolved call
  return types. Unknown return shapes remain uncertifiable.
- Owner presence is propagated across direct calls, callback identifiers,
  roots, computations, event callbacks, effect-apply callbacks, directives,
  and exported helpers. `Loading`/`Errored` aliases resolve through Type Facts.
- Async provenance covers `createMemo`, function-form `createSignal`,
  function-form `createStore`, and `createProjection`, including Promise,
  PromiseLike, and AsyncIterable computations and `<Loading>` dominance.
- Control-flow compiler roles cover `For`, `Show`, `Match`, `Switch`, `Repeat`,
  `Loading`, `Errored`, and `Reveal`; accessor callback parameters are modeled
  separately from `Repeat`'s plain numeric index.
- Directive factories have distinct owned setup and unowned application
  phases. Returned factories propagate through helpers, arrays are represented
  by compiler facts, direct callbacks are checked, application writes are
  allowed, and application-time primitive creation is rejected.
- Static API-shape diagnostics cover missing effect functions, async results on
  `sync: true` nodes, and invalid `refresh()`/`affects()` targets.
- A proof-backed safe fix rewrites a terminal `onCleanup(callback)` inside a
  leaf owner to `return callback`. Rules without a statically equivalent edit
  remain explanation-only.

Canonical, aliased, and namespace-imported primitive calls are resolved by
symbol identity. Source discovery does not treat matching local function names
as Solid primitives.

## Runtime-only boundary

`RUN_WITH_DISPOSED_OWNER`, data-dependent infinite loops and the resulting
`REACTIVITY_HALTED` state, and internal `INVARIANT_VIOLATION` assertions remain
runtime-only. Owner disposal history and scheduler convergence are not claimed
as static facts. These classifications are explicit in
[the semantic inventory](semantic-inventory.md).

## Acceptance evidence

Hand-written end-to-end fixtures cover writes/actions, compute versus apply
reads, cleanup/leaf owners, ownership, async boundaries, control flow,
directives, and static API misuse. A generated execution-role matrix covers
positive and negative solver classifications independently of those fixtures.

The completed gate consists of:

- `make verify` for Go/Rust tests, race checks, formatting, vet, Clippy, schema,
  protocol, and build checks;
- `make conformance` for 2,303 DOM Expressions transformations and
  transformation-versus-`ExecutionMap` parity; and
- `make corpus` for fixed-point generation and artifact validation across all
  98 packages at the pinned Solid Primitives `next` revision.

Milestone completion means the stated static catalog and acceptance suites are
implemented and green. It does not replace Milestone 9's real-application,
mutation, differential, performance, and incremental-equivalence hardening.
