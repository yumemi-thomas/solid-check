---
"dom-expressions": patch
---

Add `host` option to `insert` for portal-style slots. Top-level nodes managed by the slot are tagged with a live `_$host` getter after each update, replacing proxy-based DOM call interception. The mount parent is now a real element so slot-ownership checks (`parentNode` identity) behave correctly — fixes portal content accumulating on swaps (solidjs/solid#2757) — and tagging covers the `replaceChild`, reconcile, and hydration claim paths the proxy missed.
