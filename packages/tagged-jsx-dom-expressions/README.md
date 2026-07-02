# tagged-jsx-dom-expressions

A tagged-template runtime for fine-grained signals libraries such as Solid.js.
Any signals library can be hooked into `html` by implementing a `Runtime` adapter.

`html` parses templates at runtime and installs reactive bindings against the
resulting DOM. Component references are real JavaScript values — either a name
registered via `.define()` or an expression hole — never a string parsed at
render time.

## Install

```sh
npm install tagged-jsx-dom-expressions
```

## Tooling

For editor support, the
[Tagged JSX Tools VS Code extension](https://marketplace.visualstudio.com/items?itemName=DanielRKling.tagged-jsx-vscode)
provides syntax highlighting, formatting, conversion commands, and TypeScript
diagnostics for JSX inside tagged template literals.

## Quick start

`createTaggedJSXRuntime(runtime)` returns a ready-to-use tag bound to the runtime of a signals library.
Components are registered via `.define({ ... })`, which returns a new tag with the
combined registry.

```ts
import { createTaggedJSXRuntime } from "tagged-jsx-dom-expressions";

// In this example, we will specifically connect Solid.js to tagged JSX, but any
// signals-style library could export a compatible interface.
import * as web from "@solidjs/web";

import { For, Show, createSignal } from "solid-js";
import { render } from "@solidjs/web";

// This creates a tagged JSX template tag that is reactive specifically to
// Solid.js signals by passing in the Solid.js web runtime, and registers
// two components for use via PascalCase tag names inside of the template
// strings.
const html = createTaggedJSXRuntime(web).define({ For, Show });

function Counter() {
  const [count, setCount] = createSignal(0);

  // Finally, write reactive templates!
  return html`
    <button onClick=${() => setCount(c => c + 1)}>
      Count: ${count}
    </button>

    <For each=${...}>
      ...
    </For>

    <Show when=${...}>
      ...
    </Show>
  `;
}

render(Counter, document.body);
```

The returned tag can be assigned to any local variable, but `html` matches the
default editor tooling configuration. If you wrap or rename the tag, the `.jsx`
self-reference below gives tooling a stable tag name to recognize.

## API

### `createTaggedJSXRuntime(runtime): TaggedJSXInstance<{}>`

Binds the runtime once and returns a tag with an empty component registry (`{}`).
The `runtime` object provides the reactive primitives and HTML facts the tag
needs at render time. When using `@solidjs/web`, the module's exports (the `web` in
`import * as web from "@solidjs/web"`) satisfy the shape.

The exported `Runtime` type can be implemented by any signals-style library for use
with `html` templates:

```ts
import { type Runtime } from "tagged-jsx-dom-expressions";

import { ... } from '@preact/signals'; // For example, make tagged JSX work with Preact Signals

const preactTaggedJSXRuntime: Runtime = {
  // ...implement the required shape for tagged JSX compatibility, using
  // Preact Signals primitives...
}

const html = createTaggedJSXRuntime(preactTaggedJSXRuntime);
```

The `Runtime` shape is:

```ts
interface Runtime {
  insert(parent: Node, accessor: any, marker?: Node | null, init?: any): any;
  spread(node: Element, accessor: any, skipChildren?: boolean): void;
  createComponent(Comp: (props: any) => any, props: any): any;
  mergeProps(...sources: unknown[]): any;
  SVGElements: Set<string>;
  MathMLElements: Set<string>;
  VoidElements: Set<string>;
  RawTextElements: Set<string>;
}
```

### `tag.define(components): TaggedJSXInstance<PreviousComponents & NewComponents>`

Returns a new tag with the supplied components merged into the registry.
The original tag is unchanged.

```ts
const base = createTaggedJSXRuntime(web);
const withFor = base.define({ For });
const withForAndShow = withFor.define({ Show });
```

### `tag.jsx`

A self-reference. This makes it possible to write templates through a `.jsx`
tag regardless of which local variable name was used to reference the instance.
That gives codemods, syntax highlighters, and formatters a stable tag name to
recognize.

