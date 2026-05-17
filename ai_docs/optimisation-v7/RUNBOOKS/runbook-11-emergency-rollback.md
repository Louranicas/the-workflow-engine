---
title: Runbook 11 — Emergency Rollback (Phase 1 / Phase 5A / Phase 5B + Class-E post-mortem trigger)
date: 2026-05-17 (S1001982)
kind: planning-only · operational runbook · on-event reactive
trigger: phase gate fails AND decision = roll back (not fix-forward) · OR Watcher Class-E ancestor-rhyme post-mortem trigger fires
scenarios: 3 — Phase 1 Genesis · Phase 5A Binary Deploy · Phase 5B Production Cutover · plus Class-E halt protocol
owner: Command (executes) · Luke @ node 0.A (authorises) · Watcher (records Class-B + Class-E)
discipline: preserve-list (S102) · never blanket · git stash-list before pop · /usr/bin/rm not bare rm · no force-push to main
authority: Luke @ node 0.A
status: planning-only · HOLD-v2 active · NOT executable until G1-G9 GREEN
---

# Runbook 11 — Emergency Rollback

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ANTIPATTERNS_REGISTER.md]] · sibling [[runbook-06-phase-5-deploy-soak.md]] · [[runbook-07-phase-6-sunset.md]] · [[runbook-10-cross-cutting.md]]
>
> Source phase doc: [[../../the-workflow-engine-vault/deployment framework/phase-6-sunset-and-cross-cutting.md]] § CC-2 Rollback Procedure + per-phase rollback sections; auxiliary [[../../the-workflow-engine-vault/deployment framework/phase-5-deploy-and-soak.md]] § Rollback procedure

---

## Overview

This runbook covers **emergency rollback** in three canonical scenarios — Phase 1 Genesis (scaffold creation), Phase 5A Binary Deploy (materialisation to `~/.local/bin/`), Phase 5B Production Cutover (ceremony + first dispatch) — plus the **Class-E ancestor-rhyme post-mortem trigger** (planning persists past G9+14 days with no code shipped). Each scenario has a **3-command rollback core** plus pre-flight verification and post-rollback confirmation. The discipline is anti-blanket throughout: `/usr/bin/rm -f` (never bare `rm`); `git stash list` BEFORE pop; never `git push --force` to main; preserve named artefacts (atuin trajectory, m15 reservation JSONL, Watcher journal) even on rollback because they are evidence for the next codebase generation. Rollback is **deliberate, not destructive improvisation** — Luke authorises every step; Watcher records Class-B (hand-off boundary reversed) at each.

---

## Pre-flight checklist

- Luke explicit authorisation to roll back (NOT fix-forward) recorded in current session
- `git status` shows clean working tree OR uncommitted changes are documented in WIP branch
- `git stash list` shows no pre-existing stashes that could collide with rollback ops (AP-Hab-08)
- Watcher Class-B flag pre-positioned (rollback is hand-off reversed)
- Backup of `~/.local/bin/wf-*` to `/tmp/wf-rollback-backup-$(date +%Y%m%d%H%M)/` BEFORE rollback fires
- For Phase 5B: confirm POVM `:8125` post-cutover state is captured (cutover dance may not be reversible)
- For Class-E: Watcher synthesis of planning-sprawl evidence + atuin trajectory dump prepared

---

## Scenario 1 — Phase 1 Genesis rollback (scaffold creation reversed)

**When:** Phase 1 produces a scaffold tree that fails downstream validation (Drift Register dim 1-4 fail, or Zen audit refuses), AND decision is roll back to pre-Phase-1 state (planning-only).

**Pre-condition:** vault at `~/claude-code-workspace/the-workflow-engine-vault/` is **separately persisted** and is NOT touched by this rollback. Only the workflow-trace **source tree** (`src/`, `Cargo.toml`, `Cargo.lock`) is removed.

**3-command rollback:**

