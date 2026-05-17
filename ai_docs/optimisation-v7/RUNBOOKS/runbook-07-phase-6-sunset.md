---
title: Runbook 07 — Phase 6 D120 Sunset Evaluation
date: 2026-05-17 (S1001982)
kind: planning-only · operational runbook · terminal-gate of workflow-trace
phase: 6 of 6
trigger: D120 reached AND `workflow_trace.soak.d120.verdict` KV present
owner: Luke @ node 0.A (decisional authority) + m11 (binary-level enforcement) + Watcher (synthesis) + Zen (audit) + Command (orchestration)
deliverables: PASS/FAIL/DEGRADED verdict · retirement ceremony OR continuation extension · IC-N improvement candidate list
m11_immutability: sunset_at encoded at deploy time NOT runtime
authority: Luke @ node 0.A
status: planning-only · HOLD-v2 active · NOT executable until G1-G9 GREEN + Phase 5C close
---

# Runbook 07 — Phase 6 D120 Sunset Evaluation

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · prev [[runbook-06-phase-5-deploy-soak.md]] · sibling [[runbook-10-cross-cutting.md]] · [[runbook-11-emergency-rollback.md]]
>
> Source phase doc: [[../../the-workflow-engine-vault/deployment framework/phase-6-sunset-and-cross-cutting.md]] (covers D120 decision tree + retirement ceremony + 7 cross-cutting concerns; this runbook extracts the D120 operational core)

---

## Overview

Phase 6 is the **terminal evaluation gate** of the workflow-trace deployment. At D120 (120 days from first dispatch event registered in m7), the m11 lifecycle module evaluates `habitat_outcome_lift` and emits one of four verdicts: **PASS** (lift ≥ threshold, n ≥ 20, CI lower bound > 0, Cluster H active) → continuation with 180-day review cycle; **FAIL** (lift below threshold OR n < 20 OR CI spans zero) → m11 startup-refusal fires, engine retires; **DEGRADED** (mixed signal — meets threshold narrowly but Class-I Hebbian silence) → Luke decides fix-or-retire; **INSUFFICIENT_DATA** (n < 20 still) → soak extension. Sunset is **honest, not punitive** — the law exists because the two ancestor codebases (`loop-workflow-engine-project`, `habitat-loop-engine`) died from open-ended scope accumulation with no terminal gate. Watcher issues a Fossil-rhyme prevention verdict (AVERTED / PARTIAL RHYME / RHYME CONFIRMED).

---

## Pre-flight checklist (T-7 days before D120)

Run one week before the sunset deadline. All items must complete before Step 1 (D120 evaluation) fires.

- `workflow_trace.soak.d90.lift_value` + `lift_ci_lower` + `lift_n` populated in atuin KV (Phase 5C D90 preview ran)
- `wf-crystallise evidence-snapshot --window-days 120` produces non-empty `LiftSnapshot`
- Watcher synthesises the deployment-watch journal: count of Class-I (Hebbian silence), Class-E (ancestor-rhyme), Class-D (four-surface drift) flags fired across the 90-day soak
- Zen audits Watcher synthesis for Ember rubric compliance and factual correctness (no Held-as-fail per W3 unless §5.1 amendment landed)
- m11 bank state snapshot: count of ACTIVE / PRUNE_PENDING / SUNSET_EXPIRED workflows
- POVM cutover post-state confirmed (`workflow_trace.povm_overlap_active = false` since ~D25; stcortex sole substrate)
- HABITAT-CONDUCTOR still live (`:8141/health = 200`) — needed if any dispatches happen during evaluation window
- WCP notice from Watcher to Command: "D120 synthesis prepared; T-7 days; Luke decision input ready"
- Git tag `v0.1.0-soak-D120-T-7` placed for evidence preservation

If any item misses: HALT. Phase 6 evaluation does NOT proceed on partial pre-conditions. AP-Drift-11 (supervisor stub mistaken for live) applies — verify, do not infer.

---

## Step 1 — D120 m14 metric computation

**Inputs:** Phase 5C close · all D-N KV anchors present · Watcher synthesis · Zen audit.

**Commands:**

