# Rust migration plan

Goal: move the analysis engine (Reactive IR construction, solving, LSP,
certification emission) from Go to Rust, fused with the Oxc-based
`jsx-compiler`, while keeping TypeScript-Go as the type checker. The end state
is one Rust binary and a small Go type-facts service.

The two criteria are **accuracy** and **performance**. Both are enforced the
same way: no phase merges without passing its measured gate. Nothing in this
plan is justified by "Rust is faster" — every move is justified by a benchmark
taken before and after, measuring the same work on both sides.

## Constraints and non-goals

- TypeScript-Go stays in Go. There is no production-complete TypeScript
  checker in Rust; `internal/typefacts/tsgo` (the only package importing the
  tsgo shims) remains Go permanently.
- Every viable topology has exactly **one** cross-language seam. The
  migration relocates the boundary from the compiler-facts seam to the
  type-facts seam; it must never add a second one.
- Accuracy is defined as: byte-identical certification snapshots
  (`schema/certification-snapshot.schema.json`) across the conformance
  fixtures, the eslint-reactivity-v2 suite, and the Solid Primitives corpus.
  A Rust engine that diverges on any fixture does not ship, regardless of
  speed.

## Topology by phase

Exactly one process layout is legal per phase. Anything not listed is out of
scope for that phase.

| Phase | Entry point | Engine (IR + solve + emit) | Type facts | Compiler facts | Cross-language boundary |
| --- | --- | --- | --- | --- | --- |
| 0–1 | Go (`solid-check`, `solid-checkd`) | Go, in-process | Go, in-process | Rust sidecar, JSON lines | Go → Rust (ExecutionMap v1) |
| 2 | Go (authoritative); Rust engine is a separate `solid-engine-diff` binary replaying recorded job directories | Both, differential | Go, in-process; recorded fact tables feed Rust | Rust: sidecar for Go, in-process for Rust engine | Go → Rust (ExecutionMap v1) + recorded job directories (offline files, not a live protocol) |
| 3 | Rust (CLI + LSP) | Rust, in-process | Go service (transport chosen by G3 benchmark: c-archive FFI or subprocess) | Rust, in-process (sidecar retired) | Rust → Go (TypeFacts protocol) |
| 4 | Rust | Rust | Go service | Rust, in-process | Rust → Go (TypeFacts protocol) |

## Versioned data model

Two schemas cross the seam over the life of this plan. Both follow the
precedent already set by the compiler-facts boundary
(`internal/compilerfacts/protocol.go`: `ProtocolVersion = 1`, requests and
responses that do not match are rejected, unknown fields are an error):

1. **ExecutionMap protocol** — already versioned. Unchanged through Phase 2;
   retired as a wire protocol in Phase 3 when compiler facts become
   in-process Rust (the schema survives as an internal type).
2. **TypeFacts fact-table schema** — introduced in Phase 1 as Go types with a
   `TypeFactsSchema` version constant, serialized deterministically (sorted
   keys, stable ordering) so snapshots of the tables themselves can be
   diffed. In Phase 2 it is additionally frozen as a language-neutral schema
   file under `schema/` (same treatment as the certification snapshot), and
   the Rust engine's deserializer is generated or hand-written against that
   file — never against the Go structs. In Phase 3 it becomes the live wire
   protocol with a mandatory handshake: the Go service reports
   `(TypeFactsSchema, engine build ID)` on startup, the Rust binary rejects
   any mismatch and exits with a distinct error code. No silent fallback, no
   best-effort parsing.

Packaging and skew: from Phase 3 on, the Go type-facts service ships inside
the same release artifact as the Rust binary and is spawned/linked by it —
there is no supported deployment of independently versioned halves. Rolling
upgrades are therefore whole-artifact swaps; the handshake exists to turn an
operator mistake (stale binary on PATH, partial install) into a clean refusal
rather than corrupt facts.

## The seam problem, stated precisely

`internal/reactiveir/build.go` queries the type-facts `Project` interface
lazily during IR construction. Reproducible static counts (at the commit that
introduced this plan):

