---
title: Runbook 04 — Phase 3 Integration + Conductor Wiring (Days 21-26)
date: 2026-05-17 (S1001982)
kind: planning-only operational runbook
phase: 3
days: 21-26 (5 calendar days, 5 integration tracks in parallel)
owner: Command (orchestrator) + Command-3 + Zen (audit) + Watcher (witness)
status: planning-only · activates after phase-2-complete tag (runbook-03 close)
---

# Runbook 04 — Phase 3 Integration + Conductor Wiring (Days 21-26)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · sibling: [[runbook-03-phase-2B-active]] · [[runbook-05-phase-4-hardening]]
>
> Cites: `the-workflow-engine-vault/deployment framework/phase-3-integration-conductor-wiring.md` (narrative source). Cross-ref: [[../GENERATIONS/G4-gold-standard.md]] §V8↔V3 wire reuse (Track 5), [[../MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md]] (CC-1..CC-7), [[../KEYWORDS_20.md]] (Conductor, AP30, four-surface, single-phase).

---

## Overview

Phase 3 wires the workflow engine into the living habitat across **five integration tracks in parallel**. No new module code is written. Modules are closed from Phase 2B. Luke starts services from terminal; Claude wires, tests, and observes. By Day 26, every Cluster H module is communicating with an external substrate; **CC-5 substrate learning loop has closed for the FIRST time** (the most important observation event of the deployment); and m32 is either live (if Conductor Waves 1B/1C/2/3 started by Luke) or in refuse-mode with the blocker explicitly named. The gate before this runbook is `phase-2-complete` tag. The gate after is `phase-3-complete` tag → control transfers to `runbook-05-phase-4-hardening.md`.

**Hard refusals (Phase 3 throughout):**
- No new module code.
- No service start from Claude (sandbox reaps children).
- No POVM writes outside `workflow_trace_*` (AP30 enforced at m9 wire, not convention).
- No silent fallback from stcortex to POVM after 2026-07-10 cutover.

---

## Pre-flight checklist

```bash
set -o pipefail
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
WF_ROOT="/home/louranicas/claude-code-workspace/workflow-trace"
cd "$WF_ROOT"

# (1) phase-2-complete tag on both remotes:
git ls-remote --tags origin | /usr/bin/grep "phase-2-complete" | tee /tmp/p3-pre-origin.txt
git ls-remote --tags gitlab | /usr/bin/grep "phase-2-complete" | tee /tmp/p3-pre-gitlab.txt
test -s /tmp/p3-pre-origin.txt && test -s /tmp/p3-pre-gitlab.txt || exit 1

# (2) verify-sync 1-20:
./scripts/verify-sync.sh --invariants 1-20 2>&1 | tee /tmp/p3-pre-vs.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 1

# (3) Conductor maturity probe (Track 2 dependency):
curl -s -m 1 http://localhost:8141/health 2>&1 | tee /tmp/p3-pre-conductor.txt
# Outcomes:
#   200 + enforcer_active=true → Track 2 wires LIVE
#   non-200 OR enforcer_active=false → Track 2 wires DRY-RUN; flagged for Luke action
#       Luke @ terminal: devenv start weaver/zen/enforcer → curl :8141/health → Wave 2 WASM
#       → 24h NoOp soak → flip CONDUCTOR_ENFORCEMENT_ENABLED=1

# (4) stcortex reachable (Track 1 dependency):
~/.local/bin/stcortex status | tee /tmp/p3-pre-stcortex.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 1

# (5) SYNTHEX v2 :8092 reachable (Track 3 dependency):
curl -s -m 1 http://localhost:8092/health | tee /tmp/p3-pre-synthex.txt

# (6) LCM JSON-RPC socket reachable (Track 4 dependency):
test -S /run/lcm/lcm-rpc.sock 2>/dev/null || test -S ~/.local/run/lcm-rpc.sock 2>/dev/null \
  || echo "WARN: LCM UDS unreachable; Track 4 wires DRY-RUN"

# (7) DevOps Engine V3 :8082 reachable (Track 5 dependency — V8↔V3 wire reuse):
curl -s -m 1 http://localhost:8082/health | tee /tmp/p3-pre-v3.txt

# (8) Watcher carriage alive — will witness all 5 tracks:
curl -s -m 1 http://localhost:8092/health | tee /tmp/p3-pre-watcher.txt
```

---

## Track 1 — stcortex consumer registration (Days 21-22)

**Modules:** m2 + m9 + m13. **Namespace:** `workflow_trace_*`. **Watcher class B (hand-off boundary).**

