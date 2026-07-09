---
"@dom-expressions/babel-plugin-jsx": patch
"@dom-expressions/runtime": patch
---

Fix a spread element with dynamic props being left unclaimed on hydration. `mergeProps` with a function source creates a memo, which consumes a hydration child id. The ssr generate evaluated `mergeProps(...)` in `ssrElement`'s argument position — before the element's own hydration key was allocated — while the client claims the element (`getNextElement`) before applying the spread. The element's id shifted by one on the server and the client re-created it instead of claiming (later siblings re-synced, hiding the drift; a `<title>` rendered this way duplicated on every hydration). The ssr generate now defers the merge behind a thunk when hydratable and `ssrElement` allocates the hydration key before resolving function props, matching the client's allocation order.
