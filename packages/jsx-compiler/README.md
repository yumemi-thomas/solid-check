# @dom-expressions/jsx-compiler

Experimental AST-native JSX to DOM Expressions compiler implemented with Oxc.

## Installation

```bash
npm install @dom-expressions/jsx-compiler
```

The package ships prebuilt native binaries as per-platform packages
(`@dom-expressions/jsx-compiler-darwin-arm64`, `-darwin-x64`, `-linux-x64-gnu`,
`-linux-arm64-gnu`, `-win32-x64-msvc`). Your package manager installs the one
matching your platform automatically through `optionalDependencies`. On other
platforms, build from source with `pnpm run build` inside
`packages/jsx-compiler` (requires a Rust toolchain).

## Usage

This package exposes a compiler backend API. It is not a Vite, Rollup, or Babel
plugin by itself; integrations should call `transform()` once per source module.

```js
const { transform } = require("@dom-expressions/jsx-compiler");

const result = transform(`const view = <div>Hello</div>;`, {
  filename: "App.jsx",
  moduleName: "dom",
  generate: "dom"
});

console.log(result.code);
```

`transformAsync()` is also available for integration points that expect a
promise-returning transform:

```js
const { transformAsync } = require("@dom-expressions/jsx-compiler");

const result = await transformAsync(source, {
  filename: "App.jsx",
  moduleName: "dom",
  generate: "dom"
});
```

### Solid-Style DOM

Solid's DOM compiler preset uses DOM output with custom-element context
capture enabled. This compiler defaults `contextToCustomElements` to `true` to
match that behavior.

```js
const result = transform(source, {
  filename: "App.jsx",
  moduleName: "dom",
  generate: "dom",
  hydratable: true,
  builtIns: ["For", "Show"]
});
```

Use `dev: true` with `hydratable: true` to emit dev hydration walk validation
helpers such as `getFirstChild` / `getNextSibling`.

### SSR

```js
const result = transform(source, {
  filename: "entry-server.jsx",
  moduleName: "dom/server",
  generate: "ssr",
  hydratable: true,
  builtIns: ["For", "Show"]
});
```

### Universal

```js
const result = transform(source, {
  filename: "scene.jsx",
  moduleName: "renderer",
  generate: "universal"
});
```

### Dynamic Renderers

Dynamic mode uses the universal renderer as the fallback and can route a
configured set of native tags to the DOM renderer.

```js
const result = transform(source, {
  filename: "hybrid.jsx",
  moduleName: "renderer",
  generate: "dynamic",
  renderers: [
    {
      name: "dom",
      moduleName: "dom",
      elements: ["div", "span", "button", "input"]
    }
  ]
});
```

### Source Maps

Pass `sourceMap: true` to receive a JSON source map string in `result.map`.

```js
const result = transform(source, {
  filename: "App.jsx",
  moduleName: "dom",
  sourceMap: true
});

console.log(result.map);
```

### Options

Supported options track the Babel plugin where currently implemented:

- `filename`
- `moduleName`
- `generate`: `"dom"`, `"ssr"`, `"universal"`, or `"dynamic"`
- `hydratable`
- `dev`
- `sourceMap`
- `contextToCustomElements`
- `delegateEvents`
- `delegatedEvents`
- `omitQuotes`
- `omitAttributeSpacing`
- `inlineStyles`
- `effectWrapper`: `"effect"` or `false`
- paired wrapperless mode: `wrapConditionals: false` with `memoWrapper: false`
- `staticMarker`
- `validate`
- `omitNestedClosingTags`
- `omitLastClosingTag`
- `builtIns`
- `requireImportSource`
- `renderers`

## Current Scope

This package is the AST-native compiler backend. It currently has checked fixture coverage for
the DOM, hydratable DOM, dev hydratable DOM, SSR, hydratable SSR, universal, dynamic, no-inline-styles, and wrapperless renderer paths.

- `generate: "dom"`
- `generate: "ssr"`
- `generate: "universal"`
- `generate: "dynamic"`
- native elements, components, fragments, refs, spreads, dynamic text, events, and
  attribute handling covered by the checked fixture suites for those targets
- Solid-compatible defaults such as `contextToCustomElements: true`
- option coverage for `hydratable`, `dev`, `delegateEvents`, `delegatedEvents`,
  `omitQuotes`, `omitAttributeSpacing`, `inlineStyles`, `effectWrapper: false`,
  paired `wrapConditionals: false` / `memoWrapper: false`, `requireImportSource`,
  `staticMarker`, `validate`, `omitNestedClosingTags`, `omitLastClosingTag`,
  `builtIns`, and dynamic `renderers`
- source maps for the implemented path

## Not Implemented Yet

The compiler intentionally rejects unsupported features instead of pretending to support them:

- DOM `namespaceElements` sections that the current Oxc parser rejects before transform
  (for example, hyphenated JSX member segments)
- arbitrary custom renderer names beyond dynamic DOM renderer override plus universal fallback
- custom `effectWrapper` / `memoWrapper` helper names
- unpaired `wrapConditionals: false` or `memoWrapper: false`
- unknown/custom namespaced DOM attributes outside known runtime namespaces such as `xlink`

## Architecture

The implementation is AST-native:

1. Parse with Oxc.
2. Transform JSX nodes with `VisitMut`.
3. Build replacement expressions and helper declarations with `AstBuilder`.
4. Codegen once with Oxc.

The module layout mirrors the Babel plugin shape where possible:

- `src/config.rs`
- `src/shared/ast.rs`
- `src/shared/transform.rs` for shared traversal and target dispatch
- `src/shared/component.rs`
- `src/shared/utils.rs`
- `src/dom/element.rs`
- `src/dom/template.rs`
- `src/ssr/mod.rs`
- `src/ssr/transform.rs`
- `src/universal/mod.rs`
- `src/universal/transform.rs`

