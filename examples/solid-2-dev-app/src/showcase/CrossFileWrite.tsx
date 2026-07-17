import { derive, state } from "./lib/reactive-helpers";

const [count, setCount] = state(0);

// BUG (cross-file): `derive` is `createMemo` (aliased in another module), so its
// callback runs inside an owned, tracked scope where writing a signal is
// illegal. solid-check resolves the alias across `lib/reactive-helpers.ts` and
// proves the write happens in an owned scope (SC2001). A per-file linter cannot
// follow the setter through the re-export, so it stays silent.
export const doubled = derive(() => {
  setCount(count() + 1);
  return count() * 2;
});
