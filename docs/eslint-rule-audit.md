# ESLint rule audit

This is the false-positive-focused disposition of the rule catalogs in
`eslint-plugin-solid-2` and `eslint-plugin-solid`. The central rule is that
`solid-check` implements only checks that need project-wide Solid 2 proof. A
syntax-only ESLint rule remains an ESLint rule; copying it into the
certification engine would add a second parser and no stronger guarantee.

## Rules implemented by the canonical engine

| Compatibility name | Canonical proof | Default compatibility preset |
| --- | --- | --- |
| `no-owned-scope-writes` | Proven Solid setter, `refresh`, or action plus execution/owner role | Error |
| `no-leaf-owner-operations` | Proven cleanup, primitive, or `flush` call plus leaf-owner role | Error |
| `no-untracked-read-in-effect-apply` | Proven reactive provenance plus exact `createEffect apply callback` context | Warning |
| `no-reactive-read-after-await` | Proven Solid accessor, tracked async computation, and AST-proven await dominance on every reachable path | Warning |
| `no-destructure` | Destructuring is reported only for a confirmed rendering component's props symbol or stable alias | Error |
| `no-stale-props-alias` | Untracked read is reported only when `subjectKind` is proven `component-props`; aliases and Solid `merge`/`omit` proxies preserve identity | Warning |
| `components-return-once` | Only a proven reactive read controlling a component return is selected | Opt-in; overlaps props/aggregate findings |
| `no-untracked-reactivity` | Union of canonical untracked-read and after-await findings | Opt-in broad grouping |
| `no-reactive-value-misuse` | Union of canonical forbidden writes and invalid `refresh`/`affects` targets | Opt-in broad grouping |
| `no-derived-signal-in-effect` | Proven forbidden signal write whose exact context is `createEffect compute` | Opt-in; overlaps owned-scope findings |

The `recommended` preset exposes the complete canonical result as one
`certification` rule. The `compatibility` preset uses six non-overlapping
legacy names. Broader or overlapping aliases are exported but deliberately
not enabled by a preset.

## `eslint-plugin-solid-2` disposition

| Rule | Decision | Reason |
| --- | --- | --- |
| `components-return-once` | Canonical adapter, opt-in | Safe only for a proven reactive return guard; static component shape alone is too broad. |
| `jsx-no-duplicate-props` | Keep syntax-local | Exact JSX attribute duplication needs no type or execution proof. |
| `no-destructure` | Canonical adapter | Component and props-symbol identity are proven before reporting. |
| `no-leaf-owner-operations` | Canonical adapter | Requires Solid call provenance and owner-role proof. |
| `no-owned-scope-writes` | Canonical adapter | Requires write provenance and execution-role proof. |
| `no-reactive-read-after-await` | Canonical adapter | Requires accessor provenance and control-flow dominance. |
| `no-stale-props-alias` | Canonical adapter | Requires props/alias provenance and compiler execution regions. |
| `no-untracked-read-in-effect-apply` | Canonical adapter | Requires exact effect phase and reactive provenance. |
| `prefer-for` | Keep style-local | This is a performance/style preference, not a certification failure. |
| `prefer-show` | Keep style-local | This is a rendering-style preference. |
| `self-closing-comp` | Keep style-local | Formatting-only. |

## `eslint-plugin-solid` disposition

| Rule or family | Decision | Reason |
| --- | --- | --- |
| `components-return-once` | Canonical adapter, opt-in | Uses the proof-backed subset described above. |
| `no-destructure` | Canonical adapter | Uses proven Solid 2 component props. |
| `no-derived-signal-in-effect` | Canonical adapter, opt-in | Exact effect-compute context over a canonical forbidden write. |
| `no-untracked-reactivity` | Canonical adapter, opt-in | Broad canonical grouping; not duplicated in presets. |
| `no-reactive-value-misuse` | Canonical adapter, opt-in | Broad canonical grouping; not duplicated in presets. |
| `reactivity` | Use `solid-check/certification` | The canonical aggregate is more precise and includes unresolved-proof failure. |
| `no-async-tracked-computation` | Do not port | Solid 2 supports async computations; a blanket prohibition encodes obsolete semantics. |
| `require-analyzable-reactivity` | Do not port | Type Facts, compiler facts, and fail-closed certification replace heuristic analyzability policy. |
| `event-handlers`, `imports`, `jsx-no-duplicate-props`, `jsx-no-script-url`, `jsx-no-undef`, `jsx-uses-vars`, `no-array-handlers`, `no-innerhtml`, `no-invalid-enumerated-attributes`, `no-proxy-apis`, `no-react-deps`, `no-react-specific-props`, `no-unknown-namespaces`, `style-prop`, `validate-jsx-nesting` | Keep syntax/API-local | These are valuable ESLint checks, but project-wide reactive proof would not improve them. Keep using the established plugin implementation. |
| `prefer-classlist` | Do not recommend for Solid 2 | Solid 2 accepts class object/array forms; enforcing the older preference can reject valid idioms. |
| `prefer-for`, `prefer-show`, `self-closing-comp` | Keep style-local | Useful policy, not certification semantics. |

## False-positive boundary

The migration corpus contains 35 complete source files: 18 positive cases and
17 sound-negative cases. Negatives cover structurally similar plain functions,
local accessors, conditional/nested awaits, non-components, local helper name
collisions, stable props pass-through, tracked computations, direct JSX,
event handlers, arbitrary deferred callbacks, static return guards, and the
effect apply phase. The real conformance test sends all fixtures through
TypeScript-Go, the controlled compiler sidecar, the native engine, JSON
serialization, and the ESLint adapter.

Unknown callback timing is intentionally not diagnosed. Unknown reactive
provenance or missing compiler facts makes the project `uncertifiable` rather
than guessing that a rule violation exists.
