> Back to: [[HOME]] · [[MASTER_INDEX]] · `~/claude-code-workspace/the-workflow-engine/CLAUDE.local.md`
> Cross-vault: [[synthex-v2/MASTER_INDEX|synthex-v2 MASTER_INDEX]] · [[synthex-v2/SYNTHEX_V2_ULTRAMAP]] · [[synthex-v2/BRIDGE_TOPOLOGY_CONTINUITY]] · [[synthex-v2/The Watcher]]

# SYNTHEX-V2 Integration — Master Schematic

**Source of truth:** `/tmp/synthex-v2-wiring-discovery-for-workflow-trace.md` (551-line read-only Explore discovery, 2026-05-24, S1004590).
**Target:** wire workflow-trace's Cluster H (m40–m42 substrate feedback) + V3 m16 KEYSTONE into the LIVE synthex-v2 daemon on `:8092`, closing **OP-6 / NA-4 self-canary loop** (highest residual from v0.2.0 SHIPPED).

## TL;DR — the wiring problem

v0.2.0 shipped the engine half of substrate-safety: m16 emits a `Heartbeat` envelope every cycle, m40–m42 emit `NexusEvent` envelopes, m42 writes `workflow_trace_*` stcortex pathways. **No synthex-v2 consumer is wired for any of these.** The synthex-v2 vault confirms:

- :8092 already accepts `/v3/nexus/push` POST (REST) + `NexusEvent` WS variant — **just needs the workflow-trace SENDER**
- :8125 stcortex accepts `/pathways` POST with P30-enforced namespace — **m35d_povm_bridge validates `synthex_v2_*` prefix; workflow-trace must use `workflow_trace_*`**
- **m16 Heartbeat consumer DOES NOT EXIST in synthex-v2** — 5 landing options enumerated below; **Option B (m10_http_poller + new POST /v3/heartbeat) recommended** (~400 LOC, 45–60 min)
- Watcher (m46–m51) is ACTIVE; m49 Proposer is now PBFT-eligible (R13 calendar arm elapsed 2026-05-19)

## 8-layer synthex-v2 architecture (~57,998 LOC, 2850 tests, ALL SEALED)

```
L1 Foundation         m1_foundation/        — error taxonomy, types, signal bus
L2 Ingest             m2_ingest/            — m09–m13: STDB sub, HTTP poll, EventBus, SQLite, ingest router
L3 Processing         m3_processing/        — tensor 11D, NAM classifier, heat sources
L4 Regulation         m4_regulation/        — m19 PID, m20–m26 heat fusion + saturation cap
L5 Feedback Loop      m5_feedback/          — m27 classifier, m29 policy router, m30 classification feedback
L6 Action             m6_action/            — m31 k_adj, m32 broadcast, m33 consolidation, m34 broadcast, m35a–m35j 10 bridges
L7 Memory             m7_memory/            — m39–m45: 4-tier working/short/long/episodic + HMX + consolidation
L8 Watcher            m8_watcher/           — m46 Observer · m47 Critic · m48 Verifier · m49 Proposer · m50 Innovator · m51 Ember Protector
```

**Daemon tasks (Tokio runtime):** observer · observer_writer · regulation_tick · classification_broadcast · consolidation · ws_inbound_writer · thermal_sync + the 6 Watcher loops (m46–m51).

## The 3 wiring schematics (workflow-trace → synthex-v2)

| # | Schematic | Leverage | Status | LOC est |
|---|---|---|---|---|
| 1 | [[Wiring 01 — m16 Heartbeat Consumer (NA-4 Closure)]] | **🔴 HIGHEST** — loop-closes V3 KEYSTONE (operationally inert at v0.2.0) | NEW endpoint needed synthex-v2 side | ~400 |
| 2 | [[Wiring 02 — NexusEvent Bidirectional Push]] | MEDIUM — engine already EMITS, synthex-v2 already CONSUMES via m13→m29→subscribers; wire is one-sided trivial | Receiver exists; sender needs config | ~120 |
| 3 | [[Wiring 03 — stcortex Pathway Namespace Alignment]] | MEDIUM — Hebbian co-activation pairs across the loop boundary become first-class | Wire exists; namespace discipline required | ~80 |
| 4 | [[Wiring 04 — Watcher m46–m51 Integration Hooks]] | OPTIONAL — once m16 heartbeat lands, m46 Observer can propose PID adjustments via m49 PBFT | Watcher ACTIVE; needs heartbeat as observation seed first | ~200 |

## Critical surfaces at a glance

### Wire-level (synthex-v2 LIVE, port 8092)

| Surface | Protocol | Path | Direction (WFE→SX2) | Used for |
|---|---|---|---|---|
| `/api/health` | HTTP GET | — | probe-only | liveness check |
| `/v3/thermal` | HTTP GET | — | probe-only | thermal state read |
| `/v3/nexus/push` | HTTP POST | JSON `{events: [...]}` | **m40 NexusEvent emit** | drift alerts, command results, pathway-learned events |
| `/v3/heartbeat` | HTTP POST | **DOES NOT EXIST YET** | **m16 Heartbeat emit (NA-4 closure)** | substrate-drift canary |
| `/api/ingest` | HTTP POST | JSON `FieldState\|CascadeEvent\|CascadeHeartbeat` | currently PV2 fire-and-forget | could be re-used by WFE for cascade ingest |
| `/api/classify` | HTTP GET | — | probe-only | NAM class read |
| `/ws/orac` | WebSocket | 14-variant `WsEnvelope` | bidi (ORAC-only consumer today) | future WFE channel after S117 lands |

