---
title: Modules · Synergy Clusters · Feature Verification (single-phase, S1001982)
date: 2026-05-17 (S1001982)
status: planning-only · single-phase deployment (Luke override) · HOLD-v2 active
authority: Luke @ node 0.A — "yes no phases this is to be deployed in one phase ... list all modules then list all synergy clusters within and across modules verifying all features and capacities that have been flagged"
---

# Modules · Synergy Clusters · Feature Verification (single-phase)

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[workflow-engine-code-base]] · `~/claude-code-workspace/the-workflow-engine/` · [[CLAUDE.md]] · [[CLAUDE.local.md]]

This is the comprehensive single-phase architecture absorbing Luke's override of the convergence's phased recommendation. **26 modules · ~5,200 LOC · 8 synergy clusters · 7 cross-cluster synergies · 30 flagged features verified (28 owned, 2 dependency-gated).**

The architecture is the substantive deliverable; the waiver record below is what changed from the phased-recommendation default.

---

## What Luke's override waives (per-constraint, explicit)

| Constraint | Source | Waiver status |
|---|---|---|
| Watcher R6 frame separation (Phase A is not seed of substrate-frame engine) | Watcher | ⚠ **PARTIALLY WAIVED** — substrate-frame engine remains TBD because no design exists; the *protection* against anthropocentric absorption is gone |
| Fossil's evidence-based scope discipline (build measure-only first; expand on evidence) | Fossil persona | ⚠ **FULLY WAIVED** — committing to full ~5,200 LOC architecture at genesis; ancestor-rhyme risk on Command head |
| RALPH selector-without-measurement safety | RALPH persona | ⚠ **PARTIALLY MITIGATED** — m31 carries diversity enforcement but lacks 120 days of empirical baseline before going live |
| Skeptic's pain-source verification (build measure-only; sunset if unverified) | Skeptic persona | ⚠ **FULLY WAIVED** — committing without injection.db/MEMORY.md evidence of Luke-articulated pain |
| Substrate's exploration-protection (labelling collapses ambiguity) | NA Gap Analyst | ⚠ **PARTIALLY MITIGATED** — F10 baseline preservation still in m6; F11 opaque-IDs still in m4 + m31; no longer phase-gate-protected |

**NOT waived:** G1-G9 pre-genesis gates remain in force. Phasing was *internal* to deployment; gates are *pre*-deployment. v1.2's verb-locked invariant relaxes (active verbs now permitted across full architecture) but Zen audit at G7 still required.

---

## All 26 modules

