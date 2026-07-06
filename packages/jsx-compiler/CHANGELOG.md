# @dom-expressions/jsx-compiler

## 0.50.0-next.16

### Patch Changes

- 04849df: Preserve JS value semantics for wrapped `&&` conditions (#532). The dom generate's condition wrap used to emit `memo(() => !!left)() && right`, collapsing every falsy left value to `false` — visibly wrong for component props (`undefined` became `false`, breaking `== null` checks) and a hydration mismatch against the untransformed server output (`{0 && <div/>}` rendered "0" on the server, nothing on the client). `left && right` is exactly `left ? right : left`, so the wrap now emits `memo(() => !!left)() ? right : left`: branching still keys off the memoized truthiness (truthy-value churn never re-creates the right side) while the alternate returns the raw left, matching the server for free. Statically boolean lefts (comparisons, `!x`) keep the plain `memo(() => left)() && right` form — the memo's value is the expression's value, so it's already exact with no second evaluation. Ported identically to the Rust jsx-compiler.
- bb7470e: Give every dynamic child slot its own insertion marker when a parent hosts more than one (solidjs/solid#2830). Adjacent expression slots used to share a marker (`null` at the tail, a shared following sibling, or one reused `<!>` between text), which collapsed them into a single `$$SLOT` ownership region: a node migrating between adjacent slots was destroyed by the slot it left, arrays exchanging members could throw `NotFoundError`, and a slot emptied via `[]` refilled at the wrong position. Slots in multi-slot parents now ride the immediately following static sibling or get a dedicated `<!>` placeholder — the same per-slot geometry hydratable output has always produced, which is why these shapes already worked after hydration. Zero runtime changes; single-slot parents compile byte-identically to before.
- 4f00432: Port the hole id-scope design from the Babel plugin: deferred child holes that can allocate hydration ids are wrapped in `scope()` on both the dom and ssr generates (shared `child_slot_allocates_ids` + dynamic predicates so the generates can't desync), replacing the old `orderedInsert` sibling-thunking machinery. Bare getters simplified from `{sig()}` are re-wrapped as `() => sig()` on the dom side so tagging the scope doesn't mutate the user's function.
- 668264f: Universal JSX now passes compile-time static host props to `createElement(tag, staticProps)` so custom renderers can configure nodes before children are inserted. Dynamic props and elements with spreads continue to use the existing `setProp` / `spread` paths.

## 0.50.0-next.15

### Patch Changes

- dc546f3: Add initial AST-native DOM support for plain dynamic attributes by lowering them through reactive effects and `setAttribute`. The compiler now also supports the full Babel DOM `attributeExpressions` fixture, including DOM child-property, style, class/className, state-property, ref, `prop:*`, and spread attribute lowering.

  The Oxc DOM slice now supports inline event handlers for delegated and native events, follows Babel's updated removal of `on:` namespace-event handling, supports `delegateEvents` / `delegatedEvents` configuration, mirrors the Babel/runtime constants needed for void elements, child properties, namespaces, delegated events, and DOM state-property classification, honors Babel-style `omitLastClosingTag` / `omitNestedClosingTags`, `omitQuotes`, `omitAttributeSpacing`, `inlineStyles`, `effectWrapper: false`, paired `wrapConditionals: false` / `memoWrapper: false` wrapperless mode, `requireImportSource`, `staticMarker`, and `validate` template/update options, lowers known namespaced DOM attributes such as `xlink:href` through `setAttributeNS`, covers additional full Babel DOM fixtures including `components`, `SVG`, `conditionalExpressions`, `customElements`, `fragments`, `insertChildren`, `multipleClassAttributes`, and `SVGComponentPartial`, adds parseable `namespaceElements` coverage, supports Solid-compatible custom element context capture through `contextToCustomElements`, adds Babel-aligned `memo` predicate lowering for DOM conditional children and component props plus fragment/component child dynamic expressions, wraps dynamic DOM child member/call/optional/nullish expressions like Babel, handles optional-chain component children and nested fragment conditionals with Babel-shaped getters, memo wrappers, and empty-fragment arrays, supports hydratable DOM fixture output through `getNextElement` template roots, replays queued hydratable delegated events through `runHydrationEvents`, supports dev hydratable DOM validation walks through `getFirstChild` / `getNextSibling`, starts SSR mode with native element/text lowering through `ssr`, dynamic text interpolation through `escape`, plain dynamic native attributes through `escape(..., true)`, hydratable root template keys through `ssrHydrationKey`, defers later hydratable SSR child slots that allocate hydration IDs after deferred children, full coverage for all checked-in Babel SSR and SSR hydratable fixture families, begins universal mode with native elements, static attributes, dynamic text insertion, component calls through shared prop assembly, spread attributes, spread children, and full coverage for the currently checked universal fixtures, adds dynamic mode with DOM-renderer routing, universal fallback, and hybrid DOM/universal dispatch while avoiding duplicate helper import aliases, validates the supported compiler option surface so non-default unsupported Babel options throw instead of being silently ignored, updates the public README and TypeScript declarations for the current option surface, splits SSR/universal target helpers into submodules, shares common AST construction helpers across targets, and expands component lowering with member-expression prop getters, `@static` opt-out, JSX child arrays, dynamic child getters, configured `builtIns` imports, spread props through `mergeProps`, JSX member component callee construction, component ref normalization for identifier/static/simple optional/call/computed refs, return-statement JSX setup lowering, getter setup lowering, and `this` capture for supported class method/field JSX.

- df03fb8: Move all packages under the `@dom-expressions` npm scope with new names:
  - `dom-expressions` → `@dom-expressions/runtime`
  - `babel-plugin-jsx-dom-expressions` → `@dom-expressions/babel-plugin-jsx`
  - `jsx-dom-expressions-compiler` → `@dom-expressions/jsx-compiler`
  - `hyper-dom-expressions` → `@dom-expressions/hyperscript`
  - `tagged-jsx-dom-expressions` → `@dom-expressions/tagged-jsx`

  The old unscoped names stop receiving `next` prereleases and remain in use
  only by the Solid 1.x maintenance line published from `main`.

  `lit-dom-expressions` is dropped from the prerelease line; it has been
  superseded by `@dom-expressions/tagged-jsx`.

  `@dom-expressions/jsx-compiler` now distributes prebuilt native binaries
  through per-platform packages (`@dom-expressions/jsx-compiler-darwin-x64`,
  `-darwin-arm64`, `-linux-x64-gnu`, `-linux-arm64-gnu`, `-win32-x64-msvc`)
  resolved automatically via `optionalDependencies`, instead of shipping a
  binary inside the main package.
