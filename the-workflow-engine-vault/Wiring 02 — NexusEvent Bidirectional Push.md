> Back to: [[SYNTHEX-V2 Integration Master Schematic]] · [[HOME]] · [[MASTER_INDEX]] · `~/claude-code-workspace/the-workflow-engine/CLAUDE.local.md`
> Cross-vault: [[synthex-v2/MASTER_INDEX]] · [[synthex-v2/ORAC_SYNTHEX_WS_Bridge_S116]] · [[synthex-v2/BRIDGE_TOPOLOGY_CONTINUITY]]
> Module owner: m40_nexus_emit · m41_lcm_rpc · m42_stcortex_emit (Cluster H Substrate Feedback)

# Wiring 02 — NexusEvent Bidirectional Push

> Engine ALREADY emits, synthex-v2 ALREADY consumes — this wire is **largely already in place**; what's missing is the workflow-trace-side outbound client + namespace conventions for the `(source, kind)` axis. ~120 LOC.

## Current state

**workflow-trace side:** m40_nexus_emit (Cluster H) emits typed `NexusEvent` envelopes per cycle — currently into an in-memory channel with no transport adapter.

**synthex-v2 side (LIVE):** `:8092/v3/nexus/push` POST AND WebSocket `NexusEvent` variant both accepted. Routing chain:

```
m13_ingest_router → m29_policy_router → 4 subscribers:
  ├─ m31_k_adj_emitter         (PID adjustment input)
  ├─ m33_consolidation_trigger (memory consolidation signal)
  ├─ m35g_orac_bridge          (forwarded to ORAC if classified as ORAC-relevant)
  └─ m45_hmx_injector          (HMX 8-cluster pipeline seed)
```

## Wire shapes (both endpoints LIVE)

### REST POST /v3/nexus/push

**Request:**
```json
{
  "events": [
    {
      "type": "drift_canary_alert",
      "ts": 1735689600,
      "source": "workflow_trace::m40_nexus_emit",
      "data": {
        "drift_severity": 2,
        "observed_skew_ms": 45,
        "alert_budget_remaining": 5
      }
    },
    {
      "type": "command_exec_result",
      "ts": 1735689601,
      "source": "workflow_trace::m41_lcm_rpc",
      "data": { "exit_code": 0, "stdout_bytes": 1024, "lcm_cmd": "deploy --tier 3" }
    },
    {
      "type": "pathway_learned",
      "ts": 1735689602,
      "source": "workflow_trace::m42_stcortex_emit",
      "data": { "pre_id": "workflow_trace_m20_prefixspan_pattern_3", "post_id": "synthex_v2_classifier_class_B", "weight": 0.75 }
    }
  ]
}
```

**Response (200):**
```json
{
  "accepted": true,
  "event_ids": ["uuid-1", "uuid-2", "uuid-3"],
  "queued": 3,
  "synthex_v2_observed_at_ms": 1735689602015
}
```

### WebSocket `NexusEvent` variant (post-S116 ws_handler — LIVE)

```json
{
  "id": 42,
  "ts_ms": 1735689600000,
  "msg": {
    "type": "NexusEvent",
    "payload": {
      "events": [ /* same shape as REST above */ ]
    }
  }
}
```

**Async Ack back:** `{ id: 42, ts_ms: ..., msg: { type: "Ack", payload: { ack_id: 42 } } }`

## NexusEvent `(source, kind)` schema convention

The `data` field is opaque JSON — schema is owned by the (source, kind) pair. workflow-trace owns the following kinds:

| `source` | `kind` | Owner (WFE module) | `data` schema |
|---|---|---|---|
| `workflow_trace::m40_nexus_emit` | `drift_canary_alert` | m40 | `{ drift_severity: u8, observed_skew_ms: i64, alert_budget_remaining: u16 }` |
| `workflow_trace::m40_nexus_emit` | `verdict_emitted` | m40 (downstream of m33 verifier) | `{ verdict_kind, proposal_id, accepted: bool, escape_surface }` |
| `workflow_trace::m40_nexus_emit` | `bank_promotion` | m40 (downstream of m30 bank) | `{ variant_id, lift_ci_lower, generations_held }` |
| `workflow_trace::m41_lcm_rpc` | `command_exec_result` | m41 | `{ exit_code: i32, stdout_bytes: u64, lcm_cmd: String, duration_ms: u64 }` |
| `workflow_trace::m41_lcm_rpc` | `loop_create_ack` | m41 | `{ loop_id: String, scheduled_at_ms: u64 }` |
| `workflow_trace::m42_stcortex_emit` | `pathway_learned` | m42 | `{ pre_id, post_id, weight, parent_namespace }` |
| `workflow_trace::m42_stcortex_emit` | `memory_written` | m42 | `{ memory_id: u64, namespace, modality }` |

**Discipline:** every event MUST carry `source` (allows synthex-v2 m29_policy_router to dispatch to the right subscriber). The `kind` field is workflow-trace-owned namespace — synthex-v2 treats unknown kinds as opaque pass-through (forwarded to m45 HMX for episodic capture).