### Step 1.1 — WCP-NOTICE-BEGIN

```bash
~/.local/bin/watcher notify track1_begin "Phase 3 Track 1 — stcortex consumer registration starting. Refuse-write contract in force. T0: ${TS}"
```

### Step 1.2 — Registration ceremony

```bash
# Step 1 — probe stcortex reachability:
~/.local/bin/stcortex status 2>&1 | tee /tmp/p3-t1-probe.txt
test "${PIPESTATUS[0]}" -eq 0 || { echo "BLOCKED: stcortex DOWN"; exit 31; }

# Step 2 — idempotent register:
~/.local/bin/stcortex call register_consumer 'workflow-trace' 'workflow_trace_main' 'cli' \
  2>&1 | tee /tmp/p3-t1-register.txt

# Step 3 — verify:
~/.local/bin/stcortex sql \
  "SELECT name, namespace, transport, stale FROM consumer WHERE namespace LIKE 'workflow_trace_%'" \
  2>&1 | tee /tmp/p3-t1-verify.txt
/usr/bin/grep -E "workflow-trace.*workflow_trace_main.*false" /tmp/p3-t1-verify.txt \
  || { echo "FAIL: registration not visible"; exit 31; }

# Step 4 — m9 namespace guard wire check:
/usr/bin/grep -rE "workflow_trace_main|WORKFLOW_TRACE_NS_PREFIX" \
  src/m09_watcher_namespace_guard/ tee /tmp/p3-t1-m9.txt

# Step 5 — first workflow_trace_* memory write via m13 (test path):
cargo test --workspace --all-targets --all-features --release -- m13_live_smoke 2>&1 | tee /tmp/p3-t1-write.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 31

# Step 6 — verify memory landed:
~/.local/bin/stcortex sql \
  "SELECT id, namespace FROM memory WHERE namespace LIKE 'workflow_trace_%' ORDER BY id DESC LIMIT 5" \
  2>&1 | tee /tmp/p3-t1-memory.txt

# Step 7 — refuse-write verification (unregistered ns):
~/.local/bin/stcortex call register_consumer 'temp-test' 'workflow_trace_ORPHAN' 'cli'
~/.local/bin/stcortex call unregister_consumer 'temp-test'
~/.local/bin/stcortex call write_memory 'workflow_trace_ORPHAN' 'semantic' 'should-fail' \
  '1.0' 'null' 'test' 'null' '[]' 2>&1 | tee /tmp/p3-t1-refuse.txt
# Expected: 530 error — no fresh consumer
```

### Step 1.3 — WCP-NOTICE-COMPLETE

```bash
~/.local/bin/watcher notify track1_complete "Consumer registered. Refuse-write contract verified. First workflow_trace_* memory id=$(tail -1 /tmp/p3-t1-memory.txt | awk '{print $1}')"
```

**Verification:** consumer fresh=false; first write landed; refuse-write returns 530. **Failure modes:** stcortex DOWN — Track 1 BLOCKED; do NOT silently fall back to POVM; halt and document. AP30 prefix missing in m13 retrieval_ids → re-run m9 guard test. **Mitigates:** AP-Hab-03 (AP30), AP-Drift-06 (bridge contract).

---

## Track 2 — Conductor m32 dispatch wiring (Days 22-23)

**Modules:** m32. **Watcher class A (refuse→live mode transition).**

### Step 2.1 — Conductor probe + decision

```bash
~/.local/bin/watcher notify track2_begin "Phase 3 Track 2 — m32 Conductor wiring. T0: ${TS}"

curl -s -m 1 http://localhost:8141/health 2>&1 | tee /tmp/p3-t2-health.txt
# Decision branch:
#   200 + enforcer_active=true → m32 wires LIVE
#   else → m32 wires DRY-RUN; flagged
```

### Step 2.2A — LIVE path (Conductor Waves 1B/1C live)

```bash
# Set env for live mode:
export CONDUCTOR_DISPATCH_ENABLED=1

# Integration test: m32 → Conductor end-to-end (idempotent test workflow):
cargo test --workspace --all-targets --all-features --release -- \
  m32_conductor_live 2>&1 | tee /tmp/p3-t2-live.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 32

# 5-check pre-dispatch sequence audit:
#   probe_conductor → m33 TTL fresh → definition_hash match → sunset guard → cooldown
/usr/bin/grep -nE "probe_conductor|TTL.*fresh|definition_hash|sunset_guard|dispatch_cooldown" \
  src/m32_dispatcher/ | tee /tmp/p3-t2-5check.txt
test $(wc -l < /tmp/p3-t2-5check.txt) -ge 5 || exit 32

# Display-before-step banner verification:
target/release/wf-dispatch dispatch --workflow-id <test_id> --dry-run 2>&1 | head -20
# Banner must show: EscapeSurfaceProfile (e.g., "[ReadOnly]" or "[HostWrite]")

~/.local/bin/watcher notify track2_live "m32 LIVE mode active. 5-check sequence verified."
```

