// This module is intentionally invalid so `solid-checker` can demonstrate the
// async reactivity diagnostic (SC1002, `reactive-read-after-await`). It is not
// imported by the running app.
import { createMemo, createSignal } from "solid-js";

const [count, _setCount] = createSignal(0);
export const double = createMemo(async () => {
  await fetch("/api/session");
  await new Promise(r => setTimeout(r, 2000));
  return count() * 2;
});
