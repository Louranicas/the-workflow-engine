---
title: Cluster A — Substrate Ingest (m1, m2, m3) — Module Specs
date: 2026-05-17 (S1001982)
kind: detailed-module-spec
status: planning-only · HOLD-v2 active · NOT source code
authority: Luke @ node 0.A direct directive
---

# Cluster A — Substrate Ingest

> Back to: [[../HOME]] · [[../MASTER_INDEX]] · [[../workflow-engine-code-base]] · [[../Modules Synergy Clusters and Feature Verification S1001982]]

## Cluster overview

Cluster A is the data-intake boundary of the entire workflow-trace codebase. Its sole responsibility is passive substrate reading: tool-call history from atuin's SQLite, consumption events from SpacetimeDB via a narrowed stcortex consumer registration, and resolved/unresolved chains from injection.db's `causal_chain` table. Nothing in Cluster A writes, proposes, selects, or routes. Phase A verbs apply without exception: **ingest · correlate · record · emit · refuse.**

Intra-cluster coupling is deliberately absent. Each module reads a different substrate and emits typed row iterators into the downstream pipeline (Clusters B and C). The only woven synergy is that m2's `register_consumer` call — a side-effect of startup, not a write to domain data — produces a trust signal relied on by the write-gating invariants in Cluster D (m9 `watcher_namespace_guard`) and later by m13 `stcortex_writer_narrowed`. No row produced by m1, m2, or m3 is interpreted, filtered for relevance, or labelled inside this cluster. Interpretation begins at Cluster B.

---

## Cross-cluster contracts (in/out)

- **INPUT FROM:** none — Cluster A is L1, the read boundary. It has no upstream module dependency within workflow-trace.
- **OUTPUT TO:**
  - **Cluster B** (m4, m5, m6) receives `ToolCallRow` iterators from m1 for cascade correlation and context-cost pairing. m5 and m6 read atuin session-id ranges via m1's pagination interface.
  - **Cluster C** (m7, m12, m13) — m3 `CausalChainRow` output feeds m7 `workflow_arc_record` as supplementary lifecycle context. m13 `stcortex_writer_narrowed` requires that m2 registration is complete before any write attempt is made.
  - **Cluster D** (m8, m9, m10, m11) — m2 registration success is the runtime trust signal confirming namespace `workflow_trace_*` is live for subsequent write-gate enforcement in m9.

---

## m1 — `atuin_ingest`

### Purpose

`atuin_ingest` reads atuin's local SQLite history database (`~/.local/share/atuin/history.db`) and surfaces tool-call rows as a lazy, cursor-based iterator. With approximately 263,000 rows in the habitat's atuin history, a naive full-table scan at startup would be prohibitive. This module therefore implements cursor-based pagination from scratch (there is no reusable equivalent in any boilerplate source; see Boilerplate Hunt gap analysis). A graceful subprocess-based fallback wraps the `atuin` CLI binary for cases where the SQLite path is unavailable or WAL lock conflicts exceed the busy-timeout threshold.

The module is strictly read-only. It touches no atuin state whatsoever — no `INSERT`, no `UPDATE`, no schema migrations. WAL mode is respected; the connection is opened in read-only mode with a 5,000 ms busy-timeout to absorb concurrent atuin writes without blocking the workflow-trace scan.

### Public surface

```rust
/// A single row from atuin's history table, normalised for workflow-trace consumption.
/// Fields are typed: no raw `String`-for-everything antipattern.
#[derive(Debug, Clone)]
pub struct ToolCallRow {
    /// atuin's internal integer row ID — used as pagination cursor.
    pub id: i64,
    /// The full command text as recorded by atuin.
    pub command: String,
    /// atuin session UUID grouping related commands within one shell session.
    pub session: String,
    /// atuin hostname to disambiguate multi-machine histories.
    pub hostname: String,
    /// Unix timestamp (milliseconds) of command start.
    pub timestamp_ms: i64,
    /// Exit code; `None` if atuin did not record it.
    pub exit: Option<i32>,
    /// Elapsed time in milliseconds; `None` if unavailable.
    pub duration_ms: Option<i64>,
    /// atuin working directory at time of invocation; `None` if absent.
    pub cwd: Option<String>,
}

/// Configuration controlling pagination and fallback behaviour.
#[derive(Debug, Clone)]
pub struct AtuinIngestConfig {
    /// Maximum rows per paginated query. Default: 2_000.
    pub page_size: usize,
    /// Maximum rows to ingest in total across all pages. `None` = no limit.
    pub row_cap: Option<usize>,
    /// Timeout for the subprocess fallback path in milliseconds. Default: 5_000.
    pub subprocess_timeout_ms: u64,
    /// Override path to atuin.db; if `None`, resolves `~/.local/share/atuin/history.db`.
    pub db_path_override: Option<std::path::PathBuf>,
}

impl Default for AtuinIngestConfig { ... }

/// Errors specific to atuin ingestion.
#[derive(Debug, thiserror::Error)]
pub enum AtuinIngestError {
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

/// Open atuin's history database for reading and return a paginating ingestor.
///
/// Configures WAL read-only mode, sets `PRAGMA busy_timeout = 5000`, and
/// applies `PRAGMA query_only = ON` to prevent accidental writes at the
/// SQLite layer. Leaves WAL checkpointing to atuin itself.
///
/// # Errors
///
/// Returns `AtuinIngestError::DatabaseOpenFailed` if the file does not exist
/// or cannot be opened. Returns `AtuinIngestError::BusyTimeout` if the WAL
/// lock is held for longer than `config.subprocess_timeout_ms`.
pub fn open_atuin_db(config: &AtuinIngestConfig) -> Result<AtuinIngestor, AtuinIngestError>;

/// Paginating ingestor over atuin history rows.
pub struct AtuinIngestor { /* private */ }

impl AtuinIngestor {
    /// Return the next page of `ToolCallRow` values ordered by `id ASC`.
    /// Returns `Ok(None)` when the cursor has exhausted all rows.
    /// The cursor is a plain `i64` row ID — stored internally, not exposed.
    ///
    /// # Errors
    ///
    /// Returns `AtuinIngestError::QueryFailed` on SQLite error.
    pub fn next_page(&mut self) -> Result<Option<Vec<ToolCallRow>>, AtuinIngestError>;

    /// Consume all pages into a single `Vec`. Convenience for tests and small histories.
    /// Respects `config.row_cap`.
    pub fn collect_all(self) -> Result<Vec<ToolCallRow>, AtuinIngestError>;

    /// Returns an iterator adaptor that yields pages lazily.
    pub fn into_page_iter(self) -> impl Iterator<Item = Result<Vec<ToolCallRow>, AtuinIngestError>>;
}

/// Subprocess fallback: invoke `atuin history list --limit N --format json`.
/// Used when SQLite open fails due to path absence or persistent WAL lock.
///
/// # Errors
///
/// Returns `AtuinIngestError::SubprocessFailed` on timeout or non-zero exit.
pub fn fallback_subprocess_ingest(
    config: &AtuinIngestConfig,
) -> Result<Vec<ToolCallRow>, AtuinIngestError>;
```

