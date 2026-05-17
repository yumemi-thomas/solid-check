---
"dom-expressions": patch
---

Dispose the partially-created reactive scope when `render()` (and the universal renderer's `render()`) throws during initial mount. Previously a synchronous throw inside the top-level component would orphan the root and, in the DOM client, leave the delegated-root counter bumped — leaking event-delegation state with no recovery path since the caller never receives the disposer. The throw still propagates; the cleanup just happens before it does. `hydrate()` benefits transitively because it delegates to `render()`.
