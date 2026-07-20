# primitive-in-leaf-owner

`SC3002` · **error** · violation

A reactive primitive (`createSignal`, `createMemo`, `createEffect`, …) is created
inside a leaf owner (`onSettled` or `createTrackedEffect`).

## What it does

Flags calls that create reactive primitives when they are lexically contained in an
`onSettled` or `createTrackedEffect` callback.

## Why is this bad?

Leaf owners own no children. A primitive created inside one has no owner to attach
to: it is never tracked into the graph and never disposed, so its subscriptions
leak and its updates go nowhere. Solid throws in dev when this happens.

## Examples

Examples of **incorrect** code for this rule:

```tsx
onSettled(() => {
  const [visible, setVisible] = createSignal(false); // Throws: leaf owners cannot own primitives.
  setVisible(true);
});
```

Examples of **correct** code for this rule:

```tsx
// Create primitives in the component body; use their accessors in the leaf owner.
const [visible, setVisible] = createSignal(false);

onSettled(() => {
  console.log(visible());
  return () => console.log("disposing");
});
```

## How to fix

Create the primitive in the component body (or another owning scope) and read its
accessor inside the leaf owner. If the primitive's lifetime really is tied to the
callback, the logic probably belongs in a computation rather than a leaf owner.

## Related

- [cleanup-in-forbidden-scope](cleanup-in-forbidden-scope.md) — the same constraint for `onCleanup`
- [primitive-in-directive-application](primitive-in-directive-application.md) — the directive analogue
