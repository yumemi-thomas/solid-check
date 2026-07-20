# Package contracts

Milestone 5 introduces `solid-reactivity.json`, a non-executable summary that
preserves exported reactive reads when a dependency's implementation source is
not part of the consuming TypeScript project.

## Workflow

Analyze the package and emit its solved exported function summaries:

```sh
solid-checker --project package/tsconfig.json \
  --emit-contract package/solid-reactivity.json \
  --package-name reactive-package \
  --package-version 1.0.0 \
  --declaration-artifact package/index.d.ts \
  --implementation-artifact package/index.js
```

Load the contract while analyzing a consumer:

```sh
solid-checker --project app/tsconfig.json \
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

Application developers can also maintain a contract when a package does not
publish one. Put it at:

```text
.solid-checker/contracts/<package>/solid-reactivity.json
```

Scoped names retain their directory structure, for example
`.solid-checker/contracts/@scope/package/solid-reactivity.json`. Project-owned
contracts are discovered automatically and override contracts from
`node_modules`; an explicit `--contract` still has the highest precedence.
The same `--emit-contract` workflow can generate this file when the package
source and a TypeScript project for it are available, or it can be authored
against the contract schema and checked with `--validate-contract`.

Before checking, inspect imported Solid-dependent packages and their contract
coverage:

```sh
solid-checker --project app/tsconfig.json --check-contracts
```

The command reports bundled, published, local, and explicit contracts. It exits
with status 1 when a package whose manifest depends on or peers with Solid has
no contract.

Normal analysis performs the same completeness check. A missing contract emits
the uncertifiable `SC9005 package-contract-missing` finding at the package
import, changes the snapshot status to `uncertifiable`, and causes `--certify`
to exit with status 1. This behavior is shared by one-shot and retained-daemon
checks. Use `--check-contracts` when only the focused coverage report is needed.

Validate contracts and their artifacts without opening a TypeScript project:

```sh
solid-checker --validate-contract package/solid-reactivity.json
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
by this CLI use `generated` with `solid-checker` as the generator.

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

The pinned Solid Primitives `next` corpus contains 98 packages that publish
generated contracts with hashes for built declarations and implementations.
Regenerate and validate the complete corpus with:

```sh
make corpus
```

Set `SOLID_PRIMITIVES_CORPUS=/path/to/clean/checkout` to reuse a local clone.

Generation repeats to a fixed point so contracted package dependencies retain
their transitive summaries. Validation checks artifact hashes and confirms each
package manifest publishes `solid-reactivity.json`.
