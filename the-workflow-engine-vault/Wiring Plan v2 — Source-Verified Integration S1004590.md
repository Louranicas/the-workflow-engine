> Back to: [[SYNTHEX-V2 Integration Master Schematic]] · [[Wiring Gap Analysis — S1004590 Dual-Frame]] · [[HOME]] · [[MASTER_INDEX]] · `~/claude-code-workspace/the-workflow-engine/CLAUDE.local.md` (§ "Wiring Plan v2")
> **Status:** RATIFICATION-CANDIDATE (Zen audit in flight; subject to "I am ready to deploy" gate)
> **Supersedes (in part):** Wiring 01/02/03/04 + Master Schematic — they retain as historical drafts; **THIS NOTE is the integrated source-verified plan**.

# Wiring Plan v2 — Source-Verified Integration (S1004590)

> **Verdict:** dual-frame gap analysis + third-pass source verification together force a material plan revision. Five amendments + 1 new sibling schematic. The honest residual count grows, not shrinks — but the plan is now **source-grounded** rather than schematic-aspirational.

## TL;DR — what's changed since Wiring 01–04 + Gap Analysis

| Finding | Disposition |
|---|---|
| **CONV-1 CRITICAL** `/v3/nexus/push` is 501 stub (commit `c9eeb75` Bug Hunt Armada W1-005 fix) | ✅ HELD — Wiring 02 pre-flight gates synthex-v2 Phase 3 land |
| **CONV-2 HIGH** m13→m29→4-subscriber chain INTERNAL only | ✅ HELD — Wiring 02 splits current-internal vs future-HTTP-triggered |
| **CONV-3 SPECULATIVE → CRITICAL** m16 wire schema | **ESCALATED.** Actual `Heartbeat { emitted_at_ms: u64, cycle: u64 }` ONLY (verbatim from `src/m16_substrate_drift_canary/mod.rs`). The fat envelope Wiring 01 § 3.2 claimed (`sequence_number, timestamp_ms, skew_envelope, alert_budget, heartbeat_source`) does NOT exist. Either Wiring 01 enriches engine-side before transport, OR the wire shape is the minimal heartbeat. **Wiring 01 § 3.2 must be amended to actual schema.** |
| **CONV-4 SPECULATIVE → MAJOR DRIFT** 10 m42 slug families | **ESCALATED.** Only `workflow_trace_outcomes` emits today (verified via grep + tests). 9 other families are SPECIFIED-only with zero emission sites. **Wiring 03 must split IMPLEMENTED vs SPECIFIED-for-v0.3.0+ tables.** Forward-only single-write (no bidi pair as claimed). |
| **NA-1' HIGH** m46 signal_bus_recent cap of 50 evicts heartbeats | **REFUTED.** m46 does NOT read signal_bus at all — it consumes a `TensorSnapshot` passed to `tick()` (file:`src/m8_watcher/m46_watcher_observer.rs:544`). Has its own isolated 60-sample rolling baseline per dimension (deterministic FIFO). Signal bus has 8 channels × 1024 capacity (`broadcast::Sender`, default `DEFAULT_CAPACITY = 1024`). **`HeartbeatBuffer` ring requirement DROPPED.** The actual integration gap is different — see NA-1'' below. |
| **NA-1'' (NEW HIGH replacing NA-1')** — m46 consumes TensorSnapshot, not signal bus; m16 heartbeats need to influence a tensor dimension (or a new 12th dimension) to reach m46 | **NEW.** Wiring 01 must specify the TensorSnapshot path: either (a) extend the 11D tensor to 12D adding `wfe_clock_skew_ms` dimension, OR (b) feed m22 capability_trace from m16 alerts (existing pathway). |
| NA-2' HIGH "Bidirectional Push" unidirectional | ✅ HELD — Wiring 02 renames + Wiring 02b authored |
| NA-3' HIGH no reverse `WorkflowTraceParticipationStatus` | ✅ HELD — substrate-authorship request via watcher-notice to Zen |
| NA-4' HIGH single-substrate completion theatre (7-substrate generalisability) | ✅ HELD — Master schematic gains §"7-substrate generalisability audit" |
| NA-5' MED Watcher feedback loop doesn't reach WFE | ✅ HELD — Wiring 04 renamed |
| NA-6'/7'/8' MED + NA-9'/10' LOW | ✅ HELD |

## Per-schematic dispositions (final)

### S1: [[SYNTHEX-V2 Integration Master Schematic]] — AMEND

**Required amendments:**
- Add §"7-substrate generalisability audit" table (NA-4'): per-substrate consumption surface analysis for atuin / stcortex / HABITAT-CONDUCTOR / CC-5 loop clocks / Luke+Watcher / RALPH / Cargo build graph
- Add §"Opportunity cost honesty" (NA-7'): name what SX2-side v0.2.2+ items this wiring competes with
- Add §"Source-truth verification status" pointing at this Plan v2 + source-verified evidence

**Effort:** ~1 hr authoring.

### S2: [[Wiring 01 — m16 Heartbeat Consumer (NA-4 Closure)]] — MAJOR AMEND

**Required amendments:**
1. **§ 3.2 Wire schema** — REPLACE the speculative fat envelope with the actual source:

```rust
// VERBATIM from src/m16_substrate_drift_canary/mod.rs (v0.2.0):
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Heartbeat {
    pub emitted_at_ms: u64,
    pub cycle: u64,
}
```

2. **§ 3.2 Engine-side enrichment** — NEW section specifying that the WFE transport client enriches the minimal heartbeat with engine-side context BEFORE POST:

```rust
// NEW: src/m16_substrate_drift_canary/transport.rs (v0.2.2+)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatWireEnvelope {
    pub heartbeat: Heartbeat,                              // verbatim from m16
    pub heartbeat_source: &'static str,                    // "workflow-trace::m16_substrate_drift_canary"
    pub boot_id: Uuid,                                     // NEW per NA-10' — set once per process boot
    pub instance_id: &'static str,                         // process identifier
    pub skew_summary: SkewSummary,                         // derived from m16's last DetectionResult
    pub alert_budget_remaining: u16,                       // derived from m16's AlertBudget state
}
```

3. **§ 3.3 Landing flow** — REPLACE the signal-bus path. NEW path:

```
workflow-trace m16  ── POST :8092/v3/heartbeat ──┐
                                                  ↓
                       synthex-v2 m10_http_poller (L2 ingest, NEW endpoint)
                                  ↓
                       NEW DESIGN CHOICE (NA-1''):
                       Option A: 12th tensor dimension `wfe_clock_skew_ms`
                                 added to m07_tensor_registry (TensorSnapshot grows)
                       Option B: feed m22 capability_trace from m16 alerts
                                 (existing m22 pathway)
                                  ↓
                       m46 WatcherObserver.tick(snapshot, class, at)
                       reads enriched tensor via existing 60-sample
                       per-dimension rolling baseline → detects drift
```

4. **§ Goodbye / shutdown contract** — NEW per NA-8': m16 transport emits a `Goodbye { boot_id, final_cycle, reason }` envelope on graceful shutdown via the same endpoint; m10 tracks `last_heartbeat_at_ms` per `boot_id` and emits internal `WfeSilencePersisting` signal after 3× poll interval missed.

5. **§ Replay semantics** — NEW per NA-10': SX2 tracks `(boot_id, last_seen_cycle)` per emitter; cycle reset OR boot_id change after seeing seq>0 = restart event, logged but not alert.

6. **§ Bilateral V5** — NEW per NA-3': reference watcher-notice substrate-authorship request to Zen for `WorkflowTraceParticipationStatus { NotShipped, Shipping, Live }`.

**Effort:** ~2 hr authoring + need to confirm with synthex-v2 maintainer (Zen) which of Option A/B (tensor extension vs m22 capability_trace feed) is preferred — this is a substrate-design decision.

### S3: [[Wiring 02 — NexusEvent Bidirectional Push]] — RENAME + UPDATE

**Required amendments:**
- Rename file to "Wiring 02 — NexusEvent Outbound (WFE → SX2)" (deferred — git rename)
- § Current state — replace "synthex-v2 side (LIVE)" with "synthex-v2 side (501 STUB; Phase 3 pending per commit `c9eeb75`)"
- § 4.3 Routing chain — distinguish "current internal flow (working)" vs "future HTTP-triggered flow (Phase 3 pending)"
- Add § "Pre-flight gate" naming synthex-v2 Phase 3 landing as hard precondition
- Update transport client to detect 501 and route through `RefusalToken::Unavailable(SubstrateUnreachable { transport_reason: "synthex_v2_phase3_pending" })`

**Effort:** ~45 min authoring.

### S4: NEW — [[Wiring 02b — NexusEvent Inbound (SX2 → WFE)]] — AUTHOR

**New schematic specifying SX2 → WFE channel** (NA-2'):
- WFE-side consumer module `m43_synthex_v2_inbound` scaffolded (~300 LOC)
- Receiver shape: HTTP server on workflow-trace side (e.g., :8094) OR WebSocket client to /ws/orac (S117 future)
- Refusal channel design: SX2 emits `WorkflowTraceUnreachable { ... }` event when WFE silence persists
- Integration with V5 bilateral: SX2 m46 observations of WFE-state → events consumed by WFE m43 → fed to m11 fitness decay weight

**Effort:** ~2 hr authoring.

### S5: [[Wiring 03 — stcortex Pathway Namespace Alignment]] — SPLIT TABLES

**Required amendments:**
1. § "Catalogue of workflow-trace pathway slug families (canonical)" — SPLIT into:

   **IMPLEMENTED (v0.2.0 ship — 1 family):**

   | Family | Emitter | Pair-with |
   |---|---|---|
   | `workflow_trace_outcomes` | m42 `emit_feedback` via m13 `promote_run` | (forward-only, no bidi pair) |

   **SPECIFIED-FOR-v0.3.0+ (in ai_specs, zero emission sites today — 9 families):**

   | Family | Emitter (when implemented) | Pair-with (planned) |
   |---|---|---|
   | `workflow_trace_m16_heartbeat` | m16 transport (Wiring 01 v2) | `synthex_v2_pid_adjustment` |
   | `workflow_trace_m20_prefixspan_pattern_<id>` | m20 | `synthex_v2_classifier_class_<class>` |
   | `workflow_trace_m22_kmeans_cluster_<id>` | m22 | `synthex_v2_classifier_class_<class>` |
   | `workflow_trace_m23_proposer_variant_<id>` | m23 | `workflow_trace_m30_bank_promoted_<id>` |
   | `workflow_trace_m30_bank_promoted_<id>` | m30 | `workflow_trace_m32_dispatch_<id>` |
   | `workflow_trace_m32_dispatch_verdict_<id>` | m32 | `synthex_v2_m29_policy_routed_<kind>` |
   | `workflow_trace_m33_verifier_<kind>_<verdict>` | m33 | `workflow_trace_m30_bank_<verdict>_action_<id>` |
   | `workflow_trace_m40_nexus_emit_<kind>` | m40 | `synthex_v2_m13_ingest_routed_<kind>` |
   | `workflow_trace_m41_lcm_cmd_<cmd>_exit_<code>` | m41 | `synthex_v2_m31_k_adj_<delta>` |

2. § "Forward + reverse pair (Hebbian)" — UPDATE: today is FORWARD-ONLY (verified). Bidi pair convention applies only post-v0.3.0+ when emission sites land.
3. § "Bridge-validator (post-v0.2.2+ recommendation)" — KEEP (this is correct guidance; just hasn't been written yet).
4. Drop NA-9' `parent_namespace` field reference.

**Effort:** ~30 min authoring.

### S6: [[Wiring 04 — Watcher (m46–m51) Integration Hooks]] — RENAME + HONEST PASS

**Required amendments:**
- Rename file to "Wiring 04 — Watcher Observation of WFE (read-only; no WFE feedback loop)" (deferred — git rename)
- § What m46 reads — CORRECT the architectural claim. m46 consumes `TensorSnapshot` via `tick()`, not `signal_bus_recent`. Update accordingly.
- § "What this does NOT do" — NEW section: m46 observations of WFE drift inform synthex-v2's own PID/heat tuning, NOT WFE's m16 emit rate or m11 decay weight. Substrate's agency is over substrate; feedback to WFE requires `m44_watcher_pathway_crawl` (new WFE module per C1').
- Promote `m44_watcher_pathway_crawl` to first-class v0.3.0+ item, not buried as honest residual.

**Effort:** ~1 hr authoring.

## Revised NA-4 closure 7-condition acceptance test

Plan v2 §6 NA-4 row must NOT update from "mitigated structurally" → "loop-closed" until ALL of:

1. ✅ synthex-v2 ships POST `/v3/heartbeat` endpoint at m10 (CONV-1 unblock)
2. ✅ **TensorSnapshot integration path** chosen (Option A: 12th dimension `wfe_clock_skew_ms` added to m07; OR Option B: m22 capability_trace fed from m16 alerts) — m46 must actually OBSERVE m16-derived drift via its existing TensorSnapshot path, NOT signal_bus_recent (NA-1' refuted; NA-1'' substituted)
3. ✅ End-to-end acceptance test demonstrates: WFE emits 60 heartbeats over 60s; m46's 60-sample rolling baseline on the chosen dimension exhibits a measurable drift-anomaly z-score; m47 Critic detects within 10s of pattern onset
4. ✅ Bilateral V5 `WorkflowTraceParticipationStatus` ships on SX2-side (NA-3')
5. ✅ WFE-down-mid-burst liveness contract (m16 `Goodbye` envelope on graceful shutdown + m10 `WfeSilencePersisting` signal after 3× poll interval missed) (NA-8' + new NA-10')
6. ✅ Reverse-channel `m43_synthex_v2_inbound` exists in WFE for SX2→WFE refusal channel (NA-2')
7. ✅ 48h DX-Soak per OP-3 with the full chain LIVE — drift detected end-to-end at least once during soak (NOT just wire existence)

**Wire existence is necessary but not sufficient.** Condition 3 (end-to-end drift detection under load) is the actual loop-closure gate.

## v0.2.2+ horizon (final, source-grounded)

| # | Item | Effort | Source |
|---|---|---|---|
| 1 | Author amendments per §S1–S6 dispositions (in this note) | ~7 hr | this plan |
| 2 | Author Wiring 02b (SX2 → WFE) | ~2 hr | NA-2' |
| 3 | Zen audit collaboration + bilateral V5 substrate-authorship | external | NA-3' + this plan |
| 4 | Decide Option A vs Option B for TensorSnapshot integration | requires Zen + Luke @ node 0.A | NA-1'' |
| 5 | Track synthex-v2 Phase 3 landing for `/v3/nexus/push` | external | CONV-1 |
| 6 | Scaffold `m43_synthex_v2_inbound` + `m44_watcher_pathway_crawl` modules | ~600 LOC + tests | C1' |
| 7 | 7-substrate generalisability audit (Master schematic addition) | ~3 hr | NA-4' |
| 8 | NA-4 7-condition acceptance test SPEC | ~2 hr | C2' |
| 9 | Original assessment items (Conductor schema, strum::EnumCount) | per 7-facet | 7-facet |

## Methodology trail (audit-grade evidence chain)

1. **Wave 0 (S1004590 earlier)** — authored Wiring 01-04 + Master Schematic from Explore agent report `/tmp/synthex-v2-wiring-discovery-for-workflow-trace.md`
2. **Wave 1a — Conventional gap pass** — Explore agent verified vs synthex-v2 source: `/tmp/synthex-v2-deep-evidence-for-gap-analysis.md` (324 lines)
3. **Wave 1b — NA gap pass** — na-gap-analyst surfaced 10 findings + 3 tensions + 2 convergent: `/tmp/synthex-v2-wiring-plan-na-gap.md` (305 lines)
4. **Wave 2a — Source-truth verification (this note's substrate)** — 3 parallel Explore agents read `src/m16_substrate_drift_canary/mod.rs` + `src/m42_stcortex_emit/mod.rs` + `synthex-v2/src/m8_watcher/m46_watcher_observer.rs`. Found NA-1' REFUTED + CONV-3/4 ESCALATED.
5. **Wave 2b (this note)** — integrated plan synthesises all 4 evidence streams.

**Confidence:** 0.92 (high — every claim has file:line evidence or is explicitly honest-deferred).

## Honest residuals (carried forward)

- Wiring 01 § 3.3 Option A (12th tensor dim) vs Option B (m22 capability_trace) — **needs Zen + Luke decision**, not agent-resolvable.
- All amendments above are authored-as-intent, not committed source code. Wiring schematics' headline contracts will be wrong-but-honest until amendments land (this note flags the wrongness explicitly).
- Zen audit reply (filed `2026-05-24T070000Z_command_zen_audit_request_wiring_gap_analysis_s1004590.md`) is pending; "I am ready to deploy" should wait for either (a) Zen verdict, (b) explicit ratification from Luke @ node 0.A, or (c) claim-verifier + four-surface-persistence-verifier passes with this note as plan-of-record.
- The 5 wiring schematics (Wiring 01-04 + Master) remain in vault as historical drafts — this Plan v2 is the source-grounded current iteration.

## Persistence

| Surface | Anchor |
|---|---|
| ai_docs canonical | (this note) + source reports `/tmp/synthex-v2-*.md` |
| Obsidian vault | THIS NOTE + Wiring 01/02/03/04 + Master + Gap Analysis as historical |
| stcortex | ns `workflow_trace_completion_s1004115` — new mem with parent_ids chain back to gap analysis + assessment + hardening |
| POVM | mirror per overlap → 2026-07-10 |
| injection.db | `causal_chain` row (NEW) labelled `workflow_trace_wiring_plan_v2_s1004590` |
| CLAUDE.local.md | top banner § "Wiring Plan v2" |
| Cross-talk inbox | Zen AUDIT-REQUEST filed `2026-05-24T070000Z_command_zen_audit_request_wiring_gap_analysis_s1004590.md` |

---

*Plan v2 authored S1004590, 2026-05-24. Supersedes Wiring 01-04 schematic headline claims; supplements them with source-verified amendments. Ratification gated on Zen verdict OR Luke @ node 0.A explicit OR dual-verifier pass.*
