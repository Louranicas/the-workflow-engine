---
title: Runbook 05 — Phase 4 Pre-Deploy Hardening (Days 26-28)
date: 2026-05-17 (S1001982)
kind: planning-only operational runbook
phase: 4
days: 26-28 (3 calendar days; 3 sequential waves)
owner: Command (orchestrator); 4-agent dispatch (security-auditor · performance-engineer · silent-failure-hunter · zen); Watcher (witness)
status: planning-only · activates after phase-3-complete (runbook-04 close)
---

# Runbook 05 — Phase 4 Pre-Deploy Hardening (Days 26-28) — 4-AGENT GATE

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · sibling: [[runbook-04-phase-3-integration]] · runbook-06-phase-5-deploy-soak (forthcoming)
>
> Cites: `the-workflow-engine-vault/deployment framework/phase-4-pre-deploy-hardening.md` (narrative source). Cross-ref: [[../STANDARDS/GOD_TIER_RUST.md]] (4-stage QG; verify-sync 1-20), [[../ANTIPATTERNS_REGISTER.md]] (AP-Hab + AP-WT + AP-Drift), [[../KEYWORDS_20.md]] (PIPESTATUS, EscapeSurfaceProfile, four-surface).

---

## Overview

Phase 4 is the 3-day, 3-wave pre-deploy gate. **No binary lands in `~/.local/bin/` without Wave-3 Watcher witness.** Wave 1 (mechanical gate) must pass before any agent dispatch — running 4 agents against broken code wastes compute. Wave 2 dispatches 4 specialist agents in parallel (security / performance / silent-failure / zen) — each catches a class the others structurally miss. Each agent writes verdict to `$WORK/<agent>.md`. Boolean AND required across all 4: any single REJECT → halt. Wave 3 is Watcher witness + deployment-readiness receipt. The gate before this runbook is `phase-3-complete` tag. The gate after is `phase-4-complete` tag → control transfers to `runbook-06-phase-5-deploy-soak.md` (out of scope here).

**The 4-agent rule is non-negotiable.** Per `feedback_no_shortcuts.md`: "each phase gets own impl/QG/deploy/verify cycle, no merging under pressure". Compressing to one day = phase-collapse anti-pattern.

---

## Pre-flight checklist

```bash
set -o pipefail
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
SESSION="predeploy-$(date -u +%Y%m%d-%H%M)"
WORK="/tmp/predeploy-hardening-${SESSION}"
mkdir -p "$WORK"
WF_ROOT="/home/louranicas/claude-code-workspace/workflow-trace"
cd "$WF_ROOT"

# (1) phase-3-complete tag both remotes:
git ls-remote --tags origin | /usr/bin/grep "phase-3-complete" | tee /tmp/p4-pre-origin.txt
git ls-remote --tags gitlab | /usr/bin/grep "phase-3-complete" | tee /tmp/p4-pre-gitlab.txt
test -s /tmp/p4-pre-origin.txt && test -s /tmp/p4-pre-gitlab.txt || exit 1

# (2) verify-sync 1-20 fresh:
./scripts/verify-sync.sh --invariants 1-20 2>&1 | tee /tmp/p4-pre-vs.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 1

# (3) Binary SHA pre-anchors (in case prior versions deployed):
sha256sum ~/.local/bin/wf-crystallise 2>/dev/null | cut -c1-16 > "$WORK/pre-sha-crystallise.txt" || echo "MISSING" > "$WORK/pre-sha-crystallise.txt"
sha256sum ~/.local/bin/wf-dispatch 2>/dev/null | cut -c1-16 > "$WORK/pre-sha-dispatch.txt" || echo "MISSING" > "$WORK/pre-sha-dispatch.txt"

# (4) Substrate baseline live re-probe (do NOT reuse Phase 3 close values per runbook-probe-freshness):
~/.local/bin/stcortex sql \
  "SELECT COUNT(*) FROM pathway WHERE namespace LIKE 'workflow_trace_%'" > "$WORK/sc-pw-count.txt"
curl -s "http://localhost:8125/pathways?prefix=workflow_trace_" | jq '.count' > "$WORK/povm-pw-count.txt"

echo "Session: $SESSION" > "$WORK/SESSION"
echo "Project: $WF_ROOT" >> "$WORK/SESSION"
echo "Pre-flight: $(date -u +%Y-%m-%dT%H%M%SZ)" >> "$WORK/SESSION"
```

