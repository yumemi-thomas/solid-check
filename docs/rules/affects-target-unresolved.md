# affects-target-unresolved

`SC9003` · **error** · uncertifiable

The target passed to `affects()` cannot be traced back to a Solid source.

## What it does

Flags `affects(target, keys?)` calls where the target identifier cannot be resolved
to a binding the analyzer knows to be a branded Solid source — typically values
that crossed a file or package boundary the analysis cannot see through.

## Why is this analysis-limiting?

`affects()` declares which source a function invalidates; the declaration is only
meaningful (and only accepted at runtime) for branded sources. When the analyzer
cannot trace the target's provenance, it can neither certify the declaration nor
prove it wrong — the finding is uncertifiable until the provenance is visible.

## Examples

Code this rule flags:

```tsx
import { todoStore } from "some-package"; // No contract entry describing what this is.

affects(todoStore, ["items"]); // Store? Snapshot? The analyzer cannot tell.
```

Code that resolves:

```tsx
const [store, setStore] = createStore({ items: [] });
affects(store, ["items"]);
```

## How to fix

Pass the binding created by `createSignal`, `createMemo`, `createStore`, or
`createProjection` directly. If the source is re-exported or wrapped by a package,
declare that export's return kind in the package's reactivity contract so the brand
survives the import — see [package-contracts.md](../package-contracts.md).

## Related

- [invalid-affects-target](invalid-affects-target.md) — the proven-invalid variant
- [refresh-target-unresolved](refresh-target-unresolved.md) — the `refresh()` analogue
