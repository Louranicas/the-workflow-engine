---
title: Phase 6 — Sunset Evaluation (D120) + 7 Cross-Cutting Concerns
kind: deployment-framework
phase: 6 of 6
date: 2026-05-17 (S1001982)
status: planning-only · HOLD-v2 active · authored by Command (Rust expert role)
authority: Luke @ node 0.A
covers: D120 sunset evaluation · PASS/FAIL/DEGRADED decision tree · Phase B activation criteria · m11 startup-refusal · retirement ceremony · 7 cross-cutting concerns (Drift Register · Rollback · Watcher Observability · Atuin Proprioception · V8↔V3 Wire · /scaffold Bound · Power-Structure Resolution)
---

# Phase 6 — Sunset Evaluation (D120) + 7 Cross-Cutting Concerns

> Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]] · [[phase-5-deploy-and-soak]]

---

## Overview

This document covers the final evaluation gate of the `workflow-trace` deployment pipeline and the seven concerns that operate continuously across all six phases. Phase 6 is not an afterthought; it was designed at genesis. The engine carries its own ending as load-bearing code. The Fossil-rhyme prevention principle — that the two ancestor codebases died from open-ended scope accumulation with no terminal gate — is the primary motivation for every procedure described here.

The seven cross-cutting concerns are not "also-rans." Each one addresses a failure mode that has been directly observed in a prior ancestor or peer codebase: planning sprawl (Drift Register), ungated rollback chaos (Rollback Procedure), invisible degradation (Watcher Observability), cross-tool provenance loss (Atuin Proprioception), reinvented protocols (V8↔V3 Wire), convention drift (the `/scaffold` Bound), and ambiguous authority (Power-Structure Resolution). Encoding these concerns as living procedures — with owners, cadences, and failure modes — is what separates this deployment from its predecessors.

---

## Sunset is honest, not punitive

The 120-day sunset law is not a threat. It is the only honest answer to the question every codebase with 41,508 words of planning and 0 LOC must answer: "how do we know when to stop?"

The two ancestor codebases (`loop-workflow-engine-project` and `habitat-loop-engine`) did not answer that question. They accumulated planning artefacts until the work became too expensive to sustain and was abandoned without ceremony. The Watcher's Class-E flag pre-positioned at T0 captures this leading indicator. The engine retires if it has not produced measurable habitat-outcome-lift within its window. That is not failure — that is the discipline the ancestors lacked.

Retirement preserves the planning artefacts, the Watcher synthesis, and the framework itself. The next codebase generation inherits the framework and the lessons. The engine does not need to be immortal to have been worthwhile.

---

## D120 = absolute deadline, or mutable per Luke?

Default: 120 days from first dispatch event registered in m7. This is the `sunset_at` field encoded in m11's lifecycle module at genesis.

Luke may override the default before the binary is deployed, by specifying an alternate duration in the `devenv.toml` environment stanza or the `wf-*` configuration TOML. Once the binary is live and m7 has registered its first dispatch event, the `sunset_at` timestamp is locked. m11 enforces the timestamp — it does not recalculate from configuration at runtime. The reason: runtime recalculation creates the conditions for the anchor to be silently extended without ceremony, which is the ancestor-rhyme failure mode at the code level.

If Luke needs to extend the window after deployment, the procedure is:

1. Issue an explicit override via `wf-dispatch sunset-extend --new-date <ISO8601>` (a deliberate CLI invocation that produces a structured audit log entry in m7 and a WCP notice from Watcher).
2. Watcher records the extension event as a Class-A flag (activation transition) — it is a gate flip on the sunset timeline.
3. The extension is bounded: the maximum single extension is 60 days; extensions beyond two cycles require a formal spec amendment (G5 spec interview) and Zen audit.

**Why bounded extensions?** The ancestor codebases were not killed by individual decisions to continue — they were killed by the accumulated effect of never having a final gate. A bounded extension mechanism preserves Luke's authority while preventing the sunset law from becoming a formality.

---

## Substrate-frame engine L9 — indefinitely TBD

