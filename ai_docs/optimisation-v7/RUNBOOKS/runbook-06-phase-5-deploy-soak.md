---
title: Runbook 06 — Phase 5 Deploy + Cutover + 120d Soak
date: 2026-05-17 (S1001982)
kind: planning-only · operational runbook · post-G9 activation
phase: 5A (Day 28-30) · 5B (Day 30 cutover) · 5C (Day 30 → D120)
binary_targets: [wf-crystallise, wf-dispatch]
owner: Command (orchestration) + Luke @ terminal (devenv) + Watcher (carriage)
soak_window: D30 → D120 (90 days)
povm_cutover_in_window: true (~D25 mid-soak dance)
primary_watcher_flags: [A, B, D, I]
authority: Luke @ node 0.A
status: planning-only · HOLD-v2 active · NOT executable until G1-G9 GREEN
---

# Runbook 06 — Phase 5 Deploy + Cutover + 120-Day Soak

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · sibling [[runbook-05-phase-4-hardening.md]] · next [[runbook-07-phase-6-sunset.md]] · related [[runbook-08-phase-7-security.md]] · [[runbook-09-phase-8-observability.md]] · [[runbook-11-emergency-rollback.md]]
>
> Source phase docs: [[../../the-workflow-engine-vault/deployment framework/phase-5-deploy-and-soak.md]] + [[../../the-workflow-engine-vault/deployment framework/phase-6-sunset-and-cross-cutting.md]] + [[../../the-workflow-engine-vault/deployment framework/phase-8-observability-operations.md]]

---

## Overview

Phase 5 is the production-cutover and sustained-observation phase. Three sub-phases run sequentially: **5A** materialises `wf-crystallise` and `wf-dispatch` binaries to `~/.local/bin/` after a final 4-stage gate (Day 28-30); **5B** is the production-live cutover ceremony at Day 30 (Watcher Class-A timestamp, four-surface substrate touch, WCP notice); **5C** is the 90-day soak (Day 30 → D120) ending at the m11 sunset evaluation owned by [[runbook-07-phase-6-sunset.md]]. The POVM `:8125` deprecation deadline (2026-07-10) falls **inside** the soak at approximately D25 — m42's `povm_overlap_active` flag flips to `false` mid-flight; this is a separately-runnable cutover dance, not a re-deploy. workflow-trace is **CLI-only** (no devenv batch slot, no port, no `/health` HTTP endpoint) — `--version` is the canonical smoke probe.

---

## Pre-flight checklist

Before any 5A command runs, verify Phase 4 outputs are present and current:

- 4-stage QG fully clean per [[../STANDARDS/GOD_TIER_RUST.md]] (check → clippy → pedantic → test; `--workspace --all-targets --all-features`)
- `tests/ember_gate.rs` passing (no Rejected; no Held-as-fail per W3 unless Watcher §5.1 amendment landed)
- m8 build-prereq env: `POVM_CR2_DEPLOYED=1` exported in the deploy shell
- POVM `:8125` redeploy verified live (B5 blocker cleared) — `curl -s http://localhost:8125/health` returns 200 AND `learning_health` lands within `[0.05, 0.15]` magnitude-weighted band
- HABITAT-CONDUCTOR Waves 1B/1C/2/3 reachable (B3 blocker) — Luke ran `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start weaver zen enforcer` at terminal
- Git tag `v0.1.0-hardened` present in `git log` (Phase 4 close marker)
- `atuin scripts list | grep wt-` confirms 7 wt-* scripts authored per [[../GENERATIONS/G5-tooling.md]]
- Watcher journal entry recording Phase 4 close
- 11 services healthy at last `/intel` probe

If any item misses: HALT. Phase 5A does not proceed on partial pre-conditions (AP-Drift-11 — supervisor stub mistaken for live).

---

## Step 1 — Phase 5A binary build (Day 28-30)

**Inputs:** Phase 4 close commit SHA · `Cargo.toml` at workspace root · `POVM_CR2_DEPLOYED=1` env.

