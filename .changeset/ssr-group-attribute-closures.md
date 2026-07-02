---
"@dom-expressions/babel-plugin-jsx": patch
"@dom-expressions/runtime": patch
---

SSR: group contiguous attribute and `textContent` closures into a single
`_$ssrGroup(() => […], N)` call per element so the runtime can resolve
all `N` hole positions with one closure invocation instead of `N`. The
compiler walks each top-level element's `templateValues`, identifies
runs of `≥2` groupable entries (inserts/children break a run, preserving
child isolation), and replaces them with one grouped declarator repeated
`N` times in the `ssr(...)` argument list. `_$ssrGroup` tags the
function with `fn.$g = N` so `ssr()` can dispatch through a fast path
that's gated at the end of the typeof chain — non-function holes pay
nothing for the new branch.

For the async escalation path (group fn throws `NotReadyError`), every
retry slot for the group shares a module-scoped cache keyed on `fn`:
slot 0 evaluates and caches `arr` (success) or `err` (still-pending),
slots `1..N-1` short-circuit on the cached outcome, and the cache
invalidates when slot 0 re-fires next pass. Net retry cost: 1 evaluation
per group per pass on either outcome — `N²` → `N` on success, `N²` → `1`
on failure — with no per-state bookkeeping.

Bench: `+15%` on `search-results` (heavy attribute usage), neutral on
`color-picker` (no qualifying groups). Hydration ids are unaffected:
attribute/textContent expressions never allocate ids, and inserts (which
do) stay outside groups by construction.
