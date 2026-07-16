declare namespace JSX {
  interface IntrinsicElements {
    div: {};
  }
}

declare const count: () => number;

export const view = <div>{count()}</div>;
