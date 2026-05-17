---
title: m41 — `m41_lcm_rpc` Rust spec
cluster: H — Substrate Feedback
layer: L8
binary: wf-crystallise
loc_estimate: ~140
test_count_min: 65
test_kinds: [async, integration]
feature_gate: [monitoring]
verb_class: emit
cc_owns: [CC-5 (emit-half)]
cc_consumes: [CC-2]
gap_owner: [none]
boilerplate_lift_pct: 35
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A
hold_v2_compliant: true
---

# m41 — `m41_lcm_rpc` Rust spec

> Back to: [vault cluster-H spec](../../../the-workflow-engine-vault/module%20specs/cluster-H-substrate-feedback.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · [V7 cluster-H plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-H.md) · [GENESIS v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · vault [[cluster-H-substrate-feedback]]
>
> Sister anchors: [m40](m40_nexusevent_emit.md) · [m41](m41_lcm_rpc.md) · [m42](m42_stcortex_emit.md)

## 1. Purpose & invariants

`m41_lcm_rpc` IS the deploy-shaped-step routing path from workflow-trace to the LCM (Loop Engine V2) supervisor. When m32 dispatches a workflow step that has been classified by its `StepKind::Deploy` discriminator (binary deploys, `devenv start`, `cargo install`, etc.), m41 wraps the step as an **`lcm.loop.create`** JSON-RPC 2.0 call to the LCM supervisor daemon at `$XDG_RUNTIME_DIR/lcm/supervisor.sock`. The LCM supervisor owns the deploy lifecycle state machine; m41 is a *thin client* that never re-implements that state. Non-deploy steps are ignored — m41 returns `LcmDispatchOutcome::NotApplicable` without touching LCM.

The module MUST guarantee five invariants. First, **the RPC method is `lcm.loop.create` with `max_iters: 1`** — NOT a hypothetical `lcm.deploy`. The vault cluster-H spec explicitly flags `lcm.deploy` as a wrong endpoint that would break compat with LCM's existing 9-RPC surface (per [V7 cluster-H plan § m41 boilerplate-lift](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-H.md) and GOD_TIER_CONSOLIDATION Part I § Cluster H). A deploy step maps naturally to a single-iteration loop. Second, **outbox-first JSONL durability**: the request envelope is written to `{data_dir}/m41/outbox.jsonl` and `fsync`ed before the UDS call is attempted; LCM-down NEVER blocks dispatch. Third, **circuit-breaker discipline**: shared `m40_42_common::Breaker` (peer = LCM supervisor) with 5-fail-Open / 60s-HalfOpen; on HalfOpen the probe is `lcm.ping`, not a synthetic `loop.create`. Fourth, **30s read timeout** mirrored from `lcm_supervisor.rs::READ_TIMEOUT`; on timeout return `LcmDispatchOutcome::Timeout` without retry from m41 (LCM idempotency on `caller_id`+`name` is supervisor-side). Fifth, **no persistent connection**: each `dispatch()` opens a new UDS, sends one request, reads one response, closes — deploys are infrequent, keepalive complexity has no payoff.

Frame violations m41 must structurally refuse: (a) routing non-deploy steps through LCM (over-routing — `shape_predicate` MUST gate every call); (b) inferring deploy-shape on the client side without `m32::StepKind::Deploy` flag (m32 owns the classification, m41 owns the routing); (c) re-implementing LCM state (e.g., tracking `loop_id` lifecycle, polling for completion) — `lcm.loop.status` is permitted but is read-only and bounded to one call per `dispatch`; (d) blocking m32's Conductor wire — m41 supplements m32's Conductor dispatch with LCM lifecycle tracking, it does NOT replace it.

The module is `cfg(povm_calibrated)`-gated at the crate root by m8; m9 namespace-guard does not engage (m41 writes neither stcortex nor POVM; LCM has its own namespace).

## 2. Public surface (Rust types — spec only, NOT compileable)

```rust
//! # m41_lcm_rpc
//!
//! - **Layer**: L8 (Substrate Feedback, Cluster H)
//! - **Deps**: tokio (rt + spawn_blocking), serde + serde_json, thiserror, tracing, parking_lot,
//!   std::os::unix::net::UnixStream, workflow_core::types::{WorkflowId, SessionId, StepKind},
//!   workflow_core::errors::RpcError, m40_42_common::breaker::{Breaker, BreakerState}
//! - **Tests**: 65 (30 unit + 5 property + 1 fuzz target + 15 integration + 5 contract + 4 regression + mutation ≥75%)
//! - **Features**: `monitoring` (enabled; exports counters `lcm_routed_total`,
//!   `lcm_skipped_not_applicable_total`, `lcm_circuit_open_total`, `lcm_timeout_total`)
//! - **Platform**: Linux; Unix domain socket; newline-framed JSON-RPC 2.0; 30s read timeout
//! - **Impl Notes**: socket path resolved from `$XDG_RUNTIME_DIR/lcm/supervisor.sock`;
//!   `lcm.loop.create` method (NOT `lcm.deploy` — preserves LCM's 9-RPC surface);
//!   `max_iters: 1` for single-shot deploy semantics; no persistent connection
//! - **Related Docs**: [vault cluster-H](../../../the-workflow-engine-vault/module%20specs/cluster-H-substrate-feedback.md)
//!   · [V7 cluster-H plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-H.md)
//!   · [ULTRAMAP](../../../ai_docs/optimisation-v7/ULTRAMAP.md) View 2 row m41

/// Wire request for `lcm.loop.create`.
#[derive(Debug, serde::Serialize)]
pub struct LcmLoopCreateRequest {
    pub jsonrpc: &'static str,             // always "2.0"
    pub id: u64,                           // monotonic request id
    pub method: &'static str,              // always "lcm.loop.create"
    pub params: LcmLoopCreateParams,
}

#[derive(Debug, serde::Serialize)]
pub struct LcmLoopCreateParams {
    pub caller_id: SessionId,              // current workflow run id
    pub name: String,                      // e.g. "deploy:weaver:0.3.1"
    pub max_iters: u32,                    // always 1 for deploy-shaped
    pub survives_session_death: bool,      // always false for deploy steps
}

#[derive(Debug, serde::Deserialize)]
pub struct LcmLoopCreateResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<LcmLoopCreateResult>,
    pub error: Option<LcmRpcErrorBody>,
}

#[derive(Debug, serde::Deserialize)]
pub struct LcmLoopCreateResult { pub loop_id: String }

#[derive(Debug, serde::Deserialize)]
pub struct LcmRpcErrorBody { pub code: i32, pub message: String }

/// Outcome of one routing attempt.
#[derive(Debug, Clone)]
pub enum LcmDispatchOutcome {
    Registered { loop_id: String },
    NotApplicable,                         // step was not StepKind::Deploy
    ServiceUnavailable,                    // UDS connect failed OR breaker Open
    RpcError { code: i32, message: String },
    Timeout,                               // 30s read timeout exceeded
}

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("UDS connect failed: {0}")]
    Connect(#[source] std::io::Error),
    #[error("read timeout after 30s")]
    ReadTimeout,
    #[error("serialize failed: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("JSON-RPC error {code}: {message}")]
    RpcBody { code: i32, message: String },
    #[error("outbox append failed: {0}")]
    OutboxIo(#[source] std::io::Error),
    #[error("circuit open — LCM unreachable")]
    CircuitOpen,
}

#[derive(Debug, Clone)]
pub struct LcmRpcClientConfig {
    pub socket_path: std::path::PathBuf,   // $XDG_RUNTIME_DIR/lcm/supervisor.sock
    pub outbox_path: std::path::PathBuf,   // <data_dir>/m41/outbox.jsonl
    pub read_timeout_secs: u64,            // default 30 (mirrors LCM supervisor)
    pub fail_threshold: u8,                // default 5
    pub open_duration_ms: u64,             // default 60_000
}

impl Default for LcmRpcClientConfig { /* ... */ }

pub struct LcmRpcClient { /* private: config, breaker, outbox, next_id (AtomicU64) */ }

impl LcmRpcClient {
    pub fn with_config(config: LcmRpcClientConfig) -> Result<Self, RpcError>;

    /// Routes a step to LCM iff `StepKind::Deploy`. Non-deploy returns `NotApplicable`
    /// without touching the wire. Outbox is written before any UDS attempt.
    pub async fn dispatch(
        &self,
        workflow_id: WorkflowId,
        step_kind: StepKind,
        step_name: String,
    ) -> Result<LcmDispatchOutcome, RpcError>;

    /// HalfOpen probe (used internally by breaker recovery path).
    pub async fn ping(&self) -> Result<(), RpcError>;
}
```

## 3. Internal data structures

```rust
struct LcmRpcInner {
    config: LcmRpcClientConfig,
    breaker: Arc<Breaker>,             // peer = LCM supervisor
    outbox: Mutex<OutboxWriter>,
    next_id: AtomicU64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LcmOutboxEnvelope {
    id: u64,
    created_at: u64,
    workflow_id: WorkflowId,
    request: LcmLoopCreateParams,
    posted: bool,
    loop_id: Option<String>,           // populated on Registered
}
```

`shape_predicate(step_kind)` is a pure function: `step_kind == StepKind::Deploy → true`. No fuzzy matching, no string heuristics. m32 owns the classification.

## 4. Data flow

- **INPUT FROM:** `m32::DispatchOutcome` with `step_kind: StepKind` (Cluster G); m32 calls `client.dispatch(...)` after every dispatch event regardless of kind — m41 gates internally
- **OUTPUT TO:** `{data_dir}/m41/outbox.jsonl` (durable) and `$XDG_RUNTIME_DIR/lcm/supervisor.sock` (best-effort; LCM JSON-RPC 2.0 newline-framed)
- **SUBSTRATE TOUCHED:** LCM supervisor only (no SYNTHEX, no stcortex, no POVM)
- **WRITES:** outbox JSONL (always, even for `NotApplicable` — kept as audit trail of routing decisions); LCM RPC (only when `Deploy` and breaker Closed/HalfOpen)

## 5. Algorithm sketch

```text
dispatch(workflow_id, step_kind, step_name):
    # Cheap gate: non-deploy steps short-circuit
    if step_kind != StepKind::Deploy:
        metric!(lcm_skipped_not_applicable_total += 1)
        return Ok(LcmDispatchOutcome::NotApplicable)

    # Step 1: append outbox envelope (durable)
    params = LcmLoopCreateParams {
        caller_id: workflow_id.into_session_id(),
        name: step_name,
        max_iters: 1,
        survives_session_death: false,
    }
    envelope = LcmOutboxEnvelope { id: next_id.fetch_add(1), created_at: now_s(),
                                   workflow_id, request: params.clone(), posted: false, loop_id: None }
    outbox.append_jsonl(envelope).await?
    outbox.sync_data().await?

    # Step 2: circuit check
    if not breaker.allow().await:
        metric!(lcm_circuit_open_total += 1)
        return Ok(LcmDispatchOutcome::ServiceUnavailable)

    # Step 3: send (spawn_blocking, 30s cap)
    req = LcmLoopCreateRequest { jsonrpc: "2.0", id: envelope.id,
                                 method: "lcm.loop.create", params }
    line = serde_json::to_vec(&req)? ++ b"\n"

    result = tokio::time::timeout(30s, spawn_blocking(|| {
        let mut sock = UnixStream::connect(&config.socket_path)?
        sock.set_read_timeout(Some(Duration::from_secs(30)))?
        sock.write_all(&line)?
        let mut reader = BufReader::new(sock)
        let mut response_line = String::new()
        reader.read_line(&mut response_line)?
        Ok(response_line)
    })).await

    match result:
        Err(_elapsed) -> breaker.record_failure().await; metric!(lcm_timeout_total += 1); Ok(LcmDispatchOutcome::Timeout)
        Ok(Err(io)) -> breaker.record_failure().await; Ok(LcmDispatchOutcome::ServiceUnavailable)
        Ok(Ok(line)) ->
            response: LcmLoopCreateResponse = serde_json::from_str(&line)?
            match (response.result, response.error):
                (Some(r), None) -> breaker.record_success().await
                                   mark_envelope_posted(envelope.id, r.loop_id.clone())
                                   metric!(lcm_routed_total += 1)
                                   Ok(LcmDispatchOutcome::Registered { loop_id: r.loop_id })
                (None, Some(e)) -> breaker.record_failure().await
                                   Ok(LcmDispatchOutcome::RpcError { code: e.code, message: e.message })
                _ -> Err(RpcError::RpcBody { code: -32603, message: "malformed response" })

ping():
    # HalfOpen probe — send `lcm.ping` per LCM 9-RPC surface
    # bounded 5s; on success return Ok(()), on failure RpcError::ReadTimeout / Connect
```

## 6. Boilerplate lifts

Per V7 cluster-H plan § m41 § Boilerplate-lift source (Category 08 Nexus-LCM-RPC, gold standard per Command-3 E3):

| Source | Lift | % |
|---|---|---:|
| `m22_synthex_async.rs::Breaker` state machine | Closed/Open/HalfOpen 5-fail / 60s | 95% (shared via `m40_42_common::breaker`) |
| `m22_synthex_async.rs::spawn_blocking` wrapper | AP29 mitigation harness, 30s cap | 95% (cap raised 10s → 30s to mirror LCM `READ_TIMEOUT`) |
| `lcm_supervisor.rs::READ_TIMEOUT` + line framing | UnixStream + BufReader + `read_line` | 80% (client-side mirror) |
| `m38_deployment_api.rs::dispatch` glue | request id management, outbox pattern | 50% |
| `m24_povm_bridge.rs` outbox-append pattern | JSONL atomic append + `sync_data` | 70% |
| `shape_predicate` | — | 0% (novel ~5 LOC; pure `step_kind == Deploy`) |
| Newline-framed JSON-RPC 2.0 parser | — | 30% (newline-framing pattern lifted; JSON-RPC envelope wrapper novel) |

Net: ~115 LOC lifted / ~25 LOC novel (`shape_predicate` + envelope build + ping glue).

## 7. ME v2 m1_foundation patterns referenced

- `error.rs` — `RpcError` is a `thiserror::Error` enum with structured named-field variants (`RpcBody { code, message }`); callers can match programmatically rather than parsing display strings.
- `resources.rs` — `//!` module docstring block adopted verbatim.
- `shared_types.rs` — newtype discipline (`WorkflowId`, `SessionId`, `StepKind`); `LcmRpcClientConfig` follows `Default + env-override + override-field` pattern.
- `logging.rs` — tracing-subscriber structured emit on every `dispatch` (workflow_id, step_kind, outcome variant), every UDS connect (socket_path), every breaker state transition.

## 8. Test strategy

- **Test kind**: async (30 unit) + property (5) + 1 fuzz target + integration (15) + contract (5) + regression (4); total **65** (per V7 cluster-H plan § m41 § Test budget; matches MODULE_MATRIX row m41 65 tests)
- **Mutation budget**: ≥**75%** kill on `shape_predicate.rs` + `frame.rs` + breaker call sites (per V7 G6 § m41). Critical: shape_predicate false-positive routes non-deploy through LCM; false-negative drops deploy events on the floor — both classes must die.
- **Properties tested** (F-Property 5):
  - shape_predicate determinism: same `step_kind` → same routing decision under any concurrency
  - Frame parser invariant: any JSON-RPC 2.0 response line round-trips through `from_str` → `to_string` → `from_str` identity
  - Breaker idempotent under concurrent dispatch: parallel `record_failure` does not over-count
  - Cursor monotonic: `next_id` strictly increasing across `dispatch` calls
  - `max_iters` invariant: every constructed `LcmLoopCreateParams` has `max_iters == 1`
- **Fuzz target** (1): `m41_jsonrpc_frame` — feed arbitrary bytes to the newline-framed JSON-RPC parser; assert no panic, no UB (per V7 G6 § Fuzz enumeration — must not panic on malformed LCM response)

Key invariants (sample of 65):

1. Non-deploy step → `NotApplicable` without UDS connect attempt
2. Deploy step → opens UDS, sends `lcm.loop.create` request
3. RPC method is `"lcm.loop.create"` (NOT `"lcm.deploy"`) — regression slot
4. `max_iters == 1` always
5. `survives_session_death == false` always
6. Successful response → `Registered { loop_id }` and breaker `record_success`
7. RPC error body → `RpcError { code, message }` and breaker `record_failure`
8. UDS connect failure → `ServiceUnavailable` and breaker `record_failure`
9. 30s read timeout → `Timeout` and breaker `record_failure`
10. Breaker Open → `ServiceUnavailable` without UDS attempt
11. HalfOpen probe is `lcm.ping`, NOT `lcm.loop.create`
12. Each `dispatch` opens a NEW UDS connection (no keepalive)
13. Outbox envelope written even for `NotApplicable` (audit trail) — verify, then assess if too noisy in soak
14. Newline framing: request ends with `\n`, response parsed up to first `\n`
15. Malformed response → `RpcError::RpcBody { code: -32603 }`
16. Contract: `lcm.loop.create` accepted by live LCM supervisor (integration test gated `#[ignore = "requires lcm supervisor"]`)
17. Watcher Class-B test stub: every UDS attempt emits structured tracing
18. Watcher Class-C test stub: every `NotApplicable` skip emits structured tracing (refusal is correct)

(remaining 47 enumerated in vault cluster-H spec § Test coverage requirements)

## 9. Antipatterns to avoid

- **AP-V7-13** (Health-200 ≠ behaviour-verified) — m41 NEVER claims "deployed" on a successful `lcm.loop.create`; the loop is *registered*, not *completed*. Lifecycle ownership stays in LCM supervisor.
- **AP29** (sync HTTP / sync UDS in `tokio::spawn` starves) — mitigated by `spawn_blocking` + 30s cap; `tokio::time::timeout` wraps the spawn
- **AP-Drift-06** (bridge contract drift) — F-Contract tests against LCM 9-RPC surface; `bridge-contract` skill pre-merge; method-name regression slot reserved for `lcm.deploy` slip
- **Method-name drift** — `"lcm.loop.create"` is a regression-slot magic string; constant-extracted to `const LCM_METHOD: &str = "lcm.loop.create";` and tested
- **Over-routing** — `shape_predicate` is the only gate; no fallback heuristics
- **State re-implementation** — m41 does NOT track `loop_id` lifecycle, does NOT poll for completion; LCM owns the state machine
- **AP-Hab-04** (preserve-list discipline) — outbox compaction enumerates `posted=true && older_than_24h`; no blanket truncate

## 10. Useful patterns applied

- Outbox-first JSONL durability (PATTERNS.md § Substrate-write patterns)
- Circuit-breaker shared library (PATTERNS.md § Architectural patterns; `m40_42_common::breaker`)
- `spawn_blocking` + `tokio::time::timeout` wrapper (PATTERNS.md § Module-level patterns; AP29 mitigation)
- JSON-RPC 2.0 newline-framed wire (PATTERNS.md § Data-flow patterns)
- thiserror error enums (GOLD_STANDARDS rule 9)
- Newtype discipline (`StepKind`, `WorkflowId`, `SessionId`; GOLD_STANDARDS rule 8)

## 11. Cross-cluster contracts

- **CC-5 (OWNS emit-half via m40 / m41 / m42)**: m41 is the LCM-specific emit channel. Its contract is *every deploy-shaped step* gets registered with LCM as a 1-iteration loop; non-deploy steps are explicitly skipped (`NotApplicable` returned, audited via outbox). LCM's own lifecycle tracking carries forward into LCM's database; m41 does not read back.
- **CC-2 (consumes trust layer)**: m8 build-prereq gates the crate compilation on POVM-calibrated env; m9 namespace-guard does NOT engage (LCM has its own namespace); m10 Ember-CI-gate applies at source review.

## 12. Open questions for G5 interview / Zen G7 audit

1. **Outbox audit-vs-noise**: writing an envelope for every `NotApplicable` provides a full routing audit trail but inflates the outbox file. Threshold? Per-N-skips compaction? Or skip outbox for `NotApplicable`?
2. **`step_name` shape**: vault example is `"deploy:weaver:0.3.1"` — is this a hard convention? Does m32 supply the colon-delimited form, or does m41 build it? (Contract surface for the Conductor.)
3. **LCM 9-RPC surface stability**: this spec assumes `lcm.loop.create`, `lcm.loop.status`, `lcm.ping` are stable. If LCM extends to 10+ RPCs post-G9, what's the contract-drift detection cadence? Per-Wave-end `bridge-contract` run?
4. **`caller_id` collision risk**: m41 passes the `WorkflowId` as LCM's `caller_id`. LCM's idempotency dedup is `(caller_id, name)`-keyed — if the same workflow re-dispatches the same deploy step within idempotency-window, LCM may skip. Is that the desired semantic, or do we need per-attempt nonce?
5. **`survives_session_death = false` carve-out**: are there deploy steps that *should* survive a session crash? (e.g., long-running release builds that legitimately outlast the wf-crystallise binary.) Or is single-session bounding correct M0?

## 13. Implementation order (post-G9)

1. `errors.rs` — `RpcError` enum (`thiserror`); compile-only, no tests
2. `frame.rs` — newline-framed JSON-RPC 2.0 parser/writer; 8 unit tests (request shape, response parse, error body extract)
3. `shape_predicate.rs` — `StepKind::Deploy` gate; 4 unit tests (Deploy → true; all other variants → false)
4. `rpc_client.rs` — `LcmRpcClient::dispatch` + `ping` + UDS connect + `spawn_blocking` wrapper + 10 unit tests
5. `outbox.rs` — `LcmOutboxEnvelope` + writer; 6 unit tests (atomic append, `mark_envelope_posted`)
6. `mod.rs` — public surface + breaker integration; 2 unit tests (Outcome variants)
7. Property tests (5) — proptest on shape_predicate / frame parser / breaker / cursor / max_iters
8. Fuzz target (1) — `m41_jsonrpc_frame` JSON-RPC parser, 24h budget
9. Integration tests (15) — `tests/m41_integration.rs` with mock LCM UDS server (tokio `UnixListener`); also live supervisor contract test gated `#[ignore = "requires lcm supervisor"]`
10. Contract tests (5) — insta snapshots for `lcm.loop.create` request shape vs LCM 9-RPC surface
11. Regression slots (4) — reserved for first bugs (method-name drift `lcm.deploy`, `max_iters` != 1, persistent-connection leak, response without trailing `\n`)
12. Mutation pass — `cargo mutants` on `shape_predicate.rs` + `frame.rs` + `rpc_client.rs`; ≥75% kill required

---

> Back to: [vault cluster-H spec](../../../the-workflow-engine-vault/module%20specs/cluster-H-substrate-feedback.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · sister modules: [m40](m40_nexusevent_emit.md) · [m41](m41_lcm_rpc.md) · [m42](m42_stcortex_emit.md)
