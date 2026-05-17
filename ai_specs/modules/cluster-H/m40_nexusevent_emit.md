---
title: m40 — `m40_nexusevent_emit` Rust spec
cluster: H — Substrate Feedback
layer: L8
binary: wf-crystallise
loc_estimate: ~160
test_count_min: 65
test_kinds: [async, integration]
feature_gate: [monitoring]
verb_class: emit
cc_owns: [CC-5 (emit-half)]
cc_consumes: [CC-2]
gap_owner: [none]
boilerplate_lift_pct: 40
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A
hold_v2_compliant: true
---

# m40 — `m40_nexusevent_emit` Rust spec

> Back to: [vault cluster-H spec](../../../the-workflow-engine-vault/module%20specs/cluster-H-substrate-feedback.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · [V7 cluster-H plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-H.md) · [GENESIS v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · vault [[cluster-H-substrate-feedback]]
>
> Sister anchors: [m40](m40_nexusevent_emit.md) · [m41](m41_lcm_rpc.md) · [m42](m42_stcortex_emit.md)

## 1. Purpose & invariants

`m40_nexusevent_emit` IS the substrate-feedback emit path from workflow-trace into SYNTHEX v2 at `127.0.0.1:8092/v3/nexus/push`. For every workflow lifecycle transition observed by m32 (Promote / Run / Decay), m40 serialises a `WorkflowEvent` into a SYNTHEX-shaped `NexusEvent` envelope, durably appends it to an outbox JSONL file, and then (fire-and-forget) attempts an HTTP POST to SYNTHEX. SYNTHEX consumes the event into its own Hebbian loop, contributing to `substrate_LTP_density` per the Hebbian v3 reconciliation note (Phase 1 target > 0.015). m40 is the *only* path by which workflow-trace events enter SYNTHEX's coordination substrate; without m40, CC-5's SYNTHEX half is severed.

The module MUST guarantee four invariants. First, **outbox-first ordering**: the JSONL line is fully written and `fsync`ed before any HTTP attempt — substrate-down NEVER blocks dispatch and event durability is independent of SYNTHEX reachability. Second, **fire-and-forget HTTP**: the wire call is `spawn_blocking`-wrapped (per AP29 mitigation lifted from `m22_synthex_async.rs`) with a 2s cap (reduced from m22's 10s because SYNTHEX nexus push is non-critical), and its outcome never propagates as a `Result::Err` to m32. Third, **circuit-breaker discipline**: shared `m40_42_common::Breaker` (one instance per peer; for m40 the peer is SYNTHEX) with 5-failure-Open / 60s-HalfOpen, lifted from `m22_synthex_async.rs` (~95% reuse). Fourth, **no `use synthex_v2::*`**: the `NexusEvent` and `NexusPushEnvelope` shapes are *locally re-declared* in m40; workflow-trace has zero compile-time dependency on the synthex-v2 crate (per GENESIS v1.3 § 2 hard refusal).

Frame violations m40 must structurally refuse: (a) blocking m32's dispatch on substrate health — outbox decouples this absolutely; (b) typed-enum import from synthex-v2 (Option A untyped-JSON-in-`data` is the MVP routing; Option B typed-shared-crate is post-soak hardening only); (c) AP30 namespace drift — every `WorkflowEvent.id` MUST be `workflow_trace_<workflow_id>` and is validated at the m9 namespace-guard layer before the outbox write; (d) AP-V7-13 "diagnostics theatre" — m40 health-200 from SYNTHEX is NEVER taken as proof that the event landed in SYNTHEX's Hebbian path (only stcortex pathway-weight movement at the substrate-side end of CC-5 verifies that); m40 emits the `emit_circuit_open_total` counter and the outbox-append counter, never claims "delivered".

The module is `cfg(povm_calibrated)`-gated at the crate root by m8. m9 wraps the `WorkflowEvent.id` validation; m10 Ember-CI-gate applies at the source-tree review boundary (no Ember runtime check inside m40). m40 itself reads zero substrate state — it is pure write-side.

## 2. Public surface (Rust types — spec only, NOT compileable)

```rust
//! # m40_nexusevent_emit
//!
//! - **Layer**: L8 (Substrate Feedback, Cluster H)
//! - **Deps**: tokio (rt + spawn_blocking), serde + serde_json, thiserror, tracing, parking_lot,
//!   workflow_core::types::{WorkflowId, SessionId}, workflow_core::namespace::WORKFLOW_TRACE_PREFIX,
//!   workflow_core::errors::EmitError, m40_42_common::breaker::{Breaker, BreakerState}
//! - **Tests**: 65 (30 unit + 5 property + 1 fuzz target + 15 integration + 5 contract + 4 regression + mutation ≥75%)
//! - **Features**: `monitoring` (enabled; exports counters `emit_appended_total`, `emit_posted_total`,
//!   `emit_circuit_open_total`, `emit_failed_total`)
//! - **Platform**: Linux; raw TCP HTTP POST (no reqwest); 2s spawn_blocking cap
//! - **Impl Notes**: NexusEvent shape RE-DECLARED locally; `serde rename = "type"` for event_type wire field;
//!   `kind` field is internal-Rust enum discriminator. Option A untyped-`data` JSON is the MVP wire form.
//! - **Related Docs**: [vault cluster-H](../../../the-workflow-engine-vault/module%20specs/cluster-H-substrate-feedback.md)
//!   · [V7 cluster-H plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-H.md)
//!   · [ULTRAMAP](../../../ai_docs/optimisation-v7/ULTRAMAP.md) View 2 row m40

/// Lifecycle event for one workflow in the bank.
///
/// Serialised into `NexusEvent.data` as untyped JSON (Option A wire shape).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkflowEvent {
    Promote { id: WorkflowId, lineage: Vec<WorkflowId> },
    Run     { id: WorkflowId, outcome: RunOutcome },
    Decay   { id: WorkflowId, weight: f64 },
}

/// Outcome of a single workflow dispatch (mirror of m32::DispatchOutcome relevant variant).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunOutcome { Pass, PassVerified, Fail, Blocked }

/// Local re-declaration of SYNTHEX's NexusEvent wire shape.
/// MUST NOT be replaced by `use synthex_v2::NexusEvent` (GENESIS v1.3 § 2).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NexusEvent {
    #[serde(rename = "type")]
    pub event_type: String,                // e.g. "workflow_promote"
    pub ts: u64,                           // unix seconds
    pub data: serde_json::Value,           // untyped (Option A)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NexusPushEnvelope { pub events: Vec<NexusEvent> }

/// Durable outbox record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutboxEnvelope {
    pub id: u64,
    pub created_at: u64,
    pub event: WorkflowEvent,
    pub attempts: u32,
    pub posted: bool,
}

#[derive(Debug, Clone, Default)]
pub struct EmitOutcome {
    pub appended: bool,
    pub posted: bool,
    pub failed: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum EmitError {
    #[error("outbox append failed: {0}")]
    OutboxIo(#[source] std::io::Error),
    #[error("serialize failed: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("namespace guard: id={id:?} missing workflow_trace_ prefix")]
    NamespaceViolation { id: String },
    #[error("circuit open — SYNTHEX unreachable")]
    CircuitOpen,
    #[error("HTTP POST returned non-2xx: status {status}")]
    HttpStatus { status: u16 },
}

#[derive(Debug, Clone)]
pub struct NexusEmitterConfig {
    pub addr: String,                      // "127.0.0.1:8092" (no http:// prefix — BUG-033)
    pub outbox_path: std::path::PathBuf,   // <data_dir>/m40/outbox.jsonl
    pub spawn_blocking_cap_ms: u64,        // default 2_000 (m22 = 10_000, m40 reduces)
    pub fail_threshold: u8,                // default 5
    pub open_duration_ms: u64,             // default 60_000
    pub sweep_interval_secs: u64,          // default 60
    pub max_retry_attempts: u32,           // default 5
}

impl Default for NexusEmitterConfig { /* ... */ }

pub struct NexusEmitter { /* private: config, breaker, outbox_writer, next_id (AtomicU64) */ }

impl NexusEmitter {
    pub fn with_config(config: NexusEmitterConfig) -> Result<Self, EmitError>;

    /// Outbox-first emit. Returns `EmitOutcome { appended: true, ... }` whenever
    /// the outbox write succeeded, regardless of HTTP outcome.
    pub async fn emit(&self, event: WorkflowEvent) -> Result<EmitOutcome, EmitError>;

    /// Drains the outbox of `posted=false` envelopes (background sweep task).
    pub async fn retry_sweep(&self) -> Result<u32, EmitError>;

    /// Removes `posted=true` envelopes older than 24h.
    pub async fn compact(&self) -> Result<u32, EmitError>;
}
```

## 3. Internal data structures

```rust
struct EmitterInner {
    config: NexusEmitterConfig,
    breaker: Arc<Breaker>,             // shared peer (SYNTHEX)
    outbox: Mutex<OutboxWriter>,       // append-only JSONL handle
    next_id: AtomicU64,                // monotonic envelope id
}

struct OutboxWriter {
    file: tokio::fs::File,             // O_APPEND
    bytes_written: u64,
}
```

The outbox file is open-once at construction with `O_APPEND`. Each `emit` call serialises the `OutboxEnvelope` to a `Vec<u8>`, writes one `\n`-terminated line, calls `sync_data().await`, then attempts the HTTP POST. Concurrent emits are serialised by the inner `Mutex<OutboxWriter>` — JSONL line atomicity is preserved.

## 4. Data flow

- **INPUT FROM:** `m32::DispatchOutcome` (Cluster G) — m32 calls `emitter.emit(event).await` after every dispatch; m32 also calls `emit` from m11 decay events and m30 promote events
- **OUTPUT TO:** `{data_dir}/m40/outbox.jsonl` (durable) and `127.0.0.1:8092/v3/nexus/push` (best-effort)
- **SUBSTRATE TOUCHED:** SYNTHEX v2 only (no stcortex, no POVM, no atuin)
- **WRITES:** outbox JSONL (always); SYNTHEX v3 nexus bus (when breaker Closed/HalfOpen)

## 5. Algorithm sketch

```text
emit(event):
    # m9 namespace guard
    if not event.id_str().starts_with("workflow_trace_"):
        return Err(NamespaceViolation)

    # Step 1: serialise + append to outbox (durable)
    envelope = OutboxEnvelope { id: next_id.fetch_add(1), created_at: now_s(),
                                event: event.clone(), attempts: 0, posted: false }
    line = serde_json::to_vec(&envelope) ++ b"\n"
    outbox.lock().await:
        file.write_all(&line).await?
        file.sync_data().await?
    metric!(emit_appended_total += 1)

    # Step 2: HTTP fire-and-forget
    if not breaker.allow().await:
        metric!(emit_circuit_open_total += 1)
        return Ok(EmitOutcome { appended: true, posted: false, failed: false })

    nexus_event = NexusEvent {
        event_type: event.wire_type_str(),  # "workflow_promote" | "workflow_run" | "workflow_decay"
        ts: now_s(),
        data: serde_json::to_value(&event)?  # Option A: untyped data payload
    }
    envelope_json = NexusPushEnvelope { events: vec![nexus_event] }
    body = serde_json::to_vec(&envelope_json)?

    # spawn_blocking-wrapped raw TCP POST with 2s cap (AP29 mitigation)
    posted = tokio::time::timeout(2s, spawn_blocking(|| raw_http_post(addr, "/v3/nexus/push", body))).await

    match posted:
        Ok(Ok(2xx)) -> breaker.record_success().await; mark_envelope_posted(envelope.id); Ok(EmitOutcome { appended: true, posted: true, failed: false })
        Ok(Ok(non-2xx status)) -> breaker.record_failure().await; metric!(emit_failed_total += 1); Ok(EmitOutcome { appended: true, posted: false, failed: true })
        Ok(Err(io)) | Err(timeout) -> breaker.record_failure().await; metric!(emit_failed_total += 1); Ok(EmitOutcome { appended: true, posted: false, failed: true })

retry_sweep():
    for line in outbox.jsonl where posted == false and attempts < max_retry_attempts:
        attempt HTTP POST; on success mark posted=true; on failure ++attempts
    return drained_count
```

## 6. Boilerplate lifts

Per V7 cluster-H plan § m40 § Boilerplate-lift source (Category 08 Nexus-LCM-RPC, gold standard per Command-3 E3):

| Source | Lift | % |
|---|---|---:|
| `m22_synthex_bridge.rs::raw_http_post` | TCP connect + HTTP/1.1 request builder + status parse | 90% |
| `m22_synthex_async.rs::spawn_blocking` wrapper + 2s timeout cap | AP29 mitigation harness | 95% (cap reduced 10s → 2s) |
| `m22_synthex_async.rs::Breaker` state machine | Closed/Open/HalfOpen 5-fail / 60s | 95% (shared via `m40_42_common::breaker`) |
| `m24_povm_bridge.rs` outbox-append pattern | JSONL atomic append + `sync_data` | 70% (adapted for `OutboxEnvelope` shape) |
| Cursor `next_id` (AtomicU64) | — | 0% (novel ~10 LOC) |

Net: ~120 LOC lifted / ~40 LOC novel (WorkflowEvent enum + envelope build + namespace guard glue).

## 7. ME v2 m1_foundation patterns referenced

- `error.rs` — `EmitError` is a `thiserror::Error` enum with structured named-field variants (`NamespaceViolation { id }`, `HttpStatus { status }`); callers can match programmatically.
- `resources.rs` — `//!` module docstring block (Layer / Deps / Tests / Features / Platform / Impl Notes / Related Docs) adopted verbatim.
- `shared_types.rs` — newtype discipline (`WorkflowId`, `SessionId`); `NexusEmitterConfig` follows `Default + env-override + override-field` pattern.
- `logging.rs` — tracing-subscriber structured emit on every outbox append (envelope_id, workflow_id, event_kind), HTTP POST success (workflow_id), and breaker open skip (workflow_id, breaker_state).

## 8. Test strategy

- **Test kind**: async (30 unit) + property (5) + 1 fuzz target + integration (15) + contract (5) + regression (4); total **65** (per V7 cluster-H plan § m40 § Test budget; matches MODULE_MATRIX row m40 65 tests)
- **Mutation budget**: ≥**75%** kill on `outbox.rs` + `wire.rs` + breaker call sites (per V7 G6 § m40)
- **Properties tested** (F-Property 5):
  - Outbox-write-first invariant: `fsync` completes before `breaker.allow()` is consulted
  - Breaker idempotent under concurrent emits: parallel `record_failure` does not over-count
  - AP30 prefix invariant: any `WorkflowEvent.id` without `workflow_trace_` returns `NamespaceViolation` before any IO
  - Cursor monotonic: `next_id` strictly increasing across `emit` calls
  - Schema parity: roundtrip `NexusEvent` through `to_string` / `from_str` is identity-preserving (`event_type` rename = `"type"` preserved)
- **Fuzz target** (1): `m40_outbox_jsonl` — feed arbitrary bytes to `OutboxEnvelope` JSONL line parser; assert no panic, no UB (per V7 G6 § Fuzz enumeration)

Key invariants (sample of 65; full list per V7 cluster-H plan):

1. Outbox write completes before HTTP attempt (`spawn_blocking` is dispatched after `sync_data().await`)
2. `EmitOutcome { appended: true, posted: false, failed: false }` when breaker Open
3. `EmitOutcome { appended: true, posted: true, failed: false }` on 2xx
4. `EmitOutcome { appended: true, posted: false, failed: true }` on non-2xx
5. Namespace guard rejects `"foo_bar"` id, accepts `"workflow_trace_wf-abc"`
6. NexusEvent wire field is `"type"` not `"event_type"` (serde rename)
7. Option A: `data` field carries the full `WorkflowEvent` JSON; `kind` discriminator preserved
8. Breaker transitions Closed → Open after 5 consecutive failures
9. Breaker transitions Open → HalfOpen after 60s
10. HalfOpen → Closed on success; HalfOpen → Open on failure
11. spawn_blocking cap is 2s (not m22's 10s)
12. Outbox compaction removes `posted=true` envelopes older than 24h
13. Retry sweep does NOT re-attempt `posted=true` envelopes
14. Retry sweep respects `max_retry_attempts` (default 5)
15. Concurrent emit calls do NOT corrupt outbox JSONL (mutex-protected writer)
16. BUG-033: `addr` is `"127.0.0.1:8092"` — no `http://` prefix
17. Contract: SYNTHEX v3 `/v3/nexus/push` accepts `NexusPushEnvelope` shape (insta snapshot)
18. Regression: `WorkflowEvent::Run { outcome: PassVerified }` round-trips through JSON
19. Watcher Class-B test stub: every wire-attempt emits a structured tracing event

(remaining 47 enumerated in vault cluster-H spec § Test coverage requirements)

## 9. Antipatterns to avoid

- **AP30** (namespace prefix violation) — m9 validates `workflow_trace_*` at every emit; `NamespaceViolation` returned before any IO
- **AP-V7-13** (Health-200 ≠ behaviour-verified) — m40 NEVER claims "delivered" on a 2xx; observability is the `emit_posted_total` counter + the substrate-side `learning_health` delta (the latter is m42's domain, not m40's)
- **AP29** (sync HTTP in `tokio::spawn` starves the runtime) — mitigated by `spawn_blocking` + 2s cap; raw TCP via lifted `raw_http_post`, no reqwest
- **AP-Drift-06** (bridge contract drift) — F-Contract tests + `bridge-contract` skill pre-merge; serde rename `"type"` is a regression-slot
- **AP-WT-F3** (substrate-input poisoning) — m40 is write-only; reads zero state
- **AP-Hab-04** (preserve-list discipline) — outbox compaction enumerates `posted=true && older_than_24h`; no blanket truncate

## 10. Useful patterns applied

- Outbox-first JSONL durability (PATTERNS.md § Substrate-write patterns)
- Circuit-breaker shared library (PATTERNS.md § Architectural patterns; `m40_42_common::breaker`)
- `spawn_blocking` + 2s cap (PATTERNS.md § Module-level patterns; AP29 mitigation)
- thiserror error enums (GOLD_STANDARDS rule 9)
- Newtype discipline (GOLD_STANDARDS rule 8)
- `//!` docstring block (GOLD_STANDARDS rule 13)

## 11. Cross-cluster contracts

- **CC-5 (OWNS emit-half via m40 / m41 / m42)**: m40 is one of three CC-5 emit channels. Its specific contract is *every* workflow lifecycle transition observed by m32 produces a `WorkflowEvent` in the outbox; HTTP delivery to SYNTHEX is best-effort but the outbox guarantees no event is lost on substrate outage. m31's read-back from updated pathway weights closes the loop on a days/weeks timescale.
- **CC-2 (consumes trust layer)**: m9 namespace-guard validates `WorkflowEvent.id` prefix; m8 build-prereq gates the crate compilation on POVM-calibrated env (carried forward from CC-2 even though m42 is stcortex-only — m8/m9/m10 layer is engine-wide); m10 Ember-CI-gate applies at source review.

## 12. Open questions for G5 interview / Zen G7 audit

1. **Option A untyped-JSON sunset timing**: when does the MVP `data: serde_json::Value` get promoted to Option B typed-shared-crate? (V7 plan defers to "post-soak"; concrete trigger TBD — D60 substrate-condition reading? D90? Tied to substrate_LTP_density crossing Phase 2 0.05?)
2. **Outbox path conflict with synthex-v2-shadow daemon**: synthex-v2-shadow writes to `~/.local/share/synthex-v2-shadow/outbox/`; m40's `{data_dir}/m40/outbox.jsonl` should NOT collide, but the `data_dir` default must be explicit (XDG-data-home? cwd-local? per-binary?)
3. **Lineage population**: `WorkflowEvent::Promote { lineage }` requires m30 to track ancestor IDs through proposal → bank acceptance. Is the lineage chain capped (last N) or full?
4. **Decay weight source**: `WorkflowEvent::Decay { weight }` — does m40 receive the *pre-decay* or *post-decay* weight from m11? (Test invariant depends on this.)
5. **Retry sweep cadence under burst**: 60s sweep is fine for steady-state; what about ~100 events emitted within 5s when SYNTHEX is mid-restart? (Backoff with jitter ±25% on sweep interval is in scope; per-event backoff is not.)

## 13. Implementation order (post-G9)

1. `errors.rs` — `EmitError` enum (`thiserror`); compile-only, no tests
2. `wire.rs` — `NexusEvent` + `NexusPushEnvelope` + `raw_http_post` lift from m22_synthex_bridge; 6 unit tests (serde rename, envelope shape)
3. `outbox.rs` — `OutboxEnvelope` + `OutboxWriter` + `sync_data` + 10 unit tests (atomic append, concurrent writer, compaction)
4. `breaker.rs` — `m40_42_common::Breaker` shared lib (also consumed by m41/m42); 8 unit tests (state transitions, concurrent fail/success)
5. `mod.rs` — `NexusEmitter::with_config` + `emit` + `retry_sweep` + namespace-guard glue; 6 unit tests (Outcome variants, namespace violation)
6. Property tests (5) — proptest on outbox-write-first / breaker idempotency / cursor monotonic / namespace / schema parity
7. Fuzz target (1) — `m40_outbox_jsonl` JSONL line parser, 24h budget
8. Integration tests (15) — `tests/m40_integration.rs` with mock SYNTHEX `/v3/nexus/push` (axum mock-server); also live `:8092` contract test gated `#[ignore = "requires synthex-v2"]`
9. Contract tests (5) — insta snapshots for `NexusEvent` JSON vs SYNTHEX v3 wire shape
10. Regression slots (4) — reserved for first bugs (cursor wraparound, sync_data missing, breaker race, serde rename drift)
11. Mutation pass — `cargo mutants` on `outbox.rs` + `wire.rs`; ≥75% kill required

---

> Back to: [vault cluster-H spec](../../../the-workflow-engine-vault/module%20specs/cluster-H-substrate-feedback.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · sister modules: [m40](m40_nexusevent_emit.md) · [m41](m41_lcm_rpc.md) · [m42](m42_stcortex_emit.md)
