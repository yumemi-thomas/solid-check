---
"@dom-expressions/runtime": patch
---

Server rendering a plain (non-template) object child now dev-warns and skips it, matching the client, instead of crashing with `Cannot read properties of undefined (reading 'fn')` (solidjs/solid#2801 bug 6)