```bash
set -o pipefail
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# 1.1 — Full-window evidence snapshot
wf-crystallise evidence-snapshot --window-days 120 \
  > /tmp/wt-d120-lift.json 2>/tmp/wt-d120-lift.err
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "FAIL: evidence-snapshot returned non-zero"; cat /tmp/wt-d120-lift.err; exit 1; }

# 1.2 — Extract canonical metric triplet (lift, ci_half, n)
LIFT=$(jq -r '.lift // "null"'    /tmp/wt-d120-lift.json)
CI_H=$(jq -r '.ci_half // "null"' /tmp/wt-d120-lift.json)
N=$(jq    -r '.n // 0'             /tmp/wt-d120-lift.json)
echo "D120 lift=$LIFT  ci_half=$CI_H  n=$N"

# 1.3 — Per-workflow contributions (diagnostic for DEGRADED outcome)
wf-crystallise evidence-snapshot --per-workflow \
  > /tmp/wt-d120-per-workflow.json 2>/tmp/wt-d120-per-workflow.err
[[ ${PIPESTATUS[0]} -ne 0 ]] && echo "WARN: per-workflow snapshot non-zero (proceed; diagnostic only)"

# 1.4 — Compute CI lower bound for PASS gate
if [[ "$LIFT" != "null" && "$CI_H" != "null" ]]; then
  CI_LOWER=$(python3 -c "print(float('$LIFT') - float('$CI_H'))")
else
  CI_LOWER="null"
fi
echo "D120 lift_ci_lower=$CI_LOWER"
```

**Outputs:** `/tmp/wt-d120-lift.json` · `/tmp/wt-d120-per-workflow.json` · stdout triplet.

**Verification:** F2 hard gate — if `n < 20`, `lift = None` per spec (NOT zero). This is the **expected** state for insufficient data, not a code bug.

**Failure modes:**

- F-D120-1: `evidence-snapshot` exits non-zero on substrate read failure → check POVM (during overlap) AND stcortex `:3000` reachable; if substrate is down, re-run after restoration. Do NOT proceed on stale evidence.
- F-D120-2: `n < 20` AND `lift = None` — INSUFFICIENT_DATA verdict pre-determined; jump to Step 4 sub-branch.

**Watcher class:** Class-A (D120 verdict moment is highest-leverage activation transition per GOD_TIER synthesis).

---

## Step 2 — PASS / FAIL / DEGRADED / INSUFFICIENT_DATA decision tree

**Threshold:** `WF_SUNSET_LIFT_THRESHOLD = 0.05` (5% improvement over baseline). Encoded as env var at deploy time and **IMMUTABLE thereafter** (runtime recalculation creates anchor-drift risk; same reasoning as `sunset_at` immutability). G5 spec interview may have revised; verify the deploy-time value matches expectations:

```bash
THR=$(env | grep WF_SUNSET_LIFT_THRESHOLD | cut -d= -f2)
echo "Configured threshold: $THR"
```

**Decision tree (mechanical):**

```
              n >= 20?
              /      \
            YES       NO  ──► INSUFFICIENT_DATA (Step 4d)
             |
       lift >= THR AND (lift - ci_half) > 0?
             /          \
           YES            NO  ──► FAIL (Step 4b)
            |
   Cluster H active?
   (learning_health moved during soak?
    wf_m42_hebbian_reinforce_total ltp count > 0?)
        /          \
      YES            NO
       |              |
     PASS (4a)     DEGRADED (4c) — Class-I confirmed
```

**Verification commands:**

```bash
# Cluster H activity check (m42 LTP reinforcement during soak)
HEBBIAN_LTP=$(curl -s --max-time 2 http://localhost:9091/metrics 2>/dev/null \
  | grep '^wf_m42_hebbian_reinforce_total{direction="ltp"}' \
  | awk '{print $NF}')
echo "m42 LTP total since D30: $HEBBIAN_LTP"

# substrate_LTP_density trajectory (D0 vs D120)
LTP_D0=$(atuin kv get "workflow_trace.soak.d0.substrate_ltp_density")
LTP_D120=$(curl -s --max-time 2 http://localhost:8125/health \
  | python3 -c "import sys,json;print(json.load(sys.stdin).get('substrate_ltp_density','?'))")
echo "substrate_LTP_density: D0=$LTP_D0  D120=$LTP_D120"
```

**Watcher class:** Class-A (verdict emission). Class-D pre-position (if D0 vs D120 substrate metrics diverge from m14 narrative).

---

## Step 3 — Persist verdict to atuin KV and Watcher journal

```bash
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
VERDICT="<PASS|FAIL|DEGRADED|INSUFFICIENT_DATA>"

atuin kv set "workflow_trace.soak.d120.verdict"             "$VERDICT"
atuin kv set "workflow_trace.soak.d120.lift_n"              "$N"
atuin kv set "workflow_trace.soak.d120.lift_value"          "$LIFT"
atuin kv set "workflow_trace.soak.d120.lift_ci_lower"       "$CI_LOWER"
atuin kv set "workflow_trace.soak.d120.threshold"           "$THR"
atuin kv set "workflow_trace.soak.d120.cluster_h_active"    "<true|false>"
atuin kv set "workflow_trace.soak.d120.substrate_d0_d120_delta" "$(python3 -c "print(float('$LTP_D120') - float('$LTP_D0'))")"
atuin kv set "workflow_trace.soak.d120.timestamp"           "$TS"
```

