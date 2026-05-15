---
"babel-plugin-jsx-dom-expressions": patch
"dom-expressions": patch
---

Stop giving special compiler handling to `style:foo` and `class:foo` JSX namespace syntax, and rename the static compiler marker from `@once` to `@static`. `style:foo` and `class:foo` now fall through to literal HTML attributes (e.g. `<div style:border="1px solid black">` emits `style:border` verbatim).

Internal optimizations still split `style={{...}}` into `setStyleProperty` calls and `class={{...}}` into `classList.toggle` calls.
