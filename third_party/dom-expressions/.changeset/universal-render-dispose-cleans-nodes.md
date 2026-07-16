---
"@dom-expressions/runtime": patch
---

Universal renderer `render()` disposers now remove the top-level host nodes they mounted, matching DOM `render()` cleanup semantics. Custom renderers can provide `cleanupNodes(parent, nodes)` to override the default per-node `removeNode` teardown.