WCP notice to Watcher (Command drops directly):

```bash
cat > ~/projects/shared-context/watcher-notices/${TS}_workflow_trace_d120_verdict.md <<EOF
# WCP — workflow-trace D120 Verdict

**Verdict:** $VERDICT
**Lift:** $LIFT (n=$N, ci_half=$CI_H, ci_lower=$CI_LOWER, threshold=$THR)
**Cluster H active:** see KV
**Substrate D0→D120 LTP density delta:** see KV
**Action:** Step 4 sub-branch by verdict
EOF
```

---

## Step 4 — Verdict-specific action sub-branches

### 4a — PASS continuation

```bash
# Bounded sunset extension (180-day review cycle replaces 120-day default)
NEW_DATE=$(date -u -d "+180 days" +%Y-%m-%dT%H:%M:%SZ)
wf-dispatch sunset-extend --new-date "$NEW_DATE"
# m11 enforces; Watcher Class-A flag records the extension as gate flip

# Phase B activation assessment (Watcher's call):
# - if RALPH fitness > 0.80 AND LTP/LTD ratio > 1.5 SUSTAINED → Watcher submits Phase B proposal via WCP
# - else: Phase B assessment deferred to next review cycle
```

**Constraint:** maximum **single extension = 60 days bounded** when extending from a DEGRADED outcome (Step 4c); PASS continuation is 180-day review cycle. Beyond **two extension cycles**, requires formal spec amendment (G5 interview + Zen audit). This prevents the sunset law from becoming a formality.

### 4b — FAIL retirement

m11 startup-refusal fires at next binary invocation:

```
wf-crystallise: sunset evaluation FAIL
  reason: habitat_outcome_lift=None (n=14, minimum=20) after 120 days
  sunset_at: 2026-09-14T12:00:00Z (reached)
  action: binary startup refused per m11 sunset law
  retirement artefact: /tmp/wf-retirement-notice-2026-09-14.json
  vault: the-workflow-engine-vault/retirement/RETIREMENT_NOTICE_D120.md
```

Binary exits non-zero. **No silent degrade. No read-only mode. Total refusal.**

Retirement ceremony commands:

```bash
# Archive vault (preserve permanently)
mv ~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault \
   ~/projects/claude_code/workflow-trace-retired-$(date -u +%Y%m%d)/

# Freeze stcortex pathways (preserve weights; do NOT prune — evidence)
~/.local/bin/stcortex call freeze_namespace \
  '{"namespace_prefix":"workflow_trace_","target_ns":"the_workflow_engine_retired"}'

# Atuin trajectory dump for forensics
atuin search --cwd /home/louranicas/claude-code-workspace/the-workflow-engine \
  > ~/projects/shared-context/wf-retirement/atuin-trajectory-D120.txt

# Remove binaries via /usr/bin/rm (AP-Hab-04 preserve-list discipline applies — explicit, not blanket)
/usr/bin/rm -f ~/.local/bin/wf-crystallise
/usr/bin/rm -f ~/.local/bin/wf-dispatch

# Remove cron entry (explicit grep-and-replace; never blanket crontab -r)
crontab -l | grep -v 'wf-crystallise' | crontab -

# Persist retirement notice (durable; survives binary removal)
cat > ~/projects/shared-context/wf-retirement/RETIREMENT_NOTICE_D120.json <<EOF
{
  "verdict": "FAIL",
  "sunset_at": "$(atuin kv get workflow_trace.deploy.phase5b.cutover_timestamp)",
  "evaluated_at": "$TS",
  "lift": $LIFT, "n": $N, "ci_lower": $CI_LOWER, "threshold": $THR,
  "fossil_rhyme_verdict": "<AVERTED|PARTIAL_RHYME|RHYME_CONFIRMED>",
  "status": "RETIRED"
}
EOF
```

### 4c — DEGRADED Luke decision

Three options (Luke chooses):

- **Option A — Extend + fix:** identify root cause (most likely Cluster H silence or substrate LTD-dominance); bounded 60-day extension; run pre-deploy hardening on offending cluster; re-evaluate at new deadline.
- **Option B — Retire:** explicit retirement declaration (not clock running out); run retirement ceremony (Step 4b commands).
- **Option C — Scope reduction:** remove under-performing clusters via module-spec amendment; Zen audit on amended spec; re-deploy reduced scope; new D120 clock starts at re-deployment.