### Internal data structures

```rust
/// Cursor state kept inside `AtuinIngestor`. Not public.
struct IngestorState {
    conn: rusqlite::Connection,
    last_id: i64,           // Exclusive lower bound for next query.
    rows_yielded: usize,    // Running total; compared against row_cap.
    exhausted: bool,
    config: AtuinIngestConfig,
}
```

### Data flow

- **READS:** `~/.local/share/atuin/history.db` (SQLite WAL, read-only, PRAGMA query_only = ON)
- **EMITS:** `Iterator<Item = Result<Vec<ToolCallRow>, AtuinIngestError>>` consumed by m4 `cascade_correlator`, m5 `battern_step_record`, and m6 `context_cost_record`
- **WRITES:** none (Phase A read-only invariant)
- **SUBPROCESS PATH:** `atuin history list` via `std::process::Command` with `subprocess_timeout_ms` deadline

### Boilerplate lifts (with reuse %)

| Source file | What is lifted | Reuse % | What is adapted or authored fresh |
|---|---|---|---|
| `memory-injection/m06_schema.rs` — `configure_connection()` | WAL pragma batch: `journal_mode=WAL; busy_timeout=5000; foreign_keys=ON; synchronous=NORMAL; wal_autocheckpoint=100` | ~90% | Add `PRAGMA query_only = ON` (write prevention at SQLite layer, not present in injection-side open) |
| `memory-injection/m06_schema.rs` — `open_database()` | Parent-dir creation guard; `path.exists()` branch; `Connection::open()` error mapping into typed error enum | ~85% | Remove migration logic entirely (atuin schema is external; workflow-trace never migrates it); open as read-only URI |
| `memory-injection/m11_parallel_query.rs` — `QueryResult<T>` + timing harness | `elapsed_ms` + staleness annotation per page | ~80% | Rename to `PageResult`; parametrize over `Vec<ToolCallRow>` instead of injection-domain structs; adjust staleness threshold to 200ms (larger pages) |
| `memory-injection/m18_atuin_cache.rs` — graceful-degrade subprocess wrapper | `Option<T>` return, `std::process::Command` with timeout, parse-or-None pattern | ~70% | Abstract into `FallbackIngest<T>` trait; replace hardcoded JSON parsing with atuin's `--format json` field mapping into `ToolCallRow`; add structured error instead of `Option` |
| **Cursor-based pagination** | n/a — no boilerplate equivalent exists | 0% (novel) | Implement from scratch: `SELECT * FROM history WHERE id > ? ORDER BY id ASC LIMIT ?`; maintain `last_id: i64` cursor internally; accumulate `rows_yielded` for cap enforcement (~30 LOC) |

The pagination implementation is the Boilerplate Hunt's flagged absence. The cursor pattern is simple but must be authored: a prepared statement with two bind parameters (`last_id`, `page_size`), a mutable cursor state, and exhaustion detection when the returned page is shorter than `page_size`.

### ME v2 foundation patterns referenced

- `m1_foundation/error.rs` — `thiserror::Error` enum pattern with structured variant fields (`{ path, reason }` shape). `AtuinIngestError` mirrors this: each variant carries named fields rather than opaque strings so callers can match on them programmatically.
- `m1_foundation/resources.rs` — `//!` module docstring block with explicit sections: Layer, Dependencies, Tests (target count), Features (capability list), Platform Support (table), Implementation Notes (zero-unsafe commitment), Related Documentation (wikilink). `m1`'s docstring adopts this shape verbatim.
- `m1_foundation/shared_types.rs` — newtype discipline. `ToolCallRow` wraps primitive fields (e.g., `session: String`) rather than exposing raw SQLite column indices. The `AtuinIngestConfig` struct follows the `Default + env override` pattern seen in `m03_config.rs` in the boilerplate 10-foundation-direct-clones.

### Constraints satisfied

- **W1** (narrowed scope — read-only) — `PRAGMA query_only = ON` enforced at SQLite layer; `open_database` opens in read-only URI mode.
- **F8** (no feedback-loop poisoning) — m1 never writes to any stcortex namespace; it is not a stcortex consumer. Reads are from atuin only.
- **Core measurement** — atuin history is the ground-truth source for tool-call sequences; without m1 no cascade correlation (m4) or cost pairing (m6) is possible.

### Tests (50+ minimum)

Key invariants to verify:

