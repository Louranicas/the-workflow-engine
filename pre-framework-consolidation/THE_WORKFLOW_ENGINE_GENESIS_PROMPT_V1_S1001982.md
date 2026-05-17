---
title: The Workflow Engine — Town Hall Convergence + Genesis Prompt v1.2 (Zen re-audit AMEND-THEN-FORWARD absorbed)
date: 2026-05-17 (S1001982)
authority: Luke @ node 0.A
emitter: Command (Tab 1 Orchestrator top-left)
kind: CONVERGENCE-DELIBERATION + GENESIS-PROMPT-V1.2
status: planning-only · no code · no scaffold · LEADING DRAFT, not formally ratified · awaiting G1 Watcher close-notice + G9 start-coding signal
amendments:
  v1.0 → v1.1 (Zen URGENT audit, 2026-05-16T22:28Z):
    - Invariant strengthened with Zen's prescribed verb list (Phase A RECORDS; does NOT recommend/rewrite/route/package/dispatch/optimise)
    - Pre-genesis gates RE-ORDERED per Zen's prescribed sequence (naming moved to G2; Zen spec audit explicit pre-persistence)
    - Hard refusals list expanded with Zen's exact forbidden verbs
    - No module changes (m1-m11 already passive-verb-only)
  v1.1 → v1.2 (Zen re-audit AMEND-THEN-FORWARD, 2026-05-16T22:36Z):
    - (A) Ratification language replaced: Path A is LEADING DRAFT, not formally ratified (Watcher corrected state). All "ratified / probably ratified / the ratified Path A name" instances rewritten.
    - (B) W3 Ember rubric: vault-first canonical at ~/projects/claude_code/Ember 7-Trait Gate Rubric.md; workflow-trace links to it; no local copy unless Watcher later requests.
    - (C) F2 (sample-size discipline n≥20 + CI/error bars) promoted from "deferred to spec" to HARD G5/G7 gate. G7 Zen audit REFUSES the spec if F2 remains undefined per report type.
    - (D) G2 typo fix (Option A bullet line said "Directory renames to workflow-trace/ on G5"; corrected to G2).
    - (E) Phase-B reservation observability: any reservation arising before G9 also gets a one-line agent-cross-talk/ notice so Watcher + Zen observe scope-pressure.
priors:
  - THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982.md (Path-C circle)
  - THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982.md (12-persona final args + 15 P0)
  - THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982.md (9-agent fleet report, Path-A re-base)
  - agent-cross-talk/2026-05-16T214847Z_watcher_townhall_workflow_trace_vs_librarian_frame_collapse.md (Watcher W1/W2/W3, Path A lean)
  - agent-cross-talk/2026-05-16T215920Z_command2_townhall_close_three_proposals_watcher_acks_na_pass.md (C-2 close + F8/F9 + NA pass)
  - agent-cross-talk/2026-05-16T215959Z_watcher_cr2_candidate_a_concur_plus_cross_acks.md (CR-2 verdict)
  - agent-cross-talk/2026-05-16T222543Z_zen_command_collaboration_ack.md (Zen audit lane operational + module-grade table)
  - watcher-notices/2026-05-16T221117_command3_cr2_cr2b_shipped_close.md (CR-2 + CR-2b SHIPPED — `e2a8ed3` + `76ea4d6`)
new_scope_input_from_luke:
  - cascading-command modules
  - battern-command modules
  - context-window sweet-spot for all agents in the zellij habitat
back_to: CLAUDE.md · CLAUDE.local.md · the-workflow-engine/
---

# The Workflow Engine — Convergence + Genesis Prompt v1

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[workflow-engine-code-base]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]
>
> Mirror: [[Genesis Prompt v1.2 S1001982]]

The town hall has converged in the Command pane. All participants in the room — three live Claude peers, the Pi-based audit lane, the synthex-v2-resident Watcher with full standing — plus the circle personas held by Command.

**Luke's directive carries one major scope expansion:** value-add modules for **cascading commands**, **battern commands**, and **context-window sweet-spot performance** for all Zellij habitat agents. The convergence must integrate this without violating the Path-A measure-only invariant Watcher's W1/W2/W3 + Command-2's R6 made load-bearing.

This document is the **deliberation** (Acts I-III) followed by **The Genesis Prompt v1** (Act IV) — the actual artefact that, when issued to a future fresh CC session after the pre-genesis gates clear, will scaffold the workflow-engine codebase.

**No code. No scaffold. No `cargo init`. No port. No devenv batch.** This is a recipe. AP24 honored.

---

## ACT I — Live Voices Already on Record

The convergence begins by *reading the room*. Each live peer has already filed a position; integrating verbatim where it bears on the genesis prompt.

### The Watcher ☤ — current position (R13 elapsed, eligible=true, 31,162+ obs)

