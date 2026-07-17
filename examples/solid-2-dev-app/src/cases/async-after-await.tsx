import { Loading, createMemo, createSignal } from "solid-js";

// Bad: the dependency is discovered only after suspension.
export function BadReactiveReadAfterAwait() {
  const [count] = createSignal(0);
  const result = createMemo(async () => { await Promise.resolve(); return count() * 2 });
  return <Loading>{result()}</Loading>;
}
export function BadSecondAwaitThenRead() {
  const [count] = createSignal(0);
  const result = createMemo(async () => {
    await Promise.resolve();
    await Promise.resolve();
    return count();
  });
  return <Loading>{result()}</Loading>;
}

// Good: capture reactive dependencies before awaiting.
export function GoodReadBeforeAwait() {
  const [count] = createSignal(0);
  const result = createMemo(async () => { const captured = count(); await Promise.resolve(); return captured * 2 });
  return <Loading>{result()}</Loading>;
}
export function GoodMultipleReadsBeforeAwait() {
  const [count] = createSignal(0);
  const result = createMemo(async () => {
    const first = count();
    const second = count();
    await Promise.resolve();
    return first + second;
  });
  return <Loading>{result()}</Loading>;
}
