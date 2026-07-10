---
"@dom-expressions/babel-plugin-jsx": patch
---

Fix two option-handling bugs surfaced by the compiler parity sweep:

- `memoWrapper: false` no longer crashes when a conditional or logical expression is transformed (`transformCondition` registered an import under the falsy wrapper name). Conditions now compile memo-less: hoisted conditions keep a plain `var _c$ = () => !!cond` thunk and inline conditions collapse to an immediately-invoked thunk.
- `inlineStyles: false` no longer silently drops a literal `style` on a child element whose position otherwise allocates no element reference (e.g. `<svg><rect style="fill:red"/><g/></svg>` lost the style entirely). `detectExpressions` now accounts for the style-to-IIFE rewrite, so the element gets a reference and the style compiles to the expected effect.

The `effectWrapper`/`memoWrapper` config types now admit `false` alongside the import-name string.
