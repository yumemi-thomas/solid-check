# Interprocedural solver

Milestone 4 extends the strict-read proof through function calls while keeping
graph details behind the `solver` package seam. Reactive IR records function
identities, direct reads, call sites, callback invocations, returned-closure
effects, and compiler execution roles. The solver produces concrete read
obligations only when those summaries reach rendering entrypoints.

The current conformance slices cover:

- cross-file helper calls in tracked and untracked rendering regions;
- generic functions and overloaded TypeScript declarations;
- identifier callback arguments forwarded through generic helpers;
- returned zero-argument closures assigned from cross-file factories;
- mutually recursive functions solved as strongly connected components;
- direct `createStore` property paths propagated through helpers;
- project updates that remove a helper effect and invalidate its callers.

Summary solving uses Tarjan strongly connected components. Dependencies are
solved before their callers, and recursive components iterate to a fixed point.
Project updates currently invalidate conservatively through the Type Facts
affected-file graph and rebuild summaries; selective persisted summary caching
is deferred until performance measurements justify it.

This remains intentionally narrower than general JavaScript effect inference.
The current source extraction recognizes function declarations, identifier
callback arguments, direct factory bindings, zero-argument returned arrows,
and concrete dotted store paths. Arbitrary closures, destructuring, computed
properties, methods, and dynamic dispatch require later Reactive IR coverage
and must not be treated as covered by these fixtures.
