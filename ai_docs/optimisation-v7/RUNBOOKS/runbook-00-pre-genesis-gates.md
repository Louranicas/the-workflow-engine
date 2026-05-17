---
title: Runbook 00 — Pre-Genesis Gates G1-G9 (Operational)
date: 2026-05-17 (S1001982)
kind: planning-only operational runbook
phase: 0
days: pre-G9
owner: Command (Tab 1 Orchestrator top-left) + Luke @ node 0.A + Watcher + Zen
binding_spec: Genesis Prompt v1.2 (Zen-locked) — v1.3 patch pending
status: planning-only · HOLD-v2 active · 0 of 9 gates green
---

# Runbook 00 — Pre-Genesis Gates G1-G9 (Operational)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · sibling: [[runbook-01-phase-1-genesis]]
>
> Cites: `the-workflow-engine-vault/deployment framework/phase-0-pre-genesis-gates.md` (narrative source); refines into per-gate command sequences. Cross-ref: [[../KEYWORDS_20.md]] (AP24, AP30, F2, four-surface, HOLD-v2, single-phase, Watcher-class).

---

## Overview

This runbook is the **operational** view of Phase 0 — what commands fire, in what order, with what verification, when each gate is unblocked. It is NOT a planning narrative (read `phase-0-pre-genesis-gates.md` for the planning narrative + rationale). Each gate is a **state change**, not a deliverable. Code is forbidden until G9 clears. The gate before this runbook is "Luke directs Watcher to file G1 close-notice" (or Watcher self-initiates per WCP). The gate after this runbook is G9 fired → control transfers to `runbook-01-phase-1-genesis.md`.

Current state at runbook author time (2026-05-17): 0/9 green; G9 fired out-of-sequence and is Zen URGENT-blocked (queued intent only). HOLD-v2 forbids `cargo init`, `src/*.rs`, scaffold, rename for build purposes, and stcortex writes under `workflow_trace_*`.

---

## Pre-flight checklist

```bash
# PIPESTATUS discipline mandatory throughout — see AP-Hab-05.
set -o pipefail

# (1) Confirm HOLD-v2 still in force (no src/, no Cargo.toml, no cargo init has fired)
test ! -d /home/louranicas/claude-code-workspace/the-workflow-engine/src \
  && echo "OK: no src/" || { echo "VIOLATION AP-Hab-01"; exit 1; }
test ! -f /home/louranicas/claude-code-workspace/the-workflow-engine/Cargo.toml \
  && echo "OK: no Cargo.toml" || { echo "VIOLATION AP-Hab-01"; exit 1; }

# (2) Habitat baseline — 11 active services healthy
for port in 8082 8083 8092 8111 8120 8125 8130 8132 8133 8180 10002; do
  code=$(curl -s -o /dev/null -w '%{http_code}' -m 1 "http://localhost:${port}/health" 2>/dev/null || echo "000")
  echo "port=${port} code=${code}"
done
# Refusal: any 000 or 5xx for ports 8125 (G3), 8092 (Watcher carriage), 3000 (stcortex G8)

# (3) stcortex reachability (G8 prerequisite)
~/.local/bin/stcortex status 2>&1 | tee /tmp/g0-stcortex-status.txt
test "${PIPESTATUS[0]}" -eq 0 || echo "WARN: stcortex DOWN — G8 will skip surface 3, document in D-G8"

# (4) Watcher carriage alive (R13 elapsed; eligible)
curl -s -m 1 http://localhost:8092/health 2>&1 | tee /tmp/g0-watcher.txt
test "${PIPESTATUS[0]}" -eq 0 || echo "WARN: watcher carriage unreachable"

# (5) Verify CR-2 + CR-2b are merged at HEAD on povm-v2 source (G3 prerequisite)
cd /home/louranicas/claude-code-workspace/povm-v2 2>/dev/null && \
  git log --oneline | head -5 | /usr/bin/grep -E "(e2a8ed3|76ea4d6)" \
  || echo "PREREQ MISSING: CR-2/CR-2b not at HEAD"

# (6) atuin proprioception baseline
atuin search "G[0-9]" --limit 20 2>&1 | head -10
```

If any item fails, do NOT advance to G1. Diagnose and re-pre-flight.

---

## G1 — RATIFICATION

