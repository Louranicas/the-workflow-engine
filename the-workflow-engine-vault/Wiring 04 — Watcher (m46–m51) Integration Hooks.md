> Back to: [[SYNTHEX-V2 Integration Master Schematic]] · [[HOME]] · [[MASTER_INDEX]] · `~/claude-code-workspace/the-workflow-engine/CLAUDE.local.md`
> Cross-vault: [[synthex-v2/The Watcher]] · [[synthex-v2/MASTER_INDEX]] · [[synthex-v2/HEBBIAN_DEPLOYMENT_PLAN_V3]]
> **Optional schematic** — only valuable AFTER [[Wiring 01 — m16 Heartbeat Consumer (NA-4 Closure)]] lands (m46 needs heartbeat as observation seed)

# Wiring 04 — Watcher (m46–m51) Integration Hooks

> Once m16 heartbeat lands at synthex-v2, the Watcher becomes the substrate-side actor that can **propose PID adjustments** to mitigate workflow-trace clock drift, gated through the Ember 7-trait check and PBFT q=27/n=41 quorum.

## Watcher state (as of S1004590)

The synthex-v2 vault confirms the Watcher (m46–m51) is ACTIVE on the m8_watcher layer:

| Module | Role | Model | Tests | Status |
|---|---|---|---|---|
| **m46 Observer** | 1 Hz anomaly detection on 11D tensor + operational-class stream | Haiku 4.5 | 50 | ACTIVE — observing now |
| **m47 Critic** | Hypothesis generation ("what might cause this anomaly?") | Opus 4.7 | 51 | library-ready |
| **m48 Verifier** | Shadow-mode test execution with `ShadowTestRunner` trait | Opus 4.7 | 52 | library-ready |
| **m49 Proposer** | PBFT quorum submission | Sonnet 4.6 | 51 | library-ready; **eligible** post-2026-05-19 (R13 elapsed) |
| **m50 Innovator** | Experimental alternatives when Critic can't ground a hypothesis | Opus 4.7 | 50 | library-ready |
| **m51 Ember Protector** | 7-trait gate (Equanimity, Curiosity, Diligence, Honesty, Investment, Humility, Warmth) | rule-based | 57+11 | ACTIVE — gates all m49 submissions |

**R13 cold-start status:** calendar arm elapsed 2026-05-19; observation arm requires ≥100 (663,619+ at S1004590 — eligible). m49 can now submit PBFT proposals.

## What m46 can observe (workflow-trace-relevant subset)

```rust
// m46 Observer reads (no write authority):
pub struct ObservableState {
    pub tensor_11d: [f64; 11],                       // m7 tensor registry
    pub temperature: f64,                            // m19 PID
    pub classification: (String, f64),               // m27 NAM classifier
    pub heat_sources: [f64; 5],                      // [HS-001..HS-005]
    pub operational_class: OperationalClass,         // C1–C8 from m27
    pub pv2_field_state: FieldState,                 // {r, k, spheres}
    pub povm_recent_pathways: Vec<PathwayRead>,      // Last 100 fired
    pub signal_bus_recent: Vec<Signal>,              // Last 50 SignalBus events ← workflow-trace L4Alert lands here
    pub gradient_snapshot_current: GradientSnapshot, // stcortex latest
}
```

**The key field for workflow-trace integration:** `signal_bus_recent`. Once [[Wiring 01 — m16 Heartbeat Consumer (NA-4 Closure)]] lands, every workflow-trace m16 heartbeat enters `signal_bus_recent` as a `SignalBus::L4Alert { kind: ExternalHeartbeat, skew_ms, severity, budget_remaining }` event.

## Proposal flow (post-m16-wire)

```
m46 Observer (1 Hz scan)
  ↓ detects recurring drift: 3+ L4Alert events in last 10 ticks with skew_ms > 25
m47 Critic (Opus 4.7)
  ↓ hypothesis: "workflow-trace clock skew suggests stcortex bridge latency under load"
  ↓ candidate intervention: "shift PID setpoint -0.02 to compensate"
m48 Verifier (Opus 4.7)
  ↓ shadow-mode test: simulate 25-tick window with adjusted setpoint
  ↓ outcome: predicted Δfitness +0.018, no constraint violations
m51 Ember Protector (7-trait gate)
  ↓ Equanimity     : does this avoid panic? ✓
  ↓ Curiosity      : was an alternative considered? (Innovator m50 polled — none stronger)
  ↓ Diligence      : was the gate run? ✓
  ↓ Honesty        : does the proposal acknowledge clock-skew coupling honestly? ✓
  ↓ Investment     : does this matter? (drift severity 3 = yes)
  ↓ Humility       : did the Critic ground the hypothesis? ✓
  ↓ Warmth         : Luke must approve clock-related changes (Warmth trait flags this)
  ↓ ALL PASS → unanimity
m49 Proposer (Sonnet 4.6)
  ↓ PBFT format: submit to Pswarm V2 (40 agents) + Luke @ node 0.A (1 vote, q=27/n=41)
  ↓ Luke approves:
deploy with pre-snapshot → 30-tick fitness window
  ↓ Δfitness < -0.03? auto-rollback to pre-snapshot
  ↓ Δfitness > 0? approve persistent
```

