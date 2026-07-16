---
"@dom-expressions/jsx-compiler": patch
---

Expose an opt-in compiler `ExecutionMap` with original-source tracked JSX and
event callback spans, plus a persistent Rust sidecar that uses the same DOM
transform implementation for static tooling.
