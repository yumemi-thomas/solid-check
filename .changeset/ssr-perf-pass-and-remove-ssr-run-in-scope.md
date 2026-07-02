---
"@dom-expressions/runtime": patch
"@dom-expressions/babel-plugin-jsx": patch
---

SSR rendering performance pass.

**Runtime (`dom-expressions`):**

- Inline hole resolution in `ssr()`. Switch from a `(t, ...nodes)` rest
  parameter to an `arguments` walk, eliminating the per-call holes-array
  allocation. Inline `string`/`number`/`null`/`boolean` fast paths skip
  `tryResolveString` for the typical "all-static-after-eval" hole shape; only
  the heavy path (async escalation) materializes the `{ t, h, p }` result.
- Single forward-pass `escape()`. The previous implementation walked the
  string twice in the hot path (`indexOf(delim)` + `indexOf("&")` upfront
  then early-exit on the no-hit case). Replaced with a `charCodeAt` loop
  that bails after one pass for clean strings (the common case), and
  resumes the slow path from the first hit so the clean prefix isn't
  re-scanned.
- Remove the `ssrRunInScope` public export. The function had been a true
  pass-through identity (`fn => fn`) since owner-capture moved into
  `tryResolveString`'s `NotReadyError` handler, and the compiler stopped
  emitting it. With no internal callers and no behavior, the export was
  dead surface area. User code that called it can drop the wrap (it was a
  no-op) or replicate the original deferred-callback owner-capture intent
  in two lines with `getOwner()` + `runWithOwner()`.

**Compiler (`babel-plugin-jsx-dom-expressions`):**

- IIFE elision in statement-position JSX. When `<jsx/>` is the argument of
  a `return` or the initializer of a `const` (the overwhelmingly common
  shapes), the surrounding IIFE is removed and the body lifts to flat
  statements before the parent. Saves one closure allocation + one
  function-call frame per render. Applies to `dom`, `ssr`, and `universal`
  emissions; expression-position JSX (ternary branches, array elements,
  function args) keeps the IIFE since lifting would change observable
  evaluation semantics.
- SSR templates emit hoisted `var` declarations for dynamic-expression temp
  vars instead of wrapping the whole thing in an IIFE. In statement
  position the declarations precede the `ssr(...)` call; in expression
  position they hoist to the enclosing function scope and the
  assignment + call become a comma sequence expression.
- Drop `ssrRunInScope` emission around dynamic SSR expressions. The
  temp-var hoist stays — it's a V8 IC-stability tactic (keeps the `ssr()`
  call site specialized on `Identifier` argument shapes), not an
  evaluation-order requirement. Ordering is preserved by JS left-to-right
  semantics.
- Drop `createComponent` wrap on SSR component invocations. The SSR
  runtime's `createComponent` is `Comp(props || {})`; the compiler always
  emits a real `props` object, so the `|| {}` fallback never fires. Inline
  to a direct `Comp(props)` call. DOM / dev modes keep the wrapper since
  it does real work (`untrack`, dev metadata).

Net effect on representative SSR shapes (color-picker, search-results) is
fewer allocations per render and a flatter call graph through the hot path.
