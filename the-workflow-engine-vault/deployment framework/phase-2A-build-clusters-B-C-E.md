---
title: Phase 2A — Build Clusters B + C + E (Days 3-12)
date: 2026-05-17 (S1001982)
kind: deployment-framework-recipe
status: planning-only · HOLD-v2 active · pre-G9
phase: 2A
days: 3-12
modules: m7 · m4 · m5 · m6 · m12 · m13 · m14 · m15
loc_estimate: ~1,180 (m7 ~150 + m4 ~180 + m5 ~150 + m6 ~130 + m12 ~120 + m13 ~100 + m14 ~120 + m15 ~80)
verb_budget: record · correlate · emit · refuse (Phase A passive — v1.2 locked)
authority: Luke @ node 0.A
---

# Phase 2A — Build Clusters B + C + E (Days 3-12)

> Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]] · [[phase-1-genesis-day-0-3]]

Phase 2A is the **observation and evidence layer** — the nine modules that produce the data Phase B (iteration + proposal + dispatch) operates on. No Phase B module can be meaningful without what these modules measure. The build order is not arbitrary: m7's schema is the stable contract everything else writes to or reads from. Locking it on Day 3-4 prevents schema churn from propagating across the remaining seven modules.

This phase closes the first real feedback loop: cluster B writes to m7, cluster C reads m7, cluster E aggregates m7, and m14 emits a decay signal back to m11. When Day 12 closes, the engine can — for the first time — tell whether any workflow produced measurable habitat lift. That is the condition Phase 2B (Cluster F iterators) requires to operate with integrity.

---

## Why m7 FIRST — the hub argument

m7 is the single authoritative table all three Cluster B modules write to and all Cluster C and E modules read from. Building it last forces every other module to work against a placeholder schema — and schema changes cascade: m4's `cascade_cluster_id`, m6's `cost_tokens`, m12's render paths, m13's read path, m14's `find_by_outcome` query, and m15's `cascade_success` join all depend on column names that do not exist until m7 is written.

m7's `WorkflowRunRow` struct and `workflow_runs` DDL are a published API contract. Once locked on Day 4, every subsequent Phase 2A module is an implementor, not a negotiator.

**Lock signal:** Day 4 ends with `open_memory()` valid, `insert_run()` / `close_run()` passing all 50+ tests, DDL committed. Any schema change after Day 4 requires an explicit migration bump and a coordination note here.

---

## F2 hard-gate enforcement at build time

F2 is structural: `n < 20` returns `LiftSnapshot { lift: None, ci_half: None, ... }` — never `0.0` or an estimate. At m14's aggregation cycle gate:

```
if window.len() < MIN_SAMPLE_SIZE {
    return LiftSnapshot { lift: None, ci_half: None, n: window.len(), ... };
}
```

Every caller (m11 for sunset, m31 for selection weights) must hold current state on `None`. Phase 2B cluster F modules (m20-m22) inherit F2 transitively — m20 receiving `lift: None` must emit `NEEDS-MORE-DATA`, not propose. m14's `individually_significant` boolean is the per-workflow F2 flag m31 reads. The Day 12 integration smoke must verify that `None` propagates through the watch channel to the stubbed m31 receiver — testing F2 in m14 alone is insufficient.

---

## Aggregator pattern from habitat-nerve-center

The `habitat-nerve-center_m3_aggregator_mod.rs` boilerplate establishes the canonical thread-safe aggregator pattern for m14. The key structural choices, lifted verbatim:

- `Arc<AggregatorInner>` wrapping `parking_lot::RwLock<Option<T>>` — Clone is O(1), the inner Arc is reference-counted, all readers share the same lock.
- Read path is always clone-on-read: `self.inner.state.read().clone()` returns an owned value; the lock is not held past the function boundary.
- Guards are dropped inside brace blocks — never held across an `await` point or a function call boundary.
- Write path discriminates empty-input at the top of the function and returns `Err` before acquiring the write lock (no lock held on error path).

For m14, the three inner fields are `parking_lot::RwLock<Option<LiftSnapshot>>`, `parking_lot::RwLock<Vec<WorkflowLiftContribution>>`, and `parking_lot::RwLock<VecDeque<WorkflowRunRow>>`. The `VecDeque` carries the rolling window; the `Vec` carries the per-workflow contributions computed on the most recent cycle; the `Option<LiftSnapshot>` is the snapshot m11 and m31 read.

The aggregator exposes `snapshot_lift()` (returns `Option<LiftSnapshot>`, O(1) clone), `snapshot_contributions()` (returns `Vec<WorkflowLiftContribution>`, O(n) clone where n is the bank size, bounded), and `run_cycle()` (the 4-step aggregation: decay window → ingest new rows → compute → emit to channels). The timer in `wf-crystallise`'s main task loop calls `run_cycle()` every 5 minutes.

---

## m15 → agent-cross-talk emit convention

`PHASE-B-RESERVATION-NOTICE` files are JSONL, one event per file, written to `~/projects/shared-context/agent-cross-talk/`.

**Naming convention:**
```
PHASE-B-RESERVATION-NOTICE-{iso8601_date}-{session_id_prefix_8chars}-{event_id:05}.jsonl
```

Example: `PHASE-B-RESERVATION-NOTICE-2026-05-21-S1001999-00001.jsonl`

The `{iso8601_date}` is the calendar date of detection in UTC (`2026-05-21`). The `{session_id_prefix_8chars}` is the first 8 characters of `$CLAUDE_SESSION_ID` (or `UNKNWN00` if the env var is absent). The `{event_id:05}` is the monotonic event counter, zero-padded to 5 digits.

**Why one event per file:** `grep` and `jq` on a single-event JSONL directory is O(files); Watcher and Zen read independently at their own cadence. A multi-event file requires write coordination; single-event files make each write atomic (rename-into-place).

**Atomic write:** m15 writes to `.tmp` in the same directory then renames into place. Same-filesystem rename is atomic on Linux.

**Observer contract:** m15 writes and forgets. Silence is not a failure — Watcher or Zen haven't swept yet. If m15 fires more than 3 times in a session, Watcher may treat it as a Class-E (planning-sprawl) leading indicator and log at session end.