### Substrate-level (stcortex :3000, POVM :8125)

| Surface | Endpoint | Namespace | Direction | Used for |
|---|---|---|---|---|
| stcortex `write_pathway` | reducer | `workflow_trace_*` (P30 prefix discipline) | **m42 stcortex_emit** | Hebbian bidi co-activation pairs |
| stcortex `write_memory` | reducer | `workflow_trace_completion_s1004115` etc. | session anchors (existing) | already in use |
| POVM `/pathways` | HTTP POST | (no namespace concept) | overlap mirror → 2026-07-10 | legacy reader compat only |

### Signal-bus level (in-process Arc)

| Signal | Origin | Consumer | Wiring impact |
|---|---|---|---|
| `L4Alert { skew, budget, severity }` | m16 heartbeat (via m13 ingest router post-wire) | m19 PID controller | adjusts setpoint on detected substrate clock skew |
| `L8Observation { drift_event }` | m46 Observer (post-heartbeat-landing) | m47 Critic | seeds hypothesis ("what's causing the workflow-trace clock drift?") |

## The flow (post-wiring) — m16 Heartbeat → PBFT proposal

```
workflow-trace m16_substrate_drift_canary (1 Hz emit)
  ↓ POST :8092/v3/heartbeat { skew_envelope, alert_budget, ... }
synthex-v2 m10_http_poller (NEW endpoint; ingest tier L2)
  ↓ IngestSignal::Heartbeat
m13_ingest_router (NEW variant + handler)
  ↓ SignalBus::L4Alert
m19_pid_controller (CONSUMES; existing — no change needed)
  ↓ adjusts PID setpoint OR queues for m46
m46 Observer (1 Hz tensor scan, sees L4Alert via signal_bus_recent)
  ↓ if recurring drift → flag for m47
m47 Critic — hypothesis: "WFE clock drift suggests stcortex bridge latency"
  ↓
m48 Verifier — shadow test: simulate 25-tick window with PID adjustment
  ↓
m51 Ember Protector — 7-trait gate (Equanimity/Curiosity/Diligence/Honesty/Investment/Humility/Warmth)
  ↓ if PASS:
m49 Proposer — PBFT submit (Pswarm V2 40-agent quorum + Luke @ node 0.A 1 vote, q=27/n=41)
  ↓ Luke approves:
deploy with pre-snapshot → 30-tick fitness window → auto-rollback if Δfitness < -0.03
```

**This is the substrate-side completion of the v0.2.0 NA-5 audit-distinguishability contract** — once wired, the engine's NotShipped→Live transition for `SubstrateId::SynthexV2` becomes substrate-authored, not engine-imagined.

## Honest residuals (gaps the wiring exposes)

1. **m16 receiver landing is a synthex-v2-side write** — workflow-trace cannot create the endpoint; this is a coordinated cross-habitat ship per ADR D-S1004XXX-05.
2. **AP27 self-modification boundary** — Watcher cannot author its own m46 changes; if the receiver lands at L8 instead of L2, Luke @ node 0.A must author it.
3. **R13 cold-start** — m49 calendar arm elapsed 2026-05-19; observation arm requires ≥100 obs (already 663,619+ at S1004590 checkpoint — eligible).
4. **POVM CR-2 inflation** — pre-CR-2 binary still serving `learning_health=0.9146` on some configs; workflow-trace m42 ADR routes substrate-feedback writes to stcortex (POVM-decoupled) — this risk is structurally avoided.
5. **`#![forbid(unsafe_code)]`** on workflow-trace must be preserved across the bridge — synthex-v2's `WsMessage` enum and `IngestSignal` must round-trip without unsafe interop.

## Cross-vault anchors (bidirectional)

| synthex-v2 vault note | What it gives the wiring |
|---|---|
| [[synthex-v2/MASTER_INDEX]] | 110 wikilinks, gold-standard navigation |
| [[synthex-v2/SYNTHEX_V2_ULTRAMAP]] | 8-layer map, full module roster |
| [[synthex-v2/BRIDGE_TOPOLOGY_CONTINUITY]] | 7-consumer matrix (PV2/ORAC/VMS/ME V2/V3/TL V2/Nerve); add WFE as 8th |
| [[synthex-v2/The Watcher]] | m46–m51 persona doc; 10 POVM anchors |
| [[synthex-v2/DAEMON_INTEGRATION_PLAN]] | 4-surface daemon plan; m16 endpoint addition would extend this |
| [[synthex-v2/HEBBIAN_DEPLOYMENT_PLAN_V3]] | 18-phase deployment; workflow-trace pathway pairs join the Hebbian substrate |
| [[synthex-v2/ORAC_SYNTHEX_WS_Bridge_S116]] | WS protocol 14 variants; precedent for future WFE WS channel post-S117 |
| [[synthex-v2/S117_PHASE_25_SPEC]] | adds Hello/CapabilityTrace/IntentToken/A2aTask/EpistemicBroadcast — future WFE wiring lane |

## Reverse-anchor invariant

Every schematic listed above (Wiring 01–04) opens with `> Back to: [[SYNTHEX-V2 Integration Master Schematic]] · [[HOME]] · [[MASTER_INDEX]]` so the four-surface persistence discipline closes round-trip in two hops from any landing point.
