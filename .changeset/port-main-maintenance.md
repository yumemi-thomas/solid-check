---
"@dom-expressions/babel-plugin-jsx": patch
"@dom-expressions/runtime": patch
---

Port relevant maintenance fixes from the stable branch. Add `omitAttributeSpacing` for strict template attribute spacing, and align `server.js`/`server.d.ts` with the current `client.d.ts` export surface so isomorphic imports continue to resolve on the server.