1. `open_atuin_db` returns `DatabaseOpenFailed` when path does not exist
2. `open_atuin_db` returns `Ok` on a fresh in-memory schema (test helper creates atuin-schema-compatible db)
3. `configure_connection` sets `query_only = ON` — verify with `PRAGMA query_only`
4. `configure_connection` sets `busy_timeout = 5000`
5. `configure_connection` sets `journal_mode = WAL` (or `memory` in-memory fallback)
6. Pagination cursor starts at `id = 0`
7. Pagination cursor is monotonically increasing across pages
8. `next_page` returns `None` when table is empty
9. `next_page` returns `None` after single partial page (fewer rows than `page_size`)
10. `next_page` returns correct rows in `id ASC` order across two pages
11. `collect_all` aggregates all pages into a single `Vec`
12. `row_cap = Some(5)` stops ingestion after 5 rows regardless of table size
13. `row_cap = None` does not cap ingestion
14. `rows_yielded` accumulates correctly across multiple `next_page` calls
15. `into_page_iter` produces the same rows as `collect_all`
16. Empty page terminates the iterator without error
17. `ToolCallRow.id` field matches the SQLite `id` column
18. `ToolCallRow.command` is non-empty for seeded rows
19. `ToolCallRow.exit` is `None` when column is NULL
20. `ToolCallRow.duration_ms` is `None` when column is NULL
21. `ToolCallRow.cwd` is `None` when column is NULL
22. `AtuinIngestConfig::default()` produces `page_size = 2_000`
23. `AtuinIngestConfig::default()` produces `subprocess_timeout_ms = 5_000`
24. `db_path_override` overrides default path resolution
25. WAL concurrency: two connections open simultaneously do not deadlock within `busy_timeout`
26. `BusyTimeout` error variant produced when lock exceeds threshold (inject a write-holding connection in test)
27. `QueryFailed` variant produced on schema mismatch
28. `RowParseFailed` variant produced when a column type does not match
29. Large table (1,000 seeded rows): `collect_all` returns exactly 1,000 rows
30. Large table: page boundaries produce no duplicates (row `id` uniqueness invariant)
31. Large table: `page_size = 100` requires `ceil(1000/100) = 10` pages
32. `fallback_subprocess_ingest` returns `SubprocessFailed` when binary is absent
33. `fallback_subprocess_ingest` respects `subprocess_timeout_ms` deadline
34. Parsed `ToolCallRow` from subprocess path matches expected field types
35. `AtuinIngestError` implements `std::error::Error`
36. `AtuinIngestError` implements `Display` with non-empty output for all variants
37. `ToolCallRow` implements `Clone` without panic
38. `ToolCallRow` implements `Debug` with non-empty output
39. `page_size = 1` processes 10-row table in 10 pages with no gaps
40. Cursor does not restart when called again after exhaustion (idempotent `None`)
41. `PRAGMA query_only = ON` prevents INSERT (verify error returned on attempted insert via same connection in test)
42. Rows with identical `timestamp_ms` are all returned (no deduplication occurs in m1)
43. Session UUID field round-trips through string without truncation
44. Hostname field round-trips through string without truncation
45. `open_atuin_db` succeeds when parent directory of `db_path_override` exists but file does not yet (graceful path — open creates it read-only, returns schema error rather than panic)
46. `open_atuin_db` creates parent directories when `db_path_override` specifies a nested path (lifted from `m06_schema.rs` `create_dir_all` guard)
47. Integration: seed 50 rows, `page_size = 20` → 3 pages of sizes 20, 20, 10; `collect_all` returns 50 rows
48. `into_page_iter` does not double-count when chained with `.flatten()`
49. `AtuinIngestConfig` with `row_cap = Some(0)` returns empty iterator immediately
50. `next_page` called after `row_cap` reached returns `Ok(None)`, not an error

### Open questions for G5 spec interview

- Q: Should `page_size` be configurable from CLI flags, TOML config, or fixed default only? (Cost/latency tradeoff — larger pages reduce query overhead; smaller pages reduce latency-to-first-row for streaming consumers)
- Q: Should the pagination cursor persist across CLI invocations (write last-id to daemon_state), or always restart from `id = 0`?
- Q: Does m4 `cascade_correlator` need the raw `command` field, or a pre-tokenised form? (Determines whether m1 should strip shell quoting/escaping before emitting)
- Q: What is the expected freshness window? atuin writes continuously; should m1 snapshot at open-time or allow live rows to appear mid-iteration?

### LOC estimate: ~80

- Lifted (~50 LOC): `configure_connection`, `open_database` shell, `QueryResult` timing harness, subprocess wrapper
- Novel authorship (~30 LOC): cursor-based pagination (`SELECT ... WHERE id > ? LIMIT ?`; cursor state struct; cap enforcement; exhaustion detection)

---

## m2 — `stcortex_consumer`

### Purpose

`stcortex_consumer` registers `workflow-trace` as a narrowed stcortex consumer at startup and maintains that registration across the session. "Narrowed" means the subscription covers only two stcortex tables: `tool_call` rows within the `workflow_trace_*` namespace, and `consumption_event` rows (the access-gradient signal). All other stcortex tables — `memory`, `pathway`, `ghost_memory`, `consumer` — are explicitly excluded from the subscription query set.

This narrowing is the module's central invariant (W1 in the feature matrix). The stcortex architecture enforces refuse-write at the database layer: if no fresh non-stale consumer is registered for a namespace, write reducers return a 530 error. m2 therefore also serves as the trust anchor for m13 `stcortex_writer_narrowed` — m13 must not attempt writes until m2's registration confirms the consumer is live. m2 exposes a `RegistrationHandle` that m13 reads to gate its writes.

m2 never writes domain data. It calls `register_consumer` (a reducer, not a write to domain tables) and subscribes to deltas. The `consumption_event` subscription is read-only: it surfaces access-gradient signals that flow to m14 `evidence_aggregator` as supplementary evidence. This satisfies F8 (no feedback-loop poisoning) because the consumer's own reads do not loop back through a write path — the consumption event is purely observational.

### Public surface

