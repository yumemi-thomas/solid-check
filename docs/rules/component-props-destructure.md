# component-props-destructure

`SC1003` · **error** · violation · 🛠️ safe fix available

Component props are destructured, which unwraps each property once and severs
reactivity.

## What it does

Flags object destructuring of a component's props — both in the parameter list
(`function Card({ title })`) and in later bindings (`const { title } = props`).

When every destructured property is only read (never reassigned), solid-check
offers a safe fix that restores the `props` parameter and rewrites the body to
`props.<name>` accesses.

## Why is this bad?

In Solid, `props` is a reactive object: the *property access* is what subscribes.
Destructuring performs every access once, at component setup, and binds the plain
values. The component renders correctly the first time and then never updates when
the parent passes new props — one of the most common sources of "my UI doesn't
update" bugs.

## Examples

Examples of **incorrect** code for this rule:

```tsx
function Card({ title, body }) {
  return (
    <article>
      <h2>{title}</h2>
      <p>{body}</p>
    </article>
  );
}

function Avatar(props) {
  const { src } = props; // Same problem, one statement later.
  return <img src={src} />;
}
```

Examples of **correct** code for this rule:

```tsx
function Card(props) {
  return (
    <article>
      <h2>{props.title}</h2>
      <p>{props.body}</p>
    </article>
  );
}

// Splitting and defaulting props without destructuring:
function Field(props) {
  const rest = omit(props, "label");
  const merged = merge({ type: "text" }, rest);
  return <input {...merged} aria-label={props.label} />;
}
```

## How to fix

Keep the `props` object intact and read `props.<name>` inside JSX or a tracked
computation. To split props use `omit(props, ...keys)`; to default them use
`merge(defaults, props)`. Never destructure — not in the parameter list, not in the
body, and not in control-flow callbacks.

## Related

- [strict-read-untracked](strict-read-untracked.md) — the general untracked-read rule
