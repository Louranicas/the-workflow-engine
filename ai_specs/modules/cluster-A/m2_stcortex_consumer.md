---
title: m2 — `m2_stcortex_consumer` Rust spec
cluster: A — Substrate Ingest
layer: L1
binary: wf-crystallise
loc_estimate: ~80
test_count_min: 50
test_kinds: [async, unit, property, integration, contract, regression, mutation]
feature_gate: [none]
verb_class: passive
cc_owns: [CC-2 (trust-signal emission)]
cc_consumes: [CC-2]
gap_owner: [none]
boilerplate_lift_pct: 25
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A
hold_v2_compliant: true
---

# m2 — `m2_stcortex_consumer` Rust spec

> Back to: [vault cluster-A spec](../../../the-workflow-engine-vault/module%20specs/cluster-A-substrate-ingest.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · [V7 cluster-A plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-A.md) · [GENESIS v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · vault [[cluster-A-substrate-ingest]]

## 1. Purpose & invariants

`m2_stcortex_consumer` IS workflow-trace's narrowed-scope consumer registration against stcortex (SpacetimeDB at `127.0.0.1:3000`). Its job at startup is twofold: (a) call `register_consumer` so the DB-layer refuse-write gate flips OPEN for the `workflow_trace_*` namespace (enabling m13 to write later), and (b) subscribe to **exactly two** narrowed queries — `tool_call WHERE namespace LIKE 'workflow_trace_%'` and `consumption_event WHERE namespace LIKE 'workflow_trace_%'` — and forward delta rows downstream. The "narrowed" qualifier is the module's load-bearing invariant: W1 in the feature matrix. All other stcortex tables (`memory`, `pathway`, `ghost_memory`, `consumer`) are excluded from m2's subscription window at the query-construction layer, not by application-layer filtering after the fact.

The module MUST guarantee three invariants. First, **W1 narrowing** — subscription query strings are hardcoded; widening requires a v1.4 spec amendment (if widening enables Watcher event tables, AP-WT-F8 feedback-loop poisoning becomes live). Second, **AP30 namespace prefix discipline** — `ConsumerIdentity` rejects any namespace not equal to `workflow_trace_*`; reserved names (`scratch`, `claude-code`) and empty string are rejected at construction. Third, **trust-signal emission** — `RegistrationHandle::is_fresh()` is the gate m13 reads before each write attempt; m13 must not bypass it. The handle is `Send` so m13 can read from another thread; `Drop` does NOT auto-unregister (stcortex stale-GC handles it per CONSUMER-ONBOARDING.md).

Frame violations m2 must structurally refuse: (a) widening the subscription to `pathway` or `memory` tables — would import Watcher writes back into iteration evidence (F8); (b) treating `consumption_event` rows as outbound writes — m2 is purely observational, the consumption-event flow is a read of stcortex's access-gradient signal, not a write back; (c) using hyphens in the consumer namespace slug (S1001757 stcortex munge bug — AP-Hab-11). m2 IS a stcortex consumer but is NOT a stcortex writer; m13 owns writes.

m2 is the **only** Cluster A module that engages CC-2 trust layer in an emitting capacity — `RegistrationHandle` flows to m13 as the OPEN gate for the refuse-write enforcement. m9 wraps m2 on the read side, validating that every incoming row has a `workflow_trace_*`-prefixed namespace before propagating to m4/m5.

## 2. Public surface (Rust types — spec only, NOT compileable)

```rust
//! # m2_stcortex_consumer
//!
//! - **Layer**: L1 (Substrate Ingest, Cluster A)
//! - **Deps**: spacetimedb-sdk (Rust subscriber), stcortex module_bindings, workflow_core::namespace::WORKFLOW_TRACE_PREFIX, workflow_core::errors::IngestError
//! - **Tests**: 50 (25 unit + 5 property + 15 integration + 3 contract + 2 regression + mutation ≥70%)
//! - **Features**: none
//! - **Platform**: Linux primary; stcortex `:3000` reachable; tokio runtime
//! - **Impl Notes**: ~60 LOC lifted from `stcortex/clients/rust-subscriber/src/main.rs` (DbConnection builder + subscription queries) + ~15 LOC novel (narrowed-scope handler filters, ConsumerIdentity validation, RegistrationHandle wrapper). Refuse-write is enforced AT THE DB LAYER (stcortex side); m2 emits the trust signal but does not itself enforce.
//! - **Related Docs**: [cluster-A vault spec](../../../the-workflow-engine-vault/module%20specs/cluster-A-substrate-ingest.md) · [V7 cluster-A](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-A.md) · [stcortex CONSUMER-ONBOARDING](../../../stcortex/docs/CONSUMER-ONBOARDING.md) · [ULTRAMAP](../../../ai_docs/optimisation-v7/ULTRAMAP.md) View 2 row m2

/// Consumer identity newtype — never raw String.
#[derive(Debug, Clone)]
pub struct ConsumerIdentity {
    pub name: ConsumerName,
    pub namespace: Namespace,
    pub transport: Transport,
}

/// Newtype: validated consumer name, max 64 chars, alphanumeric + `_-`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerName(String);

/// Newtype: validated namespace, must start with `workflow_trace_`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace(String);

/// Transport discriminant; locked at `Subscription`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transport {
    Subscription,
}

impl ConsumerIdentity {
    /// Construct from current process's git short SHA; falls back to
    /// `"workflow-trace-unknown"` if `git` is absent.
    pub fn from_git_sha() -> Self;
}

impl ConsumerName {
    /// Validating constructor; rejects empty / over-64 / invalid chars.
    pub fn new(s: impl Into<String>) -> Result<Self, StcortexConsumerError>;
}

impl Namespace {
    /// Validating constructor; rejects anything not starting with
    /// `WORKFLOW_TRACE_PREFIX` (AP30 enforcement).
    pub fn new(s: impl Into<String>) -> Result<Self, StcortexConsumerError>;
}

/// Live registration handle; passed to m13 as the trust-signal gate.
pub struct RegistrationHandle {
    /* private: SpacetimeDB SDK connection + identity + registered_at */
}

impl RegistrationHandle {
    /// `true` if registration is fresh (< 30 days by stcortex definition).
    /// m13 calls this before every write attempt.
    #[must_use]
    pub fn is_fresh(&self) -> bool;

    pub fn identity(&self) -> &ConsumerIdentity;

    /// Explicit unregister (idempotent; call on clean shutdown).
    pub fn unregister(self) -> Result<(), StcortexConsumerError>;
}

/// Snapshot of one `consumption_event` delta row.
#[derive(Debug, Clone)]
pub struct ConsumptionEventRow {
    pub memory_id: u64,
    pub consumer_name: ConsumerName,
    pub consumed_at_ms: i64,
}

/// Snapshot of one `tool_call` delta row.
#[derive(Debug, Clone)]
pub struct ToolCallRow {
    pub call_id: u64,
    pub session_id: String,
    pub namespace: Namespace,
    pub command: String,
    pub recorded_at_ms: i64,
}

/// Typed delta row enum.
#[derive(Debug, Clone)]
pub enum StcortexRow {
    ToolCall(ToolCallRow),
    ConsumptionEvent(ConsumptionEventRow),
}

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
    #[error("consumer-name validation failed: {0}")]
    InvalidConsumerName(String),
}

/// Register as a narrowed stcortex consumer and apply both subscriptions.
///
/// Connects via `ws://127.0.0.1:3000`, calls `register_consumer`, subscribes
/// to:
///   - `SELECT * FROM tool_call WHERE namespace LIKE 'workflow_trace_%'`
///   - `SELECT * FROM consumption_event WHERE namespace LIKE 'workflow_trace_%'`
///
/// Waits up to `timeout_ms` for the `on_applied` callback before returning.
///
/// # Errors
/// See `StcortexConsumerError` variants.
pub async fn register_narrowed_consumer(
    identity: ConsumerIdentity,
    timeout_ms: u64,
) -> Result<RegistrationHandle, StcortexConsumerError>;

/// Callback type for delta rows. m14 evidence aggregator wires here.
pub type StcortexRowCallback = Box<dyn Fn(StcortexRow) + Send + Sync>;
```

## 3. Internal data structures

```rust
struct ConsumerState {
    conn: module_bindings::DbConnection,        // SpacetimeDB SDK type
    identity: ConsumerIdentity,
    registered_at: std::time::Instant,
    subscription_applied: std::sync::atomic::AtomicBool,
    on_row: Option<StcortexRowCallback>,
}
```

Subscription-applied confirmation uses `mpsc::channel::<()>()` driven by the SDK's `on_applied` callback; the `AtomicBool` mirrors that signal for non-blocking polling from `RegistrationHandle::is_fresh()`.

## 4. Data flow

- **INPUT FROM:** stcortex SpacetimeDB at `ws://127.0.0.1:3000` — narrowed delta stream limited to `tool_call` + `consumption_event` rows in `workflow_trace_*` namespace
- **OUTPUT TO:**
  - **m13 `stcortex_writer_narrowed`** (Cluster C): `RegistrationHandle` as trust-signal gate (CC-2)
  - **m4 `cascade_correlator`** (Cluster B): `ToolCallRow` deltas via callback
  - **m5 `battern_step_record`** (Cluster B): `ToolCallRow` deltas
  - **m14 `habitat_outcome_lift`** (Cluster E): `ConsumptionEventRow` deltas as access-gradient evidence
- **SUBSTRATE TOUCHED:** stcortex only
- **WRITES:** `register_consumer` reducer call (NOT a domain-data write; stcortex manages the `consumer` table internally)
- **SIDE-EFFECT:** `consumer` row created in stcortex's own schema; refuse-write gate flips OPEN for `workflow_trace_*` namespace

## 5. Algorithm sketch

```text
register_narrowed_consumer(identity, timeout_ms):
    validate identity (already enforced at type level via newtypes)
    (tx, rx) = mpsc::channel::<()>()
    conn = DbConnection::builder()
        .with_uri("ws://127.0.0.1:3000")
        .with_database_name("stcortex")
        .on_connect(|ctx, _, _| {
            ctx.reducers.register_consumer(
                identity.name.into(),
                identity.namespace.into(),
                Transport::Subscription as_str(),
            );
            subscribe_query(ctx,
                "SELECT * FROM tool_call WHERE namespace LIKE 'workflow_trace_%'",
                on_applied = || { tx.send(()).ok(); flag.store(true, Relaxed); });
            subscribe_query(ctx,
                "SELECT * FROM consumption_event WHERE namespace LIKE 'workflow_trace_%'",
                on_applied = || { /* second subscription, separate wait */ });
        })
        .on_connect_error(map_to_ConnectionFailed)
        .on_disconnect(log_warn_continue)
        .build()?;
    conn.run_threaded();
    match rx.recv_timeout(Duration::from_millis(timeout_ms)) {
        Ok(()) => Ok(RegistrationHandle { ... }),
        Err(_) => Err(SubscriptionTimeout { timeout_ms }),
    }
```

The two subscription applies are awaited in sequence so a `SubscriptionTimeout` always indicates a real failure rather than an ordering artefact.

## 6. Boilerplate lifts

| Source | Lift | % |
|---|---|---:|
| `stcortex/clients/rust-subscriber/src/main.rs` DbConnection builder block | `.with_uri()/.with_database_name()/.on_connect()/.on_connect_error()/.on_disconnect()/.build()?/.run_threaded()` | 80% (strip `pathway` + `memory` handlers; narrow to two subscriptions) |
| `stcortex/clients/rust-subscriber/src/main.rs` subscription query strings | `format!("SELECT * FROM ... WHERE namespace LIKE ...")` pattern | 85% (hardcode `workflow_trace_%`; two queries only) |
| `stcortex/clients/rust-subscriber/src/capacity.rs::wait_count` pattern | `rx.recv_timeout(Duration::from_secs(N))` confirmation | 90% (use `mpsc::channel::<()>` with `on_applied` send) |
| `stcortex/clients/rust-subscriber/src/capacity.rs::AtomicBool` state pattern | `AtomicBool` + `Ordering::Relaxed` for cross-thread flag | 90% |
| `stcortex/docs/CONSUMER-ONBOARDING.md` refuse-write reducer template | **Reference only** — documents architectural contract for m13 trust gate | n/a |
| `stcortex/docs/stcortex_API.md` | `register_consumer / access_memory` signatures + `transport` field semantics | Reference |

Net: ~65 LOC lifted / ~15 LOC novel (namespace validation, `ConsumerIdentity::from_git_sha`, trust-gate `is_fresh()`, `RegistrationHandle` wrapper).

## 7. ME v2 m1_foundation patterns referenced

- `error.rs` — `StcortexConsumerError` mirrors `{field, reason}` discipline; `InvalidNamespace`/`InvalidConsumerName` preserve the bad value for diagnostic display (data-driven variant pattern).
- `shared_types.rs` — `ConsumerIdentity`/`ConsumerName`/`Namespace`/`Transport` are value-type newtypes/enums; no allocation on `identity()` read; `#[must_use]` on `is_fresh()`.
- `resources.rs` — `//!` docstring block with Layer/Deps/Tests/Features/Platform/Impl Notes/Related Docs.
- `logging.rs` — tracing-subscriber emit on connect / subscription-applied / disconnect / refuse-write events (Watcher Class-I monitoring depends on this).

## 8. Test strategy

- **Test kind**: async (25 unit, all `tokio::test`) + property (5) + integration (15, real stcortex `:3000`) + contract (3) + regression (2)
- **Test count**: 50 minimum (per [TEST_DISCIPLINE matrix](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) row m2)
- **Mutation budget**: ≥70%
- **Properties tested** (F-Property 5):
  - For all delivered rows: `row.namespace.as_str().starts_with("workflow_trace_")` (W1 narrowing invariant)
  - Row-ordering preservation across simulated reconnect
  - `RegistrationHandle::is_fresh()` idempotent for fresh handles
  - `Namespace::new` rejects any input not starting with prefix
  - `ConsumerName::new` rejects empty, over-64, or invalid-char inputs

Key invariants (sample of 50; full list in vault cluster-A spec § Tests m2):

1. `from_git_sha` returns name starting with `"workflow-trace-"`
2. `from_git_sha` falls back to `"workflow-trace-unknown"` when `git` absent
3. `Namespace::new("workflow_trace_foo")` succeeds
4. `Namespace::new("scratch")` returns `InvalidNamespace`
5. `Namespace::new("claude-code")` returns `InvalidNamespace`
6. `Namespace::new("")` returns `InvalidNamespace`
7. `register_narrowed_consumer` returns `ConnectionFailed` when `:3000` unreachable
8. `is_fresh()` returns `true` immediately after registration
9. Subscription query strings contain exactly `namespace LIKE 'workflow_trace_%'`
10. Subscription query strings exclude `pathway`, `memory`, `ghost_memory`
11. `SubscriptionTimeout` fires when `on_applied` not received within `timeout_ms`
12. `transport` field is always `Subscription` (never `cli` / `polling`)
13. `RegistrationHandle` is `Send` (compile-time + assertion via `static_assertions`)
14. `unregister` is idempotent
15. Hyphen in name converts to underscore at storage layer (AP-Hab-11 regression slot)

(remaining 35 enumerated in vault spec)

## 9. Antipatterns to avoid

- **AP30** (namespace string drift) — newtype `Namespace` makes literals impossible to use directly; constant `WORKFLOW_TRACE_PREFIX` lives in `workflow_core::namespace`
- **AP-Hab-11** (hyphen-slug stcortex munge) — `ConsumerName` validator + storage-layer normalisation; regression slot reserved
- **AP-WT-F8** (Watcher feedback-loop poisoning) — narrowed subscription structurally excludes Watcher event tables; any spec amendment widening the subscription must re-evaluate F8
- **AP-WT-F3** (substrate-input poisoning) — m9 namespace-prefix filter at the m2 read boundary
- **AP-V7-13** (Health-200 ≠ behaviour-verified) — `is_fresh()` is the runtime check; don't trust mere connect success
- **AP29** (sync HTTP in `tokio::spawn`) — SpacetimeDB SDK is async-native; m2 never wraps blocking calls in `spawn`
- **Newly surfaced**: silent unregister on `Drop` would defeat stcortex's stale-GC contract — `Drop` is intentionally a no-op; unregister is explicit

## 10. Useful patterns applied

- Reducer-callback dedup pattern (PATTERNS.md § Data-flow row 3)
- thiserror error enums (GOLD_STANDARDS rule 9)
- Newtype discipline (GOLD_STANDARDS rule 8); `#[must_use]` on `is_fresh()`
- Structured tracing emit (GOLD_STANDARDS rule 7); never `println!` / `eprintln!` in SDK callbacks
- `//!` docstring block (GOLD_STANDARDS rule 13)

## 11. Cross-cluster contracts

- **CC-2 (owns: trust-signal emission)**: `RegistrationHandle` is the canonical OPEN gate for the refuse-write enforcement that lives in stcortex's reducer layer. m13 MUST call `is_fresh()` before every write attempt; m11's fitness-weighted decay also reads handle state when computing trust-decay weights. This is the **only Cluster A module** that owns a CC contract surface; m1 and m3 only consume.
- **CC-2 (consumes: aspect-wrap from m9)**: every incoming row's `namespace` field is validated by m9 at the m2 boundary; out-of-namespace rows are rejected before reaching m4/m5.
- **CC-1 (indirect via m13)**: m2 does not directly own CC-1, but `tool_call` deltas it forwards become CC-1 join rows once m13 writes back via the m7 hub.

## 12. Open questions for G5 interview / Zen G7 audit

1. **Auto-re-register on stale-GC**: should `RegistrationHandle` re-register transparently if stcortex evicts the consumer after 30 days no-read, or is manual restart sufficient? (Affects long-running soak windows.)
2. **`tool_call` session filter**: subscribe to all rows in `workflow_trace_*`, or filter to only rows matching the current Claude session UUID? (Latency vs comprehensiveness tradeoff.)
3. **Callback vs channel**: should `on_row` be `Box<dyn Fn>` (current spec) or `mpsc::Sender<StcortexRow>` (channel-based)? Channel decouples lifetime; callback is lower-latency.
4. **`SubscriptionTimeout` recovery**: hard-fail (current spec) or retry-with-backoff before giving up? Watcher Class-I depends on the choice.

## 13. Implementation order (post-G9)

1. `error.rs` — `StcortexConsumerError` enum
2. `row.rs` — `ConsumerIdentity` + `ConsumerName`/`Namespace`/`Transport` newtypes + validation + 8 unit tests
3. `config.rs` — connection URI / timeout / retry constants
4. `subscription.rs` — subscription-applied state machine + `AtomicBool` + `mpsc::channel` confirmation + 5 unit tests
5. `mod.rs` — `register_narrowed_consumer` async fn + DbConnection builder lift + 7 unit tests
6. Property tests (5) — proptest on newtype validators + W1 narrowing invariant
7. Integration tests (15) — `tests/m2_integration.rs` requires live local stcortex `:3000`; refuse-write enforcement test (no consumer → expect refuse from writer side)
8. Contract tests (3) — insta snapshots for `StcortexRow` schema vs `stcortex_API.md`
9. Regression slots (2) — hyphen-slug munge, double-register idempotency
10. Mutation pass — `cargo mutants` on `row.rs` + `subscription.rs`; ≥70% kill

---

> Back to: [vault cluster-A spec](../../../the-workflow-engine-vault/module%20specs/cluster-A-substrate-ingest.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · sister modules: [m1](m1_atuin_consumer.md) · [m2](m2_stcortex_consumer.md) · [m3](m3_injection_db_consumer.md)
