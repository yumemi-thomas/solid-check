# Solid 2 semantic inventory

This inventory bootstraps Milestone 0. It is derived from the current Solid 2
diagnostics RFC on the `next` branch. “Conditional” means static certification
requires type, execution-region, ownership, async, or compiler facts; it does
not mean the checker may ignore the obligation.

| Runtime diagnostic / condition | Static class | Initial proof obligation |
| --- | --- | --- |
| `SIGNAL_WRITE_IN_OWNED_SCOPE` | Conditional | Resolve a reactive write and prove whether its execution region is an owned forbidden scope or an allowed event/action/untracked region. |
| `PENDING_ASYNC_UNTRACKED_READ` | Conditional | Prove async provenance and that the read executes in a tracked, suspendable region. |
| `ASYNC_OUTSIDE_LOADING_BOUNDARY` | Conditional | Prove an async render read is dominated by a compiler-recognized `Loading` boundary. |
| `CLEANUP_IN_FORBIDDEN_SCOPE` | Statically provable | Resolve `onCleanup` and prove its call region is `createTrackedEffect` or `onSettled`. |
| Nested primitive in a leaf owner | Statically provable | Resolve primitive creation and prove the containing callback is a leaf-owner role. |
| Invalid cleanup return value | Conditional | Resolve callback role and prove every returned value is a function or `undefined`. |
| `flush()` in a forbidden scope | Statically provable | Resolve `flush` and prove the call region is a leaf-owner role. |
| Potential infinite loop | Runtime-only initially | Static rules reject known feedback writes, but the runtime iteration limit remains the oracle for data-dependent loops. |
| `STRICT_READ_UNTRACKED` | Conditional | Prove reactive provenance and that the read executes in an untracked component/effect-apply/rendering-function region. |
| `PENDING_ASYNC_FORBIDDEN_SCOPE` | Conditional | Prove async provenance and a read in a non-suspendable leaf-owner region. |
| `NO_OWNER_EFFECT` | Conditional | Resolve effect creation and prove no live owner dominates it. |
| `NO_OWNER_CLEANUP` | Conditional | Resolve cleanup registration and prove no live owner dominates it. |
| `NO_OWNER_BOUNDARY` | Conditional | Use compiler facts to resolve boundary creation and prove no live owner dominates it. |
| `RUN_WITH_DISPOSED_OWNER` | Runtime-only initially | Owner disposal is generally value- and control-flow-dependent; reject unresolved cases when certification depends on them. |

## Explicit unsupported boundaries

Each boundary creates an `uncertifiable` finding when it can affect a reactive
proof obligation:

- `any` or unknown values used as reactive sources, writes, calls, or callbacks;
- `eval`, `Function`, or code generated at runtime;
- unresolved dynamic call targets or property dispatch;
- dependencies with neither analyzable source nor a valid trusted contract;
- compiler options not represented by the compiler-facts protocol;
- mismatched source hashes, paths, spans, or UTF-8/UTF-16 mappings;
- unsupported JavaScript syntax or TypeScript project configurations;
- analyzer failures, missing backends, or stale package contracts.

## Result policy

- No violation and no unresolved obligation: `certified`.
- At least one proven breach: `violation`.
- Otherwise, at least one unresolved obligation: `uncertifiable`.
- `--certify` fails for both `violation` and `uncertifiable`.
