import { Loading, createMemo, createSignal, refresh } from "solid-js";

// Bad: refresh requires the original branded accessor, not a read or wrapper.
export function BadRefreshWrapper() {
  const value = createMemo(async () => "Stale");
  const wrapped = () => value();
  // @ts-expect-error Intentionally demonstrates an invalid refresh target.
  return <Loading><button onClick={() => refresh(wrapped)}>{value()}</button></Loading>;
}
export function BadRefreshRead() {
  const value = createMemo(async () => "Stale");
  // @ts-expect-error Intentionally demonstrates passing a read value.
  return <Loading><button onClick={() => refresh(value())}>{value()}</button></Loading>;
}

// Good: pass the computation or signal itself.
export function GoodRefreshMemo() {
  const value = createMemo(async () => "Fresh");
  return <Loading><button onClick={() => refresh(value)}>{value()}</button></Loading>;
}
export function GoodRefreshSignal() {
  const [value] = createSignal("Fresh");
  return <button onClick={() => refresh(value)}>{value()}</button>;
}
