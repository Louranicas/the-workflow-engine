---
title: Cluster C — Central Correlation + Output (m7 / m12 / m13)
date: 2026-05-17 (S1001982)
kind: module-spec
status: planning-only · HOLD-v2 active · no code until G1-G9 clear
authority: Luke @ node 0.A
---

# Cluster C — Central Correlation + Output

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Modules Synergy Clusters and Feature Verification S1001982]] · [[Genesis Prompt v1.2 S1001982]] · canonical: `~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/module specs/cluster-C-correlation-output.md`
>
> Related: [[Cluster A — Substrate Ingest]] · [[Cluster B — Habitat Observation]] · [[Cluster D — Trust]] · [[Cluster E — Evidence + Pressure]] · [[Boilerplate Hunt S1001982]]

---

## Cluster overview

Cluster C is the **hub** of Phase A. Every observation produced by Clusters A and B converges here. m7 is the central output table; m12 is the sole path through which workflow correlation data reaches a human; m13 is the only module in the entire engine that writes to stcortex.

| Module | Job | LOC est. | Boilerplate source | Reuse |
|---|---|---|---|---|
| **m7** `workflow_arc_record` | Central SQLite output table; `workflow_runs` schema; F9 zero-weight fitness column | ~150 | `03-sqlite-multi-db/m06_schema.rs` (90%) + `m07_causal_chain.rs` (70%) | HIGH |
| **m12** `report_emitter` | Format `workflow_runs` + cluster B observations into human-readable CLI reports; PASSIVE only | ~120 | `01-cli-scaffolding/` patterns; ME v2 `metrics.rs` report style | MEDIUM |
| **m13** `stcortex_writer_narrowed` | Write correlation rows to `workflow_trace_*` namespace; Hebbian LTP/LTD headroom check before promotion | ~100 | `02-stcortex-consumer/capacity.rs` (90%); `CONSUMER-ONBOARDING.md` | HIGH |

Phase A passive-verb discipline applies to m7 and m13 without exception. m12 is exclusively a formatter — it records nothing, decides nothing, routes nothing.

---

## Cross-cluster contracts

### Inputs to Cluster C

| Source | What arrives | Contract |
|---|---|---|
| **Cluster A** (m1, m2, m3) | atuin session-id ranges; injection.db causal chain snapshots; stcortex consumer-registration confirmation | m7 row creation is blocked until m2 confirms a fresh consumer is registered in `workflow_trace_*` namespace |
| **Cluster B — m4** `cascade_correlator` | Opaque cascade cluster IDs + session-id ranges (F11 enforced — no human-meaningful labels) | m7 stores the opaque cluster ID in `consumer_inputs` as a JSON blob; never normalises it into human labels |
| **Cluster B — m5** `battern_step_record` | Battern step durations and outcomes, keyed by battern-id | m7 stores as JSONB in `consumer_inputs`; join key is battern-id present in both m5 and m7 rows |
| **Cluster B — m6** `context_cost_record` | Token cost + session outcome; exploration-rate baseline (F10) | m7 records `cost_tokens` column directly from m6; exploration-rate baseline is recorded in m6, not in m7 |

### CC-1 — Cascade-Cost Coupling

The stable join contract between m4 and m6 lives in m7's schema. m4 writes cascade cluster IDs; m6 writes cost. m7 is the join plane — neither m4 nor m6 knows about the other. Queries that correlate cascade cluster with cost are expressed as SQL against m7. This is the "join schema as stable contract" pattern from CC-1.

### Outputs from Cluster C

| Destination | What departs | Contract |
|---|---|---|
| **Cluster D — m9** `watcher_namespace_guard` | m13 calls m9's namespace-check predicate before every write; m9 returns `Ok(())` or `Err(NamespaceViolation)` | m13 MUST NOT write to stcortex if m9 rejects the namespace |
| **Cluster E — m14** `evidence_aggregator` (CC-3) | m14 reads `workflow_runs` directly from the SQLite file opened by m7 | m7's SQLite file path is a shared constant in `workflow-core`; m14 opens read-only |
| **Cluster H — m40** `nexus_event_emitter` (CC-5) | m40 reads m7 events after a run is recorded to emit `WorkflowEvent` to SYNTHEX v2 | m40 reads only rows where `outcome IS NOT NULL`; stale rows are invisible to m40 |
| **Human CLI** | m12 renders m7 rows + cluster B observations to stdout | m12 is a pure formatter with no side-effects; it holds no mutable state |

