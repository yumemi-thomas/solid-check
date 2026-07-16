---
"@dom-expressions/runtime": patch
---

Handle module preload failures during hydration instead of hanging silently (solidjs/solid#2817 layer 3). `loadModuleAssets` drops rejected entries from the loading cache so later boundaries/navigations can retry, and the root `_assets` path in `hydrate()` falls back to a fresh client render (with a console diagnostic) instead of leaving the page permanently dead.
