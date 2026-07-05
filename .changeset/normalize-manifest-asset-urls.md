---
"@dom-expressions/runtime": patch
---

Normalize manifest asset URL joining in `resolveAssets` (solidjs/solid#2817 layers 1-2). A non-string `_base` (e.g. a dev-manifest proxy answering every key) falls back to `/`, leading-slash `file` values no longer produce `//` URLs, and absolute/protocol-relative URLs pass through untouched — the server runtime emits sane module URLs for any reasonable manifest shape instead of relying on bundler plugins getting the contract exactly right.
