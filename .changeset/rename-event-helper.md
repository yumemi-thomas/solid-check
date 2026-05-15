---
"babel-plugin-jsx-dom-expressions": patch
"dom-expressions": patch
"lit-dom-expressions": patch
---

Rename the generated event listener helper from `addEventListener` to `addEvent` so compiled browser bundles no longer introduce a binding that can shadow the native `window.addEventListener` method.