## Transport client (workflow-trace addition, ~120 LOC)

```rust
// src/m40_nexus_emit/transport.rs   (NEW)

use crate::refusal_token::{RefusalToken, SubstrateId, UnavailableReason};
use crate::substrate_trust::{SubstrateTrust, SubstrateParticipationStatus};

pub struct NexusTransport {
    client: reqwest::Client,
    endpoint: Url,                                       // :8092/v3/nexus/push
    substrate_trust: Arc<SubstrateTrust>,                // V5 audit-distinguishability
}

impl NexusTransport {
    pub async fn push_batch(&self, events: Vec<NexusEvent>) -> Result<NexusAck, RefusalToken> {
        // Gate on V5 substrate participation status first.
        if self.substrate_trust.is_substrate_imagined_for(SubstrateId::SynthexV2) {
            return Err(self.substrate_trust.refusal_for_unavailable(
                SubstrateId::SynthexV2,
                "synthex_v2_engine_imagined_not_shipped"
            ));
        }
        match self.client.post(self.endpoint.clone())
            .json(&NexusBatch { events })
            .send().await {
            Ok(r) if r.status().is_success() => Ok(r.json().await?),
            Ok(r) => Err(RefusalToken::Unavailable(
                UnavailableReason::SubstrateAuthored {
                    substrate_id: SubstrateId::SynthexV2,
                    substrate_reason: format!("substrate_authored:nexus_push_http_{}", r.status()),
                }
            )),
            Err(e) => Err(RefusalToken::Unavailable(
                UnavailableReason::SubstrateUnreachable {
                    substrate_id: SubstrateId::SynthexV2,
                    transport_reason: format!("substrate_unreachable:nexus_push_transport_{e}"),
                }
            )),
        }
    }
}
```

## Bidirectionality (S117 future — deferred to T2+)

Once S117 phase-2.5 lands the new WS variants on synthex-v2, **synthex-v2 → workflow-trace** NexusEvent flow becomes possible via WebSocket. Today the WS channel is ORAC-exclusive (`/ws/orac`); a `/ws/workflow-trace` mount-point would be the natural pair.

Deferred: opening this lane requires (a) S117 variant land, (b) workflow-trace WebSocket client (would be a new module — call it `m43_synthex_v2_ws` per Cluster H expansion), (c) ORAC-equivalent `Hello` handshake.

**No v0.2.x work needed here** — outbound m40 push is enough to close the substrate-feedback half of CC-5 (Substrate Learning Loop).

## Constraints + anti-patterns

| Constraint | Why | Mitigation |
|---|---|---|
| **AP39 (new)** "NexusEvent without source tag" | synthex-v2 m29_policy_router needs source to route | `NexusEvent::new()` constructor enforces `source: NonEmptyString` |
| **NexusEvent `kind` namespace** | unknown kinds → m45 HMX (opaque capture); known kinds → typed subscriber | Document every WFE kind in this schema table; never emit a kind not in the table without first adding it |
| **Batch size cap** | synthex-v2 has no hard cap published; recommend ≤32 events per batch to stay under HTTP body limits | `NexusBatch::push_capped(events: Vec<NexusEvent>, cap: usize = 32)` |
| **AP31 POVM endpoint singular** | unrelated but same shape — synthex-v2 uses `/v3/nexus/push` (singular path), POVM uses `/pathways` (plural). Don't confuse | Distinct constants in `m40_nexus_emit::endpoints` |
| **`#![forbid(unsafe_code)]`** | workflow-trace lib invariant | reqwest + serde_json are safe-Rust dependencies; no FFI bridges in this transport |

## Verification

```rust
#[tokio::test]
async fn push_batch_round_trip_against_running_synthex_v2() {
    // requires :8092 alive (skip if not)
    let transport = NexusTransport::new(":8092/v3/nexus/push");
    let evs = vec![NexusEvent::drift_canary_alert(2, 45, 5)];
    let ack = transport.push_batch(evs).await.expect("synthex_v2 alive");
    assert!(ack.accepted);
    assert_eq!(ack.queued, 1);
}

#[tokio::test]
async fn push_batch_returns_refusal_when_substrate_not_shipped() {
    let trust = Arc::new(SubstrateTrust::new());  // default = NotShipped for all
    let transport = NexusTransport::with_trust(trust);
    let evs = vec![NexusEvent::drift_canary_alert(2, 45, 5)];
    let result = transport.push_batch(evs).await;
    assert!(matches!(result, Err(RefusalToken::Unavailable(UnavailableReason::EngineImagined { .. }))));
}
```

## What this unlocks

- **CC-5 Substrate Learning Loop closes** on the engine side — m20–m23 PrefixSpan + variant proposer outcomes flow to m31 k_adj + m33 consolidation + m35g ORAC
- **m30 bank promotion events** become observable in synthex-v2's HMX episodic capture (m45) → searchable via vault recall
- **m33 verifier verdicts** become operationally consumable — `Approve` / `Reject` outcomes flow to m31 PID adjustment (creates a tight engine-substrate feedback loop)
