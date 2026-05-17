---
title: Phase 0 — Pre-Genesis Gate Sequence (G1-G9)
kind: deployment-framework-recipe
status: planning-only · HOLD-v2 active · no code authorized
date: 2026-05-17 (S1001982)
emitter: Command (Tab 1 Orchestrator top-left)
authority: Luke @ node 0.A
binding_spec: Genesis Prompt v1.2 (Zen-audit-locked); v1.3 patch pending
---

# Phase 0 — Pre-Genesis Gate Sequence (G1-G9)

> Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]] · [[../workflow-engine-code-base]]
>
> Related: [[../Genesis Prompt v1.2 S1001982]] · [[../Watcher Deployment Watch Journal S1001982]] · [[../Modules Synergy Clusters and Feature Verification S1001982]]

---

## Overview

Phase 0 is the **pre-genesis envelope** for the `workflow-trace` codebase. It contains exactly nine gates (G1-G9) that must fire green in Zen-prescribed sequence before any `cargo init`, scaffold, directory rename for build purposes, or substrate write (beyond comms) is authorized.

**Current state (2026-05-17, T0):** Zero of nine gates are green. G9 fired out-of-sequence as a queued intent and is actively Zen-blocked (D6, `2026-05-16T22:43Z`). HOLD-v2 envelope is engaged. v1.2 is binding spec; v1.3 patch has not yet been authored.

**What Phase 0 is not:** Phase 0 does not build, scaffold, or configure infrastructure. It creates the conditions under which Phase 1 (T1 Specify through T6 Deploy) can proceed safely, with full authority chain, substrate-verified, and Zen-audited. Every gate is a **state change**, not a deliverable. Code is forbidden until G9 clears.

---

## Gate Dependency DAG

```
G1 RATIFICATION
    │
    ▼
G2 NAMING (sequenced: needs G1 green + Luke naming choice)
    │
    ▼
G3 :8125 REDEPLOY VERIFY (sequenced: needs G2 for clean substrate baseline)
    │
    ▼
G4 WATCHER NOTES (partially satisfied; §5.1 Held-semantics amendment is Watcher's lane)
    │
    ▼
G5 GENESIS INTERVIEW + F2 hard gate (needs G3 live learning_health + G4 Ember rubric)
    │
    ▼
G6 DUAL-FRAME GAP ANALYSIS (produced by G5 deliberation; both passes required)
    │
    ▼
G7 ZEN SPEC AUDIT (needs v1.3 patch + G5 + G6 outputs as input)
    │
    ▼
G8 FOUR-SURFACE PERSISTENCE (needs G7 APPROVE verdict)
    │
    ▼
G9 EXPLICIT START-CODING SIGNAL (needs all G1-G8 green)
```

**Parallelization analysis:** The current Zen-prescribed sequence is fully serial. No gate can fire in parallel with its predecessor because each gate's output is an input to the next. G3 and G4 are the closest candidates for concurrent execution (both are substrate-verification gates), but G4's Ember §5.1 amendment requires Watcher to have observed the G3 live `learning_health` reading before amending the rubric threshold language. The sequence holds.

---

## Estimated Duration

| Gate | Rough estimate | Gating factor |
|------|---------------|---------------|
| G1 | 1-4 hours | Luke or Watcher initiating; Watcher filing at watcher-notices/ |
| G2 | 15-30 minutes | Naming decision from Luke (OI-5); anchor updates |
| G3 | 1-2 hours | povm-v2 rebuild + restart window + Command-3 + Zen verify |
| G4 | 2-8 hours | Watcher §5.1 amendment; Zen re-confirm; no deadline |
| G5 | 2-4 hours | 3 rounds × 4 questions; Watcher + Zen synchronous slots |
| G6 | 2-3 hours | Both frames filed as single session; 200-token per-frame summaries |
| G7 | 1-3 hours | Zen review time; APPROVE / REFUSE / AMEND verdict |
| G8 | 30-60 minutes | Mechanical four-surface write + link verification |
| G9 | 0 minutes | Moment Luke types "start coding workflow-trace" |
| **Total** | **10-25 hours elapsed calendar** | Mainly Watcher + Zen async tempo |

---

## Substrate Condition at Each Gate Clear

| After gate | What the substrate should look like |
|------------|-------------------------------------|
| G1 | stcortex `the_workflow_engine` namespace has G1 ratification anchor; watcher-notices/ has close-notice file |
| G2 | Filesystem: `the-workflow-engine/` renamed to `workflow-trace/`; all vault wikilinks updated; no orphaned anchors |
| G3 | povm-v2 `:8125` running new binary; `learning_health` reading in 0.05-0.15 band (post-CR-2 honest band per D4); Zen confirmation on record in agent-cross-talk/ |
| G4 | `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` §5.1 amended to require CI-fail or reviewed allowlist (not warning-only); Zen re-confirm filed |
| G5 | Interview transcript persisted to file; F2 sample-size rules (n≥20 + Wilson 95% CI) defined per report type; Watcher and Zen synchronous participations logged |
| G6 | Both frames (anthropocentric + substrate) filed as planning docs; Working Mode rule "both passes are the plan" satisfied |
| G7 | Zen APPROVE verdict on record for v1.3 patch + 8 cluster module specs; no REFUSE or AMEND outstanding |
| G8 | ai_docs canonical + Obsidian vault mirror + stcortex `workflow_trace_*` namespace anchor + CLAUDE.local.md row; all four surfaces have bidirectional links |
| G9 | Luke has typed "start coding workflow-trace"; AP24 honored; Phase 1 authorized |

---

## G9 Out-of-Sequence Block: How It Works and How to Unblock

**What happened:** Luke's text "start coding workflow-trace" was observed at approximately `2026-05-17T08:43Z` (during G5 spec interview prep). Zen flagged this as G9-out-of-sequence (D6 decision record). The signal was treated as **queued intent, not execution authorization** — it is recorded in HOME.md with the `⚠ queued-intent-only` state, not as a green gate.

