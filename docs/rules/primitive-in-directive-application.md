# primitive-in-directive-application

`SC6001` · **error** · violation

A reactive primitive is created inside a directive's apply callback — the function
a directive factory returns for each element.

## What it does

Flags creations of reactive primitives (`createSignal`, `createMemo`,
`createEffect`, …) inside callbacks that the compiler recognizes as `ref`/directive
applications, including callbacks returned from a directive factory.

## Why is this bad?

Directives in Solid 2.0 compose through `ref` with a two-phase factory pattern: the
factory body (setup) runs in an owned scope, while the returned callback (apply)
runs per element as an unowned leaf. Primitives created in the apply phase have no
owner — they are never tracked into the graph and never disposed, leaking once per
element the directive is applied to.

## Examples

Examples of **incorrect** code for this rule:

```tsx
const tooltip = (options) => (el) => {
  // Apply phase: unowned, runs per element.
  const [visible, setVisible] = createSignal(false); // Never tracked or disposed.
  el.addEventListener("mouseenter", () => setVisible(true));
};
```

Examples of **correct** code for this rule:

```tsx
const tooltip = (options) => {
  // Setup phase: owned scope — primitives and subscriptions live here.
  const [visible, setVisible] = createSignal(false);
  createEffect(
    () => visible(),
    (on) => (on ? show(options) : hide()),
  );
  // Apply phase: DOM work only.
  return (el) => {
    el.addEventListener("mouseenter", () => setVisible(true));
    el.addEventListener("mouseleave", () => setVisible(false));
  };
};

<button ref={tooltip({ content: "Save" })}>Save</button>;
```

## How to fix

Use the two-phase factory: create primitives, computations, and subscriptions in
the factory body, and keep the returned callback to DOM reads, writes, and listener
wiring only.

## Related

- [primitive-in-leaf-owner](primitive-in-leaf-owner.md) — the leaf-owner analogue
