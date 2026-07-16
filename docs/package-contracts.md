# Package contracts

Milestone 5 introduces `solid-reactivity.json`, a non-executable summary that
preserves exported reactive reads when a dependency's implementation source is
not part of the consuming TypeScript project.

## Workflow

Analyze the package and emit its solved exported function summaries:

```sh
solid-check --project package/tsconfig.json \
  --emit-contract package/solid-reactivity.json \
  --package-name reactive-package \
  --package-version 1.0.0 \
  --declaration-artifact package/index.d.ts \
  --implementation-artifact package/index.js
```

Load the contract while analyzing a consumer:

```sh
solid-check --project app/tsconfig.json \
  --contract package/solid-reactivity.json \
  --format json \
  --certify
```

`--contract` is repeatable. Contracts published as
`node_modules/<package>/solid-reactivity.json` are discovered automatically,
including root contracts used through package subpaths. Explicit contracts
override discovered and bundled contracts. The loader binds named imports to
matching package and export names through Type Facts; it does not inspect the
dependency's implementation source.

Validate contracts and their artifacts without opening a TypeScript project:

```sh
solid-check --validate-contract package/solid-reactivity.json
```

## Trust boundary

The schema is [solid-reactivity.schema.json](../schema/solid-reactivity.schema.json).
The loader fails closed on:

- unsupported schema or compiler-facts protocol versions;
- unknown JSON fields or malformed summaries;
- unsupported effect or evidence kinds;
- imports of exports missing from an otherwise valid contract;
- unsafe artifact paths; and
- declaration or implementation hashes that do not match the files beside the
  contract.

Artifact hashes use `sha256:<lowercase hex>`. The artifact flags hash exact file
bytes and require each file to be inside the emitted contract's directory.
Artifacts remain optional because they are not always available at emission
time, but they are verified whenever present. The contract itself is SHA-256
hashed when loaded, and that identity is included in the certification package
summary.

Evidence is explicit: `generated`, `reviewed`, or `trusted`. Contracts emitted
by this CLI use `generated` with `solid-check` as the generator.

## Effect summaries

The schema records:

- direct reactive accessor and store-path reads;
- accessor and store returns, including factory-to-factory propagation;
- inline, tracked, and deferred callback parameters;
- Promise and async-iterable behavior;
- inert exported values; and
- generated, reviewed, or trusted evidence.

Generation covers function declarations, exported arrows, overloads, nested
generics, async functions, multiple const declarations, classes, re-exports,
aliases, and subpath imports. Consumers support named imports and local aliases.
Calls in compiler-tracked JSX retain their tracked status; calls in ordinary
function bodies produce `strict-read-untracked` findings.

## Bundled and ecosystem contracts

Reviewed contracts for `solid-js` and `@solidjs/web` are embedded in the
checker and selected automatically from project imports.

The controlled Solid Primitives `next` checkout lives at
`../solid-primitives-next`. Its 98 packages publish generated contracts with
hashes for built declarations and implementations. Regenerate and validate the
corpus with:

```sh
pnpm --dir ../solid-primitives-next build
go build -o /tmp/solid-check ./cmd/solid-check

SOLID_CHECK_BIN=/tmp/solid-check \
SOLID_COMPILER_FACTS_BIN=../dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts \
  scripts/generate-solid-primitives-contracts.sh ../solid-primitives-next

SOLID_CHECK_BIN=/tmp/solid-check \
  scripts/validate-solid-primitives-contracts.sh ../solid-primitives-next
```

Generation repeats to a fixed point so contracted package dependencies retain
their transitive summaries. Validation checks artifact hashes and confirms each
package manifest publishes `solid-reactivity.json`.