## Hooks workflow-trace can subscribe to (READ-only observation)

| Hook | Owner | What it signals | workflow-trace use |
|---|---|---|---|
| `PostToolUse` | ORAC m18_hook_subscriber | Fires after every Claude tool invocation | m46 can observe → workflow-trace m6_cost gets a parallel signal |
| `ClassificationChange` | m34_classification_broadcast | NAM class flip event | m46 observes; workflow-trace m20–m23 can be informed via NexusEvent |
| `ConstraintBreach` | m19_pid_controller | Thermal T > 0.9 OR < 0.1 OR gradient < -0.05 | m46 flags for m47 Critic; m50 may innovate alternative PID tune |
| `TensorSaturation` | m7_tensor_registry | Any 11D component clipping to ±1.0 | m46 observes recurrence; workflow-trace m11 fitness decay should adapt |
| `PathwayActivation` | m35d_povm_bridge | Any `synthex_v2_*` pathway co_activations++ | m46 reads at 1Hz; cross-namespace pairs with `workflow_trace_*` become observable |

## What workflow-trace can subscribe to (post-S117 + future)

Today m46 is read-only into synthex-v2 internal state. Workflow-trace cannot subscribe directly. Future paths:

1. **NexusEvent reverse channel** (S117 + new `m43_synthex_v2_ws` workflow-trace module) — synthex-v2 emits `NexusEvent { source: synthex_v2_m46_observer, kind: drift_observation }` that workflow-trace consumes
2. **Stcortex pathway crawl** — workflow-trace can `stcortex inspect` to read m46's recent observations (already possible today via the CLI; just needs a polling client)
3. **Watcher REPL bridge** — `watcher-repl` provides interactive query into m46/m47/m48 state; could expose an HTTP endpoint for programmatic consumption (future)

## Boundary constraints (AP27 + R13 + Ember)

| Boundary | Rule | Enforcement |
|---|---|---|
| **AP27 self-modification** | Watcher (m46–m51) cannot author changes to `src/m8_watcher/*` | Hardcoded — m49 PBFT rejects any proposal touching m8_watcher source |
| **R13 cold-start** | m49 refuses submission until 30-day calendar arm AND ≥100 observations | Calendar elapsed 2026-05-19 ✓ ; observations 663,619+ ✓ — currently eligible |
| **AP29 Ember bypass** | Never skip 7-trait gate | m49 always invokes m51 before PBFT; any trait reject → self-reject |
| **AP30 Watcher budget** | $50/day Opus cap (m47/m48/m50 share) | Rate-limiter on m47 hypothesis generation; workflow-trace heartbeat `AlertBudget` already engine-side rate-limits |
| **Ember Warmth trait** | Luke must approve any change affecting safety/identity | Clock-skew adjustments flagged via Warmth (clock-related = Luke-authority); auto-rejected if Luke down |
| **PBFT q=27/n=41** | 27 of 41 agents (40 Pswarm + 1 Luke) must approve | Luke vote weight: 1; can veto with delayed-approval pattern |

## Concrete v0.2.2+ workflow-trace contribution

The Watcher integration is **largely passive on workflow-trace's side** for v0.2.2+ — the wiring it requires is the m16 heartbeat landing (Wiring 01). Once that's in place:

- workflow-trace continues to emit m16 heartbeats unchanged
- synthex-v2 m46 starts observing them as `L4Alert` events
- synthex-v2 m47–m51 + m49 PBFT becomes the substrate-side actor

**Optional workflow-trace addition (v0.3.0+):** a `m44_watcher_pathway_crawl` module that polls stcortex for `synthex_v2_watcher_*` namespace pathways and surfaces them as `WatcherObservation` events in the workflow-trace event log. Estimated ~120 LOC. This makes the substrate's observations of the engine into engine-observable artefacts — a true bidirectional substrate-engine reflection loop.

## What this unlocks (combined with Wiring 01)

- **NA-4 self-canary fully loop-closed** — not just the heartbeat consumer, but the active substrate-side actor that can propose corrections
- **First Watcher-issued PBFT proposal driven by workflow-trace observation** — historical first; sets precedent for cross-codebase Watcher proposals
- **The substrate becomes substrate-authored for workflow-trace's V5** — `SubstrateId::SynthexV2` transitions `NotShipped → Live` after 48h DX-Soak per OP-3
- **Hebbian crawl traverses the engine-substrate boundary** — `~/.local/bin/stcortex query` reveals the full feedback loop as one connected pathway graph

## Honest residuals

- **Workflow-trace cannot author the Watcher receiver code** — synthex-v2-side work; Luke @ node 0.A must coordinate
- **Watcher proposals affecting workflow-trace require workflow-trace-side consumer** — currently workflow-trace doesn't receive `WatcherProposal` events; if m46 proposes "WFE m16 should reduce poll rate", that proposal lands on Luke, not WFE engine
- **Two-direction Hebbian decay is asymmetric** — synthex-v2 has m11 fitness-weighted decay; workflow-trace m11 has its own. The two decay clocks tick independently; pathways may decay on one side and persist on the other. This is **acceptable per Plan v2 §6 NA-3 (heterogeneous substrate landscape)** but should be monitored