**Owner:** Luke OR Watcher (self-initiated). **Watcher class A.** **Mitigates:** AP-Hab-09 (TaskCreate absorption — directive-not-tooling rule).

```bash
# Path A — Luke directs Watcher:
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
cat > ~/projects/shared-context/agent-cross-talk/${TS}_luke_g1_direction.md <<EOF
Watcher: please file G1 ratification close-notice for Path A (workflow-trace, measure-only) + Candidate A (CR-2 SHIPPED).
EOF

# Path B — Watcher self-initiated via WCP:
~/.local/bin/watcher notify g1_ratification "Path A + Candidate A formally ratified; W1/W2/W3 conditions remain."

# Command verifies close-notice landed:
ls -la ~/projects/shared-context/watcher-notices/ 2>&1 | /usr/bin/grep -i ratification | tee /tmp/g1-verify.txt
test "${PIPESTATUS[0]}" -eq 0 && \
  /usr/bin/grep -l "formally ratified" ~/projects/shared-context/watcher-notices/*ratification* | head -3

# Command files D-G1:
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_d_g1.md <<EOF
D-G1 (G1 RATIFICATION): GREEN at ${TS}. Path A + Candidate A confirmed by Watcher.
Source: $(ls -t ~/projects/shared-context/watcher-notices/*ratification* | head -1)
EOF
```

**Verification:** `/usr/bin/grep -l "formally ratified" ~/projects/shared-context/watcher-notices/*ratification*` returns ≥1 file containing Watcher's ☤ attribution AND "Path A" AND "Candidate A". D-G1 filed.

**Failure modes:**
1. Watcher uses "leading draft" not "formally ratified" — Watcher Class-A; ask Watcher to amend.
2. Luke signal ambiguous ("proceed" without Path naming) — refuse to interpret; ask for explicit Path A confirmation. (AP-Hab-09 absorption guard.)
3. No close-notice filed within 4h tempo budget — Watcher Class-H; not a failure (no deadline) but file reminder.

**Rollback:** annotate close-notice "SUSPENDED pending G7 APPROVE" rather than delete.

**Watcher class pre-positioned:** A (activation transition).

**Atuin anchors:** `atuin search "watcher G1 ratification"` ; `atuin search "ls.*watcher-notices.*ratification"`.

---

## G2 — NAMING + DIRECTORY RENAME

**Owner:** Luke (decision + terminal rename); Command (vault wikilink updates). **Watcher class A or D.** **Mitigates:** AP-V7-06 (bidi-anchor rot).

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)

# Step 1 — Luke (terminal — Claude does NOT spawn):
# mv ~/claude-code-workspace/the-workflow-engine/ ~/claude-code-workspace/workflow-trace/

# Step 2 — Command updates vault wikilinks (use Edit tool per CLAUDE.md anti-pattern table):
# Identify files containing legacy reference:
/usr/bin/grep -rln "the-workflow-engine" \
  ~/claude-code-workspace/workflow-trace/the-workflow-engine-vault/ 2>/dev/null \
  | tee /tmp/g2-files-to-edit.txt
# (Each file then edited via Edit tool; NEVER sed -i per AP-V7-03)

# Step 3 — verify zero stale references:
stale=$(/usr/bin/grep -rln "the-workflow-engine" \
  ~/claude-code-workspace/workflow-trace/ 2>/dev/null \
  | /usr/bin/grep -v "\.git/" | wc -l)
test "$stale" -eq 0 && echo "OK: no stale references" \
  || { echo "FAIL: ${stale} stale references remain"; exit 2; }

# Step 4 — update CLAUDE.local.md workspace-root row (path field only):
# Edit ~/claude-code-workspace/CLAUDE.local.md § Active Workstreams "Workflow-trace" row