### Step 2.2B — DRY-RUN path (Conductor still auto_start=false)

```bash
# Refuse-mode test:
unset CONDUCTOR_DISPATCH_ENABLED
cargo test --workspace --all-targets --all-features --release -- \
  m32_refuse_mode 2>&1 | tee /tmp/p3-t2-refuse.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 32

# Verify refuse-mode error is FULL message to stdout (not silent log):
target/release/wf-dispatch dispatch --workflow-id <test_id> --dry-run 2>&1 | tee /tmp/p3-t2-refuse-stdout.txt
/usr/bin/grep -E "DispatchError::ConductorDispatchDisabled|CONDUCTOR_DISPATCH_ENABLED" /tmp/p3-t2-refuse-stdout.txt \
  || { echo "FAIL: refuse-mode message not on stdout"; exit 32; }

~/.local/bin/watcher notify track2_dry_run "m32 REFUSE-mode. Conductor Wave 1B/1C/2/3 still auto_start=false (OI-7). Luke action required."
```

**Verification:** LIVE path — 5-check fires in order; banner shows surface; DRY-RUN path — full error to stdout (not silent). **Failure modes:** silent no-op in refuse mode → Watcher Class-C escalation; http:// prefix re-introduced → BUG-033 risk. **Mitigates:** AP-WT-F4 (premature dispatch), AP-Hab-12 (silent swallow).

---

## Track 3 — SYNTHEX v2 NexusEvent (Days 22-23)

**Modules:** m40. **Watcher class B.**

```bash
~/.local/bin/watcher notify track3_begin "Phase 3 Track 3 — m40 → SYNTHEX :8092/v3/nexus/push"

# (1) SYNTHEX :8092 probe:
curl -s -m 1 http://localhost:8092/health 2>&1 | tee /tmp/p3-t3-probe.txt

# (2) bridge-contract skill (pre-merge audit per AP-Drift-06):
~/.local/bin/bridge-contract workflow-trace synthex-v2 2>&1 | tee /tmp/p3-t3-contract.txt
test "${PIPESTATUS[0]}" -eq 0 || { echo "BRIDGE CONTRACT DRIFT"; exit 33; }

# (3) m40 outbox-first integration test (fire NexusEvent → assert posted=true after HTTP 2xx):
cargo test --workspace --all-targets --all-features --release -- m40_synthex_emit_live 2>&1 | tee /tmp/p3-t3-live.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 33

# (4) Outbox sweep test — retry on 5xx with jitter:
# (synthetic 503 mock; test confirms backoff envelope ±25%)
cargo test --workspace --all-targets --all-features --release -- m40_outbox_retry 2>&1 | tail -10

# (5) Circuit breaker test — 5 consecutive failures → Open; 30s → HalfOpen → Closed:
cargo test --workspace --all-targets --all-features --release -- m40_circuit_breaker 2>&1 | tail -10

# (6) Compaction test — posted=true >24h removed:
cargo test --workspace --all-targets --all-features --release -- m40_compaction 2>&1 | tail -10

~/.local/bin/watcher notify track3_complete "m40 → SYNTHEX wired. Circuit breaker + outbox-retry + jitter all green."
```

**Verification:** bridge contract clean; NexusEvent landed at SYNTHEX; circuit breaker tested. **Mitigates:** AP-Drift-06 (bridge contract).

---

## Track 4 — LCM RPC (Days 23-24)

**Modules:** m41. **Watcher class B.**

