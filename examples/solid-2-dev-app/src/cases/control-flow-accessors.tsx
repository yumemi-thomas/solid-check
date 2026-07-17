import { For, Repeat, Show, createSignal } from "solid-js";

const initial = [{ id: 1, name: "Ada" }, { id: 2, name: "Grace" }];

// Bad: callback accessors are read into frozen locals outside JSX.
export function BadShowAccessor() {
  const [name] = createSignal("Ada");
  return <Show when={name()}>{value => { const frozen = value(); return <strong>{frozen}</strong> }}</Show>;
}
export function BadForItemAccessor() {
  const [items] = createSignal(initial);
  return <For each={items()} keyed={false}>{item => { const frozen = item(); return <span>{frozen.name}</span> }}</For>;
}
export function BadForIndexAccessor() {
  const [items] = createSignal(initial);
  return <For each={items()} keyed={() => true}>{(item, index) => { const frozen = index(); return <span>{frozen}: {item().name}</span> }}</For>;
}

// Good: Show/For values and indexes are accessors; Repeat's index is a number.
export function GoodShowAccessor() {
  const [name] = createSignal("Ada");
  return <Show when={name()}>{value => <strong>{value()}</strong>}</Show>;
}
export function GoodDefaultFor() {
  const [items] = createSignal(initial);
  return <For each={items()}>{(item, index) => <span>{index()}: {item.name}</span>}</For>;
}
export function GoodIndexKeyedFor() {
  const [items] = createSignal(initial);
  return <For each={items()} keyed={false}>{(item, index) => <span>{index}: {item().name}</span>}</For>;
}
export function GoodRepeatIndex() {
  return <Repeat count={3}>{index => <span>Skeleton {index + 1}</span>}</Repeat>;
}
