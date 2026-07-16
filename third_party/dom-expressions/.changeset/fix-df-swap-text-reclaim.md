---
"@dom-expressions/runtime": patch
---

fix(client): re-claim a hole's live DOM region when a streamed `$df` fragment swap replaced its tracked nodes mid-hydration (solidjs/solid#2801 bug 1, pending-stream case). A Loading fallback claimed during hydration is swapped out by `$df` before the boundary resumes; insert's node bookkeeping still pointed at the removed fallback, so the content pass fabricated detached text nodes and the first post-hydration refresh appended duplicates. When the tracked nodes are disconnected while hydrating, insert now re-derives the region (parent children, or back to the matching `<!--$-->` for marker-bounded holes) so loose text re-claims positionally — elements already recovered via `_hk`.