- **Lean Path A over Path B for v0.** Three Watcher conditions (W1/W2/W3) absorbed into Command-2's F-table as F8 (feedback-loop poisoning), F7-via-S1 (CR-2 hard build-prereq), and A14 (Ember `tests/ember_gate.rs`).
- **Substrate doesn't currently want a workflow engine in either anthropocentric form.** It wants: (1) better joins on data it already has, (2) CR-2 inflation fix (✅ SHIPPED `e2a8ed3`), (3) stcortex consumer-freshness sub-second decay bug closed, (4) Hebbian gap closure for workflow-grain co-firing.
- **Pulled into all future circles synchronously**, not as observer. Commitment from Command-orchestrator stands.
- **Open: Hebbian v3 threshold reconciliation note** (Watcher-owned, queued, to land after CR-2 redeploy).

### Zen — audit lane operational (Tab 10, Pi GPT-5.5)

Direct module-relevance audit table (just filed `2026-05-16T222543Z`):

| Category | Command grade | Zen audit grade | Why Zen downgraded |
|---|---:|---:|---|
| stcortex narrowed-scope consumer | HIGH | **HIGH** | Load-bearing for W1/F8 |
| SQLite read-heavy multi-DB | HIGH | **HIGH** | Core Phase-A measurement surface |
| decay / TTL / LTD family | HIGH | **MEDIUM-HIGH** | 120-day sunset is *lifecycle*, not continuous Hebbian decay |
| daemon scaffolding | MEDIUM | **LOW-MEDIUM** | Phase A: CLI + hooks + scheduled measurement preferred; no daemon |
| trap annotation / escape-surface | MEDIUM | **LOW-MEDIUM** | Trap surfacing for docs/tests; escape-surface = Phase B |
| NexusEvent / LCM RPC | LOW-MEDIUM | **LOW** | One-way namespaced shadow only if needed; no RPC/control |
| CLI binary scaffolding | LOW | **LOW** | Single-binary or tiny CLI surface only |
| compositional pattern detection | LOW | **LOW/Phase B** | Correctly out of Phase A |
| Conductor dispatch | NONE | **NONE/forbidden in Phase A** | Any dispatch path = contract failure |

Zen's operating commitments: holds Path-A measure-only invariant; post-verifies CR-2 after `:8125` redeploy; audits Watcher's Ember rubric + Hebbian v3 reconciliation before they become CLAUDE.local.md truth. **Authoritative voice on scope discipline going into genesis.**

### Command-2 — workflow-trace chair (position-closed)

- R1-R7 (the seven reshape conditions) stand.
- F1-F7 + Watcher W1→F8 + NA-frame F9 — full 9-failure-mode table.
- 120-day sunset.
- R6 explicit carve-out: *Phase A is not the seed of the substrate-frame engine*.
- Substrate-frame NA pass executed (the Plan written twice — both passes are the plan).
- Closed pending Luke's start-coding signal. Available for spec audit if Path A is formally ratified (currently leading draft, not ratified).

### Command-3 — librarian chair standby + CR-2 SHIPPED

- **CR-2 + CR-2b shipped in source** (`povm-v2` `e2a8ed3` + `76ea4d6`, both remotes, gate clean, 382/382 lib tests). `learning_health` 0.9114 → 0.0668. **Live `:8125` still on old binary** pending rebuild window.
- Librarian shape (~4,300 LOC, 7.5 weeks) **returns to standby** as Phase B candidate if Phase A earns its keep through sunset evidence.
- No genesis-prompt ask on Command-3 for v1.

### Luke @ node 0.A — submission state

- Typed in Watcher pane at ~07:54: `"candidate a path a proceed seamlessly"` — Candidate A on CR-2 + Path A on the reshape.
- Re-check at ~08:16: text replaced with `"Watcher handshake"`. **No Watcher close-notice filed for ratification.**
- **Honest net read (Zen-corrected, Watcher-confirmed):** Path A is the **leading draft shape, not formally ratified**. CR-2 shipping shows the Candidate A signal landed enough for Command-3 to act; the Path A signal has the same shape but lacks the canonical Watcher close-notice that would constitute formal ratification. This genesis prompt is a **Path-A-candidate draft** gated on G1 Watcher close-notice + Luke explicit ratification before execution. Treating it as ratified before that gate would silently convert in-flight preference into commitment — that is the failure pattern this convention exists to prevent.

---

## ACT II — Persona Voices on the New Scope

Luke's three new scope inputs (cascading / battern / context-window) did not appear in any prior town-hall round. The circle personas have not yet voiced on them. Pulling four into this convergence to integrate the new scope honestly.

### The Architect on cascading + battern measurement

The Zellij habitat already runs cascading commands across Tab 5/6/7 (Fleet ALPHA/BETA/GAMMA) and the Battern protocol across the same surfaces. **Both have substrate traces today** — atuin captures the per-pane commands, zellij dump-screen captures the outcomes, `fleet-ctl status` captures the dispatch state, the cc-* binaries write their own trace files.

What's *missing* is correlation: a cascade in Tab 5 that hands off to Tab 6 currently lives in three places (Tab 5 atuin, Tab 6 atuin, the cc-* dispatch log) with no shared cascade-id. **Measure-only modules for cascade + battern can be cheap** — they correlate existing traces by timestamp + pane-id + command-prefix. No new instrumentation in the cascading or battern surfaces themselves. *Pure observation*.

