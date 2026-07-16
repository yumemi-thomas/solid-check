# Milestone 5 acceptance

Milestone 5 is complete against the deliverables and acceptance criterion in
the Solid 2 Reactivity Checker Plan.

## Deliverables

- Contract schema: `schema/solid-reactivity.schema.json`.
- Strict decoder, validator, artifact hashing, emitter, loader, automatic
  discovery, and CLI validation: `pkg/contracts`, `internal/packagecontracts`,
  and `cmd/solid-check`.
- Reviewed bundled contracts for `solid-js` and `@solidjs/web`.
- Controlled Solid Primitives `next` checkout at `../solid-primitives-next`.
- Generated, published contracts for all 98 Solid Primitives packages.
- Fixed-point generation for dependencies between contracted packages.

## Acceptance evidence

The source-free consumer fixture
`internal/reactiveir/testdata/solid-primitives-devices-consumer` uses only a
declaration for `@solid-primitives/devices`. With the generated contract and the
real DOM Expressions compiler sidecar, `devices()` is accepted inside tracked
JSX and reported exactly once in the untracked component body.

The conformance suite additionally covers accessor and store factories, nested
store paths, inline/tracked/deferred callbacks, generics and overloads,
re-exports, aliases, subpaths, async metadata, unknown-export fail-closed
behavior, stale artifact rejection, and emitted-contract round trips.

The Solid Primitives corpus has three independent integrity checks:

1. All 98 package builds complete.
2. Every generated contract validates against its exact declaration and
   implementation hashes and is included by the package manifest.
3. Every runtime export from all 98 built `dist/index.d.ts` entrypoints is
   represented by its contract.