---

## m7 — `workflow_arc_record`

### Role

m7 owns the SQLite schema for `workflow_runs`. It is the single authoritative record of what the engine observed. Every cluster B module writes one or more columns into this table (via function calls on the struct m7 exposes), but only m7 knows the schema.

### Boilerplate lift

**90% from `03-sqlite-multi-db/m06_schema.rs`** — `open_database()`, `configure_connection()`, the idempotent migrate loop, WAL+pragma block, and `column_exists()` schema-discovery helper are carried verbatim. The pragma block from the boilerplate is:

```rust
conn.execute_batch(
    "PRAGMA journal_mode = WAL;
     PRAGMA busy_timeout = 5000;
     PRAGMA foreign_keys = ON;
     PRAGMA synchronous = NORMAL;
     PRAGMA wal_autocheckpoint = 100;",
)
.map_err(|e| SchemaError::Sqlite(e.to_string()))
```

This pragma block is mandatory. The `wal_autocheckpoint = 100` setting keeps write amplification bounded under the append-heavy workload m4/m5/m6 will generate.

**70% from `03-sqlite-multi-db/m07_causal_chain.rs`** — The `parse_row()` helper pattern, `auto_resolve_stale_typed()` logic for TTL-based lifecycle cleanup (adapted to outcome-based resolution rather than session-gap resolution), and the `find_unresolved()` → `ORDER BY … DESC LIMIT ?1` query pattern are lifted. The `CausalChainRow` struct is the template for `WorkflowRunRow`.

### Schema DDL

```sql
CREATE TABLE IF NOT EXISTS workflow_runs (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    -- Temporal anchors
    started_at                  TEXT NOT NULL,           -- ISO-8601 UTC
    ended_at                    TEXT,                    -- NULL until run completes
    -- Outcome enum: 'ok' | 'fail' | 'abort' | 'unknown'
    outcome                     TEXT CHECK(
                                    outcome IN ('ok', 'fail', 'abort', 'unknown')
                                ),
    -- Cluster B inputs (JSONB blobs — opaque, never normalised)
    consumer_inputs             TEXT NOT NULL DEFAULT '{}',
    -- Cost signal from m6 (token count for this run's session window)
    cost_tokens                 INTEGER,
    -- F9 MITIGATION: fitness dimension reserved at zero weight.
    -- This column exists to prevent the schema from calcifying without
    -- the fitness dimension. Weight is zero until Hebbian v3 telemetry
    -- is live and the LTP/LTD ratio is stable above 0.15. Any module
    -- reading this column MUST treat it as informational, not actionable.
    fitness_dimension           REAL NOT NULL DEFAULT 0.0,
    -- Audit
    created_at                  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at                  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Primary query path: all open runs (ended_at IS NULL), ordered by start time descending
CREATE INDEX IF NOT EXISTS idx_workflow_runs_open
    ON workflow_runs(started_at DESC)
    WHERE ended_at IS NULL;

-- Secondary query path: completed runs with known outcome, for m14 evidence aggregation
CREATE INDEX IF NOT EXISTS idx_workflow_runs_outcome
    ON workflow_runs(outcome, ended_at DESC)
    WHERE outcome IS NOT NULL;
```

### F9 zero-weight column rationale

Feature F9 is "workflow-grain fitness distortion" — the risk that a fitness signal computed at workflow granularity leaks upward and distorts the Hebbian substrate's 12D tensor before enough empirical data is collected. The mitigation is to reserve the column with a hard-coded `DEFAULT 0.0` and document the zero-weight contract in the schema DDL itself (the comment above is load-bearing, not decorative).

No module may write a non-zero value to `fitness_dimension` in Phase A without first passing the Ember gate (m10) and having the LTP/LTD ratio confirmed above the 0.15 threshold by Hebbian v3 telemetry. The column exists so that migrations adding the real fitness signal in a later phase are additive, not structural.

### Reconciling inputs from m4 / m5 / m6

