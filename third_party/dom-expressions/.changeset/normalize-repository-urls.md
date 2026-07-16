---
"@dom-expressions/babel-plugin-jsx": patch
"@dom-expressions/runtime": patch
"@dom-expressions/hyperscript": patch
"@dom-expressions/tagged-jsx": patch
---

Normalize the `repository` field in every package to the standard npm
convention: a `git+https://github.com/ryansolid/dom-expressions.git` URL
with a `directory` pointing at the package within the monorepo. Restores
"View source" / "Open in repo" links on the npm registry and unblocks
tooling that resolves source from package metadata.