```rust
/// Consumer identity for this workflow-trace session.
pub struct ConsumerIdentity {
    /// Stable name: `"workflow-trace-<git-short-sha>"` for reproducibility.
    pub name: String,
    /// Namespace: always `"workflow_trace_*"` (AP30 collision-avoidance prefix).
    pub namespace: String,
    /// Transport: `"subscription"` — SpacetimeDB WebSocket SDK.
    pub transport: String,
}

impl ConsumerIdentity {
    /// Construct from the current process's git short SHA.
    /// Falls back to `"workflow-trace-unknown"` if `git` is unavailable.
    pub fn from_git_sha() -> Self;
}

/// A live stcortex consumer registration.
/// Callers hold this to confirm registration is active.
/// Dropping it does NOT unregister (unregister is explicit, on clean shutdown).
pub struct RegistrationHandle {
    identity: ConsumerIdentity,
    registered_at: std::time::Instant,
    // SpacetimeDB SDK connection — private.
}

impl RegistrationHandle {
    /// Returns `true` if the registration is fresh (< 30 days by stcortex's own definition).
    pub fn is_fresh(&self) -> bool;

    /// Returns the consumer identity.
    pub fn identity(&self) -> &ConsumerIdentity;

    /// Explicitly unregister this consumer (idempotent, call on shutdown).
    ///
    /// # Errors
    ///
    /// Returns `StcortexConsumerError::UnregisterFailed` if the reducer call fails.
    pub fn unregister(self) -> Result<(), StcortexConsumerError>;
}

/// Errors from consumer registration.
#[derive(Debug, thiserror::Error)]
pub enum StcortexConsumerError {
    #[error("connection to stcortex at {uri} failed: {reason}")]
    ConnectionFailed { uri: String, reason: String },
    #[error("register_consumer reducer failed: {0}")]
    RegisterFailed(String),
    #[error("subscription apply timed out after {timeout_ms}ms")]
    SubscriptionTimeout { timeout_ms: u64 },
    #[error("unregister_consumer reducer failed: {0}")]
    UnregisterFailed(String),
    #[error("namespace validation failed: {0}")]
    InvalidNamespace(String),
}

/// Register as a narrowed stcortex consumer.
///
/// Connects to `ws://127.0.0.1:3000`, calls `register_consumer`, subscribes
/// to only two queries:
///   - `SELECT * FROM tool_call WHERE namespace = 'workflow_trace_*'`
///   - `SELECT * FROM consumption_event`
///
/// Waits up to `timeout_ms` for the subscription to apply before returning.
///
/// # Refuse-write at DB layer
///
/// After successful return, stcortex will accept writes to the `workflow_trace_*`
/// namespace. If this function is not called, writes from m13 will be rejected
/// with HTTP 530. m13 checks `RegistrationHandle::is_fresh()` before each write.
///
/// # Errors
///
/// See `StcortexConsumerError` variants.
pub fn register_narrowed_consumer(
    identity: ConsumerIdentity,
    timeout_ms: u64,
) -> Result<RegistrationHandle, StcortexConsumerError>;

/// Snapshot of a single `consumption_event` row received via subscription delta.
#[derive(Debug, Clone)]
pub struct ConsumptionEventRow {
    pub memory_id: u64,
    pub consumer_name: String,
    pub consumed_at_ms: i64,
}

/// Callback type for consumption event deltas.
/// Called by the SDK on every INSERT to `consumption_event` within the subscription.
pub type ConsumptionEventCallback = Box<dyn Fn(ConsumptionEventRow) + Send + Sync>;
```

### Internal data structures

```rust
/// Internal state managed by the SDK run loop. Not public.
struct ConsumerState {
    conn: module_bindings::DbConnection, // SpacetimeDB SDK type
    identity: ConsumerIdentity,
    registered_at: std::time::Instant,
    subscription_applied: std::sync::atomic::AtomicBool,
    // Callback storage for consumption_event deltas.
    on_consumption_event: Option<ConsumptionEventCallback>,
}
```

### Data flow

- **READS:** SpacetimeDB WebSocket delta stream from `ws://127.0.0.1:3000`, limited to `tool_call WHERE namespace = 'workflow_trace_*'` and `consumption_event`
- **EMITS:** `RegistrationHandle` to m13 `stcortex_writer_narrowed` (trust gate); `ConsumptionEventRow` delta stream to m14 `evidence_aggregator`
- **WRITES:** none — `register_consumer` is a reducer call, not a write to domain tables. The DB-layer refuse-write invariant lives in stcortex, not here.
- **SIDE-EFFECT:** consumer row created in stcortex's `consumer` table (stcortex manages this; workflow-trace does not own the schema)

### Boilerplate lifts (with reuse %)

| Source file | What is lifted | Reuse % | What is adapted |
|---|---|---|---|
| `stcortex/clients/rust-subscriber/src/main.rs` — `DbConnection::builder()` block | `.with_uri()`, `.with_database_name()`, `.on_connect()` callback registration, `.on_connect_error()`, `.on_disconnect()`, `.build()?`, `.run_threaded()` | ~80% | Strip all `pathway` + `memory` on_insert/on_update/on_delete handlers (W1 narrowing); keep only `tool_call` + `consumption_event` handlers; replace panic/eprintln error paths with `StcortexConsumerError` returns |
| `stcortex/clients/rust-subscriber/src/main.rs` — subscription query strings | `format!("SELECT * FROM ... WHERE namespace = '{}'", ns)` pattern | ~85% | Two queries only (tool_call + consumption_event); hardcode namespace to `workflow_trace_*` |
| `stcortex/clients/rust-subscriber/src/capacity.rs` — `wait_count` pattern | `rx.recv_timeout(Duration::from_secs(N))` pattern for subscription-applied confirmation | ~90% | Use `mpsc::channel::<()>` with `on_applied` callback sending `()` to confirm subscription applied; expose as configurable `timeout_ms` |
| `stcortex/clients/rust-subscriber/src/capacity.rs` — `AtomicBool` state pattern | `AtomicBool` + `Ordering::Relaxed` for cross-thread subscription state | ~90% | Apply to `subscription_applied` flag in `ConsumerState` |
| `stcortex/docs/CONSUMER-ONBOARDING.md` — refuse-write reducer template | Conceptual: `if consumer_count == 0 && namespace != "scratch" { return Err(...) }` | Reference | Confirms m2 must call `register_consumer` before m13 attempts any write; documented in m2's module docstring as architectural contract |
| `stcortex/docs/stcortex_API.md` | `register_consumer / access_memory` API signatures | Reference | `transport` field: use `"subscription"` (SDK path) not `"cli"` |