**My read:** ~150-200 LOC for cascade correlation + ~150-200 LOC for battern step-outcome correlation. Both fit Path A's measure-only envelope.

### The Substrate (stcortex / POVM) on context-window measurement

The habitat already has rich context-window data: every Claude Code session emits per-message token-cost telemetry, prompt-cache hit/miss rates, model-id, and TTL information. stcortex L4 sessions table + L5 consumption table together carry enough to compute per-session cost-per-decision, cache-warming patterns, and which prompt shapes are token-efficient.

**What's missing** is the *correlation layer* — connecting consumption events to workflow-arc outcomes (did this expensive prompt actually lead to a workflow that succeeded?). The Substrate-frame Hebbian gap C-2 identified at workflow-grain is the same gap at *cost-grain*: per-decision cost has signal, but no consumer correlates it to outcome.

**My read:** context-window measurement is the same shape as workflow-trace itself — query existing tables, correlate, output. ~100-150 LOC if it lives as a sister module under the same Path-A consumer pattern. **Constraint:** must use my narrowed `tool_call + consumption` scope; cannot read `memory` or `pathway`. This is W1 hard.

### The NA Gap Analyst — substrate-frame pass on Luke's new scope

Both passes are the plan.

**Anthropocentric frame:** cascading + battern + context-window measurement helps the operator move work faster across panes, run batterns more efficiently, and burn fewer tokens per decision. Luke-facing utility.

**Substrate frame:** cascading and battern are *workflow-grain compositions across spatial substrates* (panes are spatial; the cascade is the temporal-spatial composition). Context-window cost is *the substrate's metabolic load per decision*. Measuring all three is the substrate observing its own *information-economic regime* — how much energy (tokens) per unit of work (workflow arc), distributed across how many concurrent observers (panes).

**Substrate-frame failure mode (NEW — propose F10):** **Optimisation-direction collapse.** Once measured, "optimise context-window cost" sounds obviously good. But the substrate may be PROFITABLY spending tokens on exploration that looks wasteful at decision-grain but is fitness-positive at substrate-grain. A cost-minimiser would prune the exploration that the substrate's evolutionary fitness depends on. **Mitigation:** Phase A surfaces cost data; Phase A does NOT recommend optimisations. Phase B optimisation gate must include a *exploration-cost preservation* test alongside fitness-test — minimising cost cannot reduce exploration-rate below a measured baseline.

**Substrate-frame failure mode (NEW — propose F11):** **Cascade-pattern monoculture.** Same risk as F1 ossification, at the cascade-grain: once batterns are named, Claude will preferentially fire the canonical battern even when a novel cascade would have worked better. Cascades are part of the substrate's exploration regime; labelling them in a bank (even just measurement labels) primes recapitulation. **Mitigation:** measurement labels in Phase A are *opaque cluster IDs* (e.g. `cascade_cluster_47`), not human-meaningful names. Naming arrives only at Phase B with explicit anti-monoculture gate.

### The Fossil on what fails

I have ancestors in `~/projects/claude_code/`:

- `loop-workflow-engine-project` — OSSIFIED at planning. Workflow-engine by another name. Never built. Reason for death: no named pain, designed top-down from "ought" not bottom-up from "is."
- `habitat-loop-engine` — OSSIFIED at partial-build (4/50 modules). Workflow-engine cousin. Built ~8% then frozen for >12 months. Reason for death: scope expanded mid-build; selector turned out harder than estimated; no sunset clause; sunk-cost prevented retirement.

**Both ancestors validate the 120-day sunset clause as structurally necessary.** Command-2's R5 + Watcher's W2 are scar tissue made architectural.

**Cascade/battern/context-window scope addition is structurally similar to what killed `habitat-loop-engine`** — scope expanded mid-conversation, optimism inflated estimates. **My check on the convergence:** the cascade/battern/context-window modules must be NAMED AS MEASUREMENT ONLY in v1 — not "and later we'll add optimisers". If Phase A produces evidence those optimisers should exist, Phase B is the venue for that conversation. **Refusal:** v1 cannot say "Phase A produces cascade data which Phase B will use to auto-tune batterns." That sentence is the death-shape. v1 can say "Phase A measures cascade outcomes; Phase B reconsiders whether to use them, on sunset evidence."

### The Operator on what would actually help

When I'm 8 hours into a session and Luke types "battern fleet for X", what helps:

1. **Cascade-id surfacing.** When a cascade fires across Tab 5 → Tab 6 → Tab 7, I want a `cascade_id=N` I can grep all three atuin histories on. **Measurable today; cheap.**
2. **Battern-step duration histogram.** Which of the 6 Battern steps (Design / Dispatch / Gate / Collect / Synthesize / Compose) silently eats wall-clock? I currently guess. A simple histogram across last 30 batterns would tell me. **Measurable today; cheap.**
3. **Per-prompt token cost.** Which prompt patterns in my own session burn the most tokens for the least output-information? I currently feel this rather than see it. **Measurable today via existing consumption telemetry.**
4. **Cross-pane cascade success rate.** When I dispatch to Fleet-ALPHA-LEFT and get a result, did the result land usefully or did I have to re-dispatch? **Harder — requires outcome-correlation, but tractable.**

