# settled-cleanup-unowned

`SC3005` · **error** · violation

An `onSettled` callback returns a cleanup function, but `onSettled` runs in a scope
with no owner to register the cleanup on.

## What it does

Flags `onSettled` calls whose callback returns a cleanup function while the call
executes in an unowned scope — module scope, a bare helper, or a scope (such as an
event handler) where returned cleanup is not honored.

## Why is this bad?

Returned cleanup only works when `onSettled` can hand the function to an owner that
will eventually dispose. With no owner, the cleanup function is silently dropped:
the setup runs, the teardown never does, and the resource (interval, listener,
socket) leaks without any runtime error to point at it.

## Examples

Examples of **incorrect** code for this rule:

```tsx
// Module scope: no owner, so the returned cleanup is dropped.
onSettled(() => {
  const id = setInterval(poll, 5000);
  return () => clearInterval(id);
});

button.onclick = () => {
  onSettled(() => {
    // Valid place to defer work — but returned cleanup is not supported here.
    return () => subscription.unsubscribe();
  });
};
```

Examples of **correct** code for this rule:

```tsx
// In a component body, the component owns the cleanup.
function Poller() {
  onSettled(() => {
    const id = setInterval(poll, 5000);
    return () => clearInterval(id);
  });
  return <Status />;
}

// Deliberate module-scope reactivity keeps an explicit root.
createRoot(() => {
  onSettled(() => {
    const id = setInterval(poll, 5000);
    return () => clearInterval(id);
  });
});
```

## How to fix

Call `onSettled` where an owner is active — a component body or computation — or
wrap the scope in `createRoot`. Inside event handlers, `onSettled` is valid for
deferring work until the transition settles, but a returned cleanup is not
supported there: perform the teardown explicitly instead.

## Related

- [no-owner-cleanup](no-owner-cleanup.md) — `onCleanup` with the same problem
- [invalid-cleanup-return](invalid-cleanup-return.md) — what the return value may be