```sh
# query call sites (References, DescribeTypeAt, discoverer calls):
grep -Ec 'facts\.(References|DescribeTypeAt)|Discover' internal/reactiveir/build.go   # 32
# total references to the typefacts package in the build:
grep -c 'facts\.\|typefacts\.' internal/reactiveir/build.go                            # 326
```

Static call sites are not runtime round-trips: many sit inside per-symbol and
per-reference loops, so the runtime query count per edit is unknown until
measured. That measurement — not the static count — is what Phase 0 produces
and what gate G1 is written against. In-process these queries are cheap;
across a language boundary each one is a round-trip. The migration is
therefore sequenced so the seam is reshaped and measured before any Rust is
written.

## Benchmark infrastructure (prerequisite for every phase)

### Existing measurements

`make benchmark` runs five samples of the four lifecycle benchmarks in
`internal/engine/native_benchmark_test.go` plus
`BenchmarkProjectReferenceLookups` (`internal/typefacts`), per
`docs/performance.md`. `make profile` captures CPU and allocation profiles of
the warm editor-critical path.

### Additions required before Phase 0 concludes

1. **Seam traffic counters.** Instrument the `typefacts.Project`
   implementation to count, per incremental update: queries by kind, distinct
   symbols touched, and the serialized byte size the responses would have
   (encoded with the same codec proposed for the boundary). Instrument
   `internal/compilerfacts` to record ExecutionMap bytes per edit. Report
   both from a `-tags seamtrace` build.
2. **Stage attribution.** Split `BenchmarkNativeEngineIncrementalUpdate` into
   named stages: tsgo overlay update, type-facts queries, IR build, solve,
   snapshot emission. Record each stage's share of wall time.
3. **LSP-critical metrics.** Beyond throughput: **cancellation latency**
   (time from a superseding edit to the in-flight snapshot actually
   aborting — measured by injecting an edit mid-solve and timing context
   cancellation to goroutine/thread quiescence), **steady-state memory**
   (RSS after 1,000 alternating edit/snapshot cycles, not just peak),
   and **startup cost** (process spawn to first snapshot, measured for the
   current single-binary layout so Phase 3's two-runtime layout has a
   baseline to be compared against).
4. **Boundary cost model inputs.** The G0 decision needs a measured
   estimate of what the relocated boundary would cost, before any Rust
   engine exists: microbenchmark encode/decode of representative fact
   tables in the candidate codec, and per-round overhead of both candidate
   transports (cgo FFI call into a stub Go `c-archive`; subprocess
   round-trip over length-prefixed frames to a stub echo service). These
   stubs are throwaway measurement rigs, not product code.
5. **A deterministic representative corpus.** The 38-source fixture is too
   small. The benchmark corpus must be reproducible byte-for-byte: either a
   pinned real Solid 2 application at a recorded revision, or a generated
   project produced by a checked-in generator with a fixed seed. Either way
   the corpus manifest (file list + content hashes) is committed, and every
   benchmark report names the manifest hash. If generated, the generator
   must vary identifiers, module graphs, and body shapes rather than
   duplicating templates — duplicated content exaggerates cache locality and
   interning wins and is not representative.
6. **Comparison protocol.** All before/after claims use
   `go test -bench ... -count=10` compared with `benchstat` (criterion with
   matching sample counts on the Rust side); record commit, Go/Rust
   versions, OS, CPU, power mode, and corpus manifest hash. Latency gates
   compare p50 and p99, not just means.

### Incremental benchmark definition

"Incremental update p50/p99" is meaningless without a fixed workload, and
Phase 3 compares it across different process topologies, so the workload is
defined once and replayed identically in every phase:

- **Edit script.** A checked-in, deterministic script of ≥ 200 edit
  operations against the corpus (JSON: file, range, replacement text),
  produced once and committed next to the corpus manifest. It covers four
  edit classes in fixed proportion, reported separately because their stage
  shares differ: *leaf edit* (function body change in a file nothing
  imports), *hub edit* (change in a file with many dependents), *JSX-only
  edit* (markup change that alters compiler facts but not types), and
  *signature edit* (exported type change that invalidates dependent type
  facts).
