# Rust migration plan

Goal: move the analysis engine (Reactive IR construction, solving, LSP,
certification emission) from Go to Rust, fused with the Oxc-based
`jsx-compiler`, while keeping TypeScript-Go as the type checker. The end state
is one Rust binary and a small Go type-facts service.

The two criteria are **accuracy** and **performance**. Both are enforced the
same way: no phase merges without passing its measured gate. Nothing in this
plan is justified by "Rust is faster" — every move is justified by a
measurement taken before and after, of the same work on both sides, analyzed
with the same method.

## Decision order

Phase numbering is not decision order. Phase 1 (batching the type-facts seam)
is an independently justified Go improvement gated only on its own neutrality
(G1). The migration's investment decision (**G0**) is taken **after** Phase 1,
on the implemented batch seam — not on a projection of a seam that does not
exist yet. Phase 0 produces the measurement infrastructure and an *advisory*
projection that can kill the idea early, but cannot approve it.

```
Phase 0 (measure)  →  Phase 1 (batch seam, G1)  →  G0 investment decision
                                                        │ pass
Phase 2 (differential Rust port, G2)  →  Phase 3 (flip, G3)  →  Phase 4 (retire, G4)
```

## Constraints and non-goals

- TypeScript-Go stays in Go. There is no production-complete TypeScript
  checker in Rust; `internal/typefacts/tsgo` (the only package importing the
  tsgo shims) remains Go permanently.
- **One live seam.** Every production topology has exactly one
  cross-language boundary on the live request path. The migration relocates
  it from the compiler-facts seam to the type-facts seam and must never add
  a second one. Deterministic offline artifacts (recorded job directories
  replayed by a test binary) are test inputs, not a runtime seam, and are
  exempt from this rule.
- **Accuracy** is defined as all of the following, together:
  1. byte-identical certification snapshots
     (`schema/certification-snapshot.schema.json`) across the conformance
     fixtures, the eslint-reactivity-v2 suite, and the Solid Primitives
     corpus;
  2. CLI behavioral equivalence: golden tests for stdout/stderr, exit
     codes, and malformed-input handling;
  3. LSP protocol conformance: initialization and shutdown, document
     version handling and stale-edit rejection, diagnostic publication,
     clearing, ordering and deduplication, cancellation and
     superseding-edit behavior, position encoding and path normalization,
     and protocol error responses.
  Snapshot bytes are one required artifact, not the whole definition of
  observable equivalence. The CLI golden suite and LSP conformance suite
  are built in Phase 0–1 against the Go implementation and become
  oracle-agnostic gates from G2 on.

## Topology by phase

Exactly one process layout is legal per phase. The "live boundary" column is
what the one-seam rule governs; offline test artifacts are listed separately.

| Phase | Entry point | Engine (IR + solve + emit) | Type facts | Compiler facts | Live boundary |
| --- | --- | --- | --- | --- | --- |
| 0–1 | Go (`solid-check`, `solid-checkd`) | Go, in-process | Go, in-process | Rust sidecar, JSON lines | Go → Rust (ExecutionMap v1) |
| 2 | Go (authoritative); Rust `solid-engine-diff` replays recorded job directories offline | Both, differential | Go, in-process | Rust: sidecar for Go, in-process for the replay binary | Go → Rust (ExecutionMap v1); job directories are offline test inputs, not a seam |
| 3 | Rust (CLI + LSP) | Rust, in-process | Go service (transport chosen by G3 benchmark: c-archive FFI or subprocess) | Rust, in-process (sidecar retired) | Rust → Go (TypeFacts protocol) |
| 4 | Rust | Rust | Go service | Rust, in-process | Rust → Go (TypeFacts protocol) |

## Versioned data model

Two schemas cross the seam over the life of this plan. Both follow the
precedent set by the compiler-facts boundary
(`internal/compilerfacts/protocol.go`: `ProtocolVersion = 1`, version
mismatch rejected, unknown fields an error):

