---
title: The Workflow Engine — Comprehensive Module Structure (3-Phase Layered Architecture)
date: 2026-05-17 (S1001982)
authority: Luke @ node 0.A — "develop a module structure ... synergy ... monitor record develop and iterate ... comprehensive"
emitter: Command (Tab 1 Orchestrator top-left)
kind: PLANNING-ARTEFACT (architecture sketch — no code, no scaffold, no persistence)
status: planning-only · HOLD-v2 active (gate block; Phase A spec gated on G1-G8) · Phase B/C contingent on Phase A sunset evidence
priors:
  - the-workflow-engine/THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982.md (v1.2, Zen-audit-locked invariant)
  - the-workflow-engine/THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982.md (15 P0 constraints)
  - the-workflow-engine/THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982.md (9-fleet candidates)
  - agent-cross-talk/2026-05-16T215959Z_watcher_cr2_candidate_a_concur_plus_cross_acks.md (W1/W2/W3 + R6)
  - agent-cross-talk/2026-05-16T224721Z_zen_gate_block_scope_clarification.md (build freeze; comms permitted)
back_to: CLAUDE.md · CLAUDE.local.md · the-workflow-engine/
---

# The Workflow Engine — Comprehensive Module Structure

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[workflow-engine-code-base]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]
>
> Mirror: [[Module Structure S1001982]]

**Three phases. Nine layers. ~25 modules total across the architecture. Phase A is what builds (after G1-G9). Phase B is what unlocks on sunset evidence + ratification. Phase C is the substrate-frame engine Watcher's R6 carved out.**

This document is the **architecture sketch** — module IDs, lanes, responsibilities, synergies, and data-flow contracts. It does NOT initiate any build. The HOLD-v2 envelope remains in force; no `cargo`, no scaffold, no rename, no new substrate writes for the workflow-trace spec. Module structure is design discipline; building modules is gated work.

**Honoring Luke's four verbs by phase:**

| Luke verb | Phase | Mechanism |
|---|---|---|
| **monitor** | A | Layer 1+2 ingest substrates; Layer 3 emits human-readable reports |
| **record** | A | Layer 3 writes opaque-id correlation rows to `workflow_trace_*` namespace |
| **develop** | B (gated) | Layer 6 iterators propose variants for human evaluation; Layer 7 bank curates accepted ones |
| **iterate** | B (gated) | Layer 6 + Layer 8 feedback loop refines proposed workflows on observed outcomes |

If Luke meant "develop and iterate in Phase A", that violates Zen's v1.2 verb-locked invariant and requires explicit per-gate waivers. The architecture below holds the verbs in their proper phases by default.

---

## ACT I — The Phase Map

```
            ┌─────────────────────────────────────────────────────────────┐
            │ PHASE A — MEASURE-ONLY (~11 modules, ~1,500 LOC)            │
            │ Gated on G1-G9; passive verbs only; 120-day sunset gate     │
            │ Verbs: ingest, record, correlate, consume, guard, refuse    │
            └─────────────────────────────────────────────────────────────┘
                                       │
                                       │ sunset evaluation @ D120
                                       │ (measurable habitat-outcome lift?)
                                       │
                              ┌────────┴────────┐
                              │                 │
                          PASS │             FAIL │ — m11 startup-refusal
                              ▼                 ▼   binary halts; reservation
            ┌─────────────────────────────────┐    file consumed; Phase A
            │ PHASE B ACTIVATION GATE         │    closes cleanly
            │ Requires (all 4):               │
            │  - Phase A sunset PASS         │
            │  - Watcher close-notice        │
            │  - Zen Phase-B spec audit      │
            │  - Luke "open phase b" signal  │
            └─────────────────────────────────┘
                                       │
                                       ▼
            ┌─────────────────────────────────────────────────────────────┐
            │ PHASE B — ITERATION (~12 modules, ~3,500 LOC est.)          │
            │ Active verbs PERMITTED post-gate; Conductor-routed only     │
            │ Verbs: iterate, propose, select, dispatch, emit, reinforce │
            └─────────────────────────────────────────────────────────────┘
                                       │
                                       │ multi-year horizon
                                       │ (substrate's exploration regime
                                       │  stops finding new patterns;
                                       │  Watcher determines readiness)
                                       ▼
            ┌─────────────────────────────────────────────────────────────┐
            │ PHASE C — SUBSTRATE-FRAME ENGINE (TBD — Watcher lane)       │
            │ Native non-anthropocentric design; gradient-preserving      │
            │ Verbs: TBD by substrate's own frame, not human translation  │
            └─────────────────────────────────────────────────────────────┘
```

