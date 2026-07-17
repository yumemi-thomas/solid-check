# Documentation

## Start here

- [Reactivity checker plan](reactivity-checker-plan.md) — product guarantee,
  architecture, module interfaces, conformance strategy, and milestones.
- [Implementation status](implementation-status.md) — delivered scope,
  verification evidence, limits, effort, and next work.
- [Monorepo policy](monorepo.md) — module seams, fork provenance,
  synchronization, and corpus policy.

## Architecture and semantics

- [Glossary](glossary.md) — canonical domain language.
- [Semantic inventory](semantic-inventory.md) — Solid 2 diagnostics and
  supported/unsupported proof boundaries.
- [ESLint rule audit](eslint-rule-audit.md) — complete plugin-rule disposition,
  canonical compatibility names, and false-positive boundary.
- [Type Facts](typefacts.md) and
  [tsgolint extraction](tsgolint-extraction.md) — TypeScript backend seam and
  dependency policy.
- [Compiler facts](compiler-facts.md) — versioned `ExecutionMap` protocol and
  controlled DOM Expressions compiler.
- [Interprocedural solver](interprocedural-solver.md) — effect summaries,
  recursive fixed points, and current coverage limits.
- [Performance](performance.md) — native benchmarks, CPU/allocation profiles,
  and complete CLI timing.

## Acceptance slices

- [Tracer bullet](tracer-bullet.md) — first end-to-end proof and diagnostic.
- [Package contracts](package-contracts.md) — format, discovery, generation,
  trust boundary, and publishing workflow.
- [Milestone 5 evidence](milestone-5.md) — package corpus acceptance record.
- [Milestone 6 evidence](milestone-6.md) — core Solid 2 rules, runtime-only
  boundary, fixtures, generated role matrix, and verification gates.
- [Milestone 7 evidence](milestone-7.md) — language-server protocol, overlays,
  diagnostics, explanations, fixes, and incremental equivalence.
- [Milestone 8 evidence](milestone-8.md) — ESLint snapshot adapter and the
  Reactivity v2 semantic migration map.
- [Zed integration](zed.md) — live semantic diagnostics alongside Oxlint,
  local extension installation, and project settings.

Repository contribution and verification instructions are in
[CONTRIBUTING.md](../CONTRIBUTING.md). Third-party revisions and licenses are in
[THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).
