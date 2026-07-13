---
"@dom-expressions/runtime": patch
---

Queue streamed fragment activations whose `pl-*` marker is not yet in the live DOM instead of silently dropping them.

A fragment's marker can sit inside a flushed-but-unactivated ancestor `<template>` (a slot held by a reveal group). Template content is inert, so `document.getElementById` cannot see the marker and `$df`/`$dfl` previously returned `0` and lost the swap permanently — the fallback stayed stuck even though the content template had streamed. Today this window is masked because the server enrolls nested boundaries into the ancestor reveal group, but fixing that enrollment (solidjs/solid#2871, solidjs/solid#2872) makes nested boundaries activate independently, exposing the drop.

`$df` and `$dfl` now queue marker misses (`_$HY.dq` / `_$HY.dlq`) and a new `$dfd` drains both queues after every successful swap or fallback materialization — the only events that can bring queued markers into the live document. Content swaps drain before fallbacks so a settled fragment wins over its own pending fallback, and drains cascade through arbitrarily nested held levels. An activation whose content template is already consumed remains a plain no-op and is never queued.
