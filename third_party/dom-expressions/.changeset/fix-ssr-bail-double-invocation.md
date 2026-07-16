---
"@dom-expressions/runtime": patch
---

Fix `ssr()` double-invocation on bail paths.

A function hole whose return value walked into the bail branch (e.g., an array containing a NotReady-throwing item) was being invoked twice: once by `tryResolveString` for sync probing and again by the fallback `resolveSSRNode(hole, result)`. For closures that read stateful getters such as JSX `props.children` — whose backing component rebuilds an owner subtree on each access — the duplicate invocation produced a second owner tree with a divergent hydration-key prefix that the client could not claim, surfacing as "Hydration completed with N unclaimed server-rendered node(s)" warnings.

`tryResolveString` now evaluates each function node exactly once and threads the evaluated value through the bail object so `ssr()` can hand it to `resolveSSRNode` without re-invoking the original closure.
