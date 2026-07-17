# solid-check CLI package

This private workspace package is the user-facing launcher for the native
checker. It is not published.

An application installs one package and uses its binary directly:

```json
{
  "devDependencies": {
    "solid-check": "file:../../packages/cli"
  },
  "scripts": {
    "check": "solid-check --certify",
    "lint": "solid-check oxlint"
  }
}
```

The package also exports the snapshot adapter as `solid-check/eslint`:

```json
{
  "jsPlugins": [
    { "name": "solid-check", "specifier": "solid-check/eslint" }
  ],
  "rules": {
    "solid-check/certification": "error"
  }
}
```

Oxlint flags pass through directly. With npm scripts, use npm's `--` separator
or provide a dedicated script:

```sh
npm run lint -- --fix
npm run lint:fix
```

`npm run lint --fix` is not equivalent: npm consumes `--fix` as its own option.

The launcher forwards arguments, stdio, signals, and exit status to the native
binary. A packaged release can place `solid-check` and
`solid-compiler-facts` under `native/<platform>-<architecture>/`. While running
from this monorepo, it discovers the checkout, builds missing development
binaries with `make build`, and supplies the compiler-sidecar location
automatically.
