# flush-in-forbidden-scope

`SC3003` · **error** · violation

`flush()` is called inside `onSettled` or `createTrackedEffect`, which run as part
of the flush cycle itself.

## What it does

Flags `flush()` calls that are lexically contained in an `onSettled` or
`createTrackedEffect` callback.

## Why is this bad?

Solid 2.0 batches all writes on microtasks; `flush()` drains that queue
synchronously. `onSettled` and `createTrackedEffect` execute *during* the flush
cycle, so calling `flush()` from inside them would re-enter the scheduler. Solid
throws in dev instead of risking re-entrant flushes.

## Examples

Examples of **incorrect** code for this rule:

```tsx
onSettled(() => {
  setReady(true);
  flush(); // Throws: already inside the flush cycle.
  measure(element);
});
```

Examples of **correct** code for this rule:

```tsx
// Inside onSettled the graph has already settled — just read.
onSettled(() => {
  measure(element);
});

// If you need to observe a write synchronously, do it at the imperative boundary:
button.onclick = () => {
  setReady(true);
  flush();
  measure(element);
};
```

## How to fix

Inside these scopes the graph has already settled: signal values and the DOM are
current, so the `flush()` is usually unnecessary — delete it. If you need to
observe a write you just made, move both the write and the `flush()` out to the
event handler or imperative boundary that triggered the scope.

## Related

- [cleanup-in-forbidden-scope](cleanup-in-forbidden-scope.md) — other leaf-owner restrictions
