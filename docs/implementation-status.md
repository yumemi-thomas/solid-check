# Implementation status and roadmap

This document records delivery against the milestones in the
[Solid 2 Reactivity Checker Plan](reactivity-checker-plan.md). A milestone marked
complete has met its stated acceptance criterion; it does not imply that the
same subsystem has reached production maturity.

## Current capability

Milestones 0 through 8 are complete. The checker can currently:

- load a TypeScript project through the pinned TypeScript-Go backend;
- combine cross-file type provenance with compiler-owned JSX execution facts;
- report and explain proven untracked reactive reads;
- propagate reactive effects through functions, callbacks, generics,
  overloads, returned closures, recursion, and direct store paths;
- emit, validate, discover, and consume source-independent package contracts;
- use reviewed bundled contracts for `solid-js` and `@solidjs/web`; and
- generate and artifact-validate contracts for all 98 packages in the pinned
  Solid Primitives `next` corpus; and
- prove the static Solid 2 read, write, action, effect-phase, ownership,
  cleanup, async, control-flow, directive, and API-shape diagnostics documented
  in the semantic inventory, with a safe cleanup fix where equivalence is
  statically established; and
- expose the canonical engine snapshot through `solid-checkd`, with
  open-document overlays, incremental diagnostics, related locations,
  explanations, safe fixes, and clean-versus-incremental equivalence tests; and
- project those same findings, explanations, related locations, and safe
  same-file fixes into ESLint without duplicate AST or type analysis; and
- provide ephemeral Oxlint terminal handoff and live Zed diagnostics without a
  user-visible certification snapshot; and
- expose a private npm `solid-check` binary and `solid-check/eslint` adapter
  entry point, with no application-specific launcher or compiler paths.

The checker fails closed: unsupported behavior that can affect a proof produces
an `uncertifiable` result rather than a successful certification.

## Milestone status

| Milestone | Status | Delivered | Remaining or follow-up | One-engineer effort |
| --- | --- | --- | --- | --- |
| 0. Semantic inventory | Complete | Glossary, diagnostic classification, unsupported boundaries, result schema | Track changes to Solid 2 beta diagnostics | 0–2 days per semantic update |
| 1. Native Type Facts | Hardened | Pinned and import-audited TypeScript-Go/tsgolint shims; cross-file symbols, aliases, calls, references; transactional generation-scoped overlays; project references, package exports, mixed JS, and exact UTF-8 ranges | Eventual comparison with the official stable TypeScript API | 1–2 weeks when that API is suitable |
| 2. Compiler `ExecutionMap` | Hardened | In-repository DOM Expressions fork; persistent fail-closed Rust sidecar; deterministic UTF-8 spans; native JSX, component, event, deferred, and built-in render roles; compiler-option parity matrix | Add new roles alongside each Milestone 6 vertical slice | Included in each slice |
| 3. Tracer bullet | Complete | Imported signal provenance, tracked and untracked reads, CLI finding, corrected certification | No milestone work remaining | None |
| 4. Interprocedural solver | Complete | Function summaries, higher-order calls, generics, overloads, closures, SCCs, stores, invalidation | Methods, computed properties, destructuring, wider closure shapes, selective caching | 2–4 weeks hardening |
| 5. Package contracts | Complete | Schema, validator, emitter, loader, discovery, bundles, source-free consumer and 98-package corpus | Maintainer annotations, package publishing integration and upstream adoption | 1–3 weeks productization |
| 6. Core Solid 2 coverage | Complete | Reads, reactive writes/refresh, actions, effect phases, leaf owners, cleanup returns, owner presence, async computations and Loading dominance, control-flow accessors, directive setup/application, static API misuse, generated/hand-written suites, and a proof-backed cleanup fix | Differential runtime expansion and real-application false-positive hardening move to Milestone 9 | 2–4 weeks hardening within Milestone 9 |
| 7. Incremental LSP | Complete | `solid-checkd`, stdio JSON-RPC/LSP transport, full-document overlays, pushed and pulled diagnostics, related proof locations, explanations, safe fixes, canonical snapshots, native clean/incremental edit equivalence, and a local Zed extension | Range-edit composition, cancellation scheduling, multi-root workspaces, marketplace packaging, and latency/memory hardening | 2–4 weeks within Milestone 9 |
| 8. ESLint migration | Complete | Private ESLint flat-config plugin over canonical snapshots; aggregate, six non-overlapping preset rules, and four opt-in compatibility groupings; proven component-props/alias/destructuring and conditional-return analysis; fail-closed project findings; evidence, related locations, safe same-file fixes and UTF-16 ranges; exhaustive rule audit; 35 source-level fixtures through native engine and adapter; ephemeral Oxlint orchestration with native framed output | Package release, broader ESLint 9/10 installation matrix, watch-cache/performance hardening, and upstream adoption; syntax/style rules intentionally remain in their ESLint plugins | 1–2 weeks product hardening plus 2–5 days per downstream repository |
| 9. Hardening and TypeScript API evaluation | Partially started | Race testing, compiler conformance, fail-closed checks, package corpus, repeatable benchmarks/profiles, a per-generation reference index, TypeScript-AST call, variable-binding, async-result, and named-function discovery, build-scoped derived-region caching, and transactional single-file TypeScript program reuse; the measured editor path fell from ~89 ms initially to ~1.75 ms | Real applications, mutation/differential testing, representative latency and retained-memory budgets, remaining export/region regex migration, multi-file incremental scheduling, incremental equivalence at scale, and official API evaluation | 8–16+ weeks |

