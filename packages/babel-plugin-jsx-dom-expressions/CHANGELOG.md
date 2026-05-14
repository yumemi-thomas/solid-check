# babel-plugin-jsx-dom-expressions

## 0.50.0-next.12

### Patch Changes

- Port relevant maintenance fixes from the stable branch. Add `omitAttributeSpacing` for strict template attribute spacing, and align `server.js`/`server.d.ts` with the current `client.d.ts` export surface so isomorphic imports continue to resolve on the server.

## 0.50.0-next.11

### Patch Changes

- d5cd499: Remove `on:` namespace event support from compiler, runtime, JSX types, and renderer packages.

## 0.50.0-next.10

### Patch Changes

- ba2c493: Update the JSX compiler source to TypeScript and refresh its generated output expectations for the current Babel and Rollup toolchain.

## 0.50.0-next.9

## 0.50.0-next.8

## 0.50.0-next.7

### Patch Changes

- 0bd165e: Preserve shared class tokens when diffing object keys that contain multiple class names.
  Ensure class-method JSX captures `this` before lifted DOM setup statements run.
- e7831bd: Optimize class arrays with leading static class strings and a fixed-shape class object so the static classes are emitted in the template and dynamic object entries compile to class toggles.
- 10f3250: SSR: group contiguous attribute and `textContent` closures into a single
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

- 3574228: SSR rendering performance pass.

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

- 6ae1a40: Replace the `wrapDynamics` previous-value default-object initializer with optional chaining for both DOM and universal generators. The combined-effect arrow now takes `(_v$, _p$) => …` and reads `_p$?.<n>` instead of receiving an `_p$ = { 0: undefined, 1: undefined, … }` defaulted object literal. Removes a per-render-effect setup allocation, shrinks compiled output, and matches the shape used elsewhere in the runtime. The DOM generator special-cases `textContent` (`!_p$ || a !== _p$.a`) to keep the first-run write semantics.

## 0.50.0-next.6

## 0.50.0-next.5

## 0.50.0-next.4

## 0.50.0-next.3

### Patch Changes

- 4dae801: Normalize the `repository` field in every package to the standard npm
  convention: a `git+https://github.com/ryansolid/dom-expressions.git` URL
  with a `directory` pointing at the package within the monorepo. Restores
  "View source" / "Open in repo" links on the npm registry and unblocks
  tooling that resolves source from package metadata.
- 1cc342c: Unify the compiler's void-element list with the runtime's `VoidElements` set in `dom-expressions/src/constants`. The compiler previously kept its own array (`src/VoidElements.ts`) that still contained the long-deprecated `keygen` and `menuitem` tags. Both have been removed from the HTML standard and are no longer parsed as void by modern browsers, so the compiler now emits closing tags for them — which is the correct behaviour in current browsers and was a latent bug otherwise. All other void elements are unaffected.

## 0.50.0-next.2

### Patch Changes

- 4d14c82: Fix single-dynamic attribute accessors being silently invoked with the
  previous value. Given `<div style={source()} />`, the compiler previously
  emitted `effect(source, (v, p) => style(el, v, p))`, which causes the
  reactive core to call `source(p)` — leaking `prev` into a user-authored
  accessor that the source expression wrote as a zero-arg call. Polymorphic
  accessors (e.g. atom-style signals) would observe an unexpected argument
  and misbehave.

  The compute position now emits `() => source()` so the user's call shape
  is preserved. The prior optimization of unwrapping an IIFE
  (`(() => x)()` → `() => x`) is retained since IIFEs are zero-arg and
  cannot leak `prev`.

  Fixes #510.

- 39c207c: Fix a SyntaxError when an element has 222+ merged dynamic attributes
  (solidjs/solid#2682). The internal identifier generator produced `in` at
  index 221, and since these identifiers are emitted as object shorthand
  destructuring bindings, the resulting `({ …, in }) => …` could not be parsed.
  `getNumberedId` now shifts past any natural index that would encode to a JS
  reserved word, keeping the mapping injective and the output at 2 characters
  for all practical dynamic counts.
- 03da8a5: Fix SSR escaping gaps reachable from JSX, and tighten the compiler so
  redundant runtime `escape` calls drop out of the output.

  Security fixes:
  - `ssrStyle` and `ssrClassName` now attribute-escape object keys, not
    just values. Previously a user-controlled key in `<div style={{…}} />`
    or `<div class={{…}} />` could break out of the surrounding attribute.
  - Dynamic fragment-child expressions (`<>{state.text}</>`) now compile
    to `_$memo(() => _$escape(expr))`. Element-child expressions already
    escaped via `escapeExpression`; fragment children reached SSR through
    a separate path and were concatenated raw.
  - Computed-key object styles (`style={{ [k]: v }}`) escape the key at
    compile time.

  Compiler alignment:
  - SSR now matches DOM in rejecting fragments placed directly inside an
    element: `<div><>…</></div>` is a compile error in both renderers.
    Fragments reached via conditionals (`<div>{cond && <>…</>}</div>`)
    remain legal.

  Compiler optimizations:
  - `escapeExpression` drops the outer `_$escape` wrap on a `JSXFragment`
    when its single significant child is either a dynamic expression
    (compiles to a memoized accessor function, `escape(fn)` is a no-op)
    or a native element (compiles to an `_$ssr(…)` SSR node object,
    `escape(object)` is a no-op). This turns
    `cond && _$escape(_$memo(() => _$escape(state.text)))` into
    `cond && _$memo(() => _$escape(state.text))`, and
    `cond && _$escape(_$ssr(_tmpl$N))` into `cond && _$ssr(_tmpl$N)`.

  SSR fixtures for `components`, `conditionalExpressions`, `fragments`,
  and `attributeExpressions` regenerate. Each security fix has a JSX
  round-trip test in `packages/dom-expressions/test/ssr/jsx.spec.jsx`
  that feeds hostile input through `renderToString`.

- 305d9ce: - SSR: Duplicate attributes in JSX without spreads are now deduplicated —
  `<div class="a" class="b" />` correctly renders as `<div class="b" />`
  (last-wins), matching client behavior. Previously the compiler kept both
  attributes in the output.
  - Client: `setAttributeNS` / `removeAttributeNS` now use matching names when
    clearing namespaced attributes (e.g. `xlink:href`). Previously removal could
    leave the attribute in place because it used the local name while the set
    used the qualified name.
  - Expanded test coverage across all four packages; no other behavior changes.

## 0.50.0-next.1

### Patch Changes

- ee365e0: - `insert()` accepts an optional 5th `options` argument that is forwarded to the
  internal `effect()` call, letting callers (e.g. Solid's `render()`) opt into
  transition-aware initial mounts without otherwise changing `insert`'s
  behavior.
  - SSR: `$dflj(ids)` now materializes every id in the list in a single call
    instead of stopping after the first successful `$dfl`. Callers pass only the
    keys they intend to materialize, which simplifies the primitive and composes
    cleanly for bulk-uncollapse cases (e.g. a group activation revealing several
    held fallbacks at once).
  - SSR: Fix cascading async root holes in the streaming shell. When an inner
    Loading boundary resolved its first chunk while the outer shell was still
    pending, `flushEnd` could call `serializer.flush()` before `doShell()` had
    written the root `_assets` module map, causing seroval to silently drop the
    writes and client hydration to fail with "module was not preloaded". Root
    asset serialization is now memoized and gated on both paths.
  - Type formatting cleanup in `jsx-properties.d.ts`.
