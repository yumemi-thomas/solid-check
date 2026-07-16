---
"@dom-expressions/babel-plugin-jsx": patch
"@dom-expressions/runtime": patch
---

Rename the generated event listener helper from `addEventListener` to `addEvent` so compiled browser bundles no longer introduce a binding that can shadow the native `window.addEventListener` method.
