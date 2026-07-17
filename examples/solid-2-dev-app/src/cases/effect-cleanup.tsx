import { createEffect, createSignal, createTrackedEffect, onCleanup, onSettled } from "solid-js";

// Bad: leaf owners return cleanup; they cannot register nested cleanup.
export function BadCleanupInSettled() {
  onSettled(() => onCleanup(() => console.log("disposed")));
  return <span>Mounted</span>;
}
export function BadCleanupInTrackedEffect() {
  createTrackedEffect(() => onCleanup(() => console.log("disposed")));
  return <span>Watching</span>;
}

// Good: return cleanup from apply, tracked-effect, and settled callbacks.
export function GoodSplitEffectCleanup() {
  const [query] = createSignal("");
  createEffect(() => query(), value => {
    const timer = setTimeout(() => console.log(value), 100);
    return () => clearTimeout(timer);
  });
  return <output>{query()}</output>;
}
export function GoodTrackedCleanup() {
  createTrackedEffect(() => {
    const controller = new AbortController();
    return () => controller.abort();
  });
  return <span>Watching</span>;
}
export function GoodSettledCleanup() {
  onSettled(() => {
    const observer = new ResizeObserver(() => {});
    return () => observer.disconnect();
  });
  return <span>Observed</span>;
}
