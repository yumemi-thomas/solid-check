---
"@dom-expressions/runtime": patch
---

Root-level inserts no longer wipe foreign sibling nodes when clearing or replacing their content. Streaming appends late-flushed `<link rel="stylesheet">` tags to the end of `<body>`, inside the region a document-level hydration root tracks; previously a root expression that emptied (`textContent = ""` fast path) or swapped to text took those links with it, dropping loaded CSS (FOUC). `insert` now removes only the nodes it tracks when the parent contains children it doesn't own, keeping the fast path when it owns everything. Also documents that `registerModule` / `loadModuleAssets` mapping keys are opaque to the runtime — the reactive library chooses them (e.g. hydration ids) on both sides of the wire.
