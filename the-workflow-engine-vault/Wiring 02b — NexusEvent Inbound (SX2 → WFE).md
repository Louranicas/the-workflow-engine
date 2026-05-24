> Back to: [[Wiring Plan v2 — Source-Verified Integration S1004590]] · [[Wiring 02 — NexusEvent Bidirectional Push]] · [[SYNTHEX-V2 Integration Master Schematic]] · [[HOME]] · [[MASTER_INDEX]] · `~/claude-code-workspace/the-workflow-engine/CLAUDE.local.md`
> Cross-vault: [[synthex-v2/MASTER_INDEX]] · [[synthex-v2/The Watcher]]
> Status: **SPECIFIED, NOT IMPLEMENTED.** Authored per Plan v2 § S4 (NA-2' resolution). No source code; vault-only specification awaiting Luke explicit phrase "start coding m43" before scaffolding.

# Wiring 02b — NexusEvent Inbound (SX2 → WFE)

> Sibling to [[Wiring 02 — NexusEvent Bidirectional Push]] (now correctly framed as **NexusEvent Outbound**). This schematic specifies the **reverse** channel: SX2 → WFE inbound consumption, closing the NA-2' "bidirectional" honesty gap from the gap analysis.

## Why this exists (NA-2' honesty)

Wiring 02 was titled "Bidirectional Push" but the body deferred SX2→WFE direction to T2+. From the substrate frame (NA pass): "bidirectional" implies reciprocal channel; the substrate cannot push events to WFE today because **no WFE-side consumer module exists**. This schematic specifies that module: **`m43_synthex_v2_inbound`**.

## What SX2 wants to push to WFE

Per Zen 2026-05-24T102556Z (cross-corridor analysis) + Plan v2 ZA-2 + § S4:

| Event source (SX2-side) | Event kind | WFE-side consumption | NA-5 sub-tag mapping |
|---|---|---|---|
| m46 Observer (post-WFE-heartbeat-landing) | `wfe_drift_observed` | Feed m11 fitness decay weight adjustment | substrate-authored observation; not a refusal |
| m46 Observer (WFE silence persisting) | `wfe_silence_observed` | Feed m16 internal AlertBudget reset hint | substrate-authored alert; not a refusal |
| m47 Critic | `wfe_drift_hypothesis` | Optional engine-side log; advisory only | substrate-authored advisory |
| m51 Ember Protector | `wfe_proposal_blocked` | Engine-side log of WFE-affecting proposals that failed 7-trait gate | substrate-authored audit event |
| m10_http_poller (when WFE absent) | `wfe_unreachable_persisting` | Feed bilateral V5 `WorkflowTraceParticipationStatus` → SX2-side update | substrate-authored liveness signal |

## Module shape (NEW, awaiting authorisation)

### `m43_synthex_v2_inbound` (proposed)

```
src/m43_synthex_v2_inbound/
├── mod.rs                      ~80 LOC  -- public API + module wiring
├── server.rs                   ~120 LOC -- HTTP server on workflow-trace side (:8094)
├── handler.rs                  ~60 LOC  -- per-event-kind dispatch
├── events.rs                   ~50 LOC  -- inbound event type catalogue (5 kinds above)
└── tests.rs                    ~150 LOC -- mock SX2 client sending each event kind
```

**Estimated total: ~460 LOC + tests** (revised from gap analysis's ~300 estimate after schema fold-in).

### Public API surface

```rust
// src/m43_synthex_v2_inbound/mod.rs
pub use events::{InboundEvent, InboundEventKind};
pub use handler::InboundHandler;
pub use server::{InboundServer, InboundServerConfig};

// src/m43_synthex_v2_inbound/events.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundEvent {
    pub source: InboundSource,           // m46_observer | m47_critic | m51_ember | m10_http_poller
    pub kind: InboundEventKind,
    pub emitted_at_ms: u64,              // SX2 wall-clock
    pub boot_id: String,                 // SX2 boot identifier (replay safety)
    pub data: serde_json::Value,         // kind-specific payload (opaque-typed-at-handler)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InboundSource {
    M46Observer,
    M47Critic,
    M51EmberProtector,
    M10HttpPoller,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InboundEventKind {
    WfeDriftObserved,
    WfeSilenceObserved,
    WfeDriftHypothesis,
    WfeProposalBlocked,
    WfeUnreachablePersisting,
}
```

### Server endpoint

```
POST http://localhost:8094/sx2/inbound
Content-Type: application/json

Request body: InboundEvent (one event per request)
Response: 200 { "acked": true, "wfe_observed_at_ms": <ms> }
         400 { "error": "<typed_err>" }    -- malformed event
         413 { "error": "payload_too_large" } -- DoS guard
         503 { "error": "wfe_draining" }   -- graceful-shutdown signal
```

### Handler dispatch

```rust
impl InboundHandler {
    pub fn dispatch(&self, event: InboundEvent) -> Result<HandlerOutcome, HandlerError> {
        match event.kind {
            WfeDriftObserved => self.route_to_m11_decay_hint(&event),
            WfeSilenceObserved => self.route_to_m16_budget_hint(&event),
            WfeDriftHypothesis => self.route_to_observation_log(&event),
            WfeProposalBlocked => self.route_to_audit_log(&event),
            WfeUnreachablePersisting => self.route_to_v5_bilateral_update(&event),
        }
    }
}
```

## Bilateral V5 integration (NA-3' closure)

The most important consumer is `WfeUnreachablePersisting` — when SX2's m10 has not received a heartbeat from WFE for N intervals, SX2 emits this event, and WFE's m43 receives it. The handler updates a (substrate-side mirror of) `WorkflowTraceParticipationStatus` → which closes the loop SX2 needs:

```
SX2 sees: WFE heartbeats stop arriving
  ↓ SX2-side WorkflowTraceParticipationStatus { mode: Crashed | Drained | NeverStarted } emit
  ↓ POST :8094/sx2/inbound { kind: WfeUnreachablePersisting, data: {participation_status, duration_ms} }
WFE m43 receives → handler routes to V5 bilateral mirror
  ↓ feeds m11 fitness decay (WFE knows SX2 knows WFE is down)
  ↓ optionally triggers self-restart escalation OR logs to operator
```

This is the **first concrete cross-loop bilateral V5 wire** — engine and substrate each carry an authoritative participation-status mirror, audit-distinguishable per NA-5.

## Refusal channel (when WFE refuses SX2 inbound)

Per V1 NA-5 + ADR D-S1004XXX-04, WFE m43 can also REFUSE SX2 events:

```
SX2 → POST :8094/sx2/inbound { kind: WfeDriftHypothesis, ... }
WFE m43:
  - 503 (WFE draining):         RefusalToken::Unavailable(SubstrateAuthored("wfe_draining"))
  - 400 (malformed):            RefusalToken::Unavailable(SubstrateAuthored("wfe_malformed_inbound"))
  - 413 (oversized):            RefusalToken::Unavailable(SubstrateAuthored("wfe_payload_too_large"))
  - hypothesis disagreement:    RefusalToken::Unavailable(SubstrateAuthored("wfe_hypothesis_rejected"))
```

Note the symmetry: WFE's refusals are SubstrateAuthored from SX2's perspective. The reciprocal of V5's audit-distinguishability now applies BOTH ways — engine's refusal to substrate AND substrate's refusal to engine.

## Pre-conditions (must be true before scaffold)

1. **Luke explicit phrase** — Zen 2026-05-24T102556Z: "Do not author new production bridge modules until Luke explicitly authorizes the layer." Need a phrase like "start coding m43" or "start coding wiring 02b" before scaffolding.
2. **W1 transport client SHIPPED** — ✅ done commit `2e9edff` 2026-05-24 (this enables the V5 bilateral wire — WFE can now both emit heartbeats AND receive heartbeat-silence acknowledgments).
3. **Port 8094 reservation** — confirm `:8094` doesn't collide with another habitat service per `~/claude-code-workspace/CLAUDE.md` 14-service table.
4. **Optional ADR** — cross-habitat coordination ADR D-S1004XXX-05 amendment naming the SX2→WFE direction as v0.2.2+ work item.

## Acceptance test (mirrors Wiring 01 NA-4 cond 3)

End-to-end test: SX2 mock client sends each of 5 event kinds; WFE m43 dispatches each to its handler; handler outcomes are observable in a mock substrate-state mirror. ~10-20 integration tests.

## Honest residuals

- `data: serde_json::Value` is opaque-typed-at-handler. A future refinement would type each kind's payload with a per-kind struct (mirrors the per-kind NexusEvent pattern in Wiring 02 § (source, kind) schema).
- Port 8094 is provisional; could collide. Audit before scaffolding.
- Authentication: ADR D-S1004XXX-04 says no app-layer auth for habitat-internal services; v0.3.0+ may add HMAC if substrate-mediated trust requires it.
- The handler-side action (e.g., `route_to_m11_decay_hint`) requires `m11` to expose an `apply_external_hint(...)` method — currently does not. Scaffolding m43 will surface a follow-up amendment to m11.

## Persistence anchors

| Surface | Anchor |
|---|---|
| ai_docs canonical | (this note) |
| Obsidian vault | `the-workflow-engine-vault/Wiring 02b — NexusEvent Inbound (SX2 → WFE).md` |
| stcortex (when scaffolded) | ns `workflow_trace_completion_s1004115` mem with pathways `wiring_02b_sx2_to_wfe_inbound ↔ wiring_plan_v2_s1004590` |
| Code (when scaffolded) | `src/m43_synthex_v2_inbound/` (5 files, ~460 LOC + tests) |
| CLAUDE.local.md | top banner § "Wiring 02b authored" + scope-status flag |
