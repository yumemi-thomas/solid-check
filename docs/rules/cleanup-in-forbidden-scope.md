# cleanup-in-forbidden-scope

`SC3001` · **error** · violation · 🛠️ safe fix available

`onCleanup` is called inside a leaf owner (`onSettled` or `createTrackedEffect`),
which manages cleanup through its return value instead.

## What it does

Flags `onCleanup` calls that are lexically contained in an `onSettled` or
`createTrackedEffect` callback. When the `onCleanup` call is the trailing statement
of the callback, solid-check offers a safe fix that rewrites it to a `return`.

This is the static counterpart of Solid's dev-mode `CLEANUP_IN_FORBIDDEN_SCOPE`
error.

## Why is this bad?

`onSettled` and `createTrackedEffect` are leaf owners: they own no child scopes, so
there is nothing for `onCleanup` to register on. Their cleanup contract is the
return value — returning a function schedules it for the next run or disposal.
Calling `onCleanup` inside them throws in dev.

## Examples

Examples of **incorrect** code for this rule:

```tsx
onSettled(() => {
  const id = setInterval(tick, 1000);
  onCleanup(() => clearInterval(id)); // Throws: no owner to register on.
});
```

Examples of **correct** code for this rule:

```tsx
onSettled(() => {
  const id = setInterval(tick, 1000);
  return () => clearInterval(id); // The return value is the cleanup.
});
```

## How to fix

Return the cleanup function from the callback: do the setup, then
`return () => teardown()`. `onCleanup` remains the right tool inside computations
and component bodies — just not inside leaf owners.

## Related

- [invalid-cleanup-return](invalid-cleanup-return.md) — what the return value may be
- [primitive-in-leaf-owner](primitive-in-leaf-owner.md) — the same constraint for primitives
- [no-owner-cleanup](no-owner-cleanup.md) — `onCleanup` with no owner at all