**Commands:**

```bash
# PIPESTATUS discipline mandatory at every stage (AP-Hab-05)
set -o pipefail

# Step 1.1 — Navigate to project root (absolute path — never relative)
cd /home/louranicas/claude-code-workspace/the-workflow-engine

# Step 1.2 — Verify m8 build-prereq env is set; m8 build.rs HARD-fails otherwise
[[ "${POVM_CR2_DEPLOYED:-0}" == "1" ]] || { echo "FAIL: POVM_CR2_DEPLOYED unset (AP-WT-F7)"; exit 1; }

# Step 1.3 — Per-project CARGO_TARGET_DIR (Forge trap 8; AP-V7-07 worktree contamination)
export CARGO_TARGET_DIR=/home/louranicas/claude-code-workspace/the-workflow-engine/target

# Step 1.4 — Final 4-stage gate before release build (canonical per GOD_TIER_RUST.md)
cargo check --workspace --all-targets --all-features 2>&1 | tail -10
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "STAGE 1 FAIL"; exit 1; }
cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -10
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "STAGE 2 FAIL"; exit 2; }
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tail -10
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "STAGE 3 FAIL"; exit 3; }
cargo test --workspace --all-targets --all-features --release 2>&1 | tail -30
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "STAGE 4 FAIL"; exit 4; }
echo "ALL 4 STAGES PASS"

# Step 1.5 — Release build (default features per Forge trap 6)
cargo build --release --bin wf-crystallise --bin wf-dispatch 2>&1 | tail -10
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "BUILD FAIL"; exit 5; }

# Step 1.6 — Materialise via /usr/bin/cp -f (AP-Hab-06 alias trap)
/usr/bin/cp -f "$CARGO_TARGET_DIR/release/wf-crystallise" "$HOME/.local/bin/wf-crystallise"
/usr/bin/cp -f "$CARGO_TARGET_DIR/release/wf-dispatch"   "$HOME/.local/bin/wf-dispatch"

# Step 1.7 — Verify non-zero size + executable (smoke pre-test)
[[ -s "$HOME/.local/bin/wf-crystallise" ]] || { echo "FAIL: wf-crystallise empty"; exit 6; }
[[ -s "$HOME/.local/bin/wf-dispatch"   ]] || { echo "FAIL: wf-dispatch empty"; exit 6; }

# Step 1.8 — Smoke (Forge trap 5; CLI health probe = --version)
wf-crystallise --version
wf-dispatch --help | head -20
wf-crystallise status --brief
```

**Outputs:** `~/.local/bin/wf-crystallise` + `~/.local/bin/wf-dispatch` (non-zero, executable); smoke output stdout.

**Verification:** atuin trajectory anchor + KV record.

```bash
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
atuin kv set "workflow_trace.deploy.phase5a.timestamp" "$TS"
atuin kv set "workflow_trace.deploy.phase5a.wf_crystallise_sha" \
  "$(sha256sum "$HOME/.local/bin/wf-crystallise" | cut -c1-16)"
atuin kv set "workflow_trace.deploy.phase5a.wf_dispatch_sha" \
  "$(sha256sum "$HOME/.local/bin/wf-dispatch" | cut -c1-16)"
atuin kv set "workflow_trace.deploy.phase5a.status" "binary_materialised"
atuin scripts run wt-bridge-check >> /tmp/wt-phase5a-bridges-${TS}.log 2>&1
```

**Failure modes:**

- F-5A-1: `learning_health` outside `[0.05, 0.15]` at startup → m8 build.rs hard-fails. Resolution: Luke `devenv restart povm-engine` then re-run from Step 1.4.
- F-5A-2: `cp` aliased (interactive prompt seen) → AP-Hab-06 hit. Hardcode `/usr/bin/cp -f` (already in step 1.6); never bare `cp`.
- F-5A-3: PIPESTATUS swallow — gate prints PASS while clippy errored → AP-Hab-05. Per-stage `[[ ${PIPESTATUS[0]} -ne 0 ]] && exit N` is mandatory.

