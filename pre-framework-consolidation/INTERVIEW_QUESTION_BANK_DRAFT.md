---
title: The Workflow Engine — Genesis Interview Question Bank (DRAFT)
date: 2026-05-17 (S1001982 / S1001971)
authority: Luke @ node 0.A (presides) · Command + Command-3 (interlocutors)
kind: PRE-STAGED-INTERVIEW
status: DRAFT — not run until Luke green-lights pain-source verification (Action 1) + CR-2 ruling (Action 2)
priors:
  - the-workflow-engine/THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982.md  (Command pre-build step 2: "Structured genesis interview — 3 rounds, ≤12 questions")
  - the-workflow-engine/CONVERGENCE_COMMAND_X_COMMAND3_S1001982.md  (peer synthesis Action 4)
  - feedback_structured_interview_before_code.md  (S117 crystallisation: AskUserQuestion 4-q-per-round, sequential, persist 5 surfaces)
back_to: CLAUDE.md · CLAUDE.local.md · the-workflow-engine/
---

# Genesis Interview Question Bank (DRAFT)

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[workflow-engine-code-base]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]
>
> Mirror: [[Interview Question Bank Draft S1001982]]

Pre-staged so Luke can trigger the genesis interview in one move once **Action 1** (Skeptic pain-source verification, ≥3 hits criterion) passes and **Action 2** (CR-2 ruling) resolves. Total budget: **3 rounds, 12 questions**, per Command town-hall pre-build sequence. Sequential rounds (each round's answers inform the next). All rounds before any code.

Each question is `AskUserQuestion`-compatible: a question + 2-4 options + brief descriptions. Luke can always pick "Other" with custom text. Recommendation is the first option, suffixed "(Recommended)".

## Round 1 — Foundation (load-bearing structural decisions)

These four questions determine what the engine fundamentally *is*. They must be answered before Round 2 has meaning. Recommended ordering: Q1 → Q2 → Q3 → Q4.

### Q1.1 — Naming and framing
**Header:** Name + frame
**Question:** Which internal name + framing for the project?
**Options:**
- `workflow-cartographer` (Recommended) — archaeological framing (Fossil's town-hall ask); signals "map what is there," not "control what runs"
- `workflow-librarian` — Command-3's town-hall framing; signals "curate and propose"
- `workflow-archaeologist` — Fossil's first alternative; signals "dig out what's already there"
- `the-workflow-engine` — original framing; signals control-plane authority

### Q1.2 — Binary topology
**Header:** Binaries
**Question:** Should crystalliser and dispatcher be separate binaries with a shared library, or one binary with internal modules?
**Options:**
- Two binaries + library (Recommended) — Architect's town-hall position; protects against single-failure-mode-conflation; ~3 build artifacts
- One binary, internal modules — Command-3 Wave 2 default; smaller deployment surface; one health endpoint
- One binary now, split later — defer the decision until v0.5 if pressure emerges

### Q1.3 — Conductor integration timing
**Header:** Conductor gate
**Question:** Build cannot dispatch correctly until Conductor Waves 1B/1C/2/3 are live. Posture during the gap?
**Options:**
- Wait (Recommended; town-hall consensus) — engine planning continues, build holds until Conductor stabilises; aligns with P0 #15
- Plan + scaffold; defer wiring — author code + tests but ship gated by `CONDUCTOR_ENFORCEMENT_ENABLED` startup-refusal
- Bypass Conductor (NOT recommended) — engine dispatches directly; violates P0 #3; resurrects parallel-control-plane risk

### Q1.4 — Decay law shape
**Header:** Decay formula
**Question:** Workflow decay computed as?
**Options:**
- `frequency × fitness × recency × co-activation` (Recommended) — composite signal; integrates RALPH's fitness ask (P0 #8) with Command-3's Hebbian co-activation (Wave 2 F3)
- `frequency × fitness × recency` — RALPH's town-hall position, no co-activation term
- `frequency × recency` only — Command-3 Wave 2 default; usage + age; no fitness or co-activation
- `co-activation × recency` — Hebbian-only, no fitness (defers fitness to crystalliser-consultation gate)

---

## Round 2 — Substrate boundaries (read/write scope, trust profile)

Answered after Round 1 has settled name + topology + Conductor posture + decay shape. These four questions fix the substrate trust boundary and prevent the "first-meta-consumer" surveillance risk (Substrate's town-hall concern).

### Q2.1 — stcortex consumer scope
**Header:** Consumer scope
**Question:** What stcortex tables can the workflow engine consume?
**Options:**
- `tool_call` + `consumption` only (Recommended; Substrate's P0 #2) — narrowest scope; no read of `memory` or `pathway` writes from other consumers; surveillance-risk minimised
- Add `memory` (read-only) — engine sees other consumers' memories; useful for cross-domain pattern detection but enormous read amplification
- Add `pathway` (read-only) — engine sees co-activation graphs; useful for Hebbian-aware crystallisation but more surveillance surface

### Q2.2 — Write surface boundary
**Header:** Write scope
**Question:** Where can the engine write?
**Options:**
- Own `workflow_state.db` + own stcortex `workflow_engine_*` namespace (Recommended; town-hall consensus) — no other DB, no other namespace
- Above + atuin KV (write `workflow.last_state.*` keys) — needed if engine needs to inject state into other Claude instances on session-start
- Above + injection.db `causal_chain` table — needed if engine wants to register itself as a habitat-injection source

### Q2.3 — Stale-workflow gate threshold
**Header:** Stale threshold
**Question:** Operator's P0 #7 ("crystalliser-consultation before every dispatch") — what triggers the stale gate?
**Options:**
- Substrate evidence decayed below threshold in last 60 days (Recommended; Operator's town-hall ask) — concrete, time-bounded
- N consecutive failed runs (e.g., 3) — outcome-based; reactive not predictive
- Manual `workflow refresh <name>` only — operator-driven; lowest engine-authority
- Adaptive: function of decay law output — engine-driven; ties to Q1.4 decision

### Q2.4 — Sunset clause trigger
**Header:** Sunset trigger
**Question:** Six-month sunset auto-disable condition. Which is the binding trigger?
**Options:**
- ALL of: selector accuracy < 60% AND no Hebbian gain vs baseline AND Conductor-violation > 5% (Recommended) — three-axis with AND; auto-disable only on systemic failure
- ANY of the three — auto-disable on any single failure; stricter
- Selector accuracy alone — simpler; might miss Hebbian-stagnation case
- Hebbian gain alone — pure substrate signal; ignores user-facing accuracy

---

## Round 3 — Crystalliser internals + first-workflow choice (decided last)

Answered after Round 1 + 2 are settled. These four questions fix the crystalliser's core algorithm + which workflow gets crystallised first. Round-3 decisions are tweakable post-v0; Rounds 1 + 2 are not.

### Q3.1 — Sub-graph detection algorithm
**Header:** Detection algo
**Question:** Compositional sub-graph detection — which algorithm class?
**Options:**
- Frequent-subsequence mining on atuin command rows + stcortex tool-call graph (Recommended) — well-understood, polynomial; produces `(commands, frequency, fitness)` tuples; matches the substrate's actual shape
- Graph motif discovery on full POVM/stcortex pathway graph — richer signal but expensive; might exceed v0 scope
- LLM-based pattern extraction (offline batch) — calls Claude to label compositional patterns; high quality but expensive + introduces self-training risk
- Hand-curated seed set + Hebbian reinforcement — operator-seeded, engine reinforces; bootstraps with zero detection, depends on operator effort

### Q3.2 — First canonical workflow to crystallise
**Header:** First workflow
**Question:** Which workflow gets crystallised first as the canonical v0 acceptance?
**Options:**
- `harden-coverage-tier` (Recommended; Command-3 Wave 2 + Zen circle pick) — sharpest shape, fewest divergent observations across Sessions 117/122/1001883; smallest first target
- `genesis-microservice` (Luke's original example) — most-observed but most-divergent across S103/S117/S122; high value if it works but high risk of overfitting to first 3 runs
- `pre-deploy-hardening` (4-agent gate) — narrow, already-named pattern from the `pre-deploy-hardening` skill; should be quick to verify
- Skeptic's choice — wait for the pain-source search results in Action 1 to tell us which workflow Luke has *actually* expressed the need for

### Q3.3 — Gradient-preservation N (near-miss variants alongside canonical)
**Header:** Gradient N
**Question:** NA Gap Analyst's P0 #5 — engine must surface N near-miss variants alongside each canonical workflow. What N?
**Options:**
- N = 3 (Recommended) — Three variants ensures genuine diversity without overwhelming the operator; matches Luke's 3 decisions/week budget on Master Plan v2
- N = 2 — Minimum viable diversity; fewer choices, faster decision
- N = 5 — More entropy preservation but harder for operator to scan
- Adaptive (function of crystalliser confidence) — engine picks N based on how dominant the canonical is; complex but principled

### Q3.4 — Verification TTL
**Header:** Verify TTL
**Question:** Command-3's P0 #9 — `workflow verify <name>` TTL. After how long must a workflow be re-verified before re-running?
**Options:**
- 30 days (Recommended) — aligns with the SYNTHEX v2 m49 R13 calendar arm + matches typical habitat session cadence
- 60 days — Operator's stale-gate window (Q2.3); paired with that for consistency
- 14 days — tighter; better for fast-evolving codebases; more verification overhead
- N runs since last verify (e.g., 10) — usage-based not time-based; orthogonal axis

---

## Post-interview gates

After all 3 rounds answered:

1. **Persist the 12 answers** to `the-workflow-engine/INTERVIEW_LOG_S1001982.md`.
2. **Author the design doc** `the-workflow-engine/WORKFLOW_ENGINE_PLAN_V0.md` encoding the 15 P0 constraints + 12 interview answers.
3. **Run conventional gap analysis** (standard reviewer would flag).
4. **Run NA gap analysis** (second frame: what does the substrate-amplifier pass reveal that the control-plane pass missed?).
5. **Four-surface persist** the sealed plan (this convergence doc + town-hall + boilerplate-hunt = 3 of 4 already).
6. **Eat-the-dogfood gate:** the genesis of the engine itself follows the 6-step shape it's being built to encode.
7. **`start coding workflow-cartographer`** (or whichever name lands) — Command-3 owns build leadership per Q-OWNER claim; Command remains synthesis lead.

## Discipline notes

- **AP24 (no code before sealed spec):** if Luke types `start coding` before all 12 answers + both gap analyses + four-surface persist, both Command and Command-3 will refuse and surface the AP24 violation. On-record in CONVERGENCE doc §5.
- **Sequential rounds matter:** S117 crystallisation showed Round 2 Q2.2 → Round 6 Q6.1 mid-flight reversal. Don't batch all 12; run Round 1 → integrate answers → Round 2 → integrate → Round 3.
- **No "Other" overuse:** Luke can always pick Other, but if Round 1 sees ≥2 Others the interview is mis-scoped and needs Round 0 reset.
- **Time budget:** ~15 min per round at AskUserQuestion cadence. Total interview ~45 min wall-clock. Schedule when Luke has a full clear hour.

— Command-3 (Tab 1 Orchestrator middle-right), Action 4 of CONVERGENCE plan, 2026-05-17T08:05:00+10:00. Ready to run once Actions 1 + 2 clear.

---

## Related

- [[THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982]] — pre-build step 2 source ("Structured genesis interview — 3 rounds, ≤12 questions")
- [[CONVERGENCE_COMMAND_X_COMMAND3_S1001982]] — peer synthesis Action 4 (interview bank pre-staging)
- [[GENESIS_PROMPT_V0]] — 5-voice prompt this bank backs
- [[THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982]] — v1.2 binding spec; G5 interview = this bank
- [[THE_WORKFLOW_ENGINE_MODULE_STRUCTURE_S1001982]] — architecture sketch the interview answers will constrain
- [[Modules Synergy Clusters and Feature Verification S1001982]] — current single-phase architecture
