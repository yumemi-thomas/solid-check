import type { JSX } from "@solidjs/web";
import { createMemo, Loading } from "solid-js";

// A Solid 2 async derivation: its value only exists once the promise settles.
const user = createMemo(async () => {
  const response = await fetch("/api/me");
  return (await response.json()) as { name: string };
});

// BUG: the async value is rendered with no dominating <Loading> boundary, so
// there is nothing to show while it is pending. Proving this needs the new
// Solid Oxc compiler's JSX execution facts to see the boundary structure —
// a plain AST linter can't reason about it (SC5003).
export function Profile() {
  return <div>{user().name}</div>;
}

export function LoadingWrapper(props: { children: JSX.Element, fallback?: JSX.Element }) {
  return <Loading fallback={props.fallback ?? <span>Loading…</span>}>{props.children}</Loading>;
}

export function WrongLoadingWrapper(props: { children: JSX.Element, fallback?: JSX.Element }) {
  return <div>{props.children}</div>;
}

// Correct: the same read under a <Loading> boundary is certified.
export function ProfileOk() {
  return <LoadingWrapper fallback={<span>Loading…</span>}>{user().name}</LoadingWrapper>;
}

export function ProfileNotOk() {
  return <WrongLoadingWrapper fallback={<span>Loading…</span>}>{user().name}</WrongLoadingWrapper>;
}