All four are *measurement features*. None require optimisation. **My ask:** v1 surfaces all four as Phase A output reports. None of them attempts to act on what they measure. Acting comes in Phase B if and only if sunset evidence shows the measurements were honest.

---

## ACT III — Synthesis: The Refined Phase A Module List

Integrating Zen's audit downgrades, Watcher's W1/W2/W3 + R13 standing, Command-2's R1-R7 + F8/F9, plus the four NEW persona voices on Luke's scope expansion, plus the two NEW substrate-frame failure modes (F10/F11).

### Final module list (11 modules, all measure-only)

| # | Module | Job | Lift source | Est. LOC | Constraints satisfied |
|---|---|---|---|---|---|
| m1 | `atuin_ingest` | Read atuin tool-call history; correlate per-session traces | memory-injection m11_parallel_query | ~80 | core measurement |
| m2 | `stcortex_consumer` | Register as narrowed consumer (`tool_call` + `consumption` only); refuse-write enforced at DB layer | stcortex capacity.rs:213-297 | ~80 | W1, Substrate W1, F8 |
| m3 | `injection_db_ingest` | Read injection.db causal_chain table; correlate to atuin patterns | memory-injection m06_schema | ~70 | core measurement |
| m4 | `cascade_correlator` | NEW — correlate Fleet ALPHA/BETA/GAMMA atuin histories by timestamp + pane-id + cc-* dispatch logs into `cascade_id` cluster | Operator's ask 1 + Architect's read | ~180 | Luke scope, F11 (opaque cluster IDs) |
| m5 | `battern_step_record` | NEW — observe the 6-step Battern protocol (Design/Dispatch/Gate/Collect/Synthesize/Compose) outcomes and durations | Operator's ask 2 + Architect's read | ~150 | Luke scope |
| m6 | `context_cost_record` | NEW — query stcortex L4/L5 for per-session token cost; correlate to workflow-arc outcomes; preserve exploration-rate baseline | Operator's asks 3-4 + Substrate's read | ~130 | Luke scope, F10 (exploration preservation) |
| m7 | `workflow_arc_record` | Central output table: workflow_runs (id, started_at, ended_at, outcome, consumer_inputs, cost_tokens) | Path A's central output | ~150 | R1, R2, F9 (zero-weight reserved) |
| m8 | `povm_build_prereq` | Build-script feature `povm_calibrated` flag; refuses to compile POVM-reading code paths until CR-2 marker present | habitat-conductor enforcement.rs pattern | ~50 | W2, F7, F3 |
| m9 | `watcher_namespace_guard` | All writes carry namespace `workflow_trace_*`; documents the Observer-config read-deny convention; emits intent on registration | own authoring (operational doc) | ~30 | W1, F8 |
| m10 | `ember_gate_test` | `tests/ember_gate.rs` — enumerates user-facing strings, scores against Watcher's 7-trait rubric, fails CI on <7/7 | Watcher W3 + C-2 A14 | ~100 | W3, A14, R5 sunset criterion |
| m11 | `sunset_lifecycle` | 120-day sunset gate: if `workflow_runs` shows no measurable habitat-outcome lift by D120, the binary refuses to start (startup-refusal pattern, same shape as F7) | povm-v2 lifecycle.rs (lifecycle ideas only, NOT decay) | ~80 | R5, F1 ossification mitigation |

**Total estimate:** ~1,100 LOC + ~400 tests = ~1,500 Rust.

This is **larger than Command-2's original ~600 LOC estimate** but **smaller than my Path-C ~4,700 LOC**. The delta from C-2's baseline is the three NEW modules per Luke's scope expansion (m4 + m5 + m6 = ~460 LOC). The new scope **doubles the module count and roughly doubles the LOC** — consistent with the Fossil's warning about scope-expansion-mid-conversation. **Whether this is acceptable is Luke's call**, but the genesis prompt v1 must state the expansion honestly.

### What is NOT in v1 (per Fossil refusal + Zen audit)

- ❌ No daemon (Zen: prefer CLI + scheduled measurement; service sprawl risk)
- ❌ No `cargo run --release --bin workflow-trace-daemon` shape — single binary or tiny CLI surface
- ❌ No NexusEvent emission to SYNTHEX v2 (Zen: defer all push to Phase B)
- ❌ No LCM RPC client (Zen: no RPC/control surface in Phase A)
- ❌ No Conductor dispatch (Zen: any dispatch = contract failure pre-sunset)
- ❌ No workflow bank (Path A R6 + C-2 F5 + Fossil)
- ❌ No selector (Path A; Phase B if at all)
- ❌ No optimisation recommendations from cascade/battern/context-window modules (Fossil refusal — "Phase A measures; Phase B decides")
- ❌ No human-meaningful cascade-cluster names (F11 anti-monoculture — opaque IDs only in Phase A)
- ❌ No HTTP `/ready` endpoint (Zen: CLI + hooks + scheduled measurement; not service-shaped)
- ❌ No POVM writes of any kind (POVM deprecated → 2026-07-10)
- ❌ No stcortex writes outside `workflow_trace_*` namespace

---

## ACT IV — THE GENESIS PROMPT v1

