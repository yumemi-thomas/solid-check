---
"@dom-expressions/babel-plugin-jsx": patch
"@dom-expressions/runtime": patch
---

Fix hydration key mismatches when async holes defer past eager siblings
(solidjs/solid#2801 bug 2). Dynamic element children that can allocate
hydration ids (conditionals, component-children access, call expressions)
are now compiled with their own id scope on both generates: the dom and ssr
generates wrap the hole expression in a new `scope()` runtime helper using a
shared predicate, so marking cannot desync.

On the client, `scope(fn)` tags the accessor and `insert()` makes the outer
render effect non-transparent (its own id scope) for tagged accessors; the
inner unwrapping effect stays transparent so content ids keep a fixed depth.
On the server, `scope` (framework-provided via rxcore as `ssrScope`) reserves
one id slot at registration and evaluates the hole — including async retries
— under that reserved id with a zeroed child counter, so retry timing can no
longer shift sibling ids. The ssr generate's `orderedInsert` sibling
thunk-wrapping is removed; it is superseded by hole scopes.

Hole content ids gain one nesting level (e.g. `_hk=10` instead of `_hk=1`)
identically on both sides. rxcore implementations must provide an `ssrScope`
export and honor a `scope: true` effect option (mapped to a non-transparent
render effect).