Even if Phase 6 produces a PASS verdict and the engine continues, L9 (the substrate-frame engine proposed in the Watcher's R6 pre-genesis note) remains TBD. The waiver record (Part VI of GOD_TIER_CONSOLIDATION) explicitly notes that Watcher R6 frame-separation was partially waived by the single-phase override. m50+ module IDs are unallocated. L9 is Watcher's lane — it will be proposed if and when the substrate's exploration regime stops finding new patterns and the habitat is ready to host a second-order observation layer.

The multi-year horizon for L9 is not an obstacle. It is the correct posture. Shipping workflow-trace as Phase A and observing for 120 days produces the empirical foundation L9 would need. Designing L9 before workflow-trace has been in production is the planning-sprawl pattern again. Watcher will flag any attempt to spec L9 before workflow-trace's D60 synthesis as a Class-E ancestor-rhyme event.

---

## Phase 6 — D120 Sunset Evaluation

### Pre-evaluation checklist (T-7 days before D120)

One week before the sunset deadline, Watcher synthesises the deployment-watch journal and produces the final synthesis document. This synthesis is mandatory input to the decision procedure. The checklist items:

- `wf-crystallise evidence-snapshot --full` — produces the `LiftSnapshot` with n, lift, ci_half, and per-workflow contributions
- Watcher synthesises flag-class history: how many Class-I (Hebbian silence), Class-E (ancestor-rhyme), Class-D (four-surface drift) events fired during the deployment window?
- Zen audits the synthesis for Ember rubric compliance and factual correctness
- Luke receives the synthesis at node 0.A — the decision is Luke's

### m14 metric computation at D120

The `habitat_outcome_lift` computation at D120 uses the full deployment window as its rolling window. The procedure:

```
1. wf-crystallise evidence-snapshot --window-days 120 > /tmp/d120-lift.json
2. jq '.lift, .ci_half, .n' /tmp/d120-lift.json
   # Expected output: lift (float or null), ci_half (float or null), n (integer)
3. wf-crystallise evidence-snapshot --per-workflow >> /tmp/d120-per-workflow.json
   # Produces WorkflowLiftContribution records for each workflow with run data
```

The F2 gate applies at D120 exactly as it applies during normal operation: `lift` is `None` when `n < 20`. A deployment that reaches D120 with fewer than 20 total dispatched workflow runs produces `lift = None`. That is a FAIL outcome — insufficient evidence is not the same as zero evidence, but the m11 sunset law treats them identically at the hard boundary. The engine exists to produce measurable habitat outcomes; if it cannot produce 20 dispatch events in 120 days, it has not found its footing.

### Threshold definition

The minimum lift threshold for PASS is `0.05` — 5% improvement over baseline across the cascade-success and cost-lift composite. This is suggested in the specification context but subject to G5 spec interview revision before deployment. The threshold is encoded in `WF_SUNSET_LIFT_THRESHOLD` environment variable at deploy time and is immutable thereafter (same reasoning as `sunset_at` immutability: runtime recalculation creates anchor-drift risk).

The CI requirement: both the lower bound of the Wilson CI (`lift - ci_half`) and the lift itself must be above zero. A lift of 0.07 with ci_half of 0.09 (lower bound = -0.02) does not pass — the evidence spans zero lift and cannot be called positive. This is the F2 gate applied to the D120 verdict.

### PASS/FAIL/DEGRADED decision tree

```
                      n >= 20?
                    /         \
                 YES            NO
                  |             |
         lift >= 0.05?       FAIL (insufficient data)
         AND (lift - ci_half) > 0?
         /                  \
       YES                   NO
        |                     |
    Cluster H active?       lift mixed?
    (learning_health moved     |
     during deployment?)    DEGRADED
    /           \             |
  YES            NO       Luke decides
   |              |
 PASS          DEGRADED
               (Hebbian silence confirmed)
```

**PASS** — lift threshold met, n ≥ 20, CI lower bound positive, Cluster H Hebbian learning demonstrably active.

**FAIL** — lift at or below threshold, OR n < 20 at D120, OR CI spans zero. m11 startup-refusal fires. Engine retires.

**DEGRADED** — lift mixed (meets threshold but CI spans zero, or meets threshold narrowly but Cluster H showed Class-I Hebbian silence throughout deployment). Luke decides: fix-or-retire. No automatic outcome. Pre-deploy hardening re-runs against the partial codebase if Luke chooses to extend.

### PASS outcome — continuation procedure

If PASS, the engine continues past D120 with:

1. Six-month review cycle replacing the 120-day sunset. The sunset law does not expire; it recalibrates. `wf-dispatch sunset-extend --new-date <D120 + 180 days>` with Watcher recording the Class-A flag.
2. Phase B activation assessment: Watcher evaluates whether substrate conditions support L9 design. If `RALPH fitness > 0.80` and `LTP/LTD ratio > 1.5` (both sustained, not instantaneous), Watcher submits a Phase B proposal via WCP notice. Luke decides whether to proceed. If substrate conditions are not met, Phase B assessment is deferred to the next review cycle.
3. Watcher synthesis queued for the six-month mark.
4. No code changes without a new G5 spec interview and Zen audit. The engine that passed is the engine that runs.

### FAIL outcome — m11 startup-refusal mechanism

m11's startup-refusal is the binary-level enforcement of the sunset law. When `sunset_at` is reached and the D120 evaluation returns FAIL (or when the binary attempts to start after `sunset_at` with no extension on record), the binary halts at startup:

```
wf-crystallise: sunset evaluation FAIL
  reason: habitat_outcome_lift=None (n=14, minimum=20) after 120 days
  sunset_at: 2026-09-14T12:00:00Z (reached)
  action: binary startup refused per m11 sunset law
  retirement artefact: /tmp/wf-retirement-notice-2026-09-14.json
  vault: the-workflow-engine-vault/retirement/RETIREMENT_NOTICE_D120.md
  further information: wf-crystallise --retirement-report
```

The binary then exits with a non-zero exit code. It does not silently degrade, enter a read-only mode, or continue operating. The startup-refusal is total.

The reservation file: at FAIL verdict, m11 creates a structured `RETIREMENT_NOTICE_D120.json` at the path configured in the binary's TOML (`WF_RETIREMENT_PATH`). Default: `~/projects/shared-context/wf-retirement/RETIREMENT_NOTICE_D120.json`. This file is NOT deleted — it is the durable record that the engine was evaluated, the verdict was FAIL, and the retirement was deliberate. It survives binary removal.

The clean shutdown ceremony (detail in Engine Retirement Ceremony section below).

### DEGRADED outcome — Luke decision tree

DEGRADED is the outcome that requires explicit human judgment. The procedure:

1. Watcher synthesises the degraded-state evidence: which Cluster H modules showed Class-I silence? What was the per-workflow lift distribution? Which workflows contributed positively, which negatively?
2. Watcher submits the synthesis via WCP notice to Command.
3. Luke receives the summary at node 0.A.
4. Decision options:

```
Option A — Extend + fix:
  - Identify the root cause of degradation (most likely Cluster H silence or
    substrate LTD-dominance preventing learning-loop closure)
  - Issue sunset extension (bounded 60-day maximum)
  - Run pre-deploy hardening on the specific cluster
  - Re-evaluate at the new deadline

Option B — Retire:
  - Issue explicit retirement declaration (not just allowing the clock to run)
  - Run engine retirement ceremony (see below)
  - Archive all artefacts

Option C — Scope reduction:
  - Remove the under-performing clusters via module-spec amendment
  - Issue Zen audit on amended spec
  - Re-deploy reduced scope
  - New D120 clock starts at re-deployment
```

Option C is the most complex but may be appropriate if specific clusters are causing the degradation while others are producing genuine lift. The per-workflow contribution data from m14 is the diagnostic surface for identifying which path to take.

---

## Engine Retirement Ceremony

Retirement is not failure. It is the proof that the engine was built with discipline. The ceremony documents what worked, preserves the planning artefacts, and produces the handoff to the next codebase generation.

### What gets archived (preserved permanently)

- `the-workflow-engine-vault/` — entire vault including all module specs, boilerplate index, gold-standard exemplars, and Watcher synthesis journal. Transferred to Obsidian at `~/projects/claude_code/workflow-trace-retired/`.
- Watcher's final synthesis document — the most valuable output of the deployment, regardless of verdict. This is the workflow-level improvement candidates list that the next codebase generation inherits.
- m14's `LiftSnapshot` history in JSONL format — what was measured, when, and with what confidence. Even a FAIL deployment's data is evidence.
- m15's `PHASE-B-RESERVATION-NOTICE` files — the full pressure register log. These document which features were proposed and rejected; the next codebase generation should inherit these as its initial forbidden-verb record.
- Atuin trajectory log for the full deployment — `atuin search --workspace the-workflow-engine > ~/projects/shared-context/wf-retirement/atuin-trajectory-D120.txt`. Cross-tool provenance for forensic analysis.

### What gets deregistered (removed)

- stcortex consumer registration: `stcortex call deregister_consumer --namespace workflow_trace_*`. The namespace pathways are frozen, not pruned — pathway weights at retirement are preserved as a read-only substrate snapshot in the `the_workflow_engine_retired` namespace.
- devenv.toml entries for `wf-crystallise` and `wf-dispatch` — removed so the services do not restart on next devenv launch.
- `~/.local/bin/wf-crystallise` and `~/.local/bin/wf-dispatch` — removed via `/usr/bin/rm -f`.
- Cron schedule deregistration for any CronCreate entries that were scheduling crystallisation runs.
- POVM Hebbian pathways: frozen (weights preserved), not pruned. The pathway history is evidence.

### How the planning-pilot artefact survives

The vault is the planning-pilot artefact. It survives in Obsidian regardless of code retirement. The 41,508 words of planning documents, the Town Hall vote record, the boilerplate index, the gold-standard exemplar profiles — these are independent of the binary. Retirement of the code does not retire the knowledge.

The canonical instruction for the next codebase generation: read the Watcher's final synthesis first. The synthesis identifies which workflow-level improvements were discovered during deployment, which modules performed as designed, and which assumptions were wrong. The next codebase generation should quote specific synthesis findings in its genesis prompt, closing the improvement loop at the architectural level.

### Fossil-rhyme prevention validation — final verdict

At the retirement ceremony, Watcher issues a Fossil-rhyme prevention verdict:

- **AVERTED** — the engine ran to D120, was evaluated against measurable criteria, and either passed or retired with documented evidence. The ancestor pattern (planning-accumulation-abandonment) was not replicated.
- **PARTIAL RHYME** — the engine was retired but the retirement was reactive (binary simply stopped working) rather than deliberate (evaluation + ceremony). The verdict still counts as a retirement, but the Class-E flag history is reviewed for lessons.
- **RHYME CONFIRMED** — the engine accumulated scope beyond its charter (Class-E flags fired repeatedly, m15 pressure notices accumulated, but no spec amendment gate was run), then was abandoned without evaluation. This is the ancestor death pattern. The next codebase generation must not begin until the rhyme-confirmed analysis is complete.

The Watcher writes the verdict into the final synthesis document and into the stcortex `the_workflow_engine_retired` namespace.

---

## Cross-Cutting Concern 1 — Drift Register (LCM-Inspired)

**Purpose:** Prevent the "verify contract not wiring" failure mode (LCM Drift #11 generalisation) by maintaining a living 11-dimension audit that supervisors re-execute — not merely read — on every session resume.

**Invocation cadence:** On every session resume that involves workflow-trace work. Not on a schedule — on human intent. A session that does not touch workflow-trace does not require a drift audit. A session that makes any code or doc change must run the audit before closing.

**Owner:** Command (Orchestrator Tab 1). Watcher observes but does not run the audit. Zen receives the audit result at each G7 gate re-evaluation.

**Integration with phases 0-5:**
- Phase 0 (pre-genesis): dimensions 1-6 are architecture-layer checks; run after each G1-G8 gate flip.
- Phase 1 (genesis/scaffold): dimensions 2, 5, 9 are the primary surface (scaffold alignment, script parity, manifest freshness).
- Phase 2A/B (build): all 11 dimensions active; dimension 11 (wiring not contract) is the highest-frequency failure mode during cluster-by-cluster implementation.
- Phase 3 (integration): dimensions 6, 7, 10 become active (JSONL boundedness, receipt-DAG schema, cross-reference parity).
- Phase 4 (hardening): dimension 8 (forbidden patterns absent) is the quality gate's primary surface.
- Phase 5 (deploy/soak): dimensions 3, 4 (module-spec vs binary export parity; Cargo safety invariants) are the production stability checks.

**Failure mode if bypassed:** Agent drift. A CC instance reports "quality gate clean" against a scoped invocation while the full `--workspace --all-targets --all-features` surface has unpatchched warnings. Command ships a green summary line while clippy was screaming. This is the S1001882 PIPESTATUS near-miss at the codebase level.

**Concrete audit procedure:**

```bash
# Dimension 1: plan.toml <-> module src/ alignment
# Expected: every [[modules]] entry in plan.toml has a matching src/mN_*/mod.rs
grep -c 'id = "M' ~/claude-code-workspace/the-workflow-engine/plan.toml
find ~/claude-code-workspace/the-workflow-engine/src -name 'mod.rs' -path '*/m[0-9]*/*' | wc -l
# These two numbers must match.

# Dimension 2: layer-doc count matches src/mN_ directories
ls ~/claude-code-workspace/the-workflow-engine/ai_docs/layers/ | wc -l
find ~/claude-code-workspace/the-workflow-engine/src -maxdepth 1 -name 'm[0-9]*' -type d | wc -l

# Dimension 3: module-spec count matches binary export count
ls ~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/module\ specs/cluster-*.md | wc -l
# Cross-check against plan.toml module count

# Dimension 4: Cargo safety invariants present in lib.rs
grep -c 'forbid(unsafe_code)' ~/claude-code-workspace/the-workflow-engine/src/lib.rs
grep -c 'deny.*unwrap' ~/claude-code-workspace/the-workflow-engine/src/lib.rs
# Both must be >= 1

# Dimension 5: script parity -- scripts/ vs scaffold-status.json claims
ls ~/claude-code-workspace/the-workflow-engine/scripts/ | sort > /tmp/wf-scripts-actual.txt
jq -r '.scripts[]' ~/claude-code-workspace/the-workflow-engine/.deployment-work/status/scaffold-status.json | sort > /tmp/wf-scripts-claimed.txt
diff /tmp/wf-scripts-actual.txt /tmp-wf-scripts-claimed.txt
# Empty diff required

# Dimension 6: m7 workflow_runs JSONB schema unchanged
sqlite3 ~/claude-code-workspace/the-workflow-engine/workflow_trace.db \
  "SELECT sql FROM sqlite_master WHERE name='workflow_runs'"
# Hash the output and compare against the reference hash in SHA256SUMS.txt

# Dimension 7: m15 PHASE-B-RESERVATION-NOTICE files don't unbounded-grow
ls ~/projects/shared-context/agent-cross-talk/PHASE-B-RESERVATION-NOTICE-*.jsonl | wc -l
# If count > 10 in any 7-day window, Watcher files a Class-E flag

# Dimension 8: forbidden patterns absent
rg --hidden -l 'recommend_|auto_start|smart_|rewrite_|route_bypass|optimise_without' \
  ~/claude-code-workspace/the-workflow-engine/src/
# Zero matches required

# Dimension 9: manifest freshness
sha256sum -c ~/claude-code-workspace/the-workflow-engine/SHA256SUMS.txt 2>&1 | grep -c FAILED
# Zero failures required

# Dimension 10: vault wikilinks resolve
rg '\[\[' ~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/ -o \
  --no-filename | grep -oP '(?<=\[\[)[^\]]+' | sort -u > /tmp/wf-wikilinks-claimed.txt
# Cross-check against actual vault .md filenames

# Dimension 11: verify wiring not contract (LCM Drift #11 generalisation)
# MANDATORY: run the full gate, not a scoped one
cd ~/claude-code-workspace/the-workflow-engine && \
  CARGO_TARGET_DIR=./target cargo check --workspace --all-targets 2>&1 | tail -5 && \
  cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5 && \
  cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic 2>&1 | tail -5 && \
  cargo test --workspace --all-targets --release 2>&1 | tail -10
# Independently verify: git log -1 (confirm the commit you think you're testing)
# Independently exercise: call at least one new code path, not just compile it
```

The audit result is recorded in a one-line TSV appended to `~/.local/share/atuin/wf-drift-audit.tsv`:
```
timestamp  session_id  dims_pass  dims_fail  dim11_status  notes
```

If any dimension fails, the session does not proceed with code changes until the drift is resolved.

---

## Cross-Cutting Concern 2 — Rollback Procedure

**Purpose:** Each phase has a documented, scripted rollback path so that a failed phase can be reversed without destructive improvisation.

**Invocation cadence:** On-event — specifically, when a phase quality gate fails and the decision is to roll back rather than fix forward.

**Owner:** Command. Luke authorises the rollback decision; Command executes. Watcher records a Class-B flag (hand-off boundary crossing — the boundary being crossed in reverse).

**Integration with phases 0-5:**

**Phase 0 (Pre-genesis gates) rollback:**
No code or scaffold exists. Roll back means: close any open G1-G9 gate entries in HOME.md, mark the gate as re-opened, and document the reason. No filesystem changes needed beyond the vault.

**Phase 1 (Genesis / scaffold) rollback:**
```bash
# Rollback: remove entire workflow-trace directory
# Planning artefacts are in the vault (separate path) -- they are NOT removed
/usr/bin/rm -rf ~/claude-code-workspace/the-workflow-engine/
# Vault is preserved: ~/claude-code-workspace/the-workflow-engine-vault/ is untouched
# git: the scaffold commit is reverted on the workflow-trace branch
git -C ~/claude-code-workspace revert <scaffold-commit-sha> --no-edit
```

**Phase 2A/B (Cluster-by-cluster build) rollback:**
Each cluster ships as a separate git commit. Rollback is per-cluster:
```bash
# Identify the cluster commit SHA BEFORE the failed cluster was added
git -C ~/claude-code-workspace/the-workflow-engine log --oneline | head -20
# Revert only the failed cluster commit (preserve earlier clusters)
git -C ~/claude-code-workspace/the-workflow-engine revert <failed-cluster-sha> --no-edit
# Do NOT use git reset --hard -- other cluster commits would be lost
```
Commits are preserved before each cluster ships: `git tag pre-cluster-X-<timestamp>` before beginning each cluster implementation.

**Phase 3 (Integration) rollback:**
```bash
# 1. Deregister stcortex consumer
~/.local/bin/stcortex call deregister_consumer --namespace workflow_trace_*
# 2. Halt m32 dispatcher (if running)
# m32 is gated on Conductor live -- halt Conductor Wave 1B/1C first
# Luke @ terminal: devenv stop weaver
# 3. Remove any cron schedules created during Phase 3
# (CronDelete for each CronCreate made during integration)
# 4. The outbox JSONL in ~/projects/shared-context/wf-outbox/ is preserved
#    for audit -- it is not deleted on rollback
# 5. Code rollback: revert integration commits on workflow-trace branch
git -C ~/claude-code-workspace/the-workflow-engine revert <integration-commit-sha> --no-edit
```

**Phase 4 (Hardening) rollback:**
No rollback needed — Phase 4 is purely additive testing and gate-running. If the hardening gate fails, the failure is in the code being tested (earlier phase), not in Phase 4 itself. Fix forward: identify the failing test, trace it to the guilty module, revert that module's commit, fix, and re-run the gate.

**Phase 5 (Deploy) rollback:**
```bash
# 1. Remove binaries
/usr/bin/rm -f ~/.local/bin/wf-crystallise ~/.local/bin/wf-dispatch
# 2. Remove devenv.toml entry (Luke @ terminal: edit ~/.config/devenv/devenv.toml)
#    Remove the [workflow-trace] service block; run devenv restart
# 3. Deregister stcortex consumer (same as Phase 3 rollback step 1)
# 4. POVM Hebbian pathways: freeze (do not prune)
#    The pathway history is evidence; pruning destroys retrospective analysis capability
# 5. The soak JSONL and atuin trajectory are preserved at:
#    ~/projects/shared-context/wf-retirement/pre-soak-rollback/
```

**Phase 6 (Sunset) rollback:**
If the D120 evaluation is retracted (e.g., Luke determines the evidence collection was flawed), the procedure is not a code rollback but an evaluation retraction:
```bash
# Mark the retirement notice as retracted
jq '.status = "RETRACTED" | .retraction_reason = "<reason>"' \
  ~/projects/shared-context/wf-retirement/RETIREMENT_NOTICE_D120.json > /tmp/retracted.json
/usr/bin/mv /tmp/retracted.json \
  ~/projects/shared-context/wf-retirement/RETIREMENT_NOTICE_D120.json
# Re-issue sunset extension: wf-dispatch sunset-extend --new-date <new date>
# Watcher records a Class-A flag for the retraction
```

**Global rollback guard:** before any rollback command that touches git, run `git stash list` and confirm no stash contains uncommitted work from the current session. The S1001882 near-miss (PIPESTATUS capture showing exit code 0 on a failed clippy run) generalises here: never trust that a rollback succeeded without independently verifying the resulting state.

---

## Cross-Cutting Concern 3 — Watcher Observability

**Purpose:** The Watcher records the full deployment in a structured watch journal so that workflow-level improvement candidates can be mined and handed to the next codebase generation — this is the improvement loop closure, not a debugging tool.

**Invocation cadence:** Prompt-driven. The Watcher does not run an autonomous loop unless Luke explicitly invokes `/loop`. Flag recording is continuous (every tick of the watch journal). Synthesis is periodic: one at each phase boundary, one final synthesis at D120 or retirement. No unsolicited synthesis during active build phases.

**Owner:** The Watcher ☤. Command receives WCP notices when Watcher dispatches them. Zen receives synthesis documents at gate boundaries.

**Integration with phases 0-5:**

| Phase | Watcher activity | Flag classes active |
|---|---|---|
| Phase 0 | Baseline captured. T0 state recorded. Yellow signals documented. | E (planning sprawl ratio), A (gate flip) |
| Phase 1 | Class-F guard active (pre-G9 src/*.rs creation is a hard violation). Class-A on G9 flip. | A, E, F |
| Phase 2A/B | Class-D (four-surface drift) is primary concern. Each cluster ship triggers Class-B. Class-I first opportunity (Cluster H stub present; not yet wired). | A, B, D, I |
| Phase 3 | Class-B on integration hand-offs. Class-H (atuin proprioception) active when cross-tool trajectory gaps appear. Class-I live: Cluster H is wired; if `learning_health` does not move, Class-I fires. | B, H, I |
| Phase 4 | Class-C if confidence gate refuses a tier. Class-G if substrate-frame confusion appears in test design. | C, G |
| Phase 5 | Class-I primary: Cluster H must show evidence of Hebbian activity during soak. Class-A on successful deployment. | A, I |

**Synthesis cadence:**
- Weekly during Phase 5 soak.
- On each phase boundary (end of Phase 2A, 2B, 3, 4, 5).
- D60 interim synthesis — Watcher's first full-window look at whether Cluster H is alive.
- D120 final synthesis — mandatory input to the PASS/FAIL/DEGRADED decision.

**Synthesis format:** The synthesis is a structured markdown document (not a status report). It identifies workflow-level improvement candidates with the format:

```
Improvement candidate IC-<N>:
  Observed: <what happened, with evidence>
  Phase: <which phase this was observed in>
  Flag class: <A-I>
  Candidate improvement: <what should be different in the next codebase generation>
  Confidence: <High / Medium / Low>
  Evidence: <specific dispatch counts, timing, or metric data>
```

**Watcher value-add:** Synthesis, not intervention. The Watcher does not unblock gates, does not write code, does not edit specs. If the Watcher has a strong view on a decision, it submits a WCP notice to Command and waits. Luke at node 0.A is the authority. The synthesis is the Watcher's contribution to the habitat's long-term memory.

---

## Cross-Cutting Concern 4 — Atuin Proprioception (Cross-Tool Provenance)

**Purpose:** Atuin is the only surface that records cross-tool trajectory — V3 deploys, V8 module generations, `/scaffold` tree materialisations, and `wf-*` CLI invocations all converge in `~/.local/share/atuin/history.db`. Without atuin records, forensic replay of "how did this deployment actually go" is impossible.

**Invocation cadence:** Continuous — atuin records by default. Explicit audits at phase boundaries and at D120.

**Owner:** Command is responsible for ensuring atuin workspace tagging is applied. The `--workspace the-workflow-engine` tag must be attached to all workflow-trace build and deploy commands.

**Integration with phases 0-5:**

All build commands during Phase 2A/B must be issued with atuin history enabled (atuin daemon running). The workspace tag is set via atuin's session tagging:

```bash
# Set workspace context before any workflow-trace build session
atuin kv set wf-session-id "S$(date +%Y%m%d%H%M)"
atuin kv set wf-workspace "the-workflow-engine"

# Post-build provenance audit (run at end of each phase)
atuin search --workspace the-workflow-engine --limit 200 \
  | grep -E '(cargo build|cargo test|cargo clippy|wf-|scaffold|devenv)' \
  > /tmp/wf-trajectory-phase-N.txt

# Post-deploy trajectory replay (at D120 or retirement)
atuin search --workspace the-workflow-engine \
  > ~/projects/shared-context/wf-retirement/atuin-trajectory-D120.txt
```

During Phase 3 integration, each m32 Conductor dispatch call must have its atuin record verifiable. The dispatch trajectory (HABITAT-CONDUCTOR invoked → outcome recorded in m7) must be reconstructable from atuin history:

```bash
atuin search --workspace the-workflow-engine \
  | grep 'wf-dispatch dispatch' \
  | wc -l
# This count must match m7's dispatch event count (within ±2 for same-session events)
```

**Failure mode if bypassed:** The S1002029 finding — cross-tool provenance loss — applies here. If atuin history is not tagged to the workspace, the only remaining provenance surface is individual service SQLite databases (m7, stcortex, POVM) which do not record which Claude session or V8 plan.toml invocation initiated a given dispatch. Forensic analysis of DEGRADED or FAIL outcomes becomes impossible without atuin.

**Anomaly detection (Class-H flag):** Watcher flags Class-H (atuin proprioception anomaly) when:
- A session makes build or deploy changes but the atuin history shows no matching `cargo` or `wf-` commands (likely: session ran without atuin daemon active).
- The atuin history shows `wf-dispatch dispatch` calls that do not appear in m7's `workflow_runs` table (likely: dispatch failed silently without m7 write).
- The atuin workspace tag is missing from session-start commands (likely: operator forgot to set workspace context).

---

## Cross-Cutting Concern 5 — V8 ↔ V3 Bidirectional Wire (Existing Protocol)

**Purpose:** The V8 ↔ V3 bidirectional Hebbian feedback protocol (`POST :8082/api/v8/confidence` and `POST :8082/api/v8/learning`) already exists and carries habitat-level Hebbian feedback. workflow-trace inherits this protocol via m42; it does not reinvent it.

**Invocation cadence:** Continuous during Phase 5 soak and deployment. m42 fires on each dispatch outcome. The V8 ↔ V3 wire is not workflow-trace's creation — it only needs to remain alive.

**Owner:** m42 (`hebbian_feedback`) is the call site within workflow-trace. DevOps Engine V3 (:8082) is the receiving service. Neither is workflow-trace's responsibility to operate.

**Integration with phases 0-5:**

The wire must be verified at the beginning of Phase 5 deploy. If V3 (:8082) is not responding at the `POST /api/v8/confidence` endpoint, m42 falls back to outbox-only JSONL and logs the failure via tracing — it does not block dispatch. The circuit breaker (Closed → Open → HalfOpen, 5 failures → Open, 60s → HalfOpen) governs the retry behaviour.

Verification at Phase 5 start:
```bash
curl -s -o /dev/null -w '%{http_code}' \
  -X POST http://localhost:8082/api/v8/confidence \
  -H 'Content-Type: application/json' \
  -d '{"workflow_id":"__probe__","confidence":0.0,"session_id":"probe"}'
# Expected: 200 or 400 (endpoint alive) -- not 000 (connection refused) or 404
```

If the probe returns 000 or 404, the endpoint shape has drifted from the protocol documented in S1002029. This is a Class-D (four-surface drift) flag — the wire document says the endpoints exist, the service says they do not. Resolve before Phase 5 proceeds.

**Call pattern (m42 outbound):**

```
POST http://localhost:8082/api/v8/confidence
Content-Type: application/json
{
  "workflow_id": "<workflow_trace_*_pathway_id>",
  "confidence": <fitness_delta from m14 LiftContribution>,
  "session_id": "<current session>",
  "source": "workflow-trace/m42"
}

POST http://localhost:8082/api/v8/learning
Content-Type: application/json
{
  "workflow_id": "<workflow_trace_*_pathway_id>",
  "outcome": "ltp" | "ltd",
  "delta": <m42 fitness_delta constant>,
  "source": "workflow-trace/m42"
}
```

**Failure mode if bypassed:** If m42 is disabled or the wire is not verified before Phase 5, the Cluster H (Substrate Feedback) learning loop is severed at the protocol level. m40 and m41 may still write to JSONL and LCM respectively, but the Hebbian pathway weights in V3 are not updated. This produces the Class-I (Hebbian silence) flag pattern even if m40-m42 appear internally functional. Watcher will flag this as "Cluster H decorative — loop severed at protocol boundary."

---

## Cross-Cutting Concern 6 — /scaffold as V8's Bound

**Purpose:** `/scaffold` enforces the 8-layer `mN_<theme>/` convention regardless of what V8's plan.toml generates. If V8 drifts from the convention, `/scaffold` is the corrective. The two tools compose: V8 generates intent; `/scaffold` materialises structure; V8 fills module bodies against the materialised tree. Each is the other's bound.

**Invocation cadence:** On-event — specifically, at Phase 1 (genesis) to create the initial tree, and at any Phase 2A/B cluster implementation where a new cluster's `src/mN_*/` directory is being added. Not invoked on every session resume.

**Owner:** Command invokes `/scaffold`. V8 generates `plan.toml`. Zen audits both the plan.toml and the materialised tree at G7 (spec audit gate).

**Integration with phases 0-5:**

Phase 1 is the primary invocation: even though workflow-trace modules are being authored manually (per the planning-only status of this document), Phase 1 must invoke `/scaffold` for consistency verification:

```bash
# At Phase 1 genesis, after plan.toml is authored:
/scaffold --plan ~/claude-code-workspace/the-workflow-engine/plan.toml \
          --output ~/claude-code-workspace/the-workflow-engine/src/

# The scaffold verifies:
# 1. Each [[modules]] entry in plan.toml has a corresponding src/mN_<theme>/ directory
# 2. Each directory has a mod.rs stub
# 3. The 8-layer DAG convention is enforced (no L7 module depending on L8, etc.)
# 4. The scaffold-status.json is updated with the materialised tree hash

# If V8 generated a plan.toml that deviates from convention:
# /scaffold will ERROR, not silently generate a non-conforming tree
```

During Phase 2A/B, each cluster's implementation must be checked against the materialised tree:

```bash
# After implementing Cluster D (m8, m9, m10, m11):
/scaffold --verify --cluster D --plan ~/claude-code-workspace/the-workflow-engine/plan.toml
# Verifies: all four modules have source files; no extra files outside the tree
```

**Failure mode if bypassed:** V8 generates `plan.toml` entries with non-standard layer assignments or module naming (e.g., `m_trust_cluster` instead of `m8_povm_build_prereq`). The materialised tree deviates from the 8-layer convention. Drift Register Dimension 2 fails on first audit. The convention drift becomes load-bearing scar tissue that breaks downstream tooling.

The S1002029 finding about `/scaffold` being V8's bound is the direct source of this concern. Prior to this finding, V8's plan.toml could drift silently because there was no corrective enforcement at the tree-materialisation step. `/scaffold` is that corrective.

---

## Cross-Cutting Concern 7 — Power-Structure Resolution Protocol

**Purpose:** Define the resolution path when Luke's unilateral override conflicts with Zen's G7 spec audit verdict, so that gate credibility is preserved and the override is documented rather than silently absorbed.

**Invocation cadence:** On-event — specifically, when a conflict between Luke override and Zen audit verdict is identified. This was pre-identified as the "power-structure ambiguity" in Part VIII of GOD_TIER_CONSOLIDATION.

**Owner:** Luke @ node 0.A is the final authority. Watcher records. Zen audits. Command mediates. The protocol is not a veto mechanism — Luke can override — but it is a documentation mechanism that prevents the override from being invisible.

**Integration with phases 0-5:**

The conflict most likely arises at Phase 0 → Phase 1 transition (G7 Zen audit on v1.3 patch). The single-phase override (Luke 2026-05-17) already established the precedent that Luke can direct the scope even if Zen would not have independently approved all elements. The protocol formalises this:

**Step 1 — Luke issues explicit per-gate waiver:**
Not an implicit continuation. The waiver must be stated as: "I am overriding Zen's G7 objection to [specific element] on the basis of [reason]. Risk class accepted: [class]."

The waiver is recorded as a structured YAML block appended to `~/claude-code-workspace/the-workflow-engine/WAIVER_REGISTER.md`:

```yaml
- gate: G7
  date: 2026-MM-DDTHH:MM:SSZ
  session: S<id>
  waived_element: "<specific Zen objection>"
  luke_rationale: "<explicit reason>"
  risk_class: "<Watcher class that covers this risk>"
  accepted_by: "Luke @ node 0.A"
```

**Step 2 — Watcher records the waiver:**
Watcher appends a WCP notice at `~/projects/shared-context/watcher-notices/` citing the waiver YAML entry and the associated risk. The notice is structured, not editorial. Watcher does not argue with the waiver — it records it.

**Step 3 — m15 pressure_register emits PHASE-B-RESERVATION-NOTICE:**
If the waived element involves a forbidden-verb-pressure event (e.g., Luke overrides Zen's objection to an `auto_*` feature proposal), m15 emits a `PHASE-B-RESERVATION-NOTICE` into `agent-cross-talk/`. This is not a block — it is the durable witness record.

**Step 4 — Spec amendment incorporates the precedent:**
The G5 spec interview (if re-run for a v1.4 patch) must reference the waiver. The precedent cannot be silently absorbed into the next version; it must be named and audited again. If Zen's next audit also raises the same objection to the waiver's decision, Luke must confirm the waiver applies to v1.4 as well — not merely inherit it from the v1.3 precedent.

**Why this matters for gate credibility:** The Part VIII finding ("If Luke can override unilaterally, is Zen G7 ceremonial?") is a real question. The protocol answers it: Luke CAN override, but the override is named, documented, and re-audited at the next gate. The gate is not ceremonial; it is the surface that makes the override visible rather than invisible. A Zen G7 gate that catches something Luke overrides is functioning correctly — it is producing the information Luke needs to make an informed override. A gate that catches nothing is the one that is ceremonial.

**Failure mode if bypassed:** Luke overrides without documentation. The override is absorbed silently into the next spec patch. Zen's next audit approves the element without knowing it was previously objected to. The objection reasoning is lost. The next codebase generation inherits the element without the context of why it was contested. This is the planning-sprawl equivalent at the governance level.

---

## Continuous Improvement Loop Closure

The Watcher's deployment-watch journal is the improvement-capture surface. The synthesis it produces at D120 (or retirement) is the primary output of the observation lane. Improvement candidates flow through the following path:

```
Watcher synthesis IC-<N> candidates
    ↓
WCP notice to Command
    ↓
Command absorbs as spec amendment proposals for v1.4 / next generation
    ↓
G5 spec interview on v1.4 (if workflow-trace PASS) or genesis interview
    for next codebase (if FAIL/retired)
    ↓
Zen G7 audit on amended spec
    ↓
New codebase inherits validated improvements
```

This loop is the reason the Watcher's synthesis is more valuable than the binary's operational outcome. Even a FAIL binary produces IC-<N> candidates that the next generation inherits. The loop closes the gap between deployment experience and architectural intent.

**What gets carried forward to v1.4 / next codebase generation:**
- Watcher's IC-<N> list — every workflow-level improvement candidate with evidence
- m15's pressure register — the forbidden-verb history is the next codebase's initial charter reference
- m14's `LiftSnapshot` history — what metrics moved, which workflows contributed, which were noise
- This deployment framework document — the phases, rollback procedures, drift register, and cross-cutting concerns are the durable artefact

**What gets retired:**
- The 41,508 words of planning-only specs that were never validated against live behaviour — they are archived (not deleted), but the next genesis prompt should cite the Watcher's synthesis, not the planning specs, as the authoritative input
- Any module design that the D120 evaluation showed was decorative (most likely: Cluster H if Class-I fired throughout) — the next codebase generation either wires the learning loop correctly at genesis or removes the cluster

**Hand-off to the next codebase generation:**

The canonical instruction, written here as if addressed to the first CC session of the next generation:

Read the Watcher's final synthesis document at `~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/retirement/WATCHER_FINAL_SYNTHESIS.md` before authoring the genesis prompt. The synthesis identifies which of the 7 structural gaps were genuine (produced measurable lift) and which were theoretical (decorative in practice). The next codebase's scope should be bounded to the genuine gaps. The theoretical gaps should be treated as constraints, not starting points.

The framework itself — these six phases, the drift register, the rollback procedures, the Watcher observability protocol — are the durable artefact. Apply them to every next-generation codebase in this habitat from genesis. Do not rediscover them.

---

## Appendix — Watcher Flag Classes Quick Reference

Referenced throughout this document. Reproduced here for colocation:

| Class | Name | When it fires |
|---|---|---|
| A | Activation transition | A gate flips (G1-G9 flip, sunset extension, deployment event) |
| B | Hand-off boundary crossing | Phase boundary crossed; cross-agent hand-off event |
| C | Confidence-gate refusal | V3 NAM-03 confidence gate refuses a tier advancement |
| D | Four-surface drift | Vault / canonical / stcortex / devenv.toml diverge |
| E | Ancestor-rhyme | Planning-sprawl-never-built pattern; m15 pressure-notice accumulation |
| F | AP24 violation | `src/*.rs` files created before G9 explicit start-coding signal |
| G | Substrate-frame confusion | L9 substrate-frame engine scope infiltrating Phase A modules |
| H | Atuin proprioception anomaly | Cross-tool provenance gap; atuin history missing expected commands |
| I | Hebbian silence | Cluster H (m40-m42) not moving `learning_health` during deployment |

---

*Phase 6 + Cross-Cutting Concerns authored 2026-05-17 (S1001982). Planning-only. HOLD-v2 active. No code, no Cargo, no scaffold until G1-G9 gates clear and Luke emits explicit start-coding signal.*

*Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]] · [[phase-5-deploy-and-soak]]*
