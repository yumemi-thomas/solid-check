---
"@dom-expressions/babel-plugin-jsx": patch
---

fix(compiler): key the hole `scope()` wrap off the transform's `dynamic` flag instead of the transformed expression shape. The dom generate simplifies `{sig()}` to the bare getter `sig`, which the old `isDeferredChildSlotExpression` predicate didn't count as deferred while the matching ssr arrow was — so the server scope-wrapped the hole and the client didn't, shifting every sibling hydration id after it. Bare getters are re-wrapped as `() => sig()` on the dom side so `scope()` doesn't tag the user's function.
