> Back to: [[SYNTHEX-V2 Integration Master Schematic]] · [[HOME]] · [[MASTER_INDEX]] · `~/claude-code-workspace/the-workflow-engine/CLAUDE.local.md`
> Cross-vault: [[synthex-v2/MASTER_INDEX]] · [[synthex-v2/The Watcher]] · [[synthex-v2/DAEMON_INTEGRATION_PLAN]]
> Closes: **OP-6** (CHANGELOG `[v0.2.0]`) · **NA-4** (Plan v2 §6 in-place amendment 2026-05-24)

# Wiring 01 — m16 Heartbeat Consumer (NA-4 Closure)

> **🔴 HIGHEST-LEVERAGE wiring point.** V3 m16 KEYSTONE is **operationally inert at v0.2.0** — emits heartbeats with no consumer. This schematic specifies the synthex-v2-side receiver to close the self-canary loop.

## The current state

```
workflow-trace m16_substrate_drift_canary
  ↓ emits Heartbeat { skew_envelope, alert_budget } per cycle (1 Hz default)
  ↓ [WIRE GAP — no consumer]
  ✗ (envelopes vanish into the void)
```

**Audit consequence:** Plan v2 §6 NA-4 row carries the honesty amendment — *"V3 m16 KEYSTONE shipped (canary types + emitter + AlertBudget) BUT Watcher liveness consumer DID NOT ship — heartbeat envelope exists per cycle but no `synthex-v2/m8_watcher/*` module consumes it; self-canary loop is NOT closed at v0.2.0; risk mitigated **structurally** (heartbeat exists, shaped for consumption) but **NOT loop-closed**."*

## Landing options (5 evaluated)

| Option | Landing point | Pros | Cons | Verdict |
|---|---|---|---|---|
| **A** | new m8_watcher sub-receiver | m46 Observer reads directly | **AP27 violates** — Watcher cannot modify self | ❌ blocked |
| **B** | **m10_http_poller new endpoint** | Fits L2 ingest tier; typed `BridgeResponse<Heartbeat>` | none material | ✅ **RECOMMENDED** |
| C | WebSocket `Heartbeat` variant (S116) | Reuses existing infra | Only ORAC consumes WS today; coupling | deferred to T2 |
| D | new m56 epistemic receiver via S117 EpistemicBroadcast | Semantically "heartbeat = belief update" | S117 not yet landed | deferred to T2 |
| E | daemon-level `AppState.on_heartbeat()` callback | Centralizes alert routing | No typed response; harder to test | rejected |

## Option B — full wiring schematic

### Data path (4 hops to PBFT-observable signal)

```
workflow-trace m16  ─── HTTP POST :8092/v3/heartbeat ───┐
                                                         ↓
                              synthex-v2 m10_http_poller (L2 ingest)
                                       ↓ IngestSignal::Heartbeat
                              m13_ingest_router (route to L4)
                                       ↓ SignalBus::L4Alert { skew, budget, severity }
                              m19_pid_controller (CONSUMES)
                                       ↓ optional PID setpoint adjust
                              m46 Observer (1 Hz reads signal_bus_recent)
                                       ↓ recurring drift → flag for m47 Critic
                              m47 → m48 → m51 Ember gate → m49 PBFT
```

### Wire-level contract (REST POST /v3/heartbeat)

**Request body (JSON):**
```json
{
  "sequence_number": 42,
  "timestamp_ms": 1735689600000,
  "skew_envelope": {
    "observed_skew_ms": 12,
    "drift_severity": 1,
    "acceptable_bounds_ms": [-50, 50]
  },
  "alert_budget": {
    "alerts_used": 3,
    "alerts_remaining": 7,
    "next_reset_ms": 1735689660000
  },
  "heartbeat_source": "workflow-trace::m16_substrate_drift_canary"
}
```

**Response body (JSON), 200:**
```json
{
  "ack": true,
  "sequence_number_acked": 42,
  "synthex_v2_observed_at_ms": 1735689600015,
  "synthex_v2_observed_clock_skew_ms": 15
}
```

**Error responses:**
- `400` — schema rejected (typed `BridgeError::HeartbeatSchema`)
- `429` — alert budget exhausted on synthex-v2 side
- `503` — synthex-v2 R13 cold-start in progress, retry after the `Retry-After` header
- `5xx` — internal — workflow-trace MUST NOT silent-swallow; emit `RefusalToken::Unavailable(SubstrateUnreachable { substrate_id: SynthexV2, transport_reason: "..." })` per V1 ADR D-S1004XXX-04

### Files synthex-v2 must add/modify (~400 LOC, 45–60 min)

