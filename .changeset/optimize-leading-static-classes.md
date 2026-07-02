---
"@dom-expressions/babel-plugin-jsx": patch
---

Optimize class arrays with leading static class strings and a fixed-shape class object so the static classes are emitted in the template and dynamic object entries compile to class toggles.