**Watcher class:** Class-B (hand-off boundary — first build-env → binary-store crossing). Pre-position: Watcher records timestamp + SHAs verbatim on materialisation success.

---

## Step 2 — Phase 5B production cutover ceremony (Day 30)

**Inputs:** Phase 5A success · Conductor live (`:8141/health = 200`) · POVM live (`:8125/health = 200`) · stcortex live (`:3000`).

**Commands (pre-cutover D0 baseline snapshot):**

```bash
# 2.1 — Capture D0 baseline (denominator for 120-day m14 lift)
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
wf-crystallise status --json | python3 -c "
import sys, json
d = json.load(sys.stdin)
print('module_test_count:', d.get('module_test_count','?'))
print('schema_version:',    d.get('schema_version','?'))
" > /tmp/wt-d0-status-${TS}.log

# 2.2 — POVM substrate baseline (during overlap)
LTP=$(curl -s --max-time 2 http://localhost:8125/health \
  | python3 -c "import sys,json; print(json.load(sys.stdin).get('substrate_ltp_density','?'))")
LH=$(curl -s --max-time 2 http://localhost:8125/health \
  | python3 -c "import sys,json; print(json.load(sys.stdin).get('learning_health','?'))")

# 2.3 — Persist D0 baseline to atuin KV
atuin kv set "workflow_trace.soak.d0.lift_n"              "0"        # F2 expected None
atuin kv set "workflow_trace.soak.d0.lift_value"          "None"
atuin kv set "workflow_trace.soak.d0.substrate_ltp_density" "$LTP"
atuin kv set "workflow_trace.soak.d0.learning_health"     "$LH"
atuin kv set "workflow_trace.soak.d0.timestamp"           "$TS"

# 2.4 — Circuit breakers (m40/m41/m42) confirmed Closed
wf-crystallise status --json | python3 -c "
import sys, json
d = json.load(sys.stdin)
for k,v in d.get('circuit_breakers',{}).items(): print(f'cb.{k}: {v}')
"
```

**Cutover signal (deliberate ceremony, not auto):**

Luke/Command announces verbatim in the orchestrator pane:

```
workflow-trace production-live — Day 30 — soak clock starts now
```

Watcher records **Class-A** entry timestamping this verbatim. The timestamp becomes `D30` for all soak math.

**WCP notice (Command drops directly per `feedback_wcp_notify_weaver.md` — do NOT ask Luke to relay):**

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
cat > ~/projects/shared-context/watcher-notices/${TS}_workflow_trace_production_live.md <<EOF
# WCP Notice — workflow-trace Production-Live (D30)

**From:** Command (Tab 1 Orchestrator)
**To:** Watcher ☤
**Date:** ${TS}
**Event:** Phase 5B cutover ceremony complete. Soak clock started.

Carriage handoff: T0 baseline → T+30d production observation begins now.
Substrate D0 baseline in atuin KV \`workflow_trace.soak.d0.*\`.
m14 lift: None (n=0). Expected. F2 gate correct.
POVM cutover: T-$(python3 -c "import datetime;print((datetime.date(2026,7,10)-datetime.date.today()).days)")d (2026-07-10 deadline). m42 dual-path active.
Conductor: live (Waves 1B/1C/2/3).
Weekly synthesis cadence begins D37.
EOF
```

**First production crystalliser run:**

```bash
wf-crystallise sweep --window-days 7 --output-format json \
  >> /tmp/wt-d30-first-sweep.log 2>&1
echo "exit:$?"

# m14 returns None (n<20) — correct per F2 hard gate. Do NOT interpret as zero lift.
```

**First production dispatch (synthetic dry-run then live):**

