# Zed integration

The local extension in `packages/zed-solid-check` registers `solid-checkd` for
JavaScript, JSX, TypeScript, and TSX. It follows the command-discovery and
project binary override pattern used by the
[Oxc Zed extension](https://github.com/oxc-project/oxc-zed).

## Why two language servers

Zed runs `solid-checkd` and Oxlint together:

- `solid-checkd` owns proof-backed Solid semantic diagnostics, related
  locations, explanations, and fixes. It receives unsaved document overlays.
- Oxlint owns fast syntax and style rules.

`solid-checkd` publishes diagnostics for the complete configured TypeScript
project after Zed finishes initialization, including files that are not open.
After an editor change it fingerprints diagnostics per file and publishes only
changed files (including an empty list when a file becomes clean), so the
project-wide Problems view stays complete without retransmitting the workspace
on every keystroke. Diagnostics for open documents carry the exact LSP document
version, preventing an older on-disk result from being retained after an
unsaved edit or quick fix.

## Quick fixes

Canonical safe fixes are exposed as preferred LSP quick fixes. Place the cursor
on an `SC1003` component-props destructuring diagnostic and press `Cmd+.` on
macOS (`Ctrl+.` on Linux/Windows), or run **editor: toggle code actions** from
the command palette. The action **Fix: Keep component props reactive** applies the
coordinated parameter and JSX edits.

After rebuilding the local checker or compiler sidecar, run **editor: restart
language server** once so Zed loads the new binaries.

Do not enable `solid-check/certification` in the Oxlint editor configuration.
That adapter consumes an immutable snapshot, so it is appropriate for a lint
command but not for per-keystroke editor state. A separate syntax-only Oxlint
configuration also prevents duplicate semantic diagnostics.

## Local installation

Build the checker, compiler sidecar, and Zed extension:

```sh
make zed-setup
```

Zed development extensions require one UI installation step:

1. Run `zed: extensions` from the command palette.
2. Choose **Install Dev Extension**.
3. Select `packages/zed-solid-check` from this repository.

This does not publish or install a package from a registry. Zed reloads the
local extension during development.

## Project configuration

Configure the language server in `.zed/settings.json`:

```jsonc
{
  "languages": {
    "TSX": {
      "language_servers": ["typescript-ls", "solid-check", "oxlint", "!vtsls", "!typescript-language-server", "..."]
    }
  },
  "lsp": {
    "solid-check": {
      "binary": {
        "path": "/absolute/path/to/solid-check/bin/solid-checkd",
        "arguments": ["--project", "tsconfig.json"],
        "env": {
          "SOLID_COMPILER_FACTS_BIN": "/absolute/path/to/solid-check/third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts"
        }
      }
    },
    "oxlint": {
      "initialization_options": {
        "settings": {
          "configPath": ".oxlintrc.editor.json",
          "fixKind": "safe_fix",
          "run": "onType"
        }
      }
    }
  }
}
```

The extension accepts relative binary and compiler-sidecar paths and resolves
them from the Zed worktree root. If `solid-checkd` is on `PATH`, the binary
override can be omitted.

The repository contains two complete settings variants:

- Root `.zed/settings.json` applies when the whole `solid-check` monorepo is the
  Zed worktree. It points the language servers at the development app.
- `examples/solid-2-dev-app/.zed/settings.json` applies when only that example
  directory is opened as the Zed worktree.

Zed reads `.zed/settings.json` only from a worktree root; it does not apply the
nested example settings when the whole monorepo is open. Both variants use the
same syntax-only Oxlint configuration and create no certification snapshot.

The checked-in variants select one general TypeScript server (`typescript-ls`)
and disable `vtsls` plus the legacy `typescript-language-server`. `solid-check`
is a separate semantic proof server and cannot share a process with a general
TypeScript language server; replacing the duplicate TypeScript servers keeps
the project at two semantic servers rather than three.