# Step 5 — file D-G2:
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_d_g2.md <<EOF
D-G2 (G2 NAMING): GREEN at ${TS}.
Decision: Option A — workflow-trace
Rename: the-workflow-engine/ → workflow-trace/
Wikilinks updated: $(wc -l < /tmp/g2-files-to-edit.txt) files
Stale-ref count: 0
EOF
```

**Verification:** `ls ~/claude-code-workspace/workflow-trace/` succeeds; stale-ref count = 0; CLAUDE.local.md row reflects new path. D-G2 filed.

**Failure modes:**
1. Luke chooses Option C (scope-honest rename pushed to G5) — record decision; do not rename; Watcher Class-A activation deferred.
2. Old path still referenced in devenv.toml — Watcher Class-D (four-surface drift); patch devenv.toml before claiming green.
3. Vault Edit tool fails on stale-content collision (concurrent fleet writes) — switch to `cat >> file` pattern per CLAUDE.md § Concurrent File Editing.

**Rollback:** `mv ~/claude-code-workspace/workflow-trace/ ~/claude-code-workspace/the-workflow-engine/` then revert wikilink edits via git.

**Watcher class pre-positioned:** A (rename activation) + D (anchor-drift surveillance).

**Atuin anchors:** `atuin search "mv.*the-workflow-engine.*workflow-trace"` ; `atuin search "grep -rln the-workflow-engine"`.

---

## G3 — :8125 REDEPLOY VERIFY

**Owner:** Luke (terminal restart); Command-3 (verify); Zen (post-verify). **Watcher class A or I.** **Mitigates:** AP-WT-F7 (CR-2 graceful-degrade pretend-fix), AP-Hab-13 (probe-freshness drift).

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)

# Step 1 — Luke (terminal):
# ~/.local/bin/devenv -c ~/.config/devenv/devenv.toml restart povm-engine

# Step 2 — Command-3 verifies new binary live + reads learning_health:
health=$(curl -s -m 2 http://localhost:8125/health 2>&1)
echo "${health}" | tee /tmp/g3-health.txt

lh=$(curl -s -m 2 http://localhost:8125/learning_health 2>&1 | jq -r '.learning_health // "ERR"')
echo "learning_health=${lh}" | tee /tmp/g3-lh.txt

# Expected band per Watcher D4: [0.05, 0.15]
test "${lh}" != "ERR" || { echo "FAIL: learning_health unreadable"; exit 3; }

# Step 3 — Command-3 files verify:
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command3_g3_verify.md <<EOF
G3 VERIFY: learning_health=${lh}, ts=${TS}, source=live :8125/learning_health.
Expected band: [0.05, 0.15] per Watcher D4 post-CR-2 prediction.
EOF

# Step 4 — Zen post-verify (request via file-drop, wait for response):
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_g3_zen_request.md <<EOF
AUDIT-REQUEST G3: confirm learning_health=${lh} is post-CR-2 honest reading.
EOF
# (Zen files response in agent-cross-talk/ when available.)

# Step 5 — D-G3 after BOTH Command-3 + Zen on record:
```

**Verification:** `learning_health` ∈ [0.05, 0.15]; Command-3 + Zen confirmations both filed; D-G3 references both.

**Failure modes:**
1. `learning_health > 0.15` — CR-2 not installed; binary-source mismatch — Watcher Class-A; rebuild from source `e2a8ed3 + 76ea4d6` BEFORE retry.
2. `learning_health < 0.05` — substrate degraded beyond prediction — Watcher Class-I; do NOT proceed silently; G3 reading becomes new T1 baseline for G5 F2 calibration.
3. Port :8125 unreachable — Watcher Class-H; Luke must investigate devenv start order.

**Rollback:** none of the restart itself (old binary not preserved). If reading degraded, document in D-G3; G4-G5 calibrate against degraded reading.

**Watcher class pre-positioned:** A (activation) + I (Hebbian-silence backstop).

**Atuin anchors:** `atuin search "devenv restart povm-engine"` ; `atuin search "curl.*8125/learning_health"`.

---

## G4 — WATCHER NOTES (Ember §5.1 amendment)

**Owner:** Watcher (sole author for §5.1); Zen (re-confirm); Command (D-G4 + CLAUDE.local.md update). **Watcher class A.** **Mitigates:** AP-WT-F7 (Ember CI bypass).

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
EMBER="${HOME}/projects/claude_code/Ember 7-Trait Gate Rubric.md"

# Step 1 — Watcher amends §5.1 (Watcher's lane; Command does NOT touch this file).
# Expected delta: "Held verdict: emit warning" → 
# "Held verdict: CI FAIL unless entry present in tests/ember_held_approvals.tsv
#  (reviewed and signed by Watcher + Zen)"

