---
title: Cluster H — Substrate Feedback (m40 / m41 / m42)
date: 2026-05-17 (S1001982)
kind: module-spec
status: planning-only · HOLD-v2 active · single-phase architecture
cluster: H
modules: m40 nexus_event_emitter · m41 lcm_rpc_client · m42 hebbian_feedback
loc-estimate: ~450 (modules) + ~165 (min 55 tests each)
boilerplate-sources: 08-nexus-lcm-rpc (m22_synthex_bridge, m22_synthex_async, m24_povm_bridge, m38_deployment_api) · 01-cli-scaffolding (lcm_supervisor)
---

# Cluster H — Substrate Feedback

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Modules Synergy Clusters and Feature Verification S1001982]] · [[workflow-engine-code-base]]
>
> Sister clusters: [[cluster-G-bank-select-dispatch-verify]] (upstream dispatch events) · [[cluster-F-iteration]] (downstream selection-weight consumer)
>
> Cross-cluster synergy CC-5: G → H → back to F via m31 selection-weights-from-stcortex-pathway-reads

## Purpose

Cluster H **closes the learning loop**. When m32 (dispatcher) sends a workflow step to the habitat, Cluster H propagates the outcome to three different substrates in parallel:

- **m40** tells SYNTHEX v2 — which reinforces or depresses the event via its Hebbian loop
- **m41** routes deploy-shaped workflows through LCM's 9-RPC supervisor — so the existing state machine manages lifecycle, never workflow-trace
- **m42** tells POVM — which stores the outcome as a `workflow_trace_*`-namespaced Hebbian pathway

On the next selection cycle, m31 reads updated pathway weights back from stcortex (POVM overlap period) or directly from stcortex (post-2026-07-10). The substrate learns. Cascade monoculture shrinks over time.

---

## Hard refusals in scope for this cluster

From [[Genesis Prompt v1.2 S1001982]] §"What stays forbidden even in single-phase":

- **POVM writes (m42) are conditional on overlap window** — POVM is deprecated 2026-07-10; m42 must check the overlap flag before writing. After cutover, m42 routes to stcortex via m13. Both paths must live in the implementation.
- **stcortex writes outside `workflow_trace_*` namespace** — AP30 collision avoidance. V3 owns `P01..P16` Hebbian pathways; workflow-trace must not touch them.
- **`use synthex_v2::*` is forbidden** — m40 lifts patterns from synthex-v2 source as reference (m22_synthex_bridge.rs, m22_synthex_async.rs). It does NOT import the synthex_v2 crate. The NexusEvent struct is re-declared locally.
- **No HTTP server** — m40/m41/m42 are outbound-only clients. No axum, no inbound routes.

---

## Cluster-internal synergy

```
m32 (dispatcher, Cluster G)
    │
    ├─── dispatch event ──────────────────────────────────────────┐
    │                                                             │
    ▼                                                             │
m40 nexus_event_emitter          m41 lcm_rpc_client              m42 hebbian_feedback
    │  WorkflowEvent → SYNTHEX       │  deploy-shaped step → LCM     │  POST /reinforce → POVM
    │  (all dispatches)              │  (cargo/devenv/binary steps)  │  (all outcomes)
    │                                │                               │
    └─── SYNTHEX Hebbian ────────────┘                               │
         reinforces pathway                                          │
         weights over time                                           ▼
                                                       POVM / stcortex (post-cutover)
                                                           workflow_trace_* namespace
                                                                     │
                                                                     ▼
                                                       m31 selector reads updated
                                                       pathway weights next cycle
```

All three modules operate **fire-and-forget**: a substrate being down does not block the dispatch. The engine's workflow execution is not coupled to substrate reachability. Outbox persistence (m40), timeout-and-log (m41), and graceful degradation (m42) are the fallback paths.

---

## Cross-cluster contracts

### CC-5: Substrate Learning Loop

The full loop: m32 dispatches → m40/m41/m42 propagate outcomes → substrate pathway weights update → m31 reads updated weights → m31 selection distribution shifts → m20-m22 iterators see different selection distribution as input.

This is a **slow loop**: substrate pathway weight convergence happens over many sessions. No module in Cluster H should expect m31 to change behaviour within a single session. The loop timescale is days to weeks.

### m32 → m40/m41/m42 event fan

m32 fires a dispatch event for every workflow step that begins execution. Cluster H receives this event and decides independently:

- m40: emit to SYNTHEX always (all steps)
- m41: emit to LCM only when the step is deploy-shaped (see m41 §Detection)
- m42: emit to POVM always, with outcome from the step result

