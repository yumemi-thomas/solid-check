# solid-check CLI package

This private workspace package is the user-facing launcher for the Rust CLI
and language server. It is not published yet.

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

The launcher forwards arguments, stdio, signals, and exit status. A packaged
release places `solid-check`, `solid-checkd`, and the matching Go
`solid-typefacts` helper under `native/<platform>-<architecture>/`; the Rust
entry points supervise the helper and verify its schema/build handshake. While
running from this monorepo, the launcher builds missing binaries with
`make build-rust`.