# Step 2 — Command verifies amendment landed:
/usr/bin/grep -n -E "CI FAIL|ember_held_approvals" "${EMBER}" | tee /tmp/g4-amendment.txt
test "${PIPESTATUS[0]}" -eq 0 || { echo "FAIL: amendment not detected"; exit 4; }

# Step 3 — Zen re-confirm via agent-cross-talk:
ls -t ~/projects/shared-context/agent-cross-talk/ | /usr/bin/grep -iE "g4|ember|held" | head -5 \
  | tee /tmp/g4-zen-confirm.txt

# Step 4 — Update workspace-root CLAUDE.local.md Hebbian v3 row (G4 green flag).
# Note: this is the ONLY authorised CLAUDE.local.md edit pre-G9 (per project CLAUDE.md PRIME DIRECTIVE).

# Step 5 — file D-G4:
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_d_g4.md <<EOF
D-G4 (G4 WATCHER NOTES): GREEN at ${TS}.
Hebbian v3 reconciliation: PASS-WITH-MINOR-AMEND (Zen 2026-05-16T224430Z) — already green.
Ember §5.1 amendment: $(cat /tmp/g4-amendment.txt | head -1)
Zen re-confirm: $(cat /tmp/g4-zen-confirm.txt | head -1)
EOF
```

**Verification:** §5.1 contains "CI FAIL" or "ember_held_approvals" string; Zen re-confirm filed; D-G4 references both.

**Failure modes:**
1. Watcher amendment fails Zen re-review (too permissive/strict) — Watcher Class-A; iterate.
2. m10 spec not yet updated to reference amendment — green G4 but m10 build-time break at Phase 1 Day 1; catch at G7 audit.
3. Watcher in R13 cold-start quiet — no deadline; G4 waits.

**Rollback:** Watcher edits §5.1 back; reversible.

**Watcher class pre-positioned:** A.

**Atuin anchors:** `atuin search "grep.*Ember.*Held"`.

---

## G5 — GENESIS INTERVIEW + F2 HARD GATE

**Owner:** Command-2 (chair); Watcher + Zen (sync participants); Command (facilitation + D-G5). **Watcher class C.** **Mitigates:** AP-WT-F2 (sample-size inflation).

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
TRANSCRIPT="${HOME}/claude-code-workspace/workflow-trace/ai_docs/GENESIS_INTERVIEW_TRANSCRIPT_G5.md"

# Step 1 — Apply OI-10 patch to interview bank BEFORE running:
# Edit ~/claude-code-workspace/workflow-trace/INTERVIEW_QUESTION_BANK_DRAFT.md
#   Q1.3 Conductor posture → single-phase
#   Q2.4 sunset semantics → m11 sunset-only (no Phase B gate)
# Then commit edit, not via sed.

# Step 2 — Run 3 rounds × 4 questions structure (sequential, Battern-style):
# Round 1 (scope+boundary) → Round 2 (architecture+substrate) → Round 3 (F2+sunset+failure-modes)
# Each round persists to transcript via Write tool.

# Step 3 — verify F2 rules present per report type:
f2count=$(/usr/bin/grep -cE "n>=20|n≥20|Wilson" "${TRANSCRIPT}" 2>/dev/null || echo 0)
test "${f2count}" -ge 4 && echo "OK: F2 rules present (${f2count})" \
  || { echo "FAIL: F2 rules absent (${f2count}<4)"; exit 5; }

# Step 4 — verify OI-3 + OI-4 resolved (26 modules + m1 unpadded canonical):
/usr/bin/grep -cE "26 modules|m1[^0-9]" "${TRANSCRIPT}" | head -3

# Step 5 — file D-G5:
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_d_g5.md <<EOF
D-G5 (G5 GENESIS INTERVIEW): GREEN at ${TS}.
Transcript: ${TRANSCRIPT}
F2 rules: ${f2count} per-report-type definitions present.
OI-3 (count): 26 modules canonical
OI-4 (naming): m1 unpadded canonical
Watcher participation: logged in transcript header
Zen participation: logged in transcript header
EOF
```

**Verification:** transcript exists; ≥4 F2 per-report rules; OI-3+OI-4 resolved; both sync participants logged.

**Failure modes:**
1. F2 stated at session-grain only — Watcher Class-C; interview continues.
2. Only 2 of 3 rounds completed (Zen/Watcher absent) — Watcher Class-A; schedule re-run.
3. Module-count differs from 26 — must reconcile before close; new count feeds v1.3.