```bash
# 2.5 — Conductor sanity probe before dispatch
curl -s --max-time 2 http://localhost:8141/health \
  | python3 -c "import sys,json;d=json.load(sys.stdin);print('conductor:',d.get('status','?'))"

# 2.6 — Dry-run first (m32 5-check sequence verification: Conductor live → m33 TTL fresh
#       → definition_hash match → sunset guard → dispatch cooldown)
wf-dispatch execute --workflow-id <synth-test-id> --dry-run \
  2>&1 | tee /tmp/wt-d30-dispatch-dry.log

# 2.7 — Live dispatch (only after dry-run green)
wf-dispatch execute --workflow-id <synth-test-id> \
  2>&1 | tee /tmp/wt-d30-dispatch-live.log
```

**Four-surface substrate touch verification (Phase 5B NOT complete until all 4 show writes):**

```bash
# Surface 1 — stcortex (canonical post-2026-07-10)
~/.local/bin/stcortex sql \
  "SELECT namespace, COUNT(*) FROM pathway WHERE namespace LIKE 'workflow_trace_%' GROUP BY namespace;"

# Surface 2 — POVM (overlap period only)
curl -s --max-time 2 "http://localhost:8125/pathways?namespace=workflow_trace" \
  | python3 -c "import sys,json;d=json.load(sys.stdin);print('povm rows:',len(d.get('pathways',[])))"

# Surface 3 — SYNTHEX outbox JSONL fan-out
ls -la /home/louranicas/claude-code-workspace/the-workflow-engine/outbox/ 2>/dev/null \
  || echo "outbox empty"

# Surface 4 — LCM if deploy-shape step ran
# (not required if dispatched workflow was read-only shape)
```

**Outputs:** Class-A Watcher entry · WCP notice file · D0 baseline KV · 4-surface confirmation · `/tmp/wt-d30-*.log`.

**Failure modes:**

- F-5B-1: Conductor returns 503 / refused → m32 emits typed `DispatchError::ConductorDispatchDisabled`. CORRECT behaviour (refuse-mode is the safe path per `EscapeSurfaceProfile` discipline). Luke restarts Conductor at terminal; re-run from Step 2.5.
- F-5B-2: 4-surface check shows fewer than 4 surfaces written → Phase 5B incomplete. Investigate stcortex consumer freshness (`~/.local/bin/stcortex consumers | grep workflow_trace`); if stale, m13 backpressure activated to local JSONL buffer (designed behaviour, but flag).
- F-5B-3: Zellij silent suspended-pane swallow on cutover announcement (AP-Hab-12) → dump-screen verify; use `zellij action write 13` for CR byte.

**Watcher class:** Class-A (activation transition; verbatim timestamp). Class-B (hand-off boundary — soak clock starts). Class-I pre-position (first m40 outbox emit — if outbox empty after first sweep, Cluster H is silent from Day 1, flag immediately).

---

## Step 3 — Phase 5C 90-day soak (Day 30 → D120) — continuous tracks

**Inputs:** Phase 5B success · D0 baseline KV · WCP notice filed · cutover ceremony timestamp.

### 3.1 — Daily cron schedule for `wf-crystallise sweep`

```bash
# /etc/cron.d/wf-crystallise OR crontab -e entry (Forge trap 7 — flock prevents overlap)
# NO set -e — wf-crystallise may exit non-zero on partial substrate (CLAUDE.md § Shell Scripting)
0 6 * * * /usr/bin/flock -n /tmp/wf-crystallise.lock \
  /home/louranicas/.local/bin/wf-crystallise sweep --window-days 1 \
  >> /tmp/wf-crystallise.log 2>&1; \
  echo "exit:$? ts:$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> /tmp/wf-crystallise.log
```

`flock -n` is **non-blocking** — overlapping invocations skip (correct behaviour; one-day accumulation gap is preferable to overlap).

### 3.2 — Weekly Watcher synthesis trigger (every 7d from D30)

