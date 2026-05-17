---
title: m7 — workflow_runs (central hub spec)
module_id: m7
module_name: workflow_runs
cluster: C — Correlation + Output
layer: L3
binary: wf-crystallise
verb_class: record
feature_gate: [api]
loc_estimate: 140-200
test_budget: 60-70
boilerplate_lift: ~40-50%
gap_owner: none (storage-of-record; F9 zero-weight mitigation owner)
cc_contracts_owned: [CC-1]
cc_contracts_consumed: [CC-2, CC-3, CC-5]
status: SPEC · planning-only · HOLD-v2 active · no code until G1-G9 clear
authority: Luke @ node 0.A
date: 2026-05-17 (S1001982)
---

# m7 — `workflow_runs`

> Back to: [`../../INDEX.md`](../../INDEX.md) · [`../../MODULE_MATRIX.md`](../../MODULE_MATRIX.md) · [`../../../CLAUDE.md`](../../../CLAUDE.md) · [`../../../CLAUDE.local.md`](../../../CLAUDE.local.md) · vault [[cluster-C-correlation-output]] · canonical V7 [cluster-C plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-C.md) · v1.3 spec [`../../../ai_docs/GENESIS_PROMPT_V1_3.md`](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 1
>
> Sister modules: [m7](m7_workflow_runs.md) · [m12](m12_cli_reports.md) · [m13](m13_stcortex_writer.md)
> Cluster peers in-flow: [cluster-A](../cluster-A/) (m1/m2/m3 ingest) · [cluster-B](../cluster-B/) (m4/m5/m6 observation streams write to m7) · [cluster-D](../cluster-D/) (aspect-wrap m8/m9/m10/m11) · [cluster-E](../cluster-E/) (m14 reads m7) · [cluster-G](../cluster-G/) (m30 evidence-derived from m7)

---

## 1 — Role + one-line purpose

m7 is the **central SQLite hub** of the workflow-trace engine: the single authoritative `workflow_runs` table that every Cluster B observation (m4 cascade, m5 battern step, m6 context cost) and every Cluster A ingest snapshot (m3 injection chain) converges into, and that every downstream evidence consumer (m12 reports, m13 stcortex writer, m14 lift, m30 bank, m40/m42 substrate emitters) reads from. m7 owns the schema, owns the migration discipline, and owns the F9 zero-weight `fitness_dimension` column mitigation.

m7 records. m7 does not recommend, route, or emit. The cluster verb-class is `record`; m7 is the primary record-er.

## 2 — Cluster + layer + binary placement

| Axis | Value |
|---|---|
| Cluster | **C — Central Correlation + Output** |
| Layer | **L3** (above L1 ingest, L2 observation; below L4 trust aspect-wrap) |
| Binary | **`wf-crystallise`** (read-heavy ingest + correlation + iteration; m7 is its persistence-of-record) |
| Feature gate | **`api`** — `workflow-core` exposes `WorkflowRunRow` + CRUD; downstream consumers depend on this surface |
| Verb class | **`record`** (passive; lifts cluster B writes; never decides) |
| src/ path | `src/m7_workflow_runs/` with `mod.rs`, `schema.rs`, `migrations/`, `query.rs`, `consumer_inputs.rs`, `error.rs` |

## 3 — Upstream-IN (what arrives)

| Source | Wire shape | Write path |
|---|---|---|
| **m4** `cascade_correlator` | `ClusterBObservation::Cascade { cluster_id: String, session_range: (i64, i64) }` — opaque ID per F11 (no human-meaningful labels) | `merge_observation(conn, run_id, &obs)` — JSON patch into `consumer_inputs` blob |
| **m5** `battern_step_record` | `ClusterBObservation::BatternStep { battern_id: String, step_index: u8, duration_ms: u64, outcome: StepOutcome }` — 6 steps per battern invocation | same merge entry point |
| **m6** `context_cost_record` | `ClusterBObservation::ContextCost { session_id: i64, cost_tokens: u64 }` + writes the `cost_tokens` column directly | same merge entry point + `update_cost_tokens` helper |
| **m3** `injection_db_consumer` | Direct join key — `chain_id` is stored in the `consumer_inputs` blob under `"injection_chain_id"` | merge under discriminant |
| **Run lifecycle (caller)** | `insert_run(conn, started_at)` opens the row; `close_run(conn, run_id, ended_at, outcome)` finalises | direct CRUD |

m7 does NOT read from any source — its inbound surface is its own public-API function set. Callers (m4/m5/m6) hold the connection handle (opened via m7's `open_database`) and invoke the merge functions directly. This keeps m7 single-writer at the SQLite layer (no contention between cluster members).

## 4 — Downstream-OUT (what departs)

| Destination | Wire shape | Read path |
|---|---|---|
| **m12** `cli_reports` | `&[WorkflowRunRow]` slice returned by `find_open` / `find_by_outcome` | pure read; m12 holds no mutable state |
| **m13** `stcortex_writer` | `&WorkflowRunRow` then derived `CorrelationMemory` payload | m13 reads m7 in read-only mode; never writes back |
| **m14** `habitat_outcome_lift` (CC-3) | direct SQLite open in read-only mode against the m7 file path (shared constant in `workflow-core`) | Wilson CI computed externally; m7 stays a bucket |
| **m30** `curated_bank` | row-by-row read for `definition_hash` derivation at proposal-accept time | indirect via m23 proposer + operator-review handoff |
| **m40** `nexusevent_emit` (CC-5) | rows where `outcome IS NOT NULL` only — stale open rows are invisible to m40 | filtered read |
| **m11** `fitness_weighted_decay` (Gap 2 owner) | `last_run_at` / `frequency` / `recency` aggregates derived from m7 | m11 computes the formula; m7 is the data substrate |

## 5 — Aspect-IN (Cluster D trust-layer wraps)

| Aspect | Wrapping point |
|---|---|
| **m8** `povm_build_prereq` | compile-time gate — m7 compiles only if `cargo:rustc-cfg=povm_calibrated` is set (env-only, defense-in-depth alongside DB-layer refuse-write) |
| **m9** `watcher_namespace_guard` | write-time validator on every row payload destined for stcortex via m13 (m9 is invoked from m13, not m7; m7 stores plain JSON, m13 wraps with the namespace prefix at promotion) |
| **m10** `ember_ci_gate` | DDL changes are PR-text; trait audit applies to schema-comment language (no forbidden verbs in the schema comments either; F9 mitigation note is the load-bearing example) |
| **m11** `fitness_weighted_decay` | m7's `fitness_dimension` column stays at zero-weight default until m11 confirms LTP/LTD ratio above the Hebbian v3 threshold (`> 0.15`); m11 reads m7 stats; m7 never reads m11 |

## 6 — Public API (lifted verbatim from vault canonical)

```rust
/// Open (or create) the `workflow_runs` database and run idempotent migrations.
pub fn open_database(path: &Path) -> Result<Connection, WorkflowError>;

/// Open an in-memory database with full schema. Used in tests.
pub fn open_memory() -> Result<Connection, WorkflowError>;

/// Insert a new open run (no outcome yet). Returns the freshly-allocated row id.
pub fn insert_run(conn: &Connection, started_at: &str) -> Result<i64, WorkflowError>;

/// Merge a cluster B observation into an existing run's `consumer_inputs` blob.
/// JSON-patch under the observation's discriminant key; existing keys are overwritten.
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

### Row type

```rust
/// A single row from the `workflow_runs` table.
///
/// All columns are present; `ended_at` and `outcome` are nullable and
/// represented as [`Option`] until the run completes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowRunRow {
    pub id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub outcome: Option<String>,
    pub consumer_inputs: String,
    pub cost_tokens: Option<i64>,
    pub fitness_dimension: f64,
}
```

### Cluster B observation enum

```rust
pub enum ClusterBObservation {
    Cascade {
        cluster_id: String,   // opaque ID from m4; never human-meaningful (F11)
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

All public items carry rustdoc comments (god-tier rule 6); the `consumer_inputs` JSONB column doc comment cites CC-1 explicitly so future readers find the join contract from the type alone.

## 7 — Schema DDL (the load-bearing contract)

The full DDL is in vault canonical [[cluster-C-correlation-output]] § m7 Schema DDL. Locked invariants:

- Primary key `INTEGER PRIMARY KEY AUTOINCREMENT` — never reused, even after row delete (m7 never deletes; sunset is m11 / m30's concern via decay weighting).
- `outcome TEXT CHECK(outcome IN ('ok', 'fail', 'abort', 'unknown'))` — closed enum at the DB layer; mirrored by a Rust `Outcome` enum in `workflow-core::types`.
- **`consumer_inputs TEXT NOT NULL DEFAULT '{}'`** — JSONB blob (TEXT-stored; sqlite JSON1 functions for query). This is the CC-1 join surface — no internal struct sharing between m4 / m6; the join is a SQL query against m7.
- **`fitness_dimension REAL NOT NULL DEFAULT 0.0`** — F9 zero-weight mitigation (§ 12).
- Pragma block is **mandatory** (lifted verbatim from `03-sqlite-multi-db/m06_schema.rs`): `journal_mode=WAL`, `busy_timeout=5000`, `foreign_keys=ON`, `synchronous=NORMAL`, `wal_autocheckpoint=100`. The `wal_autocheckpoint=100` keeps write amplification bounded under m4/m5/m6's append-heavy load.

Two indexes:
- `idx_workflow_runs_open` on `(started_at DESC) WHERE ended_at IS NULL` — primary live-runs query path.
- `idx_workflow_runs_outcome` on `(outcome, ended_at DESC) WHERE outcome IS NOT NULL` — m14 evidence aggregation path.

## 8 — Error taxonomy (thiserror)

```rust
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("sqlite error: {0}")]
    Sqlite(String),
    #[error("schema migration failed at step {step}: {source}")]
    Migration { step: u32, source: rusqlite::Error },
    #[error("row {id} not found in workflow_runs")]
    RowNotFound { id: i64 },
    #[error("invalid outcome '{0}' — must be one of ok|fail|abort|unknown")]
    InvalidOutcome(String),
    #[error("consumer_inputs JSON patch failed: {0}")]
    JsonPatch(String),
    #[error("connection acquisition failed: {0}")]
    Connection(String),
}
```

No `Box<dyn Error>` in any public signature (god-tier rule 9). Every error variant is constructable from the call-site without losing source-location data; conversion impls bridge `rusqlite::Error` and `serde_json::Error` into the local taxonomy.

## 9 — Newtype discipline + types

m7 leans on `workflow-core::types` for cross-module shared types: `SessionId(i64)`, `RunId(i64)`, `BatternId(String)`, `ClusterId(String)`, `Outcome` enum, `StepOutcome` enum. These newtype wrappers are NOT optional — they prevent the AP-V7-class "raw String drift" failure mode where a `cluster_id` from m4 silently becomes a `battern_id` at the m7 write boundary.

## 10 — Tests (60-70 minimum; per `TEST_DISCIPLINE.md` row m7)

Allocation lifted from V7 cluster-C plan § m7 Test-pattern allocation:

| Pattern | Count | Coverage |
|---|---|---|
| F-Unit | 30 | `WorkflowRunRow` field-defaults · `Outcome` enum per-arm · `consumer_inputs` JsonValue construction per B-cluster shape · insert/select per column · **F9 default-application** (`fitness_dimension` always `0.0` after `insert_run`) |
| F-Property | 5 | schema invariants: `fitness_dimension` never null · `consumer_inputs` roundtrips (insert → select → deserialize → equal) · `started_at ≤ ended_at` · `outcome ∈ {ok,fail,abort,unknown}` |
| F-Fuzz | 1 | JSONB blob roundtrip (`fuzz_targets/m7_jsonb_fuzz.rs`); assertion `serde_json::from_value(serde_json::to_value(blob)) == blob` |
| F-Integration | 18 | m4 → m7, m5 → m7, m6 → m7 insert chains · m7 → m11/m12/m13/m14 read chains · concurrent insert · migration up/down · schema-stability across `migrate run` |
| F-Contract | 5 | `WorkflowRunRow` insta snapshot · SQL DDL stability snapshot · `consumer_inputs` JSON schema |
| F-Regression | 3 | **F9 regression** (any commit removing NOT NULL DEFAULT 0.0 fails) · `consumer_inputs` column omission · migration-applied-but-schema-unchanged (AP-Drift-05) |
| F-Mutation | budget | ≥70% on `schema.rs` + `query.rs` |

Tests live in `src/m7_workflow_runs/tests/` (unit + property + fuzz) and `tests/cluster_c_m7_*.rs` (integration). Integration tests use real local SQLite, never mocks (per TEST_DISCIPLINE § Integration-test pattern; AP-Test-03 avoidance).

## 11 — Reuse + boilerplate lift

| Source | Lift % | What |
|---|---|---|
| `03-sqlite-multi-db/m06_schema.rs` | **90%** | `open_database()` · `configure_connection()` (the pragma block above) · idempotent migrate loop · `column_exists()` schema-discovery helper |
| `03-sqlite-multi-db/m07_causal_chain.rs` | **70%** | `parse_row()` helper pattern · `auto_resolve_stale_typed()` (adapted to outcome-based, not session-gap, resolution) · `find_unresolved()` → `ORDER BY ... DESC LIMIT ?1` query pattern. `CausalChainRow` is the template for `WorkflowRunRow` |
| ME v2 `src/m07_*/schema.rs` | ~60% | workflow-runs DDL constants + sqlx migration scaffolding |
| LCM `src/loop_runs/` | ~40% | insert/select helpers for `query.rs` |
| `consumer_inputs` JSONB join | **0% (fresh)** | ~60 LOC novel-but-bounded; CC-1 join surface (vault canonical § m7) |

Net lift across m7: ~40-50%. Fresh authorship concentrated in the `consumer_inputs` JSON-patch merge function and the F9 zero-weight invariant tests.

## 12 — F9 mitigation (the load-bearing rationale)

**F9** is "workflow-grain fitness distortion" — the risk that a fitness signal computed at workflow granularity leaks upward into the Hebbian substrate's 12D tensor before enough empirical data is collected.

The mitigation is structural, not procedural:

1. The column exists (`fitness_dimension REAL NOT NULL DEFAULT 0.0`) so future migrations adding a real fitness signal are **additive**, not structural — no schema rewrite to introduce the dimension.
2. The default is hard-coded `0.0` at the DB layer; `insert_run` never accepts a fitness argument.
3. The schema-DDL comment block is load-bearing documentation (NOT decorative) — it cites the zero-weight contract verbatim. Removing or weakening that comment is a v1.3 violation surface (m10 Ember CI gate audits DDL change PRs).
4. No module may write a non-zero value to `fitness_dimension` in Phase A without first passing the m10 Ember gate AND having the LTP/LTD ratio confirmed above the 0.15 threshold by Hebbian v3 telemetry. The guard is convention enforced by m10 — NOT a CHECK constraint (intentionally permissive at the DB layer so future migrations can populate it without DB-side drama).
5. F-Property test: assert insert + select roundtrip preserves `fitness_dimension = 0.0` for every test run.
6. F-Regression test: any commit that removes `NOT NULL DEFAULT 0.0` from the DDL fails the regression suite.

**Reference:** vault canonical [[cluster-C-correlation-output]] § F9 zero-weight column rationale · V7 cluster-C plan § m7 Failure-modes covered · Hebbian v3 reconciliation note `~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md` (substrate_LTP_density > 0.015 Phase 1 target).

## 13 — Cross-cluster contracts owned + consumed

### CC-1 — Cascade-Cost Coupling (m7 is the canonical owner)

The stable join contract between m4 (cascade) and m6 (cost) lives in m7's schema. m4 writes cascade cluster IDs; m6 writes cost. m7 is the join plane — neither m4 nor m6 knows about the other. Queries that correlate cascade cluster with cost are SQL against m7. This is the **"join schema as stable contract"** pattern (vault canonical § CC-1).

Wire shape: `consumer_inputs` JSONB column with discriminants `"cascade"`, `"battern_step"`, `"context_cost"`, `"injection_chain_id"`. Synergy file: [`../../synergies/CC-1.md`](../../synergies/CC-1.md) (TBD Wave 2).

### CC-2 — Trust Layer Woven (consumed)

m7's compile is gated on `cfg(povm_calibrated)` from m8; m7's downstream-OUT to stcortex (via m13) is gated on m9 namespace validation; m7's DDL changes are audited by m10. m7 itself never invokes the aspect-layer — it is wrapped by it.

### CC-3 — Evidence-Driven Iteration (consumed via m14)

`WorkflowRunRow` is the primary input to m14's lift evidence. m14 opens m7's SQLite file read-only. CC-3 guarantee: every `WorkflowRunRow.id` is queryable forever (no row deletion; sunset via m11 decay weighting + m30 bank pruning at the proposal layer).

### CC-5 — Substrate Learning Loop (consumed via m13 + m42)

m7's rows feed m13's stcortex writer and m42's substrate-feedback emitter; pathway weights written by those modules eventually flow back into m31's selector. m7 is two hops upstream of the loop closure.

## Failure-modes covered (subset of `ANTIPATTERNS_REGISTER`)

- **AP-WT-F9** workflow-grain fitness distortion — **prime owner** (§ 12).
- **AP-Drift-05** migration "applied" but schema unchanged — F-Integration `migrate run` cross-check + sqlx `__migrations` table inspection.
- **AP-Drift-06** bridge contract drift — `WorkflowRunRow` insta snapshot guards stability vs m13/m40/m42 consumers.
- **AP-V7-09** substrate-frame engine confusion — `consumer_inputs` JSONB column is substrate-frame discipline; anthropocentric back-decoding is Class-G drift (Watcher class pre-position).
- **AP-Test-03** integration-test mock leak — m7 integration tests use real local SQLite.

## Atuin trajectory anchor

- `wt-pulse` (proposed per V7 T5.2; reads `SELECT COUNT(*) FROM workflow_runs WHERE started_at > ?`).
- `wt-gate-status` (proposed; reads m7's most-recent-row timestamp).
- `habitat-bootstrap` consumes m7's row count for context construction at session boot.

## Watcher class pre-position

- **Class A** — first `INSERT INTO workflow_runs` post-Genesis (substrate-write activation).
- **Class D** — four-surface drift if m7 row exists without corresponding ai_docs / vault / stcortex / CLAUDE.local.md anchor.
- **Class I** — Hebbian silence (firing live per tick·16); m7's row count over time is the direct measure.

## Implementation order within Cluster C

m7 **first** — schema DDL + `open_database()` + `WorkflowRunRow` struct + CRUD + 60+ tests. m12 and m13 depend on this stable contract. (Vault canonical § Implementation sequence.)

---

*m7 spec authored 2026-05-17 (S1001982) by Command for the Cluster C author wave. Planning-only; HOLD-v2 active; no code until G1-G9 clear. Risk surface on Command's head per Luke single-phase override § 4 W-1 through W-5.*

> Sister-module bottom anchors: [m7](m7_workflow_runs.md) · [m12](m12_cli_reports.md) · [m13](m13_stcortex_writer.md) · vault [[cluster-C-correlation-output]] · canonical V7 [cluster-C plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-C.md)