- **Measurement endpoints.** One sample = the time from the edit being
  delivered to the engine (overlay apply call in Go; LSP `didChange`
  receipt in the Phase 3 Rust server) to a complete certification snapshot
  consistent with that edit being available. The endpoints are the same
  events in every topology; transport between editor and engine is inside
  the timed region on both sides or on neither.
- **Aggregation.** p50/p99 are computed across the script's samples after a
  fixed warm-up prefix (first 20 edits discarded), per edit class and
  overall. Cancellation cases (superseding edits) are a separate scripted
  scenario and never mixed into these distributions.

### Benchmark parity rule (applies to every cross-language comparison)

A Go-vs-Rust number is admissible only if both sides measure the same work
with the same boundaries: identical inputs loaded from the same serialized
artifacts (fact tables, ExecutionMaps, source manifest), identical required
output (a certification snapshot that byte-matches the oracle), timing
started after inputs are resident and stopped after the output is
serialized. Neither side may amortize work outside the timed region that the
other performs inside it (parsing, checking, warm-up iterations are excluded
identically). Every gate below that compares languages cites this rule.

## Phases and gates

### Phase 0 — Measure (no code moves)

Build the instrumentation above and record the baseline table:

| Metric | Fixture (38 files) | Large corpus |
| --- | --- | --- |
| Cold snapshot | | |
| Warm snapshot | | |
| Incremental update p50 / p99 | | |
| Stage shares (tsgo / typefacts / IR / solve / emit) | | |
| Type-facts traffic per edit (runtime queries, bytes) | | |
| Compiler-facts traffic per edit (bytes) | | |
| Cancellation latency p50 / p99 | | |
| Steady-state RSS after 1,000 edits | | |
| Startup to first snapshot | | |
| Peak RSS | | |

**Gate G0 (go / no-go for the whole migration):** two conditions, both
required — feasibility does not substitute for payoff, or vice versa.

1. **Feasibility.** Measured type-facts traffic per edit is boundable:
   runtime queries collapse into ≤ 10 batched rounds with payloads whose
   projected boundary cost (bytes × measured codec throughput + rounds ×
   measured transport overhead, from the boundary cost model) is < 10% of
   current incremental p99. If the seam cannot be made cheap, no amount of
   engine speedup justifies relocating the boundary onto it.
2. **Payoff.** A projected end-to-end p99 improvement of **≥ 15%** on the
   large corpus, computed from measured inputs only:
   `(movable-stage share) × (1 − 1/S) + (measured ExecutionMap transport
   share) − (projected boundary cost share)`, where the assumed Rust
   speedup S is capped at 3× until G2 replaces it with a measurement. Every
   term must come from the Phase 0 table — no unmeasured estimates.

If either condition fails, stop: optimize Go (allocation, parallelism
across files) instead, and re-run Phase 0 afterwards. Phase 1 (seam
batching) may still proceed on its own merits, since G1 requires it to be
performance-neutral or better in pure Go.

### Phase 1 — Reshape the seam, still all Go

Convert `reactiveir` from lazy per-symbol queries to batched, phase-shaped
requests: IR build emits one demand list per round (symbols needing
references, locations needing type descriptors), `typefacts` answers with
materialized fact tables typed against `TypeFactsSchema` v1. The `Project`
interface gains a batch API; the lazy one is deleted when no caller remains.

Deliverables: versioned batch request/response types with deterministic
serialization (they become the Phase 3 wire protocol), `reactiveir`
consuming fact tables only, seam counters reporting rounds per update.

**Gate G1:** zero certification-snapshot diffs on all suites and the corpus
(`make verify`, `make conformance`, `make corpus`); benchstat shows no
lifecycle benchmark regressed > 5% (improvement is expected from batching);
measured round-trips per incremental update ≤ 10; cancellation latency and
steady-state RSS within 10% of the Phase 0 baseline.

This phase is valuable even if the migration stops here.

### Phase 2 — Port solver and IR build to Rust, run differentially

Freeze `TypeFactsSchema` as a language-neutral schema file under `schema/`.
Implement `reactiveir` + `internal/solver` rule families as crates in the
`jsx-compiler` workspace, consuming ExecutionMaps in-process and fact tables
deserialized from the frozen schema.