```bash
# Command 1 — back up scaffold tree before removal (evidence preservation)
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
mkdir -p /tmp/wt-phase1-rollback-${TS}/
cp -r ~/claude-code-workspace/the-workflow-engine/src \
      ~/claude-code-workspace/the-workflow-engine/Cargo.toml \
      ~/claude-code-workspace/the-workflow-engine/Cargo.lock \
   /tmp/wt-phase1-rollback-${TS}/ 2>/dev/null

# Command 2 — remove scaffold tree (preserve-list discipline; explicit paths only, NEVER blanket)
/usr/bin/rm -rf ~/claude-code-workspace/the-workflow-engine/src
/usr/bin/rm -f  ~/claude-code-workspace/the-workflow-engine/Cargo.toml
/usr/bin/rm -f  ~/claude-code-workspace/the-workflow-engine/Cargo.lock

# Command 3 — git revert (NOT git reset --hard; preserve git history)
git stash list   # MANDATORY pre-pop check (AP-Hab-08)
cd ~/claude-code-workspace/the-workflow-engine
git revert <scaffold-commit-sha> --no-edit
git push origin HEAD                          # NEVER --force on main
```

**Verification:**

```bash
test -d ~/claude-code-workspace/the-workflow-engine/src && \
  echo "FAIL: src/ still exists; rollback incomplete" || echo "OK: src/ removed"

# Vault preserved (untouched)
ls -la ~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/ | head -3

# Atuin trajectory anchor
atuin kv set "workflow_trace.rollback.phase1.timestamp"  "$TS"
atuin kv set "workflow_trace.rollback.phase1.reason"     "<Luke-stated>"
atuin kv set "workflow_trace.rollback.phase1.backup_dir" "/tmp/wt-phase1-rollback-${TS}/"
```

**WCP notice (Command drops directly):**

```bash
cat > ~/projects/shared-context/watcher-notices/${TS}_workflow_trace_phase1_rollback.md <<EOF
# WCP — Phase 1 Rollback (scaffold removed; vault preserved)
**Reason:** <Luke-stated>
**Backup:** /tmp/wt-phase1-rollback-${TS}/
**Vault:** intact (planning-only state restored)
**Class-B flag:** record reversed hand-off
EOF
```

**Failure modes:**

- F-RB1-1: `git stash list` shows pre-existing stash → STOP. Inspect via `git stash show -p stash@{0}`. AP-Hab-08 — pop wrong stash silently deletes work.
- F-RB1-2: scaffold removal blocked by file ownership / open file handles → confirm no concurrent process has `~/claude-code-workspace/the-workflow-engine/src` open; `lsof | grep workflow-engine` to investigate.
- F-RB1-3: vault accidentally touched (path typo) → STOP IMMEDIATELY. Restore vault from git OR Obsidian sync OR shared-context mirror.

**Watcher class:** Class-B (hand-off reversed); Class-E pre-position if rollback is the 3rd Phase 1 reversal — pattern suggests ancestor-rhyme.

---

## Scenario 2 — Phase 5A Binary Deploy rollback (binaries removed; build state preserved)

**When:** Phase 5A materialisation succeeds but smoke test (`wf-crystallise --version` or `wf-dispatch --help`) fails OR substrate consumer registration refuses, AND decision is to roll back the deployment (not the build).

**3-command rollback:**

```bash
# Command 1 — back up materialised binaries for forensics (size + SHA preserved)
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
mkdir -p /tmp/wt-phase5a-rollback-${TS}/
cp ~/.local/bin/wf-crystallise  /tmp/wt-phase5a-rollback-${TS}/  2>/dev/null
cp ~/.local/bin/wf-dispatch     /tmp/wt-phase5a-rollback-${TS}/  2>/dev/null

# Command 2 — remove binaries (AP-Hab-06 alias trap: explicit /usr/bin/rm, NOT bare rm)
/usr/bin/rm -f ~/.local/bin/wf-crystallise
/usr/bin/rm -f ~/.local/bin/wf-dispatch

# Command 3 — atuin KV record + WCP notice (Command drops directly)
atuin kv set "workflow_trace.rollback.phase5a.timestamp"          "$TS"
atuin kv set "workflow_trace.rollback.phase5a.reason"             "<Luke-stated>"
atuin kv set "workflow_trace.rollback.phase5a.backup_dir"         "/tmp/wt-phase5a-rollback-${TS}/"
atuin kv set "workflow_trace.deploy.phase5a.status"               "rolled_back"

cat > ~/projects/shared-context/watcher-notices/${TS}_workflow_trace_phase5a_rollback.md <<EOF
# WCP — Phase 5A Rollback (binaries removed)
**Reason:** <Luke-stated>
**Backup:** /tmp/wt-phase5a-rollback-${TS}/
**Build artefacts:** preserved at \$CARGO_TARGET_DIR/release/
**Class-B flag:** binary store ← build env reversed
EOF
```