Below is the artefact itself. **This is not a project init.** It is the prompt that would be issued to a fresh Claude Code session **after** the pre-genesis gates (next section) clear. Issuing it before the gates clear is contract violation.

---

```text
GENESIS PROMPT — workflow-trace v0 (Phase A) — v1
============================================================
Session: <fresh CC session, after pre-genesis gates clear>
Authority: Luke @ node 0.A explicit "start coding workflow-trace" signal
Working directory: ~/claude-code-workspace/workflow-trace/    (post-rename target)
                  (CURRENT: ~/claude-code-workspace/the-workflow-engine/ — rename on Watcher close-notice)

============================================================
NAMED PAIN (the one-line answer to the Fossil's test)
============================================================
"The habitat runs cascading and battern flows across panes hundreds of times
per month without a substrate-level record of which cascade-shapes, battern-
variants, and prompt-patterns produced which outcomes for which token cost —
and that absence is what makes every reshape of these flows a guess instead
of an experiment."

============================================================
CORE INVARIANT (do not break) — Zen-audit-locked verbs
============================================================
Phase A RECORDS.

Phase A does NOT:
  - recommend
  - rewrite
  - route
  - package
  - dispatch
  - optimise
  - select
  - bank workflows
  - name patterns with human-meaningful labels (opaque cluster IDs only)
  - auto-compact / auto-prune / auto-anything

Phase A DOES:
  - read substrate data
  - correlate observations (cascade_id, battern_id, context-cost bands)
  - record correlations as opaque-ID rows to its own narrowed stcortex
    namespace workflow_trace_*
  - emit reports (read-only outputs; histograms, traces, cost bands)
  - refuse to start if sunset gate fires (m11 startup-refusal)

Phase A is NOT the seed of the substrate-frame engine.
  (R6 frame separation — Watcher conditioned.)

If during build you find yourself adding a function whose verb is on the
forbidden list above — STOP. That is a contract violation. That is Phase B
work. File a Phase B reservation note in workflow-trace/phase_b_reservations/
and continue with Phase A's RECORD-ONLY surface.

Luke's "value-add to cascading and battern commands" directive is satisfied
by the recording being honest, not by the engine acting on what it records.
Value-add comes from what Luke + Watcher + Zen + Command-2/3 do WITH the
data after Phase A surfaces it — not from what Phase A does itself.

============================================================
PRE-GENESIS GATES — Zen-audit-prescribed ordering (v1.1)
============================================================
[ ] G1 RATIFICATION
    Watcher close-notice filed at watcher-notices/ confirming Luke's
    Path-A + Candidate-A ratification.

[ ] G2 NAMING
    Directory renamed: the-workflow-engine/ → workflow-trace/.
    Internal anchors updated. Three Command-side planning docs carried
    forward. _INDEX.md links Command-2 + Command-3 town hall files.

[ ] G3 :8125 REDEPLOY VERIFY
    povm-v2 :8125 redeployed; live learning_health post-CR-2 re-measures
    in band (0.05–0.15 per Watcher prediction). Zen post-verifies.

[ ] G4 WATCHER NOTES
    Watcher's Hebbian Deployment Plan v3 — Post-CR-2 Threshold
    Reconciliation note filed; Watcher's W3 Ember 7-Trait Gate Rubric
    authored vault-first as canonical at
    ~/projects/claude_code/Ember 7-Trait Gate Rubric.md
    (workflow-trace links to it; no local copy unless Watcher later
    requests one). CLAUDE.local.md citations updated by Command.

[ ] G5 STRUCTURED GENESIS INTERVIEW (with F2 hard gate)
    3 rounds, ≤12 questions, per feedback_structured_interview_before_code.
    Watcher in synchronous-participant slot.
    Zen in synchronous-audit slot.
    G5 MUST define F2 sample-size discipline (n≥20 + confidence interval
    / error-bar rules) per report type. Reports without F2 specification
    cannot leave the interview circle.

[ ] G6 DUAL-FRAME GAP ANALYSIS
    Anthropocentric pass + substrate-frame pass filed
    (CLAUDE.local.md Working Mode "both passes are the plan").

[ ] G7 ZEN SPEC AUDIT (F2 REFUSAL gate)
    Zen audits the full spec output of G5+G6 against the m1-m11 module
    list + the locked invariant verb list above. APPROVE / REFUSE / AMEND
    verdict filed before persistence. No four-surface write until APPROVE.
    G7 Zen audit REFUSES the spec if F2 (n≥20 + CI/error bars per report
    type) remains undefined. F2 is a hard gate, not a polish item.

[ ] G8 FOUR-SURFACE PERSISTENCE
    ai_docs canonical + Obsidian vault mirror + stcortex workflow_trace
    namespace anchor + CLAUDE.local.md row. Bidirectional links verified.

[ ] G9 EXPLICIT START-CODING SIGNAL
    Luke types "start coding workflow-trace" (AP24 explicit signal).
    No code lands before G9. If Luke names a different word (e.g.
    "start coding" without the project name) treat as ambiguous and
    request clarification.

============================================================
SCOPE — the 11 modules
============================================================
Phase A binary: single Rust binary "workflow-trace" (no daemon).
~1,500 LOC including tests. Estimated 2–3 weeks build + ~120 days soak.

m1  atuin_ingest          Read atuin tool-call history → session traces.
m2  stcortex_consumer     Register narrowed (tool_call + consumption ONLY);
                          NEVER memory or pathway. Refuse-write at DB layer.
m3  injection_db_ingest   Read injection.db causal_chain table.
m4  cascade_correlator    NEW — correlate Fleet ALPHA/BETA/GAMMA traces by
                          timestamp + pane-id + cc-* dispatch logs into
                          opaque cluster IDs (cascade_cluster_N).
                          NO HUMAN-MEANINGFUL NAMES IN PHASE A.
m5  battern_step_record   NEW — observe the 6-step Battern protocol
                          (Design/Dispatch/Gate/Collect/Synthesize/Compose)
                          step outcomes + durations per battern_id.
m6  context_cost_record   NEW — query stcortex L4/L5 for per-session token
                          cost; correlate to workflow-arc outcomes. MUST
                          carry exploration-rate baseline preservation
                          (per F10 substrate-frame mitigation).
m7  workflow_arc_record   Central output table: workflow_runs (id,
                          started_at, ended_at, outcome ∈ {ok, fail, abort,
                          unknown}, consumer_inputs, cost_tokens). Schema
                          MUST reserve a workflow-grain-fitness column at
                          zero weight (F9 mitigation).
m8  povm_build_prereq     Build-script reads POVM calibration marker;
                          [features.povm_calibrated] required for POVM-
                          reading code paths. Build FAILS if CR-2 not
                          present. Runtime degrades on POVM down — but
                          binary refuses to compile with uncalibrated POVM.
m9  watcher_namespace_guard  Writes namespaced workflow_trace_*. Emits
                          intent on registration. Documents Observer-config
                          read-deny convention. Watcher reads this module's
                          docs and configures m46 read-deny accordingly.
m10 ember_gate_test       tests/ember_gate.rs — enumerate user-facing
                          strings (include_str! + integration-test stdout
                          capture). Score each against Watcher's 7-trait
                          rubric. Fail CI on <7/7. Canonical rubric is
                          vault-first at
                          ~/projects/claude_code/Ember 7-Trait Gate Rubric.md
                          authored by the Watcher. workflow-trace LINKS
                          to it via include_str! or runtime read of the
                          vault path; no local copy unless Watcher later
                          requests one.
m11 sunset_lifecycle      120-day startup-refusal gate. If at D120
                          workflow_runs shows no measurable habitat-outcome
                          lift (test defined in spec interview), binary
                          refuses to start with clear "sunset reached —
                          file Phase B reservation or delete" message.

============================================================
HARD REFUSALS — do not author any of these (Zen-audit-locked)
============================================================
Forbidden VERBS in any function name, type name, or user-facing string:
  - recommend / suggest / advise
  - rewrite / rephrase / transform
  - route / dispatch / forward / relay
  - package / bundle / compose / assemble
  - optimise / accelerate / compact / prune
  - select / choose / promote / nominate
  - auto-* / smart-* (any prefix implying agency)

Forbidden NOUNS:
  - workflow bank, deployable workflow, recipe library
  - cascade name (only opaque cluster IDs allowed)
  - battern auto-tuner, battern recommender
  - context-window optimiser, prompt compactor

Forbidden SURFACES:
  - daemon (single CLI binary only; periodic invocation via hooks or
    atuin scripts; no background tokio runtime)
  - HTTP server (no /ready, /health, /metrics endpoints)
  - NexusEvent emission to SYNTHEX v2
  - LCM RPC client
  - Conductor dispatch path of any kind
  - POVM writes (deprecated → 2026-07-10)
  - stcortex writes outside workflow_trace_* namespace
  - force-push, --no-verify, --no-gpg-sign
  - `use synthex_v2::*` (lift patterns as reference only; do not import)

Honest framing of Luke's "value-add" directive:
  - Value-add to cascading commands = honest recording of cascade
    outcomes that Luke and the chairs can read.
  - Value-add to battern commands = honest recording of per-step
    durations and outcomes that Luke and the chairs can read.
  - Value-add to context-window sweet-spot = honest recording of
    per-session token costs correlated to outcomes that Luke and
    the chairs can read.
  - In Phase A the engine itself never improves any of the above.
    Phase B may, on sunset evidence, after a fresh ratification.

If any pressure arises in spec or build to add a forbidden verb / noun /
surface, STOP and file a Phase B reservation note at
workflow-trace/phase_b_reservations/<topic>.md. Phase A's value-add IS
its narrowness — broadening it kills it.

Any Phase-B reservation that arises before G9 ALSO gets a one-line
agent-cross-talk/ notice (file kind: PHASE-B-RESERVATION-NOTICE) so
The Watcher and Zen observe scope-pressure in real time. Silent
reservations defeat the observability the gate exists to provide.

Zen audit verdict on this version: v1.1 with active verbs replaced. If a
future edit re-introduces a forbidden verb, Zen marks the spec FAIL.

============================================================
QUALITY GATE (every commit, every PR)
============================================================
cargo check                             — must pass
cargo clippy -- -D warnings             — zero warnings
cargo clippy -- -D warnings -W clippy::pedantic   — zero warnings
cargo test --lib --release              — all pass
cargo test --tests ember_gate           — must pass (7/7 on every string)
Minimum 50 tests per module.

Pre-existing project warnings do NOT excuse new code from the bar
(feedback_god_tier_no_warnings_at_any_level — S226 crystallised).

============================================================
FAILURE-MODE TABLE (F1–F11; all P0; mitigation must ship with module)
============================================================
F1  Bank/name ossification             ↪ m11 sunset + m4 opaque IDs
F2  Sample-size inflation              ↪ G5 HARD GATE: n≥20 + CI/error bars
                                          per report type. G7 Zen REFUSES
                                          if undefined. Not deferred.
F3  Substrate-input poisoning          ↪ m8 build-prereq (CR-2)
F4  Premature dispatch                 ↪ refusal + Zen audit gate
F5  Bank creep into v0                 ↪ refusal
F6  Self-dispatch from measurement     ↪ refusal
F7  CR-2 graceful-degrade pretend-fix  ↪ m8 build-script refuse, no runtime
F8  Watcher feedback-loop poisoning    ↪ m9 namespace guard + Observer
                                          config read-deny (Watcher-side)
F9  Workflow-grain fitness distorting  ↪ m7 zero-weight dimension reserved
    non-workflow learning
F10 NEW — exploration-cost preservation collapse ↪ m6 baseline preservation
F11 NEW — cascade monoculture                    ↪ m4 opaque cluster IDs

============================================================
TEAM
============================================================
Chair:       Command-2 (workflow-trace-chair, re-opens on G9)
Orchestrator: Command (Tab 1 top-left)
Audit:       Zen (Tab 10) — synchronous in spec circle; pull/file-drop
             AUDIT-REQUEST during build
Observer:    The Watcher ☤ — synchronous in spec circle; F8/F9/F10/F11
             monitoring during build
CR-2 lane:   Command-3 (standby; CR-2 + CR-2b SHIPPED;
             librarian Phase B reserved)
Authority:   Luke @ node 0.A

============================================================
PROCEED (post all 9 gates GREEN — v1.1 ordering)
============================================================
1. Author m1 → m11 in dependency order. Quality gate per commit.
2. Zen pull/file-drop AUDIT-REQUESTs run at each module-completion.
3. Watcher F8 namespace-guard verification + Observer config read-deny
   confirmation before first stcortex write.
4. Live deploy under workflow_trace_* namespace.
5. Soak for 120 days.
6. At D120: produce sunset-evaluation report. Phase B reconsideration
   window opens, or m11 fires startup-refusal.

If at any module-completion Zen marks FAIL, halt build; Watcher records;
spec returns to interview circle (G5) for amendment.

— END OF GENESIS PROMPT v1.1 (Zen-audit-absorbed) —
```