```bash
~/.local/bin/watcher notify track4_begin "Phase 3 Track 4 — m41 → LCM JSON-RPC 2.0 over UDS"

# (1) LCM socket probe:
test -S /run/lcm/lcm-rpc.sock || test -S ~/.local/run/lcm-rpc.sock \
  || { echo "LCM UDS unreachable"; exit 34; }

# (2) bridge-contract:
~/.local/bin/bridge-contract workflow-trace lcm 2>&1 | tee /tmp/p3-t4-contract.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 34

# (3) m41 integration test (single lcm.loop.create with max_iters: 1):
cargo test --workspace --all-targets --all-features --release -- m41_lcm_live 2>&1 | tee /tmp/p3-t4-live.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 34

# (4) 30s timeout test:
cargo test --workspace --all-targets --all-features --release -- m41_timeout 2>&1 | tail -10

# (5) AP-Drift-11 supervisor stub check (LCM Drift #11):
~/.local/bin/lcm-supervisor --help 2>&1 | head -5
# Verify binary exists; live RPC, not stub.

~/.local/bin/watcher notify track4_complete "m41 → LCM RPC live. max_iters:1 enforced."
```

**Verification:** UDS reachable; bridge contract clean; lcm.loop.create succeeds. **Mitigates:** AP-Drift-11.

---

## Track 5 — DevOps V3 + POVM/stcortex dual-path (Days 23-25)

**Modules:** m42 (POVM dual-path) + new outbound POST to :8082/api/v8/learning (V8↔V3 wire reuse per G4). **Watcher class A (CC-5 first close) + I (Hebbian).**

### Step 5.1 — POVM/stcortex dual-path (CC-5 first close)

```bash
~/.local/bin/watcher notify track5_begin "Phase 3 Track 5 — CC-5 substrate learning loop FIRST CLOSURE"

# (1) Set dual-path flag (override → 2026-07-10):
export POVM_OVERLAP_ACTIVE=true

# (2) Drive a synthetic workflow through m32 dispatch → m42 reinforce:
cargo test --workspace --all-targets --all-features --release -- m42_dual_path_live 2>&1 | tee /tmp/p3-t5-dual.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 35

# (3) Confirm POVM pathway written (during overlap):
curl -s "http://localhost:8125/pathways?prefix=workflow_trace_" 2>&1 | jq '.count' | tee /tmp/p3-t5-povm-count.txt

# (4) Confirm stcortex pathway written (post-cutover behaviour):
~/.local/bin/stcortex sql \
  "SELECT COUNT(*) FROM pathway WHERE namespace LIKE 'workflow_trace_%'" 2>&1 | tee /tmp/p3-t5-sc-count.txt

# (5) Negative test — POVM_OVERLAP_ACTIVE=false + stcortex DOWN:
#   Expected: ERROR log + SubstrateUnavailable returned; NO silent POVM fallback
unset POVM_OVERLAP_ACTIVE
cargo test --workspace --all-targets --all-features --release -- m42_no_silent_fallback 2>&1 | tail -10
export POVM_OVERLAP_ACTIVE=true

# (6) AP30 audit on m42 retrieval_ids:
~/.local/bin/stcortex sql \
  "SELECT pre_id, post_id FROM pathway WHERE namespace = 'workflow_trace_main' LIMIT 20" \
  | /usr/bin/grep -vE "workflow_trace_" | tee /tmp/p3-t5-ap30.txt
test ! -s /tmp/p3-t5-ap30.txt || { echo "AP-Hab-03 / AP30 VIOLATION"; exit 35; }
```

### Step 5.2 — V8↔V3 wire reuse (G4 GAP-Gold-04 closure)

Per G4 § "V8 ↔ V3 wire reuse" — m32 DispatchOutcome::PassVerified fans out an additional POST to :8082/api/v8/learning. Phase 3 wires this **once** in Track 5 (m40 already does NexusEvent; this is a new emit).

```bash
# (1) V3 :8082 probe:
curl -s -m 1 http://localhost:8082/health | tee /tmp/p3-t5-v3.txt

# (2) bridge-contract:
~/.local/bin/bridge-contract workflow-trace devops-engine-v3 2>&1 | tee /tmp/p3-t5-v3-contract.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 35

# (3) Integration test: m32 PassVerified → POST /api/v8/learning {confidence_delta: +0.10}:
cargo test --workspace --all-targets --all-features --release -- m32_v8_learning_emit 2>&1 | tee /tmp/p3-t5-v8.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 35

# (4) V3 resume_from contract — m41 routes deploy with resume_from: "T2":
cargo test --workspace --all-targets --all-features --release -- m41_resume_from 2>&1 | tee /tmp/p3-t5-resume.txt

~/.local/bin/watcher notify track5_cc5_close \
  "CC-5 substrate learning loop FIRST CLOSURE measured. POVM count: $(cat /tmp/p3-t5-povm-count.txt). stcortex count: $(cat /tmp/p3-t5-sc-count.txt). V8 learning wire live."
```

