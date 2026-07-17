// Project-local re-exports of Solid primitives under custom names. solid-check
// follows these aliases across files, so it still knows `derive` is really
// `createMemo` and `state` is really `createSignal`. A per-file ESLint rule only
// sees imports of unknown local functions and gives up.
export { createMemo as derive, createSignal as state } from "solid-js";
