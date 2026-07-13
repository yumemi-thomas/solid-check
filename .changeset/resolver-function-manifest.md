---
"@dom-expressions/runtime": patch
---

The `manifest` option of `renderToString`/`renderToStream` now also accepts a
resolver — `{ resolve(key), resolveSync?(key) }` — as an alternative to a
static manifest object, letting dev servers answer asset lookups from their
live module graph while production keeps passing the built manifest object.
`resolve` may return a promise and may resolve CSS entries to inline-style
descriptors (`{ id, content, attrs }`) for HMR adoption; `resolveSync`
answers with what is knowable without async work (typically js URLs) for
sync consumers like a lazy component's `moduleUrl` getter used by islands,
and is exposed on the render context as `resolveAssetsSync` (object
manifests, being sync by nature, expose it too). A bare function is accepted
as shorthand for `{ resolve }`. The consumer contract stays
`renderToStream(fn, { manifest })` in both modes. Entry-asset
auto-registration only applies to object manifests, since a resolver cannot
be enumerated for entries.
