import { createMemo } from "solid-js";

// `{ sync: true }` declares a synchronous computation, but the callback is
// async and returns a Promise. solid-checker resolves the option and proves the
// callback's return type contradicts it (SC7002). This needs the callback's
// *type*, not just its syntax, so it's out of reach for an AST-only linter.
export const value = createMemo(async () => 1, { sync: true });
