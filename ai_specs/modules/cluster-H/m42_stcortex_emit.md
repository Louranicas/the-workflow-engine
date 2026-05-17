---
title: m42 — `m42_stcortex_emit` Rust spec (POVM-decoupled per 2026-05-17 ADR)
cluster: H — Substrate Feedback
layer: L8
binary: wf-crystallise
loc_estimate: ~150
test_count_min: 70
test_kinds: [async, integration]
feature_gate: [monitoring]
verb_class: emit
cc_owns: [CC-5 (emit-half; primary substrate-feedback channel)]
cc_consumes: [CC-2]
gap_owner: [none]
boilerplate_lift_pct: 30
status: SPEC (POVM **DECOUPLED**)
povm_decoupled: true
adr_ref: "2026-05-17-m42-stcortex-only-pivot.md"
date: 2026-05-17
authority: Luke @ node 0.A (12-round AskUserQuestion grilling; 48/48 Command recommendations accepted)
hold_v2_compliant: true
---

# m42 — `m42_stcortex_emit` Rust spec

> **POVM-DECOUPLED.** Per [2026-05-17 ADR](../../../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md), workflow-trace ships m42 stcortex-only from M0. POVM `:8125` is not a dependency. The src/ directory is `src/m42_stcortex_emit/` (NOT `src/m42_povm_dual/`).
>
> Back to: [vault cluster-H spec](../../../the-workflow-engine-vault/module%20specs/cluster-H-substrate-feedback.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · [V7 cluster-H plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-H.md) · [m42 ADR](../../../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md) · [GENESIS v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) Appendix A · vault [[cluster-H-substrate-feedback]]
>
> Sister anchors: [m40](m40_nexusevent_emit.md) · [m41](m41_lcm_rpc.md) · [m42](m42_stcortex_emit.md)

## 1. Purpose & invariants

`m42_stcortex_emit` IS the canonical substrate-feedback emit path from workflow-trace into the habitat's pioneer memory substrate, stcortex (SpacetimeDB module at `127.0.0.1:3000`). After every workflow dispatch event with a known outcome, m42 computes a `fitness_delta` (PassVerified +0.25 / Pass +0.15 / Blocked −0.05 / Fail −0.10) and emits a Hebbian pathway-write under the `workflow_trace_*` namespace via m13 (`stcortex_writer_narrowed`). Pathway-weight updates surface on m31's next selection cycle as part of CC-5's substrate learning loop (timescale: days/weeks). m42 is the *primary* substrate-feedback channel — m40 (SYNTHEX coordination) and m41 (LCM lifecycle) serve different concerns; m42 is the Hebbian-grain reinforcement path.

**The POVM decoupling (ADR-2026-05-17) is the load-bearing fact for this module.** Pre-ADR, m42 was specified as a POVM-dual-path emitter with a D25 mid-soak cutover dance and a Luke-physical-action POVM restart (D-B5). A live probe on 2026-05-17 found POVM `:8125` returning HTTP 200 on `/health` but serving a pre-CR-2 binary (`learning_health=0.9146` vs the post-CR-2 expected ~0.067) — the F7 "graceful-degrade pretend-fix" antipattern was active. Luke directed a 12-round AskUserQuestion grilling; 48/48 Command recommendations were accepted. The decision: m42 routes substrate-feedback to stcortex **exclusively** from M0. **No POVM read. No POVM write.** Workflow-trace starts fresh on stcortex's `workflow_trace_*` namespace (which begins empty per AP30). Other habitat services' POVM usage is unaffected; the workspace charter 2026-07-10 decommission still applies habitat-wide. Per ADR R3 + R10: this is permanent. Future Claude sessions evaluating "should we re-introduce POVM dependency in m42?" — answer: no, the ADR rejects it.

The crystallised lesson is **AP-V7-13: Health-200 ≠ behaviour-verified** (per [ANTIPATTERNS.md](../../../ANTIPATTERNS.md)). A 200 on `/health` is a liveness signal, not a behaviour-verification. m42 NEVER takes stcortex's HTTP-200 on a pathway-write as proof that pathway-weights actually moved — that verification happens on the substrate side via the Class-I Watcher monitor (rolling 7-day `stcortex.pathway.weight` delta on `workflow_trace_*` IDs); if delta == 0 for 4+ weeks, Class-I fires. This monitoring is m42's CC-5 closure-test surrogate (replacing the pre-pivot "POVM learning_health moved" assertion).

The module MUST guarantee six invariants. First, **stcortex-only routing**: there is no POVM branch, no `povm_overlap_active` config flag, no `route_reinforcement` switch — single code path to stcortex via m13. Second, **AP30 namespace prefix is mandatory and machine-enforced**: every pathway write has the form `workflow_trace_<workflow_id>` → `workflow_trace_outcome_<outcome>`; the `workflow_core::namespace::WORKFLOW_TRACE_PREFIX` constant is the only acceptable source for the prefix; m9 namespace-guard validates at write-time. Third, **fitness_delta constants are module-level `const`**: PassVerified +0.25 / Pass +0.15 / Blocked −0.05 / Fail −0.10 — not hardcoded at call sites. Clamping to `[−1.0, 1.0]` is post-compute defense in depth (Hebbian v3 clamp pattern). Fourth, **outbox-first JSONL durability**: stcortex-down NEVER blocks dispatch; the m13 write is preceded by an outbox append + `sync_data`. Per ADR R5 + CLAUDE.md stcortex policy, m42 NEVER silently falls back to POVM if stcortex is unreachable post-cutover (the cutover IS M0); instead it logs `ERROR`, increments `reinforce_substrate_unavailable_total`, returns `ReinforceOutcome::SubstrateUnavailable`, and the outbox carries the durable record for offline-JSONL-snapshot replay. Fifth, **AP-Hab-11 hyphen-slug discipline**: stcortex `pre_id`/`post_id` slugs replace hyphens with underscores (S1001757 munge bug); `workflow_trace_wf-abc-123` is converted to `workflow_trace_wf_abc_123` at the slug boundary. Sixth, **idempotency**: `request_id` (UUIDv4 per dispatch event) deduplicates within stcortex's idempotency window (default 1h, matching the m24_povm_bridge.rs pattern lift).

The module is `cfg(povm_calibrated)`-gated at the crate root by m8 (the gate-name is historical — the actual constraint is "POVM-calibrated env was the pre-pivot prereq"; post-ADR m8 may be retired or renamed in a future spec revision, tracked as open question 4 below). m9 wraps every write. m10 Ember-CI-gate applies at source review.

## 2. Public surface (Rust types — spec only, NOT compileable)

```rust
//! # m42_stcortex_emit
//!
//! - **Layer**: L8 (Substrate Feedback, Cluster H)
//! - **Deps**: tokio (rt + spawn_blocking), serde + serde_json, thiserror, tracing, parking_lot,
//!   uuid (v4 for request_id), workflow_core::types::{WorkflowId, SessionId},
//!   workflow_core::namespace::WORKFLOW_TRACE_PREFIX, workflow_core::errors::ReinforceError,
//!   m40_42_common::breaker::{Breaker, BreakerState}, m13_stcortex_writer (for the actual write)
//! - **Tests**: 70 (30 unit + 5 property + 0 fuzz + 15 integration + 5 contract + 4 regression + mutation ≥80%)
//! - **Features**: `monitoring` (enabled; exports counters `reinforce_accepted_total`,
//!   `reinforce_substrate_unavailable_total`, `reinforce_circuit_open_total`,
//!   `reinforce_namespace_violation_total`)
//! - **Platform**: Linux; raw HTTP via m13 to stcortex `:3000`; offline JSONL snapshot fallback per workspace charter
//! - **Impl Notes**: POVM-DECOUPLED per 2026-05-17 ADR; src/ path `src/m42_stcortex_emit/`;
//!   fitness_delta constants module-level; AP30 prefix machine-enforced;
//!   AP-Hab-11 hyphen-slug encoding applied at the slug boundary
//! - **Related Docs**: [ADR](../../../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md)
//!   · [vault cluster-H](../../../the-workflow-engine-vault/module%20specs/cluster-H-substrate-feedback.md)
//!   · [V7 cluster-H plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-H.md)
//!   · [GENESIS v1.3 Appendix A](../../../ai_docs/GENESIS_PROMPT_V1_3.md)
//!   · [ULTRAMAP](../../../ai_docs/optimisation-v7/ULTRAMAP.md) View 2 row m42

/// Outcome of a workflow dispatch (mirror of m32::DispatchOutcome relevant variant).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunOutcome { Pass, PassVerified, Fail, Blocked }

/// Hebbian fitness-delta constants. Module-level — NOT hardcoded at call sites.
///
/// Per [vault cluster-H spec § m42 § fitness_delta calculation] and
/// [V7 cluster-H plan § m42 § Purpose] — preserved 1:1 by the 2026-05-17 ADR.
pub const FITNESS_PASS_VERIFIED: f64 =  0.25;
pub const FITNESS_PASS:          f64 =  0.15;
pub const FITNESS_BLOCKED:       f64 = -0.05;
pub const FITNESS_FAIL:          f64 = -0.10;

/// Clamping bounds (Hebbian v3 defense-in-depth pattern).
pub const FITNESS_DELTA_MIN: f64 = -1.0;
pub const FITNESS_DELTA_MAX: f64 =  1.0;

#[inline]
pub fn fitness_delta_for(outcome: RunOutcome) -> f64 {
    let raw = match outcome {
        RunOutcome::PassVerified => FITNESS_PASS_VERIFIED,
        RunOutcome::Pass         => FITNESS_PASS,
        RunOutcome::Blocked      => FITNESS_BLOCKED,
        RunOutcome::Fail         => FITNESS_FAIL,
    };
    raw.clamp(FITNESS_DELTA_MIN, FITNESS_DELTA_MAX)
}

/// Reinforcement payload routed to stcortex via m13.
///
/// Namespace convention: pathway IDs prefixed `workflow_trace_` (AP30 enforced).
/// Slug convention: hyphens replaced with underscores (AP-Hab-11 enforced).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReinforcePayload {
    pub session_id: SessionId,             // current workflow run id (slug-encoded)
    pub fitness_delta: f64,                // clamped to [-1.0, 1.0]
    pub retrieval_ids: Vec<String>,        // all prefixed `workflow_trace_` (slug-encoded)
    pub request_id: uuid::Uuid,            // v4; 1h idempotency dedup
}

#[derive(Debug, Clone)]
pub enum ReinforceOutcome {
    /// Reinforcement was accepted by stcortex.
    Accepted,
    /// stcortex is unreachable; outbox carries the durable record for replay.
    /// NEVER falls back to POVM (CLAUDE.md stcortex policy).
    SubstrateUnavailable,
    /// stcortex responded with a non-2xx (e.g., schema mismatch).
    SubstrateError { status: u16, body: String },
    /// Idempotency dedup hit (request_id seen within 1h window).
    Skipped,
}

#[derive(Debug, thiserror::Error)]
pub enum ReinforceError {
    #[error("stcortex unreachable")]
    StcortexUnavailable,
    #[error("serialize failed: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("namespace guard: id={id:?} missing workflow_trace_ prefix")]
    NamespaceViolation { id: String },
    #[error("outbox append failed: {0}")]
    OutboxIo(#[source] std::io::Error),
    #[error("circuit open — stcortex unreachable")]
    CircuitOpen,
}

#[derive(Debug, Clone)]
pub struct StcortexEmitterConfig {
    pub addr: String,                      // "127.0.0.1:3000" (no http:// prefix — BUG-033 lineage)
    pub outbox_path: std::path::PathBuf,   // <data_dir>/m42/outbox.jsonl
    pub fail_threshold: u8,                // default 5
    pub open_duration_ms: u64,             // default 60_000
    pub idempotency_window_secs: u64,      // default 3_600 (1h)
    pub offline_snapshot_path: Option<std::path::PathBuf>, // optional offline-replay sink
}

impl Default for StcortexEmitterConfig { /* ... */ }

pub struct StcortexEmitter { /* private: config, breaker, outbox, m13_writer_handle */ }

impl StcortexEmitter {
    pub fn with_config(config: StcortexEmitterConfig) -> Result<Self, ReinforceError>;

    /// Outbox-first reinforcement. Returns `SubstrateUnavailable` on stcortex outage —
    /// NEVER falls back to POVM.
    pub async fn reinforce(
        &self,
        workflow_id: WorkflowId,
        outcome: RunOutcome,
    ) -> Result<ReinforceOutcome, ReinforceError>;

    pub async fn retry_sweep(&self) -> Result<u32, ReinforceError>;
}
```

## 3. Internal data structures

```rust
struct StcortexEmitterInner {
    config: StcortexEmitterConfig,
    breaker: Arc<Breaker>,             // peer = stcortex :3000
    outbox: Mutex<OutboxWriter>,
    next_id: AtomicU64,
    idempotency_seen: Mutex<TtlSet<uuid::Uuid>>,   // 1h TTL set; AP-WT-F TTL-sweep aware
    m13_writer: Arc<m13_stcortex_writer::Writer>,  // canonical stcortex write path
}

#[inline]
fn slug_encode(id: &str) -> String {
    // AP-Hab-11: hyphen-slug munge bug
    id.replace('-', "_")
}
```

The `TtlSet<uuid::Uuid>` uses realistic timestamps (`now_ms() + i`) per `feedback_ttl_sweep_test_timestamps.md` to avoid the test-timestamp 0..5 sweep trap. Sweep cadence is hourly; the structure is a bounded LRU map gating duplicate `request_id` within `idempotency_window_secs`.

## 4. Data flow

- **INPUT FROM:** `m32::DispatchOutcome` (Cluster G) — m32 calls `emitter.reinforce(workflow_id, outcome).await` after every dispatch event with a known outcome
- **OUTPUT TO:** `{data_dir}/m42/outbox.jsonl` (durable) and `127.0.0.1:3000` stcortex pathway-write via m13 (`stcortex_writer_narrowed`); optionally `{offline_snapshot_path}/m42-replay.jsonl` if configured and stcortex unreachable for replay-after-recovery
- **SUBSTRATE TOUCHED:** stcortex only (no POVM, no SYNTHEX, no atuin) — per ADR M0 hard-cut
- **WRITES:** outbox JSONL (always); stcortex pathway (`pre_id → post_id`, `weight += fitness_delta`) when breaker Closed/HalfOpen AND not idempotency-deduped

## 5. Algorithm sketch

```text
reinforce(workflow_id, outcome):
    # 1. Compute fitness_delta + clamp (defense in depth)
    fitness_delta = fitness_delta_for(outcome)  # clamped [-1.0, 1.0]

    # 2. Build pathway IDs with AP30 prefix + AP-Hab-11 slug-encode
    wf_slug = slug_encode(workflow_id.as_str())                                # AP-Hab-11
    pre_id  = format!("{}{}", WORKFLOW_TRACE_PREFIX, wf_slug)                  # AP30
    post_id = format!("{}outcome_{}", WORKFLOW_TRACE_PREFIX, outcome.as_str())

    # 3. m9 namespace guard (defense in depth — m13 also checks)
    if not pre_id.starts_with(WORKFLOW_TRACE_PREFIX):
        metric!(reinforce_namespace_violation_total += 1)
        return Err(NamespaceViolation { id: pre_id })

    # 4. Idempotency check
    request_id = uuid::Uuid::new_v4()
    payload = ReinforcePayload { session_id: workflow_id.into_session_id(), fitness_delta,
                                 retrieval_ids: vec![pre_id, post_id], request_id }

    # 5. Outbox-first (durable)
    envelope = OutboxEnvelope { id: next_id.fetch_add(1), created_at: now_s(),
                                payload: payload.clone(), posted: false }
    outbox.append_jsonl(envelope).await?
    outbox.sync_data().await?

    # 6. Circuit check
    if not breaker.allow().await:
        metric!(reinforce_circuit_open_total += 1)
        write_offline_snapshot(payload).ok()
        return Ok(ReinforceOutcome::SubstrateUnavailable)

    # 7. Idempotency dedup
    if idempotency_seen.contains_within(request_id, idempotency_window_secs):
        return Ok(ReinforceOutcome::Skipped)
    idempotency_seen.insert(request_id, now_ms() + 0)  # realistic ts to survive TTL sweep

    # 8. Route via m13 (stcortex-only; NO POVM branch)
    result = m13_writer.write_pathway(pre_id, post_id, fitness_delta).await

    match result:
        Ok(()) -> breaker.record_success().await; mark_envelope_posted(envelope.id)
                  metric!(reinforce_accepted_total += 1); Ok(ReinforceOutcome::Accepted)
        Err(M13Error::Unavailable) ->
                  breaker.record_failure().await
                  metric!(reinforce_substrate_unavailable_total += 1)
                  write_offline_snapshot(payload).ok()
                  # NEVER fall back to POVM (CLAUDE.md stcortex policy)
                  Ok(ReinforceOutcome::SubstrateUnavailable)
        Err(M13Error::Schema { status, body }) ->
                  breaker.record_failure().await
                  Ok(ReinforceOutcome::SubstrateError { status, body })
```

## 5.1. Outbox-policy (NA-GAP-06 closure: drain ordering, saturation, snapshot staleness)

The Algorithm sketch above details the per-call write path. NA-GAP-06 surfaced three under-specified outbox-policy axes that govern **what happens between fresh writes**: drain ordering on substrate recovery, saturation behaviour, and offline-snapshot staleness threshold. All three are policy-level (not per-write) and must be machine-checked by integration tests.

### 5.1.a. Drain ordering on substrate recovery

When the breaker transitions Closed (after HALF_OPEN probe success), m42 has a backlog: every payload written to `outbox.jsonl` while breaker was Open carries `posted: false`. Per ADR R3 + the offline-snapshot path (Algorithm sketch step 6), unposted envelopes have also been written to `{offline_snapshot_path}/m42-replay.jsonl` (if configured). On recovery:

| Phase | Action | Invariant |
|---|---|---|
| **Detection** | `retry_sweep` (background task; cadence `outbox_retry_interval_secs`, default 60s) discovers `posted: false` envelopes in `outbox.jsonl` and the breaker is Closed | sweep is single-task per binary (no race) |
| **Replay order** | Envelopes drained in `envelope.id` ascending order (creation order = causal order; the `id` field is monotonic from `AtomicU64::fetch_add`) | causal ordering preserved across recovery |
| **Per-envelope retry** | Each envelope re-runs the m13 write path; on success → mark `posted: true` (in-place line-rewrite via marker, NOT delete); on failure → break sweep, leave breaker management to the next allow-check | idempotency_seen entry (TtlSet) re-honoured to avoid duplicate reinforce on stcortex |
| **Idempotency** | If stcortex still has the prior `request_id` in its idempotency window (default 1h), the replay returns `Skipped` — counts as success for drain accounting | matches m24_povm_bridge.rs idempotency pattern lift |
| **Offline-snapshot reconciliation** | After full outbox drain, `m42-replay.jsonl` is compared against the now-`posted: true` envelopes; replay-only entries (no matching outbox row) are also drained, then archived to `{offline_snapshot_path}/m42-replay-archive-{ts}.jsonl` | offline snapshot does not silently grow; archival cadence is per-recovery |
| **Throttle** | Drain rate capped at `outbox_drain_max_per_sec` (default 20) to avoid post-recovery stcortex thundering-herd (per [`../substrates/stcortex.md`](../substrates/stcortex.md) backpressure signals) | substrate-load-aware; observed via `reinforce_drain_throttled_total` |

### 5.1.b. Outbox saturation limit

Outbox is JSONL-on-disk, durability-by-`sync_data`; under sustained stcortex outage with high dispatch rate, file size grows unbounded. Saturation behaviour:

| Threshold | Action | Metric |
|---|---|---|
| `outbox_saturation_warn_bytes` (default 64 MB) | Log warning; emit `reinforce_outbox_warn_total += 1`; surface via WireEvent::Refusal (`SubstrateAuthored { S-C, OutboxApproachingSaturation }`) | `reinforce_outbox_warn_total` |
| `outbox_saturation_refuse_bytes` (default 256 MB) | New `reinforce()` calls return `Ok(ReinforceOutcome::OutboxSaturated)` immediately (without appending) → m32 surfaces as `SubstrateAuthored { S-C, OutboxSaturated }`; the outcome is **logged as a dropped reinforce** in m12 reports | `reinforce_outbox_saturated_total` |
| `outbox_saturation_panic_bytes` (default 1 GB) | Panic with `outbox.jsonl exceeded 1 GB — operator intervention required` (operator must rotate / replay manually); engine intentionally fails-loud rather than fails-silent | none (panic) |

Rationale: **failing-loud at saturation is the AP-V7-13-aware path** — silent unbounded outbox growth is the worst-case substrate-drift accomplice (engine looks healthy while accumulating un-replayed reinforce events for weeks).

Outbox rotation: on every `retry_sweep` cycle, drained envelopes (now `posted: true`) older than `outbox_compact_age_days` (default 7) are compacted out via in-place truncate-and-rewrite; the compacted-out envelope summaries are archived to `{outbox_path}/m42-compact-archive-{ts}.jsonl` for audit.

### 5.1.c. Offline-snapshot staleness threshold

Per [workspace CLAUDE.md memory row 8](../../../CLAUDE.md) stcortex policy, on stcortex unreachable, reads should fall back to `data/snapshots/latest.json` and writes go to the offline-snapshot path. The read-fallback's staleness must be bounded:

| Threshold | Action |
|---|---|
| `snapshot_staleness_warn_secs` (default 300s / 5 min) | Engine logs warning on every read using the snapshot; metric `m13_snapshot_stale_read_total += 1`; surfaces via WireEvent::Refusal (`SubstrateAuthored { S-C, SnapshotApproachingStaleness }`) on first cross of threshold |
| `snapshot_staleness_refuse_secs` (default 3600s / 1 hr) | Reads from snapshot return `Err(StcortexWriterError::SnapshotTooStale { stale_secs })` — engine no longer trusts the offline snapshot |
| `snapshot_staleness_panic_secs` (default 86400s / 24 hr) | Engine refuses to proceed at all — startup probe fails if snapshot stale > 1 day at boot |

The staleness threshold is applied **per-read**, not per-snapshot-age: if a fresh snapshot lands at `T`, reads up to `T + warn` are quiet, between `warn` and `refuse` log + emit, beyond `refuse` fail-fast.

m42 itself **does NOT read snapshots** (m42 is write-only — see § 11 invariant "AP-WT-F3 substrate-input poisoning: m42 is write-only; reads zero state"); this policy governs sister modules (m13 stcortex-writer reads, m31 selection reads via m14). It is documented here for symmetry with the write-side outbox policy.

### 5.1.d. Substrate-confirmable receipt

Per [`../substrate-couplings/CC-5-decomposed.md`](../substrate-couplings/CC-5-decomposed.md) § 3, m42's drain operation may carry a substrate-confirmable receipt: stcortex writes `cc5_replay_observed_at` on a pathway when it detects an idempotency-cache-miss reinforce arriving from a replay path. This receipt allows the engine to confirm "drain reached the substrate end-to-end" rather than just "outbox marked posted" — the difference matters under network partitions where m13 saw HTTP 200 but the substrate-internal Hebbian update silently dropped. Substrate-side change request tracked in [`../../../ai_docs/decisions/`](../../../ai_docs/decisions/).

### 5.1.e. Metric inventory (additions for NA-GAP-06)

| Metric | Trigger | Owner |
|---|---|---|
| `reinforce_outbox_warn_total` | outbox crosses warn threshold | retry_sweep |
| `reinforce_outbox_saturated_total` | reinforce() returned OutboxSaturated | reinforce() |
| `reinforce_drain_throttled_total` | retry_sweep paced down due to drain rate cap | retry_sweep |
| `reinforce_drain_replayed_total` | envelope marked posted via drain | retry_sweep |
| `reinforce_drain_skipped_idempotent_total` | replay returned Skipped (substrate already saw request_id) | retry_sweep |
| `m13_snapshot_stale_read_total` | snapshot-fallback read crossed warn threshold | m13 (sister module; documented here for symmetry) |

These metrics close the **observability gap** NA-GAP-06 surfaced: today the engine has no signal between "outbox written" and "stcortex acknowledged"; with these, every state transition has an emit-point.

## 6. Boilerplate lifts

Per V7 cluster-H plan § m42 § Boilerplate-lift source (Category 08 Nexus-LCM-RPC gold standard + Category 02 stcortex consumer):

| Source | Lift | % |
|---|---|---:|
| `m24_povm_bridge.rs` outbox-append + raw_http_post pattern | atomic append + `sync_data` + raw TCP scaffold | 85% (POVM call site dropped; m13 write substituted) |
| `m22_synthex_async.rs::Breaker` state machine | Closed/Open/HalfOpen 5-fail / 60s | 95% (shared via `m40_42_common::breaker`) |
| `subscriber_main.rs` (stcortex consumer) | namespace + slug-encode conventions | 80% (adapted for write-side) |
| `m22_synthex_async.rs::spawn_blocking` wrapper | AP29 mitigation, 2s cap | 95% |
| Fitness-delta `const` + `fitness_delta_for` | — | 0% (novel ~10 LOC) |
| `dual_path.rs` cutover logic | — | **DELETED per ADR** (no longer needed; stcortex-only) |
| `povm_client.rs` | — | **DELETED per ADR** (POVM-decoupled) |

Net: ~120 LOC lifted / ~30 LOC novel (fitness-delta constants + slug-encode glue + idempotency TtlSet wiring).

## 7. ME v2 m1_foundation patterns referenced

- `error.rs` — `ReinforceError` is a `thiserror::Error` enum with structured named-field variants (`NamespaceViolation { id }`, `SubstrateError { status, body }`); callers can match programmatically.
- `resources.rs` — `//!` module docstring block adopted verbatim; the POVM-DECOUPLED status is highlighted in the docstring.
- `shared_types.rs` — newtype discipline (`WorkflowId`, `SessionId`); fitness-delta constants are `pub const`, not magic numbers.
- `logging.rs` — tracing-subscriber structured emit on every reinforce (workflow_id, outcome, fitness_delta, pathway_count), every circuit open skip, every namespace violation, every offline-snapshot write.

## 8. Test strategy

- **Test kind**: async (30 unit) + property (5) + 0 fuzz (per V7 plan) + integration (15) + contract (5) + regression (4); total **70** (per V7 cluster-H plan § m42 § Test budget; matches MODULE_MATRIX row m42 70 tests — m42's allocation is the highest in Cluster H by 5 tests due to dual-path-cutover-safety-equivalent stcortex-edge-case rebudget per ADR R7)
- **Mutation budget**: ≥**80%** kill (highest in Cluster H per V7 G6 § m42 — substrate-coherence is critical; a surviving mutation that drops the stcortex write silently breaks CC-5)
- **Properties tested** (F-Property 5):
  - fitness_delta bounded: `fitness_delta_for(any outcome) ∈ [FITNESS_BLOCKED, FITNESS_PASS_VERIFIED]` (within the `[-1.0, 1.0]` clamp)
  - AP30 prefix invariant: every `retrieval_id` produced starts with `WORKFLOW_TRACE_PREFIX`
  - AP-Hab-11 slug invariant: encoded `pre_id`/`post_id` contain no `-` characters
  - Idempotency: re-reinforce with same `request_id` within window returns `Skipped`
  - Write-then-read identity: serde round-trip of `ReinforcePayload` is identity-preserving
- **Fuzz target**: per V7 plan, 0 fuzz for m42 (the fuzz surface lives in m13 stcortex writer and in m40/m41 JSONL parsers; m42 has no novel parser)

Key invariants (sample of 70):

1. fitness_delta constants match spec table (PassVerified +0.25 / Pass +0.15 / Blocked −0.05 / Fail −0.10)
2. fitness_delta clamping holds for outcomes outside table (defense in depth)
3. NO POVM branch in any code path (regression slot — `grep -r povm src/m42_stcortex_emit/` returns empty)
4. NO `povm_overlap_active` flag exists (regression slot)
5. AP30 prefix on every emitted pathway ID
6. AP-Hab-11 hyphen → underscore on `workflow_trace_wf-abc-123` → `workflow_trace_wf_abc_123`
7. Namespace violation pre-empts outbox write (fail-fast)
8. Outbox-write-first: `sync_data` completes before breaker.allow check
9. Circuit Open → `SubstrateUnavailable` + offline snapshot write
10. Circuit Closed + m13 success → `Accepted` + envelope marked posted
11. Circuit Closed + m13 unavailable → `SubstrateUnavailable` + breaker `record_failure` + **NO POVM fallback**
12. Circuit Closed + m13 schema error → `SubstrateError { status, body }` + breaker `record_failure`
13. Idempotency: duplicate `request_id` within 1h → `Skipped`
14. TTL sweep does NOT remove fresh `request_id` (realistic timestamps; per `feedback_ttl_sweep_test_timestamps.md`)
15. BUG-033: `addr` is `"127.0.0.1:3000"` — no `http://` prefix
16. m13 is the only write path (no direct HTTP from m42)
17. Concurrent reinforce calls do NOT corrupt outbox JSONL (mutex-protected writer)
18. Watcher Class-B test stub: every m13 write attempt emits structured tracing
19. Watcher Class-I test stub: rolling 7d pathway.weight delta probe surface exists
20. Contract: stcortex schema-pin version recorded; bump triggers AP-Drift-05 audit
21. Regression: m42 must NEVER produce a pathway under any namespace other than `workflow_trace_*`
22. Regression: integration test stcortex `:3000` required; `#[ignore = "requires stcortex"]` when absent

(remaining 48 enumerated in vault cluster-H spec § Test coverage requirements + ADR § R7 reallocation)

## 9. Antipatterns to avoid

- **AP-V7-13 — Health-200 ≠ behaviour-verified** (the ORIGIN STORY of this module's pivot). Per [ANTIPATTERNS.md](../../../ANTIPATTERNS.md) row AP-V7-13: refresh-date stamps and `/health` 200s MUST be paired with re-executed probes. The live POVM `:8125` probe that triggered this ADR is the canonical demonstration: HTTP 200 on `/health` while serving the pre-CR-2 binary. m42 carries the lesson forward: stcortex's HTTP 200 on a pathway-write is liveness; pathway.weight delta on the substrate side is the behaviour-verification. Class-I Watcher monitor is the verification surface.
- **AP30** (namespace prefix violation) — m9 + intrinsic check; `NamespaceViolation` returned before any IO; regression slot for any literal-string drift outside `WORKFLOW_TRACE_PREFIX` constant
- **AP-Hab-11** (hyphen-slug munge bug, S1001757 origin) — slug-encode at the boundary; tested invariant
- **AP29** (sync HTTP in `tokio::spawn` starves) — mitigated by m13's `spawn_blocking` + 2s cap (inherited)
- **AP-Drift-05** (schema bump silently breaks) — stcortex schema-pin version constant; bump triggers explicit migration step + audit
- **AP-Drift-06** (bridge contract drift) — F-Contract tests against stcortex pathway-write schema; `bridge-contract` skill per Wave-end
- **F7** (graceful-degrade pretend-fix) — ELIMINATED by ADR (POVM `:8125` is no longer a workflow-trace dependency); the antipattern remains relevant habitat-wide but is structurally out-of-scope for workflow-trace
- **Silent POVM fallback** — explicitly forbidden by CLAUDE.md stcortex policy and ADR R3; m42 returns `SubstrateUnavailable` and writes offline-snapshot, never POVM
- **AP-WT-F3** (substrate-input poisoning) — m42 is write-only; reads zero state
- **Hardcoded fitness magic numbers** — every value goes through `fitness_delta_for(outcome)`; constants are `pub const`

## 10. Useful patterns applied

- Outbox-first JSONL durability (PATTERNS.md § Substrate-write patterns)
- Circuit-breaker shared library (PATTERNS.md § Architectural patterns; `m40_42_common::breaker`; peer count reduced 3 → 2 per ADR)
- Idempotency dedup via TtlSet with realistic timestamps (PATTERNS.md § Module-level patterns; `feedback_ttl_sweep_test_timestamps.md`)
- Offline JSONL snapshot fallback (CLAUDE.md stcortex policy; per ADR R3)
- thiserror error enums (GOLD_STANDARDS rule 9)
- Newtype discipline (`WorkflowId`, `SessionId`; GOLD_STANDARDS rule 8)
- `pub const` discipline for fitness constants (GOLD_STANDARDS rule 8 — no magic numbers)

## 11. Cross-cluster contracts

- **CC-5 (OWNS emit-half; primary substrate-feedback channel)**: m42 is the only Hebbian-grain reinforcement path. The contract: every dispatched workflow with a known `RunOutcome` produces a stcortex pathway-write `(workflow_trace_<wf>, workflow_trace_outcome_<oc>, fitness_delta)`. Pathway weights update over the days/weeks timescale; m31's next selection cycle reads the updated weights and shifts the selection distribution. Per ADR R6, the CC-5 closure-test moves from "POVM learning_health moved" to **"stcortex pathway.weight delta observable on `workflow_trace_*` IDs"**, monitored by Watcher Class-I on a rolling 7d window (trigger if delta == 0 for 4+ weeks).
- **CC-2 (consumes trust layer)**: m8 build-prereq (legacy POVM-calibrated gate — see open question 4); m9 namespace-guard validates `workflow_trace_*` prefix at every write; m10 Ember-CI-gate applies at source review.

## 12. Open questions for G5 interview / Zen G7 audit

1. **`StcortexEmitter` vs direct m13 calls**: this spec wraps m13 in a `StcortexEmitter` struct that owns the breaker + outbox + idempotency set. Alternative: m42 is a thin function `m42::reinforce(workflow_id, outcome, m13_writer)` and the state lives in `wf-crystallise` daemon. Tradeoff = encapsulation vs sharing breaker state with potential future emitters.
2. **Offline JSONL snapshot replay-trigger**: the offline snapshot accumulates while stcortex is down. Who/what drains it on recovery? `retry_sweep` is the candidate, but does it need a separate `replay-from-snapshot` CLI command for ops?
3. **Class-I Watcher window**: 7-day rolling is the ADR-specified value. For a single-binary deploy with low workflow throughput at M0, is 7 days too short? Tied to the "≥20 invocations/week threshold" Class-E trigger.
4. **`cfg(povm_calibrated)` rename**: now that m42 is POVM-decoupled, the `cfg(povm_calibrated)` gate name is misleading. Rename to `cfg(stcortex_calibrated)` or retire entirely? (Touches m8 which lives in Cluster D.)
5. **Per-binary outbox or shared**: `wf-crystallise` and `wf-dispatch` are separate binaries; the ADR R4 specifies "two consumers, one per binary". Does each binary own a separate `m42` outbox file, or do they share? (Atomicity concerns if shared.)
6. **stcortex offline-snapshot file naming under burst**: under sustained stcortex outage, the offline JSONL could grow unbounded. Rotation policy (size cap + rolling rename)? Compaction (drop entries older than N days)?

## 13. Implementation order (post-G9)

1. `errors.rs` — `ReinforceError` enum (`thiserror`); compile-only, no tests
2. `fitness.rs` — fitness-delta constants + `fitness_delta_for` + clamp; 8 unit tests (constants match table, clamp invariant, every variant covered)
3. `slug.rs` — `slug_encode` (hyphen → underscore); 4 unit tests (AP-Hab-11 invariant)
4. `idempotency.rs` — `TtlSet<uuid::Uuid>` with realistic-timestamp pattern; 6 unit tests (insert / contains / TTL sweep does not reap fresh)
5. `outbox.rs` — `OutboxEnvelope` writer (mirrors m40 pattern); 6 unit tests
6. `mod.rs` — `StcortexEmitter::with_config` + `reinforce` + namespace guard + offline snapshot wiring; 6 unit tests (Outcome variants, namespace violation, no-POVM-branch regression)
7. Property tests (5) — proptest on fitness bounds / AP30 / AP-Hab-11 / idempotency / serde roundtrip
8. Integration tests (15) — `tests/m42_integration.rs` with mock m13 + live stcortex `:3000` gated `#[ignore = "requires stcortex"]`; offline-snapshot replay roundtrip
9. Contract tests (5) — insta snapshots for `ReinforcePayload` vs stcortex pathway-write schema (pinned version)
10. Regression slots (4) — reserved for first bugs (POVM branch regression, fitness drift, namespace literal drift, TTL sweep timestamp 0..5 trap)
11. Mutation pass — `cargo mutants` on `fitness.rs` + `mod.rs` + outbox/breaker call sites; ≥80% kill required

---

> **Decision authority for POVM decoupling:** Luke @ node 0.A via 12-round AskUserQuestion grilling (2026-05-17); 48/48 Command recommendations accepted. Canonical ADR: [2026-05-17-m42-stcortex-only-pivot.md](../../../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md).
>
> Back to: [vault cluster-H spec](../../../the-workflow-engine-vault/module%20specs/cluster-H-substrate-feedback.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · [m42 ADR](../../../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md) · sister modules: [m40](m40_nexusevent_emit.md) · [m41](m41_lcm_rpc.md) · [m42](m42_stcortex_emit.md)
