# Performance benchmarking and profiling

The native checker benchmarks split the analysis lifecycle into four stable
measurements over the 38-source Reactivity v2 fixture:

- `BenchmarkNativeEngineColdSnapshot` opens the TypeScript project, builds
  compiler execution maps, constructs Reactive IR, and solves one snapshot;
- `BenchmarkNativeEngineSnapshot` rebuilds and solves snapshots in an already
  open project session;
- `BenchmarkNativeEngineIncrementalUpdate` alternates one full-document editor
  overlay and measures the Type Facts update path; and
- `BenchmarkNativeEngineIncrementalSnapshot` measures the editor-critical
  update followed by a complete immutable snapshot.

`BenchmarkProjectReferenceLookups` separately measures a representative batch
of Type Facts reference queries. It keeps reference-index improvements visible
even when another snapshot stage dominates the complete checker profile.

The benchmarks use the real TypeScript-Go adapter and native checker. A
deterministic in-process compiler-facts analyzer isolates Go and TypeScript
costs from Rust process startup and JSON transport. Use the CLI timing command
below when measuring the complete production path through the Rust/Oxc
sidecar.

## Stable baseline

Run five samples with allocation reporting:

```sh
make benchmark
```

For comparable results, record the commit, Go version, operating system,
architecture, CPU, power mode, and whether the machine was otherwise idle.
Do not compare results collected with different fixture revisions as if they
were the same workload.

`ns/op` measures latency, `B/op` exposes allocation pressure, and `allocs/op`
helps distinguish retained data from transient object churn. The benchmark
commands intentionally do not enforce provisional budgets until measurements
exist on representative applications and CI hardware.

## CPU and allocation profiles

Capture a ten-second profile of the warm editor-critical path:

```sh
make profile
```

This writes ignored artifacts to `.profiles/` and prints the top CPU and
allocation-space entries. Inspect the complete call graph interactively with:

```sh
go tool pprof -http=:0 .profiles/incremental.cpu.pprof
go tool pprof -http=:0 -alloc_space .profiles/incremental.mem.pprof
```

Use `alloc_space` to optimize total allocation traffic. Use the default
in-use view when investigating retained heap after first reproducing the
retention with a long-lived language-server workload.

Profile one stage independently by changing the `-bench` expression, for
example:

```sh
go test ./internal/engine -run '^$' \
  -bench '^BenchmarkNativeEngineIncrementalUpdate$' -benchtime=10s \
  -cpuprofile .profiles/update.cpu.pprof
```

## End-to-end CLI timing

Build the checker and sidecar, then time the complete process boundary:

```sh
make build
/usr/bin/time -p env \
  SOLID_COMPILER_FACTS_BIN="$PWD/third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts" \
  ./bin/solid-check \
  --project internal/engine/testdata/eslint-reactivity-v2/tsconfig.json \
  --format json >/dev/null
```

Repeat this command rather than treating the first run as representative; the
first sample includes cold filesystem and executable-page effects. This timing
starts a new Go process and Rust sidecar each time. The incremental Go
benchmarks instead model a persistent language-server session.

## Interpreting changes

Compare benchmark revisions with the same command and environment. Prefer a
statistical comparison tool such as `benchstat` when it is available:

```sh
make benchmark > before.txt
make benchmark > after.txt
benchstat before.txt after.txt
```

A performance change is credible when repeated samples move consistently and
the relevant CPU or allocation profile changes in the predicted call path.
Always run correctness tests after an optimization; faster analysis that
changes the certification snapshot is not an improvement.

## Initial baseline and profile

The first baseline was captured on an Apple M4 Pro (`darwin/arm64`) with Go
1.26. The median of five benchmark samples measured approximately:

| Benchmark | Latency | Allocated bytes | Allocations |
| --- | ---: | ---: | ---: |
| Cold snapshot | 88.1 ms/op | 37.2 MB/op | 213,435/op |
| Snapshot in open session | 69.7 ms/op | 13.7 MB/op | 174,288/op |
| Incremental update | 16.8 ms/op | 22.9 MB/op | 36,136/op |
| Incremental update + snapshot | 88.6 ms/op | 37.1 MB/op | 212,455/op |

