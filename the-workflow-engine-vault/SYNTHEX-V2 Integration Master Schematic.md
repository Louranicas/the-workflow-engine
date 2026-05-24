> Back to: [[HOME]] · [[MASTER_INDEX]] · `~/claude-code-workspace/the-workflow-engine/CLAUDE.local.md`
> Cross-vault: [[synthex-v2/MASTER_INDEX|synthex-v2 MASTER_INDEX]] · [[synthex-v2/SYNTHEX_V2_ULTRAMAP]] · [[synthex-v2/BRIDGE_TOPOLOGY_CONTINUITY]] · [[synthex-v2/The Watcher]]

# SYNTHEX-V2 Integration — Master Schematic

> **⚠️ SUPERSEDED IN PART 2026-05-24 by [[Wiring Plan v2 — Source-Verified Integration S1004590]].**
> Source verification (Wave 2, S1004590) found material schema + architecture drift in the 4 child schematics (Wiring 01 § 3.2 wire shape; Wiring 02 stub vs LIVE; Wiring 03 slug-families; Wiring 04 signal-bus claim). Plan v2 is the integrated source-grounded plan; this Master Schematic retains as historical umbrella with per-schematic AMENDMENT markers.
> **§ "7-substrate generalisability audit" still pending — author per Plan v2 § S1.**

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

## 7-substrate generalisability audit (NA-4' resolution, Plan v2 § S1)

> Per NA-4' (gap analysis HIGH finding): the synthex-v2 pattern is NOT generalisable. Each of the 7 NA-2 substrates has a different consumption surface. This table is the honest per-substrate audit — for each, what wire shape (if any) is the integration target, and what's the honest blocker.

| Substrate | Consumption surface | Heartbeat-shape fit? | Integration path | Honest blocker |
|---|---|---|---|---|
| **synthex-v2** | HTTP daemon `:8092` + WebSocket `/ws/orac` + 8-channel signal bus + m46 TensorSnapshot consumer | YES (engine-side enrichment per Plan v2 § S2; m16 heartbeat lands at m10 → TensorSnapshot path per NA-1'') | [[Wiring 01 — m16 Heartbeat Consumer (NA-4 Closure)]] + [[Wiring Plan v2 — Source-Verified Integration S1004590]] | synthex-v2-side endpoint authoring (~400 LOC, operator-only); Option A vs B TensorSnapshot decision (Zen + Luke) |
| **stcortex** | SpacetimeDB module `:3000` + write_pathway / write_memory reducers + Rust SDK | NO (heartbeat is liveness signal; stcortex is durable substrate — wrong shape) | [[Wiring 03 — stcortex Pathway Namespace Alignment]] (pathway-pair emission, not heartbeat) | Slug-namespace discipline already in place (P30); workflow_trace_outcomes single family today |
| **HABITAT-CONDUCTOR** | Per-pane Zellij IPC + file-drop comms + 4 binaries (`weaver`/`weaver-tail`/`zen`/`enforcer`) + `auto_start=false` Batch 5 | NO (per-pane dispatch coordination via IPC; no HTTP daemon for heartbeats; lifecycle/control plane requires Luke per CLAUDE.md) | Conductor-side liveness via existing `:8141/health` probe (LCM corridor — Command-2's lane per Zen 102556Z) | LCM chunk-3a not yet authorised; Wiring schematic for WFE→Conductor not yet authored (separate scope) |
| **CC-5 loop clocks** | In-process m11 fitness decay + m14 lift evidence + m15 pressure register (workflow_core lib internals) | YES intra-process (already in place) | Internal m11→m14→m15 wire (CC-7 PressureEvent → m23 compose-priority, shipped v0.1.0) | No external integration needed — CC-5 is engine-internal |
| **Luke + Watcher ☤** | Human + persona pair; cross-pane file-drop (`~/projects/shared-context/agent-cross-talk/`) + WCP (`watcher notify`) | NO (no machine consumer — Luke is decisional, Watcher is observer) | Operator-only via D1-D7 + WCP notices | This substrate is decisional/observational, not heartbeat-consuming. The "heartbeat" Luke sees is the commit chain itself; the Watcher's is the synthex-v2 m46 stream |
| **RALPH** | In-process fitness loop on ORAC sidecar `:8133`; observable via `/ralph` endpoint | NO direct (RALPH is observer of fitness gradient, not heartbeat consumer) | Read-only fitness gradient pull (LCM Wiring W5 lane per Zen 102556Z); informs WFE's own m11 decay weight | RALPH is on ORAC, not addressable as WFE→RALPH push; only WFE→ORAC observation pull |
| **Cargo build graph** | Build-time substrate; observable via `cargo metadata` + `cargo check` + `cargo clippy` JSON output | NO (build-time only; not runtime; no live heartbeat surface) | CI integration via `.github/workflows/ci.yml` + `.gitlab-ci.yml` (already in place per v0.1.0) | None — CI is the heartbeat path for this substrate |

**Honest generalisation conclusion:**

Of the 7 NA-2 substrates:
- **1** (synthex-v2) has a true HTTP heartbeat-consumer integration path — that's what Wiring 01 + Plan v2 § S2 specifies
- **1** (stcortex) has a wholly different pathway-emission path — Wiring 03
- **1** (HABITAT-CONDUCTOR) is operator-coordinated (Command-2's lane)
- **1** (CC-5) is engine-internal — already wired
- **2** (Luke+Watcher, RALPH) are observational/decisional — heartbeat-shape is a category error
- **1** (Cargo build graph) is build-time — CI is the heartbeat

**No single "wiring pattern" generalises across all 7.** Each substrate needs its own design, audited honestly. v0.2.2+ horizon focuses on synthex-v2 (the highest-leverage HTTP daemon integration target); the other 6 are operator-coordinated, already wired, or out-of-scope for engine-driven push integration.

---

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