```bash
# Command drops weekly WCP synthesis request
WEEK_N=$(( ($(date +%s) - $(date -d "<D30 cutover ts>" +%s)) / 86400 / 7 ))
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
cat > ~/projects/shared-context/watcher-notices/${TS}_workflow_trace_week_${WEEK_N}_synthesis_request.md <<EOF
# WCP — workflow-trace Week ${WEEK_N} Synthesis Request

**To:** Watcher ☤
**Cover:**
  - new Class-E flags (planning sprawl)
  - Class-I status (Hebbian silence; learning_health movement?)
  - new m15 pressure notices
  - m14 lift trajectory (n accumulation; first CI bars; direction)
  - m11 decay cycle state (any PrunePending pre-D60?)
  - Cluster H circuit breaker state
EOF
```

Watcher emits synthesis to `WATCHER_DEPLOYMENT_WATCH_JOURNAL_S1001982.md`. Cadence **prompt-driven** (no autonomous loop).

### 3.3 — m14 lift trend bands (per [[../../the-workflow-engine-vault/deployment framework/phase-5-deploy-and-soak.md]] § decay trajectory)

| Trajectory band | D60 weight | D90 weight | D120 weight | Action |
|---|---|---|---|---|
| **Healthy** | 0.97-0.99 | 0.94-0.98 | 0.90-0.97 | none |
| **Warning** (never-dispatched) | 0.30 | 0.08 | 0.02 | confirm m32 dispatch path alive; check m15 for alternative-dispatch pressure notices |
| **Alert** (premature prune) | < 0.10 | n/a | n/a | investigate m11 freq×fitness×recency formula; check substrate condition |

Use `wt-soak-pulse` atuin script (30s probe interval) per [[../GENERATIONS/G5-tooling.md]].

### 3.4 — POVM cutover ~D25 mid-soak dance

POVM `:8125` deprecation = 2026-07-10. If Day 0 deploy ≈ 2026-06-15, then ~D25 falls inside the soak. m42 dual-path flips:

```bash
# Pre-cutover (D < 25): m42 routes to BOTH stcortex (via m13) AND POVM (:8125)
atuin kv get "workflow_trace.povm_overlap_active"  # expected: "true"

# At T-1d (2026-07-09): pre-flip sanity check
curl -s --max-time 2 http://localhost:8125/health | python3 -c \
  "import sys,json;print('povm:',json.load(sys.stdin).get('status','?'))"
~/.local/bin/stcortex sql "SELECT COUNT(*) FROM pathway WHERE namespace LIKE 'workflow_trace_%';"

# At T-0 (2026-07-10 00:00Z): m42 reads config flip; routes EXCLUSIVELY through stcortex
# This is NOT a re-deploy — m42 reads the flag at each sweep start
atuin kv set "workflow_trace.povm_overlap_active" "false"

# Post-cutover (D > 25): verify no more :8125 reinforce calls
atuin history search --cwd /home/louranicas/claude-code-workspace/the-workflow-engine \
  --after "2026-07-10" | grep "8125/pathways" | wc -l
# Expected: 0

# Verify stcortex pathway growth post-cutover
~/.local/bin/stcortex sql \
  "SELECT COUNT(*) FROM pathway WHERE namespace LIKE 'workflow_trace_%' AND created_at > strftime('%s','2026-07-10') * 1000;"
```

**Rollback if cutover fails:** flip `povm_overlap_active=true` AND restart `wf-crystallise` cron pause:
```bash
atuin kv set "workflow_trace.povm_overlap_active" "true"
crontab -l | grep -v wf-crystallise | crontab -   # disable cron temporarily
# Then debug stcortex availability; do NOT silently re-route to POVM (CLAUDE.md § Memory Systems #8)
```

**Critical anti-pattern:** do NOT silently fall back to POVM if stcortex `:3000` unreachable. m13 backpressure writes to local JSONL buffer; deferral is the design.

### 3.5 — D60 mid-soak review (Luke decision: continue / amend_spec / early_sunset)

Decision criteria + KV anchor (see [[runbook-07-phase-6-sunset.md]] § D60 for full mid-soak logic):