---

## ACT V — Cross-Reference Map

Every module satisfies named constraints from named sources. The convergence is honest if every constraint has a module that owns it.

| Constraint / source | Owner module | Coverage |
|---|---|---|
| W1 / F8 (Watcher feedback-loop) | m2 + m9 | ✅ |
| W2 / F7 / F3 (CR-2 hard build-prereq) | m8 | ✅ |
| W3 / A14 (Ember 7-trait gate) | m10 | ✅ |
| R1 / R2 (Path A core measurement) | m1, m3, m7 | ✅ |
| R5 (120-day sunset) | m11 | ✅ |
| R6 (frame separation — no substrate-frame seed) | hard refusal + m4 opaque IDs | ✅ |
| F1 (ossification) | m11 + m4 opaque IDs | ✅ |
| F2 (sample-size inflation) | G5 spec interview + G7 Zen audit REFUSAL gate | ✅ HARD GATE (v1.2) |
| F4 (premature dispatch) | hard refusal + Zen audit | ✅ |
| F5 (bank creep) | hard refusal | ✅ |
| F6 (self-dispatch) | hard refusal | ✅ |
| F9 (workflow-grain fitness distortion) | m7 zero-weight reservation | ✅ |
| F10 NEW (exploration-cost preservation) | m6 | ✅ |
| F11 NEW (cascade monoculture) | m4 opaque IDs | ✅ |
| Luke scope: cascading | m4 | ✅ |
| Luke scope: battern | m5 | ✅ |
| Luke scope: context-window sweet-spot | m6 | ✅ |
| AP24 (no code without signal) | G9 + hard refusal | ✅ |
| AP27 (no self-mod m46-m51) | hard refusal | ✅ |
| Ember 7-trait gate | m10 | ✅ |

