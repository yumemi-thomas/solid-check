# Rust migration plan

Goal: move the analysis engine (Reactive IR construction, solving, LSP,
certification emission) from Go to Rust, fused with the Oxc-based
`jsx-compiler`, while keeping TypeScript-Go as the type checker. The end state
is one Rust binary and a small Go type-facts service.

The two criteria are **accuracy** and **performance**. Both are enforced the
same way: no phase merges without passing its measured gate. Nothing in this
plan is justified by "Rust is faster" — every move is justified by a benchmark
taken before and after.

## Constraints and non-goals

- TypeScript-Go stays in Go. There is no production-complete TypeScript
  checker in Rust; `internal/typefacts/tsgo` (the only package importing the
  tsgo shims) remains Go permanently.
- Every viable topology has exactly **one** cross-language seam. Today the
  compiler-facts seam crosses a process boundary
  (`internal/compilerfacts/client.go`, JSON lines) and the type-facts seam is
  in-process. The migration *relocates* the boundary to the type-facts seam;
  it must never add a second one.
- Accuracy is defined as: byte-identical certification snapshots
  (`schema/certification-snapshot.schema.json`) across the conformance
  fixtures, the eslint-reactivity-v2 suite, and the Solid Primitives corpus.
  A Rust engine that diverges on any fixture does not ship, regardless of
  speed.

## Why the seam shape is the whole problem

`internal/reactiveir/build.go` currently makes lazy, fine-grained calls into
the type-facts `Project` interface — `References()`, `DescribeTypeAt()`, and
the discoverer interfaces — at ~219 call sites interleaved through IR
construction. In-process these are cheap. Across a language boundary each one
is a round-trip. The migration is therefore sequenced so that the seam is
reshaped and measured **before** any Rust is written.

## Benchmark infrastructure (prerequisite for every phase)

### Existing measurements

`make benchmark` runs five samples of the four lifecycle benchmarks in
`internal/engine/native_benchmark_test.go` plus
`BenchmarkProjectReferenceLookups` (`internal/typefacts`), per
`docs/performance.md`. `make profile` captures CPU and allocation profiles of
the warm editor-critical path.

### Additions required before Phase 0 concludes

1. **Seam traffic counters.** Instrument the `typefacts.Project`
   implementation to count, per incremental update: queries by kind,
   distinct symbols touched, and the serialized byte size the responses
   would have (encode with the same codec proposed for the boundary).
   Instrument `internal/compilerfacts` to record ExecutionMap bytes per
   edit. Report both from a `-tags seamtrace` build.
2. **Stage attribution.** Split `BenchmarkNativeEngineIncrementalUpdate`
   into named stages: tsgo overlay update, type-facts queries, IR build,
   solve, snapshot emission. Record each stage's share of wall time.
3. **A representative corpus.** The 38-source fixture is too small to
   expose scaling behavior. Add a benchmark project of ≥500 source files
   (generated from the fixture patterns is acceptable; a pinned real
   Solid 2 application is better) and run the lifecycle benchmarks
   against it.
4. **Comparison protocol.** All before/after claims use
   `go test -bench ... -count=10` compared with `benchstat`; record
   commit, Go/Rust versions, OS, CPU, power mode, and fixture revision.
   Latency-sensitive gates compare p50 and p99, not just means — GC tail
   behavior is one of the claimed Rust benefits and must be visible in
   the data.

## Phases and gates

### Phase 0 — Measure (no code moves)

Build the instrumentation above and record the baseline table:

| Metric | Fixture (38 files) | Large corpus |
| --- | --- | --- |
| Cold snapshot | | |
| Warm snapshot | | |
| Incremental update p50 / p99 | | |
| Stage shares (tsgo / typefacts / IR / solve / emit) | | |
| Type-facts traffic per edit (queries, bytes) | | |
| Compiler-facts traffic per edit (bytes) | | |
| Peak RSS | | |

**Gate G0 (go / no-go for the whole migration):** on the large corpus,
the stages that would move to Rust (IR build + solve + emission) account
for **≥ 25%** of incremental p99 latency, or type-facts traffic per edit
is demonstrably boundable (queries collapse into few batched rounds with
payloads in the low hundreds of KB). If tsgo checking dominates beyond
that, stop: optimize Go (allocation, parallelism across files) instead,
and re-run Phase 0 afterwards.

### Phase 1 — Reshape the seam, still all Go

Convert `reactiveir` from lazy per-symbol queries to batched, phase-shaped
requests: IR build emits one demand list per round (symbols needing
references, locations needing type descriptors), `typefacts` answers with
materialized fact tables. The `Project` interface gains a batch API; the
lazy one is deleted when no caller remains.

Deliverables: batch request/response types (serializable — they become the
wire protocol in Phase 3), `reactiveir` consuming fact tables only, seam
counters showing round count.

**Gate G1:** zero certification-snapshot diffs on all suites and the
corpus (`make verify`, `make conformance`, `make corpus`); benchstat shows
no lifecycle benchmark regressed > 5% (an improvement is expected from
batching); round-trips per incremental update ≤ 10.

This phase is valuable even if the migration stops here.

### Phase 2 — Port solver and IR build to Rust, run differentially

Implement `reactiveir` + `internal/solver` rule families as crates in the
`jsx-compiler` workspace, consuming ExecutionMaps in-process (no sidecar
JSON) and type-fact tables via the Phase 1 protocol. The regex-based
source scans in `build.go` are replaced by Oxc AST queries — record any
fixture whose outcome *improves* separately; those are accuracy fixes,
not diffs to suppress.

Build a differential harness: `solid-check --engine=both` runs Go and
Rust engines on the same inputs and diffs snapshots. Wire it into CI for
every fixture suite and the corpus.

**Gate G2 (accuracy):** differential diff empty across all suites for
two consecutive weeks of development.
**Gate G2 (performance):** isolated IR+solve stage benchmarked in Rust
(criterion) vs Go (benchstat baseline) on identical fact-table inputs;
Rust must win ≥ 1.5× — below that the boundary cost in Phase 3 will
likely eat the gain, so pause and re-evaluate at G3 with a prototype.

### Phase 3 — Flip the boundary

The Rust binary becomes the entry point (CLI + LSP). Go shrinks to a
type-facts service exposing the Phase 1 batch protocol. Benchmark **both**
transports before choosing:

- Go `c-archive` linked into the Rust binary, bytes over cgo FFI;
- Go subprocess over length-prefixed binary frames (mirror of today's
  `compilerfacts` client, direction reversed).

**Gate G3:** end-to-end incremental p50 and p99 on the large corpus beat
the Phase 1 Go baseline; cold snapshot and peak RSS within 10% of
baseline. If the FFI/IPC tax loses to in-process Go, hold here — Phase 2's
Rust engine still serves as a differential oracle and the fused compiler
facts still removed one JSON boundary.

### Phase 4 — Retire the Go engine

Delete Go `reactiveir`, `solver`, `engine`, and `lsp`; keep `typefacts` +
tsgo adapter as the service. Move conformance harnesses to drive the Rust
binary. Update `docs/monorepo.md` seam documentation, `CONTRIBUTING.md`,
and CI. Re-run the full Phase 0 table and commit it to
`docs/performance.md` as the new baseline.

**Gate G4:** `make verify` green with the Go engine gone; no open
differential regressions; benchmark table shows the migration's net
effect honestly, including any metric that got worse.

## Rollback rules

Each phase lands behind the previous engine until its gate passes; the Go
engine is authoritative until G3. A gate failure means the phase does not
merge — there is no "ship now, optimize later" path, because the only two
criteria are accuracy and performance and both are gate-defined.
