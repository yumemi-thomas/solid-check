---
"@dom-expressions/babel-plugin-jsx": patch
---

Fix hydration id drift for spread elements with dynamic children. The ssr generate's spread-element path (`ssrElement`) never applied the hole id `scope()` wrap that `transformChildren` applies on the template path — while the dom generate scope-wraps the matching insert accessor regardless of spread. For a shape like `<a {...props}>{children()}</a>`, the client reserved one hydration id for the hole and the server did not, so every hydration key allocated after the hole drifted and the following siblings were left unclaimed (duplicated DOM, "unclaimed server-rendered node" warnings). `createElement` now wraps dynamic, id-allocating children holes in `scope()` exactly like the template path.
