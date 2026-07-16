# Strict-read tracer bullet

The first end-to-end Reactive IR rule implements the Solid 2
`STRICT_READ_UNTRACKED` distinction for a deliberately narrow source shape:

```ts
export const [count, setCount] = createSignal(0);
```

The `createSignal` call must resolve through Type Facts to a declaration from
`solid-js`. The accessor may be exported, imported through aliases, and read in
capitalized function components that return JSX.

The engine joins each resolved accessor reference to its compiler facts:

- A read contained by a tracked JSX region is valid.
- A read contained by a deferred event callback is valid.
- A read in the immediate rendering-function body outside both regions emits
  `SC1001 strict-read-untracked`.

The finding links the call site to the original signal declaration and records
the provenance and execution-region proof steps. When every read in this
deliberately narrow scenario is compiler-tracked or deferred, the project is
certified; `--certify` succeeds for the corrected fixture.

This completes the Milestone 3 scenario, not the broader Solid 2 catalog.
Aliased primitive constructors, props, stores, memos, effect phases,
control-flow callbacks, and interprocedural forwarding remain work for later
milestones and are not claimed as covered by this tracer bullet.