The key adaptation relative to `stcortex_subscriber_main.rs`: that reference subscribes to `pathway` + `memory` tables across all namespaces. m2 subscribes to exactly two narrowed queries. The handler wiring in `on_connect` is substantially simpler.

### ME v2 foundation patterns referenced

- `m1_foundation/error.rs` — `StcortexConsumerError` mirrors the `{field, reason}` variant structure. Each variant is distinct and matchable: callers can distinguish `ConnectionFailed` (retry candidate) from `RegisterFailed` (configuration problem) from `SubscriptionTimeout` (transient).
- `m1_foundation/shared_types.rs` — `ConsumerIdentity` is a value type (no methods that allocate on every call); `from_git_sha()` is a named constructor following the `must_use` pattern.
- `m1_foundation/resources.rs` — module docstring block: Layer (L1 Substrate Ingest), Dependencies (SpacetimeDB SDK, stcortex module bindings), Tests (50 target), Features (narrowed subscription, refuse-write enforcement, trust-signal emission), Platform Support (Linux primary; stcortex `:3000` required), Implementation Notes (W1 narrowing enforced at subscription query level, not at application layer), Related Documentation.

### Constraints satisfied

- **W1** (narrowed scope) — subscription queries are hardcoded to two tables; all other stcortex tables are outside the query window.
- **F8** (no feedback-loop poisoning) — m2 is a read-only consumer of stcortex deltas. Consumption events observed via the subscription do not trigger any write back into stcortex from m2. m13 owns writes; m2 owns the trust gate only.
- **AP30** (namespace collision avoidance) — namespace is `workflow_trace_*`; m2 validates this at construction time and returns `InvalidNamespace` if caller attempts a different prefix.
- **Feature #1** (W1 narrowed-scope consumer) — m2 is one of two modules that own this feature; m9 owns the write-time enforcement complement.

### Tests (50+ minimum)

Key invariants to verify:

1. `ConsumerIdentity::from_git_sha()` returns a name starting with `"workflow-trace-"`
2. `ConsumerIdentity::from_git_sha()` returns `"workflow-trace-unknown"` when `git` binary absent
3. `ConsumerIdentity` namespace is always `"workflow_trace_*"` — constructor enforces this
4. `StcortexConsumerError::InvalidNamespace` returned when namespace does not start with `workflow_trace_`
5. `register_narrowed_consumer` returns `ConnectionFailed` when stcortex is not reachable (no server on `:3000`)
6. `RegistrationHandle::is_fresh()` returns `true` immediately after registration
7. `RegistrationHandle::identity()` returns the same identity passed to `register_narrowed_consumer`
8. `unregister` is idempotent — calling twice does not panic
9. Subscription query string for `tool_call` contains `namespace = 'workflow_trace_*'`
10. Subscription query string does NOT include `pathway`, `memory`, or `ghost_memory`
11. `SubscriptionTimeout` returned when stcortex does not call `on_applied` within `timeout_ms`
12. `on_connect` callback calls `register_consumer` reducer exactly once
13. `register_consumer` call uses `transport = "subscription"`
14. `ConsumptionEventRow.memory_id` is populated from delta insert
15. `ConsumptionEventRow.consumer_name` matches the registered identity name
16. `ConsumptionEventRow.consumed_at_ms` is a positive integer
17. Multiple concurrent `register_narrowed_consumer` calls for the same identity are idempotent (stcortex call is idempotent per CONSUMER-ONBOARDING.md)
18. `RegistrationHandle` is `Send` (required for multi-threaded use by m13)
19. `StcortexConsumerError` implements `std::error::Error`
20. `StcortexConsumerError` implements `Display` for all variants
21. Namespace validation rejects empty string
22. Namespace validation rejects `"scratch"` (reserved by stcortex)
23. Namespace validation rejects `"claude-code"` (habitat-shared well-known name — must not be claimed by workflow-trace)
24. `RegistrationHandle::unregister` calls `unregister_consumer` reducer
25. `UnregisterFailed` returned when reducer call fails (simulate by connecting to mock server)
26. Consumer name is stable across two calls with the same git SHA (deterministic)
27. `timeout_ms = 1` produces `SubscriptionTimeout` in almost all test environments (fast failure path)
28. Subscription applied signal arrives before `register_narrowed_consumer` returns
29. `ConsumerIdentity.transport` is always `"subscription"` — not `"cli"` or `"polling"`
30. `RegisterFailed` variant produced when `register_consumer` reducer returns error (mock server)
31. Tool call rows in `tool_call` table with wrong namespace are NOT delivered via subscription
32. Tool call rows with namespace `workflow_trace_foo` ARE delivered (wildcard prefix match)
33. `consumption_event` rows ARE delivered regardless of which consumer generated them
34. `on_consumption_event` callback is invoked for every `consumption_event` INSERT
35. `RegistrationHandle` does not auto-unregister on `Drop` (stale GC handles it per onboarding doc)
36. Mock integration: `register_narrowed_consumer` → `unregister` round-trip succeeds
37. `ConsumerIdentity` is `Clone` + `Debug`
38. `RegistrationHandle::is_fresh()` returns `false` if simulated `last_read_at` is > 30 days (test via internal state injection)
39. `register_narrowed_consumer` with valid `timeout_ms = 5_000` returns within that window on a healthy stcortex instance
40. Error message for `ConnectionFailed` includes the URI string
41. Error message for `SubscriptionTimeout` includes the timeout value
42. `ConsumptionEventRow` implements `Clone` + `Debug`
43. Subscription delta for `tool_call` INSERT fires the correct on_insert handler
44. `RegistrationHandle::identity()` returns a reference (no clone allocation per call)
45. Two separate `ConsumerIdentity` values with different names compare unequal
46. `ConsumerIdentity::from_git_sha()` name length is bounded (< 64 chars to fit stcortex name column)
47. Consumer name passes `normalize_namespace`-equivalent validation (alphanumeric + `_-`, max 64)
48. `register_narrowed_consumer` does not panic when stcortex is reachable but subscription applied timeout fires
49. Module compiles with `#![forbid(unsafe_code)]` (inherited from workspace `workflow-core` crate root)
50. `StcortexConsumerError` is exhaustive — pattern match in tests without `_` arm to catch future variants early

