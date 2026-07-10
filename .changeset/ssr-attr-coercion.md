---
"@dom-expressions/runtime": patch
---

Align SSR serialization of non-string attribute values (arrays, objects) with client-side `setAttribute` coercion so both environments produce the same final attribute string.