These estimates assume one experienced compiler/tooling engineer and reasonably
stable Solid 2 semantics. Milestones 6, 7, and parts of 9 can be parallelized by
a small team once the relevant interfaces are held stable.

## Verification evidence

The current monorepo has passed:

- `go test -race ./...` and `go vet ./...`;
- Go and Rust formatting checks;
- Rust Clippy with warnings denied;
- Rust sidecar protocol integration tests;
- malformed-request, cancellation, crash, stale-hash, unknown-kind, ordering,
  and UTF-8 boundary protocol tests;
- 2,303 controlled DOM Expressions compiler tests;
- compiler transformation versus `ExecutionMap` conformance; and
- a fresh build, fixed-point contract generation, and artifact validation for
  all 98 Solid Primitives packages;
- hand-written Milestone 6 positive/negative source fixtures; and
- a generated execution-role matrix for writes, actions, async reads, and
  directive application; and
- LSP protocol, UTF-16 range, overlay lifecycle, arbitrary edit-sequence, and
  real native clean-versus-incremental snapshot equivalence tests; and
- ESLint snapshot projection, fail-closed behavior, fix, range, cache-input,
  and Reactivity v2 migration-map tests; and
- real Oxlint framed-output conformance with an ephemeral snapshot and Zed
  extension command/path tests; and
- 35 migrated source fixtures (18 positive and 17 sound-negative) covering
  re-exported ownership operations, accessor-typed effect-apply reads,
  guaranteed-await reads, namespace imports, component props/aliases,
  JSX, events, unknown callbacks, and structurally identical negatives
  through the real TypeScript-Go engine, CLI, compiler sidecar, and adapter.

Use `make verify` for the standard local gate, `make conformance` for the full
compiler suite, and `make corpus` for the pinned ecosystem check.

## Known limits

The present checker covers the declared static Solid 2 catalog, but it is not
yet production-hardened for every real application shape. Disposed-owner
history, data-dependent scheduler loops/`REACTIVITY_HALTED`, and internal
runtime invariants remain explicitly runtime-only. Dynamic dispatch, unresolved
branded `refresh`/`affects` targets, and cleanup return types that cannot be
proved statically produce `uncertifiable`, never `certified`.

The current source extraction also deliberately rejects or leaves
uncertifiable arbitrary dynamic dispatch, relevant `any` flows, runtime code
generation, unsupported compiler options, stale contracts, and dependencies
with neither analyzable source nor a trusted contract.

## Next milestone

Milestone 9 is the remaining roadmap destination for runtime differential testing, real
applications, mutation testing, performance budgets, memory profiling, and
broader source-shape and LSP hardening.