**Verification:** dual-path writes confirmed in both substrates; AP30 grep clean; V8 learning POST returns 2xx; resume_from contract honoured. **Watcher class:** A (CC-5 first close — most-important Phase 3 event), I (Hebbian signal expected post-CC-5 close). **Mitigates:** AP-Hab-03, AP-Drift-06.

---

## Day 25-26 — Convergence integration smoke + Phase-3 close

### Step 25.1 — 5-track convergence smoke

```bash
# Author tests/integration/phase3_convergence.rs (NOT new module — integration test):
#   - dispatch one synthetic workflow (mock Conductor accepting OR live if Track 2 live)
#   - assert m32 DispatchOutcome captured
#   - assert m40 NexusEvent posted to SYNTHEX (HTTP 2xx)
#   - assert m41 lcm.loop.create succeeded (if deploy-shape) OR skipped (if not deploy)
#   - assert m42 reinforce written to POVM AND stcortex (dual-path active)
#   - assert POST /api/v8/learning landed (V3 receives confidence_delta)
#   - assert m13 stcortex memory written for the closed run
#   - assert m14 lift snapshot updated (LiftSnapshot.n incremented)
#   - assert Watcher Class-I flag cleared (LTP signal observable)

cargo test --workspace --all-targets --all-features --release -- phase3_convergence 2>&1 | tee /tmp/p3-d25-conv.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 36

# Serde rename = "type" trap audit (Phase 3 wire is the typical surface for this trap):
/usr/bin/grep -nE 'rename = "type"' src/ | tee /tmp/p3-d25-serde.txt
# If any rename = "type" appears: ensure deserialize_with handles snake_case/PascalCase variants
```

### Step 25.2 — Full --workspace 4-stage gate

```bash
cargo check --workspace --all-targets --all-features 2>&1 | tee /tmp/p3-d25-check.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 41

cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/p3-d25-clippy.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 42

cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tee /tmp/p3-d25-pedantic.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 43

cargo test --workspace --all-targets --all-features --release 2>&1 | tee /tmp/p3-d25-test.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 44
```

### Step 25.3 — Substrate snapshot + Watcher Class-I assessment

```bash
# Substrate snapshot for baseline.json (Phase 4 input):
WORK="/tmp/phase3-close-${TS}"
mkdir -p "${WORK}"

# stcortex pathway count under workflow_trace_*:
~/.local/bin/stcortex sql \
  "SELECT COUNT(*) FROM pathway WHERE namespace LIKE 'workflow_trace_%'" \
  > "${WORK}/stcortex_pw_count.txt"

# POVM pathway count under workflow_trace_*:
curl -s "http://localhost:8125/pathways?prefix=workflow_trace_" | jq '.count' \
  > "${WORK}/povm_pw_count.txt"

# substrate_LTP_density:
~/.local/bin/stcortex sql \
  "SELECT (SELECT COUNT(*) FROM pathway WHERE weight > 0.5) * 1.0 / COUNT(*) AS density FROM pathway" \
  > "${WORK}/ltp_density.txt"

# m14 lift baseline:
target/release/wf-crystallise lift --raw > "${WORK}/lift_raw.txt" 2>&1
target/release/wf-crystallise lift --cost > "${WORK}/lift_cost.txt" 2>&1

# Watcher Class-I status — is LTP signal observable post-CC-5 first close?
~/.local/bin/watcher list --class I --since 5d > "${WORK}/class_i_status.txt"

# Phase-3 close WCP notify:
~/.local/bin/watcher notify phase3_close \
  "5 tracks integrated. CC-5 first close MEASURED. stcortex_pw_count=$(cat ${WORK}/stcortex_pw_count.txt). POVM_pw_count=$(cat ${WORK}/povm_pw_count.txt). LTP_density=$(cat ${WORK}/ltp_density.txt)."
```

### Step 25.4 — Wave-end orchestrator checklist + phase-3 tag

```bash
./scripts/wave-end-checklist.sh 3.5 2>&1 | tee /tmp/p3-d25-wave-end.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 45

git tag -a "phase-3-complete" -m "Phase 3 close: 5 integration tracks live; CC-5 first close measured"
git push origin main --tags
git push gitlab main --tags 2>&1 | tee /tmp/p3-d25-push.txt

# Document Track 2 state (LIVE or DRY-RUN) in tag annotation:
git tag --list "phase-3-complete" -n10
```

---

## Phase-end gate (must be green before runbook-05)