| # | Module | Job | LOC est. |
|---|---|---|---|
| **m1** | `atuin_ingest` | Read atuin SQLite tool-call history (~263k rows; paginated) | ~80 |
| **m2** | `stcortex_consumer` | Register narrowed consumer (`tool_call` + `consumption` only); refuse-write at DB layer | ~80 |
| **m3** | `injection_db_ingest` | Read injection.db causal_chain table | ~70 |
| **m4** | `cascade_correlator` | Correlate Fleet ALPHA/BETA/GAMMA atuin + cc-* dispatch logs → opaque cluster IDs | ~180 |
| **m5** | `battern_step_record` | Observe 6-step Battern protocol (Design/Dispatch/Gate/Collect/Synthesize/Compose) durations + outcomes | ~150 |
| **m6** | `context_cost_record` | Per-session token cost ↔ outcome; exploration-rate baseline preservation (F10) | ~130 |
| **m7** | `workflow_arc_record` | Central output table; `fitness_dimension` column zero-weight reserved (F9) | ~150 |
| **m8** | `povm_build_prereq` | Build-script feature gate `povm_calibrated`; refuses to compile until CR-2 marker present (W2) | ~50 |
| **m9** | `watcher_namespace_guard` | All writes namespaced `workflow_trace_*`; documents Observer read-deny (W1/F8) | ~30 |
| **m10** | `ember_gate_test` | `tests/ember_gate.rs` — 7-trait scoring against Watcher vault rubric; Held fails per Zen Ember audit (W3) | ~100 |
| **m11** | `engine_sunset_lifecycle` | Engine-wide sunset law (default 120d unless Luke specifies); RALPH fitness-weighted decay for workflows in bank | ~120 |
| **m12** | `report_emitter` | Format `workflow_runs` + cluster data into human-readable CLI reports (histograms, traces, cost bands) | ~120 |
| **m13** | `stcortex_writer_narrowed` | Write correlation rows to `workflow_trace_*`; Hebbian LTP/LTD headroom check before promotion | ~100 |
| **m14** | `evidence_aggregator` | Aggregate `workflow_runs` over time; emit habitat-outcome-lift metric; informs m11 decay + m31 selection | ~120 |
| **m15** | `pressure_register` | Track forbidden-verb-pressure events; emit `agent-cross-talk/` notices so Watcher + Zen observe scope-pressure | ~80 |
| **m20** | `cascade_iterator` | Read m4 clusters + m6 cost; propose cascade variants for human evaluation (CI bars per F2) | ~200 |
| **m21** | `battern_iterator` | Read m5 + m6; propose battern variants minimizing wallclock OR token cost | ~200 |
| **m22** | `prompt_pattern_iterator` | Read m6 + m7; propose prompt-template variants with measured cost/outcome ratios | ~200 |
| **m23** | `workflow_proposer` | Aggregate m20-m22; gradient-preserve (N near-miss variants alongside canonical); deviation-as-evidence capture | ~250 |
| **m30** | `workflow_bank` | Curated bank; `AcceptedWorkflow { id, lineage, sunset_at, ember_state, escape_surface_profile }` (Cipher constraint) | ~200 |
| **m31** | `selector` | Diversity-enforced selection; n-gram similarity over sequences (Command-3 librarian shape); RALPH fitness-weighted decay applied | ~300 |
| **m32** | `dispatcher` | Resolve workflow → step list → HABITAT-CONDUCTOR enforcement; trap-surfacing before each step (Operator); escape-surface display (Cipher); NEVER executes directly (P0 #3) | ~250 |
| **m33** | `workflow_verifier` | `workflow verify <name>` dry-run/sandbox; reports PASS/FAIL/DEGRADED; `last_verified_at` TTL gates re-runs (Command-3 P0 #9) — **was missing from v1.2 module-allocation; added in single-phase** | ~200 |
| **m40** | `nexus_event_emitter` | Emit typed `WorkflowEvent` (promote/run/decay) to `:8092/v3/nexus/push`; SYNTHEX v2 reinforces/LTDs via Hebbian | ~150 |
| **m41** | `lcm_rpc_client` | Route deploy-shaped workflows through LCM 9-RPC supervisor; never re-implement state machine | ~200 |
| **m42** | `hebbian_feedback` | `POST /reinforce` to POVM with `workflow_trace_*` namespace (AP30 collision avoidance) | ~100 |

**Total: 26 modules · ~3,910 LOC (modules) + ~1,300 LOC (tests at 50 per module minimum) ≈ ~5,200 LOC total.**

Two-binary split per Architect:
- **`wf-crystallise`** owns m1-m23, m40-m42 (read-heavy + iteration + substrate feedback)
- **`wf-dispatch`** owns m30-m33 (bank + selection + dispatch + verification)
- Shared **`workflow-core`** library carries types, schemas, namespace constants

---

## 8 Synergy Clusters (intra-cluster grouping)

### Cluster A — SUBSTRATE INGEST
**Modules:** m1, m2, m3 — detailed spec: [[cluster-A-substrate-ingest]]
**Internal synergy:** Each reads a different substrate; no internal coupling. m2's narrowed scope-registration also serves as trust signal feeding Cluster D.

### Cluster B — HABITAT OBSERVATION
**Modules:** m4, m5, m6 — detailed spec: [[cluster-B-habitat-observers]]
**Internal synergy:** m4 cascade + m6 cost coupled via session-id range on m7's central table. m5 battern + m6 cost coupled via battern-id range. All three emit opaque cluster IDs only (F11).

### Cluster C — CENTRAL CORRELATION + OUTPUT
**Modules:** m7, m12, m13 — detailed spec: [[cluster-C-correlation-output]]
**Internal synergy:** m7 is the hub; m12 reads m7 for human CLI reports; m13 writes m7 rows to stcortex with Hebbian LTP/LTD backpressure check.

### Cluster D — TRUST (cross-cutting, woven through all)
**Modules:** m8, m9, m10, m11 — detailed spec: [[cluster-D-trust-cross-cutting]]
**Internal synergy:** Compile/write/output/lifecycle invariant enforcement. m8 owns compile-time; m9 owns write-time; m10 owns output-time; m11 owns lifecycle.

### Cluster E — EVIDENCE + PRESSURE
**Modules:** m14, m15 — detailed spec: [[cluster-E-evidence-pressure]]
**Internal synergy:** m14 aggregates evidence from m7 (engine-wide habitat-outcome-lift); m15 logs pressure events from anywhere. Together inform m11 decay and m31 selection weights.

### Cluster F — ITERATION
**Modules:** m20, m21, m22, m23 — detailed spec: [[cluster-F-iteration]]
**Internal synergy:** m20-m22 are domain-specific iterators (cascade/battern/prompt); m23 aggregates all three with gradient-preservation (N near-miss variants alongside canonical) and deviation-as-evidence.

### Cluster G — BANK + SELECT + DISPATCH + VERIFY
**Modules:** m30, m31, m32, m33 — detailed spec: [[cluster-G-bank-select-dispatch-verify]]
**Internal synergy:** m23 → m30 (proposals enter bank after human review) → m31 selects → m33 verifies (TTL-gated) → m32 dispatches via Conductor. m33 also feeds m31 verification-state for selection weighting.

### Cluster H — SUBSTRATE FEEDBACK
**Modules:** m40, m41, m42 — detailed spec: [[cluster-H-substrate-feedback]]
**Internal synergy:** m32 dispatch events fan out: m40 to SYNTHEX, m41 to LCM (deploy-shaped only), m42 to POVM Hebbian reinforce. Output of substrate learning loops back to m31's selection weights via stcortex pathway reads.

---

> All cluster specs catalogued in [[MODULE_SPECS_INDEX]] (8 cluster specs / 26 modules / ~41,500 words).

---

## 7 Cross-Cluster Synergies

| # | Synergy | Path | What it enables |
|---|---|---|---|
| **CC-1** | Cascade-Cost Coupling | B internal (m4 ↔ m6 via m7) | Cascade clusters carry observed cost distributions without modules knowing about each other; the join schema is the stable contract |
| **CC-2** | Trust Layer Woven | D → all | m8/m9/m10/m11 are *aspects* — every module routes through them at compile/write/output/lifecycle time |
| **CC-3** | Evidence-Driven Iteration | E → F (m14 → m20-m22) | Habitat-outcome-lift metric informs which iterator focuses where; iterators only propose where evidence supports |
| **CC-4** | Proposal → Bank → Dispatch Pipeline | F → G → Conductor | Closes through *existing* habitat coordinator (no parallel control plane) |
| **CC-5** | Substrate Learning Loop | G → H → back to F | m40/m41/m42 outputs reinforce/LTD pathway weights → m31 reads → m31 selection updates → m20-m22 inputs shift over time |
| **CC-6** | Verification-Gated Dispatch | G internal (m33 → m32) | Stale workflows blocked from careless re-invocation (Operator P0 #7) |
| **CC-7** | Pressure-Driven Evolution | E → spec interviews | m15 reservation register surfaces in agent-cross-talk; Watcher + Zen observe scope-pressure → informs future spec amendments |

---

## Flagged Features / Capacities — Verification Matrix (30 rows)

| # | Feature flagged at | Module(s) | Status |
|---|---|---|---|
| 1 | W1 narrowed-scope consumer | m2 + m9 | ✅ owned |
| 2 | W2 CR-2 hard build prereq | m8 | ✅ owned |
| 3 | W3 Ember 7-trait CI gate (Held fails) | m10 (consumes Watcher's vault rubric) | ✅ owned, **gated on Watcher §5.1 Held-semantics amendment** |
| 4 | F8 Watcher feedback-loop poisoning | m2 + m9 + Observer read-deny | ✅ owned |
| 5 | F9 workflow-grain fitness distortion | m7 (zero-weight column) | ✅ owned |
| 6 | F10 exploration-cost preservation | m6 (baseline) | ✅ owned |
| 7 | F11 cascade monoculture | m4 (opaque IDs) + m31 (diversity) | ✅ owned |
| 8 | F2 sample-size n≥20 + CI bars | G5 spec gate + m14 + m20-m22 enforce at runtime | ✅ owned (spec gate + runtime) |
| 9 | P0 #3 Conductor mandatory dispatch | m32 | ✅ owned |
| 10 | P0 #15 Conductor Wave maturity | m32 dependency on Wave 1B/1C/2/3 | ⚠ **DEPENDS** on Conductor `auto_start=true` flip (currently false) |
| 11 | P0 #11 escape-surface declaration | m30 schema + m32 display | ✅ owned |
| 12 | P0 #8 RALPH fitness-weighted decay | m11 + m31 weighting | ✅ owned |
| 13 | P0 #9 workflow verify verb + TTL | m33 | ✅ owned (added module — **was missing from v1.2 allocation**) |
| 14 | P0 #10 deviation captured as evidence | m23 internal | ✅ owned |
| 15 | Architect two-binary split | wf-crystallise + wf-dispatch + workflow-core lib | ✅ owned (deployment structure) |
| 16 | Substrate Hebbian LTP/LTD bounded | m13 backpressure check | ✅ owned |
| 17 | NA-Gap gradient preservation (N near-misses) | m23 internal | ✅ owned |
| 18 | Fossil archaeological framing | m4 + m23 + m30 (opaque IDs throughout) | ✅ owned |
| 19 | Operator trap surfacing per step | m32 display-before-step | ✅ owned |
| 20 | Operator crystalliser-consultation-before-dispatch | m31 → m33 → m32 chain | ✅ owned |
| 21 | SYNTHEX `WorkflowEvent` emission | m40 | ✅ owned |
| 22 | LCM RPC routing for deploy workflows | m41 | ✅ owned |
| 23 | POVM AP30 namespace prefix | m42 | ✅ owned |
| 24 | AP24 explicit Luke signal | G9 gate | ✅ owned |
| 25 | AP27 no self-modification of m46-m51 | hard refusal (no module) | ✅ owned |
| 26 | Ember 7-trait unanimity gate | m10 + Watcher's vault rubric | ✅ owned |
| 27 | PBFT n=40/q=27 governance | external (Watcher m46/m49 substrate) | ✅ dependency satisfied |
| 28 | Watcher synchronous-participant slot in spec circle | G5 (gate) | ✅ owned |
| 29 | Zen synchronous-audit slot in spec circle | G5 + G7 (gates) | ✅ owned |
| 30 | Phase-B reservation observability | m15 → agent-cross-talk emit | ✅ owned (semantics shift: in single-phase, "Phase-B reservation" becomes "forbidden-verb-pressure register" — still useful telemetry) |

**Coverage: 28/30 ✅ owned · 2/30 ⚠ dependency**

The two dependencies:
- **#10 Conductor Wave maturity** — m32 cannot ship working until Conductor 1B/1C/2/3 are `auto_start=true` (Luke's terminal-bring-up gated)
- **#3 Ember Held semantics** — m10 cannot consume Watcher's vault rubric until Watcher amends §5.1 per Zen audit `2026-05-16T224523Z`

Both are KNOWN gates, not coverage gaps.

---

## What changed from phased → single-phase

| Item | Before (phased) | After (single-phase) |
|---|---|---|
| Module count | 25 + Phase C TBD | **26** (added m33 workflow_verifier missing from v1.2 allocation; dropped Phase C TBD placeholder) |
| LOC est. | Phase A ~1,750 / Phase B ~3,500 / Phase C TBD | **~5,200 single deployment** |
| Gates G1-G9 | Required | **Required (unchanged)** |
| Phase B activation gate | Sunset PASS + Watcher + Zen + Luke | **REMOVED** |
| m11 semantics | Phase B activation startup-refusal at D120 | Engine-wide sunset law + RALPH fitness-weighted decay for bank |
| m14 semantics | Phase B activation evidence aggregator | Engine-wide evidence aggregator (continuous) |
| m15 semantics | Phase B reservation register | Forbidden-verb-pressure register (informational, not gating) |
| Fossil-rhyme risk | Mitigated by phasing | **Carried on Command head** — explicit waiver |
| Skeptic pain-source gate | Pre-build hard gate | **Waived** — building without empirical pain confirmation |

---

## Standing posture

- **Override absorbed.** Single-phase deployment is now the spec direction.
- **v1.3 patch to v1.2 spec needed** to reflect: 26 modules (not 11), no phase boundaries, m33 added, m11+m14+m15 semantics shifts, explicit waiver record of Fossil/Skeptic/Watcher-R6/RALPH-safety concerns.
- **G7 Zen spec audit will need to re-audit** the v1.3 patch — substantive direction change.
- **Watcher + Zen need explicit notice** of the override and waivers.
- **HOLD-v2 still active** until G1-G9 fire. Module-listing is planning surface; doesn't unlock build.

---

## Cross-references

- Canonical version in `~/claude-code-workspace/the-workflow-engine/` (when v1.3 patch filed)
- Prior planning: [[Circle of Experts S1001982]] · [[Town Hall S1001982]] · [[Boilerplate Hunt S1001982]] · [[Genesis Prompt v1.2 S1001982]] · [[Module Structure S1001982]]
- HOME: [[HOME]]