### Open questions for G5 spec interview

- Q: Should `RegistrationHandle` re-register automatically if stcortex detects the consumer went stale (30-day no-read eviction)? Or is manual restart sufficient?
- Q: Should the `tool_call` subscription filter to only rows with `session_id` matching the current Claude session's UUID, or all rows in `workflow_trace_*` namespace?
- Q: Does m14 `evidence_aggregator` consume `ConsumptionEventRow` via callback or via a channel/queue? (Determines whether `ConsumptionEventCallback` is a `Box<dyn Fn>` or an `mpsc::Sender<ConsumptionEventRow>`)

### LOC estimate: ~80

- Lifted (~65 LOC): `DbConnection::builder()` block, subscription query construction, `AtomicBool` + `mpsc::channel` confirmation pattern, `on_connect_error` + `on_disconnect` handlers
- Novel authorship (~15 LOC): namespace validation, `ConsumerIdentity::from_git_sha()`, trust-gate `is_fresh()`, `RegistrationHandle` wrapper

---

## m3 — `injection_db_ingest`

### Purpose

`injection_db_ingest` reads the `causal_chain` table from injection.db (`~/.local/share/habitat/injection.db`). This table is the primary ledger of habitat-level bugs, traps, plans, and patterns with reinforcement-count tracking. workflow-trace ingests it to correlate long-lived chains (high `reinforcement_count`, unresolved) against tool-call sequences, producing evidence for the `workflow_arc_record` (m7) about which workflow patterns co-occur with persistent unresolved chains.

Like m1, m3 is strictly read-only. It does not write to injection.db, does not resolve chains, and does not create new rows. The injection.db schema is owned by the `memory-injection` service; workflow-trace is a passive reader. The `causal_chain` table schema is well-established and stable (schema version 5 per `m06_schema.rs`); no migration logic is needed on m3's side.

The module reads unresolved chains (where `resolved_session IS NULL`) ordered by `reinforcement_count DESC`, which surfaces the most-repeatedly-encountered issues first. It also reads recently resolved chains within a configurable recency window, so m7 can record co-occurrence patterns near resolution events.

### Public surface

```rust
/// Configuration for injection.db ingestion.
#[derive(Debug, Clone)]
pub struct InjectionDbConfig {
    /// Path to injection.db. Defaults to `~/.local/share/habitat/injection.db`.
    pub db_path: std::path::PathBuf,
    /// Maximum unresolved chains to read per ingest call. Default: 50.
    pub max_unresolved: usize,
    /// Maximum recently-resolved chains to read. Default: 20.
    pub max_recently_resolved: usize,
    /// "Recently resolved" cutoff: chains resolved within this many sessions
    /// back from the largest `resolved_session` value in the table. Default: 10.
    pub resolved_recency_sessions: u32,
}

impl Default for InjectionDbConfig { ... }

/// A causal chain row from injection.db, normalised for workflow-trace consumption.
/// Mirrors `memory-injection/m07_causal_chain.rs::CausalChainRow` but is
/// a separate type: workflow-trace does not import memory-injection.
#[derive(Debug, Clone)]
pub struct CausalChainRow {
    pub id: i64,
    pub origin_session: u32,
    pub resolved_session: Option<u32>,
    pub chain_type: ChainType,
    pub label: String,
    pub description: String,
    pub reinforcement_count: u32,
    pub last_reinforced_session: Option<u32>,
    pub consent: ConsentLevel,
}

/// Typed chain_type discriminant (replaces raw `String`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainType {
    Bug,
    Trap,
    Plan,
    Pattern,
}

/// Typed consent level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsentLevel {
    Emit,
    Store,
    Forget,
}

/// Errors from injection.db ingestion.
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

/// Open injection.db (read-only) and return an ingestor.
///
/// Configures WAL pragmas (`PRAGMA query_only = ON`) and opens in
/// read-only mode. Does NOT run migrations (injection.db schema is
/// owned by the memory-injection service).
///
/// # Errors
///
/// Returns `InjectionDbError::DatabaseOpenFailed` if the file does not exist.
pub fn open_injection_db(
    config: &InjectionDbConfig,
) -> Result<InjectionDbIngestor, InjectionDbError>;

/// Ingestor over injection.db's causal_chain table.
pub struct InjectionDbIngestor { /* private */ }

impl InjectionDbIngestor {
    /// Read unresolved chains ordered by `reinforcement_count DESC`, limited
    /// to `config.max_unresolved`. Only returns rows with `consent IN ('Emit', 'Store')`.
    ///
    /// # Errors
    ///
    /// Returns `InjectionDbError::QueryFailed` on SQLite error.
    pub fn read_unresolved(&self) -> Result<Vec<CausalChainRow>, InjectionDbError>;

    /// Read recently-resolved chains (those with `resolved_session` within
    /// `config.resolved_recency_sessions` of the maximum resolved session
    /// in the table). Limited to `config.max_recently_resolved`.
    ///
    /// # Errors
    ///
    /// Returns `InjectionDbError::QueryFailed` on SQLite error.
    pub fn read_recently_resolved(&self) -> Result<Vec<CausalChainRow>, InjectionDbError>;

    /// Return the count of unresolved chains. Useful for m7 summary metadata.
    pub fn count_unresolved(&self) -> Result<u64, InjectionDbError>;
}
```