1. **ExecutionMap protocol** — already versioned. Unchanged through Phase 2;
   retired as a wire protocol in Phase 3 (the schema survives as an internal
   type).
2. **TypeFacts protocol** — introduced in Phase 1, specified before it is
   implemented (see below), frozen as a language-neutral schema file under
   `schema/` in Phase 2, and made the live wire protocol in Phase 3 behind a
   mandatory handshake: the Go service reports
   `(TypeFactsSchema, engine build ID)` on startup; the Rust binary rejects
   any mismatch with a distinct exit code. No silent fallback.

### Wire codec (decided now, used from Phase 0 on)

The boundary cost model is meaningless if it measures a representation the
product will not use, so the codec is policy, fixed before Phase 0
measurements: **deterministic CBOR (RFC 8949 §4.2 core deterministic
encoding)**, chosen for defined canonical bytes and mature Go and Rust
libraries. Canonical rules:

- integers as the shortest CBOR encoding; all counts and offsets are
  unsigned 64-bit, encode-time overflow is an error, never a wrap;
- strings are valid UTF-8; paths are exchanged as repo-relative,
  forward-slash-normalized UTF-8 strings (normalization happens on the Go
  side, once);
- maps are encoded with deterministically ordered keys per RFC 8949;
  collections that represent sets are sorted by their canonical key;
- optional fields are omitted, never null; duplicate or unknown fields are
  a decode error;
- decoder limits: maximum nesting depth, per-message size, per-collection
  length, and per-generation total size are declared constants in the
  schema file; exceeding any is a protocol error, and malformed input
  terminates the request with an error response, never a partial result;
- framing is a 4-byte little-endian length prefix, identical for the FFI
  and subprocess transports, so transport choice at G3 does not change the
  bytes.

### Batch protocol specification (required before Phase 1 merges)

A round limit is an observation, not a design. Before the Phase 1 types
land, `docs/typefacts-protocol.md` must specify, and G1 tests must cover:

- **identity:** project identity and a monotonically increasing analysis
  generation; every request and fact table carries both, and mixing
  generations is an error;
- **request keys:** deterministic, deduplicated request keys per fact kind;
  re-requesting a key within a generation returns the cached table;
- **termination:** demand discovery is a fixed point over a finite fact
  universe — each round may only request keys not yet requested this
  generation, so the round count is bounded by construction; the spec
  states the bound and the property test that enforces monotonicity;
- **round limit:** exceeding the declared round limit fails the analysis
  closed with a diagnostic naming the requesting rule — never a silent
  partial snapshot;
- **errors:** missing, stale, invalid, and backend-error facts are distinct
  response states with defined engine behavior for each;
- **invalidation:** which cached tables survive an edit (keyed by the
  affected-set computation) and which generation bump discards them;
- **cancellation:** honored at round boundaries; a cancelled generation
  never contributes cached tables to a later one;
- **limits:** the codec's size limits, restated as protocol-level
  requirements.

## The seam problem, stated precisely

`internal/reactiveir/build.go` queries the type-facts `Project` interface
lazily during IR construction. Reproducible static counts (at the commit that
introduced this plan):

```sh
grep -Ec 'facts\.(References|DescribeTypeAt)|Discover' internal/reactiveir/build.go   # 32
grep -c 'facts\.\|typefacts\.' internal/reactiveir/build.go                            # 326
```

Static call sites are not runtime round-trips: many sit inside per-symbol and
per-reference loops, so runtime query volume per edit is unknown until
measured. In-process these queries are cheap; across a language boundary each
round is a round-trip. Hence the decision order above: the seam is reshaped
(Phase 1) and measured before the investment decision (G0), and long before
any Rust engine work (Phase 2).

## Benchmark infrastructure

### Existing measurements and their limits

