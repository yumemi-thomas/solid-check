import type { JSX } from "@solidjs/web";
import { Errored, Loading, createMemo } from "solid-js";

const value = createMemo(async () => "Ready");

// Bad: async render reads need a real, dominating Loading boundary.
export function BadNoLoadingBoundary() {
  return <strong>{value()}</strong>;
}
function FakeLoading(props: { children: JSX.Element }) {
  return <section aria-busy="true">{props.children}</section>;
}
export function BadFakeLoadingBoundary() {
  return <FakeLoading>{value()}</FakeLoading>;
}

// Good: direct and user-defined wrappers around Solid's Loading are recognized.
export function GoodDirectLoadingBoundary() {
  return <Loading fallback={<span>Loading…</span>}><strong>{value()}</strong></Loading>;
}
function RealLoading(props: { children: JSX.Element }) {
  return <Loading fallback={<span>Loading…</span>}>{props.children}</Loading>;
}
export function GoodWrappedLoadingBoundary() {
  return <Errored fallback={error => <pre>{String(error)}</pre>}><RealLoading>{value()}</RealLoading></Errored>;
}