```bash
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
atuin kv set "workflow_trace.soak.d60.lift_n"             "<n>"
atuin kv set "workflow_trace.soak.d60.lift_value"         "<lift or None>"
atuin kv set "workflow_trace.soak.d60.lift_ci_lower"      "<ci_lower or None>"
atuin kv set "workflow_trace.soak.d60.substrate_ltp_density" "<value>"
atuin kv set "workflow_trace.soak.d60.class_i_active"     "<true|false>"
atuin kv set "workflow_trace.soak.d60.luke_decision"      "<continue|amend_spec|early_sunset>"
atuin kv set "workflow_trace.soak.d60.timestamp"          "$TS"
```

### 3.6 — D90 pre-sunset preview

```bash
wf-crystallise report --format d90-preview >> /tmp/wt-d90-preview.log 2>&1
# Possible outputs: PASS | FAIL | DEGRADED | INSUFFICIENT_DATA
# FAIL at D90 with 30d remaining → notify Luke immediately
```

**Outputs:** KV anchors at D0 / D7 / D14 / ... / D60 / D90 / D120; Watcher journal weekly ticks; 12+ WCP notices in `watcher-notices/`; soak-arc synthesis at D120.

**Failure modes:**

- F-5C-1: cron silently stops (atuin gap >24h) → AP-Hab-13 runbook probe freshness. Detection: `atuin history list --cwd <project> | grep wf-crystallise | tail -2` shows >24h gap. Resolution: investigate `flock` lock staleness; cron daemon health.
- F-5C-2: substrate_LTP_density drops >10% week-over-week → ALERT-5; Watcher Class-G (substrate-frame confusion — engine may be corrupting substrate). HALT `wf-dispatch`; run `wf-crystallise` read-only; inspect m42 `fitness_delta` sign-flip per [[../../the-workflow-engine-vault/deployment framework/phase-8-observability-operations.md]] § Incident: m14 lift dropped suddenly.
- F-5C-3: `wf_m31_selection_weight max/sum > 0.5` → monoculture (BUG-035-equivalent). Increase diversity δ in m31 composite score; flag Class-G.

**Watcher class:** Class-I primary (Hebbian silence — sustained `learning_health` non-movement is the death signal); Class-E periodic (any new architectural proposal inflating planning-to-code ratio without code); Class-D (four-surface drift between m7 SQLite / stcortex / m30 bank / m15 JSONL); Class-A on D60 mid-soak decision + sunset extension events.

---

## Phase-end gate (D120 hand-off to Phase 6)

Phase 5C terminates at D120. Hand-off to [[runbook-07-phase-6-sunset.md]] requires:

```bash
# Mandatory hand-off package — Phase 6 will NOT proceed without these KVs
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
atuin kv set "workflow_trace.soak.d120.verdict"            "<PASS|FAIL|DEGRADED|INSUFFICIENT_DATA>"
atuin kv set "workflow_trace.soak.d120.lift_n"             "<n>"
atuin kv set "workflow_trace.soak.d120.lift_value"         "<lift or None>"
atuin kv set "workflow_trace.soak.d120.lift_ci_lower"      "<ci_lower or None>"
atuin kv set "workflow_trace.soak.d120.bank_active"        "<count>"
atuin kv set "workflow_trace.soak.d120.bank_prune_pending" "<count>"
atuin kv set "workflow_trace.soak.d120.bank_sunset_expired" "<count>"
atuin kv set "workflow_trace.soak.d120.timestamp"          "$TS"
```

Watcher emits 90-day soak-arc synthesis to `WATCHER_DEPLOYMENT_WATCH_JOURNAL_S1001982.md`. Final lift snapshot + CI bars printed to `/tmp/wt-d120-final.log`. Phase 6 [[runbook-07-phase-6-sunset.md]] reads these KVs as its first gate.

---

## Failure modes register (≥3 surfaced per sub-phase)