### Conductor wire vs LCM wire

This distinction is handled by m32 (dispatcher), not Cluster H. From Cluster H's perspective:

- m40 sees every dispatch event and emits regardless of which conductor wire m32 used
- m41 only routes to LCM when m32 has flagged the step as `StepKind::Deploy`

---

## m40 — `nexus_event_emitter`

**LOC estimate:** ~150 | **Boilerplate:** m22_synthex_bridge.rs (~90%) + m22_synthex_async.rs (~95%)

### Responsibility

Emit a typed `WorkflowEvent` to SYNTHEX v2 at `127.0.0.1:8092/v3/nexus/push` for every workflow lifecycle transition: promote (entry into bank), run (each dispatch step), decay (m11 sunset or m31 weight drop). SYNTHEX then applies its own Hebbian reinforcement or LTD to the event, contributing to `substrate_LTP_density` (Hebbian v3 reconciliation note target: >0.015 Phase 1).

### WorkflowEvent enum

```rust
/// Typed workflow lifecycle events emitted to SYNTHEX via the Nexus Bus.
///
/// Option A (recommended): the enum is serialized into `NexusEvent.data` as an
/// untyped `serde_json::Value`. SYNTHEX sees the `event_type` discriminator
/// and dispatches internally. No co-change required on the SYNTHEX side.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkflowEvent {
    /// A workflow has been promoted from the proposer into the bank (m30).
    Promote {
        /// Workflow ID from the bank (opaque, m30 schema).
        id: String,
        /// Lineage chain: IDs of ancestor workflows this was derived from.
        lineage: Vec<String>,
    },
    /// A workflow step has been dispatched and its outcome is known.
    Run {
        /// Workflow ID from the bank.
        id: String,
        /// Outcome of the run.
        outcome: RunOutcome,
    },
    /// A workflow's weight has dropped below the floor (m11/m31 decay).
    Decay {
        /// Workflow ID.
        id: String,
        /// Weight at time of decay trigger.
        weight: f64,
    },
}

/// Outcome of a single workflow dispatch.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunOutcome {
    /// All steps completed successfully.
    Pass,
    /// Steps completed and were independently verified (m33).
    PassVerified,
    /// One or more steps failed.
    Fail,
    /// Dispatch was blocked (escape-surface raised, Conductor refused).
    Blocked,
}
```

### OutboxEnvelope

```rust
/// A durable outbox record for a single WorkflowEvent.
///
/// Written to the outbox JSONL file before HTTP delivery is attempted.
/// On successful HTTP delivery the record is marked `posted = true`.
/// Failed deliveries remain in the outbox and are retried by the
/// background sweep task (interval: 60s, max_attempts: 5).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutboxEnvelope {
    /// Monotonically increasing envelope ID (for dedup).
    pub id: u64,
    /// Unix timestamp (seconds) of envelope creation.
    pub created_at: u64,
    /// The event payload.
    pub event: WorkflowEvent,
    /// Number of HTTP delivery attempts so far.
    pub attempts: u32,
    /// Whether the HTTP delivery succeeded.
    pub posted: bool,
}
```

### Dual-transport: outbox-first, HTTP fire-and-forget

This is the same pattern as m22_synthex_bridge.rs (90% reuse):

```
emit(event)
  │
  ├─ 1. serialize event → OutboxEnvelope
  ├─ 2. append to outbox JSONL file (durable)
  ├─ 3. attempt HTTP POST to 127.0.0.1:8092/v3/nexus/push
  │       ├─ success: mark envelope posted=true in outbox
  │       └─ failure: log + leave in outbox for retry sweep
  └─ 4. return EmitOutcome
```

HTTP delivery is **always fire-and-forget**: the engine does not wait for SYNTHEX to acknowledge. The circuit breaker (see §Circuit breaker) governs whether to skip the HTTP attempt when SYNTHEX is known-down.

```rust
/// Tally of a single emit call.
#[derive(Debug, Clone, Default)]
pub struct EmitOutcome {
    /// Whether the envelope was appended to the outbox JSONL file.
    pub appended: bool,
    /// Whether the HTTP POST to SYNTHEX succeeded.
    pub posted: bool,
    /// True if the call produced any error (outbox append or HTTP).
    pub failed: bool,
}
```

### Option A vs Option B: type strategy recommendation