---

## Day 26 morning — Wave 1: Mechanical Gate

**Goal:** 4-stage QG green; forge 8-trap audit; PRAGMA portability audit; Ember gate string enumeration.

### Step W1.1 — 4-stage QG with PIPESTATUS

```bash
GATE_LOG="$WORK/mechanical-gate.txt"

echo "## STAGE 1: cargo check --workspace --all-targets --all-features" | tee -a "$GATE_LOG"
cargo check --workspace --all-targets --all-features 2>&1 | tee -a "$GATE_LOG"
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "ABORT W1.1 STAGE 1"; exit 11; }

echo "## STAGE 2: clippy -D warnings" | tee -a "$GATE_LOG"
cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tee -a "$GATE_LOG"
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "ABORT W1.1 STAGE 2"; exit 12; }

echo "## STAGE 3: clippy -W pedantic -D warnings" | tee -a "$GATE_LOG"
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tee -a "$GATE_LOG"
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "ABORT W1.1 STAGE 3"; exit 13; }

echo "## STAGE 4: test --release --all-targets --all-features" | tee -a "$GATE_LOG"
CARGO_TARGET_DIR=./target cargo test --workspace --all-targets --all-features --release 2>&1 | tee -a "$GATE_LOG"
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "ABORT W1.1 STAGE 4"; exit 14; }

echo "WAVE 1.1 MECHANICAL GATE PASS" | tee -a "$GATE_LOG"
echo "Wave 1 PASS: $(date -u +%Y-%m-%dT%H%M%SZ)" > "$WORK/wave1-pass.txt"
```

**Zero tolerance:** 0 errors AND 0 warnings every stage. Pre-existing project warnings do NOT excuse new code (S226 + AP-Hab-14).

### Step W1.2 — Forge 8-trap audit (static against pre-release codebase)

```bash
AUDIT="$WORK/forge-trap-audit.txt"
{
  echo "## Forge 8-trap audit"
  # T1 cp alias:
  echo "T1: $(/usr/bin/grep -nrE '\bcp -f\b' src/ | /usr/bin/grep -v '/usr/bin/cp -f' | wc -l) raw cp -f sites"
  # T2 pkill exit 144:
  echo "T2: $(/usr/bin/grep -nrE 'pkill .*&&' src/ | wc -l) pkill-chained sites"
  # T3 SIGPIPE:
  echo "T3: spawn audit — all wf-crystallise daemons use tokio::spawn + channel fan-out: $(/usr/bin/grep -c "tokio::spawn" src/bin/wf_crystallise.rs)"
  # T4 CARGO_TARGET_DIR conflicts:
  echo "T4: two-binary split uses per-binary path: $(/usr/bin/grep -nE 'CARGO_TARGET_DIR' scripts/ Cargo.toml | head -3)"
  # T5 port occupied:
  echo "T5: wf-dispatch HTTP server: $(/usr/bin/grep -nE 'bind|listen' src/m32_dispatcher/ | wc -l) — should be 0 (CLI-first per G4 Axis 3)"
  # T6 health path variance:
  echo "T6: health path: $(/usr/bin/grep -nE '/health' src/ | head -3)"
  # T7 feature flag bypass:
  echo "T7: povm_calibrated is cfg not feature: $(/usr/bin/grep -nE 'cargo:rustc-cfg=povm_calibrated' build.rs)"
  # T8 stale PID files:
  echo "T8: m32 probes Conductor via live socket: $(/usr/bin/grep -nE 'probe_conductor' src/m32_dispatcher/)"
} | tee "$AUDIT"
```

