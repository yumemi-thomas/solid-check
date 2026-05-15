---
"babel-plugin-jsx-dom-expressions": patch
"dom-expressions": patch
---

Remove the `style:foo` and `class:foo` JSX namespaces and rename the static marker from `@once` to `@static`. `style:foo` and `class:foo` no longer get special handling — they fall through to literal HTML attributes (e.g. `<div style:border="1px solid black">` emits `style:border` verbatim). Use `@static` to mark expressions the compiler should not wrap in effects.

Internal optimizations still split `style={{...}}` into `setStyleProperty` calls and `class={{...}}` into `classList.toggle` calls.