Per-workflow contribution data from `/tmp/wt-d120-per-workflow.json` is the diagnostic surface for choosing among A/B/C.

### 4d — INSUFFICIENT_DATA soak extension

Not a verdict — a soak-schedule problem. Most likely cause: crystalliser cron skipped many invocations (verify atuin gaps), OR `wf-dispatch` was rarely invoked (n accumulation slow).

```bash
# Diagnose cron gap
atuin history list --cwd /home/louranicas/claude-code-workspace/the-workflow-engine \
  | grep wf-crystallise | wc -l
# Expected over 120 days of daily cron: ~120 entries. Far fewer → schedule problem.

# Extend soak (bounded; single extension cap 60 days)
NEW_DATE=$(date -u -d "+60 days" +%Y-%m-%dT%H:%M:%SZ)
wf-dispatch sunset-extend --new-date "$NEW_DATE"
# Fix cron schedule before extension begins; document fix in WCP notice
```

---

## Step 5 — Improvement candidate (IC-N) format

Whether PASS / FAIL / DEGRADED — Watcher's synthesis emits IC-N candidates for the next codebase generation. **The IC list is more valuable than the binary's operational outcome.**

```markdown
Improvement candidate IC-<N>:
  Observed:        <what happened, with evidence>
  Phase:           <which phase this was observed in>
  Flag class:      <A-I>
  Candidate:       <what should be different in the next codebase generation>
  Confidence:      <High | Medium | Low>
  Evidence:        <specific dispatch counts, timing, metric data, atuin trajectory IDs>
```

IC-N candidates land in `~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/retirement/WATCHER_FINAL_SYNTHESIS.md` (or `…/PASS_CONTINUATION_SYNTHESIS.md` if PASS).

**Hand-off rule:** the next codebase generation MUST read the final synthesis BEFORE authoring its own genesis prompt. The synthesis identifies which of the 7 structural gaps were genuine (produced measurable lift) vs theoretical (decorative in practice). The next codebase's scope should be bounded to genuine gaps.

---

## m11 immutability discipline (cross-cutting reminder)

The `sunset_at` field is encoded in m11's lifecycle module **at deploy time, NOT at runtime**. Reasoning: runtime recalculation creates the conditions for the anchor to be silently extended without ceremony — the ancestor-rhyme failure mode at the code level.

| Property | Rule | Enforcement |
|---|---|---|
| `sunset_at` value | locked at first dispatch event registered in m7 | m11 reads-once at startup; no setter |
| `WF_SUNSET_LIFT_THRESHOLD` | locked at deploy time | env var read at startup; no runtime setter |
| Extension mechanism | `wf-dispatch sunset-extend --new-date <ISO8601>` only | CLI invocation produces audit log + Watcher Class-A flag |
| Single extension cap | **60 days bounded** | m11 rejects extension > 60d with `ExtensionTooLong` error |
| Extension cycles | **maximum 2 cycles** before formal spec amendment + Zen audit | m11 tracks `extension_count`; rejects 3rd extension without amendment marker |
| Extension audit | structured audit-log entry in m7 + WCP notice from Watcher (Class-A) | mandatory; no silent extensions |

---

## Phase-end gate (D120 close)

```bash
# Mandatory close artefacts
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
atuin kv set "workflow_trace.phase6.close.timestamp" "$TS"
atuin kv set "workflow_trace.phase6.close.verdict"   "$VERDICT"
atuin kv set "workflow_trace.phase6.close.action"    "<continuation|retired|scope_reduction|soak_extended>"

# IC-N file path persisted
atuin kv set "workflow_trace.phase6.close.synthesis_path" \
  "~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/retirement/WATCHER_FINAL_SYNTHESIS.md"

# Fossil-rhyme verdict (Watcher's call)
atuin kv set "workflow_trace.phase6.close.fossil_rhyme" "<AVERTED|PARTIAL_RHYME|RHYME_CONFIRMED>"
```

Hand-off targets:

- If PASS continuation: soak resumes; Phase 5C runbook re-engages with `sunset_at` shifted +180d; weekly Watcher synthesis continues
- If FAIL or retired: hand-off to [[runbook-11-emergency-rollback.md]] § retirement archival path; next codebase generation reads `WATCHER_FINAL_SYNTHESIS.md` BEFORE its genesis prompt
- If scope reduction (DEGRADED Option C): re-deploy with reduced scope; new D120 clock; restart [[runbook-06-phase-5-deploy-soak.md]] from Phase 5A
- If INSUFFICIENT_DATA: extension; fix cron schedule first; document fix