`make benchmark` runs the four lifecycle benchmarks in
`internal/engine/native_benchmark_test.go` plus
`BenchmarkProjectReferenceLookups`. Note their actual coverage:
`BenchmarkNativeEngineIncrementalUpdate` calls only `session.Update` — it
measures overlay application and tsgo invalidation, not IR, solving, or
emission. It is retained for exactly that isolated purpose. The
edit-to-snapshot path is `BenchmarkNativeEngineIncrementalSnapshot`
(`Update` + `Snapshot`), and that is what the staged benchmark below
instruments.

### The staged edit-to-snapshot benchmark (Phase 0 deliverable)

A dedicated benchmark replays the edit script (below) through
`Update` + `Snapshot` and records, **for every edit**, a paired timing
vector with at least these stages:

1. overlay update and tsgo invalidation;
2. compiler-facts generation and current sidecar transport;
3. type-facts demand and materialization;
4. IR construction;
5. solving;
6. certification emission and serialization.

Every payoff-model input comes from these paired per-edit vectors. Aggregate
stage shares are never used in projections (see the payoff model below).

### LSP-critical metrics

Beyond throughput: **cancellation latency** (superseding edit to in-flight
work quiescent), **steady-state memory** (RSS after 1,000 edit/snapshot
cycles), and **startup cost** (process spawn to first snapshot), each
measured for the current layout so Phase 3's two-runtime layout has a
baseline.

### Boundary cost model and its validation

The G0 feasibility term needs a measured boundary cost before any Rust
engine exists:

- microbenchmark encode/decode of representative fact tables in the
  production codec (deterministic CBOR, above);
- measure per-round overhead of both candidate transports with stub rigs
  (cgo call into a stub Go `c-archive`; subprocess round-trip over the
  production framing) — the stubs replay **realistic request/response sizes
  and round sequences recorded from the seam-trace counters**, not empty or
  uniform frames, so allocation, copying, and buffering effects are inside
  the measurement;
- report the analytic model's error (`bytes × codec throughput + rounds ×
  transport overhead`) against the stub's measured end-to-end latency; the
  projection used at G0 is the stub measurement, with the analytic model as
  a cross-check;
- at G3, publish projected vs realized boundary cost and explain the
  difference.

### Deterministic corpus

The benchmark corpus must be reproducible byte-for-byte: a pinned real
Solid 2 application at a recorded revision, or a generated project from a
checked-in generator with a fixed seed. The corpus manifest (file list +
content hashes) is committed and every report names its hash. A generated
corpus must vary identifiers, module graphs, and body shapes — duplicated
templates exaggerate cache locality and interning wins.

### Edit workload and statistics

- **Edit script.** A checked-in deterministic script of edit operations
  (JSON: file, range, replacement) in four classes, reported separately:
  *leaf edit*, *hub edit*, *JSX-only edit*, *signature edit*.
- **Sample population.** One pass of the script is nowhere near enough for
  a p99 gate. The script is repeated until **≥ 5,000 measured samples
  overall and ≥ 800 per edit class** exist; repetition order is
  counterbalanced by seeded permutation of edit-class interleaving
  (deterministic across runs), with a fixed warm-up prefix discarded per
  repetition.
- **Measurement endpoints.** One sample = edit delivered to the engine
  (overlay apply in Go; LSP `didChange` receipt in Phase 3) to a complete
  certification snapshot consistent with that edit. Same events in every
  topology; editor-to-engine transport is inside the timed region on both
  sides or neither.
- **Statistical rule.** p50/p99 and improvement percentages are reported
  with 95% bootstrap confidence intervals. A latency gate passes only if
  the point estimate meets its threshold **and** the CI half-width is
  ≤ 5 percentage points **and** the CI lower bound stays above
  (threshold − 5 points). If the CI is too wide, collect more repetitions;
  a gate is never passed on a wide interval.
- **Cancellation scenarios** are scripted separately and never mixed into
  latency distributions.

### One statistical method across languages

Go `testing.B` and Rust criterion outputs are not directly comparable.
Cross-language comparisons use one of:

- an external process-level runner timing both implementations over the
  same replayed inputs, or
- raw per-iteration timings exported from both harnesses and analyzed by a
  single tool with one warm-up and outlier policy.

Thread count, allocator, CPU/power configuration, warm-up, repetition order,
and outlier policy are pinned and recorded with every comparison.

### Timed unit and parity rule

The movable unit is **IR + solve + emit**, timed from fact tables and
ExecutionMaps resident in memory to a serialized certification snapshot.
This same unit is used in the Phase 0 stage vectors, the G0 projection, and
the G2 replay benchmark — no gate mixes units. A cross-language number is
admissible only if both sides measure this unit on identical serialized
inputs, produce byte-identical required output, and exclude identical work
(parsing, checking, warm-up) from the timed region.

## Payoff model (per-edit projection, not aggregate arithmetic)

Tail latency is not additive: each stage's worst observation can occur on a
different edit, so dividing aggregate stage p99s produces invalid shares.
The projection therefore transforms **each observed edit's** stage vector:

```
for each edit e in the sample population:
    projected(e) = observed_end_to_end(e)
                 − movable(e) × (1 − 1/S)        # movable = IR+solve+emit stages of e
                 − sidecar_transport(e)           # measured ExecutionMap transport for e
                 + boundary_cost(e)               # stub-measured cost for e's recorded
                                                  #   rounds and bytes
