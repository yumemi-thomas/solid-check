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
- Return an opaque canonical type identity at a source location.
- Resolve call targets and instantiated return types.
- Apply monotonically versioned in-memory file overlays, rebuild project facts,
  and report the changed files plus their transitive importers without writing
  editor contents to disk.

Opaque identities are valid for the current project analysis version. Callers
must reacquire identities after `Update`; stable cross-version identities are a
later incremental-backend obligation.

## Native integration pins

- tsgolint: `c3269c01a0c894a31330e1b4c3bd4edc6eb7694b`
- typescript-go: `2bd066d87f5b`

All unstable shim modules are replaced from the same tsgolint pseudo-version in
`go.mod`. No `go:linkname` or native compiler type is allowed outside
`internal/typefacts/tsgo` and its pinned dependencies.

`AffectedSet` is derived from the union of the old and rebuilt resolved-module
graphs. This keeps removed or redirected imports conservative while excluding
unrelated project files. The compiler program is still rebuilt as a whole; a
later cycle can reuse typescript-go's incremental project state without changing
the interface.

The retained shim surface and evidence-based removal sequence are documented in
[tsgolint-extraction.md](tsgolint-extraction.md).
