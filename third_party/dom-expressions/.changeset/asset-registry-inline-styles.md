---
"@dom-expressions/runtime": patch
---

Add a client-side asset registry and an `inline-style` server asset type, closing the CSS lifecycle loop for SSR'd applications.

**`acquireAsset(descriptor)` (client)** — ref-counted ownership of shared document assets. Consumers (routers, lazy wrappers, metadata components) acquire an asset when content that needs it mounts and call the returned release function on cleanup:

```js
const release = acquireAsset({ type: "style", href: "/assets/route.css" });
// … on unmount:
release();
```

- First acquire creates the element in `<head>` — or adopts an SSR/stream-emitted one (links matched by `href`, inline styles by their `data-asset` id) instead of duplicating it.
- Last release removes the element after a short grace period, so release/re-acquire cycles during route transitions keep the live stylesheet instead of flashing unstyled content.
- Supported descriptors: `{ type: "style", href, attrs? }`, `{ type: "inline-style", id, content?, attrs? }`, `{ type: "module", href }`.
- Additionally, `{ policy: "exclusive", key, value, get, set }` provides singleton-slot semantics (last-writer-wins with restore-on-release) as the substrate for future `<Title>`/`<Meta>`-style metadata components.

**`registerAsset("inline-style", { id, content, attrs? })` (server)** — registers CSS by content rather than URL, for styles that have no `.css` file to link (dev-mode CSS collected from the bundler's module graph, critical CSS). Entries dedupe by `id` and emit as `<style data-asset="…">` tags: in `<head>` for anything registered before the shell flushes, inline in the stream for late boundary styles. Extra `attrs` pass through to the tag (e.g. `data-vite-dev-id` so Vite's HMR client adopts the server-rendered style in dev). Unlike stylesheet links, inline styles never gate streamed fragment reveal — they are applied as soon as they are parsed.