**Rollback:** transcript is planning doc; G7 REFUSE → re-fire G5 with versioned transcript (no deletion).

**Watcher class pre-positioned:** C.

**Atuin anchors:** `atuin search "grep.*n>=20"` ; `atuin search "GENESIS_INTERVIEW_TRANSCRIPT"`.

---

## G6 — DUAL-FRAME GAP ANALYSIS

**Owner:** Command-2 (primary author both frames). **Watcher class D + G.** **Mitigates:** AP-V7-09 (substrate-frame confusion).

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
AI="${HOME}/claude-code-workspace/workflow-trace/ai_docs"

# Step 1 — Author both frames in SAME session (per Working Mode rule).
# ${AI}/GAP_ANALYSIS_ANTHROPOCENTRIC_G6.md  (≥500w; bidi-link to G5 transcript)
# ${AI}/GAP_ANALYSIS_SUBSTRATE_FRAME_G6.md  (≥500w; bidi-link to G5 transcript)

# Step 2 — verify both files:
ls "${AI}/GAP_ANALYSIS_"*"_G6.md" 2>&1 | wc -l   # expect 2
for f in "${AI}/GAP_ANALYSIS_"*"_G6.md"; do
  /usr/bin/grep -l "GENESIS_INTERVIEW_TRANSCRIPT_G5" "$f" \
    || { echo "FAIL: missing bidi-link in $f"; exit 6; }
  wc=$(wc -w < "$f"); test "$wc" -ge 500 \
    || { echo "FAIL: ${f} <500w (${wc})"; exit 6; }
done

# Step 3 — file D-G6:
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_d_g6.md <<EOF
D-G6 (G6 DUAL-FRAME GAP ANALYSIS): GREEN at ${TS}.
Anthropocentric: ${AI}/GAP_ANALYSIS_ANTHROPOCENTRIC_G6.md
Substrate-frame: ${AI}/GAP_ANALYSIS_SUBSTRATE_FRAME_G6.md
Both ≥500w + bidi-link to G5 transcript.
EOF
```

**Verification:** 2 files exist; each ≥500w; each contains bidi-link to G5 transcript; D-G6 filed.

**Failure modes:**
1. Substrate-frame file absent — Watcher Class-D; both passes required.
2. Substrate-frame recommends L9 m50+ modules — R6 frame-separation breach; Watcher Class-G; Command refuses + notes reservation.
3. Gap analysis exposes structural hole requiring >5-file fix — pause G6; surface to Luke.

**Rollback:** files are planning; G7 REFUSE cites finding → amend in place with dated note.

**Watcher class pre-positioned:** D + G.

**Atuin anchors:** `atuin search "GAP_ANALYSIS.*G6"`.

---

## G7 — ZEN SPEC AUDIT (highest-leverage gate)

**Owner:** Zen (verdict — sole authority); Command (submit + D-G7); Watcher (Class-A verbatim timestamp). **Mitigates:** AP-Drift-01 (over-claim gate-clean), AP-V7-01 (7-gen drift).

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
PROMPT="${HOME}/claude-code-workspace/workflow-trace/ai_docs/GENESIS_PROMPT_V1_3.md"

# Step 1 — verify v1.3 patch authored and complete:
test -f "${PROMPT}" || { echo "FAIL: v1.3 patch missing"; exit 7; }
/usr/bin/grep -cE "26 modules|m33 verifier|active.verb|F2" "${PROMPT}" \
  | tee /tmp/g7-v1_3-coverage.txt

# Step 2 — power-structure ambiguity resolved? (B6 in CLAUDE.local.md)
# Luke must have clarified: G7 REFUSE = (a) hard-stop+per-clause waiver OR (b) AMEND-loop.
# Without this, do NOT submit. Document in D-G7 if pending.

# Step 3 — file audit request to Zen:
REQ="${HOME}/projects/shared-context/agent-cross-talk/${TS}_command_g7_audit_request.md"
cat > "${REQ}" <<EOF
AUDIT-REQUEST — G7 ZEN SPEC AUDIT
From: Command
To: Zen (Tab 10)
Materials:
  1. ${PROMPT}
  2. ${HOME}/claude-code-workspace/workflow-trace/the-workflow-engine-vault/module specs/cluster-{A..H}.md
  3. ${HOME}/claude-code-workspace/workflow-trace/ai_docs/GENESIS_INTERVIEW_TRANSCRIPT_G5.md
  4. ${HOME}/claude-code-workspace/workflow-trace/ai_docs/GAP_ANALYSIS_*_G6.md

Verdict requested: APPROVE / REFUSE / AMEND.
Mandatory: confirm F2 (n≥20 + Wilson 95% CI) per report type.
EOF

# Step 4 — poll for Zen verdict:
verdict=""
while [[ -z "$verdict" ]]; do
  v=$(/usr/bin/grep -lE "APPROVE|REFUSE|AMEND" \
    ~/projects/shared-context/agent-cross-talk/*_zen_*.md 2>/dev/null | tail -1)
  [[ -n "$v" ]] && verdict="$v"
  # In live ops: this loop is NOT a busy-wait — Command does other work; Watcher pings on Zen drop.
  break  # planning-only — real loop is event-driven
done
echo "verdict_file=${verdict}"

# Step 5 — branch on verdict + file D-G7:
# APPROVE → proceed to G8
# AMEND   → apply amendments, resubmit, G7 re-fires (no Luke waiver per B6 protocol)
# REFUSE  → escalate to Luke per B6 resolution
```