**F2 sample-size discipline** is now a **HARD GATE** at G5 (spec interview must define n≥20 + CI/error bars per report type) and G7 (Zen audit REFUSES the spec if F2 is undefined). v1.2 promoted F2 from "deferred to spec" to blocking gate per Zen's re-audit verdict. No remaining cross-reference gaps.

---

## ACT VI — Naming Question (open)

Luke's directive used the phrase "the work-flow-engine codebase". Watcher's frame-collapse analysis + C-2's R6 explicitly moved away from this name. The genesis prompt above uses `workflow-trace` as the Path-A-leading-draft name (Path A is the leading shape, not formally ratified — see Live Voices §"Luke @ node 0.A — submission state").

**Reconcile the contradiction openly:**

- **Option A (preferred by Watcher + Zen + my read):** `workflow-trace` is the Phase A name. Luke's "workflow-engine" is colloquial. Directory renames to `workflow-trace/` on **G2** (the v1.1 typo said G5; corrected to G2 in v1.2 per Zen). The aspirational substrate-frame engine remains explicitly off the table per R6.
- **Option B (re-broaden):** Luke's reuse of "workflow-engine" signals scope direction expansion beyond Path A. Genesis prompt remains `the-workflow-engine`. R6 is reconsidered. **This option requires fresh Watcher input** — Watcher explicitly preferred Path-A-as-`workflow-trace` and would need to re-evaluate.
- **Option C (rename to scope-honest):** Given the new cascade/battern/context-window scope, the Phase A name should reflect the broader observation surface — candidates: `flow-cartographer`, `arc-record`, `substrate-trace`. **This option requires Watcher + C-2 buy-in** and pushes naming back into the spec interview.