---

## Day-by-Day Recipe

---

### Days 3-4 — m7 `workflow_arc_record` (Cluster C HUB)

#### Inputs (must be live before m7 starts)

Phase 1 genesis must be complete: `workflow-core` crate created, `Cargo.toml` with `rusqlite`, `serde`, `thiserror`, `parking_lot` dependencies present, `src/lib.rs` stub compiling. No cluster B or E module is required — m7 bootstraps the schema that all others write to.

#### Outputs

- `workflow-core/src/cluster_c/m7_workflow_arc_record.rs` — published module
- `workflow_runs` DDL locked (schema contract)
- `WorkflowRunRow` struct exported from `workflow-core::types`
- `ClusterBObservation` enum exported from `workflow-core::types`
- `open_database()`, `open_memory()`, `insert_run()`, `merge_observation()`, `close_run()`, `find_open()`, `find_by_outcome()` — 6 public functions

#### Build steps

Day 3 is test-first (`bacon test -- --test m7_workflow_arc_record`). Write the test file before any implementation; let failing tests drive the impl order: pragma block → DDL string → `open_memory()` → `configure_connection()` → `create_all_tables()` → `insert_run()` → `close_run()` → `merge_observation()` → `find_open()` → `find_by_outcome()`.

Day 4: impl complete → 4-stage gate:

```bash
CARGO_TARGET_DIR=./target cargo check 2>&1 | tail -20 && \
cargo clippy -- -D warnings 2>&1 | tail -20 && \
cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=./target cargo test --lib --release -p workflow-core 2>&1 | tail -30
```

Verify >= 50 tests: `cargo test --lib ... 2>&1 | grep 'test result'`.

#### Boilerplate lifts

- `03-sqlite-multi-db/m06_schema.rs` — `open_database()`, `open_memory()`, `configure_connection()` (WAL + busy_timeout=5000 + foreign_keys + synchronous=NORMAL + wal_autocheckpoint=100): **90% lift, ~40 LOC**. Pragma block carried verbatim.
- `03-sqlite-multi-db/m07_causal_chain.rs` — `parse_row()` pattern, `auto_resolve_stale_typed()` TTL logic → outcome-based resolution, `find_by_label()` → `find_by_outcome()` query structure: **70% lift, ~35 LOC**. Adaptation diff: `label` → `outcome`; `resolved_session IS NULL` → `ended_at IS NULL`; table name `causal_chain` → `workflow_runs`.

#### ME v2 m1_foundation patterns referenced

- `resources.rs` docstring style — module-level doc with Layer / Dependencies / Features table. m7 doc: `//! m7_workflow_arc_record — Central SQLite hub. Layer: cluster_c. Dependencies: m06_schema, workflow-core::types.`
- `error.rs` taxonomy — `WorkflowError`: `DatabaseOpenFailed { path, reason }`, `Sqlite(String)`, `MigrationFailed(String)`, `RowNotFound(i64)`.
- DDL comment on `fitness_dimension` is load-bearing — zero-weight contract documented in schema, not just in code.

#### Tests required (50+ minimum)

Priority invariants: F9 — `insert_run()` always produces `fitness_dimension = 0.0` (convention, not CHECK constraint — intentional); schema idempotency — `open_database()` twice on same path safe; WAL pragma — `PRAGMA journal_mode` returns `wal` after `configure_connection()`; insert + close round-trip — `find_by_outcome("ok", 10)` returns the closed row; `find_open` ordering — three open runs in `started_at DESC`; `merge_observation::Cascade` — `consumer_inputs` blob round-trips cluster_id; `merge_observation::BatternStep` — step_index + duration_ms round-trip through JSONB; `merge_observation::ContextCost` — `cost_tokens` column updated (not just JSONB); `close_run` with invalid outcome returns `WorkflowError::Sqlite`; `open_memory()` usable in tests with full schema.

Additional: `ended_at` null on fresh insert, `find_by_outcome` unknown outcome → empty vec, concurrent reads safe, `find_open(limit=0)` empty, `find_open(limit=1)` exact.

#### Quality gate

4-stage: `cargo check` → `cargo clippy -- -D warnings` → `cargo clippy -- -D warnings -W clippy::pedantic` → `cargo test --lib --release`. Zero warnings at all stages. `fitness_dimension` zero-weight invariant is a test, not just a comment.

**Ember gate:** m7 has no user-facing strings. The DDL comments are internal. No Ember gate check required for m7 itself; m12 is where Ember compliance is tested.

#### Cross-cluster touch-points

- m4, m5, m6 all write to m7 (via `merge_observation` and `close_run`)
- m12 reads m7 via `find_open` and `find_by_outcome` (pure reads)
- m13 reads m7 to build `CorrelationMemory` payload before stcortex promotion
- m14 reads m7 via `find_by_outcome` for the rolling aggregation window
- CC-1 (cascade-cost coupling) lives in m7's schema — the `consumer_inputs` JSONB blob is the stable join plane

#### Failure modes + Watcher flag class