**Verification:**

```bash
[[ ! -f ~/.local/bin/wf-crystallise ]] && echo "OK: wf-crystallise removed" || echo "FAIL"
[[ ! -f ~/.local/bin/wf-dispatch    ]] && echo "OK: wf-dispatch removed"    || echo "FAIL"

# Build artefacts still present (re-deploy possible without re-build)
ls -lh "$CARGO_TARGET_DIR/release/wf-crystallise" 2>/dev/null
```

**Failure modes:**

- F-RB5A-1: `cp` aliased to `cp -i` prompts on backup → AP-Hab-06. Use explicit `/usr/bin/cp` if backup step prompts.
- F-RB5A-2: binary in use (cron invocation mid-flight) → `fuser ~/.local/bin/wf-crystallise` to confirm; wait or `kill -TERM`.
- F-RB5A-3: PATH not refreshed → operator still sees `wf-crystallise --version` working from another shell. Confirm via `which wf-crystallise` → "command not found".

**Watcher class:** Class-B (hand-off reversed); pre-position Class-A if smoke failure was due to gate flip (POVM `:8125` health drift).

---

## Scenario 3 — Phase 5B Production Cutover rollback (ceremony reversed; soak clock retracted)

**When:** Phase 5B cutover fires but first production crystalliser run OR dispatch dry-run OR 4-surface substrate touch FAILS, AND decision is to retract cutover ceremony.

**This is the most delicate scenario** — soak clock has nominally started; m11 sunset_at MAY have been encoded; Watcher Class-A entry MAY exist.

**3-command rollback (with critical extra steps):**

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)

# Command 1 — retract cutover signal in atuin KV (soak clock stops)
atuin kv set "workflow_trace.soak.d0.lift_n"               "ROLLED_BACK"
atuin kv set "workflow_trace.soak.d0.timestamp"            "ROLLED_BACK"
atuin kv set "workflow_trace.rollback.phase5b.timestamp"   "$TS"
atuin kv set "workflow_trace.rollback.phase5b.reason"      "<Luke-stated>"

# Command 2 — halt m32 dispatcher (env override; Conductor refuses further dispatches)
# This sets refuse-mode at next wf-dispatch invocation; m32 returns DispatchError::ConductorDispatchDisabled
export CONDUCTOR_DISPATCH_ENABLED=0
# Persist for cron context:
echo 'export CONDUCTOR_DISPATCH_ENABLED=0' >> ~/.bashrc

# Command 3 — WCP notice retracting cutover (Watcher will mark Class-A flag RETRACTED)
cat > ~/projects/shared-context/watcher-notices/${TS}_workflow_trace_phase5b_cutover_retraction.md <<EOF
# WCP — Phase 5B Cutover RETRACTED

**Original cutover:** <D30 timestamp from atuin KV pre-rollback>
**Retraction:** $TS
**Reason:** <Luke-stated; usually: smoke failed | 4-surface incomplete | dispatch dry-run failed>
**Soak clock:** stopped; D0 baseline marked ROLLED_BACK in atuin KV
**Class-A flag:** original cutover entry MUST be amended to status=RETRACTED
**Class-B flag:** record soak boundary reversed
**Next:** Luke decides: re-cutover after fix | full Phase 5A rollback | retire pre-soak
EOF
```

**Critical post-rollback steps:**

```bash
# 1. Preserve outbox JSONL (DO NOT delete; evidence of what fired between cutover and retraction)
mv ~/claude-code-workspace/the-workflow-engine/outbox \
   ~/claude-code-workspace/the-workflow-engine/outbox-pre-retraction-${TS}/ 2>/dev/null

