import { createMemo, createSignal } from "solid-js";

const [count] = createSignal(0); // count: a branded Solid Accessor (reactive)
const total = (): number => 42; // an ordinary function — identical call syntax

// Only possible with real types: `count()` and `total()` are syntactically
// identical, so an AST linter must either flag both (false positive) or neither
// (false negative). solid-checker knows from typescript-go that `count` is a
// branded Solid Accessor and `total` is a plain function, so it flags ONLY the
// reactive read after await (SC1002) and leaves the plain call alone.
export const report = createMemo(async () => {
  await fetch("/api/warmup");
  const plain = total(); // fine — not reactive
  const live = count(); // BUG: reactive read after await (SC1002)
  return plain + live;
});
