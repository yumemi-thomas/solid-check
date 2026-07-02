---
"@dom-expressions/runtime": patch
---

Schedule `insert()` function-child DOM writes when the parent insert effect is updating an existing slot. This lets async reads inside nested render effects hold the active transition before replacing already-mounted content, and mirrors the fix in the universal renderer.