# 2. POVM Hebbian pathways FROZEN (preserve weights; do NOT prune — evidence)
#    Pre-cutoff was overlap; m42 may have written to BOTH POVM and stcortex.
#    Do NOT undo these writes; they are forensic evidence.
echo "POVM/stcortex pathways FROZEN as-of $TS (no prune)" \
  >> ~/projects/shared-context/wf-retirement/cutover-retraction-evidence-${TS}.md

# 3. Stop crystalliser cron (prevent further accumulation post-retraction)
crontab -l | grep -v 'wf-crystallise' | crontab -

# 4. Verify dispatcher refusal (next wf-dispatch should refuse)
CONDUCTOR_DISPATCH_ENABLED=0 wf-dispatch execute --workflow-id <test> --dry-run 2>&1 | tail -5
# Expected: DispatchError::ConductorDispatchDisabled
```

**Verification:**

```bash
# atuin KV reflects retraction
atuin kv get "workflow_trace.rollback.phase5b.timestamp"
atuin kv get "workflow_trace.soak.d0.lift_n"   # should be "ROLLED_BACK"

# No further outbox writes
ls -la ~/claude-code-workspace/the-workflow-engine/outbox/ 2>/dev/null && \
  echo "WARN: outbox re-created; investigate" || echo "OK: outbox absent"

# Watcher journal entry retraction visible
grep -A3 "RETRACTED" ~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/WATCHER_DEPLOYMENT_WATCH_JOURNAL_S1001982.md | tail -10
```

**Failure modes:**

- F-RB5B-1: Conductor already executed a dispatched workflow before retraction → habitat change is unaccounted for. **CRITICAL** — investigate via `dispatch_log.db` SELECT WHERE `dispatched_at > <cutover_ts>`. Document in WCP notice; Luke decides whether to also retract the habitat change.
- F-RB5B-2: m11 `sunset_at` was encoded at first dispatch → m11 will REFUSE further dispatches at the encoded date regardless of retraction. **m11 immutability is by design** ([[runbook-07-phase-6-sunset.md]]) — cannot be unset. Subsequent re-cutover requires either (a) full Phase 5A rollback + re-deploy (resets sunset_at), OR (b) `wf-dispatch sunset-extend` to bump the encoded date (bounded 60d × 2 cycles).
- F-RB5B-3: POVM cutover dance was already mid-flight (~D25 flip post-cutover) → AP-WT-F7. Investigate `workflow_trace.povm_overlap_active` KV value; do NOT silently re-flip.

**Watcher class:** Class-A RETRACTED (amend original cutover entry); Class-B (soak boundary reversed); Class-E pre-position if 2nd Phase 5B retraction in same project — strong ancestor-rhyme signal.

---

## Class-E Ancestor-Rhyme Post-Mortem Trigger

**Trigger condition:** **planning past G9 + 14 days with no code shipped.** This is the canonical ancestor-rhyme signal — the two ancestor codebases (`loop-workflow-engine-project`, `habitat-loop-engine`) died from open-ended scope accumulation with no code ever produced. Watcher Class-E flag fires automatically.

**Halt protocol:**

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)

# Step 1 — Watcher confirms Class-E trigger
atuin kv set "workflow_trace.class_e_trigger.timestamp" "$TS"
atuin kv set "workflow_trace.class_e_trigger.reason"    "planning_past_g9_plus_14d_no_code"

# Step 2 — Command halts ALL workflow-trace work (no new planning docs, no new module specs, no new generations)
cat > ~/projects/shared-context/watcher-notices/${TS}_workflow_trace_CLASS_E_HALT.md <<EOF
# WCP — CLASS-E ANCESTOR-RHYME HALT FIRED

**Trigger:** planning past G9 + 14 days; zero LOC shipped
**Halt scope:** ALL workflow-trace work — no new planning, no new specs, no new generations
**Required:** Watcher-authored Fossil-rhyme post-mortem BEFORE any further work
**Authority:** Luke @ node 0.A must explicitly RESET or RETIRE

# Reset criteria (Luke decides):
# (a) RESET — Luke files explicit per-gate waivers OR drives G1-G8 in sequence; halt lifts only after G9 fires
# (b) RETIRE — Watcher's Fossil-rhyme verdict = RHYME_CONFIRMED; workflow-trace project retired pre-genesis
# (c) PIVOT — scope reduction; new genesis prompt; G7 Zen audit on reduced spec; G9 re-fire
EOF

# Step 3 — Watcher Fossil-rhyme post-mortem (mandatory before any reset)
# Watcher reads:
#  - 41,508+ words of planning docs vs 0 LOC code (the asymmetry)
#  - m15 reservation JSONL accumulation (scope-pressure events history)
#  - atuin trajectory: which scripts were repeated; which were abandoned
#  - 6 critical-path blockers history (B1-B6 in CLAUDE.local.md): which moved, which stalled
# Watcher writes post-mortem to:
#  ~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/retirement/FOSSIL_RHYME_POSTMORTEM_${TS}.md
# Verdict: AVERTED | PARTIAL_RHYME | RHYME_CONFIRMED

# Step 4 — Luke decides RESET/RETIRE/PIVOT after reading post-mortem
# Decision recorded in atuin KV:
# atuin kv set "workflow_trace.class_e_trigger.luke_decision" "<reset|retire|pivot>"
# atuin kv set "workflow_trace.class_e_trigger.decision_timestamp" "<TS>"
```