**Option A (recommended for initial build):** `WorkflowEvent` is serialized into `serde_json::Value` and placed in the existing `NexusEvent.data` field. `NexusEvent.event_type` carries the discriminator string (`"workflow_promote"`, `"workflow_run"`, `"workflow_decay"`). The `NexusEvent` struct is re-declared locally in workflow-trace — it does NOT import synthex-v2.

Rationale: SYNTHEX v2's `/v3/nexus/push` endpoint accepts `NexusEvent { event_type: String, ts: u64, data: serde_json::Value }`. Any JSON shape is accepted in `data`. No co-change on the synthex-v2 side is required, and the schema evolution risk is isolated to the `data` field.

**Option B (future hardening, S118-pattern):** `WorkflowEvent` is promoted to a typed enum shared via a common crate that both workflow-trace and synthex-v2 import. This requires a co-change in synthex-v2 to add inbound dispatch routing for the new event kind. The migration gate is: add a round-trip integration test (S118 pattern) that sends a `WorkflowEvent::Run` and verifies SYNTHEX echoes the correct `event_type` on pull. Only migrate to Option B when that test passes.

### NexusEvent re-declaration (local copy, not import)

```rust
/// Local re-declaration of SYNTHEX's NexusEvent shape.
///
/// DO NOT import from synthex_v2::*. This struct is intentionally
/// re-declared so workflow-trace has no compile-time dependency on
/// the synthex-v2 crate. The wire shape is identical.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NexusEvent {
    /// Event type discriminator (e.g. "workflow_promote").
    #[serde(rename = "type")]
    pub event_type: String,
    /// Unix timestamp (seconds).
    pub ts: u64,
    /// Event-specific payload (untyped JSON, Option A).
    pub data: serde_json::Value,
}

/// Envelope for a batch POST to /v3/nexus/push.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NexusPushEnvelope {
    pub events: Vec<NexusEvent>,
}
```

### Schema evolution risk and mitigation

`NexusEvent.data` is untyped JSON. If SYNTHEX v2 changes its inbound dispatch shape (e.g., renames the `kind` discriminator field, adds a required `session_id` field), m40 emissions could silently produce events that SYNTHEX ignores or misroutes. This is the schema drift risk.

Mitigations:
1. **S118-pattern round-trip test** (minimum viable): a `#[tokio::test]` that calls `emit(WorkflowEvent::Run { ... })` against a mock SYNTHEX endpoint, then pulls via `/v3/nexus/pull` and asserts the event appears with the correct `event_type`. This catches shape drift on each PR.
2. **Outbox survival check**: because the outbox JSONL is durable, any SYNTHEX endpoint change that causes 4xx responses will leave envelopes in the outbox with `posted=false`. The retry sweep will surface these as metrics (`emit_failed_total` counter), which is observable without schema validation.
3. **Option B migration** (future): when the typed enum path is ready, the round-trip test becomes a contract test that exercises the full codec path. Until then, Option A's untyped JSON is the intended operating mode.

### Circuit breaker (m40)

Lifted from m22_synthex_async.rs (95%):

```
States: Closed → Open → HalfOpen → Closed

Closed  (normal):
  - HTTP attempts proceed normally
  - On SUCCESS: reset failure_count → 0
  - On FAILURE or TIMEOUT: failure_count += 1
  - When failure_count >= OPEN_THRESHOLD (default: 5): → Open

Open (SYNTHEX known-down):
  - HTTP attempts are SKIPPED entirely
  - Outbox write still proceeds (event is durable)
  - After OPEN_DURATION (default: 60s): → HalfOpen

HalfOpen (probe):
  - ONE HTTP attempt is permitted
  - On SUCCESS: → Closed, reset failure_count → 0
  - On FAILURE or TIMEOUT: → Open, restart timer
```

Exponential backoff with ±25% jitter is applied to retry sweep intervals, NOT to individual HTTP calls (which are fire-and-forget). The circuit breaker governs whether to attempt the POST at all.

**When SYNTHEX is down:** the engine continues dispatching workflows. m40 appends to the outbox (durable JSONL), logs at `WARN` level, increments `emit_circuit_open_total` counter, and returns `EmitOutcome { appended: true, posted: false, failed: false }`. The emit is not an error from the engine's perspective.

---

## m41 — `lcm_rpc_client`

**LOC estimate:** ~200 | **Boilerplate:** m22_synthex_async.rs (~95%) + m38_deployment_api.rs (~50%) + lcm_supervisor.rs (client-side mirror)

### Responsibility

When m32 dispatches a **deploy-shaped** workflow step, m41 wraps it as an LCM RPC call to the LCM supervisor daemon at `$XDG_RUNTIME_DIR/lcm/supervisor.sock`. The LCM supervisor owns the deploy state machine; m41 is a thin client that never re-implements state.