**Executable topology.** The Rust engine ships as a `solid-engine-diff`
binary in the `jsx-compiler` workspace. `solid-check --engine=both` runs the
Go engine normally while recording a **job directory**: source manifest,
serialized fact tables (every batch response), ExecutionMaps, and the Go
snapshot. It then invokes `solid-engine-diff <job-dir>`, which replays the
recorded inputs with no live seam to Go, emits its snapshot, and the driver
diffs the two. Job directories are deterministic and self-contained, so any
CI diff is committed as a replayable repro. This same replay path is the
input side of the G2 performance benchmark, satisfying the parity rule by
construction.

**Backport rule.** The regex-based source scans in `build.go` are replaced
by Oxc AST queries; any fixture whose outcome *improves* is recorded as an
accuracy fix, not a diff to suppress, and back-ported to the Go oracle so
the engines re-converge. Because a backport changes the oracle mid-
migration, each one must land as its own reviewed change that passes the
full Go correctness suites, and triggers a re-run of the Phase 1 benchmark
baseline before differential comparisons resume — otherwise later Go-vs-Rust
numbers would be measured against a baseline the oracle no longer matches.
The G2 accuracy clock does not restart on a backport, but the differential
diff must be empty against the updated oracle from that point on.

The differential harness runs in CI for every fixture suite and the corpus.

**Gate G2 (accuracy):** differential diff empty across all suites for two
consecutive weeks of development.
**Gate G2 (performance):** IR+solve stage compared under the benchmark
parity rule, both engines replaying the same job directories to
byte-identical snapshot output. Rust must win ≥ 1.5×; below that the
boundary cost in Phase 3 will likely eat the gain, so pause and re-evaluate
at G3 with a prototype. The measured speedup replaces the capped S in the
G0 payoff model; if the recomputed projection falls under 15%, G0 is
re-decided before Phase 3 starts.

### Phase 3 — Flip the boundary

The Rust binary becomes the entry point (CLI + LSP). Go shrinks to a
type-facts service speaking the Phase 1 protocol behind the version
handshake described above. Benchmark **both** transports before choosing:

- Go `c-archive` linked into the Rust binary, bytes over cgo FFI;
- Go subprocess over length-prefixed binary frames (mirror of today's
  `compilerfacts` client, direction reversed).

Cancellation must propagate across the boundary: an aborted request on the
Rust side must cancel the corresponding tsgo work, and cancellation latency
is measured end-to-end through the seam.

**Gate G3:** end-to-end incremental p50 and p99 on the large corpus beat the
Phase 1 Go baseline; cold snapshot, startup-to-first-snapshot, cancellation
latency, steady-state RSS, and peak RSS all within 10% of baseline. If the
FFI/IPC tax loses to in-process Go, hold here — Phase 2's Rust engine still
serves as a differential oracle and the fused compiler facts still removed
one JSON boundary.

### Phase 4 — Retire the Go engine, after a stabilization window

The flip (Phase 3) and the deletion are separate releases:

1. Tag the last release in which the Go engine is buildable and
   authoritative-capable as the designated rollback release, and record the
   tag in this document.
2. Keep the Go engine and the `--engine=both` differential harness in-tree
   and running in CI for a stabilization window of **one release cycle or
   four weeks, whichever is longer**, with the Rust engine authoritative.
   Any differential regression during the window reverts authority to Go
   via the rollback tag.
3. Only after a clean window: delete Go `reactiveir`, `solver`, `engine`,
   and `lsp`; keep `typefacts` + tsgo adapter as the service. Move
   conformance harnesses to drive the Rust binary. Update
   `docs/monorepo.md`, `CONTRIBUTING.md`, and CI. Re-run the full Phase 0
   table and commit it to `docs/performance.md` as the new baseline.

**Gate G4:** stabilization window completed with zero differential
regressions; `make verify` green with the Go engine gone; benchmark table
shows the migration's net effect honestly, including any metric that got
worse.

## Rollback rules

Each phase lands behind the previous engine until its gate passes; the Go
engine is authoritative until G3, and remains one tagged release away until
G4's window closes. A gate failure means the phase does not merge — there is
no "ship now, optimize later" path, because the only two criteria are
accuracy and performance and both are gate-defined.