The three Cluster B modules write at different observation grains:

- **m4** writes a cascade cluster observation once per detected cascade completion. The grain is one Zellij-dispatch batch.
- **m5** writes one row per Battern protocol step (6 steps per workflow invocation).
- **m6** writes one row per session window (one context window = one session = one cost figure).

All three land in `consumer_inputs` as JSON blobs merged under the run's `id`. The struct exposes a `merge_cluster_b_observation()` method that takes a `ClusterBObservation` enum:

```rust
pub enum ClusterBObservation {
    Cascade {
        cluster_id: String,   // opaque ID from m4; never human-meaningful
        session_range: (i64, i64),
    },
    BatternStep {
        battern_id: String,   // opaque ID from m5
        step_index: u8,       // 0-5
        duration_ms: u64,
        outcome: StepOutcome,
    },
    ContextCost {
        session_id: i64,
        cost_tokens: u64,
    },
}
```

Merging is a pure JSON patch operation — the existing `consumer_inputs` blob is read, the new observation is merged under its discriminant key, and the blob is written back. This means m7 does not know what `cluster_id` means; it just stores the opaque string. The intelligence lives in m4. m7 is a bucket.

### Rust struct (WorkflowRunRow)

```rust
/// A single row from the `workflow_runs` table.
///
/// All columns are present; `ended_at` and `outcome` are nullable and
/// represented as [`Option`] until the run completes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowRunRow {
    /// Primary key, auto-assigned by SQLite.
    pub id: i64,
    /// ISO-8601 UTC timestamp at which the run started being recorded.
    pub started_at: String,
    /// ISO-8601 UTC timestamp at which the run was closed, or `None` if open.
    pub ended_at: Option<String>,
    /// Run outcome: one of `ok`, `fail`, `abort`, `unknown`.
    pub outcome: Option<String>,
    /// JSON blob of cluster B observations merged into this run.
    pub consumer_inputs: String,
    /// Token cost for the session window associated with this run.
    pub cost_tokens: Option<i64>,
    /// Reserved at zero weight; see F9 mitigation note in schema DDL.
    pub fitness_dimension: f64,
}
```

### Public API

```rust
/// Open (or create) the `workflow_runs` database and run idempotent migrations.
pub fn open_database(path: &Path) -> Result<Connection, WorkflowError>;

/// Open an in-memory database with full schema. Used in tests.
pub fn open_memory() -> Result<Connection, WorkflowError>;

/// Insert a new open run (no outcome yet).
pub fn insert_run(conn: &Connection, started_at: &str) -> Result<i64, WorkflowError>;

/// Merge a cluster B observation into an existing run's `consumer_inputs` blob.
pub fn merge_observation(
    conn: &Connection,
    run_id: i64,
    obs: &ClusterBObservation,
) -> Result<(), WorkflowError>;

/// Close a run by recording `ended_at` and `outcome`.
pub fn close_run(
    conn: &Connection,
    run_id: i64,
    ended_at: &str,
    outcome: &str,
) -> Result<(), WorkflowError>;

/// Fetch all open runs (ended_at IS NULL), ordered by started_at DESC, limited to `limit`.
pub fn find_open(conn: &Connection, limit: usize) -> Result<Vec<WorkflowRunRow>, WorkflowError>;

/// Fetch completed runs with a specific outcome, limited to `limit`.
pub fn find_by_outcome(
    conn: &Connection,
    outcome: &str,
    limit: usize,
) -> Result<Vec<WorkflowRunRow>, WorkflowError>;
```

### Test density target

Minimum 50 tests. The boilerplate `m07_causal_chain.rs` demonstrates the test organisation: separate sub-sections per public function, edge cases on nullability, constraint violations, ordering guarantees. The `fitness_dimension` zero-weight contract warrants its own sub-section: a test asserting that `insert_run()` always produces a row with `fitness_dimension = 0.0`, and a test asserting that a direct SQL write of a non-zero value does NOT violate any constraint (the guard is not a CHECK — it is convention enforced by m10).

---

## m12 — `report_emitter`

### Role

m12 is the sole path from machine-readable `workflow_runs` rows to human-readable CLI text. It RECORDS nothing. It ACTS on nothing. It does not write to any database. It does not hold mutable state. Its entire API is a set of pure functions that accept query results and return `String` or `Vec<String>`.

