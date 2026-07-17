declare namespace JSX {
  interface IntrinsicElements { div: {}; span: {} }
  interface Element {}
}

declare module "solid-js" {
  export function createSignal<T>(value: T): [() => T, (value: T) => void];
  export function Show<T>(props: { when: T; children: (value: () => T) => JSX.Element }): JSX.Element;
}
