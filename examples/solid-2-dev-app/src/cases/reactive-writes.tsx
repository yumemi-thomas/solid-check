import { createEffect, createMemo, createSignal, onSettled, untrack } from "solid-js";

// Bad: writes in component bodies and tracked computations are forbidden.
export function BadComponentWrite() {
  const [count, setCount] = createSignal(0);
  setCount(1);
  return <output>{count()}</output>;
}
export function BadMemoWrite() {
  const [count, setCount] = createSignal(0);
  const doubled = createMemo(() => { setCount(count() + 1); return count() * 2 });
  return <output>{doubled()}</output>;
}
export function BadEffectComputeWrite() {
  const [count, setCount] = createSignal(0);
  createEffect(() => setCount(count() + 1), () => {});
  return <output>{count()}</output>;
}

// Good: writes belong in events, settled callbacks, actions, or untrack.
export function GoodEventWrite() {
  const [count, setCount] = createSignal(0);
  return <button onClick={() => setCount(value => value + 1)}>{count()}</button>;
}
export function GoodSettledWrite() {
  const [ready, setReady] = createSignal(false);
  onSettled(() => { setReady(true) });
  return <output>{String(ready())}</output>;
}
export function GoodUntrackedWrite() {
  const [value, setValue] = createSignal(0);
  return <button onClick={() => untrack(() => setValue(0))}>{value()}</button>;
}