These numbers characterize the fixture and machine, not a production budget.
The ten-second incremental profile attributed about 55% of cumulative sampled
CPU to snapshot construction and 11% to Type Facts update. Regular-expression
execution accounted for roughly one third of total CPU, led by repeated Solid
call-argument region discovery. TypeScript program update accounted for about
62% of allocation space because the current overlay path reparses and rebinds
the project. Reference traversal was only about 2% of sampled CPU in this
fixture, so its whole-project algorithm remains a scaling risk to test with
larger reactive applications rather than the first measured optimization.

## Per-generation reference index result

The first measured optimization replaced one whole-project AST traversal per
queried symbol with one lazy canonical-symbol index per TypeScript program
generation. Successful overlay updates invalidate the index; repeated queries
within the generation are map lookups returning defensive copies.

On the same machine and fixture, five-sample medians changed as follows:

| Benchmark | Before | After | Change |
| --- | ---: | ---: | ---: |
| Focused reference-query batch | 761.0 µs/op | 0.619 µs/op | ~1,230× faster |
| Cold snapshot | 88.1 ms/op | 84.1 ms/op | 4.6% faster |
| Snapshot in open session | 69.7 ms/op | 66.1 ms/op | 5.1% faster |
| Incremental update | 16.8 ms/op | 16.6 ms/op | 1.2% faster |
| Incremental update + snapshot | 88.6 ms/op | 84.4 ms/op | 4.7% faster |

Focused lookup allocation fell from four to two allocations per batch and from
about 176 to 64 bytes. Whole-checker allocation volume remained effectively
flat because the editor benchmark creates a fresh index after every update.
The follow-up CPU profile no longer showed `References` as a material hotspot;
regular-expression call-region discovery remained about one third of total CPU
and is the next measured optimization target.

## Build-scoped Solid call discovery result

The second measured optimization consolidated argument-region, call-span, and
source-call discovery behind one build-scoped cache keyed by source path and
Solid API name. Each API's candidate scan and Type Facts resolution now runs
once per immutable Reactive IR build; all downstream rules share those call
records. No cached value crosses a snapshot or TypeScript project generation.

Compared with the post-reference-index baseline, five-sample medians changed
approximately as follows:

| Benchmark | Before | After | Change |
| --- | ---: | ---: | ---: |
| Cold snapshot | 84.1 ms/op | 27.0 ms/op | 67.8% faster |
| Snapshot in open session | 66.1 ms/op | 9.8 ms/op | 85.2% faster |
| Incremental update | 16.6 ms/op | 16.1 ms/op | 3.1% faster |
| Incremental update + snapshot | 84.4 ms/op | 28.2 ms/op | 66.6% faster |

Warm snapshot allocation fell from about 13.6 MB and 174,000 allocations per
operation to 2.9 MB and 32,000 allocations. The complete editor path fell from
about 37.0 MB and 213,000 allocations to 26.2 MB and 70,000 allocations.

The follow-up profile reduced regular-expression discovery from roughly 34% to
11% of sampled CPU, and `solidCallArgumentRegions` disappeared from the
material hotspot list. TypeScript program rebuilding became dominant at about
31% of CPU and 87% of allocation space, making actual incremental TypeScript
program reuse the next measured optimization target.

## Single-file TypeScript program reuse result

The third measured optimization uses TypeScript-Go's `Program.UpdateProgram`
for exactly one accepted, non-deleted source-file edit. The update is still
transactional, and TypeScript-Go performs its own structural checks: changes to
imports or other program-shape inputs automatically fall back to constructing a
new program. Added or deleted files, `tsconfig` edits, and multi-file batches
continue through the explicit full-build path.

Compared with the post-call-cache baseline, five-sample medians changed
approximately as follows:

| Benchmark | Before | After | Change |
| --- | ---: | ---: | ---: |
| Incremental update | 16.1 ms/op | 0.214 ms/op | ~75× faster (98.7%) |
| Incremental update + snapshot | 28.2 ms/op | 10.4 ms/op | 63.0% faster |
| Incremental update allocation | 22.9 MB/op | 0.33 MB/op | 98.6% lower |
| Update + snapshot allocation | 26.2 MB/op | 3.6 MB/op | 86.3% lower |

