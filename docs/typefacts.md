# Type Facts module

`internal/typefacts` is the seam between the certification engine and native
TypeScript analysis. Its interface exposes only opaque identities and original
source facts. TypeScript AST nodes, checker types, symbols, programs, and shim
terminology stay inside the `tsgo` adapter.

## Current behavior

- Open a real `tsconfig.json` project.
- Resolve the symbol at a UTF-8 byte location.
- Resolve import aliases to their original declarations.
- Return declaration locations and non-declaration references across files.
- Build one lazy canonical-symbol reference index per project generation, so
  multiple reference queries traverse the TypeScript AST only once. Successful
  updates discard the index together with generation-scoped symbol identities.
- Return an opaque canonical type identity at a source location.
- Optionally describe a named type's text and source alias declarations without
  exposing native checker objects. Reactive IR uses this to distinguish a
  `solid-js` `Accessor` from a structurally identical plain function.
- Resolve call targets and instantiated return types.
- Optionally enumerate parsed source calls in source order with exact whole-call,
  callee, and argument byte ranges plus an opaque alias-resolved target.
  Reactive IR uses this bulk capability instead of reconstructing call
  structure with regexes and reuses target identities for generation-scoped
  type-description caching; backend AST nodes remain private to the adapter.
- Optionally enumerate call-initialized variable bindings, preserving direct
  versus array destructuring, exact bound-name ranges, and omitted tuple slots.
  Reactive IR shares these facts across reactive-value and setter discovery.
- Optionally enumerate named block-bodied function declarations and direct
  identifier-bound arrows with exact name, body, and parameter ranges plus
  export, async, and declaration-kind flags.
- Optionally enumerate function expressions and declarations with checker-backed
  async return capability and AST-derived calls dominated by `await` on every
  reachable path. Branches, loops, `try`/`catch`/`finally`, concise expressions,
  imported callbacks, and local alias chains are handled conservatively; nested
  function calls are excluded. This keeps async classification and SC1002 flow
  facts inside the tsgo adapter.
- Apply monotonically versioned in-memory file overlays, incrementally update a
  single existing source file when safe, and report the changed files plus
  their transitive importers without writing editor contents to disk.
- Discover overlay-added and overlay-deleted files through the same `tsconfig`
  include rules and module-resolution view used by clean projects.
- Reject invalid ranges and ranges that split a UTF-8 code point.
- Apply updates transactionally: cancellation or a rebuild failure leaves the
  prior project generation queryable and unchanged.

Opaque identities include their project generation and are valid only for that
analysis version. Callers must reacquire identities after every successful
`Update`; stale identities cannot collide with identities in the new program.

## Native integration pins

- tsgolint: `c3269c01a0c894a31330e1b4c3bd4edc6eb7694b`
- typescript-go: `2bd066d87f5b`

All unstable shim modules are replaced from the same tsgolint pseudo-version in
`go.mod`. No `go:linkname` or native compiler type is allowed outside
`internal/typefacts/tsgo` and its pinned dependencies.

`AffectedSet` is derived from the union of the old and updated resolved-module
graphs. This keeps removed or redirected imports conservative while excluding
unrelated project files. A single accepted, non-deleted source edit uses
TypeScript-Go's `Program.UpdateProgram`; its structural checks fall back to a
new program for import-graph changes. Added or deleted files, `tsconfig` edits,
and multi-file batches explicitly rebuild the complete program. Both paths
remain behind the same transactional interface.

The retained shim surface and evidence-based removal sequence are documented in
[tsgolint-extraction.md](tsgolint-extraction.md).

## Hardening evidence

The integration suite exercises chained, namespace, default, and aliased
re-exports; package export subpaths through package-manager symlinks; project
references; JavaScript with JSDoc generics; overload and generic substitution;
CRLF and multibyte source; clean/overlay equivalence; failed and canceled
updates; overlay file creation/deletion; named-type alias origin; semantic
async returns; await dominance; nested callback exclusion; and conservative
affected-file sets. Reference-index coverage verifies imported
aliases, deterministic source ordering, and invalidation after an overlay
update.
Separate audits enforce the reviewed shim import allowlist and the single
tsgolint revision pin.
