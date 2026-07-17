import { createEffect, createMemo, createSignal } from "solid-js";

// Bad: these reads happen once in untracked component/effect-apply code.
export function BadComponentRead() {
  const [count] = createSignal(0);
  const frozen = count();
  return <output>{frozen}</output>;
}
export function BadAliasedRead() {
  const [count] = createSignal(0);
  const read = count;
  const frozen = read();
  return <output>{frozen}</output>;
}
export function BadEffectApplyRead() {
  const [count] = createSignal(0);
  createEffect(() => true, () => console.log(count()));
  return <output>{count()}</output>;
}

// Good: JSX and the compute half of an effect are tracked.
export function GoodJsxRead() {
  const [count] = createSignal(0);
  return <output>{count()}</output>;
}
export function GoodMemoRead() {
  const [count] = createSignal(0);
  const doubled = createMemo(() => count() * 2);
  return <output>{doubled()}</output>;
}
export function GoodEffectComputeRead() {
  const [count] = createSignal(0);
  createEffect(() => count(), value => console.log(value));
  return <output>{count()}</output>;
}
