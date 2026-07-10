---
"@dom-expressions/jsx-compiler": patch
---

Close the remaining configuration gaps between the AST-native compiler and the babel plugin:

- `effectWrapper` and `memoWrapper` now accept custom import names (babel's string form): `effectWrapper: "createRenderEffect"` imports and calls `createRenderEffect` instead of `effect`. `false` (or `""`) still disables the wrapper. The options no longer need to be disabled as a pair — `wrapConditionals: false`, `effectWrapper: false`, and `memoWrapper: false` each work independently, matching babel (except `memoWrapper: false` with conditionals present, which crashes babel and compiles memo-less here).
- `requireImportSource` is implemented: when set, only files carrying a `@jsxImportSource <source>` comment are transformed (same comment-splitting match as babel — the comment's remainder after `@jsxImportSource` must equal the configured source exactly); other files return their source text untouched.
- `validate` is implemented (default `true`, like babel): DOM template markup is re-parsed with a spec HTML parser (html5ever, the Rust counterpart of the babel plugin's parse5) and a warning is printed to stderr when a browser's `innerHTML` would restructure the markup (implied end tags, foster parenting, nested `<a>`/`<form>`/`<button>`, misplaced hydration markers). Warning text, text-node normalization, table-partial wrapping, and the skip list match the babel plugin's `isInvalidMarkup`.
- `inlineStyles: false` parity in spreads: a `style` on a native element with spread attributes now wraps in the same IIFE getter babel produces (previously it could land as a plain static prop), and a `/*@static*/` marker on a style stops applying under `inlineStyles: false` because the rewrap discards the original node (babel behavior).

An option-matrix parity suite now compiles the whole fixture corpus under each flag flipped from its default (140 mode × variant combinations) and requires identical normalized output from both compilers. One known babel bug is documented and not replicated: with `inlineStyles: false`, babel silently drops a static `style` on a non-root element whose position allocates no element id (`<svg><rect style="fill:red"/><g/></svg>` loses the style); the AST-native compiler emits the style write.