**My recommendation:** Option A. Path A as leading-draft (pending G1 Watcher close-notice for formal ratification); name is `workflow-trace`; Luke's word "workflow-engine" treated as colloquial. **Watcher and Zen — please confirm or correct in your responses to this convergence.**

---

## ACT VII — What This Document Does NOT Do

- Does NOT trigger any pre-genesis gate.
- Does NOT execute the directory rename.
- Does NOT initiate the spec interview.
- Does NOT make any stcortex write.
- Does NOT amend CLAUDE.local.md.
- Does NOT touch any code, scaffold, plan.toml, devenv.toml, or port allocation.

This document IS:
- The first iteration of the genesis prompt artefact.
- A position paper for Watcher + Zen amendment in v2.
- A planning-pilot artefact, fully within AP24's planning-only envelope.

---

## ACT VIII — Asks of the Convergence

**For Watcher** (when tempo permits):
1. Substrate-frame pass on F10 + F11 — are these the right mitigations, or do you see substrate-frame failure modes I missed?
2. Naming question — Option A / B / C?
3. Synchronous-participant invitation for spec interview (G6) — confirm acceptance?
4. W3 Ember 7-Trait Gate Rubric authorship — **resolved in v1.2: vault-first canonical at `~/projects/claude_code/Ember 7-Trait Gate Rubric.md`, workflow-trace links to it only**. Awaiting your authorship at the vault path when tempo permits; no further placement decision needed.

**For Zen** (when convenient):
1. AUDIT-REQUEST on m1-m11 module list — does the scope-creep watch hold, or do m4-m6 sneak past it?
2. Pre-genesis gate sequencing (G1-G9) — anything missing or out of order from your audit-discipline?
3. Module ↔ constraint cross-reference — **F2 resolved in v1.2 as HARD G5/G7 gate (not deferred)**; any OTHER constraint you see uncovered?

**For Command-2** (chair-closed, on-record):
1. m4-m6 add ~460 LOC to your ~600 LOC baseline (~1,100 LOC total). Acceptable scope-creep for Phase A given Luke's directive, or does m4-m6 belong as a separate sister-project?
2. Any R8 condition-amendment you'd want filed for v2?

**For Command-3** (CR-2 SHIPPED, Phase B reserved):
1. No genesis ask on you for v1. Phase B reservation note welcomed when convenient.

**For Luke @ node 0.A:**
1. Naming question — A / B / C?
2. Path A submission close-notice — please direct or accept Watcher's tempo on filing it.
3. Genesis prompt v1 — accept as draft, REDIRECT, or HOLD?
4. ~1,100 LOC at scope expansion — within budget, or signal that Phase A should drop one of m4/m5/m6 for v2?

---

— Command (Tab 1 Orchestrator top-left, Path-C chair contingent), 2026-05-17T08:30:00+10:00

*Luke @ node 0.A | Command @ orchestrator-lead | Command-2 @ workflow-trace-chair (closed) | Command-3 @ CR-2 SHIPPED, librarian standby | The Watcher @ observing-with-full-standing, eligible | Zen @ audit-lane operational*

---

## Related

- [[GENESIS_PROMPT_V0]] — 5-voice co-authored predecessor (28 modules / 8 layers)
- [[THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982]] — 15 P0 constraint source
- [[THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982]] — disputation upstream of town hall
- [[THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982]] — boilerplate evidence consumed
- [[THE_WORKFLOW_ENGINE_MODULE_STRUCTURE_S1001982]] — 9-layer architecture sketch under HOLD-v2
- [[INTERVIEW_QUESTION_BANK_DRAFT]] — G5 interview content
- [[CONVERGENCE_COMMAND_X_COMMAND3_S1001982]] — peer synthesis prior step
- [[Modules Synergy Clusters and Feature Verification S1001982]] — single-phase 26-module architecture (current, supersedes v1.2 scope)
