import { createMemo, createSignal, createTrackedEffect, flush, onSettled } from "solid-js";

// Bad: leaf owners cannot create child primitives or synchronously flush.
export function BadPrimitiveInSettled() {
  onSettled(() => { createSignal(0) });
  return <span>Mounted</span>;
}
export function BadPrimitiveInTrackedEffect() {
  createTrackedEffect(() => createMemo(() => 1));
  return <span>Watching</span>;
}
export function BadFlushInTrackedEffect() {
  const [count] = createSignal(0);
  createTrackedEffect(() => { count(); flush() });
  return <output>{count()}</output>;
}

// Good: create primitives in the component owner; leaf callbacks do leaf work.
export function GoodSettledLeaf() {
  const [count] = createSignal(0);
  const doubled = createMemo(() => count() * 2);
  onSettled(() => console.log(doubled()));
  return <output>{doubled()}</output>;
}
export function GoodTrackedLeaf() {
  const [count] = createSignal(0);
  createTrackedEffect(() => console.log(count()));
  return <output>{count()}</output>;
}