### Deploy-shape detection

m32 marks each step with a `StepKind` enum before dispatch. m41 checks `StepKind::Deploy` and only activates for those steps. The detection logic lives in m32, not m41. m41's only question is: "was this step flagged `Deploy`?".

Steps that are deploy-shaped (examples from the habitat vocabulary):
- `cargo install <binary>`
- `devenv start <service-id>`
- Binary deployment to `~/.local/bin/`
- `devenv restart <service-id>`

Steps that are NOT deploy-shaped (handled by Conductor wire in m32):
- Cargo check/clippy/test (quality gate steps)
- File edits
- Database queries
- HTTP probes

### Conductor wire vs LCM wire — when each applies

| Step type | Dispatch wire | Cluster H activation |
|---|---|---|
| Quality gate (check, clippy, test) | Conductor enforcement | m40 only |
| File edit / read | Conductor enforcement | m40 only |
| Binary deploy / `devenv start` | LCM RPC | m40 + m41 |
| `cargo install` | LCM RPC | m40 + m41 |
| HTTP probe / curl | Conductor enforcement | m40 only |

The Conductor wire (m32's primary path) handles all workflow steps. LCM is an additional routing layer applied specifically when the step is classified as a deployment operation. m41 does not replace m32's Conductor dispatch — it supplements it with LCM lifecycle tracking.

### RPC wire envelope

The LCM supervisor uses JSON-RPC 2.0 over a Unix domain socket, newline-framed. One request per `\n`-terminated line, one response per `\n`-terminated line. Read timeout: 30 seconds (mirroring lcm_supervisor.rs `READ_TIMEOUT`).

For deploy-shaped workflow steps, m41 uses `lcm.loop.create` (not a hypothetical `lcm.deploy`). The rationale: LCM's fundamental primitive is a loop with bounded iterations. A deploy workflow step maps naturally to a single-iteration loop (`max_iters: 1`) with a `name` derived from the workflow step description. This keeps m41 compatible with LCM's existing RPC surface without requiring new methods.

```rust
/// Wire request for creating an LCM loop from a workflow deploy step.
#[derive(Debug, serde::Serialize)]
struct LcmLoopCreateRequest {
    jsonrpc: &'static str,   // always "2.0"
    id: u64,                 // monotonic request ID
    method: &'static str,    // always "lcm.loop.create"
    params: LcmLoopCreateParams,
}

/// Parameters for lcm.loop.create from a workflow deploy step.
#[derive(Debug, serde::Serialize)]
struct LcmLoopCreateParams {
    /// Caller session ID (current workflow run ID).
    caller_id: String,
    /// Human-readable step name (e.g. "deploy:weaver:0.3.1").
    name: String,
    /// Always 1 for single-shot deploy steps.
    max_iters: u32,
    /// Deploy steps do not survive session death.
    survives_session_death: bool,
}

/// Wire response after successful lcm.loop.create.
#[derive(Debug, serde::Deserialize)]
struct LcmLoopCreateResponse {
    result: LcmLoopCreateResult,
}

#[derive(Debug, serde::Deserialize)]
struct LcmLoopCreateResult {
    loop_id: String,
}
```

Full exchange:

```
// Request (newline-terminated)
{"jsonrpc":"2.0","id":1,"method":"lcm.loop.create",
 "params":{"caller_id":"wf-abc123","name":"deploy:weaver:0.3.1",
           "max_iters":1,"survives_session_death":false}}\n

// Response (newline-terminated)
{"jsonrpc":"2.0","id":1,"result":{"loop_id":"a3f2..."}}\n
```

For status polling (used by the retry/verification path if needed):

```
// Request
{"jsonrpc":"2.0","id":2,"method":"lcm.loop.status",
 "params":{"loop_id":"a3f2..."}}\n

// Response
{"jsonrpc":"2.0","id":2,"result":{"loop_id":"a3f2...",
 "state":"Running","created_at_unix_nanos":1716000000000000000}}\n
```

### Circuit breaker (m41)

Same pattern as m40 (m22_synthex_async.rs lineage), adapted for the UDS transport:

```
Closed  → attempt UDS connect + send → on 5 consecutive failures → Open
Open    → skip attempts for 60s → HalfOpen
HalfOpen → send lcm.ping → success: Closed / failure: Open
```

**When LCM is down:** m41 logs at `WARN`, records the step as `LcmOutcome::ServiceUnavailable`, and returns without blocking. The workflow step's Conductor-side execution is not affected. The missing LCM registration is an advisory tracking gap, not a failure. m41 never blocks the dispatch.

### LcmDispatchOutcome

```rust
/// Result of attempting to route a deploy step through LCM.
#[derive(Debug, Clone)]
pub enum LcmDispatchOutcome {
    /// LCM accepted the loop creation; `loop_id` is the LCM-assigned identifier.
    Registered { loop_id: String },
    /// Step is not deploy-shaped; LCM routing was skipped.
    NotApplicable,
    /// LCM supervisor is unreachable or circuit is open.
    ServiceUnavailable,
    /// LCM responded with an error (RPC-level, not connection-level).
    RpcError { code: i32, message: String },
    /// UDS timeout (30s read timeout exceeded).
    Timeout,
}
```

---

## m42 — `hebbian_feedback`

**LOC estimate:** ~100 | **Boilerplate:** m24_povm_bridge.rs (~85% GOLD STANDARD)

### Responsibility

After every workflow dispatch event (outcome known), m42 calls `POST /reinforce` on POVM Engine at `127.0.0.1:8125` with a `workflow_trace_*`-namespaced reinforcement signal. During the POVM overlap period (now → 2026-07-10), this writes Hebbian pathway data to POVM. After the cutover, m42 routes to stcortex via m13 (`stcortex_writer_narrowed`).

The reinforcement feeds POVM's learning loop, contributing to `substrate_LTP_density`. Per the Hebbian v3 reconciliation note, Phase 1 target is `substrate_LTP_density > 0.015`.

### AP30 namespace prefix

The AP30 anti-pattern is namespace collision between workflow-trace pathways and ORAC/VMS pathways already in POVM. V3 owns `P01..P16` Hebbian pathway IDs. m42 avoids this by prefixing all pathway IDs with `workflow_trace_`:

```
pathway source: workflow_trace_<workflow_id>
pathway target: workflow_trace_outcome_<outcome_variant>

examples:
  workflow_trace_wf-abc123  →  workflow_trace_outcome_pass_verified
  workflow_trace_wf-abc123  →  workflow_trace_outcome_fail
```

This namespace is the same one m9 (`watcher_namespace_guard`) enforces at write-time. m42 relies on m9 being wired into the write path.

### POST /reinforce payload

From m24_povm_bridge.rs (GOLD STANDARD per Command-3 E3), the reinforcement endpoint shape. BUG-033 fix applies: socket address is raw `host:port`, no `http://` prefix:

```rust
/// Reinforcement payload for POST /reinforce.
///
/// Namespace convention: all pathway IDs prefixed `workflow_trace_`
/// to avoid AP30 collision with V3's P01..P16 and VMS pathways.
#[derive(Debug, serde::Serialize)]
pub struct ReinforcePayload {
    /// Session identifier (current workflow run ID).
    pub session_id: String,
    /// Fitness delta calculated from the workflow outcome.
    pub fitness_delta: f64,
    /// Pathway IDs to reinforce or depress.
    pub retrieval_ids: Vec<String>,
    /// Optional 1h idempotency key (prevents double-reinforcement on retry).
    pub request_id: Option<String>,
}
```

Wire call (following m24_povm_bridge.rs `raw_http_post` pattern):

```rust
// BUG-033: addr is "127.0.0.1:8125" — no http:// prefix
// F-001 silent-swallow fix: log the HTTP status code, don't ignore it
// BUG-034: this endpoint is /reinforce, not /memories or /pathways
raw_http_post("127.0.0.1:8125", "/reinforce", payload_bytes, "povm")
```

### fitness_delta calculation

The fitness delta is calculated from the workflow outcome. The goal is to reinforce outcomes that are genuinely better (PASS + VERIFY > PASS > BLOCKED > FAIL) and to produce meaningful LTP signals without over-saturating POVM:

| Outcome | fitness_delta | Rationale |
|---|---|---|
| `RunOutcome::PassVerified` | `+0.25` | Best possible signal: dispatched AND independently verified by m33 |
| `RunOutcome::Pass` | `+0.15` | Standard success |
| `RunOutcome::Blocked` | `-0.05` | Step was refused by Conductor — mild negative, may be appropriate |
| `RunOutcome::Fail` | `-0.10` | Step failed — stronger LTD signal |

These values are constants defined in the module, not hardcoded in the call sites:

```rust
const FITNESS_PASS_VERIFIED: f64 = 0.25;
const FITNESS_PASS: f64 = 0.15;
const FITNESS_BLOCKED: f64 = -0.05;
const FITNESS_FAIL: f64 = -0.10;
```

The delta is clamped to `[-1.0, 1.0]` before sending, matching the Hebbian v3 clamp pattern.

### POVM deprecation cutover — dual-path implementation

This is the most important invariant for m42. The POVM overlap window ends 2026-07-10. m42 must implement both paths:

```rust
/// Route a reinforcement signal to the appropriate substrate.
///
/// During the POVM overlap window (→ 2026-07-10): write to POVM.
/// After cutover: write to stcortex via m13.
///
/// The cutover flag is read from the module config at startup.
/// Default: `povm_overlap_active = true` (safe default — prefers POVM
/// until explicitly flipped, avoiding accidental stcortex writes before
/// m13 is wired).
pub fn route_reinforcement(
    &self,
    payload: ReinforcePayload,
) -> ReinforceOutcome {
    if self.config.povm_overlap_active {
        self.post_to_povm(payload)
    } else {
        self.write_to_stcortex_via_m13(payload)
    }
}
```

The stcortex path (post-cutover) maps the reinforcement payload to a stcortex `write_memory` call in the `workflow_trace_*` namespace. The stcortex namespace convention carries over from POVM: `workflow_trace_<workflow_id>_<outcome>`.

**Important:** the stcortex path is only activated when `povm_overlap_active = false`. If stcortex is unreachable and POVM overlap has ended, m42 logs at `ERROR` and returns `ReinforceOutcome::SubstrateUnavailable`. It does NOT silently fall back to POVM after the cutover (per CLAUDE.md stcortex policy: "do not silently fall back to POVM").

### ReinforceOutcome

```rust
/// Result of a reinforcement call.
#[derive(Debug, Clone)]
pub enum ReinforceOutcome {
    /// Reinforcement was accepted by the substrate.
    Accepted,
    /// POVM overlap has ended; routed to stcortex.
    RoutedToStcortex,
    /// Substrate is unreachable.
    SubstrateUnavailable,
    /// Substrate responded with an error.
    SubstrateError { status: u16, body: String },
    /// Reinforcement was skipped (e.g., idempotency dedup hit).
    Skipped,
}
```

### Circuit breaker (m42)

Same m24_povm_bridge.rs pattern: raw TCP socket, no HTTP client library. The circuit breaker operates on consecutive failures to `127.0.0.1:8125`:

```
Closed  → POST /reinforce → on 5 failures → Open
Open    → skip POST for 60s → HalfOpen
HalfOpen → health probe GET /health → success: Closed / failure: Open
```

**When POVM is down:** m42 logs at `WARN`, returns `ReinforceOutcome::SubstrateUnavailable`, and does not block. Missed reinforcements are not retried (reinforcement is idempotent-on-outcome, not critical-for-correctness). The engine's dispatch continues.

---

## Circuit breaker state diagram (shared pattern)

All three modules (m40/m41/m42) use the same circuit breaker state machine lifted from m22_synthex_async.rs:

```
         ┌─────────────────────────────────────────┐
         │              CLOSED                     │
         │  (normal — attempts proceed)            │
         │                                         │
         │  on SUCCESS: reset failure_count = 0    │
         │  on FAILURE/TIMEOUT: failure_count += 1 │
         └──────────────┬──────────────────────────┘
                        │ failure_count >= 5
                        ▼
         ┌─────────────────────────────────────────┐
         │               OPEN                      │
         │  (substrate known-down — skip attempts) │
         │                                         │
         │  outbox/log still written (durable)     │
         │  after OPEN_DURATION (60s): → HalfOpen  │
         └──────────────┬──────────────────────────┘
                        │ 60s elapsed
                        ▼
         ┌─────────────────────────────────────────┐
         │             HALF-OPEN                   │
         │  (one probe attempt allowed)            │
         │                                         │
         │  on SUCCESS: → Closed, reset count = 0  │
         │  on FAILURE/TIMEOUT: → Open, restart    │
         └─────────────────────────────────────────┘
```

Exponential backoff with ±25% jitter is applied to the retry sweep task interval (for m40's outbox), not to individual HTTP/UDS calls. Individual calls are attempted once and either succeed or update the failure counter.

Each module exposes circuit breaker state as a metric:

```rust
// Counters incremented per module
emit_circuit_open_total      // m40: times Nexus push was skipped (circuit open)
lcm_circuit_open_total       // m41: times LCM was skipped (circuit open)
reinforce_circuit_open_total // m42: times POVM was skipped (circuit open)
```

---

## Error taxonomy

Following ME v2 gold standard (`error.rs`), each module defines its own error variants using `thiserror`:

```rust
// m40
#[derive(Debug, thiserror::Error)]
pub enum EmitError {
    #[error("outbox append failed: {0}")]
    OutboxIo(std::io::Error),
    #[error("serialize failed: {0}")]
    Serialize(serde_json::Error),
    #[error("circuit open — SYNTHEX unreachable")]
    CircuitOpen,
    #[error("HTTP post failed: status {status}")]
    HttpError { status: u16 },
}

// m41
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("UDS connect failed: {0}")]
    Connect(std::io::Error),
    #[error("read timeout after 30s")]
    ReadTimeout,
    #[error("JSON-RPC error {code}: {message}")]
    RpcError { code: i32, message: String },
    #[error("circuit open — LCM unreachable")]
    CircuitOpen,
}

// m42
#[derive(Debug, thiserror::Error)]
pub enum ReinforceError {
    #[error("POVM unreachable")]
    PovmUnreachable,
    #[error("stcortex unavailable (post-cutover)")]
    StcortexUnavailable,
    #[error("serialize failed: {0}")]
    Serialize(serde_json::Error),
    #[error("circuit open")]
    CircuitOpen,
}
```

Errors in all three modules are logged via `tracing::warn!` or `tracing::error!` but NOT propagated to the caller in a way that blocks dispatch. The dispatch pipeline in m32 must not fail because a substrate write failed.

---

## Structured logging (m40 outbox + m41 RPC trace)

Following ME v2 `logging.rs` pattern, structured fields on every log line:

```rust
// m40 outbox append success
tracing::debug!(
    workflow_id = %envelope.event.id(),
    event_kind = %envelope.event.kind_str(),
    envelope_id = envelope.id,
    "WorkflowEvent appended to outbox"
);

// m40 HTTP post success
tracing::info!(
    workflow_id = %event.id(),
    event_kind = %event.kind_str(),
    "WorkflowEvent posted to SYNTHEX nexus bus"
);

// m40 circuit open skip
tracing::warn!(
    workflow_id = %event.id(),
    "SYNTHEX circuit open — WorkflowEvent queued in outbox only"
);

// m41 LCM RPC registered
tracing::info!(
    workflow_id = %workflow_id,
    step_name = %step_name,
    loop_id = %loop_id,
    "Deploy step registered with LCM supervisor"
);

// m41 LCM not applicable
tracing::debug!(
    workflow_id = %workflow_id,
    step_kind = ?step_kind,
    "Step is not deploy-shaped — LCM routing skipped"
);

// m42 POVM reinforce success
tracing::info!(
    workflow_id = %payload.session_id,
    fitness_delta = payload.fitness_delta,
    pathway_count = payload.retrieval_ids.len(),
    "Hebbian reinforcement posted to POVM"
);
```

---

## Test coverage requirements (50+ per module)

### m40 test categories

1. `WorkflowEvent` enum serialization roundtrips (all 3 variants)
2. `OutboxEnvelope` serialization roundtrips
3. `EmitOutcome` accumulation (appended+posted, appended only, failed)
4. Dual-transport: outbox append before HTTP (order invariant)
5. Circuit breaker state transitions (Closed → Open → HalfOpen → Closed)
6. Circuit breaker: open skips HTTP, still appends outbox
7. Empty events skipped (no outbox write for zero-content)
8. Retry sweep: posted envelopes not re-attempted
9. Retry sweep: failed envelopes re-attempted up to max_attempts
10. NexusEvent local re-declaration matches wire shape (serde roundtrip vs expected JSON)
11. Schema drift detection: `event_type` field name must be `"type"` (serde rename)
12. Option A untyped JSON: `kind` discriminator preserved through serialization
13. `Send + Sync` bounds on emitter struct
14. Thread safety: concurrent emit calls don't corrupt outbox

### m41 test categories

1. `LcmLoopCreateRequest` serialization (JSON-RPC 2.0 shape)
2. `LcmLoopCreateResponse` deserialization (result.loop_id extraction)
3. Deploy-shape detection: `StepKind::Deploy` routes to LCM
4. Non-deploy step: `LcmDispatchOutcome::NotApplicable` returned without UDS attempt
5. Circuit breaker Closed/Open/HalfOpen transitions
6. Circuit breaker open: skip UDS, return `ServiceUnavailable`
7. Read timeout: 30s exceeded returns `LcmDispatchOutcome::Timeout`
8. RPC error response: `code` and `message` extracted into `RpcError`
9. Unknown method response: handled gracefully
10. Newline framing: request ends with `\n`, response parsed line-by-line
11. `lcm.ping` health probe used in HalfOpen state
12. `Send + Sync` bounds on client struct
13. Reconnect on each call (no persistent connection)

### m42 test categories

1. `ReinforcePayload` serialization roundtrip
2. AP30 namespace: all retrieval IDs start with `workflow_trace_`
3. fitness_delta constants: values match spec table
4. fitness_delta clamping: values outside [-1.0, 1.0] are clamped
5. Overlap window active: routes to POVM (`povm_overlap_active = true`)
6. Overlap window inactive: routes to stcortex (`povm_overlap_active = false`)
7. Post-cutover: stcortex unavailable → `SubstrateUnavailable`, NOT POVM fallback
8. Circuit breaker Closed/Open/HalfOpen transitions (POVM path)
9. POVM unreachable: returns `SubstrateUnavailable`, does not block
10. Idempotency: duplicate `request_id` within 1h → `Skipped`
11. BUG-033: socket addr has no `http://` prefix
12. BUG-034: endpoint is `/reinforce`, not `/pathways` or `/memories`
13. F-001 silent-swallow: non-200 status is logged, not silently dropped
14. `ReinforceOutcome` variants all constructed in tests
15. `Send + Sync` bounds

---

## Dependency manifest (no Cargo.toml — planning only)

| Dependency | Used by | Purpose |
|---|---|---|
| `serde` + `serde_json` | m40/m41/m42 | Serialization |
| `thiserror` | m40/m41/m42 | Error taxonomy |
| `tracing` | m40/m41/m42 | Structured logging |
| `tokio` | m40/m41 | Async runtime (spawn_blocking for HTTP) |
| `parking_lot` | m40/m41/m42 | RwLock for circuit breaker state |
| `std::os::unix::net` | m41 | Unix domain socket client |

No external HTTP client library. All HTTP is raw TCP (following m22_synthex_bridge.rs / m24_povm_bridge.rs gold standard). Raw TCP avoids the tokio `spawn_blocking` footgun documented in m22_synthex_async.rs R6.

---

## Implementation notes and traps

**AP29 (sync HTTP in tokio::spawn starves): acknowledged and mitigated.** m40's HTTP calls use `tokio::task::spawn_blocking` following m22_synthex_async.rs pattern. The 10s cap from m22_synthex_async.rs is reduced to 2s for workflow-trace (SYNTHEX nexus push is fire-and-forget; a 10s block would visibly stall dispatch). m41's UDS calls are similarly `spawn_blocking`-wrapped with the 30s read timeout from lcm_supervisor.rs.

**m41 has no persistent connection to LCM.** Each `dispatch()` call opens a new UDS connection, sends one request, reads one response, and closes. This is intentional: deploy steps are infrequent, and a persistent connection would require keepalive management that adds complexity without benefit.

**m42 idempotency dedup.** The 1h idempotency window from m24_povm_bridge.rs (citing povm-v2_reinforcement.rs) is replicated: a `request_id` field (UUID v4, generated per dispatch event) is sent with each reinforce call. POVM deduplicates within 1h. This prevents double-reinforcement if the engine retries a failed dispatch.

**Outbox JSONL location.** m40's outbox file lives at `{data_dir}/workflow_trace_outbox.jsonl`. The `data_dir` is resolved from the module config (same `DaemonConfig` pattern as m22_synthex_bridge.rs `with_config`). The file is append-only. Compaction (removing `posted=true` entries older than 24h) runs in the background sweep task.

**m42 does NOT read back from POVM.** m42 is write-only. Reading pathway weights back is done by m31 (selector) via stcortex during the overlap period, or directly via stcortex post-cutover. m42 has no read path and does not hydrate any local state from POVM.

---

## Four-surface persistence anchors

Per working-mode discipline (CLAUDE.local.md §Working Mode), major plans persist across four surfaces. This spec is surface one (vault canonical). The remaining three:

- **stcortex**: `workflow_trace_cluster_h_spec` namespace — write after G8 gate clears
- **POVM** (read-only during overlap): update existing `workflow_trace_*` pathway if present
- **CLAUDE.local.md**: add cluster-H anchor row to Active Workstreams table

---

*Cluster H spec · S1001982 · planning-only · HOLD-v2 active*
*authored: 2026-05-17 | back to [[HOME]] · [[Modules Synergy Clusters and Feature Verification S1001982]]*
