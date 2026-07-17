import type { JSX } from "@solidjs/web";
import { merge, omit } from "solid-js";

interface CardProps { title: string; subtitle?: string; children?: JSX.Element; id?: string }

// Bad: destructuring freezes reactive props at component creation time.
export function BadParameterDestructure({ title }: CardProps) {
  return <h2>{title}</h2>;
}
export function BadBodyDestructure(props: CardProps) {
  const { subtitle } = props;
  return <p>{subtitle}</p>;
}
export function BadAliasedDestructure(props: CardProps) {
  const card = props;
  const { title } = card;
  return <h2>{title}</h2>;
}

// Good: read through the proxy, and use Solid's reactive prop helpers.
export function GoodDirectProps(props: CardProps) {
  return <article><h2>{props.title}</h2><p>{props.subtitle}</p></article>;
}
export function GoodDefaultProps(props: CardProps) {
  const merged = merge({ subtitle: "No subtitle" }, props);
  return <p>{merged.subtitle}</p>;
}
export function GoodForwardedProps(props: CardProps) {
  const rest = omit(props, "title", "subtitle", "children");
  return <section {...rest}><h2>{props.title}</h2>{props.children}</section>;
}
