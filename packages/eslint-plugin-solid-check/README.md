# eslint-plugin-solid-check

This private workspace package adapts canonical `solid-check` certification
snapshots to ESLint. It performs no AST or type analysis.

```js
const solidCheck = require("./packages/eslint-plugin-solid-check");

module.exports = [
  {
    ...solidCheck.configs.recommended,
    settings: {
      solidCheck: {
        command: "/absolute/path/to/solid-check",
        cwd: "/absolute/path/to/project",
        project: "tsconfig.json"
      }
    }
  }
];
```

The adapter runs `solid-check --format json` once per project/configuration and
projects findings for the current file into ESLint reports. It preserves
canonical IDs and messages, includes proof evidence and related locations, and
exposes safe same-file fixes. Project-level uncertifiable results are reported
at the start of each linted file so missing analysis can never look clean.

The default `recommended` config enables the aggregate
`solid-check/certification` rule. For migration from `eslint-plugin-solid-2`,
use `configs.compatibility`, which exposes six exact snapshot-backed names:

- `solid-check/no-owned-scope-writes`;
- `solid-check/no-leaf-owner-operations`;
- `solid-check/no-untracked-read-in-effect-apply`;
- `solid-check/no-reactive-read-after-await`;
- `solid-check/no-destructure`; and
- `solid-check/no-stale-props-alias`.

Four broader or overlapping names are also available for explicit opt-in:
`components-return-once`, `no-untracked-reactivity`,
`no-reactive-value-misuse`, and `no-derived-signal-in-effect`. They are omitted
from presets so one canonical finding cannot produce duplicate reports.

Do not enable `certification` together with those compatibility rules unless
duplicate reports are desired. All rules share the same cached project
snapshot, so named configuration never starts additional analysis processes.

For CI pipelines that already generate a snapshot, set
`settings.solidCheck.snapshotPath`. Tests and embedded integrations may provide
`settings.solidCheck.snapshot` directly.

See the [rule audit](../../docs/eslint-rule-audit.md) for the migration map and the
rules deliberately retained in syntax-oriented ESLint plugins.

## Oxlint

Oxlint can load the same snapshot adapter through its ESLint-compatible
JavaScript plugin API. Run it through the checker so analysis happens exactly
once and no snapshot file becomes part of the user workflow:

```sh
solid-check oxlint --project tsconfig.json -- --format=default
```

Arguments after `--` are passed to Oxlint. The wrapper creates an ephemeral
snapshot, injects its path into the Oxlint child process, propagates Oxlint's
output and exit status, and removes the snapshot afterward. Set `OXLINT_BIN`
when Oxlint is not available as `oxlint` on `PATH`.

An explicit `settings.solidCheck.snapshotPath` remains available for debugging
and CI systems that intentionally retain an artifact.