### Step W1.3 — PRAGMA portability audit

```bash
/usr/bin/grep -nrE 'PRAGMA journal_mode' src/ --include="*.rs" | tee "$WORK/pragma-audit.txt"
# Expected: every db constructor (m7, m30, m32) references shared pragma sequence
fail=$(/usr/bin/grep -cE 'SqliteOpenFlags.*open' src/ --include="*.rs" | /usr/bin/grep -v ".*0$" | wc -l)
test "$fail" -eq 0 || echo "PRAGMA AUDIT: review raw flag openers"
```

### Step W1.4 — Ember gate full string enumeration

```bash
cargo test --workspace --all-targets --all-features --release -- ember_gate 2>&1 | tee "$WORK/ember-gate.txt"
# Verify zero Held-as-warning: § 5.1 amendment must be in force (G4 closed pre-G9)
/usr/bin/grep -E "HELD|HOLD" "$WORK/ember-gate.txt" | tee "$WORK/ember-held.txt"
test ! -s "$WORK/ember-held.txt" || { echo "WARN: Held verdicts present — confirm allowlist entries"; }
```

### Step W1.5 — Pre-flight snapshot baseline.json (Day 26 afternoon)

```bash
# Probes RE-EXECUTED LIVE — no stale Phase 3 numbers (AP-Hab-13):
cat > "$WORK/baseline.json" <<EOF
{
  "session": "$SESSION",
  "wave1_pass_at": "$(date -u +%Y-%m-%dT%H%M%SZ)",
  "binary_sha_pre": {
    "wf_crystallise": "$(cat $WORK/pre-sha-crystallise.txt)",
    "wf_dispatch":    "$(cat $WORK/pre-sha-dispatch.txt)"
  },
  "pathway_counts": {
    "stcortex_workflow_trace_ns": "$(cat $WORK/sc-pw-count.txt | tail -1)",
    "povm_workflow_trace_ns":     "$(cat $WORK/povm-pw-count.txt)"
  },
  "substrate_ltp_density": "$(~/.local/bin/stcortex sql \"SELECT (SELECT COUNT(*) FROM pathway WHERE weight > 0.5) * 1.0 / COUNT(*) FROM pathway\" | tail -1)",
  "m14_lift_baseline_raw": "$(target/release/wf-crystallise lift --raw 2>&1 | head -3 | tr '\n' '|')",
  "m14_lift_n_qualifying": "$(target/release/wf-crystallise lift --raw 2>&1 | /usr/bin/grep -oE 'n=[0-9]+' | head -1)",
  "cluster_test_counts": {
    "cluster_a": "$(/usr/bin/grep 'test result' $GATE_LOG | head -1)",
    "cluster_g": "$(/usr/bin/grep 'test result' $GATE_LOG | tail -3 | head -1)"
  },
  "ember_held_semantics_amendment_landed": $(test -z "$(grep HELD $WORK/ember-held.txt 2>/dev/null)" && echo "true" || echo "false"),
  "conductor_wave_maturity": {
    "wave_1b_live": "$(curl -s -m 1 http://localhost:8141/health 2>/dev/null | jq -r '.enforcer_active // \"unknown\"')",
    "conductor_dispatch_enabled": "${CONDUCTOR_DISPATCH_ENABLED:-unset}"
  },
  "freshness_anchor": "$(date -u +%Y-%m-%dT%H%M%SZ)"
}
EOF
echo "Baseline: $WORK/baseline.json"
```

**Watcher class:** A (Wave 1 pass = activation), D (baseline.json drift surveillance from Phase 3 close). **Mitigates:** AP-Hab-05 (PIPESTATUS), AP-Hab-13 (probe-freshness drift), AP-Hab-14 (god-tier dilution).

---

## Day 27 — Wave 2: 4-Agent Parallel Gate

**Dispatch all 4 agents in a SINGLE message (one Agent tool call per agent; no sequential dependency).** Each writes verdict to `$WORK/<agent>.md`. Collect ALL 4 before Wave-2 boolean-AND verdict.