Phase A passive-verb discipline: m12 EMITS text. It does not recommend, route, package, dispatch, optimise, or select.

### Boilerplate lift

**ME v2 `metrics.rs` report style** — the `Labels` builder pattern, the histogram bucketing approach (counter per bucket, rendered as a bar chart), and the per-module metric-registry organisation inform m12's report structure. The key pattern from ME v2 metrics is that formatting is separated from aggregation: a `MetricsRegistry` accumulates; a `render_*()` function produces text. m12 follows the same separation.

**`01-cli-scaffolding` patterns** — the CLI output conventions from `weaver.rs` / `enforcer.rs` (structured error boundaries, tracing JSON output for machine-readable mode, EnvFilter) inform m12's dual-mode output: plain text for interactive use, JSON for `--json` flag.

### Report formats

m12 produces three report types, each corresponding to a direct question a human might ask at the CLI.

#### 1. Histogram report — cost bands

Answers: "How are my workflow runs distributed by token cost?"

```
workflow-trace cost distribution (last 100 runs)
------------------------------------------------
   0 - 1k tokens |████████████████████| 23 runs
  1k - 5k tokens |███████████████     | 17 runs
 5k - 20k tokens |████████            |  9 runs
20k - 50k tokens |███                 |  3 runs
   > 50k tokens  |                    |  0 runs
------------------------------------------------
median: 3,412 tokens  p95: 18,204 tokens
```

Buckets are fixed: `[0, 1_000)`, `[1_000, 5_000)`, `[5_000, 20_000)`, `[20_000, 50_000)`, `[50_000, ∞)`. Bar width is normalised to the maximum bucket count, capped at 20 characters. No floating-point arithmetic in the render path — counts are integers.

#### 2. Trace report — outcome timeline

Answers: "What happened in the last N runs?"

```
workflow-trace timeline (last 20 runs)
---------------------------------------
2026-05-17T11:04:12Z  ok      3,412 tok
2026-05-17T10:51:33Z  fail    7,001 tok
2026-05-17T10:44:09Z  ok      2,189 tok
2026-05-17T10:38:55Z  abort   1,043 tok
2026-05-17T10:30:21Z  ok      4,817 tok
...
---------------------------------------
20 runs shown  |  ok: 14  fail: 4  abort: 2  unknown: 0
```

Columns are fixed-width: `started_at` ISO-8601 (20 chars), `outcome` padded to 7 chars, `cost_tokens` right-aligned to 9 chars. Open runs (no `ended_at`) are rendered as `---` in the cost column. Outcome symbols are plain ASCII — no emoji.

#### 3. Cost-band-by-cluster report — cascade correlation

Answers: "Is there a cost pattern by cascade cluster type?"

```
workflow-trace cost by cascade cluster (last 100 runs)
-------------------------------------------------------
cluster-id  runs   median-tok   p95-tok   ok-rate
A4f9c2      12     2,311        8,910     92%
B7e1a8      8      5,902        22,100    75%
(ungrouped) 80     3,412        18,204    88%
-------------------------------------------------------
```

Cluster IDs are printed as their opaque 6-char prefix. The "ungrouped" row covers runs where `consumer_inputs` contains no cascade observation. No human-meaningful labels are inferred or displayed — F11 enforced at the render layer as well as the storage layer.

### Ember gate compliance

