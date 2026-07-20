# cleanup-return-unresolved

`SC9002` · **error** · uncertifiable

The return value of an effect, tracked-effect, or `onSettled` callback cannot be
resolved statically, so solid-check cannot prove it is a valid cleanup value.

## What it does

Flags callbacks whose return value the analyzer cannot classify as "a function or
`undefined`" — returns of member expressions, call results, or values that flow in
from other files.

## Why is this analysis-limiting?

Solid treats the return value of these callbacks as cleanup; anything other than a
function or `undefined` throws at runtime
(see [invalid-cleanup-return](invalid-cleanup-return.md)). When the return value's
shape cannot be determined, solid-check can neither certify the callback nor prove
it wrong — so the finding is uncertifiable, and the project cannot be certified
until the shape is explicit.

## Examples

Code this rule flags:

```tsx
onSettled(() => {
  return registry.acquire(key); // What does acquire return? The analyzer cannot tell.
});
```

Code that resolves:

```tsx
onSettled(() => {
  const release = registry.acquire(key);
  return () => release(); // A function literal: provably valid cleanup.
});
```

## How to fix

Make the return shape explicit at each return site: return a function literal, a
named local function, or nothing. If the cleanup comes from a helper, capture the
helper's result in a local and return an arrow function that invokes it.

## Related

- [invalid-cleanup-return](invalid-cleanup-return.md) — the proven-invalid variant