### Internal data structures

```rust
/// Internal SQLite connection. Not public.
struct IngestorInner {
    conn: rusqlite::Connection,
    config: InjectionDbConfig,
}

/// Parse a raw SQLite row into a `CausalChainRow`, mapping string chain_type
/// and consent columns to typed enums.
fn parse_causal_chain_row(row: &rusqlite::Row<'_>) -> Result<CausalChainRow, InjectionDbError>;

fn parse_chain_type(s: &str) -> Result<ChainType, InjectionDbError>;
fn parse_consent(s: &str) -> Result<ConsentLevel, InjectionDbError>;
```

### Data flow

- **READS:** `~/.local/share/habitat/injection.db` (SQLite WAL, read-only, `PRAGMA query_only = ON`; does not apply migrations)
- **EMITS:** `Vec<CausalChainRow>` to m7 `workflow_arc_record` (unresolved chains as supplementary lifecycle context); `count_unresolved()` scalar to m12 `report_emitter` for CLI summary
- **WRITES:** none (Phase A read-only invariant)

### Boilerplate lifts (with reuse %)

| Source file | What is lifted | Reuse % | What is adapted |
|---|---|---|---|
| `memory-injection/m06_schema.rs` — `configure_connection()` | Full WAL pragma batch, `busy_timeout = 5000`, `foreign_keys = ON` | ~90% | Add `PRAGMA query_only = ON` (read-only enforcement); drop migration logic entirely |
| `memory-injection/m06_schema.rs` — `open_database()` | Path-existence check, parent-dir creation guard, `Connection::open()` → typed error mapping | ~85% | Do not call `create_all_tables` or `migrate`; do not set schema version. Open read-only URI. |
| `memory-injection/m07_causal_chain.rs` — `CausalChainRow` struct | Field list (`id, origin_session, resolved_session, chain_type, label, description, reinforcement_count, last_reinforced_session, consent`) | ~70% | Rename to workflow-trace local type (no import of memory-injection); replace `String` chain_type with `ChainType` enum; replace `String` consent with `ConsentLevel` enum |
| `memory-injection/m07_causal_chain.rs` — `parse_row()` function | `row.get::<_, T>(N)?` column extraction pattern for each field | ~75% | Map string column values to typed enums via `parse_chain_type` / `parse_consent`; return `InjectionDbError::RowParseFailed` rather than `rusqlite::Error` |
| `memory-injection/m07_causal_chain.rs` — `find_unresolved()` query | `SELECT ... FROM causal_chain WHERE resolved_session IS NULL ORDER BY reinforcement_count DESC LIMIT ?` | ~80% | Add `consent NOT IN ('Forget')` filter; rename error type; parametrize limit from `InjectionDbConfig` |
| `memory-injection/m11_parallel_query.rs` — timing harness | Per-query `Instant::now()` + `elapsed_ms` annotation | ~60% | Use for `read_unresolved()` + `read_recently_resolved()` timing; emit to tracing span, not to `QueryResult<T>` wrapper |

The `m07_causal_chain.rs` boilerplate is the most direct lift: the row parsing function (`parse_row`) and the unresolved-query pattern (`find_unresolved`) are essentially identical in structure. The primary adaptation is replacing raw `String` columns with typed enums and attaching the `InjectionDbError` domain.

### ME v2 foundation patterns referenced

- `m1_foundation/error.rs` — `InjectionDbError` follows the same `thiserror` + named-field variant discipline. `UnknownChainType(String)` and `UnknownConsent(String)` mirror the "data-driven error variant" pattern where the bad value is preserved for diagnostic display.
- `m1_foundation/shared_types.rs` — `ChainType` and `ConsentLevel` are value-type enums following the ME v2 convention: exhaustive, `#[derive(Debug, Clone, PartialEq, Eq)]`, no `Default` (because there is no meaningful default for an enum discriminant).
- `m1_foundation/resources.rs` — module docstring block: Layer (L1 Substrate Ingest), Dependencies (rusqlite, memory-injection schema knowledge as read-only reference only — not as a Rust import), Tests (50 target), Features (read unresolved chains, read recently-resolved chains, consent filtering, typed enum row representation), Platform Support (Linux; injection.db path `~/.local/share/habitat/injection.db`), Implementation Notes (zero migration logic; schema is external; `PRAGMA query_only = ON`), Related Documentation.

### Constraints satisfied

- **W1** (narrowed scope — read-only) — `PRAGMA query_only = ON`; `open_database` uses read-only connection mode.
- **F8** (no feedback-loop poisoning) — m3 never writes to injection.db. Ingested chain data is forwarded to m7 only; m3 does not act on chain content.
- **Core measurement** — causal chain reinforcement counts are a habitat-level signal for which problems recur across sessions; without m3, m7 cannot record co-occurrence of workflow patterns with open habitat issues.

### Tests (50+ minimum)

Key invariants to verify:

