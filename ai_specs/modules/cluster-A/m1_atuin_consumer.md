---
title: m1 — `m1_atuin_consumer` Rust spec
cluster: A — Substrate Ingest
layer: L1
binary: wf-crystallise
loc_estimate: ~80
test_count_min: 50
test_kinds: [unit, property, integration, contract, regression, mutation]
feature_gate: [none]
verb_class: passive
cc_owns: []
cc_consumes: [CC-1, CC-3]
gap_owner: [none]
boilerplate_lift_pct: 30
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A
hold_v2_compliant: true
decisions_applied: [D-B]
---

# m1 — `m1_atuin_consumer` Rust spec

> Back to: [vault cluster-A spec](../../../the-workflow-engine-vault/module%20specs/cluster-A-substrate-ingest.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · [V7 cluster-A plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-A.md) · [GENESIS v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · vault [[cluster-A-substrate-ingest]]

## 1. Purpose & invariants

`m1_atuin_consumer` IS the read-only ingress to atuin's local shell-history SQLite database (`~/.local/share/atuin/history.db`, ~263k rows in the live habitat). Its sole job is to surface raw tool-call rows — cursor-paginated — to downstream observers in Cluster B without interpretation, filtering, or rewriting. m1 is the canonical example of a Phase A passive verb: **ingest**.

The module MUST guarantee three invariants. First, it never writes to atuin — enforced both at the URI level (`?mode=ro`) and at the SQLite-pragma level (`PRAGMA query_only = ON`), defense-in-depth against any accidental call site. Second, it preserves byte-for-byte command/session/cwd payloads so that m4's FNV-1a-XOR opaque-ID derivation downstream is deterministic across runs (any pre-folding here would silently corrupt CC-1 cascade correlation). Third, cursor monotonicity holds across `next_page()` calls so pagination is deterministic and re-readable — same cursor in, same rows out.

Frame violations m1 must structurally refuse: (a) treating an `AtuinHistoryRow` as anthropocentric "user intent" rather than opaque substrate trajectory (Watcher Class-G); (b) cascade monoculture (AP-WT-F11) — m1 cannot fold rows into derived shapes; that is m4's responsibility; (c) substrate-input poisoning (AP-WT-F3) — the read-only enforcement and WAL busy-timeout block any accidental write back into atuin.

The module is `cfg(povm_calibrated)`-gated at the crate root by m8; the entire crate refuses to build if POVM `learning_health` is out of band (CC-2 trust layer woven). m1 itself emits nothing into stcortex or POVM and so does not engage m9's namespace guard; m9 wraps any downstream m13/m42 derived from m1's data.

## 2. Public surface (Rust types — spec only, NOT compileable)

```rust
//! # m1_atuin_consumer
//!
//! - **Layer**: L1 (Substrate Ingest, Cluster A)
//! - **Deps**: rusqlite (read-only WAL), workflow_core::types::{SessionId}, workflow_core::errors::IngestError
//! - **Tests**: 50 (25 unit + 5 property + 15 integration + 3 contract + 2 regression + mutation budget ≥70%)
//! - **Features**: none
//! - **Platform**: Linux (atuin path conventions); read-only URI; busy_timeout = 5000 ms
//! - **Impl Notes**: Cursor-based pagination authored fresh (~30 LOC, no boilerplate equivalent); WAL pragma batch lifted from `memory-injection/m06_schema.rs::configure_connection` with `PRAGMA query_only = ON` added
//! - **Related Docs**: [cluster-A vault spec](../../../the-workflow-engine-vault/module%20specs/cluster-A-substrate-ingest.md) · [V7 cluster-A](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-A.md) · [ULTRAMAP](../../../ai_docs/optimisation-v7/ULTRAMAP.md) View 2 row m1

/// A single normalised atuin history row.
/// Newtype discipline: `SessionId` is wrapped, not raw `String`.
#[derive(Debug, Clone)]
pub struct AtuinHistoryRow {
    pub id: i64,
    pub command: String,
    pub session: SessionId,
    pub hostname: String,
    pub timestamp_ms: i64,
    pub exit: Option<i32>,
    pub duration_ms: Option<i64>,
    pub cwd: Option<String>,
}

/// Configuration controlling pagination and fallback behaviour.
#[derive(Debug, Clone)]
pub struct AtuinConsumerConfig {
    /// Page size, bounded [100, 10_000]. Default: 2_000 (per D-B, Luke S1002127 — fewer round-trips on hot session-start path).
    pub page_size: usize,
    /// Total ingest cap across all pages. `None` = no limit.
    pub row_cap: Option<usize>,
    /// Subprocess fallback timeout (ms). Default: 5_000.
    pub subprocess_timeout_ms: u64,
    /// Override path; defaults to `~/.local/share/atuin/history.db`.
    pub db_path_override: Option<std::path::PathBuf>,
}

impl Default for AtuinConsumerConfig { /* ... */ }

/// Typed error taxonomy (no `Box<dyn Error>`).
#[derive(Debug, thiserror::Error)]
pub enum AtuinConsumerError {
    #[error("database open failed at {path}: {reason}")]
    DatabaseOpenFailed { path: std::path::PathBuf, reason: String },
    #[error("WAL busy-timeout exceeded after {timeout_ms}ms")]
    BusyTimeout { timeout_ms: u64 },
    #[error("query failed: {0}")]
    QueryFailed(String),
    #[error("subprocess fallback failed: {0}")]
    SubprocessFailed(String),
    #[error("row parse error at id={row_id}: {reason}")]
    RowParseFailed { row_id: i64, reason: String },
}

/// Page result with cursor + exhaustion flag (lifted from m11_parallel_query::QueryResult).
#[derive(Debug, Clone)]
pub struct PageResult {
    pub rows: Vec<AtuinHistoryRow>,
    pub last_id: i64,
    pub exhausted: bool,
    pub elapsed_ms: u64,
}

/// Open atuin's history database for read-only paginated ingest.
///
/// # Errors
/// `DatabaseOpenFailed` if the file is missing or unopenable.
/// `BusyTimeout` if WAL lock exceeds `subprocess_timeout_ms`.
pub fn open_readonly(config: &AtuinConsumerConfig) -> Result<AtuinConsumer, AtuinConsumerError>;

pub struct AtuinConsumer { /* private */ }

impl AtuinConsumer {
    /// Next paginated page in `id ASC` order; `Ok(None)` on exhaustion.
    pub fn next_page(&mut self) -> Result<Option<PageResult>, AtuinConsumerError>;

    /// Collect all pages (respects `row_cap`). Convenience for tests/small histories.
    pub fn collect_all(self) -> Result<Vec<AtuinHistoryRow>, AtuinConsumerError>;

    /// Lazy page iterator.
    pub fn into_page_iter(self) -> impl Iterator<Item = Result<PageResult, AtuinConsumerError>>;
}

/// Subprocess fallback: `atuin history list --limit N --format json`.
pub fn fallback_subprocess_ingest(
    config: &AtuinConsumerConfig,
) -> Result<Vec<AtuinHistoryRow>, AtuinConsumerError>;
```

## 3. Internal data structures

```rust
struct ConsumerState {
    conn: rusqlite::Connection,
    last_id: i64,
    rows_yielded: usize,
    exhausted: bool,
    config: AtuinConsumerConfig,
}
```

`ConsumerState` is the cursor primitive: `last_id` is the exclusive lower bound for the next query (`WHERE id > ? ORDER BY id ASC LIMIT ?`); `rows_yielded` enforces `row_cap`; `exhausted` is sticky so re-calls after exhaustion are idempotent `Ok(None)`.

## 4. Data flow

- **INPUT FROM:** `~/.local/share/atuin/history.db` (SQLite WAL, read-only URI, `PRAGMA query_only = ON`)
- **OUTPUT TO:** Cluster B — `Iterator<Item = Result<PageResult, AtuinConsumerError>>` consumed by m4 `cascade_correlator`, m5 `battern_step_record`, m6 `context_cost`
- **SUBSTRATE TOUCHED:** atuin only (m1 is not a stcortex consumer; no POVM/injection.db touch)
- **WRITES:** none (Phase A read-only invariant; enforced at URI + PRAGMA layers)

## 5. Algorithm sketch

```text
open_readonly(config):
    path = config.db_path_override OR ~/.local/share/atuin/history.db
    conn = rusqlite::Connection::open_with_flags(uri(path, ro), READ_ONLY)
    configure_connection(&conn)            // lift from m06_schema.rs + add query_only
    return AtuinConsumer { state: { conn, last_id: 0, rows_yielded: 0, exhausted: false, config } }

next_page(&mut self):
    if self.state.exhausted: return Ok(None)
    stmt = "SELECT id, command, session, hostname, timestamp, exit, duration, cwd
            FROM history WHERE id > ?1 ORDER BY id ASC LIMIT ?2"
    rows = stmt.query_map([last_id, effective_page_size()], parse_row)
    if rows.is_empty(): self.state.exhausted = true; return Ok(None)
    self.state.last_id = rows.last().id
    self.state.rows_yielded += rows.len()
    if row_cap reached: self.state.exhausted = true
    return Ok(Some(PageResult { rows, last_id, exhausted, elapsed_ms }))
```

`effective_page_size()` clamps `page_size` to `[100, 10_000]` per V7 plan and respects remaining `row_cap` budget.

## 6. Boilerplate lifts

Per vault cluster-A spec § Boilerplate lifts table:

| Source | Lift | % |
|---|---|---:|
| `memory-injection/m06_schema.rs::configure_connection` | WAL/busy_timeout/foreign_keys/synchronous/wal_autocheckpoint batch | 90% (add `query_only = ON`) |
| `memory-injection/m06_schema.rs::open_database` | Path-existence guard, parent-dir creation, typed-error mapping | 85% (strip migration logic; open as read-only URI) |
| `memory-injection/m11_parallel_query.rs::QueryResult<T>` | Timing harness, staleness annotation | 80% (rename `PageResult`, threshold 200ms) |
| `memory-injection/m18_atuin_cache.rs` | Graceful-degrade subprocess wrapper | 70% (abstract into `FallbackIngest<T>` trait; structured error) |
| **Cursor-based pagination** | — | **0% (novel ~30 LOC)** |

Net: ~50 LOC lifted / ~30 LOC novel. The cursor primitive is the Boilerplate Hunt's flagged absence.

## 7. ME v2 m1_foundation patterns referenced

- `error.rs` — `thiserror::Error` enum with structured named-field variants (`{ path, reason }`, `{ row_id, reason }`, `{ timeout_ms }`); callers can match programmatically rather than parsing display strings.
- `resources.rs` — `//!` module docstring block (Layer / Deps / Tests / Features / Platform / Impl Notes / Related Docs); adopted verbatim shape.
- `shared_types.rs` — newtype discipline (`SessionId(String)`, not bare `String`); `AtuinConsumerConfig` follows `Default + env-override + override-field` pattern from `m03_config.rs`.
- `logging.rs` — tracing-subscriber structured emit on every `next_page()` (page count, elapsed_ms, last_id).

## 8. Test strategy

- **Test kind**: unit (25) + property (5) + integration (15) + contract (3) + regression (2)
- **Test count**: 50 minimum (per [TEST_DISCIPLINE matrix](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) row m1; cluster A total 150)
- **Mutation budget**: ≥70% kill on `cursor.rs` + `row.rs`
- **Properties tested** (F-Property 5):
  - Monotonic cursor non-decrease across pages: `prop_assume!(p2.last_id >= p1.last_id)`
  - Idempotent re-read of same `last_id` returns identical rows
  - Page size bound: `0 <= returned.len() <= page_size`
  - Exhaustion stickiness: after `Ok(None)`, further calls return `Ok(None)`
  - No duplicate `id` across pages
  - **page_size config override is respected** (per D-B, Luke S1002127): construction with explicit `page_size = N` (within `[100, 10_000]`) yields pages of size `min(N, remaining_rows, row_cap_budget)`; default-construction yields 2_000

Key invariants (sample of 50; full list in vault cluster-A spec § Tests m1):

1. `open_readonly` returns `DatabaseOpenFailed` when path missing
2. `PRAGMA query_only = ON` actually rejects INSERTs on the opened connection
3. `busy_timeout = 5000` set
4. Pagination cursor starts at `id = 0`
5. Cursor monotonic across pages
6. Empty table → first `next_page` returns `Ok(None)`
7. `page_size = 1` walks 10-row table in 10 pages with no gaps
8. `row_cap = Some(5)` stops at 5 rows regardless of table size
9. `collect_all` matches `into_page_iter().flatten()` row-for-row
10. WAL concurrency: two readers do not deadlock within busy_timeout
11. Large table (1000 seeded): no duplicates across page boundaries
12. `fallback_subprocess_ingest` respects `subprocess_timeout_ms`
13. `RowParseFailed` produced when column type mismatch occurs
14. `ToolCallRow.exit/duration_ms/cwd` are `None` for NULL columns
15. Cursor does not restart after exhaustion (idempotent `Ok(None)`)

(remaining 35 enumerated in vault spec)

## 9. Antipatterns to avoid

- **AP-WT-F3** (substrate-input poisoning) — read-only URI + `PRAGMA query_only = ON`; m1 cannot write to atuin
- **AP-WT-F11** (cascade monoculture) — m1 emits verbatim rows; no pre-folding (m4 owns opaque-ID derivation)
- **AP-V7-09** (substrate-frame engine confusion) — `AtuinHistoryRow` is opaque trajectory, not user intent
- **AP-Drift-05** (migration "applied" but schema unchanged) — m1 never migrates atuin; mitigated by construction
- **AP-Hab-04** (preserve-list discipline) — never blanket-execute; pagination is bounded
- **Newly surfaced**: cursor rollback on partial-read failure — must NOT advance `last_id` if `parse_row` failed mid-page (regression slot reserved)

## 10. Useful patterns applied

- Lazy cursor-based pagination (PATTERNS.md § Data-flow row 1)
- Subprocess CLI fallback (PATTERNS.md § Data-flow row 2)
- thiserror error enums (GOLD_STANDARDS rule 9)
- Newtype discipline (GOLD_STANDARDS rule 8)
- `//!` docstring block (GOLD_STANDARDS rule 13)

## 11. Cross-cluster contracts

- **CC-1 (consumes via downstream m7 join)**: m1 emits `AtuinHistoryRow` carrying `session: SessionId` that survives the join in m7's JSONB hub; m1 must guarantee the `session` field is byte-stable across re-reads of the same atuin row (no normalisation).
- **CC-3 (consumes via E→F)**: m1 is the upstream provenance source; `AtuinHistoryRow.id` + `session` must propagate through m5 → m20 → m14's evidence layer without loss.
- **CC-7 closure target (consumes config feedback)**: per V7 cluster-A plan § Cluster-level synergies, m15's pressure register can escalate to spec amendment that updates `m1.config.page_size` or cursor strategy in a subsequent session. m1 is one of three Cluster A modules whose config is a CC-7 feedback target.

## 12. Open questions for G5 interview / Zen G7 audit

1. **Page-size source of truth — DECIDED.** **DECIDED 2026-05-17 (S1002127, Luke directive "best practice aligning with impactful performance"):** page_size default = **2_000**. Rationale: fewer round-trips on hot session-start path; ~263k atuin rows → ~131 pages vs ~263 pages; memory cost ~400KB per page negligible. Config knob retained for opt-out (TOML/CLI override, bounded `[100, 10_000]`). Test strategy § 8 adds property test that the page_size config override is respected.
2. **Cursor persistence**: persist `last_id` to `daemon_state` across CLI invocations, or always restart from 0? (Streaming consumers vs snapshot semantics.)
3. **Row freshness window**: snapshot at open, or allow live rows mid-iteration? (atuin writes continuously; behaviour must be specified for m4's cascade correlator.)
4. **Pre-tokenisation**: does m4 need raw `command` or stripped/tokenised form? (Determines whether m1 should strip shell quoting at parse time.)

## 13. Implementation order (post-G9)

1. `error.rs` — `AtuinConsumerError` enum (`thiserror`); compile-only, no tests
2. `row.rs` — `AtuinHistoryRow` + `parse_row` + 5 unit tests (column extraction + NULL handling)
3. `config.rs` — `AtuinConsumerConfig` + `Default` + 3 unit tests (default values + override resolution + page_size clamp)
4. `cursor.rs` — `ConsumerState` + cursor primitive + 10 unit tests (monotonic / exhaustion / cap)
5. `mod.rs` — `open_readonly` + `AtuinConsumer::next_page` + WAL pragma batch lift + 7 unit tests (open paths + pragma assertion)
6. Property tests (5) — proptest on cursor invariants
7. Integration tests (15) — `tests/m1_integration.rs` with fixture atuin.db at `tests/fixtures/atuin-mini.db`
8. Contract tests (3) — insta snapshots for `AtuinHistoryRow` schema vs atuin v18.10
9. Regression slots (2) — placeholders for first bugs (cursor-rollback-on-error, page_size=0)
10. Mutation pass — `cargo mutants` on `cursor.rs` + `row.rs`; ≥70% kill required

---

> Back to: [vault cluster-A spec](../../../the-workflow-engine-vault/module%20specs/cluster-A-substrate-ingest.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · sister modules: [m1](m1_atuin_consumer.md) · [m2](m2_stcortex_consumer.md) · [m3](m3_injection_db_consumer.md)
