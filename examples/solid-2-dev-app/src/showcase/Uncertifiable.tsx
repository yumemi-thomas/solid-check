import { createEffect } from "solid-js";

// This helper is exported, so it can be called from anywhere — including from
// outside any reactive owner. solid-checker cannot prove, within this project,
// that every caller runs it under a live owner that will dispose the effect.
//
// The result is neither "clean" nor "broken": it is UNCERTIFIABLE (SC4001).
// solid-checker is fail-closed — rather than guessing, it refuses to certify, so
// `--certify` exits non-zero here. eslint has no equivalent "can't prove it"
// verdict; it would simply stay silent.
export function installWatcher() {
  createEffect(
    () => 1,
    () => {},
  );
}