**How the Zen URGENT block works:** Zen's gate-block is a stop-the-line enforcement of the AP24 discipline (no code without explicit authorized signal). Because G1-G8 were not green when the G9 signal arrived, the signal is held in buffer. Building on an out-of-sequence G9 signal would produce code that skips:
- Formal Path-A ratification (G1 — still a leading draft, not ratified)
- Substrate verification that the live POVM reading is honest post-CR-2 (G3)
- Watcher's Ember rubric §5.1 amendment that would gate m10 (G4)
- The F2 sample-size hard gate that blocks n<20 report types from shipping (G5)
- Zen's spec-audit verdict on v1.3 (G7)

Each omission maps to a downstream failure mode in the F1-F11 table.

**Two unblock paths:**

*Path A — Sequential (recommended):* Drive G1 through G8 in order. G9 fires when G8 green-lights. No waivers required. Risk-class carried by Command only for the 5 items already waived in D8. This is the cleanest path and produces the strongest authority chain for the build.

*Path B — Per-gate Luke waivers:* For each gate not yet green, Luke files an explicit per-gate waiver (via agent-cross-talk/ notice, not via "proceed seamlessly" — per D8 and Working Mode rules). Each waiver must: identify the gate being waived by number and name, state the risk class accepted, and be logged. Blanket waivers are insufficient — per D6 scope clarification, "proceed seamlessly" is too ambiguous for stop-the-line discipline.