### Agent 1 — security-auditor

**Scope:** vulnerabilities, attack surfaces, auth/authz gaps, EscapeSurfaceProfile correctness, habitat security traps.

**Dispatch command (Agent tool):**
```
agent: security-auditor
scope:
  - rg over src/ for secrets (glpat-, github_pat_, gho_, API tokens)
  - cargo audit on Cargo.lock (RUSTSEC advisories)
  - read m30 AcceptedWorkflow, m32 ConductorDispatchRequest, m9 namespace guard
  - read m32 probe_conductor — no http:// prefix (raw socket addr only per m24 gold standard)
  - verify CONDUCTOR_DISPATCH_ENABLED bypass-impossible
  - F8 owner: confirm no RPC/HTTP shortcut bypasses CC-4 human accept gate
  - F11 owner: confirm m4 cluster IDs opaque (FNV-1a XOR; no ALPHA/BETA/GAMMA leak)
  - EscapeSurfaceProfile cardinality = 5, Ord derive, ReadOnly < Destructive
output: $WORK/security.md with Verdict {APPROVE|APPROVE-WITH-NITS|REJECT}
```

**REJECT criteria (any one):** plaintext credential present; http:// prefix in outbound HTTP; m32 has direct exec path; CONDUCTOR_DISPATCH_ENABLED bypass possible; m9 namespace guard absent/bypassable; new RUSTSEC dep.

### Agent 2 — performance-engineer

**Scope:** hot-path allocations, async-blocking calls, channel backpressure, mutation overhead, criterion benches against budgets.

**Dispatch command:**
```
agent: performance-engineer
scope:
  - flamegraph on synthetic m20 PrefixSpan workload
  - criterion bench: m11 compute_decay_factor <100ns (G15 hot-path)
  - criterion bench: m4 cascade_correlator on 10k-row atuin window
  - criterion bench: m32 5-check sequence <50ms total
  - async-blocking audit (AP29): any sync HTTP in tokio::spawn?
  - channel backpressure: tokio::sync::mpsc with bounded capacity?
  - allocation audit: format!/String::from in m4/m20 hot loops?
output: $WORK/performance.md
```

**REJECT criteria:** m11 >100ns; m32 5-check >100ms; AP29 sync-in-async detected; allocation in inner loop.

### Agent 3 — silent-failure-hunter

**Scope:** swallowed errors, unwrap_or success-sentinels, Default-collapse, missing `?` propagation.

**Dispatch command:**
```
agent: silent-failure-hunter
scope:
  - rg ban: unwrap_or(true|false|0|Default::default) on health/consent/success paths
  - rg ban: .ok() without // rationale comment
  - rg audit: tokio::spawn returning JoinHandle silently dropped
  - rg audit: let _ = without // rationale comment
  - m13 SubstrateUnavailable path: never silent POVM fallback
  - m40 outbox: posted=false retries; never silent drop after max_attempts
  - m41 LCM timeout: explicit timeout; never indefinite block
  - m42 dual-path: no silent fallback on stcortex down
  - serde rename="type" trap: deserialize_with handles case variants
output: $WORK/silent-failure.md
```

**REJECT criteria:** any unwrap_or-success-sentinel; any .ok()-without-rationale; any silent fallback path.

### Agent 4 — zen (audit)

**Scope:** god-tier code quality, v1.3 spec adherence, verify-sync invariants, antipattern register coverage, Watcher class pre-positioning correctness.

**Dispatch command:**
```
agent: zen
scope:
  - re-read ai_docs/GENESIS_PROMPT_V1_3.md
  - verify all 26 modules present at src/mN_*/
  - verify-sync invariants 1-20 each independently checked
  - antipattern audit: every AP-Hab + AP-WT-F1..F11 + AP-Drift + AP-V7 mitigated
  - Watcher class pre-positioning per phase correctness
  - god-tier 18 rules adherence (GOD_TIER_RUST.md)
  - test-discipline: per-module ≥50 tests; KEYSTONE ≥90; mutation kill-rate ≥70%
output: $WORK/zen.md with verdict
```

