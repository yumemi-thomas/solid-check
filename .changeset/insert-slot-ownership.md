---
"dom-expressions": patch
---

Harden DOM-runtime insertion against nodes that have migrated out of their
original slot between renders. Resolves the class of bugs reported as
solidjs/solid#2030 (a new JSX value that wraps the previous slot's node) and
solidjs/solid#2357 (a single node referenced from multiple sibling slots).
Previously, `cleanChildren` and `reconcileArrays` could either throw
`replaceChild` "new child contains the parent", or silently destroy the
migrated node by trusting a stale `current.parentNode === parent` check.

Every runtime insertion site (`appendNodes`, `insertExpression`'s
element-node branch, the replacement path in `cleanChildren`, all four
insertion sites in `reconcileArrays`) now tags the inserted node with a
per-slot `$$SLOT` Symbol property carrying the slot's marker. Every
destructive operation (`remove`, `replaceChild`, `insertBefore` against a
sibling anchor) is now gated on parent-and-tag ownership: an untagged node
is treated as unclaimed (the slot may manage it), a tagged node is touched
only when its tag matches the current slot's marker. Foreign nodes — refs
appended by user code, nodes that have migrated to another slot, content
inserted by other runtimes — are left alone.

The `tail.nextSibling` `after` anchor in `reconcileArrays` is also gated:
if `a`'s tail has migrated, the `after` falls back to the slot's marker
rather than reading a sibling pointer that now points into another slot's
region. The symmetric end-swap fast-path (`a[0]===b[n-1] && b[0]===a[n-1]`)
gains an anchor-ownership check so it cannot stage moves against a foreign
front anchor; mismatched anchors fall through to the map branch which
re-gates each destructive op.

Scope: DOM renderer (`client.js`) only. `universal.js` is intentionally
unchanged — universal hosts target older JS environments (Chrome 38+),
expando writes on platform nodes can collide with proxy-based node wrappers,
and the JSX-DOM-ref migration patterns this fix addresses are not idiomatic
on non-DOM platforms. If a real case surfaces on a universal renderer it
can be revisited with a host-appropriate storage strategy.
