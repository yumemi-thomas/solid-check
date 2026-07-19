# solid-checker

Project-level reactivity checker and language server for Solid. The package
ships native `solid-check`, `solid-checkd`, and `solid-typefacts` executables
for supported platforms.

Install it as a development dependency:

```sh
npm install --save-dev solid-checker
```

Then run `solid-check --certify` or start the `solid-checkd` language server.

In StackBlitz, WebContainers, or a browser worker, import the process-free
WASM API from the same package:

```js
import { checkSync } from "solid-checker";
```

Supported targets are Linux (x64 and arm64), macOS (x64 and arm64), and
Windows (x64). npm installs only the matching
`@solid-checker/binding-<target>` optional dependency; the portable package
contains the launchers. The launcher forwards arguments, stdio, signals, and
exit status. While running from this monorepo, it builds missing development
binaries with `make build-rust`.