**REJECT criteria:** any verify-sync invariant fails; any AP-WT-F1..F11 not mitigated; KEYSTONE m20 mutation kill-rate <70%.

### Step W2.collect — Verdict collection (Day 27 evening)

```bash
for agent in security performance silent-failure zen; do
  test -f "$WORK/${agent}.md" || { echo "MISSING: $agent verdict"; exit 21; }
done

# Boolean AND extraction:
RESULTS="$WORK/wave2-results.txt"
{
  echo "## Wave 2 boolean AND collection"
  for agent in security performance silent-failure zen; do
    verdict=$(/usr/bin/grep -oE "APPROVE|APPROVE-WITH-NITS|REJECT" "$WORK/${agent}.md" | head -1)
    echo "${agent}: ${verdict}"
  done
} | tee "$RESULTS"

# Decision:
rejects=$(/usr/bin/grep -c "REJECT" "$RESULTS")
nits=$(/usr/bin/grep -c "APPROVE-WITH-NITS" "$RESULTS")
test "$rejects" -eq 0 || { echo "HALT: $rejects REJECT(s) — DO NOT PROCEED"; exit 22; }
if [[ "$nits" -gt 0 ]]; then
  echo "DEGRADED: $nits APPROVE-WITH-NITS — Luke decision required"
  echo "DEGRADED — Luke decision queued" > "$WORK/wave2-status.txt"
else
  echo "PASS — all 4 APPROVE" > "$WORK/wave2-status.txt"
fi
```

**Watcher class:** C (per-agent refusal). **Mitigates:** AP-Hab-15 (flex-verify skip — Wave 2 IS the flex-verify), AP-Drift-01 (over-claim gate-clean).

---

## Day 28 morning — Decision Branch

```bash
status=$(cat "$WORK/wave2-status.txt")
case "$status" in
  "PASS"*)
    echo "Wave 2 PASS — proceed to Wave 3"
    ~/.local/bin/watcher notify wave2_pass "4-agent gate APPROVE; proceeding to Wave 3 Watcher witness"
    ;;
  "DEGRADED"*)
    echo "Wave 2 DEGRADED — file Luke decision drop:"
    cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_p4_wave2_degraded.md <<EOF
DECISION-REQUEST: Phase 4 Wave 2 DEGRADED.
APPROVE-WITH-NITS count: $nits
Per-agent verdict files: $WORK/{security,performance,silent-failure,zen}.md
Options: (a) merge with nits-as-followup; (b) iterate fixes + re-run Wave 2; (c) HALT.
EOF
    # Wait for Luke directive before proceeding.
    ;;
  *)
    echo "ABORT: Wave 2 REJECT or unknown status"
    exit 22
    ;;
esac
```

---

## Day 28 afternoon — Wave 3: Watcher Witness + Deployment-Readiness Receipt