```

p50/p99 are then recomputed from the transformed per-edit distribution and
compared against the observed distribution, with the statistical rule above.
The identical per-edit method — with measured values replacing S and
`boundary_cost` — is used when comparing the G0 projection to the G3
realization, so projection and validation are the same computation on
different inputs.

## Phases and gates

### Phase 0 — Measurement infrastructure (no engine changes)

Deliverables: seam-trace counters (`-tags seamtrace`) recording per-edit
type-facts queries by kind, distinct symbols, and serialized response bytes
in the production codec, plus ExecutionMap bytes; the staged edit-to-snapshot
benchmark; LSP-critical metrics; codec and transport stub measurements with
model-error report; the corpus and edit script; the CLI golden and LSP
conformance suites recorded against the Go implementation.

Output: the baseline table (per edit class and overall) —

| Metric | Fixture (38 files) | Large corpus |
| --- | --- | --- |
| Cold snapshot | | |
| Warm snapshot | | |
| Incremental p50 / p99 (95% CI), per edit class and overall | | |
| Per-edit stage vectors (six stages, archived raw) | | |
| Type-facts traffic per edit (runtime queries, bytes), per class | | |
| Compiler-facts traffic per edit (bytes) | | |
| Cancellation latency p50 / p99 | | |
| Steady-state RSS after 1,000 edits | | |
| Startup to first snapshot | | |
| Peak RSS | | |
| Boundary model error (stub-measured vs analytic) | | |

**Advisory projection (not a gate):** run the payoff model on the raw
(pre-batching) demand. This can only *stop* work early — if the movable unit
is a trivial share of every edit class even at S = 3×, the migration is dead
and Phase 1 proceeds, if at all, purely as a Go improvement. It cannot
approve the migration; that requires the implemented seam.

### Phase 1 — Batch the seam, still all Go (independently justified)

Write `docs/typefacts-protocol.md` to the specification requirements above.
Convert `reactiveir` from lazy per-symbol queries to batched demand rounds:
IR build emits one demand list per round, `typefacts` answers with
materialized fact tables typed against `TypeFactsSchema` v1 with
deterministic serialization. The lazy `Project` API is deleted when no
caller remains.

**Gate G1:** zero certification-snapshot diffs on all suites and the corpus
(`make verify`, `make conformance`, `make corpus`); CLI golden and LSP
conformance suites pass unchanged; protocol property tests (request-key
monotonicity, generation isolation, round-limit fail-closed, cancellation at
round boundaries) pass; no lifecycle benchmark regressed beyond the
statistical rule's noise bound; measured rounds per edit ≤ the limit derived
in the protocol spec, per edit class; cancellation latency and steady-state
RSS within 10% of the Phase 0 baseline.

### Gate G0 — Investment decision (after G1, on the implemented seam)

Two conditions, both required, both computed from post-Phase 1
measurements:

1. **Feasibility.** Per-edit-class boundary cost projection (stub-measured
   transport and codec on each edit's recorded rounds and bytes) is < 10%
   of that class's observed p99. A class that fails marks the seam
   unaffordable for that workload; signature edits are expected to be the
   binding class.
2. **Payoff.** The per-edit projection yields a **≥ 15%** end-to-end p99
   improvement on the large corpus at the **conservative scenario
   S = 1.5×**, passing the statistical rule. The report publishes the
   sensitivity table at S = 1.5× / 2× / 3×; the 2× and 3× rows are upside
   context and never approve the investment.

If either fails, stop: Phase 1's batching is kept (it passed G1 on its own
merits), and further Go optimization proceeds instead. Re-running G0 later
requires re-running the Phase 0 measurements it consumes.

### Phase 2 — Port solver and IR build to Rust, run differentially

Freeze `TypeFactsSchema` as a language-neutral schema file under `schema/`.
Implement `reactiveir` + `internal/solver` rule families as crates in the
`jsx-compiler` workspace.

**Executable topology.** The Rust engine ships as a `solid-engine-diff`
binary. `solid-check --engine=both` runs the Go engine normally while
recording a **job directory** containing: the source manifest, the
**canonical fact universe** for the affected generation (demand-independent
— every fact the protocol could serve for the affected set, not just what
Go requested), the Go engine's **normalized demand trace**, ExecutionMaps,
and the Go snapshot. `solid-engine-diff <job-dir>` replays with no live seam
to Go: the Rust engine selects facts from the universe by its own demand
algorithm. Two comparison channels result:

- **snapshot diff** — gating: must be empty;
- **demand-trace diff** — reported, not snapshot-gating: Oxc-based
  discovery legitimately differs from the Go regex scans, but every
  divergence must be triaged and classified (expected improvement vs bug)
  before G2 closes.

**CI artifacts.** A failing differential run uploads its job directory as a
content-addressed CI artifact, printing the hash and replay command;
retention long enough for investigation. Developers minimize interesting
failures into committed regression fixtures; CI never commits job
directories itself.

**Backport rule.** Fixtures whose outcomes *improve* under Oxc AST queries
are accuracy fixes, back-ported to the Go oracle as individually reviewed
changes passing the full Go suites. Each semantic backport triggers a
re-run of the Phase 1 benchmark baseline **and restarts the G2 accuracy
window** — evidence accumulated against the previous oracle is not evidence
about the new one. Documentation and test-suite additions without
expected-output changes restart nothing.

**Gate G2 (accuracy):** the differential diff is empty and the CLI golden
and LSP conformance suites pass against both engines, sustained for a
two-week window (restarted on semantic oracle changes) **and** meeting
minimum evidence counts: differential runs over ≥ 25 distinct commits, the
complete pinned fixture/corpus matrix on every qualifying run, seeded
randomized edit-sequence differentials, protocol decoder fuzzing with no
crashes or hangs, and property tests over fact tables, IR, and snapshots.
Elapsed time is supplementary; the evidence counts are the requirement.

**Gate G2 (performance):** the timed unit (IR + solve + emit), both engines
replaying the same job directories to byte-identical snapshots, analyzed
under the single-statistical-method rule. Rust must win ≥ 1.5× per edit
class on the per-edit paired samples. The measured per-edit speedups replace
S in the payoff model; if the recomputed projection falls under 15%, G0 is
re-decided before Phase 3 starts. (The 1.5× floor and G0's conservative
scenario are deliberately the same number: an engine that cannot beat the
scenario the investment was approved at has already invalidated the case.)

### Phase 3 — Flip the boundary

The Rust binary becomes the entry point (CLI + LSP). Go shrinks to the
type-facts service speaking the Phase 1 protocol behind the version
handshake. Benchmark both transports (c-archive FFI; subprocess with the
production framing) before choosing. Cancellation propagates across the
boundary and is measured end-to-end.

**Gate G3 (payoff validation):** realized end-to-end p99 improvement on the
large corpus, computed with the same per-edit method and statistical rule as
G0, is **≥ 15%** against the current Phase 1 Go baseline. Merely beating
the baseline does not ship the flip. The G3 report publishes
projected-vs-realized values per payoff-model term — movable-unit speedup,
sidecar-transport removal, boundary cost — so the model is validated or
corrected, not just the outcome. Additionally: p50 does not regress; CLI
golden and LSP conformance suites pass against the Rust entry point; cold
snapshot, startup-to-first-snapshot, cancellation latency, steady-state
RSS, and peak RSS are each within 10% of baseline **and** within the
absolute budgets recorded for them at Phase 0 (percentages alone can hide
unacceptable absolute regressions on fast metrics). If realized improvement
lands under 15%, hold here — Phase 2's Rust engine remains a differential
oracle and the fused compiler facts still removed one JSON boundary.

### Phase 4 — Retire the Go engine, after a stabilization window

The flip and the deletion are separate releases:

1. Tag the last release with a buildable, authoritative-capable Go engine
   as the rollback release; record the tag here.
2. Keep the Go engine and `--engine=both` differential CI for one release
   cycle or four weeks, whichever is longer, with Rust authoritative. Any
   differential regression reverts authority to Go via the tag.
3. After a clean window: delete Go `reactiveir`, `solver`, `engine`, and
   `lsp`; keep `typefacts` + tsgo adapter as the service. Conformance
   harnesses drive the Rust binary. Update `docs/monorepo.md`,
   `CONTRIBUTING.md`, CI. Re-run the full Phase 0 table and commit it to
   `docs/performance.md` as the new baseline.

**Gate G4:** stabilization window with zero differential regressions;
`make verify` green with the Go engine gone; final benchmark table shows
the net effect honestly, including any metric that got worse.

## Policy register

These are product policy, recorded so they are changed deliberately, not
drifted past:

- **15%** projected (G0) and realized (G3) end-to-end p99 improvement.
- **S = 1.5×** conservative scenario; equal to the G2 per-class floor by
  design. 2× / 3× are published upside scenarios only.
- **10%** per-edit-class boundary-cost ceiling (feasibility), computed per
  class, not against one aggregate.
- **10% and an absolute budget** for each secondary metric (cold latency,
  startup, cancellation, RSS); absolute budgets are set from the Phase 0
  baseline and recorded in the Phase 0 report. This knowingly accepts
  bounded secondary regressions in exchange for the p99 win.
- **Round limit** is derived from the Phase 1 protocol's termination bound
  and validated per edit class; it is a correctness property with a
  fail-closed behavior, not a tuning knob.
- **Codec: deterministic CBOR** (RFC 8949 §4.2) with the canonical rules
  above; changing it after Phase 0 measurements invalidates the boundary
  cost model and forces a re-measure.
- **Statistical rule:** 95% bootstrap CIs; a gate needs its point estimate
  past threshold, CI half-width ≤ 5 points, CI lower bound above
  threshold − 5 points; ≥ 5,000 samples overall, ≥ 800 per edit class.

## Rollback rules

Each phase lands behind the previous engine until its gate passes; the Go
engine is authoritative until G3 and remains one tagged release away until
G4's window closes. A gate failure means the phase does not merge — there is
no "ship now, optimize later" path, because the only two criteria are
accuracy and performance and both are gate-defined.