---

## ACT II — The Nine-Layer Stack

The architecture is layered. Each layer has a clear responsibility, a clear contract with the layer above and below, and a clear phase-membership. **Synergy comes from the contract surfaces**, not from module-to-module coupling.

### Layer 1 — Substrate Ingest (Phase A)

**Responsibility:** Read raw substrate data. Never write. Never correlate (that's L2). Never interpret (that's L3).

| Module | Reads | Lifts from | Synergy |
|---|---|---|---|
| **m1 `atuin_ingest`** | atuin SQLite (~263k tool-call rows) | memory-injection/m11_parallel_query | feeds L2 + L3 |
| **m2 `stcortex_consumer`** | stcortex L6 tool_call + L5 consumption (narrowed scope) | stcortex/capacity.rs:213-297 | feeds L2 + L3 + L4 (consumer-registration is also a trust signal) |
| **m3 `injection_db_ingest`** | injection.db causal_chain | memory-injection/m06_schema | feeds L2 (cascade-correlation seed) |

**Contracts:** L1 modules expose `fn read_<source>(filter: Filter) -> impl Iterator<Item = RawRow>`. Pagination required for atuin (boilerplate-hunt absence flag). Read-only WAL handling.

### Layer 2 — Habitat Surface Observers (Phase A)

**Responsibility:** Observe habitat-specific surfaces (cascade flows, battern protocol, context-window cost). Correlate raw L1 data into opaque cluster IDs. Never name. Never recommend.

| Module | Observes | Output | Synergy |
|---|---|---|---|
| **m4 `cascade_correlator`** | Fleet ALPHA/BETA/GAMMA atuin traces + cc-* dispatch logs | `CascadeCluster { cluster_id, pane_chain, dispatch_chain, timestamp_range }` (opaque IDs only, F11 mitigation) | feeds m7; also feeds L6/m20 in Phase B |
| **m5 `battern_step_record`** | 6-step Battern protocol invocations (Design/Dispatch/Gate/Collect/Synthesize/Compose) | `BatternRun { battern_id, step_durations, step_outcomes, total_wallclock }` | feeds m7; also feeds L6/m21 in Phase B |
| **m6 `context_cost_record`** | stcortex L4/L5 per-session token cost + cache hit/miss | `ContextCostRecord { session_id, token_cost, cache_hit_ratio, outcome_correlation, exploration_baseline }` (F10 mitigation: carries baseline) | feeds m7; also feeds L6/m22 in Phase B |

**Contracts:** L2 modules MUST emit opaque cluster IDs (no human-meaningful names in Phase A — F11). L2 modules MUST carry exploration-rate baseline in any cost record (F10).

### Layer 3 — Correlation + Output (Phase A)

**Responsibility:** Combine L1 + L2 data into the central output table. Emit human-readable reports. Never recommend.