```js
const withForAndShow = withFor.define({ Show }); // from above

console.log(withForAndShow === withForAndShow.jsx) // true

// Both of these are functionally equivalent, but the second one
// helps tooling that specifically looks for "jsx" in code.

withForAndShow`
    <For each=${...}>
      ...
    </For>

    <Show when=${...}>
      ...
    </Show>
`

withForAndShow.jsx`
    <For each=${...}>
      ...
    </For>

    <Show when=${...}>
      ...
    </Show>
`
```

### `tag.components`

The current registry, as a plain object.

## Template syntax

### Elements and components

```ts
html`<div />`; // self-closing
html`<div></div>`; // matched
html`<MyComponent />`; // registered component (capitalized)
html`<${MyComponent} />`; // inline component via expression hole
html`<${MyComponent}>...<//>`; // shorthand close for inline component (see Limitations)
```

- Tag names start with `a-zA-Z$_` and may contain `a-zA-Z0-9$.:-_`.
- Capitalized tag names are looked up in the registry. An unregistered
  capitalized name throws.
- Lowercase tag names are HTML/SVG/MathML elements; namespace is inferred
  from the element name and walked into nested children.

### Text and whitespace

- Text content is decoded as HTML (`&copy;` → `©`, `&gt;` → `>`).
- Pure-whitespace runs between elements are dropped from the AST.
- Leading and trailing whitespace inside an element is dropped when the
  element contains at least one expression hole.
- When in doubt, use an expression: `` html`<p>${" exact  spaces   "}</p>` ``.

### Attributes and properties

```ts
html`<input value="hi" />`           // static string attribute
html`<input disabled />`             // static boolean attribute
html`<input value=${val} />`         // dynamic attribute or property (auto)
html`<input prop:value=${val} />`    // forced DOM property
html`<input attr:foo=${val} />`      // forced HTML attribute
html`<input ...${props} />`          // spread
html`<input ref=${el => ...} />`     // ref (not reactive)
html`<input onClick=${handler} />`   // delegated event when supported by the runtime
html`<input onclick=${handler} />`   // bound listener (legacy lowercase)
```

`children` as an attribute is honored only when the element has no template
children, matching JSX behavior.

### Reactivity

A function passed to a non-event, non-`ref` attribute is auto-wrapped as a
getter if it takes zero arguments. Both forms below are reactive and
equivalent:

```ts
const [count] = createSignal(0);

html`<button count=${() => count()} />`;
html`<button count=${count} />`;
```

If the value you want to pass is itself a zero-arg function and you don't
want it auto-wrapped, wrap it again to break the heuristic:

```ts
html`<Route component=${() => Counter} />`;
```

`on*` and `ref` props are never auto-wrapped — they're passed as-is.

## JSX vs `html`

| Feature            | Solid JSX                               | `html` tagged template                                    |
| :----------------- | :-------------------------------------- | :------------------------------------------------------- |
| **Fragments**      | Required: `<>...</>` for multiple roots | None needed: returns a node, or array of nodes           |
| **Spread**         | `<div {...props} />`                    | `<div ...${props} />`                                    |
| **Comments**       | `{/* ... */}`                           | `<!-- ... -->` (stripped)                                |
| **Raw-text tags**  | `innerHTML` workaround                  | `<style>` / `<script>` bodies are raw text               |
| **Whitespace**     | JSX-style stripping                     | Trims between tags; preserves inside text                |
| **Reactivity**     | Signals auto-wrapped                    | Zero-arg functions auto-wrapped (use `() =>` to opt out) |
| **Component refs** | Identifier in scope                     | Registered name (`<Foo />`) or expression (`<${Foo} />`) |

Because `html` returns a `JSX.Element` — which can be a single node when the template
resolves to one root, or an array when it resolves to many — consumers that
need to iterate or spread should normalize the result:

```ts
const result = html`<div />`; // div
const nodes = [result].flat(); // [div]
```
```ts
const result = html`
  <div />
  <span />
`; // [div, span]

const nodes = [result].flat(); // [div, span]
```

## License

You've got a license to build awesome apps: MIT