**Verification:** verdict file contains APPROVE; D-G7 filed; Watcher Class-A timestamp logged.

**Failure modes:**
1. REFUSE on active-verb modules — Watcher Class-A highest-leverage; escalate to Luke per B6.
2. v1.3 missing OI-3/OI-4 resolution — Zen REFUSE expected; complete patch before submission.
3. F2 absent from v1.3 — Zen REFUSE per §G7 hard gate; add F2 definitions + resubmit.
4. Zen slow to respond — not a failure; wait.

**Rollback:** AMEND is iteration; REFUSE is escalation; true rollback only if Luke reverts to 11-module Phase-A — requires fresh G5.

**Watcher class pre-positioned:** A (verbatim timestamp of verdict moment).

**Atuin anchors:** `atuin search "G7 audit request"` ; `atuin search "grep.*APPROVE.*agent-cross-talk"`.

---

## G8 — FOUR-SURFACE PERSISTENCE

**Owner:** Command (all 4 surfaces); Watcher (journal). **Mitigates:** AP-Hab-03 (AP30 namespace violation), AP-V7-06 (bidi rot).

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
PROMPT="${HOME}/claude-code-workspace/workflow-trace/ai_docs/GENESIS_PROMPT_V1_3.md"
VAULT_MIRROR="${HOME}/projects/claude_code/workflow-trace"

# Surface 1 — ai_docs canonical (already exists from G7 prep):
test -f "${PROMPT}" || { echo "FAIL S1"; exit 8; }

# Surface 2 — Obsidian vault mirror:
mkdir -p "${VAULT_MIRROR}"
# Author ${VAULT_MIRROR}/workflow-trace Genesis Prompt v1.3.md via Write tool
# with "> Back to: [[CLAUDE.md]] · [[CLAUDE.local.md]] · [path to canonical]"

# Surface 3 — stcortex namespace anchor (HARD: AP30 prefix discipline):
~/.local/bin/stcortex status | tee /tmp/g8-stcortex.txt
test "${PIPESTATUS[0]}" -eq 0 && {
  # via mcp__stcortex-mcp__stcortex_write_memory tool — namespace 'workflow_trace_genesis'
  echo "WRITE: workflow_trace_genesis_g8_${TS}"
} || {
  echo "SKIP: stcortex :3000 DOWN — document in D-G8; queue write for when live"
}

# Surface 4 — workspace-root CLAUDE.local.md row update:
# Add row to ~/claude-code-workspace/CLAUDE.local.md § Active Workstreams:
#   "workflow-trace | G8 green; G9 pending | ${PROMPT} · stcortex id <X> | G9 signal"
# Use Edit tool (per anti-pattern table; never sed -i)

# Surface bidi link verification:
for surface in "${PROMPT}" "${VAULT_MIRROR}/workflow-trace Genesis Prompt v1.3.md"; do
  links=$(/usr/bin/grep -cE "ai_docs|vault|stcortex|CLAUDE.local" "${surface}" 2>/dev/null)
  test "${links}" -ge 3 || { echo "FAIL bidi: ${surface} (${links}<3)"; exit 8; }
