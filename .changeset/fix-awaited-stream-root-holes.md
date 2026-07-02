---
"dom-expressions": patch
---

Awaited renderToStream (`then()`, which renderToStringAsync wraps) now waits out blocking promises and re-pulls pending root holes before completing, matching `pipe()`. Previously a render whose only async was a blocked root hole (e.g. `lazy()` or an async component source with no registered fragment) completed immediately with an unfinished shell.