---

## Failure modes register

| ID | Trigger | Detection | Mitigation | Antipattern |
|---|---|---|---|---|
| F-PH6-1 | `evidence-snapshot` runs on stale substrate (POVM down post-cutover not handled) | `/tmp/wt-d120-lift.err` shows substrate timeout | restore substrate; re-run; do NOT cite stale data | AP-Hab-13 runbook probe freshness |
| F-PH6-2 | `sunset_at` runtime mutation attempted (someone tries to edit the deployed value) | m11 setter does not exist | architectural — no setter at all | (ancestor-rhyme prevention) |
| F-PH6-3 | Extension > 60 days requested | m11 returns `ExtensionTooLong` | use 2× cycles max; then formal spec amendment + Zen audit | (bounded by design) |
| F-PH6-4 | FAIL verdict + silent retry (operator re-runs wf-crystallise expecting different result) | m11 startup-refusal fires same result | acknowledge retirement; do NOT bypass | (m11 total-refusal discipline) |
| F-PH6-5 | DEGRADED Option C re-deploy without Zen audit | git log shows no Zen-audited spec amendment commit | refuse re-deploy; route to Zen G7 audit | AP-Drift-01 (over-claim) |
| F-PH6-6 | IC-N synthesis missing or empty | `WATCHER_FINAL_SYNTHESIS.md` not present or 0 ICs listed | refuse hand-off to next codebase; require Watcher synthesis | (improvement loop closure) |
| F-PH6-7 | Fossil-rhyme verdict skipped | `workflow_trace.phase6.close.fossil_rhyme` KV absent | Watcher MUST issue AVERTED / PARTIAL_RHYME / RHYME_CONFIRMED | (terminal-gate ceremony) |

---

## Watcher flag pre-positioning

| Class | Pre-position trigger | What it captures |
|---|---|---|
| **A** | D120 verdict emission; every sunset extension; PASS continuation 180-day flip | activation transitions verbatim — highest-leverage moment |
| B | retirement ceremony commands crossed | hand-off boundary (engine → archive) |
| C | DEGRADED Option C scope reduction re-audit refusal | confidence-gate refusal |
| D | per-workflow contribution evidence vs aggregate lift discrepancy | four-surface drift in evaluation data |
| **E** | Fossil-rhyme verdict = PARTIAL_RHYME or RHYME_CONFIRMED | ancestor-rhyme final reckoning |
| F | N/A post-Phase 1 | (would have caught pre-G9 code; not relevant here) |
| G | substrate_LTP_density delta D0 → D120 inconsistent with m14 narrative | substrate-frame confusion at evaluation |
| H | atuin trajectory shows fewer wf-crystallise runs than expected over 120d | atuin proprioception anomaly (INSUFFICIENT_DATA root cause) |
| I | Cluster H active = false at D120 with PASS-shaped lift | Hebbian silence confirmed; pushes verdict to DEGRADED |

---

## Atuin trajectory anchors

```bash
# Pre-evaluation
atuin search "wf-crystallise evidence-snapshot" --cwd /home/louranicas/claude-code-workspace/the-workflow-engine
atuin kv get  "workflow_trace.soak.d90.lift_value"
atuin kv get  "workflow_trace.soak.d90.lift_ci_lower"

# Evaluation moment
atuin kv get  "workflow_trace.soak.d120.verdict"
atuin kv get  "workflow_trace.soak.d120.cluster_h_active"
atuin scripts run wt-substrate-pulse

# Post-evaluation
atuin kv get  "workflow_trace.phase6.close.action"
atuin kv get  "workflow_trace.phase6.close.fossil_rhyme"
atuin scripts run wt-wave-status            # confirm no leftover Wave locks (G2-stage)
```

---

## Sign-off

This runbook is **planning-only** (HOLD-v2). It becomes executable only at D120 in a deployed system, after [[runbook-06-phase-5-deploy-soak.md]] completes Phase 5C. The verdict at D120 determines whether [[runbook-11-emergency-rollback.md]] § retirement archival activates, OR whether soak continues under extended `sunset_at`. m11's immutability is load-bearing — runtime mutation defeats the ancestor-rhyme prevention.

*Runbook 07 authored 2026-05-17 by Command (V7 optimisation, parallel author). D120 sunset evaluation operational. m11 immutability + bounded-extension rule encoded. IC-N format defined. ~1,720 words. Source: phase-6-sunset-and-cross-cutting.md. Sibling: runbook-06 / runbook-10 / runbook-11.*