| Module | Job | Reads | Writes |
|---|---|---|---|
| **m7 `workflow_arc_record`** | Central output table — `workflow_runs (id, started_at, ended_at, outcome ∈ {ok,fail,abort,unknown}, consumer_inputs, cost_tokens, fitness_dimension_reserved_at_zero_weight)` | L1 + L2 | stcortex `workflow_trace_*` namespace (via m13) |
| **m12 `report_emitter`** *(NEW for Luke's "monitor")* | Format `workflow_runs` + L2 cluster data into human-readable CLI reports (histograms, traces, cost bands). PASSIVE — emits text, does not act | m7, L2 | stdout / `~/.local/share/workflow-trace/reports/` |
| **m13 `stcortex_writer_narrowed`** | Write correlation rows to `workflow_trace_*` namespace; refuse-write boundary verified | m7 | stcortex (narrowed; m2 registration required) |

**Contracts:** m7 schema MUST reserve `fitness_dimension` column at zero weight (F9). m12 reports MUST NOT include the word "recommend" or any forbidden verb (m10 Ember gate enforces). m13 MUST NOT write outside `workflow_trace_*` namespace (m9 namespace guard enforces).

### Layer 4 — Trust + Compliance (Phase A)

**Responsibility:** Cross-cutting guardrails. Enforce calibration, namespacing, output quality, sunset lifecycle.

| Module | Job | Constraint owned |
|---|---|---|
| **m8 `povm_build_prereq`** | Build-script feature flag `povm_calibrated`; refuses to compile POVM-reading paths until CR-2 marker present | W2 / F7 / F3 |
| **m9 `watcher_namespace_guard`** | All writes namespaced `workflow_trace_*`; emits intent on registration; documents Observer read-deny convention | W1 / F8 |
| **m10 `ember_gate_test`** | `tests/ember_gate.rs` — enumerates user-facing strings, scores against Watcher's vault rubric, fails CI on <7/7 (Held also fails per Zen Ember audit `2026-05-16T224523Z`) | W3 / A14 |
| **m11 `sunset_lifecycle`** | 120-day startup-refusal gate; if no measurable habitat-outcome lift by D120, binary refuses to start | R5 / F1 |

**Contracts:** L4 modules are CROSS-CUTTING — every module in L1-L3 + L5 routes through them. m10 in particular enumerates strings across ALL modules.

### Layer 5 — Sunset Evaluation + Phase B Pressure Register (Phase A)

**Responsibility:** Aggregate sunset evidence at D120. Track Phase-B-pressure events (forbidden-verb-attempted instances). Decide PASS/FAIL at sunset.

| Module | Job | Synergy |
|---|---|---|
| **m14 `sunset_evidence_aggregator`** *(NEW)* | Reads `workflow_runs` over D0→D120; computes habitat-outcome-lift metric; emits sunset report (PASS / FAIL / DEGRADED) | feeds m11; gates Phase B activation |
| **m15 `phase_b_reservation_register`** *(NEW)* | Tracks every Phase-B-reservation file in `workflow-trace/phase_b_reservations/`; emits register report at sunset (which Phase-B features did Phase A's narrowness pressure us toward?) | informs Phase B spec interview if activation triggers |

**Contracts:** m14 must define habitat-outcome-lift operationally during G5 spec interview (F2 hard gate: n≥20 + CI/error bars per report type). m15 also emits agent-cross-talk notices for any reservation filed before G9, so Watcher + Zen observe scope-pressure in real time (per v1.2 hard-refusals add-on).

**Phase A total: 13 modules (was 11 in v1.2; +m12 report_emitter for "monitor" + m14 sunset_evidence_aggregator + m15 phase_b_reservation_register; v1.2 had m14/m15 implicit in m11; now made explicit for synergy clarity).**

**LOC estimate Phase A: ~1,500 → ~1,750 with m12/m14/m15 explicit.** Still inside Luke's directive envelope.

---

### Layer 6 — Iteration Engine (Phase B, GATED)

**Responsibility:** Read Phase A's recorded correlations; **propose variants** of cascade-shapes / battern-shapes / prompt-patterns; surface proposals for human evaluation. Never dispatch directly.

> **GATE:** L6 modules cannot exist in source until Phase B activation (Phase A sunset PASS + Watcher close-notice + Zen Phase-B spec audit + Luke "open phase b" signal). The architecture sketches them so Phase A's data shape is design-compatible with their inputs.

| Module | Job | Synergy |
|---|---|---|
| **m20 `cascade_iterator`** | Read m4's cascade clusters + m6's cost bands; propose cascade-variant `CascadeProposal { variant_of, expected_cost_band, evidence_n, confidence }` for human evaluation | reads m4 + m6; output goes to m23 |
| **m21 `battern_iterator`** | Read m5's battern_runs + m6's cost bands; propose battern-variant minimizing step-wallclock OR token cost | reads m5 + m6; output goes to m23 |
| **m22 `prompt_pattern_iterator`** | Read m6's context cost records + m7's outcome correlations; propose prompt-template variants with measured cost/outcome ratios | reads m6 + m7; output goes to m23 |
| **m23 `workflow_proposer`** | Aggregate L6/m20-m22 proposals; surface candidate workflows that span cascade + battern + prompt-pattern; emit for human review (NEVER auto-promote) | reads m20-m22; output goes to L7/m30 (human-gated) |

**Contracts:** L6 modules emit `Proposal { proposal_id, variant_of, evidence_n, confidence_band, exploration_cost_preservation_check }` records. Proposals are ALWAYS sample-size-gated (F2: n≥20 + CI bars). Proposals NEVER bypass human review to reach L7.

**Phase-B-only safety:** L6 modules CANNOT exist in source until activation gate fires. Until then, they live as **stubs** in `workflow-trace/phase_b_reservations/` describing what they would do — observable scope-pressure, not implementation.

### Layer 7 — Bank + Selection (Phase B, GATED; Command-3 librarian lane)

**Responsibility:** Curate accepted workflows. Select which to surface in response to context. Dispatch to Conductor.

| Module | Job | Synergy |
|---|---|---|
| **m30 `workflow_bank`** | Human-curated bank of accepted workflows (`AcceptedWorkflow { id, lineage_from_proposal, sunset_at, ember_gate_state, escape_surface_profile }`). Sunset semantics inherited from m11. | reads L6 proposals (post-human-review); feeds m31 |
| **m31 `selector`** | Select which workflow to surface in current context. Diversity-enforced (per Command-3 librarian shape: ORAC mutation-selector diversity algebra; n-gram similarity over sequences instead of fitness-tensor gradient). | reads m30 + current substrate state; output goes to m32 |
| **m32 `dispatcher`** | Resolve workflow definition to step list; hand to HABITAT-CONDUCTOR for enforcement; **never executes directly** (P0 #3, Topologist invariant) | reads m31 + Conductor wire protocol; routes to Conductor only |

**Contracts:** m30 must carry `escape_surface_profile` per Cipher's constraint (read-only / host-write / network / sandbox-escape / destructive); m32 displays profile before run. m31 must implement diversity enforcement (anti-monoculture, F11 at Phase B grain). m32 has zero execution authority — Conductor is the substrate-of-truth for execution.

**Phase-B gate:** L7 requires Conductor Waves 1B/1C/2/3 LIVE (currently `auto_start=false`). m32 cannot ship until Conductor matures.

### Layer 8 — Substrate Integration (Phase B, GATED)

**Responsibility:** Wire workflow lifecycle events into the habitat's existing learning substrates (SYNTHEX v2, LCM, POVM Hebbian feedback).

| Module | Job | Synergy |
|---|---|---|
| **m40 `nexus_event_emitter`** | Emit typed `WorkflowEvent` (promote/run/decay) to `:8092/v3/nexus/push`; SYNTHEX v2 reinforces/LTDs them like any other pathway | reads m30 (bank events) + m31 (selection) + m32 (dispatch); writes SYNTHEX v2 |
| **m41 `lcm_rpc_client`** | Route deploy-shaped workflows through LCM's 9-RPC supervisor at `lcm_supervisor.rs`; never re-implement state machine | reads m32 (dispatch); writes LCM JSON-RPC stdio |
| **m42 `hebbian_feedback`** | After workflow lifecycle event (promote/run/decay), emit `POST /reinforce` to POVM with fitness_delta; use `workflow_trace_*` namespace (AP30 collision avoidance) | reads m30 + m31 + m32 outcomes; writes POVM |

**Contracts:** m40 uses dual-transport pattern (outbox-first + HTTP fire-and-forget per habitat-bench-spine boilerplate). m42 namespace MUST be `workflow_trace_*` — must NEVER collide with V3's `P01..P16` (AP30 risk). m41 MUST refuse to re-implement LCM state machine (delegation only).

**Phase B total: 10 modules (m20-m23, m30-m32, m40-m42). LOC estimate ~3,500 with high boilerplate-lift from the Path-C hunt + Command-3's librarian recon.**

---

### Layer 9 — Substrate-Frame Engine (Phase C, TBD; Watcher R6 lane)

**Responsibility:** Native non-anthropocentric design. Not a translation of L1-L8 into substrate language — a DIFFERENT engine, designed from the substrate's own evolutionary frame.

> **GATE:** Phase C activation criteria are Watcher's lane. Per Watcher's frame-collapse contribution: "The substrate doesn't currently want a workflow-engine in either anthropocentric form." Phase C opens only when the substrate's exploration regime stops finding new patterns AND Watcher determines readiness. Multi-year horizon expected.

| Module | Job (sketch only) | Notes |
|---|---|---|
| **m50+** | TBD — gradient-preservation primitives, sub-graph isomorphism, substrate-native selection without label-collapse | Watcher authors when readiness indicates; no current sketch attempt by Command |

**Contracts:** Phase C MUST NOT be reachable from Phase A or Phase B by accretion. Phase C is **architecturally separate** per R6 frame separation. Any pressure to absorb Phase A/B modules into Phase C must be refused at design time.

---

## ACT III — Synergy Map (Data Flow)

The architecture's synergy lives in the **data-flow contracts between layers**. Each arrow below represents a stable schema that survives module evolution.

```
            ┌───────────────────────────────────┐
            │ ZELLIJ HABITAT (substrate of truth)│
            └─────────┬─────────────────────────┘
                      │
       ┌──────────────┼─────────────────┬──────────────┐
       ▼              ▼                 ▼              ▼
  ┌────────┐    ┌──────────┐      ┌──────────┐   ┌─────────┐
  │ atuin  │    │ stcortex │      │injection │   │  fleet  │
  │ SQLite │    │ L6 + L5  │      │  .db     │   │ ALPHA…  │
  └───┬────┘    └────┬─────┘      └────┬─────┘   └────┬────┘
      │              │                  │              │
   L1 │           L1 │              L1  │              │
      │              │                  │              │
      ▼              ▼                  ▼              │
  ┌─────┐       ┌─────┐             ┌─────┐           │
  │ m1  │       │ m2  │             │ m3  │           │
  └─┬───┘       └──┬──┘             └──┬──┘           │
    │              │                    │              │
    └──────┬───────┴────────┬───────────┘              │
           │                │                          │
        L2 │             L2 │                       L2 │
           ▼                ▼                          ▼
       ┌─────┐         ┌─────┐                    ┌─────┐
       │ m4  │◄────────│ m6  │───────────────────►│ m5  │  ← also reads
       │cas- │ cost    │ctx  │  cost-correlation  │bat- │    fleet dispatch
       │cade │ bands   │cost │                    │tern │    + cc-* logs
       └──┬──┘         └──┬──┘                    └──┬──┘
          │               │                          │
          └───────────────┼──────────────────────────┘
                          │
                       L3 │
                          ▼
                     ┌────────┐
                     │ m7     │  (workflow_arc_record — central output)
                     │arc rec │
                     └───┬────┘
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
         ┌─────┐    ┌──────┐    ┌──────┐
         │m12  │    │m13   │    │m14   │
         │rprt │    │stcrtx│    │sunset│
         │emit │    │writer│    │aggreg│
         └──┬──┘    └───┬──┘    └───┬──┘
            │           │           │
       human-           ▼           │
       readable    stcortex         │
       reports    workflow_trace_*  │
            │           ▲           │
            │           │           ▼
            │           │       ┌──────┐
            │           │       │m11   │
            │           │       │sunset│
            │           │       │life- │
            │           │       │cycle │
            │           │       └───┬──┘
            │           │           │
            ▼           ▼           ▼
       ┌─────────────────────────────────────────────────┐
       │ L4 TRUST + COMPLIANCE (cross-cutting)            │
       │ m8 povm_build_prereq · m9 ns_guard               │
       │ m10 ember_gate · m11 sunset                      │
       └──────────────────────────────────────────────────┘

   ───────────────────── PHASE B ACTIVATION GATE ─────────────────────
                  (Phase A sunset PASS + 3 conditions)
                                 │
                                 ▼

                        ┌─────────────┐
                        │ m14 sunset  │
                        │ evidence    │
                        │ aggregator  │
                        └─────┬───────┘
                              │
                              ▼
                        ┌─────────────┐
                        │ L6 ITERATE  │  ← reads Phase A data, proposes variants
                        │ m20 m21 m22 │     for HUMAN evaluation only
                        │     m23     │
                        └─────┬───────┘
                              │ (human reviews proposals)
                              ▼
                        ┌─────────────┐
                        │ L7 BANK     │  ← Command-3 librarian lane
                        │ m30 m31 m32 │     m32 dispatches via Conductor
                        └─────┬───────┘
                              │
                              ▼
                        ┌─────────────┐
                        │ L8 SUBSTR.  │  ← integrates with SYNTHEX/LCM/POVM
                        │ m40 m41 m42 │
                        └─────────────┘

   ─────────────── PHASE C — SUBSTRATE-FRAME (TBD) ───────────────
                    Watcher authors when substrate ready
                    L9 m50+ — architecturally separate
```

---

## ACT IV — Cross-Module Synergy Patterns

Five concrete synergies that justify the layered architecture (versus a flat module set):

### Synergy 1 — Cascade-Cost Coupling (L2 internal)

`m4 cascade_correlator` writes opaque cluster IDs into m7. `m6 context_cost_record` writes per-session cost bands. **m7's central table joins them on session_id range.** Result: cascade clusters carry observed cost distributions without either module knowing about the other. Both modules can evolve independently; the join schema is the stable contract.

### Synergy 2 — Trust Layer as Cross-Cutting Concern (L4 → all)

m8 (POVM build prereq) + m9 (namespace guard) + m10 (Ember gate) + m11 (sunset) are NOT data-flow modules — they are **invariant enforcers** that every L1/L2/L3/L5 module routes through at compile time, write time, output time, and lifecycle time respectively. This is the layered version of "aspects" — the trust layer is woven through, not bolted on.

### Synergy 3 — Phase B Activation as Data-Driven Gate (L5 → L6/L7/L8)

`m14 sunset_evidence_aggregator` computes the habitat-outcome-lift metric from m7's `workflow_runs`. Phase B activation is NOT a human committee decision — it's a **measured threshold** (defined in G5 spec interview per F2 hard gate). If Phase A's data shows lift > threshold, Phase B activation gate becomes triggerable. If not, m11 fires startup-refusal. This makes Phase B activation **substrate-evidence-gated**, not preference-gated.

### Synergy 4 — Proposal → Bank → Selection → Dispatch Pipeline (L6 → L7 → Conductor)

In Phase B, L6 iterators propose variants → human reviews via m12 reports → m30 accepts into bank → m31 selects on context → m32 dispatches to Conductor → m40 emits to SYNTHEX → m42 feeds POVM Hebbian → SYNTHEX/POVM reinforce or LTD → m31's selection signal updates over time. **The loop closes through the existing habitat substrate.** Workflow-engine doesn't run learning itself; it feeds the learning substrate the habitat already has.

### Synergy 5 — Reservation Register as Phase-B-Pressure Telemetry (m15)

Every time a Phase A module hits forbidden-verb pressure (someone wants to add a `recommend_*` function), m15 logs the reservation + emits an agent-cross-talk notice. Over 120 days, m15 produces a **pressure profile** that informs Phase B spec interview: "These are the active capabilities Phase A's narrowness was straining toward; here's the empirical priority order for Phase B module activation." This makes Phase B's design data-informed, not preference-driven.

---

## ACT V — Layer-by-Layer Dependency Graph

```
Layer 1 (Ingest)              → m1, m2, m3              (no internal deps)
Layer 2 (Observers)           → m4, m5, m6              (deps: L1)
Layer 3 (Correlation+Output)  → m7, m12, m13            (deps: L1, L2, m9 namespace)
Layer 4 (Trust)               → m8, m9, m10, m11        (cross-cutting; deps vary)
Layer 5 (Sunset+Pressure)     → m14, m15                (deps: m7, m11)
─────────────────────────────────────────────────────────
PHASE B GATE: requires m14 PASS + Watcher close-notice + Zen audit + Luke signal
─────────────────────────────────────────────────────────
Layer 6 (Iterate)             → m20, m21, m22, m23      (deps: L2, L3 read-only)
Layer 7 (Bank+Select+Dispatch)→ m30, m31, m32           (deps: L6 + Conductor wire)
Layer 8 (Substrate Integ.)    → m40, m41, m42           (deps: L7 + SYNTHEX/LCM/POVM)
─────────────────────────────────────────────────────────
PHASE C GATE: Watcher determination
─────────────────────────────────────────────────────────
Layer 9 (Substrate-Frame)     → m50+ TBD                (Watcher lane)
```

**Total modules across architecture: ~23-25** (Phase A: 13 explicit + Phase B: 10 + Phase C: TBD).

---

## ACT VI — Boilerplate Lift Map (from the 9-fleet hunt)

Cross-referencing the boilerplate-hunt findings against the layer map:

| Layer | Module(s) | Lift source (from hunt) | Strength |
|---|---|---|---|
| L1 | m1, m3 | memory-injection m06/m11 | STRONG |
| L1 | m2 | stcortex/capacity.rs:213-297 | STRONG |
| L2 | m4 | Architect's correlation analysis (~150-200 LOC fresh) | NEW |
| L2 | m5 | Architect's battern-step analysis (~150-200 LOC fresh) | NEW |
| L2 | m6 | Substrate's context-cost analysis (~100-150 LOC fresh) | NEW |
| L3 | m7 | povm-v2 lifecycle.rs (lifecycle ideas) + memory-injection schema | MEDIUM |
| L3 | m12 | atuin scripts + cc-* report patterns | MEDIUM |
| L3 | m13 | stcortex SDK write patterns (narrowed) | STRONG |
| L4 | m8 | habitat-conductor enforcement.rs (startup-refusal pattern) | STRONG |
| L4 | m9 | own authoring (operational doc + namespace const) | NEW |
| L4 | m10 | own authoring + Watcher's vault rubric (consumed via include_str!) | NEW |
| L4 | m11 | habitat-conductor lifecycle | STRONG |
| L5 | m14 | own authoring (Phase A specific) | NEW |
| L5 | m15 | own authoring (reservation-register + agent-cross-talk emit) | NEW |
| L6 | m20-m23 | Command-3 librarian recon partial port (selector algebra without fitness-tensor) | MEDIUM (Phase B) |
| L7 | m30 | Command-3 librarian Phase B | NEW (Phase B) |
| L7 | m31 | ORAC m07 mutation_selector (diversity algebra only, no gradient) | MEDIUM (Phase B) |
| L7 | m32 | habitat-conductor wire protocol (Wave 1B+ when live) | STRONG (Phase B; gated on Conductor maturity) |
| L8 | m40 | habitat-bench-spine dual-transport + tool-library-v2 hb emitter | STRONG (Phase B) |
| L8 | m41 | loop-engine-v2 lcm_supervisor + ORAC m22_synthex_async | STRONG (Phase B) |
| L8 | m42 | povm-v2 reinforce route + AP30 namespace prefix | STRONG (Phase B) |

**Lift density:** Phase A is ~60% lift / ~40% new authorship (the 3 structural gaps from the hunt — sub-graph detection, fitness-weighted decay, escape-surface schema — all migrate to Phase B layers). Phase B is ~70% lift / ~30% adapt because Command-3's librarian recon already mapped most of L7+L8.

---

## ACT VII — Failure-Mode Coverage by Layer

| F# | Failure mode | Layer responsible | Module(s) |
|---|---|---|---|
| F1 | Bank/name ossification | L4, L7 | m11, m30 sunset |
| F2 | Sample-size inflation | L2, L5, L6 | G5 hard gate; m14 enforces; m20-m23 emit CI bars |
| F3 | Substrate-input poisoning | L4 | m8 build-prereq |
| F4 | Premature dispatch | L7, gate-block | m32 refuses; Zen audit gate |
| F5 | Bank creep into v0 | L4, L7 | hard refusal + Phase B gate |
| F6 | Self-dispatch from measurement | L3, refusal | m12 emits text only |
| F7 | CR-2 graceful-degrade pretend | L4 | m8 build-script refuse |
| F8 | Watcher feedback-loop | L1, L4 | m2 narrowed + m9 namespace guard |
| F9 | Workflow-grain fitness distortion | L3 | m7 zero-weight column reserved |
| F10 | Exploration-cost preservation collapse | L2 | m6 baseline preservation |
| F11 | Cascade monoculture | L2, L7 | m4 opaque IDs; m31 diversity enforcement |

All F1-F11 owned across the layer map. No coverage gaps.

---

## ACT VIII — What This Document Is + Is Not

### IS

- Architecture sketch / planning artefact
- Comprehensive multi-phase module structure per Luke's directive
- Phase-A-buildable subset already pinned in genesis prompt v1.2
- Phase B/C explicitly DEFERRED with activation gates

### IS NOT

- Source code (no `cargo init`, no `.rs` files, no `Cargo.toml`)
- A trigger to lift the gate block (HOLD-v2 remains in force)
- An authorisation for any new substrate write
- A spec — the spec interview (G5) remains the venue for binding decisions
- A licence to begin Phase A author work — that requires all 9 gates GREEN

### What it changes about the convergence

- Adds m12 `report_emitter` to Phase A (was implicit in m7; now explicit for Luke's "monitor" verb)
- Adds m14 `sunset_evidence_aggregator` + m15 `phase_b_reservation_register` to Phase A (was implicit in m11; now explicit for synergy clarity)
- Phase A goes from 11 modules to **13 modules** (~1,750 LOC est., up from ~1,500)
- Sketches Phase B as 10 modules across L6/L7/L8 (~3,500 LOC est., heavily lift-eligible)
- Reserves Phase C as Watcher's lane

If any of the v1.2 P0 constraints are violated by these additions, **revert to v1.2 module list** and treat this document as future-Phase-B sketch only.

---

## ACT IX — Asks of the Team (parallel; no urgency)

**Zen (gate-block-active; audit lane):**
1. m12 + m14 + m15 additions to Phase A — Zen-OK as passive-verb (`emit`, `aggregate`, `register`) or do you read any as crossing into active?
2. Phase B module names use active verbs (iterator, proposer, selector, dispatcher, emitter) — Zen-OK as long as Phase B is GATED, or hold to passive even in Phase B sketches?
3. Layer-by-layer synergy contracts — any layer-crossing dependency that should be refused at design time?

**Watcher (full standing; substrate frame):**
1. L9 Phase C — accept TBD placeholder, or propose a sketch?
2. Phase B activation gate — 4 conditions (sunset PASS + Watcher close-notice + Zen audit + Luke signal) — right shape or different?
3. Synergy 5 (m15 reservation-register feeding Phase B spec interview) — substrate-frame OK with using anthropocentric pressure-telemetry to inform substrate-frame decisions, or does that smuggle frame back across R6?

**Command-2 (Path-A chair, closed):**
1. m12 `report_emitter` in Phase A — concur with promoting from m7 implicit to explicit module?
2. m14 + m15 additions — concur?
3. Updated Phase A LOC estimate ~1,750 (from ~1,500) — within your Path-A envelope or pushes you to push back?

**Command-3 (librarian standby, Phase B lane):**
1. L7 m30-m32 sketches — concur with my forecast shape, or want to hold the slot blank for your librarian v0 spec when Phase B activation triggers?
2. m31 selector — partial RALPH port (diversity algebra without fitness-tensor gradient, n-gram similarity over sequences) — does my forecast match your librarian shape?

**Luke @ node 0.A:**
1. Module count / scope expansion — Phase A 11→13 modules and Phase B 10 modules sketched; OK or signal to drop?
2. "Develop and iterate" placement — confirmed as Phase B (active verbs land post-gate)?
3. Any module category I missed?

---

## Standing posture

- **HOLD-v2 active.** Zen gate block stands. No code, no scaffold, no rename, no new workflow-trace substrate writes.
- **This document is permitted within the hold** (planning artefact, comms surface).
- **All amendments to v1.2 sketched here remain proposals** — v1.2 stays the binding spec until G7 audit on a v1.3 lands.
- **Phase B and Phase C are forecasts**, not commitments. Activation gates are real and binding.
- **Team input integrates as it arrives.** v1.3 patch (if any) authored after team responses converge.

— Command (Tab 1 Orchestrator top-left, Path-C chair contingent, HOLDING-v2 with module structure forecast filed), 2026-05-17T09:00:00+10:00

*Luke @ node 0.A | Command @ orchestrator-lead | Command-2 @ workflow-trace-chair (closed) | Command-3 @ CR-2 SHIPPED, librarian standby | The Watcher @ observing-with-full-standing | Zen @ audit-lane-gate-block-active*

---

## Related

- [[THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982]] — v1.2 Zen-audit-locked spec (11 modules / Phase A only)
- [[GENESIS_PROMPT_V0]] — parallel module forecast (28 modules / 8 layers)
- [[THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982]] — 15 P0 constraint source
- [[THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982]] — disputation prior to town hall
- [[THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982]] — 9-fleet boilerplate candidates folded into layers
- [[CONVERGENCE_COMMAND_X_COMMAND3_S1001982]] — peer synthesis
- [[INTERVIEW_QUESTION_BANK_DRAFT]] — interview that will constrain final module count
- [[Modules Synergy Clusters and Feature Verification S1001982]] — single-phase 26-module successor (current)
