# affects-keys-on-accessor

`SC7004` · **error** · violation

`affects()` is given a key list, but its target is a signal accessor.

## What it does

Flags two-argument `affects(target, keys)` calls where the target resolves to a
signal accessor rather than a store.

## Why is this bad?

The key array narrows an invalidation declaration to specific paths *inside a
store*. A signal accessor has no paths — it is a single value — so the keys are
meaningless. Either the keys are leftovers from a store refactor, or the wrong
binding is being passed; both deserve a hard error rather than silent acceptance.

## Examples

Examples of **incorrect** code for this rule:

```tsx
const [count, setCount] = createSignal(0);

affects(count, ["value"]); // Signals have no keys.
```

Examples of **correct** code for this rule:

```tsx
affects(count); // Signal target: no keys.

const [store, setStore] = createStore({ todos: [], filter: "all" });
affects(store, ["todos"]); // Store target: keys scope the declaration.
```

## How to fix

Drop the key array for signal targets, or pass the store binding if you meant to
scope invalidation to specific store keys.

## Related

- [invalid-affects-target](invalid-affects-target.md) — target shape rules
- [affects-target-unresolved](affects-target-unresolved.md) — when the target cannot be traced