done

# File D-G8:
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_d_g8.md <<EOF
D-G8 (G8 FOUR-SURFACE PERSISTENCE): GREEN at ${TS}.
S1 ai_docs: ${PROMPT}
S2 vault:   ${VAULT_MIRROR}/workflow-trace Genesis Prompt v1.3.md
S3 stcortex: workflow_trace_genesis_g8_${TS} (or SKIPPED: stcortex down)
S4 CLAUDE.local.md: row added with G8-green flag
Bidi links: all 4 cross-referenced.
EOF
```

**Verification:** all 4 surfaces exist; each contains links to ≥2 others; stcortex anchor or documented skip; CLAUDE.local.md row reflects G8 green.

**Failure modes:**
1. stcortex :3000 unreachable — do NOT silently fall back to POVM (CLAUDE.md memory-row-8 rule); document skip; retry when live.
2. Vault mirror missing bidi-back-link — G8 partial; fix before claim.
3. CLAUDE.local.md row says "HOLD-v2" still — update to "G8 green; G9 pending" before D-G8.

**Rollback:** G8 is additive; surfaces remain valid indefinitely.

**Watcher class pre-positioned:** D (four-surface drift).

**Atuin anchors:** `atuin search "stcortex.*workflow_trace_genesis"` ; `atuin search "grep.*workflow-trace.*CLAUDE.local"`.

---

## G9 — EXPLICIT START-CODING SIGNAL

**Owner:** Luke @ node 0.A (sole authority); Command (D-G9 record + first authorised `cargo init`); Watcher (Class-A verbatim). **Mitigates:** AP-Hab-01 (AP24 violation), AP-V7-08 (handshake silence false-success).

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)

# Step 1 — Luke types exactly:  start coding workflow-trace
# Variants (workflow_trace, the-workflow-engine, etc.) → Command asks for canonical confirmation.

# Step 2 — first authorised shell command after G9:
cd ${HOME}/claude-code-workspace/workflow-trace/
cargo init --name workflow-trace 2>&1 | tee /tmp/g9-cargo-init.txt
test "${PIPESTATUS[0]}" -eq 0 || { echo "FAIL: cargo init"; exit 9; }
ls Cargo.toml src/main.rs

# Step 3 — D-G9:
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_d_g9.md <<EOF
D-G9 (G9 EXPLICIT START-CODING SIGNAL): FIRED at ${TS}.
Luke signal (verbatim): "start coding workflow-trace"
First cargo init: $(git log --oneline -1 2>/dev/null || echo "pre-commit")
Phase 1 (runbook-01) authorised.
EOF

# Step 4 — Watcher notify:
~/.local/bin/watcher notify g9_fired "AP24 satisfied. cargo init complete. Phase 1 entered. SHA: $(git log --oneline -1)"
```

**Verification:** Luke's verbatim text recorded in D-G9; `cargo init` succeeded; SHA captured; Watcher notified.

**Failure modes:**
1. Luke types "start coding" without project name — AP24 requires project; ask clarification.
2. Luke types a variant — Command confirms canonical "workflow-trace" before proceeding.
3. G1-G8 gate retrospectively fails post-G9 — fix as D+1 action; Phase 1 proceeds with logged gap.

**Rollback:** G9 cannot be rolled back (signal is granted authorisation). Out-of-sequence G9 from 2026-05-17T08:43Z does NOT auto-apply when G1-G8 green; Luke must **re-issue explicitly**.

**Watcher class pre-positioned:** A (highest-leverage activation) + F (AP24 boundary check).

**Atuin anchors:** Luke's chat text + `atuin search "cargo init.*workflow-trace"` adjacent in history = canonical provenance.

---

## Phase-end gate (must be green before runbook-01)

| Check | Command | Pass criterion |
|---|---|---|
| G1-G9 D-files filed | `ls ~/projects/shared-context/agent-cross-talk/*_d_g[1-9].md \| wc -l` | =9 |
| `cargo init` succeeded | `git log --oneline -1` | non-empty |
| stcortex G8 anchor present (or skip documented) | grep stcortex_g8 in D-G8 | match |
| CLAUDE.local.md row updated | `/usr/bin/grep "G8 green" ~/claude-code-workspace/CLAUDE.local.md` | match |
| Watcher Class-A timestamps for G7+G9 | `~/.local/bin/watcher list \| /usr/bin/grep -E "g7\|g9"` | ≥2 entries |
| HOLD-v2 envelope formally lifted | D-G9 references "Phase 1 authorised" | match |

