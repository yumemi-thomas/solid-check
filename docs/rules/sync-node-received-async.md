# sync-node-received-async

`SC7002` · **error** · violation

A computation is marked `sync: true`, but its callback can return a Promise or
AsyncIterable.

## What it does

Flags `createMemo`, `createSignal`, `createStore`, `createEffect`, and related
calls whose options include `sync: true` while the computation's callback is async
or statically returns a Promise/AsyncIterable.

## Why is this bad?

`sync: true` promises the scheduler that the node settles within the same flush —
it can neither defer nor suspend. An async result breaks that promise: the node
cannot wait for the Promise, so the combination throws at runtime the moment the
computation returns one.

## Examples

Examples of **incorrect** code for this rule:

```tsx
const user = createMemo(async () => fetchUser(id()), { sync: true });
```

Examples of **correct** code for this rule:

```tsx
// Let the async value suspend to a Loading boundary:
const user = createMemo(() => fetchUser(id()));

// Or keep the sync node synchronous and read the settled async value from it:
const user = createMemo(() => fetchUser(id()));
const initials = createMemo(() => initialsOf(user().name), { sync: true });
```

## How to fix

Drop `sync: true` and let the graph suspend to a `<Loading>` boundary, or make the
computation synchronous by moving the async work into its own computation and
reading the settled accessor here.

## Related

- [async-outside-loading-boundary](async-outside-loading-boundary.md) — consuming async computations