Incremental update allocations also fell from about 36,100 to 1,209 per
operation; the complete editor path fell from about 70,200 to 35,300. Focused
equivalence coverage confirms that an import-changing edit takes the compiler's
fallback path and produces the same updated type facts.

The follow-up profile no longer identifies TypeScript program rebuilding as a
material CPU hotspot. Snapshot construction now dominates the editor path,
with Solid call discovery and checker queries accounting for most useful work.
Repeated `filepath.Abs`/`os.Getwd` calls also became visible once updates became
cheap, so path normalization was measured next.

## Session-relative path normalization result

The editor update boundary previously normalized each relative JSX edit more
than once and passed the original relative path to Type Facts, which normalized
it again. More importantly on macOS, even one `filepath.Abs` per edit consulted
the process working directory through filesystem `stat` calls. Sessions now
capture their opening working directory once, canonicalize each changed path
once against it, and pass that canonical path to compiler and Type Facts
updates.

The five-sample incremental snapshot median moved from approximately 11.06 ms
to 10.68 ms (about 3.5%); allocation volume was effectively unchanged. This is
a modest and somewhat noise-sensitive latency result, not a major throughput
gain. The stronger evidence is structural: the ten-second CPU profile removed
`os.Getwd` and `syscall.rawsyscalln`, previously 20–28% of samples, from the
profile entirely. Regular-expression Solid call discovery is now the dominant
measured CPU path.

## TypeScript-AST call discovery result

The next optimization moved call-expression discovery and argument structure
behind the Type Facts seam. The TypeScript-Go adapter now walks parsed call
expressions once per source and returns only source byte locations, argument
locations, and an opaque alias-resolved target identity. Reactive IR indexes
those facts by the target's Solid declaration name. TypeScript AST nodes and
checker objects remain private to the adapter, and non-TypeScript test adapters
retain the prior text fallback through an optional capability.

Compared with the session-relative-path baseline, five-sample medians changed
approximately as follows:

| Benchmark | Before | After | Change |
| --- | ---: | ---: | ---: |
| Incremental update + snapshot | 10.94 ms/op | 4.13 ms/op | 62.3% faster |
| Editor-path allocation | 3.60 MB/op | 2.25 MB/op | 37.5% lower |
| Editor-path allocations | 35,240/op | 19,119/op | 45.7% lower |

Namespace imports, aliased targets, generic calls, and nested argument syntax
are resolved through TypeScript rather than reconstructed from text. The
follow-up profile reduced `solidSourceCalls` to about 5% cumulative CPU; the AST
visitor itself was about 1–2%. Remaining regular-expression cost comes from
binding/function/export structure extraction and dynamically compiled local
patterns, which can migrate incrementally through similarly opaque Type Facts
queries.

The same parser-derived call list was then reused for typed-accessor reads.
Because each call includes an opaque target identity, Reactive IR can describe
each target's type once per build instead of rendering the same accessor type
at every call site. The five-sample snapshot median fell again from 4.14 ms to
3.38 ms (18.4%); allocation fell from 2.25 MB to 1.70 MB (24.7%), and allocation
count fell from about 19,120 to 14,062 (26.5%).

Finally, fixed local patterns previously passed to `regexp.MustCompile` inside
hot functions were promoted to package-level immutable matchers. This modestly
reduced median latency from 3.38 ms to 3.23 ms (4.2%), while allocation fell to
1.37 MB (19.1% lower) and roughly 11,221 allocations (20.2% lower). The
follow-up profile confirms that parser-derived variable binding facts are the
next coherent structural migration: `collectReactiveBindingFacts` and
overlapping owned-write/write binding scans are now the leading removable regex
family.

That migration adds one optional bulk Type Facts query for variables initialized
directly by resolved calls. It preserves direct versus array bindings, exact
top-level name ranges, and omitted tuple slots while keeping AST nodes private
to TypeScript-Go. Reactive binding collection and signal/store setter discovery
share the same per-source facts; other adapters retain the regex fallback.

Compared with the post-hoisting baseline, the five-sample incremental snapshot
median fell from 3.23 ms to 2.73 ms (15.6%). Allocation moved from 1.37 MB to
1.34 MB and allocation count from about 11,221 to 10,745. The gain is primarily
CPU from removing repeated structural scans. The follow-up profile no longer
shows reactive binding collection or signal-write discovery as major hotspots;
action/factory bindings and owned-write region extraction are the next
overlapping regex consumers.