| File | Action | LOC |
|---|---|---|
| `src/m1_foundation/mod.rs` | Add `WorkflowTraceHeartbeat` newtype + `SkewEnvelope` + `AlertBudget` (mirrors WFE's `m16_substrate_drift_canary` types byte-for-byte at wire level) | +80 |
| `src/m1_foundation/error_taxonomy.rs` | Add `BridgeError::WorkflowTraceHeartbeat(HeartbeatError)` enum entries | +20 |
| `src/m2_ingest/m10_http_poller.rs` | Add `poll_heartbeat()` (if pulling) OR HTTP server route for inbound POST (recommended) | +40 |
| `src/m2_ingest/m13_ingest_router.rs` | Add `IngestSignal::Heartbeat(WorkflowTraceHeartbeat)` variant + router arm | +60 |
| `src/daemon/http.rs` | Register `POST /v3/heartbeat → handle_heartbeat()` | +30 |
| `tests/m10_heartbeat_integration.rs` | Integration test: POST → signal_bus → observe | +120 |
| `ai_docs/modules/M10_HTTP_POLLER.md` + vault mirror | Document new endpoint | +50 |

### Files workflow-trace already has (no change needed)

- `src/m16_substrate_drift_canary/mod.rs` — `Heartbeat` struct + emitter exist (v0.2.0, 13 tests)
- `src/m16_substrate_drift_canary/tests.rs` — emitter coverage exists
- ADR `ai_docs/decisions/2026-05-23-refusal-token-authorship-typing.md` — RefusalToken envelope for transport failures

### Optional v0.2.2+ workflow-trace addition (post synthex-v2 endpoint land)

```rust
// src/m16_substrate_drift_canary/transport.rs   (NEW, ~80 LOC)
pub struct HeartbeatTransport {
    client: reqwest::Client,
    endpoint: Url,                                          // :8092/v3/heartbeat
    refusal_emitter: Arc<RefusalTokenEmitter>,
}

impl HeartbeatTransport {
    pub async fn send(&self, hb: Heartbeat) -> Result<HeartbeatAck, RefusalToken> {
        match self.client.post(self.endpoint.clone()).json(&hb).send().await {
            Ok(r) if r.status().is_success() => Ok(r.json().await?),
            Ok(r) if r.status() == 503 => Err(RefusalToken::Unavailable(
                UnavailableReason::SubstrateUnreachable {
                    substrate_id: SubstrateId::SynthexV2,
                    transport_reason: format!("substrate_unreachable:r13_cold_start_503"),
                }
            )),
            Ok(r) => Err(RefusalToken::Unavailable(
                UnavailableReason::SubstrateAuthored {
                    substrate_id: SubstrateId::SynthexV2,
                    substrate_reason: format!("substrate_authored:http_{}", r.status()),
                }
            )),
            Err(e) => Err(RefusalToken::Unavailable(
                UnavailableReason::SubstrateUnreachable {
                    substrate_id: SubstrateId::SynthexV2,
                    transport_reason: format!("substrate_unreachable:transport_{e}"),
                }
            )),
        }
    }
}
```

This honors the V1 NA-5 audit-distinguishability contract (Plan v2 §15 DX-V5.b 3-variant sub-tag) for every transport-layer outcome — `SubstrateUnreachable` for transport failures, `SubstrateAuthored` for substrate-emitted refusals, `EngineImagined` never used here because the substrate is reachable per the request itself.

## Constraints + anti-patterns

| Constraint | Why | Mitigation |
|---|---|---|
| **AP27 self-modification** | Watcher (m46–m51) cannot author m8_watcher changes | This wiring lands at L2 (m10/m13) — NOT m8_watcher. Luke @ node 0.A authors the receiver code |
| **R13 cold-start** (elapsed 2026-05-19) | m49 won't submit PBFT until observation arm ≥100 + calendar elapsed | Already eligible (663,619+ obs at S1004590); heartbeats accumulate as m46 observations |
| **AP29 Ember bypass** | Never skip 7-trait gate | m49 always calls m51 before PBFT — workflow-trace heartbeat is observation input, not direct PID write |
| **AP30 Watcher budget** | $50/day Opus cap (m47/m48/m50 share) | Heartbeat alerts must be rate-limited (`AlertBudget` does this engine-side) so m47 hypothesis doesn't fire >N times/day |
| **AP42 (new this wiring)** | "Assume m16 consumer exists" — DON'T | Until synthex-v2-side endpoint ships, m16 heartbeats MUST land in `RefusalToken::Unavailable(SubstrateUnreachable)` — never silent-swallow |
| **AP43 (new this wiring)** | Silent swallow on heartbeat 5xx | Always wrap in `Result<HeartbeatAck, RefusalToken>` — never `_ = client.post(...)` |

## Verification path (post-landing)

1. **synthex-v2 side:** `cargo test --test m10_heartbeat_integration` → POST → signal_bus → observe (assert L4Alert received with correct skew)
2. **workflow-trace side:** new `m16_substrate_drift_canary::transport::tests` mocks `:8092/v3/heartbeat` and asserts Ok/SubstrateUnreachable/SubstrateAuthored routing per status code
3. **End-to-end:** start synthex-v2 daemon; run workflow-trace `m16` for 60 seconds; verify `~/.local/bin/stcortex query` shows 60 `workflow_trace_m16_heartbeat ↔ synthex_v2_pid_adjustment` pathway co-activations
4. **Watcher observation:** `watcher-repl` → query m46 observations → confirm `L4Alert` events from workflow-trace appearing in `signal_bus_recent`

## What this unlocks (v0.2.2+ horizon)

- **NA-4 self-canary loop CLOSED** — Plan v2 §6 NA-4 row updates from "mitigated structurally, NOT loop-closed" to "loop-closed via synthex-v2 m46 Observer consuming m16 Heartbeat L4Alert events"
- **V5 SubstrateParticipationStatus transition:** `SubstrateId::SynthexV2` moves from `NotShipped` → `Shipping` → `Live` once the endpoint accepts production traffic for 48h DX-Soak per OP-3
- **First substrate-authored refusal channel** — `RefusalToken::SubstrateAuthored { substrate_id: SynthexV2, ... }` becomes emittable, audit-distinguishable from EngineImagined per NA-5
- **m49 Proposer can now propose PID adjustments based on workflow-trace clock skew** — turning the engine's substrate-drift signal into actionable Watcher hypotheses gated by Ember 7-trait check

## Honest residual

- This wiring requires synthex-v2-side code Luke must author (or coordinate with the synthex-v2 maintainer). Workflow-trace **cannot ship the receiver** — only the transport client + RefusalToken routing.
- If the synthex-v2 endpoint never lands, workflow-trace's m16 emits indefinitely into `RefusalToken::Unavailable(SubstrateUnreachable)`. The V1 contract preserves audit-distinguishability — the failure is honest, not silent.