| Check | Command | Pass criterion |
|---|---|---|
| All 5 tracks complete or documented DRY-RUN | grep WCP notices | 5 watcher notifications filed |
| Full --workspace 4-stage QG | re-run Day 25 chain | exit 0 |
| Convergence smoke green | `phase3_convergence` test | PASS |
| bridge-contract clean × 3 (SYNTHEX, LCM, V3) | all 3 outputs | clean |
| CC-5 first close measured | `${WORK}/stcortex_pw_count.txt` + `povm_pw_count.txt` | both >0 |
| AP30 prefix clean on m42 + m13 retrieval_ids | grep audit | clean |
| Substrate snapshot files written | `ls ${WORK}/*.txt` | 5+ files |
| Watcher Class-I assessment recorded | `${WORK}/class_i_status.txt` | non-empty |
| phase-3-complete tag both remotes | `git ls-remote --tags origin/gitlab` | match |

---

## Failure modes register

| # | Failure | Detection | Mitigation |
|---|---|---|---|
| 1 | **Conductor `auto_start=false`** (Wave 1B/1C still down) | curl :8141/health non-200 | Track 2 DRY-RUN; Luke @ terminal `devenv start weaver/zen/enforcer` |
| 2 | **stcortex 530 refuse-write on m13** | refuse-write test fails or m13 live test errors | m9 namespace guard verify; consumer registration idempotent re-fire |
| 3 | **Silent POVM fallback on stcortex down** | grep `povm.*write` in fallback path | hard refusal; per CLAUDE.md row-8: log ERROR + return SubstrateUnavailable |
| 4 | **Bridge contract drift** (m40↔SYNTHEX wire shape diverged) | `bridge-contract` skill returns non-zero | re-align schema; do NOT merge across drift; AP-Drift-06 |
| 5 | **AP-Drift-11 LCM supervisor stub mistaken for live** | `lcm-supervisor --help` errors or stub message | verify binary present + RPC reachable; documented per Drift register |
| 6 | **Serde `rename = "type"` trap** (deserialize fails on case variant) | convergence smoke test errors | use `deserialize_with` to accept snake_case + PascalCase; per S116 Stream B crystallisation |
| 7 | **PIPESTATUS swallow** in track scripts | green PASS; stderr screaming | always `${PIPESTATUS[0]}` (AP-Hab-05) |
| 8 | **CC-5 first close shows zero LTP signal** | substrate_LTP_density unchanged from Phase 2 baseline | NOT necessarily failure; expected if small-N substrate; document for Phase 5C soak monitoring |
| 9 | **POVM cutover triggered mid-Phase-3** (2026-07-10) | calendar check | set POVM_OVERLAP_ACTIVE=false; stcortex-only writes; coordinate with m42 |
| 10 | **Handshake dual-silence** if Command-3 unavailable for Track 2 | no Command-3 drop within 60min | escalate via file-drop; do NOT proceed without verification (AP-V7-08) |

---

## Watcher flag pre-positioning (Phase 3)

| Class | Activates | Track |
|---|---|---|
| **A** activation | Conductor live transition; CC-5 first close | 2, 5 |
| **B** hand-off boundary | every cross-substrate write (m13, m40, m41, m42, V3 emit) | 1, 3, 4, 5 |
| **C** confidence-gate refusal | m32 refuse-mode message; m33 4-agent REJECT | 2 |
| **D** four-surface drift | Phase 3 WCP notices vs Wave-3 plan.toml | 25 |
| **G** substrate-frame confusion | Watcher must NOT interpret CC-5 LTP signal as user-intent feedback | 5 |
| **I** Hebbian silence | continuous; CC-5 first close → LTP signal expected; clearance moment | 5, 25 |

---

## Atuin trajectory anchors (Phase 3)

```bash
atuin search "stcortex.*register_consumer" --before 6d
atuin search "curl.*8141/health" --before 6d
atuin search "bridge-contract" --before 6d
atuin search "cargo test.*m13_live_smoke\|m40_synthex_emit_live\|m41_lcm_live\|m42_dual_path_live\|m32_v8_learning_emit" --before 6d
atuin search "cargo test.*phase3_convergence" --before 6d
atuin search "wf-crystallise lift" --before 6d
atuin search "git tag.*phase-3-complete" --before 6d
atuin scripts run habitat-cross-tensor --before 6d
atuin scripts run habitat-evolution-delta --before 6d
```

Each must return ≥1 hit. CC-5 first close is THE atuin trajectory anchor.

---

*runbook-04 authored 2026-05-17 by Command (V7 author wave subagent)*