```bash
# Step W3.1 — Watcher synchronous witness:
~/.local/bin/watcher notify wave3_witness_begin "Phase 4 Wave 3 — Watcher synchronous witness begins. Materials: $WORK/baseline.json + wave2-results.txt"

# Step W3.2 — Wave-end orchestrator checklist (per LCM Drift #1..#11 discipline):
./scripts/wave-end-checklist.sh 4 2>&1 | tee "$WORK/wave-end-checklist.txt"
test "${PIPESTATUS[0]}" -eq 0 || exit 31

# Step W3.3 — Re-run full --workspace 4-stage QG one more time (Wave-end independent re-exercise per AP-Drift-01):
cargo check --workspace --all-targets --all-features 2>&1 | tail -5
test "${PIPESTATUS[0]}" -eq 0 || exit 32
cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -5
test "${PIPESTATUS[0]}" -eq 0 || exit 33
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tail -5
test "${PIPESTATUS[0]}" -eq 0 || exit 34
cargo test --workspace --all-targets --all-features --release 2>&1 | tail -5
test "${PIPESTATUS[0]}" -eq 0 || exit 35

# Step W3.4 — Build release binaries (NOT deploy yet — deploy is Phase 5):
cargo build --release --bin wf-crystallise 2>&1 | tee "$WORK/build-crystallise.txt"
test "${PIPESTATUS[0]}" -eq 0 || exit 36
cargo build --release --bin wf-dispatch 2>&1 | tee "$WORK/build-dispatch.txt"
test "${PIPESTATUS[0]}" -eq 0 || exit 37

# Step W3.5 — Post-build binary SHA + size:
sha256sum target/release/wf-crystallise | cut -c1-16 > "$WORK/post-sha-crystallise.txt"
sha256sum target/release/wf-dispatch | cut -c1-16 > "$WORK/post-sha-dispatch.txt"
ls -la target/release/wf-crystallise target/release/wf-dispatch | tee "$WORK/post-build-meta.txt"

# Step W3.6 — Final SHIP_RECEIPT.md:
cat > "$WORK/SHIP_RECEIPT.md" <<EOF
# SHIP RECEIPT — workflow-trace Phase 4 close

**Session:** $SESSION
**Timestamp:** $(date -u +%Y-%m-%dT%H%M%SZ)

## Wave 1 — Mechanical Gate
- 4-stage QG: PASS (see mechanical-gate.txt)
- Forge 8-trap audit: see forge-trap-audit.txt
- PRAGMA portability: see pragma-audit.txt
- Ember gate: see ember-gate.txt

## Wave 2 — 4-Agent Parallel
$(cat $WORK/wave2-results.txt)

## Wave 3 — Watcher Witness + Re-Build
- Full 4-stage re-run: PASS
- Wave-end orchestrator checklist: PASS
- Binary SHA (post-build):
  - wf-crystallise: $(cat $WORK/post-sha-crystallise.txt)
  - wf-dispatch:    $(cat $WORK/post-sha-dispatch.txt)
- Pre-build (was-deployed) SHA:
  - wf-crystallise: $(cat $WORK/pre-sha-crystallise.txt)
  - wf-dispatch:    $(cat $WORK/pre-sha-dispatch.txt)

## Substrate baseline (Phase 4 pre-flight, re-probed live)
$(cat $WORK/baseline.json)

## Authorisation
Per CLAUDE.md AP-Hab-06 / feedback_binary_deployment.md:
Binary placement uses /usr/bin/cp -f ONLY. Bare cp is forbidden.

Phase 5 deploy authorised. See runbook-06-phase-5-deploy-soak.md for cp + verify + soak protocol.
EOF
cat "$WORK/SHIP_RECEIPT.md"

# Step W3.7 — Tag + push:
git tag -a "phase-4-complete" -m "Phase 4 close: 4-agent gate APPROVE; binaries built; SHIP_RECEIPT.md generated"
git push origin main --tags
git push gitlab main --tags 2>&1 | tee /tmp/p4-d28-push.txt

~/.local/bin/watcher notify phase4_close "SHIP RECEIPT generated. Binaries built. Phase 5 deploy authorised (see runbook-06)."
```

---

## Phase-end gate (must be green before runbook-06)

| Check | Command | Pass criterion |
|---|---|---|
| Wave 1 PASS file present | `cat $WORK/wave1-pass.txt` | timestamp |
| Wave 2 boolean AND PASS or DEGRADED-with-Luke-OK | `cat $WORK/wave2-status.txt` | PASS or DEGRADED + Luke decision filed |
| Wave 3 re-run 4-stage QG PASS | re-run | exit 0 |
| Wave-end orchestrator checklist | `./scripts/wave-end-checklist.sh 4` | PASS |
| Both binaries built (post-SHA captured) | `cat $WORK/post-sha-*.txt` | two non-empty SHAs |
| SHIP_RECEIPT.md complete | `wc -l $WORK/SHIP_RECEIPT.md` | ≥30 lines |
| phase-4-complete tag both remotes | `git ls-remote --tags origin/gitlab` | match |
| Forge 8-trap audit clean | `cat $WORK/forge-trap-audit.txt` | all 8 traps OK |
| baseline.json freshness <24h old | check freshness_anchor | <24h |

