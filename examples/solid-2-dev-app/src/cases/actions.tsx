import { action, createMemo, createSignal } from "solid-js";

// Bad: calling an action from a non-leaf owned computation is forbidden.
export function BadActionInMemo() {
  const save = action(function* () {});
  createMemo(() => save());
  return <span>Saving</span>;
}
export function BadActionInComponent() {
  const save = action(function* () {});
  save();
  return <span>Saved</span>;
}

// Good: event handlers are an action boundary; actions may perform writes.
export function GoodActionInEvent() {
  const [name, setName] = createSignal("");
  const save = action(function* (next: string) { setName(next) });
  return <button onClick={() => save("Ada")}>{name()}</button>;
}
export function GoodActionReference() {
  const save = action(function* () { console.log("saved") });
  return <button onClick={save}>Save</button>;
}
