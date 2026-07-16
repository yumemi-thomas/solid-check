---
"@dom-expressions/runtime": patch
---

Speed up DOM and universal reconcile's symmetric end-swap branch on reorder-heavy
patterns (e.g. `<For>` reverse / large rotations).

The trigger condition is unchanged
(`a[aStart] === b[bEnd-1] && b[bStart] === a[aEnd-1]`), but the body now
walks inward against a single stable front anchor (`a[aStart]`) instead
of issuing two cross-anchored `insertBefore` calls per pair. Each move
targets the same DOM position so the browser's adjacency cache stays
warm and per-call native `insertBefore` cost drops sharply. The inner
loop also continues consuming consecutive symmetric swaps without
re-entering the outer dispatch.

Behaviorally equivalent to the previous implementation: same DOM
mutation count, same correctness surface, no false-positive widening.
Validated against `dom-expressions` reconcile tests, the full Solid
test suite, UIBench `tree/[500]/[reverse]`, and `js-framework-benchmark`
`05_swap1k`.
