---
"dom-expressions": patch
"lit-dom-expressions": patch
---

Make `ClassValue` recursive so nested arrays type-check. The runtime already
flattens arbitrarily nested class arrays (e.g. `class={["a", ["b"]]}`), but the
type only allowed a single level. `ClassValue` is now `string | number |
boolean | null | undefined | Record<string, boolean> | ClassValue[]`.
