# Milestone 8: ESLint migration

Milestone 8 adds a compatibility adapter for ESLint workflows without adding a
second checker. The package at
`packages/eslint-plugin-solid-check` consumes the same immutable certification
snapshot used by the CLI and LSP.

## Guarantee

The adapter does not import TypeScript, parse Solid syntax, infer component
roles, or implement reactivity rules. It either:

- invokes `solid-check --format json` and caches the resulting project
  snapshot for the lint run;
- reads an already serialized snapshot; or
- consumes an injected snapshot in an embedded/test integration.

Every ESLint report retains the canonical diagnostic ID and message. Proof
evidence and related locations are appended as explanatory context. Safe fixes
are exposed only when every edit belongs to the current file, because ESLint's
rule fixer cannot safely perform a multi-file workspace edit. Findings without
a primary location are reported at the start of each linted file; this keeps
the adapter fail-closed when compiler facts or another project-level proof
dependency is unavailable.

## Reactivity v2 migration

The previous `eslint-plugin-solid-2` type-aware tier combined semantic checks
with style policy. Only semantic checks move to certification:

| Previous rule | Canonical engine rules |
| --- | --- |
| `no-owned-scope-writes` | `reactive-write-in-owned-scope`, `action-called-in-owned-scope` |
| `no-leaf-owner-operations` | `cleanup-in-forbidden-scope`, `primitive-in-leaf-owner`, `flush-in-forbidden-scope` |
| `no-untracked-read-in-effect-apply` | `strict-read-untracked` |
| `no-reactive-read-after-await` | `reactive-read-after-await` |
| `no-destructure` | `component-props-destructure` |
| `no-stale-props-alias` | `strict-read-untracked` with `subjectKind: component-props` |
| `components-return-once` | `strict-read-untracked` with conditional-return context (opt-in) |
| `no-derived-signal-in-effect` | `reactive-write-in-owned-scope` with effect-compute context (opt-in) |
| `no-untracked-reactivity` | canonical strict/after-await read union (opt-in) |
| `no-reactive-value-misuse` | canonical forbidden-write/invalid-target union (opt-in) |

Thirty-five applicable source fixtures are copied into
`internal/engine/testdata/eslint-reactivity-v2` and recorded in
`reactivity-v2-migration.json`. They cover re-exported ownership operations,
effect phases, guaranteed-await tracking, component props and aliases,
conditional component returns, JSX, events, deferred callbacks, helper-name
collisions, non-components, and derived writes.
They execute through native TypeScript-Go facts, Reactive IR, the
solver, snapshot serialization, and the ESLint rule. This migration also added
source-origin type descriptions so imported `solid-js` `Accessor` parameters
and members are not confused with ordinary `() => T` functions, plus support
for omitted tuple elements in re-exported signal bindings.

Syntax and style rules such as `prefer-for`, `prefer-show`, duplicate JSX props,
and self-closing components remain in their ESLint plugin. The semantic subset
of `components-return-once` and `no-stale-props-alias` is now proof-backed.
`no-reactive-read-after-await` is now canonical checker policy: it reports only
Solid-proven accessors in an async tracked computation after an unconditional,
completed top-level statement containing `await`. Conditional awaits and reads
inside nested closures remain clean, preserving the old rule's soundness bias.

Migration can replace semantic rule entries with:

```js
"solid-check/certification": "error"
```

and retains any desired style rules from the existing plugin.

For configurations that need stable per-rule severity and suppression, the
adapter also exposes six non-overlapping legacy names directly. They are exact
filters
over canonical findings, not independent implementations. An optional
`analysisContext` snapshot field distinguishes a proven `createEffect` apply
read from other `STRICT_READ_UNTRACKED` contexts, preventing the compatibility
rule from reporting a broader category than its old name promises.

The presets are intentionally non-overlapping: `recommended` enables the
aggregate rule, while `compatibility` enables only six named filters. Four
broader or overlapping aliases are available only through explicit
configuration.

## Acceptance evidence

- The adapter operates only on serialized `certification.Snapshot` fields.
- File filtering and UTF-8 byte to JavaScript UTF-16 range conversion are
  tested, including astral characters.
- Evidence, related locations, project-level uncertifiable findings, and safe
  same-file fixes are tested.
- The migration manifest requires every old catalog-backed semantic type-aware
family to map to canonical rules and explicitly classifies remaining style
  families.
- `TestNativeEngineMigratesReactivityV2SemanticSourceFixtures` executes all
  35 migrated sources and checks 18 positive and 17 negative outcomes.
- `scripts/eslint-migration-conformance.mjs` builds the real CLI, obtains a
  snapshot using the compiler sidecar, and verifies ESLint projection for the
  same 35 sources through their named compatibility rules.
- Adapter tests run under both `make test` and `make verify`.

No package is published by this milestone; the workspace package is marked
private.

## Oxlint terminal integration

`solid-check oxlint --project tsconfig.json -- [oxlint arguments]` performs one
native project analysis, writes the immutable snapshot to a temporary file,
injects that path only into its Oxlint child process, then deletes it. Oxlint's
formatter output and exit status pass through unchanged. This keeps the framed
Oxlint experience while removing snapshot generation from normal user setup.

Safe canonical fixes flow through the same adapter. The initial `SC1003`
autofix covers simple shorthand component parameter destructures when every
binding reference is in a compiler-recorded JSX expression container. It keeps
the props object intact and rewrites those JSX references. Complex patterns
and non-JSX references are deliberately not fixed.

The same multi-edit fix is presented as a preferred LSP `quickfix`, so Zed and
other LSP clients apply the canonical edit without reimplementing it.

The snapshot-backed semantic rule is intentionally not enabled inside an
editor. Editors use `solid-checkd` so unsaved overlays are analyzed
incrementally; Oxlint can run alongside it for syntax and style rules. See the
[Zed integration guide](zed.md).
