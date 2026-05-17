---
title: m3 — `m3_injection_db_consumer` Rust spec
cluster: A — Substrate Ingest
layer: L1
binary: wf-crystallise
loc_estimate: ~70
test_count_min: 50
test_kinds: [unit, property, integration, contract, regression, mutation]
feature_gate: [none]
verb_class: passive
cc_owns: []
cc_consumes: [CC-1]
gap_owner: [none]
boilerplate_lift_pct: 40
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A
hold_v2_compliant: true
---

# m3 — `m3_injection_db_consumer` Rust spec

> Back to: [vault cluster-A spec](../../../the-workflow-engine-vault/module%20specs/cluster-A-substrate-ingest.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · [V7 cluster-A plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-A.md) · [GENESIS v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · vault [[cluster-A-substrate-ingest]]

## 1. Purpose & invariants

`m3_injection_db_consumer` IS the read-only ingress to the habitat's `causal_chain` ledger at `~/.local/share/habitat/injection.db`. The `causal_chain` table is the canonical record of habitat-level bugs, traps, plans, and patterns with `reinforcement_count` tracking; workflow-trace reads it to correlate long-lived unresolved chains (high reinforcement, `resolved_session IS NULL`) against tool-call sequences observed by Clusters A/B. The output flows to m7's central correlation hub as supplementary lifecycle context — m7 joins on `session_id` and `chain_id` so workflow patterns can be recorded against open habitat issues.

The module MUST guarantee three invariants. First, **read-only enforcement** — the SQLite connection is opened in read-only URI mode with `PRAGMA query_only = ON`; m3 never migrates, resolves, inserts, or updates a chain. injection.db's schema (v5 per `memory-injection/m06_schema.rs`) is owned by the `memory-injection` service; workflow-trace is a passive reader and runs no migration logic on its side. Second, **typed enum reflection** — raw string columns `chain_type` and `consent` are mapped to closed `ChainType` / `ConsentLevel` enums at parse time, so downstream modules cannot drift into stringly-typed handling. Third, **preserve-list discipline (AP-Hab-04)** — `consent = 'Forget'` rows are filtered at the SQL query layer; m3 never emits them downstream. This is the read-side mirror of the habitat's preserve-list rule.

Frame violations m3 must structurally refuse: (a) treating `chain.label` as natural-language user intent rather than an opaque habitat tag (Watcher Class-G — `label` is a stable identifier for cross-session correlation, not human prose for the engine to reason about); (b) any write back to injection.db, even diagnostic — m3 is one-way; (c) silent migration when injection.db schema version changes — m3 must fail loudly on schema mismatch (`PRAGMA user_version` mismatch surfaces as `QueryFailed` or `RowParseFailed`, never silent best-effort).

m3 does not own any CC contract surface; it is a pure data source. CC-1 join semantics close at m7 (the central hub joins m3's rows with m4's cascade rows). CC-3 closure target — `chain_id` from m3 surfaces in m14's evidence layer (workflow patterns that co-occur with persistent unresolved chains are higher-signal evidence).

## 2. Public surface (Rust types — spec only, NOT compileable)

```rust
//! # m3_injection_db_consumer
//!
//! - **Layer**: L1 (Substrate Ingest, Cluster A)
//! - **Deps**: rusqlite (read-only WAL), workflow_core::types::{SessionId}, workflow_core::errors::IngestError
//! - **Tests**: 50 (25 unit + 5 property + 15 integration + 3 contract + 2 regression + mutation ≥70%)
//! - **Features**: none
//! - **Platform**: Linux; injection.db path `~/.local/share/habitat/injection.db`
//! - **Impl Notes**: ~65 LOC lifted from `memory-injection/m06_schema.rs` (open + WAL pragmas) and `memory-injection/m07_causal_chain.rs` (CausalChainRow + parse_row + find_unresolved query). ~15 LOC novel — typed `ChainType` / `ConsentLevel` enums + parse functions + workflow-trace-local error domain. No migration logic; `PRAGMA query_only = ON` enforced.
//! - **Related Docs**: [cluster-A vault spec](../../../the-workflow-engine-vault/module%20specs/cluster-A-substrate-ingest.md) · [V7 cluster-A](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-A.md) · [ULTRAMAP](../../../ai_docs/optimisation-v7/ULTRAMAP.md) View 2 row m3 · [memory-injection schema](../../../memory-injection/EXECUTION_PLAN.md)

#[derive(Debug, Clone)]
pub struct InjectionDbConfig {
    pub db_path: std::path::PathBuf,
    /// Bounded [100, 5_000]; default 500 (per V7 cluster-A plan).
    pub max_unresolved: usize,
    pub max_recently_resolved: usize,
    /// "Recently resolved" cutoff = max resolved_session minus this; default 10.
    pub resolved_recency_sessions: u32,
}

impl Default for InjectionDbConfig { /* ... */ }

/// Workflow-trace-local mirror of injection.db's causal_chain row.
/// Distinct type — workflow-trace never imports memory-injection's struct.
#[derive(Debug, Clone)]
pub struct CausalChainRow {
    pub id: ChainId,
    pub origin_session: u32,
    pub resolved_session: Option<u32>,
    pub chain_type: ChainType,
    pub label: ChainLabel,
    pub description: String,
    pub reinforcement_count: u32,
    pub last_reinforced_session: Option<u32>,
    pub consent: ConsentLevel,
}

/// Newtypes for primary key + opaque label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChainId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChainLabel(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainType {
    Bug,
    Trap,
    Plan,
    Pattern,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsentLevel {
    Emit,
    Store,
    Forget,
}

#[derive(Debug, thiserror::Error)]
pub enum InjectionDbError {
    #[error("database open failed at {path}: {reason}")]
    DatabaseOpenFailed { path: std::path::PathBuf, reason: String },
    #[error("query failed: {0}")]
    QueryFailed(String),
    #[error("row parse error at id={row_id}: {reason}")]
    RowParseFailed { row_id: i64, reason: String },
    #[error("unknown chain_type value: {0}")]
    UnknownChainType(String),
    #[error("unknown consent value: {0}")]
    UnknownConsent(String),
}

/// Open injection.db read-only; no migration.
///
/// # Errors
/// `DatabaseOpenFailed` if file is missing or unopenable.
pub fn open_readonly(
    config: &InjectionDbConfig,
) -> Result<InjectionDbConsumer, InjectionDbError>;

pub struct InjectionDbConsumer { /* private */ }

impl InjectionDbConsumer {
    /// Read unresolved chains ordered by `reinforcement_count DESC`;
    /// excludes `consent = 'Forget'`; limited to `config.max_unresolved`.
    pub fn read_unresolved(&self) -> Result<Vec<CausalChainRow>, InjectionDbError>;

    /// Read recently-resolved chains within `config.resolved_recency_sessions`
    /// of `MAX(resolved_session)`; excludes `Forget`; limited to
    /// `config.max_recently_resolved`.
    pub fn read_recently_resolved(&self) -> Result<Vec<CausalChainRow>, InjectionDbError>;

    /// Scalar count of unresolved chains; m12 uses for CLI summary.
    pub fn count_unresolved(&self) -> Result<u64, InjectionDbError>;
}
```

## 3. Internal data structures

```rust
struct ConsumerInner {
    conn: rusqlite::Connection,
    config: InjectionDbConfig,
}

fn parse_causal_chain_row(row: &rusqlite::Row<'_>) -> Result<CausalChainRow, InjectionDbError>;
fn parse_chain_type(s: &str) -> Result<ChainType, InjectionDbError>;
fn parse_consent(s: &str) -> Result<ConsentLevel, InjectionDbError>;
```

`parse_chain_type` and `parse_consent` are the closed-set translators that prevent stringly-typed drift; `UnknownChainType` and `UnknownConsent` preserve the bad value for diagnostics (data-driven variant pattern, mirror of ME v2 `error.rs`).

## 4. Data flow

- **INPUT FROM:** `~/.local/share/habitat/injection.db` (SQLite WAL, read-only URI, `PRAGMA query_only = ON`)
- **OUTPUT TO:**
  - **m7 `workflow_runs`** (Cluster C): `Vec<CausalChainRow>` as supplementary lifecycle context (unresolved chains anchor workflow boundaries; CC-1 join)
  - **m4 `cascade_correlator`** (Cluster B): chain rows anchor cascade boundaries
  - **m5 `battern_step_record`** (Cluster B): chain labels seed battern step labels
  - **m12 `cli_reports`** (Cluster C): `count_unresolved()` scalar surfaces in CLI summary output
- **SUBSTRATE TOUCHED:** injection.db only
- **WRITES:** none (Phase A read-only invariant)

## 5. Algorithm sketch

```text
open_readonly(config):
    path = config.db_path
    conn = rusqlite::Connection::open_with_flags(uri(path, ro), READ_ONLY)
    configure_connection(&conn)            // lift from m06_schema.rs + add query_only
    return InjectionDbConsumer { inner: { conn, config } }

read_unresolved(&self):
    stmt = "SELECT id, origin_session, resolved_session, chain_type, label,
                   description, reinforcement_count, last_reinforced_session, consent
            FROM causal_chain
            WHERE resolved_session IS NULL
              AND consent NOT IN ('Forget')
            ORDER BY reinforcement_count DESC
            LIMIT ?1"
    rows = stmt.query_map([clamp(config.max_unresolved, 100, 5000)], parse_causal_chain_row)
    return rows.collect::<Result<Vec<_>, _>>()

read_recently_resolved(&self):
    max_resolved = "SELECT COALESCE(MAX(resolved_session), 0) FROM causal_chain"
    cutoff = max_resolved.saturating_sub(config.resolved_recency_sessions)
    stmt = "... WHERE resolved_session IS NOT NULL
                  AND resolved_session > ?1
                  AND consent NOT IN ('Forget')
            ORDER BY resolved_session DESC
            LIMIT ?2"

count_unresolved(&self):
    "SELECT COUNT(*) FROM causal_chain WHERE resolved_session IS NULL"
```

## 6. Boilerplate lifts

| Source | Lift | % |
|---|---|---:|
| `memory-injection/m06_schema.rs::configure_connection` | Full WAL pragma batch | 90% (add `query_only = ON`) |
| `memory-injection/m06_schema.rs::open_database` | Path-existence check, parent-dir creation, `Connection::open` typed-error mapping | 85% (strip `create_all_tables` / `migrate`; read-only URI; do not touch `user_version`) |
| `memory-injection/m07_causal_chain.rs::CausalChainRow` | Field list (id / origin_session / resolved_session / chain_type / label / description / reinforcement_count / last_reinforced_session / consent) | 70% (rename to workflow-trace-local type; replace raw String `chain_type` / `consent` with typed enums) |
| `memory-injection/m07_causal_chain.rs::parse_row()` | `row.get::<_, T>(N)?` column extraction pattern | 75% (map string columns through `parse_chain_type` / `parse_consent`; return `RowParseFailed` not `rusqlite::Error`) |
| `memory-injection/m07_causal_chain.rs::find_unresolved()` | `SELECT ... WHERE resolved_session IS NULL ORDER BY reinforcement_count DESC LIMIT ?` | 80% (add `consent NOT IN ('Forget')` preserve-filter; parametrise limit) |
| `memory-injection/m11_parallel_query.rs` timing harness | Per-query `Instant::now() + elapsed_ms` | 60% (emit to tracing span; not a `QueryResult<T>` wrapper) |

Net: ~50 LOC lifted / ~20 LOC novel (typed enums + parse functions + `read_recently_resolved` recency-window query + `count_unresolved` scalar).

## 7. ME v2 m1_foundation patterns referenced

- `error.rs` — `InjectionDbError` mirrors the structured-field discipline; `UnknownChainType(String)` and `UnknownConsent(String)` preserve the bad value (data-driven variant pattern).
- `shared_types.rs` — `ChainType` / `ConsentLevel` are exhaustive value-type enums; no `Default` (no meaningful default discriminant); `ChainId`/`ChainLabel` are newtypes preventing primitive obsession.
- `resources.rs` — `//!` docstring block (Layer/Deps/Tests/Features/Platform/Impl Notes/Related Docs).
- `logging.rs` — tracing-subscriber emit on every `read_unresolved` / `read_recently_resolved` (count + elapsed_ms).

## 8. Test strategy

- **Test kind**: unit (25) + property (5) + integration (15) + contract (3) + regression (2)
- **Test count**: 50 minimum (per [TEST_DISCIPLINE matrix](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) row m3)
- **Mutation budget**: ≥70%
- **Properties tested** (F-Property 5):
  - Returned rows ≤ `max_unresolved` (limit invariant)
  - Ordering: `reinforcement_count` monotonically non-increasing across returned vec
  - Consent filter: no returned row has `consent == ConsentLevel::Forget`
  - All returned `read_unresolved` rows have `resolved_session.is_none()`
  - `count_unresolved` ≥ `read_unresolved(max=usize::MAX).len()` for tables < 10k rows

Key invariants (sample of 50; full list in vault cluster-A spec § Tests m3):

1. `open_readonly` returns `DatabaseOpenFailed` when path missing
2. `PRAGMA query_only = ON` rejects INSERT
3. `read_unresolved` returns empty `Vec` on empty table
4. `read_unresolved` orders by `reinforcement_count DESC`
5. `read_unresolved` respects `max_unresolved` limit
6. `read_unresolved` excludes `consent = 'Forget'`
7. `ChainType::Bug` parses from `"bug"`
8. `ChainType::Trap` parses from `"trap"`
9. `UnknownChainType` returned for unrecognised value
10. `ConsentLevel::Emit` parses from `"Emit"`
11. `UnknownConsent` returned for unrecognised value
12. `CausalChainRow.resolved_session` is `None` for unresolved row
13. `count_unresolved` returns 0 on empty table
14. `read_recently_resolved` filters to within `resolved_recency_sessions` of max
15. `read_unresolved` query uses partial index `idx_causal_unresolved` (verify via `EXPLAIN QUERY PLAN`)

(remaining 35 enumerated in vault spec)

## 9. Antipatterns to avoid

- **AP-Hab-04** (preserve-list discipline) — `consent NOT IN ('Forget')` is the SQL-level preserve filter; never enumerate-and-bulk-process
- **AP-Drift-05** (migration "applied" but schema unchanged) — m3 never migrates injection.db; mitigated by construction
- **AP-WT-F3** (substrate-input poisoning) — read-only enforcement at URI + pragma layers; m9 validates row structure pre-emit downstream
- **AP-V7-09** (substrate-frame engine confusion) — `ChainLabel` is opaque; m3 does not parse it for natural-language meaning
- **AP30** (namespace string drift) — `ChainType` enum prevents stringly-typed `chain_type` drift; closed set rejected at parse
- **Newly surfaced**: scalar `count_unresolved` must not run on a separate connection — single connection ensures consistent snapshot semantics

## 10. Useful patterns applied

- thiserror error enums (GOLD_STANDARDS rule 9)
- Newtype + value-type enum discipline (GOLD_STANDARDS rule 8)
- Closed-set parse functions for string→enum coercion (data-driven error variants)
- WAL pragma batch with `query_only = ON` defense-in-depth
- `//!` docstring block (GOLD_STANDARDS rule 13)

## 11. Cross-cluster contracts

- **CC-1 (consumes via m7 join)**: m3's `CausalChainRow.id` (newtype `ChainId`) is the join column that m7 uses to attach workflow_runs to open habitat chains. m3 must guarantee `ChainId` byte-stability across reads of the same row.
- **CC-3 (consumes via E→F)**: `ChainId` and `ChainLabel` propagate through m7 → m14 evidence layer so workflow patterns that co-occur with persistent unresolved chains carry higher signal weight.
- **CC-7 closure target (consumes config feedback)**: per V7 cluster-A plan, `m3.config.limit` (`max_unresolved`) is one of three Cluster A config fields targeted by m15's pressure register for spec-amendment feedback.

## 12. Open questions for G5 interview / Zen G7 audit

1. **Preserve-filter layer**: SQL-level (current spec — more efficient) or application-layer (more testable)? Couples m3 to consent semantics vs leaves them in m9 / pre-emit validator.
2. **Phase A scope**: does m7 need both unresolved AND recently-resolved at genesis, or only unresolved? If only unresolved, `read_recently_resolved` deferred to a v1.4 amendment.
3. **Polling vs snapshot**: read once at startup (snapshot) or periodically (continuous)? Continuous needs polling loop or WAL filesystem-watch; impacts daemon-mode design.
4. **`ChainType` exhaustiveness**: are the four variants (Bug / Trap / Plan / Pattern) the complete set, or will injection.db's schema drift add more? `UnknownChainType` catches drift but downstream modules must handle the variant set explicitly.

## 13. Implementation order (post-G9)

1. `error.rs` — `InjectionDbError` enum
2. `enums.rs` — `ChainType` + `ConsentLevel` + `parse_chain_type` + `parse_consent` + 6 unit tests (each variant + Unknown for both)
3. `causal_chain.rs` — `CausalChainRow` + `ChainId` + `ChainLabel` newtypes + `parse_causal_chain_row` + 8 unit tests
4. `config.rs` — `InjectionDbConfig` + `Default` + 3 unit tests (defaults + clamp + override resolution)
5. `query.rs` — `read_unresolved` + `read_recently_resolved` + `count_unresolved` + 8 unit tests
6. `mod.rs` — `open_readonly` + WAL pragma batch lift + `query_only` assertion + 5 unit tests
7. Property tests (5) — proptest on limit invariant + ordering + consent filter
8. Integration tests (15) — `tests/m3_integration.rs` with fixture `tests/fixtures/injection-mini.db` carrying seeded chains
9. Contract tests (3) — insta snapshots for `CausalChainRow` + `ChainType` + `ConsentLevel` schema vs injection.db v5
10. Regression slots (2) — reserved (e.g., `Forget` consent leak, recency cutoff overflow)
11. Mutation pass — `cargo mutants` on `enums.rs` + `query.rs`; ≥70% kill required

---

> Back to: [vault cluster-A spec](../../../the-workflow-engine-vault/module%20specs/cluster-A-substrate-ingest.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · sister modules: [m1](m1_atuin_consumer.md) · [m2](m2_stcortex_consumer.md) · [m3](m3_injection_db_consumer.md)