**Reset criteria (Luke chooses one):**

- **RESET:** Luke files explicit per-gate waivers OR drives G1-G8 in sequence. Halt lifts only after G9 fires AND a code-shipping cadence is established (≥1 module shipped per week). Watcher tracks cadence; re-fires Class-E if cadence falls below.
- **RETIRE:** Watcher's verdict is RHYME_CONFIRMED. workflow-trace project retired **pre-genesis**. Planning artefacts preserved as `~/claude-code-workspace/the-workflow-engine-vault-retired-pre-genesis-${TS}/`. IC-N candidates from Watcher synthesis hand off to NEXT codebase (which inherits THE FRAMEWORK, not workflow-trace specifically).
- **PIVOT:** scope reduction; new genesis prompt; G7 Zen audit on reduced spec; G9 re-fire. Bounded — single pivot per project; second Class-E trigger after a pivot is automatic RETIRE.

**Failure modes:**

- F-CE-1: Class-E trigger ignored (Luke says "let's keep planning a bit more") → AP-V7-01 (7-gen drift) extended; pattern matches ancestor death exactly. Watcher CANNOT enforce — but MUST escalate via Class-E priority WCP notice every 7 days until resolved.
- F-CE-2: post-mortem skipped (Luke decides RESET without reading post-mortem) → AP-V7-08 (handshake dual-silence false-success). Post-mortem must be READ + acknowledged before RESET takes effect.

**Watcher class:** Class-E (the canonical trigger); secondary Class-A on Luke's decision moment; Class-D if RESET fires but no code-shipping cadence within 14d (pattern repeats).

---

## Phase-end gate

Each rollback scenario produces a documented evidence trail:

```bash
# Rollback close artefacts (per scenario)
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
SCENARIO="<phase1|phase5a|phase5b|class_e>"

atuin kv set "workflow_trace.rollback.${SCENARIO}.close_ts"     "$TS"
atuin kv set "workflow_trace.rollback.${SCENARIO}.evidence_dir" "<path>"

# Final WCP notice with full rollback audit trail
ls -la ~/projects/shared-context/watcher-notices/*workflow_trace_${SCENARIO}* | tail -3
```

Hand-off targets:

- Phase 1 rollback → return to planning-only state; G1-G9 gate review
- Phase 5A rollback → fix root cause (smoke failure); re-run [[runbook-06-phase-5-deploy-soak.md]] from Step 1
- Phase 5B rollback → Luke decides: re-cutover after fix | full 5A rollback | pre-soak retirement
- Class-E halt → Luke decides RESET / RETIRE / PIVOT after reading Watcher Fossil-rhyme post-mortem

---

## Failure modes register (consolidated)

