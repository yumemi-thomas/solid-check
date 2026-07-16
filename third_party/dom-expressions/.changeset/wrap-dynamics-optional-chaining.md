---
"@dom-expressions/babel-plugin-jsx": patch
---

Replace the `wrapDynamics` previous-value default-object initializer with optional chaining for both DOM and universal generators. The combined-effect arrow now takes `(_v$, _p$) => …` and reads `_p$?.<n>` instead of receiving an `_p$ = { 0: undefined, 1: undefined, … }` defaulted object literal. Removes a per-render-effect setup allocation, shrinks compiled output, and matches the shape used elsewhere in the runtime. The DOM generator special-cases `textContent` (`!_p$ || a !== _p$.a`) to keep the first-run write semantics.