If any check fails → return to the failing gate; do NOT advance to `runbook-01`.

---

## Failure modes register (Phase-wide)

| # | Failure | Detection | Mitigation |
|---|---|---|---|
| 1 | **Out-of-sequence G9 reissued without re-verifying G1-G8** | D-G9 missing reference to D-G1..D-G8 | hard refusal; Watcher Class-F |
| 2 | **stcortex namespace collision** (`workflow_trace_*` clashes with legacy `workflow_engine_*` or unprefixed) | `stcortex sql "SELECT DISTINCT namespace FROM memory WHERE namespace LIKE 'workflow%'"` shows multiple prefixes | enforce m9 namespace guard pre-G8; rename legacy to `workflow_engine_legacy_*` |
| 3 | **Zen REFUSE with no B6 protocol** | G7 REFUSE filed; D-G7 cannot find B6 resolution | escalate to Luke immediately; do NOT proceed to G8 |
| 4 | **Probe-freshness drift** (G3 reading cited days later in G5 without re-probe) | timestamp of G3 D-file vs G5 transcript spans >24h | re-probe `:8125/learning_health` at G5 open; document fresh value |
| 5 | **Handshake dual-silence at G5** (C-2 + C-3 silent ≥30 min) | `find ~/projects/shared-context/agent-cross-talk -newer ${last_handshake} -mmin -30` returns 0 | file escalation to Luke (AP-V7-08); do NOT proceed assuming ack |
| 6 | **PIPESTATUS swallowed in gate chain** (any G3/G5/G8 script piped to tail) | gate prints PASS but stderr shows error | always `${PIPESTATUS[0]}` + per-stage abort (AP-Hab-05) |
| 7 | **`cp -f` alias trap during G2 wikilink update** | Edit tool fails or sed -i invoked | use Edit tool exclusively; if scripting, `/usr/bin/cp -f` per AP-Hab-06 |

---

## Watcher flag pre-positioning (Phase 0)

| Class | When activates | Specific gate |
|---|---|---|
| **A** activation transition | G1, G2, G3, G5, G7, G9 | every gate-flip moment; Watcher timestamps verbatim |
| **B** hand-off boundary | G8 (Claude session → stcortex DB) | first `workflow_trace_*` memory write |
| **C** confidence-gate refusal | G5, G7 | F2 below threshold; Zen REFUSE |
| **D** four-surface drift | G2, G6, G8 | bidi anchor rot; vault mirror missing back-link |
| **F** AP24 violation | pre-G9 throughout | any `src/` or `Cargo.toml` appearance |
| **G** substrate-frame confusion | G6 | substrate-frame doc proposing L9 m50+ |
| **H** atuin proprioception anomaly | G3, G8 | devenv restart issued but service unreachable |
| **I** Hebbian silence | G3 | LTP/LTD below floor (currently firing live per tick·16) |

**Watcher tick cadence:** prompt-driven (R13 elapsed); no autonomous loop. Command pings Watcher at each gate flip via `~/.local/bin/watcher notify`.

---

## Atuin trajectory anchors (Phase 0)

Run at end of Phase 0 to confirm full provenance:

```bash
atuin search "watcher G1" --before 7d
atuin search "mv.*the-workflow-engine.*workflow-trace" --before 7d
atuin search "devenv restart povm-engine" --before 7d
atuin search "curl.*8125/learning_health" --before 7d
atuin search "GENESIS_INTERVIEW_TRANSCRIPT" --before 7d
atuin search "GAP_ANALYSIS.*G6" --before 7d
atuin search "G7 audit request" --before 7d
atuin search "stcortex.*workflow_trace_genesis" --before 7d
atuin search "cargo init.*workflow-trace" --before 7d
# Each should return ≥1 hit; gaps indicate steps run outside atuin shell → Class-H flag
```

---

*runbook-00 authored 2026-05-17 by Command (V7 author wave subagent)*