---

## Failure modes register

| # | Failure | Detection | Mitigation |
|---|---|---|---|
| 1 | **Wave 1 cargo check FAIL** | exit non-zero | fix; re-run; DO NOT dispatch agents on broken code |
| 2 | **Wave 1 pedantic FAIL** | clippy::pedantic warning | fix per GOD_TIER_RUST rule 3; AP-Hab-14 prohibits dilution |
| 3 | **Wave 2 agent missing verdict** | `$WORK/<agent>.md` absent | re-dispatch missing agent; DO NOT proceed |
| 4 | **Wave 2 REJECT** | rejects > 0 in wave2-results.txt | HALT; address; re-run Wave 2 |
| 5 | **DEGRADED but no Luke decision** | status=DEGRADED + 0 Luke drop within 24h | escalate; do NOT proceed to Wave 3 |
| 6 | **PRAGMA portability fails (raw flag opener)** | pragma-audit.txt shows raw opener | refactor through shared pragma helper |
| 7 | **Forge trap T6 health path mismatch** | devenv.toml path ≠ src expectation | align devenv.toml entry before deploy |
| 8 | **Build fails Wave 3** | cargo build --release errors | almost impossible (4-stage already passed); investigate cargo cache |
| 9 | **Post-SHA == Pre-SHA** (no actual rebuild) | sha256sum identical | force-rebuild with `cargo clean && cargo build --release`; AP-Drift-04 risk |
| 10 | **PIPESTATUS swallow** anywhere in Wave 1 | green PASS; stderr screaming | always `${PIPESTATUS[0]}` (AP-Hab-05) |
| 11 | **probe-freshness drift** (baseline.json reuses Phase 3 numbers) | freshness_anchor old | re-execute live probes before SHIP_RECEIPT (AP-Hab-13) |
| 12 | **Bare cp in build scripts** | grep `^cp ` in scripts | use `/usr/bin/cp -f` exclusively (AP-Hab-06) |

---

## Watcher flag pre-positioning (Phase 4)

| Class | Activates | Wave |
|---|---|---|
| **A** activation | Wave 1 PASS; Wave 2 boolean AND PASS; Wave 3 SHIP_RECEIPT generated | 1, 2, 3 |
| **B** hand-off boundary | binary deploy queued (Phase 5 next) | 3 |
| **C** confidence-gate refusal | per-agent REJECT in Wave 2 | 2 |
| **D** four-surface drift | SHIP_RECEIPT.md vs Phase 3 substrate baseline | 1, 3 |
| **F** AP24 boundary; god-tier dilution | every Wave 1 stage; every agent finding | continuous |
| **H** atuin proprioception | every audit command must appear in atuin | continuous |
| **I** Hebbian silence | substrate_LTP_density vs Phase 3 close | 1 baseline |

---

## Atuin trajectory anchors (Phase 4)

```bash
atuin search "cargo check --workspace --all-targets --all-features" --before 4d
atuin search "cargo clippy --workspace .* -W clippy::pedantic" --before 4d
atuin search "cargo build --release --bin wf-" --before 4d
atuin search "sha256sum.*target/release" --before 4d
atuin search "stcortex sql .* pathway WHERE namespace" --before 4d
atuin search "wave-end-checklist" --before 4d
atuin search "git tag.*phase-4-complete" --before 4d
atuin scripts run hab-quality-gate --before 4d
atuin scripts run habitat-fingerprint --before 4d
```

Each must return ≥1 hit. Gaps = Class-H. The forge-trap-audit + pragma-audit + ember-gate must each have an atuin entry as provenance.

---

*runbook-05 authored 2026-05-17 by Command (V7 author wave subagent)*