1. `open_injection_db` returns `DatabaseOpenFailed` when path does not exist
2. `open_injection_db` succeeds on an in-memory database with injection.db's schema applied
3. `PRAGMA query_only = ON` set — verify INSERT fails on the opened connection
4. `busy_timeout = 5000` set
5. `read_unresolved` returns empty `Vec` on empty `causal_chain` table
6. `read_unresolved` returns only rows with `resolved_session IS NULL`
7. `read_unresolved` orders rows by `reinforcement_count DESC`
8. `read_unresolved` respects `max_unresolved` limit
9. `read_unresolved` excludes `consent = 'Forget'` rows
10. `read_unresolved` includes `consent = 'Emit'` rows
11. `read_unresolved` includes `consent = 'Store'` rows
12. `read_recently_resolved` returns empty `Vec` when no rows have `resolved_session IS NOT NULL`
13. `read_recently_resolved` limits results to `max_recently_resolved`
14. `read_recently_resolved` filters to rows within `resolved_recency_sessions` of max resolved session
15. `count_unresolved` returns 0 on empty table
16. `count_unresolved` returns correct count with mixed resolved/unresolved rows
17. `CausalChainRow.chain_type` is `ChainType::Bug` for `"bug"` column value
18. `CausalChainRow.chain_type` is `ChainType::Trap` for `"trap"` column value
19. `CausalChainRow.chain_type` is `ChainType::Plan` for `"plan"` column value
20. `CausalChainRow.chain_type` is `ChainType::Pattern` for `"pattern"` column value
21. `UnknownChainType` error returned for unrecognised `chain_type` value (schema constraint violation)
22. `CausalChainRow.consent` is `ConsentLevel::Emit` for `"Emit"` column value
23. `CausalChainRow.consent` is `ConsentLevel::Store` for `"Store"` column value
24. `CausalChainRow.consent` is `ConsentLevel::Forget` for `"Forget"` column value
25. `UnknownConsent` error returned for unrecognised consent value
26. `CausalChainRow.resolved_session` is `None` for unresolved rows
27. `CausalChainRow.resolved_session` is `Some(n)` for resolved rows
28. `CausalChainRow.last_reinforced_session` is `None` when column is NULL
29. `CausalChainRow.reinforcement_count` is `1` for newly seeded rows (injection.db default)
30. `InjectionDbConfig::default()` produces `max_unresolved = 50`
31. `InjectionDbConfig::default()` produces `max_recently_resolved = 20`
32. `InjectionDbConfig::default()` produces `resolved_recency_sessions = 10`
33. `db_path` field defaults to `~/.local/share/habitat/injection.db` (homedir resolution)
34. `read_unresolved` with 100 seeded unresolved rows + `max_unresolved = 10` returns exactly 10
35. Returned rows are ordered correctly — highest `reinforcement_count` first
36. `QueryFailed` returned on schema mismatch (wrong table name in query — test with DROP TABLE)
37. `RowParseFailed` produced when `id` column contains unexpected type
38. `InjectionDbError` implements `std::error::Error`
39. `InjectionDbError` implements `Display` for all variants with non-empty output
40. `CausalChainRow` implements `Clone` without panic
41. `CausalChainRow` implements `Debug` with non-empty output
42. `ChainType` implements `PartialEq` — `Bug == Bug`, `Bug != Trap`
43. `ConsentLevel` implements `PartialEq` — `Emit == Emit`, `Emit != Store`
44. `open_injection_db` creates parent directories when `db_path` specifies a nested path (from `m06_schema.rs` guard)
45. Two concurrent `InjectionDbIngestor` instances reading the same file do not deadlock (WAL shared read)
46. `read_recently_resolved` with `resolved_recency_sessions = 0` returns no rows
47. Schema version column `PRAGMA user_version` is NOT modified by m3 (read-only enforcement)
48. `read_unresolved` query uses partial index `idx_causal_unresolved` — verify via `EXPLAIN QUERY PLAN` returning index scan
49. `count_unresolved` agrees with `read_unresolved(config.max_unresolved = usize::MAX).len()` for tables < 10,000 rows
50. Module compiles with `#![forbid(unsafe_code)]`

### Open questions for G5 spec interview

- Q: Should `consent = 'Forget'` rows be excluded at the SQL query level (current spec) or at the application layer? SQL-level exclusion is more efficient but couples m3 to the consent semantics; application-layer exclusion is more testable.
- Q: Does m7 need both unresolved and recently-resolved chains, or only unresolved? (Determines whether `read_recently_resolved` is needed at all in Phase A)
- Q: Should `read_unresolved` be called once at startup (snapshot) or periodically (continuous ingest)? If continuous, m3 needs a polling loop or a filesystem-watch on injection.db's WAL file.

### LOC estimate: ~70

- Lifted (~50 LOC): `configure_connection`, `open_database` shell, `parse_row` column extraction pattern, `find_unresolved` SQL query structure
- Novel authorship (~20 LOC): typed `ChainType` + `ConsentLevel` enums + parse functions; `read_recently_resolved` query (recency window join); `count_unresolved` scalar query

---

## Cluster A summary

| Metric | Value |
|---|---|
| Total LOC | ~230 (m1 ~80 + m2 ~80 + m3 ~70) |
| Total tests | 150 minimum (50 per module) |
| Boilerplate reuse | ~70% across the cluster |
| Novel authorship | ~30% (cursor-based pagination m1; narrowed-subscription wiring m2; typed enum parsing + recency query m3) |
| Phase A verb compliance | Reads, correlates, records, emits — no active verbs |
| Cross-cluster outputs | Feeds Clusters B (m4/m5/m6 via m1 rows), C (m7 via m3 rows; m13 trust gate via m2), D (m9 write-gate trust signal via m2 registration) |
| Boilerplate sources | `memory-injection/m06_schema.rs`, `memory-injection/m07_causal_chain.rs`, `memory-injection/m11_parallel_query.rs`, `memory-injection/m18_atuin_cache.rs`, `stcortex/clients/rust-subscriber/src/main.rs`, `stcortex/clients/rust-subscriber/src/capacity.rs` |
| ME v2 patterns applied | `thiserror` error taxonomy, `//!` module docstring block, newtype + value-type enum discipline, `Default + named constructor` config pattern |
| Primary absence (must author) | Cursor-based pagination in m1 (~30 LOC) — flagged by Boilerplate Hunt as cannot-lift gap |
| Secondary absence (must author) | Typed enum parsing for `ChainType` + `ConsentLevel` in m3 (~20 LOC); narrowed-subscription wiring in m2 (~15 LOC) |

---

*Cluster A spec complete. Next cluster specs: [[cluster-B-habitat-observation]] · [[cluster-C-central-correlation]] · [[cluster-D-trust]] · [[cluster-E-evidence-pressure]] · [[cluster-F-iteration]] · [[cluster-G-bank-select-dispatch]] · [[cluster-H-substrate-feedback]]*
