# Solid Check for Zed

This private Zed extension registers `solid-checkd` for JavaScript, JSX,
TypeScript, and TSX. Its structure and binary-configuration behavior follow
the official `oxc-project/oxc-zed` extension.

Until the extension is published, install it locally in Zed:

1. Run `make build` at the repository root.
2. Open `zed: extensions` from the command palette.
3. Choose **Install Dev Extension**.
4. Select `packages/zed-solid-check`.

When a project installs `solid-checker`, the extension automatically uses
`node_modules/.bin/solid-checkd`. An explicit binary configuration takes
precedence:

The default project is `tsconfig.app.json` when present (as in Vite
applications), otherwise `tsconfig.json`.

```jsonc
{
  "lsp": {
    "solid-check": {
      "binary": {
        "path": "../../bin/solid-checkd",
        "arguments": ["--project", "tsconfig.json"],
        "env": {
          "SOLID_COMPILER_FACTS_BIN": "../../third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts"
        }
      }
    }
  }
}
```

Relative binary and compiler paths resolve from the Zed worktree root. Without
an explicit path or local npm package, the extension falls back to
`solid-checkd` on `PATH`.

Safe canonical fixes are available through Zed code actions. On macOS, place
the cursor on a diagnostic and press `Cmd+.`; use `Ctrl+.` on Linux or Windows.

The repository root and the development app each provide worktree-relative Zed
settings. Zed only reads `.zed/settings.json` at the active worktree root, so
use the root variant when the whole monorepo is open.