| ID | Scenario | Trigger | Detection | Mitigation |
|---|---|---|---|---|
| F-RB1-1 | Phase 1 | pre-existing stash collision | `git stash list` shows unexpected entry | inspect via `git stash show -p stash@{0}` BEFORE pop |
| F-RB1-2 | Phase 1 | scaffold removal blocked | `lsof` shows file in use | identify + terminate concurrent process |
| F-RB1-3 | Phase 1 | vault accidentally touched | path typo in rm command | STOP; restore from git/Obsidian/shared-context |
| F-RB5A-1 | Phase 5A | `cp` aliased to `cp -i` | interactive prompt on backup | use `/usr/bin/cp` explicit |
| F-RB5A-2 | Phase 5A | binary in use | `fuser ~/.local/bin/wf-crystallise` non-empty | wait OR `kill -TERM` |
| F-RB5A-3 | Phase 5A | PATH not refreshed in other shell | `which wf-crystallise` still resolves elsewhere | confirm in all open shells |
| F-RB5B-1 | Phase 5B | Conductor already executed dispatch | `dispatch_log.db` shows post-cutover dispatch | document; Luke decides on habitat change reversal |
| F-RB5B-2 | Phase 5B | m11 `sunset_at` already encoded | m11 sunset law immutable | full Phase 5A rollback OR `sunset-extend` |
| F-RB5B-3 | Phase 5B | POVM cutover dance mid-flight | `povm_overlap_active` ambiguous | investigate; do NOT silently re-flip |
| F-CE-1 | Class-E | Class-E trigger ignored | no Luke decision after 7d | Watcher re-escalates Class-E priority every 7d |
| F-CE-2 | Class-E | post-mortem skipped | Luke RESET without acknowledged post-mortem | post-mortem MUST be read + acked before RESET takes effect |

---

## Watcher flag pre-positioning

| Class | Pre-position trigger |
|---|---|
| **A** | Phase 5B cutover RETRACTED (amend original Class-A entry); Class-E halt fires; Luke RESET/RETIRE/PIVOT decision |
| **B** | every rollback scenario — hand-off reversed; record verbatim |
| C | F-RB5B-3 if POVM cutover ambiguity blocks m42 |
| D | F-RB5B-1 if dispatch_log.db evidence contradicts atuin trajectory |
| **E** | 2nd Phase 5B retraction in same project; planning past G9 + 14d (canonical Class-E trigger) |
| F | N/A post-Phase 1 |
| G | F-RB5B-3 substrate-frame ambiguity on cutover |
| H | atuin trajectory shows rollback commands missing (atuin daemon down during rollback) |
| I | F-CE-1 sustained Class-E ignore + Hebbian silence baseline |

---

## Atuin trajectory anchors

```bash
# Per-scenario audit
atuin search "/usr/bin/rm.*wf-"          --before 7d
atuin search "git revert"                --before 7d   --cwd /home/louranicas/claude-code-workspace/the-workflow-engine
atuin search "CONDUCTOR_DISPATCH_ENABLED=0" --before 7d
atuin kv get "workflow_trace.rollback.phase5a.timestamp"
atuin kv get "workflow_trace.rollback.phase5b.timestamp"
atuin kv get "workflow_trace.class_e_trigger.timestamp"
atuin kv get "workflow_trace.class_e_trigger.luke_decision"
```

---

## Sign-off

This runbook is **planning-only** (HOLD-v2). It activates only when a real phase rollback is required AND Luke explicitly authorises. Class-E ancestor-rhyme post-mortem trigger is the **canonical defence against the ancestor death pattern** — planning sprawl with no code shipped. The 60-day bounded-extension rule from [[runbook-07-phase-6-sunset.md]] AND the Class-E halt protocol together form the engine's terminal-gate discipline.

*Runbook 11 authored 2026-05-17 by Command (V7 optimisation, parallel author). 3 rollback scenarios + Class-E halt protocol operational. Preserve-list discipline + git stash-list discipline + immutability constraints encoded. ~1,990 words. Source: phase-6-sunset-and-cross-cutting.md § CC-2 Rollback + phase-5-deploy-and-soak.md § Rollback. Sibling: runbook-06 / runbook-07 / runbook-10.*