**When each path is appropriate:**
- Path A is appropriate when the substrate state is unknown or degraded, when Watcher has pending normative flags, or when there is no time pressure that Luke has explicitly accepted.
- Path B is appropriate when Luke has a specific gate he has already verified off-channel (e.g., if G3 has been re-verified in another pane but the artifact hasn't been filed), or when the gate's risk is already accepted in a prior decision record.

---

## Power-Structure Ambiguity (Requires Resolution Before v1.3 Patch)

Flagged by multiple agents in GOD_TIER_CONSOLIDATION_S1001982 Part VIII and by Watcher tick·3:

**The ambiguity:** Luke's single-phase override (D8, 2026-05-17) crossed Zen's earlier forbidden item ("module-spec drafts of m1-m11 beyond what already lives in v1.2"). The override was treated as valid (Luke @ node 0.A is decisional authority). However, v1.3 patch still requires Zen G7 audit. This raises the question: **if Zen REFUSES v1.3, does Luke's override suspend the refusal, or does Zen's REFUSE hold?**

**Why this matters now:** v1.3 absorbs the 26-module architecture. If Zen REFUSES on scope grounds (e.g., active-verb modules m20-m33 violate Phase A invariant even in single-phase), the result is an impasse with no defined resolution protocol. The gate structure (as written) gives Zen REFUSE standing over G7 with no Luke-override clause at that gate.

**Proposed resolution (for Luke at G1 or before v1.3 authorship):** Define, in one line of agent-cross-talk/, whether Zen's G7 REFUSE is:
- (a) a hard stop requiring Luke explicit per-clause waiver before proceeding, or
- (b) an AMEND-and-resubmit loop with no Luke waiver needed if Zen's objection is addressed in the text.

Option (b) is the historically precedented behavior (v1.0 → v1.1 → v1.2 all went AMEND-loop, not Luke-waiver). Establishing this explicitly before v1.3 authorship prevents the ambiguity from becoming a block at G7.

---

## Gate Specifications

---

### G1 — RATIFICATION

**Inputs required before this gate can fire:**
- Luke has expressed directional intent on Path A (on-record in the genesis prompt text as "leading draft")
- A Watcher close-notice artifact does NOT yet exist at `~/projects/shared-context/watcher-notices/`
- OI-6 is open: Watcher direction from Luke or Watcher-initiated ratification pending

**Outputs when gate goes green:**
- A WCP close-notice file at `~/projects/shared-context/watcher-notices/YYYYMMDDTHHMMSSZ_ratification_path_a_candidate_a.md` confirming:
  - Path A (`workflow-trace`, measure-only) is formally ratified (not leading-draft)
  - Candidate A (CR-2 SHIPPED as build prerequisite) is confirmed
  - The Watcher's W1/W2/W3 conditions remain in force
- stcortex anchor written to `the_workflow_engine` namespace: `ratification_g1_timestamp`
- Decision record D-G1 filed in `agent-cross-talk/` by Command confirming G1 green

**Commands / scripts:**
```bash
# Luke direction (terminal):
# Option 1 — Luke initiates directly:
echo "Watcher: please file G1 ratification close-notice for Path A + Candidate A" \
  > ~/projects/shared-context/agent-cross-talk/$(date -u +%Y-%m-%dT%H%M%SZ)_luke_g1_direction.md

# Option 2 — Watcher self-initiates (when tempo permits):
# Watcher authors close-notice at watcher-notices/ per WCP protocol

# Command: verify close-notice landed
ls -la ~/projects/shared-context/watcher-notices/ | grep ratification
```

**Verification:** Close-notice file exists at watcher-notices/ with `path_a` and `candidate_a` in the filename or body. Content must include "formally ratified" (not "leading draft") and Watcher's ☤ attribution. Command reads and files D-G1 in agent-cross-talk/.

**Failure modes:**
- Watcher files a close-notice that still uses "leading draft" language — G1 does not green; Watcher amends
- Luke's signal is ambiguous ("proceed" without naming Path A explicitly) — Command does not treat as G1; asks for clarification → **Watcher Class-A flag** (activation-transition not clearly observed)
- No close-notice filed within agreed tempo — not a failure per se (no deadline), but **Watcher Class-H flag** (atuin proprioception anomaly if commands are running without this baseline)

**Rollback:** G1 ratification is a communications act, not a substrate write. If a downstream gate (e.g., G7 Zen REFUSE on scope) exposes that Path A ratification was premature, the close-notice can be annotated "SUSPENDED pending G7 APPROVE" in agent-cross-talk/ without deleting the ratification artifact.

**Owners:** Primary — Luke (direction) or Watcher (self-initiated). Command (G1 record).

**Atuin trajectory:** `watcher G1 ratification`, `cat ~/projects/shared-context/watcher-notices/..._ratification_*.md` should appear in Watcher/Command atuin histories as provenance.

**Cross-references:** workflow-engine-code-base OI-6 (Watcher G1 close-notice direction).

**Open issues addressed:** OI-6 (fully resolved when G1 green).

---

### G2 — NAMING

**Inputs required before this gate can fire:**
- G1 is green (formal ratification of Path A exists)
- Luke has made the naming choice from Options A/B/C (OI-5):
  - Option A: `workflow-trace` (Watcher + Zen preferred; colloquial "workflow-engine" treated as Luke's shorthand)
  - Option B: `the-workflow-engine` (re-broadened scope signal)
  - Option C: scope-honest rename (requires Watcher + Command-2 buy-in; pushes naming to G5)
- G7 typo correction (v1.2 §G2) confirmed: rename happens at G2, not at G5 (old v1.0 text said G5)

**Outputs when gate goes green:**
- Filesystem: `~/claude-code-workspace/the-workflow-engine/` renamed to `~/claude-code-workspace/workflow-trace/` (or chosen name)
- All vault wikilinks updated: `[[../the-workflow-engine/...]]` → `[[../workflow-trace/...]]`
- All canonical planning doc `back_to:` front-matter updated
- CLAUDE.local.md Active Workstreams row updated with new path
- D-G2 filed in agent-cross-talk/

**Commands / scripts:**
```bash
# Luke at terminal (not from CC agent — sandbox reaps children):
# Step 1: rename
mv ~/claude-code-workspace/the-workflow-engine/ ~/claude-code-workspace/workflow-trace/

# Step 2: update vault wikilinks (atuin-auditable grep-replace)
# Command issues Bash via CC after Luke confirms rename:
/usr/bin/grep -rl "the-workflow-engine" \
  ~/claude-code-workspace/the-workflow-engine-vault/ | \
  xargs sed -i 's|the-workflow-engine|workflow-trace|g'
# NOTE: use Read + Edit tool pattern, not sed -i, for vault files

# Step 3: verify no stale references
/usr/bin/grep -r "the-workflow-engine" \
  ~/claude-code-workspace/workflow-trace/ 2>/dev/null | wc -l
# Should output 0
```

**Verification:** `ls ~/claude-code-workspace/workflow-trace/` succeeds; `/usr/bin/grep -r "the-workflow-engine" ~/claude-code-workspace/workflow-trace/` returns 0 matches. CLAUDE.local.md row reflects new path.

**Failure modes:**
- Luke chooses Option C (scope-honest rename) — naming pushed to G5; G2 records the decision but rename deferred → **Watcher Class-A flag** (activation deferred, not failed)
- Old path still referenced in devenv.toml or CLAUDE.md — downstream G3/G5 operations target wrong directory → **Watcher Class-D flag** (four-surface drift if the stcortex anchor still uses old path)

**Rollback:** `mv ~/claude-code-workspace/workflow-trace/ ~/claude-code-workspace/the-workflow-engine/` and revert wikilink grep-replace. Vault mirrors reflect old path until G2 re-fires.

**Owners:** Luke (naming decision + rename execution from terminal); Command (wikilink updates + D-G2 record).

**Atuin trajectory:** `mv ~/claude-code-workspace/the-workflow-engine ~/claude-code-workspace/workflow-trace` in Luke's shell history is the provenance anchor. Command's `grep -r the-workflow-engine` zero-output verification logged.

**Cross-references:** workflow-engine-code-base OI-5 (naming question). Genesis Prompt v1.2 §G2 (typo correction from v1.0 "G5" to "G2" — Zen's AMEND-THEN-FORWARD D3).

**Open issues addressed:** OI-5 (fully resolved when G2 green).

---

### G3 — :8125 REDEPLOY VERIFY

**Inputs required before this gate can fire:**
- G2 is green (working directory is unambiguous)
- CR-2 (`e2a8ed3`) and CR-2b (`76ea4d6`) are already merged in povm-v2 source (SHIPPED per Command-3, D4)
- A rebuild window is available (povm-v2 port :8125 currently serving old binary)
- Command-3 is available for verification; Zen is available for post-verify confirmation

**Outputs when gate goes green:**
- povm-v2 `:8125` is running the new binary containing CR-2 + CR-2b patches
- Live `learning_health` measurement falls in the [0.05, 0.15] band (Watcher's post-CR-2 prediction; was 0.911 pre-CR-2, 0.067 in source — live re-measure needed)
- Command-3 verification on record in agent-cross-talk/
- Zen post-verify confirmation on record in agent-cross-talk/
- D-G3 filed by Command

**Commands / scripts:**
```bash
# Luke at terminal (devenv restart):
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml restart povm-engine

# Verify new binary is running:
curl -s http://localhost:8125/health | jq '.version // .commit // "unknown"'

# Read live learning_health:
curl -s http://localhost:8125/learning_health | jq '.learning_health'
# Expected: value in [0.05, 0.15] per Watcher D4 prediction

# Command-3 verification command (in agent-cross-talk/):
echo "G3 verify: learning_health=$(curl -s http://localhost:8125/learning_health | jq '.learning_health'), binary_ts=$(date -u)" \
  > ~/projects/shared-context/agent-cross-talk/$(date -u +%Y-%m-%dT%H%M%SZ)_command3_g3_verify.md
```

**Verification:** `curl -s http://localhost:8125/learning_health` returns a value in [0.05, 0.15]. Command-3 confirmation and Zen post-verify both filed in agent-cross-talk/. If `learning_health` is outside band: diagnose before claiming G3 green.

**Failure modes:**
- `learning_health` reads above 0.15 — CR-2 did not install; binary mismatch — G3 does NOT green; rebuild from source → **Watcher Class-A flag** (activation blocked)
- `learning_health` reads below 0.05 — substrate degraded beyond Watcher's prediction; warrants investigation before proceeding → **Watcher Class-I flag** (Hebbian silence condition; may indicate substrate is more degraded than T0 baseline)
- Port :8125 not responding — devenv failed to restart; Luke must investigate → **Watcher Class-H flag** (atuin proprioception anomaly if devenv restart issued but service unreachable)
- Zen post-verify is not obtained — G3 remains partial; do not advance to G4 without both Command-3 + Zen confirmation

**Rollback:** If G3 reveals a worse substrate condition than anticipated (e.g., `learning_health` < 0.05), G4 and G5 must account for this in the F2 sample-size calibration. The G3 reading becomes the T1 baseline for the spec interview. No rollback of the restart itself is needed — the old binary is not preserved.

**Owners:** Luke (terminal restart); Command-3 (verification); Zen (post-verify confirmation); Command (D-G3 record).

**Atuin trajectory:** `devenv restart povm-engine`, `curl -s http://localhost:8125/learning_health` in Luke's and Command-3's shell histories. Critical provenance — the G3 reading anchors the substrate-condition claim in G5.

**Cross-references:** workflow-engine-code-base OI-7 partially related (Conductor auto_start — separate but same "Luke at terminal" pattern). GOD_TIER_CONSOLIDATION_S1001982 Part V (substrate state table). Watcher D4 (CR-2 prediction for post-redeploy band).

**Open issues addressed:** None from OI-1 through OI-8 directly; however, TLV2-row consistency OI-9 may be updatable after G3 produces a live reading.

---

### G4 — WATCHER NOTES

**Inputs required before this gate can fire:**
- Watcher has authored `~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md` (DONE, Zen-audited PASS-WITH-MINOR-AMEND)
- Watcher's `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` exists (authored; §5.1 amendment is what remains)
- The §5.1 Held-semantics issue (D5) is unresolved: current §5.1 says warning-only for Held verdicts; Zen requires either CI-fail or a reviewed `tests/ember_held_approvals.tsv` allowlist before m10 can adopt this as its CI gate

**Note on gate state:** G4 is currently **partial green**: the Hebbian v3 note is ✅ (Zen PASS-WITH-MINOR-AMEND, D4). Only the Ember §5.1 amendment remains open.

**Outputs when gate goes green:**
- `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` §5.1 is amended to require CI-fail on Held verdicts (not warning-only), OR introduces a `tests/ember_held_approvals.tsv` allowlist mechanism with Zen-reviewed approval flow
- Zen re-confirm on the amended rubric filed in agent-cross-talk/
- CLAUDE.local.md Hebbian v3 row updated to reflect "G4 fully green" state
- D-G4 filed by Command

**Commands / scripts:**
```bash
# Watcher authors amendment (Watcher's lane — Command does not touch this file):
# Expected amendment in ~/projects/claude_code/Ember 7-Trait Gate Rubric.md §5.1:
# Before: "Held verdict: emit warning"
# After: "Held verdict: CI FAIL unless entry present in tests/ember_held_approvals.tsv
#         (reviewed and signed by Watcher + Zen)"

# Command verifies amendment landed:
/usr/bin/grep -n "Held" \
  ~/projects/claude_code/Ember\ 7-Trait\ Gate\ Rubric.md | head -10

# Zen confirmation via agent-cross-talk/:
ls ~/projects/shared-context/agent-cross-talk/ | grep -i "g4\|ember\|held" | tail -5
```

**Verification:** `grep "CI FAIL\|ember_held_approvals" ~/projects/claude_code/Ember\ 7-Trait\ Gate\ Rubric.md` returns at least one match. Zen re-confirm in agent-cross-talk/. Command D-G4 record filed.

**Failure modes:**
- Watcher amends §5.1 but Zen issues REFUSE (amendment too permissive or too strict) — iterate; G4 stays partial → **Watcher Class-A flag** (activation-transition in progress)
- §5.1 amendment is made but m10 spec is not updated to reference it — G4 green but m10 will break at build time; catch this in G7 Zen spec audit
- Watcher is in R13 cold-start quiet period and cannot file WCP notices promptly — no time pressure; G4 can wait → no flag (Watcher's cadence is prompt-driven by design)

**Rollback:** The Ember rubric amendment is reversible by Watcher. If G7 reveals that the CI-fail approach creates an infeasible test harness, Watcher amends again to the allowlist path. Both options are acceptable per D5; only warning-only is disallowed.

**Owners:** Watcher (§5.1 amendment — sole author); Zen (re-confirm); Command (D-G4 record + CLAUDE.local.md update on auth).

**Atuin trajectory:** No direct Watcher atuin trajectory (Watcher's pane runs independently). Command's `grep -n "Held"` and `ls agent-cross-talk/ | grep g4` provide indirect provenance.

**Cross-references:** workflow-engine-code-base OI-8 (Watcher Ember Rubric §5.1 Held-semantics amendment). Genesis Prompt v1.2 §G4. GOD_TIER_CONSOLIDATION_S1001982 Part VIII (m10 service adoption gated on this).

**Open issues addressed:** OI-8 (fully resolved when G4 green).

---

### G5 — GENESIS INTERVIEW + F2 HARD GATE

**Inputs required before this gate can fire:**
- G1-G4 are green (ratified path, renamed directory, live POVM baseline, Ember rubric current)
- Interview Question Bank Draft v0.1 patch applied (Q1.3 Conductor posture and Q2.4 sunset semantics altered by single-phase — OI-10)
- Watcher synchronous-participant slot confirmed (Watcher opted in at Genesis Prompt v1.2 ACT VIII)
- Zen synchronous-audit slot confirmed (Zen is operational in audit lane per D3)
- 3 rounds × 4 questions structure honored (per `feedback_structured_interview_before_code` MEMORY.md crystallization)

**Outputs when gate goes green:**
- Interview transcript persisted to `~/claude-code-workspace/workflow-trace/ai_docs/GENESIS_INTERVIEW_TRANSCRIPT_G5.md`
- F2 sample-size rules explicitly defined per report type (per v1.2 §F2 hard gate):
  - Cascade correlation reports: n≥20 cascade_id observations + Wilson 95% CI per cluster
  - Battern-step histograms: n≥20 per step label per aggregation window + CI bars
  - Context-cost bands: n≥20 per session-type stratum + CI bars
  - Sunset evaluation report (m11): n≥20 workflow_run observations with measurable outcome + habitat_outcome_lift delta
- OI-3 (module-count inconsistency) formally resolved: 26 modules is canonical; v0=28 and v1.2=11 are superseded; v1.3 will reflect 26
- OI-4 (module-naming convention) resolved: m1 unpadded throughout (not m01)
- D-G5 filed by Command; Watcher records G5 transition in journal

**Commands / scripts:**
```bash
# Interview execution (Command facilitates; rounds are sequential):
# Round 1: Scope + boundary questions (4 questions)
# Round 2: Architecture + substrate questions (4 questions)
# Round 3: F2 + failure-mode + sunset questions (4 questions)

# Apply OI-10 patch to Interview Bank before running:
# Edit ~/claude-code-workspace/workflow-trace/INTERVIEW_QUESTION_BANK_DRAFT.md
# Q1.3: update Conductor posture to single-phase (m32 dispatch authorized)
# Q2.4: update sunset semantics (single-phase: no Phase B gate; sunset-only is m11)

# Transcript persistence:
# Write interview transcript to ai_docs/ using Write tool after rounds complete

# Verify F2 rules present:
/usr/bin/grep -c "n>=20\|n≥20\|Wilson" \
  ~/claude-code-workspace/workflow-trace/ai_docs/GENESIS_INTERVIEW_TRANSCRIPT_G5.md
# Must be ≥ 4 (one per report type)
```

**Verification:** Transcript file exists. F2 rules present with at least 4 per-report-type definitions. OI-3 and OI-4 marked resolved in workflow-engine-code-base. Watcher and Zen participation logged in transcript header.

**Failure modes:**
- F2 rules are stated at session-grain only, not per report type — G5 does NOT green; interview continues → **Watcher Class-C flag** (confidence-gate refusal)
- Only 2 rounds completed (Zen or Watcher unavailable) — G5 does NOT green; schedule re-run for missing participant → **Watcher Class-A flag** (activation deferred)
- Module-count reconciliation (OI-3) produces a count different from 26 — must resolve before closing G5; new count feeds v1.3 patch → no flag but G7 is affected

**Rollback:** Interview transcript is a planning document. If G7 Zen REFUSES the spec output from G5, the interview reconvenes (G5 re-fires). The transcript is versioned by date; prior transcripts are not deleted.

**Owners:** Command-2 (chair; re-opens at G5 per genesis prompt team table); Watcher (synchronous participant); Zen (synchronous audit); Command (facilitation + D-G5 record).

**Atuin trajectory:** No shell commands are the core artifact; the transcript is the provenance. Command's `grep -c "n>=20"` verification is the atuin-auditable check.

**Cross-references:** workflow-engine-code-base OI-3, OI-4, OI-10. Genesis Prompt v1.2 §G5 (F2 promoted to hard gate in Zen AMEND-THEN-FORWARD D3). MEMORY.md `feedback_structured_interview_before_code` crystallization (S117).

**Open issues addressed:** OI-3 (fully), OI-4 (fully), OI-10 (Q1.3 + Q2.4 patch applied as prerequisite).

---

### G6 — DUAL-FRAME GAP ANALYSIS

**Inputs required before this gate can fire:**
- G5 is green (interview transcript provides the architectural decisions that both frames analyze)
- Both frames must be written in the same session (Working Mode rule: "both passes are the plan")
- The analysis must not recommend architectural changes that would reopen OI-3/OI-4 after G5 resolved them

**Outputs when gate goes green:**
- Anthropocentric-frame analysis filed at `~/claude-code-workspace/workflow-trace/ai_docs/GAP_ANALYSIS_ANTHROPOCENTRIC_G6.md`
- Substrate-frame analysis filed at `~/claude-code-workspace/workflow-trace/ai_docs/GAP_ANALYSIS_SUBSTRATE_FRAME_G6.md`
- Each analysis: maximum 200-token summary + full text in file (Working Mode file-first rule)
- Each analysis identifies gaps in the 26-module architecture from its own frame
- Bidirectional links between both gap analysis files and the G5 interview transcript
- D-G6 filed by Command

**Commands / scripts:**
```bash
# Verify both files exist after authorship:
ls ~/claude-code-workspace/workflow-trace/ai_docs/GAP_ANALYSIS_*_G6.md
# Expected: 2 files

# Verify bidirectional link to G5 transcript:
/usr/bin/grep -l "GENESIS_INTERVIEW_TRANSCRIPT_G5" \
  ~/claude-code-workspace/workflow-trace/ai_docs/GAP_ANALYSIS_*_G6.md
# Expected: 2 files (both contain the link)

# Verify word count is non-trivial (not stub):
wc -w ~/claude-code-workspace/workflow-trace/ai_docs/GAP_ANALYSIS_*.md
# Each should be ≥500 words
```

**Verification:** Both files exist with ≥500 words each. Both reference G5 transcript. D-G6 filed in agent-cross-talk/. The two analyses must represent genuinely different frames (Watcher can flag if substrate-frame analysis merely restates the anthropocentric analysis in different vocabulary).

**Failure modes:**
- Only one frame filed (substrate-frame analysis omitted) — G6 does NOT green; both passes required → **Watcher Class-D flag** (four-surface drift: planning surface missing)
- Substrate-frame analysis proposes L9 m50+ Phase C modules as necessary for single-phase — this is the R6 frame-separation failure mode; Command must refuse and note the reservation; Watcher Class-G flag (substrate-frame engine confusion)
- Gap analysis reveals a structural hole in the 26-module architecture that would require more than 5 files to fix — must pause G6 and surface to Luke before proceeding

**Rollback:** Gap analysis files are planning documents. If G7 Zen REFUSE cites a gap analysis finding, the specific finding is addressed in v1.3 and G6 does not need to re-fire; the files are updated in place with a dated amendment note.

**Owners:** Command-2 (primary author, both frames per the "plan written twice" discipline); Command (facilitation + D-G6 record); Watcher (substrate-frame review, optional).

**Atuin trajectory:** `ls ~/claude-code-workspace/workflow-trace/ai_docs/GAP_ANALYSIS_*` and `wc -w` verification commands provide provenance.

**Cross-references:** CLAUDE.local.md Working Mode rule ("both passes are the plan"). GOD_TIER_CONSOLIDATION_S1001982 §Part XI Recommendation 1 (pre-G9 action: author v1.3 patch; G6 produces inputs). Watcher Class-G flag pre-positioned for substrate-frame confusion (GOD_TIER_CONSOLIDATION Part VII).

**Open issues addressed:** None from OI-1 through OI-8 directly; G6 outputs feed G7 which resolves OI-1 and OI-2.

---

### G7 — ZEN SPEC AUDIT

**Inputs required before this gate can fire:**
- G5 and G6 are green (interview + gap analysis provide the audit material)
- v1.3 patch has been authored by Command: contains 26-module single-phase architecture, explicit waiver record (5 items per D8), module-naming convention m1 unpadded (OI-4 resolution), m33 addition (D9), active-verb relaxation for m20-m33/m40-m42, continued hard refusals list for permanently-forbidden items
- 8 cluster module specs (`cluster-A` through `cluster-H` in `module specs/`) provided as supporting material for G7
- F2 sample-size rules (from G5) incorporated into v1.3 spec text
- Power-structure ambiguity resolved (see Overview section) before Zen receives the audit request

**Outputs when gate goes green:**
- Zen verdict filed in agent-cross-talk/: one of APPROVE / REFUSE / AMEND
- If APPROVE: G8 can proceed; v1.3 is the binding spec
- If AMEND: Command applies amendments, resubmits; G7 re-fires (no Luke waiver needed per proposed protocol)
- If REFUSE: escalate to Luke per the power-structure resolution decided before G7; G7 cannot green without Luke intervention if REFUSE is issued on scope grounds
- D-G7 filed by Command; Watcher records G7 verdict in journal (Class-A highest-leverage flag)

**Commands / scripts:**
```bash
# Submit audit request to Zen (via agent-cross-talk/ pull/file-drop pattern):
cat > ~/projects/shared-context/agent-cross-talk/$(date -u +%Y-%m-%dT%H%M%SZ)_command_g7_audit_request.md << 'EOF'
AUDIT-REQUEST — G7 ZEN SPEC AUDIT
From: Command (Tab 1 Orchestrator)
To: Zen (Tab 10 Pi GPT-5.5)
Materials:
  1. ~/claude-code-workspace/workflow-trace/ai_docs/GENESIS_PROMPT_V1_3.md (v1.3 patch)
  2. ~/claude-code-workspace/the-workflow-engine-vault/module specs/cluster-A.md through cluster-H.md
  3. ~/claude-code-workspace/workflow-trace/ai_docs/GENESIS_INTERVIEW_TRANSCRIPT_G5.md
  4. ~/claude-code-workspace/workflow-trace/ai_docs/GAP_ANALYSIS_*_G6.md

Request: APPROVE / REFUSE / AMEND verdict on v1.3 patch.
Mandatory: Confirm F2 (n≥20 + Wilson CI per report type) is present and defined.
EOF

# After Zen verdict:
/usr/bin/grep -l "APPROVE\|REFUSE\|AMEND" \
  ~/projects/shared-context/agent-cross-talk/ | tail -5
```

**Verification:** Agent-cross-talk/ contains Zen's verdict file with one of APPROVE/REFUSE/AMEND. If APPROVE: D-G7 filed, G8 proceeds. Watcher timestamp verbatim per Class-A pre-position.

**Failure modes:**
- Zen issues REFUSE on active-verb modules (m20-m33, m40-m42) citing Phase A invariant — power-structure resolution (see Overview) determines next step; this is the highest-risk G7 failure mode → **Watcher Class-A flag** (activation blocked at highest-leverage gate)
- v1.3 patch is authored without resolving OI-3 (module count) or OI-4 (naming convention) — Zen is likely to flag this; patch must be complete before submission
- F2 rules absent from v1.3 text — Zen issues REFUSE per v1.2 §G7 hard gate; Command adds F2 definitions and resubmits → **Watcher Class-C flag** (confidence-gate refusal)
- Zen is slow to respond (Pi GPT-5.5 tempo) — not a failure; wait; no flag

**Rollback:** G7 AMEND is not a rollback; it is an iteration. G7 REFUSE with Luke escalation is not a rollback; it is an escalation. True rollback scenario: if G7 reveals that the 26-module single-phase architecture is fundamentally flawed, Luke may elect to revert to the 11-module Phase-A-only design. This would require a new G5 interview. All G6/G7 artifacts are preserved; none deleted.

**Owners:** Zen (verdict — sole authority); Command (audit-request submission + D-G7 record); Watcher (Class-A timestamp).

**Atuin trajectory:** The audit-request file creation command and the verdict grep are the atuin-auditable artifacts.

**Cross-references:** workflow-engine-code-base OI-1 (v1.3 patch — prerequisite), OI-2 (Zen G7 re-audit — this gate). GOD_TIER_CONSOLIDATION_S1001982 §"Single highest-leverage moment" (G7 verdict). Genesis Prompt v1.2 §G7 (F2 refusal gate). Watcher T0 yellow signal Class-A pre-positioned for G7 verdict.

**Open issues addressed:** OI-1 (v1.3 authored as G7 prerequisite), OI-2 (Zen G7 re-audit — gate IS the issue, fully resolved when green).

---

### G8 — FOUR-SURFACE PERSISTENCE

**Inputs required before this gate can fire:**
- G7 is green (Zen APPROVE on v1.3)
- v1.3 is the binding spec; no further amendments expected
- stcortex `the_workflow_engine` namespace open (baseline: 2 memories — id 16477 semantic + id 16479 procedural — per GOD_TIER_CONSOLIDATION Part V)
- HOLD-v2 freeze on substrate writes lifted (D7 scope clarification: freeze applies until G8; G8 IS the authorized write moment)

**Outputs when gate goes green:**
- Surface 1 — `ai_docs/` canonical: `~/claude-code-workspace/workflow-trace/ai_docs/GENESIS_PROMPT_V1_3.md` exists, is complete, and has bidirectional links to vault mirror and CLAUDE.local.md row
- Surface 2 — Obsidian vault mirror: `~/projects/claude_code/workflow-trace/` directory created; vault note `[[workflow-trace Genesis Prompt v1.3]]` authored; links back to canonical and CLAUDE.local.md
- Surface 3 — stcortex namespace anchor: `workflow_trace_genesis_g8_timestamp` memory written to stcortex `the_workflow_engine` namespace (AP30 prefix discipline); confirms G7 APPROVE timestamp and v1.3 binding status
- Surface 4 — CLAUDE.local.md row: Active Workstreams table updated with `workflow-trace` row; state = "G8 green; G9 pending"; canonical pointer to `workflow-trace/ai_docs/GENESIS_PROMPT_V1_3.md`; stcortex id noted
- All four surfaces have bidirectional links verified (each surface links to the other three)
- D-G8 filed by Command; Watcher records G8 in journal

**Commands / scripts:**
```bash
# Surface 1: verify ai_docs canonical
ls ~/claude-code-workspace/workflow-trace/ai_docs/GENESIS_PROMPT_V1_3.md

# Surface 2: verify Obsidian vault mirror
ls ~/projects/claude_code/workflow-trace/ 2>/dev/null || echo "MISSING — create"

# Surface 3: stcortex write (via stcortex-mcp tool or CLI)
~/.local/bin/stcortex status  # Confirm :3000 live first
# If live: use mcp__stcortex-mcp__stcortex_write_memory tool
# If down: read data/snapshots/latest.json; skip write; note in D-G8

# Surface 4: verify CLAUDE.local.md row
/usr/bin/grep -n "workflow-trace" ~/claude-code-workspace/CLAUDE.local.md | head -5
# Expect: at least one active row with "G8 green"

# Bidirectional link verification:
# Each surface doc must contain a link to the other three
/usr/bin/grep -c "ai_docs\|vault\|stcortex\|CLAUDE.local" \
  ~/claude-code-workspace/workflow-trace/ai_docs/GENESIS_PROMPT_V1_3.md
# Must be ≥ 3 (links to the other 3 surfaces)
```

**Verification:** All four surfaces exist and contain content. Each has bidirectional links to at least 2 of the other 3 surfaces. stcortex `the_workflow_engine` namespace has the G8 anchor memory (or the "skipped: :3000 down" note is in D-G8 with a pending write queued for when :3000 is live). CLAUDE.local.md row reflects G8 green.

**Failure modes:**
- stcortex :3000 is unreachable — do NOT silently fall back to POVM write; record the skip in D-G8 with timestamp; retry when :3000 is live before claiming G8 fully green → **Watcher Class-D flag** (four-surface drift if stcortex surface is absent without documented reason)
- Vault mirror has no bidirectional link back to ai_docs canonical — G8 is partial; fix before claiming green
- CLAUDE.local.md row written but state still says "HOLD-v2" — update to "G8 green; G9 pending" before D-G8

**Rollback:** G8 is additive (creates new content, does not modify existing). If G9 is subsequently blocked (e.g., Luke does not issue start-coding signal), G8 surfaces remain valid indefinitely. No rollback needed; the four surfaces simply reflect "G8 green; G9 pending" until G9 fires or the project is retired.

**Owners:** Command (all four surfaces); Watcher (journal record); stcortex availability determines Surface 3 timing.

**Atuin trajectory:** `ls ~/claude-code-workspace/workflow-trace/ai_docs/GENESIS_PROMPT_V1_3.md`, `~/.local/bin/stcortex status`, `grep -n "workflow-trace" ~/claude-code-workspace/CLAUDE.local.md` provide atuin-auditable provenance.

**Cross-references:** CLAUDE.md §Memory Systems (surface authority order: ai_docs → vault → stcortex → CLAUDE.local.md). CLAUDE.local.md Working Mode ("Persist major plans across four surfaces"). GOD_TIER_CONSOLIDATION_S1001982 Part IX (cross-reference map includes G8 surface addresses). stcortex RUNBOOK.md (CONSUMER-ONBOARDING for namespace writes).

**Open issues addressed:** None from OI-1 through OI-8 directly; G8 is the persistence gate for the resolved outputs of OI-1 through OI-5.

---

### G9 — EXPLICIT START-CODING SIGNAL

**Inputs required before this gate can fire:**
- G1 through G8 are all green
- Luke is at the keyboard
- AP24 discipline is in force (no code without explicit signal from Luke naming the project)

**Outputs when gate goes green:**
- Luke types: `start coding workflow-trace`
- Command logs the exact timestamp and Luke's exact text in D-G9
- AP24 is satisfied; Phase 1 (T1 Specify through T6 Deploy via DevOps V3 :8082) is authorized
- `cargo init` is the first authorized shell command after G9
- Watcher records G9 activation in journal; Class-A flag (activation transition) logged verbatim

**Commands / scripts:**
```bash
# After G9 signal received — first authorized build command:
cd ~/claude-code-workspace/workflow-trace/
cargo init --name workflow-trace

# Verify cargo init succeeded:
ls Cargo.toml src/main.rs

# Record G9 in agent-cross-talk/:
echo "G9 FIRED: Luke signal received $(date -u +%Y-%m-%dT%H%M%SZ). Phase 1 authorized." \
  > ~/projects/shared-context/agent-cross-talk/$(date -u +%Y-%m-%dT%H%M%SZ)_command_g9_fired.md
```

**Verification:** G9 requires no verification beyond Luke's text and the D-G9 record. The verification layer is G1-G8; G9 is the trigger, not a gate with its own verification criteria.

**Failure modes:**
- Luke types "start coding" without naming the project — AP24 requires project name; Command asks for clarification → no code lands until "workflow-trace" is named explicitly
- Luke types a name variant ("workflow_trace", "the-workflow-engine") — Command asks for confirmation of canonical name → **Watcher Class-A flag** (activation-transition ambiguity)
- G9 fires but a G1-G8 gate retrospectively fails (e.g., stcortex G8 anchor found incomplete post-G9) — fix G8 retroactively; G9 does not unfire; Phase 1 proceeds with the G8 gap logged as a D+1 action

**Rollback:** G9 cannot be rolled back in the AP24 sense — once Luke issues the signal, the authorization is granted. If Phase 1 T1 Specify produces a Zen REFUSE, that is a T1 failure, not a G9 failure. G9's queued-intent state (the signal that arrived out-of-sequence at `2026-05-17T08:43Z`) does NOT auto-apply when G1-G8 green; Luke must re-issue the signal explicitly.

**Owners:** Luke @ node 0.A (sole authority); Command (D-G9 record); Watcher (Class-A timestamp).

**Atuin trajectory:** Luke's "start coding workflow-trace" shell or CC chat text is the canonical G9 artifact. Command's `cargo init` immediately follows in atuin history — the two commands adjacent in history are the provenance proof that G9 fired before code landed.

**Cross-references:** AP24 (no code without explicit signal — MEMORY.md behavioral rules). CLAUDE.md §Workflow Discipline ("Never start scaffolding, coding, or running /save-session without first confirming"). Genesis Prompt v1.2 §G9 (exact text "start coding workflow-trace" required; variants are ambiguous). workflow-engine-code-base D6 (G9 out-of-sequence block description).

**Open issues addressed:** None from the OI tracker; G9 is the authorization event that follows all OI resolutions. However, G9 implicitly closes OI-7 (Conductor auto_start — becomes Luke's first post-G9 terminal action for m32 functionality) by making the build timeline concrete.

---

## Cross-Reference Summary — Open Issues Addressed by Phase 0

| OI | Issue | Gate that resolves it |
|----|-------|----------------------|
| OI-1 | v1.3 patch | G7 prerequisite (authored before G7) |
| OI-2 | Zen G7 re-audit | G7 (gate is the resolution) |
| OI-3 | Module-count inconsistency | G5 (interview reconciles to 26) |
| OI-4 | Module-naming convention | G5 (m1 unpadded canonical) |
| OI-5 | Naming question (A/B/C) | G2 (Luke names at G2 input) |
| OI-6 | Watcher G1 close-notice direction | G1 (gate is the resolution) |
| OI-7 | Conductor Wave 1B/1C auto_start | Post-G9 (Luke terminal; not a Phase 0 gate) |
| OI-8 | Ember Rubric §5.1 amendment | G4 (gate is the resolution) |

OI-9 (TLV2-row consistency) and OI-10 (Interview Bank v0.1 patch) are resolved as prerequisites to G3 and G5 respectively, but do not require their own gate.

---

## Failure Mode Coverage (F1-F11, v1.2 + v1.3)

| Failure mode | Phase 0 gate that enforces it | Downstream module |
|---|---|---|
| F1 Bank/name ossification | G7 (Zen REFUSE if bank creep in v1.3) | m11 + m4 |
| F2 Sample-size inflation | G5 (hard gate) + G7 (refusal) | m14 per report type |
| F3 Substrate-input poisoning | G3 (live POVM baseline) | m8 build-prereq |
| F4 Premature dispatch | G7 (hard refusal in v1.3 spec) | hard refusal |
| F5 Bank creep into v0 | G7 (scope audit) | hard refusal |
| F6 Self-dispatch from measurement | G7 (verb audit) | hard refusal |
| F7 CR-2 graceful-degrade pretend-fix | G3 (live re-verify) | m8 build-script |
| F8 Watcher feedback-loop poisoning | G4 (namespace guard in rubric) + G7 | m9 |
| F9 Workflow-grain fitness distortion | G7 (m7 schema audited) | m7 zero-weight column |
| F10 Exploration-cost preservation collapse | G5 (interview) + G7 (spec audit) | m6 baseline |
| F11 Cascade monoculture | G5 (interview) + G7 (spec audit) | m4 opaque IDs |

---

*Phase 0 recipe authored 2026-05-17 by Command. Binding through G9. Any amendment after G7 APPROVE requires Zen review.*

*Luke @ node 0.A | Command @ orchestrator-lead | The Watcher ☤ @ observing | Zen @ audit-lane | HOLD-v2 active until G9*