Action results and direct factory instances now consume that same binding list.
The AST path filters actions by their alias-resolved Solid target and indexes
factory return summaries by the initializer target directly; the text path
remains available to adapters without binding discovery. The five-sample median
fell from 2.73 ms to 2.60 ms (4.6%), while allocation fell from about 1.34 MB
and 10,745 allocations to 1.29 MB and 10,404 allocations.

The subsequent profile no longer showed action or factory discovery among the
leading consumers. It did show the identical owned-write regions being assembled
several times during one build, so those derived regions are now cached beside
the per-build call and binding facts. Together, the binding reuse and region
cache bring the five-sample median to about 2.56 ms, roughly 6.4% below the
post-binding baseline, and reduce allocation count further to about 10,014.

Async reactive-read discovery was the last high-cost consumer still scanning
every source once for each of `createMemo`, `createSignal`, `createStore`, and
`createProjection`. It now selects those bindings from the shared AST-derived
list and uses the parser-provided first-argument range for async-computation
classification. The compatibility path retains the four text matchers.

That change reduced the five-sample median from about 2.56 ms to 1.99 ms
(22.1%), allocation from about 1.29 MB to 1.24 MB, and allocation count from
about 10,014 to 9,472. In the fresh ten-second profile, `addAsyncReads` no
longer appears among the leading consumers. Parsed function-structure discovery
(`declaredFunctions`, about 9.6% cumulative CPU) is now the clearest remaining
structural migration candidate; checker type rendering and TypeScript AST walks
are also increasingly prominent as text discovery recedes.

Named function discovery then moved behind another optional parser capability.
The TypeScript-Go adapter reports exact name, block body, and parameter ranges
for function declarations and direct identifier-bound arrow functions, together
with declaration/arrow, export, and async flags. Reactive IR still determines
rendering status and returned closures, and adapters without the capability keep
the existing text implementation.

The five-sample incremental median fell from about 1.99 ms to 1.75 ms (12.1%).
Allocation was nearly flat at about 1.23 MB, with count moving from roughly
9,472 to 9,455. The ten-second profile measures `SourceFunctions` at under 1%
cumulative CPU, replacing the former `declaredFunctions` hotspot of about 9.6%.
At this point repeated path normalization, checker type rendering, AST traversal,
and the smaller remaining regex families dominate rather than one large
structural text scan.

Three follow-up micro-optimizations were measured and rejected: a conservative
fast path around `filepath.Clean`, caches for the small tracked/read execution
region lists, and a combined retained syntax index for calls, bindings, and
functions. Each increased median latency (to roughly 2.12 ms, 1.81 ms, and
1.81 ms respectively), so none remains in the implementation.

Reactive-read-after-await analysis now consumes the already cached parser call
facts inside async computation bodies instead of running its own call regex.
Type descriptions are also cached by resolved target across both typed-accessor
passes for one build. Against a re-established five-sample baseline of about
1.78 ms, median latency moved modestly to about 1.75 ms (roughly 2%), while
allocation improved from about 1.23 MB and 9,454 allocations to 1.22 MB and
9,313 allocations. The allocation result is the clearer signal at this scale.

## Project-wide async summaries and editor publication

Cross-file async callback summaries initially raised the five-sample warm
snapshot median to about 2.25 ms and incremental update-plus-snapshot to about
3.36 ms. Profiling identified repeated summary searches, path normalization,
and checker type-string rendering in async return classification.

Async summaries are now normalized once and indexed by source path and symbol.
Promise-like returns use the checker awaited-type relation instead of rendering
types to strings. On the same fixture and machine, the five-sample medians are
about 1.78 ms for a warm snapshot and 2.86 ms for update plus snapshot: roughly
21% and 15% faster than the first project-wide implementation. The warm path is
back at the documented pre-project-summary level while retaining cross-file and
alias-aware analysis.

The language server separately avoids diagnostic transport amplification. It
publishes the complete workspace once after initialization, then fingerprints
the encoded diagnostic list per file and sends only changed file payloads.
