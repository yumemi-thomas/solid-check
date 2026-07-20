# invalid-cleanup-return

`SC3004` · **error** · violation

An effect, tracked-effect, or `onSettled` callback returns a value that is not a
cleanup function.

## What it does

Flags callbacks whose return value is statically known to be something other than a
function or `undefined` — including the implicit Promise returned by an `async`
callback.

## Why is this bad?

Solid treats the return value of these callbacks as cleanup: a function is
scheduled to run before the next execution or on disposal. Any other value is a
contract violation and throws in dev. The `async` case is the sneaky one — an
`async` callback always returns a Promise, so it can never return valid cleanup,
and the intended teardown silently never registers.

## Examples

Examples of **incorrect** code for this rule:

```tsx
createEffect(
  () => count(),
  (value) => {
    return value; // A number is not a cleanup function.
  },
);

onSettled(async () => {
  await connect(); // Implicitly returns a Promise — never valid cleanup.
  return () => disconnect(); // Wrapped in the Promise; never registered.
});
```

Examples of **correct** code for this rule:

```tsx
createEffect(
  () => count(),
  (value) => {
    const id = setInterval(() => console.log(value), 1000);
    return () => clearInterval(id);
  },
);

onSettled(() => {
  // Synchronous callback: start async work inside it, register cleanup up front.
  const controller = new AbortController();
  connect(controller.signal);
  return () => controller.abort();
});
```

## How to fix

Return a cleanup function or nothing at all. If the callback is `async`, make it
synchronous: start the async work inside the callback and register the teardown
(an `AbortController`, an unsubscribe handle) before the first asynchronous step.

## Related

- [cleanup-return-unresolved](cleanup-return-unresolved.md) — when the return value cannot be analyzed
- [cleanup-in-forbidden-scope](cleanup-in-forbidden-scope.md) — where cleanup must be returned instead of registered