- Schema drift between test-time `open_memory()` and runtime `open_database()`: if `create_all_tables()` diverges from `configure_connection()`, WAL pragmas apply only to some connections. Watcher **Class-D** (four-surface drift) if this happens post-Day 4.
- `fitness_dimension` written non-zero without Ember gate approval: Watcher **Class-G** (substrate-frame confusion — premature fitness signal).
- `wal_autocheckpoint` omitted: write amplification under append-heavy m4/m5/m6 load, not observable until Day 10+ stress. Watcher **Class-I** (Hebbian silence — engine appears functional but substrate isn't being fed because write latency caused m13 to defer all rows).

#### Verification gate + atuin trajectory

Substrate writes: none (local SQLite only). Day 4 closes when >= 50 tests pass, DDL committed, `WorkflowRunRow`/`ClusterBObservation` re-exported, zero pedantic warnings. Atuin: `atuin search "cargo test --lib --release -p workflow-core"` and `atuin search "cargo clippy -- -D warnings -W clippy::pedantic"` — both commands log to atuin history as the proprioceptive gate record.

---

### Days 5-6 — m4 `cascade_correlator` (Cluster B)

#### Inputs (must be live before m4 starts)

m7 schema locked (Day 4 close). m4 does not write to m7 directly — it emits `CascadeCluster` values to the caller — but the `cascade_cluster_id` string format must match what m7's `ClusterBObservation::Cascade` variant expects. Coordinate the format on Day 4 end: `"cascade_cluster_{:016x}"`.

atuin history.db must be accessible at `~/.local/share/atuin/history.db` (read-only; always true in habitat). No other live service required.

#### Outputs

- `workflow-core/src/cluster_b/m4_cascade_correlator.rs`
- `CascadeClusterId(pub String)`, `AtuinStep`, `DispatchRecord`, `CascadeCluster`, `CascadeCorrelatorConfig`, `CascadeCorrelator` — types exported
- `read_atuin_since()`, `correlate()`, `assign_cluster_id()` — 3 public functions

#### Build steps

Day 5 test-first (`bacon test -- --test m4_cascade_correlator`). Write F11 compliance + gap-boundary tests first — these are the two invariants where silent incorrectness is possible. Impl order: types → FNV-1a hash function → sliding window → Kahn sort → `assign_cluster_id` → `read_atuin_since`. Run bacon between functions; don't batch impl. Day 5 gate (partial): `cargo check + clippy`. Day 6: complete `read_atuin_since` (rusqlite paginated read) + remaining tests → full 4-stage gate.

#### Boilerplate lifts

- `04-pattern-detection/m49_task_graph.rs` — Kahn's sort: `HashMap<String, Vec<usize>>` adjacency list + DAG depth computation: **50% lift, ~50 LOC**. Adapt: `TaskNode` → `AtuinStep`; `TaskEdge` → temporal-gap-inference rule; remove plan-phase FSM.
- `04-pattern-detection/m20_heat_source_hebbian.rs` — `CoActivationPair` O(n²) pairwise iteration pattern: **30% lift, ~25 LOC**. N-step generalisation (pairwise → single sub-graph identity) is the ~80 LOC fresh authorship keystone.
- FNV-1a: ~15 LOC fresh (no external crate); seed `0xcbf29ce484222325u64`, magic `0x00000100000001b3u64`.

#### ME v2 m1_foundation patterns referenced

- `signals.rs` — `observed_at_ms` captured once at `correlate()` entry; never per-step `SystemTime::now()`.
- `resources.rs` docstring — Layer / Dependencies / Features table.
- `error.rs` — `CascadeError`: `AtuinDbNotAccessible { path, reason }`, `SchemaVersionMismatch { expected, found }`, `Sqlite(#[from] rusqlite::Error)`.

#### Tests required (50+ minimum)

F11 compliance (IDs match `cascade_cluster_[0-9a-f]{16}`; no ALPHA/BETA/GAMMA/LEFT/RIGHT/TR/BR); gap boundary (29,999ms together, 30,001ms split); min_pane_count filter (single-session → zero clusters); ID stability; collision rate < 0.01% over 10k pairs; Kahn dag_depth (3-step chain → depth=2); cycle detection (logged). Additional: empty atuin DB → empty vec, max_steps split, micro-burst valid, pagination, cc-* binary detection, overlap boundary.

#### Quality gate

4-stage as above. Additional: `clippy::pedantic` will flag `#[allow(clippy::cast_precision_loss)]` on the `step_count as u64` cast — use `u64::from(step_count)` or ensure step_count is already `u64` to avoid the lint entirely.

#### Cross-cluster touch-points

- Writes to m7 via the calling sweep loop (not directly; emits `CascadeCluster` to caller which calls `m7::merge_observation(ClusterBObservation::Cascade { ... })`)
- CC-1: the `cascade_cluster_id` string becomes the join key that m12 and m14 use via m7's `consumer_inputs` blob
- CC-3: m20 (Phase 2B cascade iterator) reads cluster IDs from m7 rows; the ID format must match what m20 expects

#### Failure modes + Watcher flag class

- atuin schema version change (timestamp field migrates from nanoseconds to microseconds): silent wrong cluster boundaries. Watcher **Class-H** (atuin proprioception anomaly).
- cc-* binary renamed or new fleet binary with different prefix: `DispatchRecord` filter misses entries; cascade clustering silently degrades. Watcher **Class-I** (Hebbian silence — observation layer stops feeding m7 with cascade correlation data).
- FNV-1a collision at habitat scale: two distinct cascades receive same cluster ID. Watcher **Class-D** (four-surface drift — m14 aggregate lift is computed against the wrong cascade population).

#### Verification gate + atuin trajectory

No substrate writes (read-only atuin; in-memory only). Day 6 closes when >= 50 m4 tests pass (F11 compliance + gap boundary both included), full 4-stage quality gate clean. Atuin: `atuin search "cascade_correlator"` and `atuin search "cargo test.*m4"` as proprioceptive gate record.

---

### Days 6-7 — m5 `battern_step_record` (Cluster B)

#### Inputs (must be live before m5 starts)

m7 schema locked. m4's `AtuinStep` type exported from `workflow-core::types` (m5 re-uses this type for its input). No live service required.

#### Outputs

- `workflow-core/src/cluster_b/m5_battern_step_record.rs`
- `BatternId(pub String)`, `BatternStepObservation`, `BatternStepLabel`, `BatternRecord`, `BatternStepRecordConfig`, `BatternStepRecord` — types exported
- `observe()`, `summarise()` — 2 public functions

#### Build steps

Day 6 afternoon to Day 7. Test-first (`bacon test -- --test m5_battern_step_record`): boundary detection, timeout-close, partial flag, min_steps filter. Impl order: types → heuristic regex table (6 entries compiled once at `new()`) → `observe()` → `summarise()`. Full 4-stage gate at Day 7 end.

#### Boilerplate lifts

- m5 is predominantly new authorship. The one structural note: `BatternStepLabel` (a soft enum of 6 step states) mirrors the `TaskNodeState` FSM shape from `m49_task_graph.rs`, but no code is lifted — the pattern informs the design.
- The `regex` crate is already used in habitat hooks; no new dependency required.

The `inter_step_timeout_ms = 1_800_000` (30-minute auto-close) is a literal from the Battern protocol spec — not a magic number. Document in the config struct doc comment.

#### ME v2 m1_foundation patterns referenced

- Builder: `BatternStepRecord::new(config)`. `recorded_at_ms` captured once at `observe()` entry — entire batch carries same stamp. `observe()` returns `Vec` (infallible); I/O methods return `Result<_, BatternError>`.

#### Tests required (50+ minimum)

Priority invariants: Design→Dispatch boundary detection (`cc-dispatch` preceded by `rg` within 5s opens a new battern; solo `cc-dispatch` does not); two consecutive boundaries produce distinct BatternIds; 30-minute gap auto-closes then reopens; `is_partial = true` when no Design step; unlabelled steps preserved (3 steps with 1 labelled → 3 observations); `min_steps` boundary (exactly N included, N-1 excluded); BatternId stability (same `first_dispatch_ts_ns` → same id); `summarise.is_complete` true iff Compose step present; `total_duration_ms` = last step end − first step start; `observe([])` returns empty vec without panic.

Additional: step_index contiguity, BatternId uniqueness (<0.1% collision over 10k pairs), divergent battern (7 steps all recorded), `summarise.labelled_steps` count.

#### Quality gate

4-stage. Note: the `regex` crate produces `clippy::pedantic` warnings on `Lazy<Regex>` patterns without the `once_cell` newtype wrapper — follow the habitat convention of compiling regex in `new()` and storing in a `Vec<(Regex, BatternStepLabel)>` rather than using `lazy_static!` or `once_cell`.

#### Cross-cluster touch-points

- Emits `BatternStepObservation` and `BatternRecord` to caller; caller writes to m7 via `merge_observation(ClusterBObservation::BatternStep { ... })`
- CC-1b: m5's battern_id joins with m6's session_id via the `battern_observations` conceptual table in m7's JSONB blobs
- m21 (Phase 2B battern iterator) reads m5 step records from m7's `consumer_inputs` — the step_label format must be stable

#### Failure modes + Watcher flag class

- Design→Dispatch heuristic false positive: a manual `cc-dispatch` without a planning prefix opens a spurious battern. Watcher **Class-H** (atuin proprioception anomaly — m5 sees tool-call sequences that don't match the expected protocol shape).
- Multi-session batterns (context handoff mid-battern) silently truncated: the partial flag is set but the cross-session reconstruction is deferred. Watcher **Class-I** (Hebbian silence — battern step costs are systematically underestimated because multi-session batterns are always partial).

#### Verification gate + atuin trajectory

No substrate writes (read-only atuin; in-memory only). Day 7 closes when >= 50 m5 tests pass — boundary detection test must not be `#[ignore]` — full 4-stage quality gate clean. Atuin: `atuin search "battern_step_record"` and `atuin search "cargo test.*m5"`.

---

### Days 7-8 — m6 `context_cost_record` (Cluster B)

#### Inputs (must be live before m6 starts)

m7 schema locked. `WorkflowOutcome` enum (defined in `workflow-core::types`) available. stcortex `data/snapshots/latest.json` readable at `~/claude-code-workspace/stcortex/data/snapshots/latest.json` (fallback path; live `:3000` optional).

#### Outputs

- `workflow-core/src/cluster_b/m6_context_cost_record.rs`
- `SessionCostRecord`, `WorkflowOutcome`, `CostBand`, `ExplorationBaseline`, `ContextCostRecordConfig`, `ContextCostRecord` — types exported
- `read_session_costs()`, `record_and_update_baseline()`, `baseline_snapshot()` — 3 public functions

#### Build steps

Day 7 afternoon to Day 8. Test-first — write `ExplorationBaseline` EMA math, bootstrap gate, and Converged-exclusion tests before any implementation (`bacon test -- --test m6_context_cost_record`). Impl order: types → `ExplorationBaseline::new/update/classify` → `ContextCostRecord` with `parking_lot::Mutex<ExplorationBaseline>` → stcortex snapshot read (`serde_json` over `latest.json`; fallback to NULL costs if namespace absent). Full 4-stage gate at Day 8 end.

#### Boilerplate lifts + ME v2 patterns

- `m39_fitness_tensor.rs` — rolling-mean → N-sample EMA: **70% infrastructure lift, ~20 LOC**. Adapt: `alpha = 2 / (N + 1)` formula; bootstrap gate `n >= baseline_bootstrap_n`. `ws_inbound_writer.rs` — session-retention concept informs `DEFAULT_WINDOW_SIZE = 120` constant (reference only).
- `parking_lot::Mutex` wraps `ExplorationBaseline`. `#[must_use]` on `baseline_snapshot()`. `WorkflowOutcome::is_exploration()` predicate mirrors `TaskNodeState::is_terminal()` convention.

#### Tests required (50+ minimum)

F10 contract: EMA bootstrap gate (n=4→None, n=5→Some); EMA excludes Converged and Repeated; includes Explored/Diverged/Unknown; `cost_band None` iff `exploration_baseline None`; BelowBaseline boundary (0.79); NearBaseline at 0.8; AboveBaseline at 1.2; stcortex snapshot fallback (no panic); both sources absent → `ContextCostError`; EMA convergence at 100 sessions; concurrent `baseline_snapshot()` safe. Additional: proxy cost round-trip, NULL baseline during bootstrap, zero tool calls, n=5 boundary, degenerate window=1.

#### Quality gate

4-stage. `clippy::pedantic` will flag `#[allow(clippy::cast_precision_loss)]` on `cost as f64` — store costs as `f64` internally to avoid the cast.

#### Cross-cluster touch-points

- Emits `SessionCostRecord` to caller; caller writes to m7 via `merge_observation(ClusterBObservation::ContextCost { ... })`
- CC-1: m6's cost_tokens column in m7 is the join target for m12's cost-band histogram and m14's cost_lift computation
- F10 baseline EMA is the single defence against exploration-cost preservation collapse — test coverage of the exclusion logic is load-bearing for Phase 2B correctness

#### Failure modes + Watcher flag class

- Converged sessions accidentally included in EMA: baseline drifts toward zero exploration cost; any exploratory session looks anomalously expensive; Cluster F proposes variants that eliminate exploration. Watcher **Class-I** (Hebbian silence — substrate learns to suppress exploration, engine appears functional but produces anti-patterns).
- stcortex snapshot stale (not refreshed since last session): `read_session_costs()` returns costs from a prior session's tool calls. Watcher **Class-H** (atuin proprioception anomaly — cost signals misattributed to wrong sessions).

#### Verification gate + atuin trajectory

No substrate writes (stcortex snapshot read-only; in-memory EMA). Day 8 closes when >= 50 m6 tests pass — F10 Converged-exclusion tests and stcortex fallback test both included — full 4-stage quality gate clean. Atuin: `atuin search "context_cost_record"` and `atuin search "exploration_baseline"`.

---

### Days 8-9 — m12 `report_emitter` + m13 `stcortex_writer_narrowed` (Cluster C)

m12 and m13 are built in parallel (they do not depend on each other). m12 is pure formatting; m13 is the stcortex write path. Both read from m7.

#### Inputs (must be live before m12/m13 start)

m7 schema locked. `WorkflowRunRow` available from `workflow-core::types`. For m13 additionally: m9's `check_namespace()` predicate available (Cluster D; either stubbed or real). If m9 is not yet built, stub it as `fn check_namespace(key: &str) -> Result<(), NamespaceViolation> { if key.starts_with("workflow_trace_") { Ok(()) } else { Err(NamespaceViolation) } }`.

#### Outputs (m12)

- `workflow-core/src/cluster_c/m12_report_emitter.rs`
- `render_cost_histogram()`, `render_outcome_timeline()`, `render_cluster_cost_table()`, `render_summary_line()` — 4 pure functions, all `#[must_use]`

#### Outputs (m13)

- `workflow-core/src/cluster_c/m13_stcortex_writer_narrowed.rs`
- `CorrelationMemory`, `StcortexWriter`, `PromoteOutcome` — types exported
- `StcortexWriter::new()`, `promote_run()`, `flush_deferred()` — 3 public functions
- `~/.local/share/workflow-trace/deferred_writes.jsonl` — defer buffer path (created on first deferral)

#### Build steps

m12 and m13 built in parallel across Days 8-9.

m12: test-first on pure render functions (`bacon test -- --test m12_report_emitter`). Impl order: `render_cost_histogram` → `render_outcome_timeline` → `render_cluster_cost_table` → `render_summary_line`. No I/O; all functions take `&[WorkflowRunRow]` slices.

m13: test-first with mock ORAC probe responses (`bacon test -- --test m13_stcortex_writer_narrowed`). Impl order: namespace guard → LTP/LTD HTTP probe (mock via `cfg(test)` stub) → `promote_run` → defer JSONL path → `flush_deferred` async task.

Combined 4-stage gate at end of Day 9.

#### Boilerplate lifts (m12)

- ME v2 `metrics.rs` — `Labels` builder pattern, histogram bucket rendering, aggregation-formatting separation: **60% lift**. m12 render functions accept pre-aggregated `&[WorkflowRunRow]`; zero internal aggregation. Buckets fixed: `[0, 1k)`, `[1k, 5k)`, `[5k, 20k)`, `[20k, 50k)`, `[50k, ∞)`. Bar width capped at 20 chars. All arithmetic is integer.
- `01-cli-scaffolding/weaver.rs` — `--json` flag dual-mode output convention: reference only.

#### Boilerplate lifts (m13)

- `02-stcortex-consumer/capacity.rs` — `DbConnection::builder()` + `write_memory_then()` reducer-callback + `Ok(Ok(()))` vs `Err()` discrimination + atomic `reducer_ok/err` counters: **90% lift, ~60 LOC**. Adapt: capacity probe target → `substrate_LTP_density` ORAC blackboard key; write target → `workflow_trace_*` namespace. `CONSUMER-ONBOARDING.md` refuse-write pattern: relied on at DB layer; m13 registers `workflow-trace-m13` on construction.
- `03-sqlite-multi-db/m06_schema.rs` — `configure_connection()` pragma block for m7 read-only open: **90% lift, ~10 LOC**.

#### ME v2 m1_foundation patterns referenced (m12)

- `metrics.rs` — separation of aggregation (happens upstream in caller) from formatting (m12's only job).
- Forbidden-verb Ember gate: m12's user-facing strings must pass m10's verb scanner. Doc comment on each render function cites Ember gate: `// Ember gate: output text must not contain "recommend", "optimise", "select", "route", "dispatch", "auto".`

#### ME v2 m1_foundation patterns referenced (m13)

- `tracing::instrument` on `promote_run` and `flush_deferred` — all network I/O is instrumented.
- `signals.rs` — `promoted_under_pressure: true` tag in the stcortex memory payload (analogous to a signal-context anomaly flag).

#### Tests required m12 (50+ minimum)

Priority invariants: all four render functions return non-empty string on empty input (no panic, no division by zero); forbidden-verb scan — `assert!(!fn(runs).contains("recommend"))` (and `optimise`, `select`, `route`, `dispatch`, `auto`) for all four functions; `render_cluster_cost_table` truncates cluster ID to 6-char opaque prefix; open run (`ended_at = None`) displays `---` in cost column; histogram buckets sum to `runs.len()`; `render_outcome_timeline` ends with a summary line.

Additional: single-run, 100-run, ungrouped-row (no cascade observation) cases.

#### Tests required m13 (50+ minimum)

Priority invariants: namespace guard (`workflow_trace_` prefix required, else `Err(NamespaceViolation)`); three LTP/LTD threshold branches — ≥0.15 → `Written`, 0.05–0.15 → `WrittenUnderPressure` (row tagged `promoted_under_pressure=true`), <0.05 → `Deferred` with one JSONL line; ORAC 5xx/timeout → `Deferred` + `tracing::error!`; two deferred writes → two JSONL lines; `flush_deferred()` re-promotes on ORAC recovery and returns `Ok(count)`; F9 invariant (`CorrelationMemory.tensor` always `None`); relevance mapping (`ok`→1.0, `fail`→0.5, `abort`→0.3, `unknown`→0.1); `StcortexWriter::new()` with unreachable URI returns `Err(StcortexUnavailable)` gracefully.

#### Quality gate

4-stage at pedantic. m12: all render functions `#[must_use]`. m13: `#[tracing::instrument]` on async functions, `#[forbid(unsafe_code)]`, zero `unwrap()` outside test blocks.

**Ember gate on m12:** a test in `tests/ember_gate.rs` (the m10 module's test file, or a separate fixture) passes a sample m12 report string through a forbidden-verb scanner and asserts zero hits. This test must be in the passing set before Day 9 closes.

#### Cross-cluster touch-points

- m12 reads m7 (`find_open`, `find_by_outcome`) — pure reads
- m13 reads m7 (to build CorrelationMemory) + calls m9 (namespace guard) + probes ORAC blackboard + writes stcortex
- CC-3 (evidence-driven iteration): m12's cluster cost table is the human-readable analogue of m14's machine-readable lift metric — they answer the same question from different vantage points
- m13 → stcortex `workflow_trace_*` namespace is the substrate write for CC-5 (substrate learning loop)

#### Failure modes + Watcher flag class

- m13 defer buffer grows unboundedly (ORAC down for days): disk space on `~/.local/share/workflow-trace/` exhausted. Watcher **Class-I** (Hebbian silence — m13 cannot write to stcortex; CC-5 loop cannot close).
- m12 render functions produce non-deterministic output (e.g., HashMap iteration order in cluster table): test flakiness; hard to diagnose in CI. Watcher **Class-D** (four-surface drift — test output differs from production output).
- m13 writes to stcortex with `substrate_LTP_density < 0.05` (current habitat state: 0.043): all Phase A writes will be deferred. This is expected and correct behaviour — flag it as **informational** in Day 12 smoke, not an error.

#### Verification gate + atuin trajectory

m13 writes stcortex `workflow_trace_*` only when `substrate_LTP_density >= 0.05`; at current habitat state (0.043) all Phase A writes defer to JSONL — correct, not an error. Day 9 closes when m12 50+ tests pass (Ember gate included) and m13 50+ tests pass (all three LTP/LTD branches included), full 4-stage quality gate clean. Atuin: `atuin search "report_emitter"`, `atuin search "stcortex_writer"`, `atuin search "flush_deferred"`.

---

### Days 10-11 — m14 `evidence_aggregator` (Cluster E)

#### Inputs (must be live before m14 starts)

m7 schema locked. `WorkflowRunRow` and `WorkflowLiftContribution` available from `workflow-core::types`. The `tokio::sync::watch` channel infrastructure must be available (m14 writes to watch channels that m11 and m31 will read; in Phase 2A, m11 and m31 are not yet built — stub the watch channel send as a no-op if the receiver is not connected).

#### Outputs

- `workflow-core/src/cluster_e/m14_evidence_aggregator.rs`
- `LiftSnapshot`, `WorkflowLiftContribution`, `EvidenceAggregator`, `AggregatorConfig`, `AggregatorError` — types exported
- `EvidenceAggregator::new()`, `run_cycle()`, `snapshot_lift()`, `snapshot_contributions()` — 4 public functions
- `pub const MIN_SAMPLE_SIZE: usize = 20`

#### Build steps

Day 10 test-first (`bacon test -- --test m14_evidence_aggregator`): write F2 gate tests and Wilson CI tests before any lift formula implementation. Impl order: types → `VecDeque<WorkflowRunRow>` ring buffer → `Arc<EvidenceAggregatorInner>` with three `parking_lot::RwLock` fields → `run_cycle()` 4-step → `snapshot_lift()` clone-on-read. Day 10 partial gate: `cargo check + clippy`. Day 11: per-workflow contribution scores + m42 LTP/LTD list emit + remaining tests → full 4-stage gate.

#### Boilerplate lifts + ME v2 patterns

- `habitat-nerve-center_m3_aggregator_mod.rs` — `Arc<AggregatorInner>` + `parking_lot::RwLock<Option<T>>` + clone-on-read + `Default` impl: **70% lift, ~80 LOC**. Adapt: `HabitatState` → `LiftSnapshot`; add two additional `RwLock` fields for contributions vec and `VecDeque` window.
- `m16_hebbian_engine.rs` — 4-step cycle (decay → ingest → compute → emit): **80% structural**. Step 1 → `VecDeque::pop_front()`, Step 2 → SQL `SELECT` from m7, Step 3 → lift formula, Step 4 → watch send.
- Wilson CI fresh (~15 LOC): `ci_half = z * sqrt(p*(1-p)/n + z^2/(4*n^2)) / (1 + z^2/n)`, `z = 1.96`. Guard: `if n == 0 { return 1.0 }`. `resources.rs` docstring; `individually_significant` doc comment must call it a signal-context flag.

#### Tests required (50+ minimum)

Priority invariants: F2 boundary — 19 rows → `lift: None`, 20 rows → `lift: Some(_)`; `ci_half` always `Some` when `lift` is `Some`; `cost_lift` negative when workflow more expensive than baseline; `cascade_weight + cost_weight == 1.0` enforced at `new()`; window eviction drops oldest first (121 rows with size=120); `workflow_delta` positive when above aggregate cascade_success; `individually_significant = false` when run_count < 20 (per-workflow F2); `run_cycle()` + `snapshot_lift()` concurrent without deadlock; watch channel sends `None` at n < 20; delta bounded `clamp(-0.3, +0.3)`.

Additional: composite formula verified (`0.6 × cascade_success_rate + 0.4 × cost_lift.clamp(-1,1)` against known inputs), `window_size_zero_is_error`, `run_cycle_on_empty_db_returns_none`, timer constant defined and sane.

#### Quality gate

4-stage at pedantic. Note: `#[allow(clippy::cast_precision_loss)]` required on the `n as f64` cast in the Wilson CI formula — document why (usize→f64 cast; n is bounded at DEFAULT_WINDOW_SIZE=120 so precision loss is impossible in practice).

#### Cross-cluster touch-points

- Reads m7 (`find_by_outcome`) — the primary input to the rolling window
- Writes to m11 watch channel (lift snapshot) — m11 uses this to modulate the sunset clock
- Writes to m31 watch channel (contribution vec) — m31 uses this to adjust selection weights
- Writes to m42 (LTP/LTD lists) — m42 converts to stcortex `POST /reinforce` under `workflow_trace_*`
- CC-3 (evidence-driven iteration) — m14 is the gate; m20-m22 check `individually_significant` before proposing variants

#### Failure modes + Watcher flag class

- LTP/LTD lists sent to m42 with n < 20 workflows: early reinforcement locks in selection bias before evidence is meaningful. Watcher **Class-I** (Hebbian silence — the loop fires but with noise, not signal). Prevention: m42 checks `individually_significant` before writing any pathway.
- Wilson CI computed with Wald formula instead of Wilson formula: negative lower bounds at small n produce nonsensical CI bars. Watcher **Class-D** (four-surface drift — reported CI does not match the spec).
- `cascade_weight + cost_weight != 1.0` slips through config validation: composite lift formula is miscalibrated. Watcher **Class-D**.

#### Verification gate + atuin trajectory

No direct substrate writes — m14 feeds m42 (LTP/LTD lists) which owns stcortex/POVM paths; AP30 enforcement at m42, not m14. Day 11 closes when >= 50 m14 tests pass — F2 boundary (n=19 and n=20), Wilson CI formula, and concurrent safety tests all included — full 4-stage quality gate clean. Atuin: `atuin search "evidence_aggregator"`, `atuin search "habitat_outcome_lift"`, `atuin search "wilson_ci"`.

---

### Days 11-12 — m15 `pressure_register` (Cluster E)

#### Inputs (must be live before m15 starts)

`agent-cross-talk/` directory must exist at `~/projects/shared-context/agent-cross-talk/`. m15 creates it on first write if absent, but the shared-context directory itself must exist (always true in habitat). `serde_json` available (already a dependency).

#### Outputs

- `workflow-core/src/cluster_e/m15_pressure_register.rs`
- `PressureEvent`, `ForbiddenCategory`, `PressureSource`, `CharterSection`, `PressureRegister` — types exported
- `PressureRegister::new()`, `detect_and_emit()`, `scan_spec_patch()` — 3 public functions
- `~/projects/shared-context/agent-cross-talk/PHASE-B-RESERVATION-NOTICE-*.jsonl` — output files

#### Build steps

Day 11 afternoon to Day 12 morning. Test-first (`bacon test -- --test m15_pressure_register`): notice file created, JSONL round-trip, forbidden verb detection, 512-char truncation. Impl order: types (PressureEvent + enums) → monotonic event counter → `write_atomic()` (write to `.tmp`, `fsync`, `rename` — same-filesystem rename is atomic at the Linux VFS layer) → `detect_and_emit()` → `scan_spec_patch()`. Full 4-stage gate at Day 12 morning.

#### Boilerplate lifts + ME v2 patterns

Predominantly new authorship. `logging.rs` — `tracing::warn!` structured emission (~5 LOC/site): fields `event=forbidden_verb_pressure category={} source={} session_id={} proposed_feature={}`. `hookify.preserve-blanket-guard.local.md` — 3-step enumeration-diff-rewrite protocol informs `scan_spec_patch()`. `error.rs` — `PressureRegisterError`: `NoticeWrite { path, source }`, `DirectoryUnavailable { path }`, `Serialize(#[from] serde_json::Error)`. `detected_at` captured once at detection entry.

#### Tests required (50+ minimum)

Priority invariants: notice file created (confirmed via `read_dir`); filename matches naming convention regex; `serde_json` round-trip (`from_str` back to `PressureEvent`); `recommend` verb in function signature → `RecommendVerb`; `auto_promote` → `AutoOrSmartVerb`; `axum::Router` → `HttpServerSurface`; 600-char excerpt stored as 512-char truncated value; missing `agent-cross-talk/` → `Err(DirectoryUnavailable)` not panic; 3 events → 3 files; `tracing::warn!` with `event=forbidden_verb_pressure` fields (mock subscriber); `.tmp` absent after successful write (atomicity); `event_id` strictly monotonic across sequential calls.

Additional: `CharterSection` discriminant per verb category, schema version in JSON output, `session_id` defaults to `"UNKNWN00"` when env var absent, `serde` round-trip per `ForbiddenCategory` variant, empty excerpt handled.

#### Quality gate

4-stage at pedantic. `clippy::pedantic` will flag `std::process::Command` usage if any; m15 uses only `std::fs` for the atomic write. No network I/O.

#### Cross-cluster touch-points

- Writes to `agent-cross-talk/` (shared filesystem, no service dependency)
- Watcher ☤ reads `agent-cross-talk/` at own cadence (no confirm)
- Zen reads `agent-cross-talk/` at own cadence (no confirm)
- CC-7 (pressure-driven evolution): m15 notices are the durable signal that scope-pressure occurred; accumulation triggers spec amendment interview at Watcher's discretion
- m10 Ember gate: m15 detects pressure at the spec/cross-talk level; m10 detects pressure at the code level. They are complementary, not duplicative.

#### Failure modes + Watcher flag class

- `agent-cross-talk/` directory unavailable (shared-context not mounted): m15 logs `tracing::error!` and returns `Err(DirectoryUnavailable)` — the engine continues operating, pressure events are lost. Watcher **Class-D** (four-surface drift — pressure events not persisted to the shared surface). The `tracing::error!` is the fallback signal.
- m15 fires on false positives (e.g., a test file containing `fn recommend_` in a comment): detection heuristic too broad. Watcher **Class-H** (atuin proprioception anomaly — m15 history shows pressure events that don't correspond to real scope-pressure). Fix by requiring the forbidden verb to appear in a function signature or import, not a comment.

#### Verification gate + atuin trajectory

No substrate writes (filesystem only — `agent-cross-talk/`; no stcortex, POVM, or SQLite). Day 12 morning closes when >= 50 m15 tests pass — atomic write test, forbidden verb detection for all major categories both included — full 4-stage quality gate clean. Atuin: `atuin search "pressure_register"` and `atuin search "PHASE-B-RESERVATION-NOTICE"`.

---

### Day 12 — Phase 2A Integration Smoke

Six-check manual verification that the four module layers compose end-to-end (implemented as `cargo test --test integration_phase_2a_smoke`):

1. **B → m7** — `m4::correlate()` on mock atuin batch → `m7::merge_observation()` → row readable via `m7::find_open()`.
2. **m7 → m12** — `m12::render_cluster_cost_table()` on those rows; verify 6-char opaque prefix appears; no forbidden verbs.
3. **m13 defer** — at current LTP/LTD ≈ 0.043 (<0.05), `m13::promote_run()` returns `PromoteOutcome::Deferred`; defer buffer has exactly one line.
4. **m14 F2 boundary** — `m14::run_cycle()` with 19 rows → `snapshot_lift() = None`; add 20th row → `Some(LiftSnapshot { lift: Some(_), ci_half: Some(_), n: 20 })`.
5. **Watch channel** — stubbed m11 receiver gets `None` at n=19, `Some(...)` at n=20.
6. **m15** — `detect_and_emit()` with forbidden-verb excerpt → JSONL file appears in `agent-cross-talk/`.

```bash
CARGO_TARGET_DIR=./target cargo test --test integration_phase_2a_smoke --release 2>&1 | tail -10
sqlite3 ~/.local/share/workflow-trace/workflow_trace.db \
  "SELECT COUNT(*) FROM workflow_runs WHERE outcome IS NOT NULL;"
ls ~/projects/shared-context/agent-cross-talk/PHASE-B-RESERVATION-NOTICE-*.jsonl 2>/dev/null | wc -l
```

**Day 12 close condition:** 6/6 smoke checks pass; `cargo test --lib --release -p workflow-core` reports >= 450 tests passing.

---

## Hand-off to Phase 2B

When Day 12 closes, the state is:

| Module | Key invariant | LOC | Tests |
|---|---|---|---|
| m7 `workflow_arc_record` | Schema locked; F9 zero-weight column | ~150 | 50+ |
| m4 `cascade_correlator` | F11 opaque IDs; gap-boundary split | ~180 | 50+ |
| m5 `battern_step_record` | Soft schema; partial flag; 30-min auto-close | ~150 | 50+ |
| m6 `context_cost_record` | F10 EMA Converged-exclusion; stcortex fallback | ~130 | 50+ |
| m12 `report_emitter` | Ember-gate clean; 6-char opaque prefix render | ~120 | 50+ |
| m13 `stcortex_writer_narrowed` | LTP/LTD backpressure; deferred JSONL | ~100 | 50+ |
| m14 `evidence_aggregator` | F2 n≥20; Wilson CI; per-workflow delta | ~120 | 50+ |
| m15 `pressure_register` | JSONL one-event-per-file; atomic write | ~80 | 50+ |

**First evidence-driven feedback loop closed.** m7 has rows → m14 can produce `Some(LiftSnapshot)` at n≥20 → m11 receives the decay signal → m15 has emitted at least one notice to `agent-cross-talk/`.

**What Phase 2B needs from this state:**
- m20 `cascade_iterator` calls `m14::snapshot_contributions()` and checks `individually_significant` before proposing — F2 contract is operational.
- m21 `battern_iterator` reads `BatternStepLabel` from m7's `consumer_inputs` JSONB — `snake_case` serde format must be stable.
- m22 `cost_iterator` reads m6 EMA baseline from m7 rows.
- m23 `proposal_builder` re-enforces F2 at `build()`: `AggregatorError::InsufficientSamples` when n < 20.
- **Watcher T1 baseline:** first `LiftSnapshot` logged in Deployment Watch Journal. Class-I flag testable: if lift metric does not move across Phase 2B weeks 1-4, Cluster H (m40-m42) is decorative. Watcher flags at T+4 weeks.

---

## Standing Constraints (Phase 2A scope)

| Constraint | Source | Enforcement |
|---|---|---|
| v1.2 verb-locked (record / correlate / emit / refuse) | Genesis Prompt v1.2 | Cluster B + C modules: no recommend/select/optimise verbs in public API |
| F2 n≥20 hard gate | GOD_TIER § Gap 1 | m14 `ProposalBuilder::build()` returns `None` for n < 20 |
| F9 zero-weight fitness column | Cluster C spec | m7 DDL `DEFAULT 0.0`; m13 `tensor: None` always in Phase A |
| F10 EMA Converged-exclusion | Cluster B spec | m6 `WorkflowOutcome::is_exploration()` gates the EMA update |
| F11 opaque cascade cluster IDs | Cluster B spec | m4 FNV-1a XOR derivation destroys semantic content; m12 renders 6-char prefix only |
| AP30 namespace prefix `workflow_trace_*` | GOD_TIER § Glossary | m13 calls m9 namespace guard before every stcortex write |
| No direct stcortex reads in m14/m15 | Cluster E spec | m14 reads m7 SQLite directly; m15 writes to filesystem only |
| m15 POVM write prohibition | Cluster E spec | m42 owns POVM writes; m14/m15 emit workflow ID lists only |
| Hard refusal: no HTTP server surface | Genesis Prompt v1.2 | None of m4-m7, m12-m15 expose a listening socket |
| Watcher flag classes A-I | GOD_TIER § Watcher | Pre-positioned flags per module (see failure modes sections above) |

---

*Phase 2A deployment framework authored 2026-05-17 (S1001982) · planning-only · HOLD-v2 active · pre-G9*
*Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]] · [[phase-1-genesis-day-0-3]]*
