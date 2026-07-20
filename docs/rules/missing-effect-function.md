# missing-effect-function

`SC7001` · **error** · violation

`createEffect` is called without an effect (apply) function — the removed Solid 1.x
single-callback form.

## What it does

Flags `createEffect` calls with fewer than two arguments, or with `undefined` as
the second argument.

## Why is this bad?

Solid 2.0 split effects into two phases: `createEffect(compute, apply)`. The
compute function runs in the tracking phase and returns a value; the apply function
runs after flush, receives that value, performs the side effect, and may return
cleanup. The 1.x single-callback form no longer exists — with only one function,
there is no apply phase to run the side effect in, and mixing tracking with side
effects is exactly what the split removed.

## Examples

Examples of **incorrect** code for this rule:

```tsx
// Solid 1.x form — no apply function.
createEffect(() => {
  console.log(name());
});
```

Examples of **correct** code for this rule:

```tsx
createEffect(
  () => name(), // compute: tracks dependencies, returns a value
  (value) => {
    // apply: side effect, runs untracked after flush
    const id = setInterval(() => console.log(value), 1000);
    return () => clearInterval(id); // optional cleanup
  },
);

// With error handling, the second argument is an object:
createEffect(() => data(), {
  effect: (value) => render(value),
  error: (err, cleanup) => reportError(err),
});
```

## How to fix

Split the callback: reactive reads go in the compute function, the side effect in
the apply function, and cleanup is returned from apply. Two adjacent changes from
1.x to keep in mind: the `initialValue` second argument is gone (use a default
parameter, `(prev = 0) => ...`), and the apply phase runs untracked, so extract
everything it needs in compute.

## Related

- [strict-read-untracked](strict-read-untracked.md) — reads in the apply phase
- [invalid-cleanup-return](invalid-cleanup-return.md) — what apply may return
