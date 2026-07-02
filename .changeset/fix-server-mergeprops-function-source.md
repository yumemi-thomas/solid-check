---
"@dom-expressions/runtime": patch
---

Source server `mergeProps` from rxcore like the client and universal entries instead of shipping a local copy. The local merger resolved function sources for key enumeration only — the per-key getter read values off the raw, un-invoked function — so SSR dropped spread props whose source is a function (`<div {...fn()}>`, `<Dynamic {...props}>`; solidjs/solid#2815). Prop-merge semantics belong to the framework core; renderers must now export `mergeProps` from their server rxcore module (the universal entry already required this).