| ID | Sub-phase | Trigger | Detection | Mitigation | Antipattern |
|---|---|---|---|---|---|
| F-5A-1 | 5A | `learning_health` outside band at build | m8 build.rs hard-fail | Luke restart povm-engine; re-run gate | AP-WT-F7 |
| F-5A-2 | 5A | `cp` alias prompt | interactive prompt seen | hardcode `/usr/bin/cp -f` | AP-Hab-06 |
| F-5A-3 | 5A | PIPESTATUS swallow | gate prints PASS while clippy errored | `[[ ${PIPESTATUS[0]} -ne 0 ]] && exit N` | AP-Hab-05 |
| F-5B-1 | 5B | Conductor down | `:8141/health = 503` | typed `DispatchError`; Luke restart weaver | (correct behaviour) |
| F-5B-2 | 5B | 4-surface incomplete | < 4 surfaces show writes | check stcortex consumer freshness; investigate m13 backpressure | AP-WT-F8 |
| F-5B-3 | 5B | Zellij silent swallow on announcement | dump-screen unchanged | `zellij action write 13` CR byte | AP-Hab-12 |
| F-5C-1 | 5C | cron silently stops | atuin gap > 24h | investigate flock + cron daemon | AP-Hab-13 |
| F-5C-2 | 5C | substrate_LTP_density drops > 10% w/w | ALERT-5 | HALT dispatch; inspect m42 sign-flip | AP-V7-09 |
| F-5C-3 | 5C | m31 monoculture > 0.5 | wf_m31_selection_weight max/sum | bump diversity δ | (RALPH analog of BUG-035) |
| F-5C-4 | 5C | POVM cutover stcortex unreach | stcortex `:3000` refused at D25 | DO NOT fall back to POVM; m13 backpressure to JSONL | (CLAUDE.md row 8) |

---

## Watcher flag pre-positioning

| Class | Pre-position trigger | What it captures |
|---|---|---|
| **A** | D30 cutover announcement; every sunset extension; CONDUCTOR_DISPATCH_ENABLED toggle | activation transitions verbatim |
| **B** | binary materialisation (5A complete); soak clock start (5B); each weekly synthesis | hand-off boundaries |
| **D** | weekly four-surface audit (m7 SQLite vs stcortex vs m30 bank vs m15 JSONL) | four-surface drift |
| **I** | continuous — sustained `learning_health` non-movement; outbox empty after first sweep; m42 reinforce silence | Hebbian silence (currently firing live per tick·16) |
| E pre-position | any new arch proposal during soak inflating planning-to-code ratio | ancestor-rhyme escalation |
| G pre-position | substrate-frame confusion if dispatch outcomes flip sign | substrate corruption |

---

## Atuin trajectory anchors

```bash
# Per-phase audit at any soak day
atuin search "wf-crystallise"               --cwd /home/louranicas/claude-code-workspace/the-workflow-engine --before 7d
atuin search "wf-dispatch execute"          --cwd /home/louranicas/claude-code-workspace/the-workflow-engine --before 7d
atuin scripts run wt-soak-pulse                       # 30s probe per G5
atuin scripts run wt-bridge-check                     # all 5 substrate peers
atuin scripts run wt-substrate-pulse                  # LTP/LTD + RALPH gen + Watcher flag counts
atuin scripts run wt-cc5-trace                        # CC-5 first-closure
atuin kv get "workflow_trace.soak.d0.timestamp"       # baseline
atuin kv get "workflow_trace.povm_overlap_active"     # cutover state
```

---

## Sign-off

This runbook is **planning-only** (HOLD-v2). It becomes executable only when G1-G9 GREEN and Luke emits explicit `start coding workflow-trace` signal. Until then, treat every command above as a documented procedure, not an authorisation.

*Runbook 06 authored 2026-05-17 by Command (V7 optimisation, parallel author). Phase 5A + 5B + 5C operational. POVM cutover dance encoded. ~1,820 words. Source: phase-5-deploy-and-soak.md + phase-6-sunset-and-cross-cutting.md + phase-8-observability-operations.md. Sibling: runbook-05 / runbook-07 / runbook-08 / runbook-09 / runbook-11.*