m12's user-facing strings must pass m10's Ember gate. The gate checks for forbidden verbs in output text. A test in `tests/ember_gate.rs` (m10's module) includes a fixture that passes a sample m12 report string through the verb scanner and asserts zero forbidden-verb hits.

The forbidden verbs that could accidentally appear in m12 output are: "recommend", "optimise", "select", "route", "dispatch", "auto". m12's report labels use only passive descriptors: "recorded", "observed", "emitted", "cost", "outcome", "rate". The render functions carry a doc comment citing the Ember gate requirement.

### Public API

```rust
/// Render a cost-band histogram from a slice of completed runs.
/// Returns a multi-line `String` suitable for stdout.
#[must_use]
pub fn render_cost_histogram(runs: &[WorkflowRunRow]) -> String;

/// Render an outcome timeline from a slice of runs, newest first.
/// Open runs (no `ended_at`) are included with cost rendered as `---`.
#[must_use]
pub fn render_outcome_timeline(runs: &[WorkflowRunRow]) -> String;

/// Render a cost-by-cascade-cluster table from a slice of runs.
/// Cluster IDs are truncated to their 6-char opaque prefix.
/// Runs with no cascade observation appear in the `(ungrouped)` row.
#[must_use]
pub fn render_cluster_cost_table(runs: &[WorkflowRunRow]) -> String;

/// Render a compact summary line: total runs, outcome counts, median cost.
/// Used as the last line of every report.
#[must_use]
pub fn render_summary_line(runs: &[WorkflowRunRow]) -> String;
```

All functions are `#[must_use]` and `&[WorkflowRunRow]` (slice, not owned). They do not open database connections, do not write to files, do not call any external service.

### Test density target

Minimum 50 tests. Key test categories:
- Empty slice input: all four render functions return a non-empty string with zero-count labels (never panic on empty input).
- Single-run slice: all four functions produce correct output without division-by-zero or index-out-of-bounds.
- 100-run slice: histogram buckets sum to total; timeline contains exactly N lines before the footer; cluster table "ungrouped" row is correct.
- Forbidden-verb scan: `assert!(!render_cost_histogram(...).contains("recommend"))` and analogues for all four render functions and all forbidden verbs.
- Open-run rendering: `render_outcome_timeline` with a run where `ended_at = None` renders `---` in the cost column.

---

## m13 — `stcortex_writer_narrowed`

### Role

m13 is the only stcortex writer in the workflow-trace engine. It writes correlation rows to the `workflow_trace_*` namespace in stcortex, subject to two gates:

1. **m9 namespace guard** — m13 calls m9's `check_namespace()` before every write. Any key not prefixed `workflow_trace_` is rejected at m9's boundary. This enforces AP30 namespace collision avoidance.
2. **Hebbian LTP/LTD headroom check** — before promoting a row to stcortex, m13 queries the current `substrate_LTP_density` from a read-only probe of the Hebbian v3 telemetry endpoint. If the ratio falls below the refuse threshold, the write is deferred and logged.

### Boilerplate lift

**90% from `02-stcortex-consumer/capacity.rs`** — the `DbConnection::builder()` + `with_uri()` + `with_database_name()` construction pattern, `on_connect()` callback registration, and the `write_memory_then()` / `write_pathway_then()` reducer-callback pattern with `Ok(Ok(()))` vs `Err(...)` discrimination are lifted verbatim. The capacity probe's `reducer_ok` / `reducer_err` atomic counter pattern is retained for m13's internal success/failure accounting.

**`CONSUMER-ONBOARDING.md` refuse-write reducer template** — the reducer body `if consumer_count == 0 && namespace != "scratch" { return Err(...) }` is enforced at the DB layer, not at m13. m13 relies on this architectural commitment and does not re-implement it. What m13 does own is ensuring it has called `register_consumer` before any write attempt, following the onboarding checklist: stable name (`workflow-trace-m13`), namespace (`workflow_trace_*`), transport (`cli`), registered idempotently on startup.

**`03-sqlite-multi-db/m06_schema.rs` `open_database()` pattern** — m13 uses the same `configure_connection()` pragma block to open the local m7 SQLite file in read-only mode when constructing the correlation payload to write.

### Hebbian LTP/LTD backpressure

#### Where the ratio comes from

The Hebbian v3 deployment writes LTP and LTD density signals to the `substrate_LTP_density` key in the ORAC blackboard (per Hebbian Deployment Plan v3 telemetry spec). m13 reads this key via a lightweight HTTP GET to `http://localhost:8133/blackboard/substrate_LTP_density` before each promotion attempt.

The Hebbian v3 reconciliation note specifies: the field is a floating-point ratio of (cumulative LTP events in the last 1,000 ticks) / (cumulative LTD events in the last 1,000 ticks). A healthy habitat shows this ratio in the range `[2.0, 4.0]` per ME v2's STDP learning parameters. Hebbian v3 soak target is `> 0.15` on the absolute LTP/LTD ratio (normalised scale used in CLAUDE.local.md session notes).

#### Threshold and refuse logic

m13 uses two thresholds:

| Condition | Action |
|---|---|
| `substrate_LTP_density >= 0.15` | Write proceeds normally |
| `0.05 <= substrate_LTP_density < 0.15` | Write proceeds with a `tracing::warn!` emitted; the row is tagged `promoted_under_pressure = true` in the stcortex memory payload |
| `substrate_LTP_density < 0.05` | Write is **deferred**: row is appended to a local JSONL defer file at `~/.local/share/workflow-trace/deferred_writes.jsonl`; no stcortex write is attempted |
| `substrate_LTP_density` unavailable (ORAC unreachable, 5s timeout) | Write is **deferred** with the same JSONL path; a `tracing::error!` is emitted; m13 does not block the calling thread |

The defer file is a local JSONL append buffer. A background task in `wf-crystallise` re-attempts deferred writes on a 60-second interval when ORAC becomes reachable and `substrate_LTP_density >= 0.15`.

#### Why this threshold

The LTP/LTD soak target `> 0.15` is the same threshold cited in CLAUDE.local.md's Hebbian v3 row ("LTP/LTD ratio 0.055 → target > 0.15"). Promoting correlation rows to stcortex when the substrate is in LTD dominance risks amplifying suppression signals — the pathway weights being written would themselves be subject to LTD decay before the next learning step. The backpressure check ensures m13 writes only when the substrate is in a receptive (LTP-dominant or balanced) state.

### stcortex write shape

Each promoted row is written as a stcortex `memory` with a structured `content` string and an optional vector:

```rust
pub struct CorrelationMemory {
    /// Namespace — MUST be prefixed `workflow_trace_`; enforced by m9.
    pub namespace: String,
    /// Memory type: always "semantic" for correlation rows.
    pub memory_type: String,
    /// Content: JSON string encoding run_id, outcome, cost_tokens, cluster digest.
    pub content: String,
    /// Relevance: derived from outcome (ok=1.0, fail=0.5, abort=0.3, unknown=0.1).
    pub relevance: f32,
    /// Session ID: the atuin session-id of the originating run.
    pub session_id: String,
    /// Source tag: always "workflow-trace-m13".
    pub source_tag: Option<String>,
    /// Tensor: None in Phase A (fitness_dimension is zero-weight).
    pub tensor: Option<Vec<f32>>,
}
```

The `relevance` mapping (`ok=1.0, fail=0.5, abort=0.3, unknown=0.1`) is a fixed heuristic for Phase A. It is NOT derived from the fitness_dimension column (F9 mitigation — that column is at zero weight). In a future phase, relevance would be derived from Hebbian feedback; in Phase A it is a static encoding of outcome ordinal.

### Public API

```rust
/// Build an m13 writer, registering the stcortex consumer on construction.
///
/// # Errors
///
/// Returns `WorkflowError::StcortexUnavailable` if the connection cannot
/// be established within `connect_timeout`. The caller should log and
/// continue without stcortex writes rather than panicking.
pub fn StcortexWriter::new(
    stcortex_uri: &str,
    orac_uri: &str,
    connect_timeout: Duration,
) -> Result<Self, WorkflowError>;

/// Attempt to promote a completed `WorkflowRunRow` to stcortex.
///
/// Performs namespace guard check (m9), LTP/LTD headroom check,
/// and either writes immediately or defers to the local JSONL buffer.
///
/// # Errors
///
/// Returns `WorkflowError::NamespaceViolation` if m9 rejects the key.
/// Returns `WorkflowError::WriteDeferred` if the LTP/LTD ratio is below
/// threshold; the caller should treat this as informational, not fatal.
pub fn promote_run(
    &self,
    run: &WorkflowRunRow,
    namespace_key: &str,
) -> Result<PromoteOutcome, WorkflowError>;

/// Retry all rows in the local JSONL defer buffer.
///
/// Called by the `wf-crystallise` background task on a 60-second interval.
pub async fn flush_deferred(&self) -> Result<u32, WorkflowError>;
```

### Graceful degradation

m13 is designed to degrade gracefully when stcortex is unavailable. The entire stcortex write path is optional for the core recording loop — m7 is the source of truth, not stcortex. If stcortex is down:

1. `StcortexWriter::new()` returns `Err(StcortexUnavailable)`.
2. The `wf-crystallise` binary logs the error at `tracing::warn!` level and proceeds without an m13 instance.
3. m7 rows are still written; m12 reports still render; m14 evidence aggregation still runs.
4. When stcortex recovers, the deferred JSONL file (if it exists) is flushed on the next 60-second interval.

This matches the "probe, don't block" operational principle from CLAUDE.md § Habitat Operations.

### Test density target

Minimum 50 tests. Key test categories:
- Namespace guard: `promote_run()` with a key not prefixed `workflow_trace_` returns `Err(NamespaceViolation)`.
- LTP/LTD threshold branches: mock the ORAC probe; test all three threshold bands (`>= 0.15`, `0.05-0.15`, `< 0.05`).
- Deferred write JSONL: after a deferred write, the JSONL file contains exactly one line; `flush_deferred()` re-attempts and removes the line on success.
- Relevance mapping: `ok` row maps to `1.0`, `fail` to `0.5`, `abort` to `0.3`, `unknown` to `0.1`.
- Graceful degradation: when ORAC returns 5xx or times out, `promote_run()` returns `WriteDeferred` rather than propagating the network error.
- F9 invariant: the constructed `CorrelationMemory.tensor` is always `None` in Phase A.

---

## Cluster-internal synergy (m7 ↔ m12 ↔ m13)

The three modules are coupled in a strict read-direction DAG with no cycles:

```
m7 (write: Cluster B, Cluster A)
  |
  +-- m12 (read: m7; emit: stdout)
  |
  +-- m13 (read: m7; write: stcortex)
```

m12 and m13 never interact directly. Both read from m7. Neither writes back to m7. This keeps m7's SQLite file as the single-writer / multi-reader source of truth — no concurrent writes between cluster members.

The module docstring style follows ME v2 `resources.rs` format: Layer, Dependencies, Tests, Features table, Related Documentation. Each module docstring identifies its cluster and its position in the data-flow DAG.

---

## Quality gate requirements

All three modules are subject to the same 4-stage quality gate as the rest of the engine (from `09-trap-verify-escape-skills/SKILL-quality-gate.md`):

```bash
CARGO_TARGET_DIR=./target cargo check 2>&1 | tail -20 && \
cargo clippy -- -D warnings 2>&1 | tail -20 && \
cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=./target cargo test --lib --release 2>&1 | tail -30
```

Zero tolerance at every stage. Specific requirements for Cluster C:
- No `unwrap()` or `expect()` outside `#[cfg(test)]` blocks.
- No `unsafe`. No `panic!()`.
- All public items carry doc comments.
- `tracing::instrument` on any function that touches network I/O (m13's `promote_run` and `flush_deferred`).
- `#[must_use]` on all m12 render functions (pure functions with no side effects).
- The `fitness_dimension` zero-weight invariant is verified by a test in m7's test suite (see F9 above).
- m12's forbidden-verb invariant is verified by a test in m10's `tests/ember_gate.rs`.

---

## Implementation sequence (within Cluster C)

When the G1-G9 gates clear and Luke emits the start-coding signal, the recommended authoring sequence is:

1. **m7 first** — schema DDL + `open_database()` + `configure_connection()` (lifted from `m06_schema.rs`) + `WorkflowRunRow` struct + CRUD functions + 50+ tests. The schema is the stable contract that m12 and m13 depend on.
2. **m12 second** — render functions are pure; they can be developed against a `Vec<WorkflowRunRow>` fixture without any live database. The Ember gate test fixture is written here, cross-referencing m10.
3. **m13 last** — depends on m7 (reads `WorkflowRunRow`), m9 (namespace guard), and the ORAC blackboard probe. The deferred-write JSONL path is testable with mock ORAC responses.

The three modules are co-located in `workflow-core/src/cluster_c/` (or equivalent in the final directory layout). They share the same `WorkflowRunRow` type from `workflow-core/src/types.rs`.

---

*Cluster C spec authored S1001982 · HOLD-v2 active · no code until G1-G9 clear · next: Cluster D Trust spec*
