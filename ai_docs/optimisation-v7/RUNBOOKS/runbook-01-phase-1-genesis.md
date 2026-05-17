---
title: Runbook 01 — Phase 1 Genesis (Day 0-3)
date: 2026-05-17 (S1001982)
kind: planning-only operational runbook
phase: 1
days: 0-3 (72h after G9)
owner: Command (orchestrator) + Command-2 (build-executor) + rust-pro subagents
binary_targets: wf-crystallise + wf-dispatch + workflow-core lib
status: planning-only · activates ONLY at G9-green per AP24
---

# Runbook 01 — Phase 1 Genesis (Day 0-3)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · sibling: [[runbook-00-pre-genesis-gates]] · [[runbook-02-phase-2A-measure-only]]
>
> Cites: `the-workflow-engine-vault/deployment framework/phase-1-genesis-day-0-3.md` (narrative source); refines into 72h command sequence with verify-sync invariants + Watcher class pre-positioning. Cross-ref: [[../GENERATIONS/G2-consolidation.md]] (src/ layout), [[../GENERATIONS/G4-gold-standard.md]] (ORAC single-crate pattern), [[../STANDARDS/GOD_TIER_RUST.md]] (4-stage QG), [[../KEYWORDS_20.md]] (AP24, verify-sync, two-binary, gold-standard, Cluster, PIPESTATUS).

---

## Overview

Phase 1 covers hours 0-72 after G9 fires. Foundation layer ships before observers; trust layer (Cluster D, m8-m11) ships before substrate readers (Cluster A, m1-m3); compile-time invariants ship before runtime paths. By Day 3 close, the substrate is observable read-only, the first stcortex consumer is registered under `workflow_trace_*`, and the Watcher Class-E ancestor-rhyme flag is resolved by first `git commit` with passing `cargo check`. The gate before this runbook is G9 fired (runbook-00). The gate after is Wave-1 merge tag `wave-1-complete` → control transfers to `runbook-02-phase-2A-measure-only.md`.

**G2 critical fix per V7 KEYWORDS_20:** the existing phase-1 doc uses **m11-Day-1 with formula AND state machine**. V7 G2 invariant requires m11 Day 1 ship to be **pure formula** (`compute_decay_factor`) only; SunsetPhase state machine deferred to Cluster F integration. This runbook enforces that split — see Day 1 § m11.

---

## Pre-flight checklist

```bash
set -o pipefail
TS=$(date -u +%Y-%m-%dT%H%M%SZ)

# (1) G9 confirmed (runbook-00 phase-end gate green):
test -f ~/projects/shared-context/agent-cross-talk/*_d_g9.md \
  || { echo "BLOCKED: D-G9 missing; return to runbook-00"; exit 1; }

# (2) Habitat baseline + neighbour-service health probe (BEFORE any new build):
atuin scripts run habitat-bootstrap 2>&1 | tail -5
atuin scripts run habitat-intel 2>&1 | tail -5
atuin scripts run habitat-fingerprint 2>&1 | tail -5

declare -A hpath=([8082]="/health" [8083]="/health" [8092]="/health" [8111]="/health" \
  [8120]="/health" [8125]="/health" [8130]="/health" [8132]="/health" [8133]="/health" \
  [8140]="/health" [8180]="/api/health" [10002]="/health")
unhealthy=0
for port in "${!hpath[@]}"; do
  code=$(curl -s -o /dev/null -w '%{http_code}' -m 1 "http://localhost:${port}${hpath[$port]}" 2>/dev/null || echo "000")
  test "$code" = "200" || { echo "WARN port=${port} code=${code}"; unhealthy=$((unhealthy+1)); }
done
test "$unhealthy" -eq 0 || { echo "REFUSAL: ${unhealthy} services down — do not deploy onto degraded habitat"; exit 1; }

# (3) POVM_CR2_DEPLOYED env present (m8 prerequisite):
test "${POVM_CR2_DEPLOYED}" = "1" \
  || { echo "REFUSAL: export POVM_CR2_DEPLOYED=1 after Luke confirms povm-v2 e2a8ed3 live"; exit 1; }

# (4) stcortex reachable (m2 prerequisite Day 2):
~/.local/bin/stcortex status | tee /tmp/p1-stcortex-pre.txt
test "${PIPESTATUS[0]}" -eq 0 \
  || echo "WARN: stcortex DOWN; m2 registration deferred; m2 tests via mock path"

# (5) atuin history readable (m1 prerequisite Day 2):
sqlite3 -readonly ~/.local/share/atuin/history.db "SELECT COUNT(*) FROM history" 2>&1 | tee /tmp/p1-atuin-pre.txt
test "${PIPESTATUS[0]}" -eq 0 || { echo "FAIL: atuin DB unreadable"; exit 1; }

# (6) injection.db readable (m3 prerequisite Day 2):
sqlite3 -readonly ~/.local/share/habitat/injection.db \
  "SELECT COUNT(*) FROM causal_chain WHERE resolved_session IS NULL" 2>&1 | tee /tmp/p1-injection-pre.txt
test "${PIPESTATUS[0]}" -eq 0 || { echo "FAIL: injection.db unreadable"; exit 1; }
```

If any step fails, fix before advancing. Phase 1 must not start on a degraded habitat (per `feedback_god_tier_no_warnings_at_any_level.md`).

---

## Day 0 (Hours 0-12) — Workspace init + Cat 10 foundation

**Goal:** bare workspace compiles; `cargo check` exits 0; `plan.toml` + `bacon.toml` authored; first commit recorded (resolves Class-E flag); 4 Cat-10 foundation modules shipped (m01, m02, m03, m05).

### Step 1.1 — Workspace init (Hours 0-3)

```bash
WF_ROOT="/home/louranicas/claude-code-workspace/workflow-trace"
cd "$WF_ROOT"

# cargo init (single-crate per G4 gold-standard divergent-axis Axis 1 — ORAC pattern):
cargo init --name workflow-trace 2>&1 | tee /tmp/p1-cargo-init.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 10

# Create src/ DAG per G2 consolidation (verify-sync invariant #1):
mkdir -p src/{m01_foundation,m02_error_taxonomy,m03_config,m05_metrics_collector}
mkdir -p src/{m08_povm_build_prereq,m09_watcher_namespace_guard}
mkdir -p src/{m10_ember_gate,m11_engine_sunset_lifecycle}
mkdir -p src/{m1_atuin_consumer,m2_stcortex_consumer,m3_injection_consumer}
mkdir -p src/bin migrations tests/{integration,fixtures} ai_docs ai_specs templates scripts

# Authored via Write tool (NOT cat <<EOF — Write tool is mandatory):
#   Cargo.toml (per GOD_TIER_RUST.md boilerplate header rules)
#   src/lib.rs (#![forbid(unsafe_code)] #![deny(clippy::unwrap_used)] ...)
#   src/bin/wf_crystallise.rs (≤50 LOC main)
#   src/bin/wf_dispatch.rs (≤50 LOC main)
#   plan.toml (LCM-pattern, all 26 modules declared with phase markers)
#   bacon.toml (4-stage on_success chain)
#   build.rs (empty stub — m8 fills Day 1)
#   .cargo/config.toml (build profile)

# Smoke compile:
CARGO_TARGET_DIR=./target cargo check 2>&1 | tee /tmp/p1-d0-check.txt
test "${PIPESTATUS[0]}" -eq 0 || { echo "FAIL: cargo check"; exit 10; }
```

**Inputs:** D-G9 confirmed; G2 src/ layout (ULTRAMAP View 2). **Outputs:** Cargo.toml + lib.rs + 2 bin stubs + plan.toml + bacon.toml + empty build.rs; first `cargo check` PASS. **Verification:** `cargo check` exit 0; directory tree matches ULTRAMAP View 2 verify-sync invariant #1. **Failure modes:** clippy::pedantic warns on missing doc comments → add `#![allow(missing_docs)]` temporary until first real module ships then remove (AP-Hab-14 god-tier dilution risk); fix before commit. **Watcher class:** A (genesis activation), F (AP24 last-check). **Mitigates:** AP-Hab-01, AP-V7-05 (module-plan-to-src drift).

### Step 1.2 — Class-E resolution commit (Hour 3)

```bash
git add Cargo.toml build.rs src/ plan.toml bacon.toml migrations/ ai_docs/ ai_specs/ templates/
git commit -m "$(cat <<'EOF'
genesis: workflow-trace workspace init — lint policy + plan.toml skeleton

Cargo.toml: forbid(unsafe_code), deny(unwrap_used/expect_used/panic/todo/dbg_macro);
feature gates (api/intelligence/monitoring/evolution/full). Two binaries
(wf-crystallise + wf-dispatch) + shared workflow_trace lib (ORAC pattern, G4).

plan.toml: all 26 modules declared with phase markers and authorization block.
bacon.toml: 4-stage on_success chain (check → clippy → pedantic → test --release).

Resolves Watcher Class-E (ancestor-rhyme). First cargo check PASS.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
SHA=$(git log --oneline -1 | cut -d' ' -f1)
~/.local/bin/watcher notify class_e_resolved "first commit ${SHA}; cargo check PASS"
```

**Verification:** `git log --oneline -1` non-empty; Watcher carriage acked. **Watcher class:** A resolved.

### Step 1.3 — Cat-10 foundation direct clones (Hours 3-12)

Per ULTRAMAP View 5 Wave 1: Command-2 owns L0+L4 worktrees. For Day 0 Cat-10 lift, Command-2 dispatches 4 rust-pro subagents in parallel (Battern Step 2 fan-out), one per module, all in main worktree (per AGENT_VIEW worktree post-G9 only — Day 0 lift uses subagents-in-main).

Per-module recipe (apply to m01_core_types, m02_error_taxonomy, m03_config, m05_metrics_collector):

```bash
MODULE="m01_foundation"     # or m02_error_taxonomy / m03_config / m05_metrics_collector
SRC="the-workflow-engine-vault/boilerplate modules/10-foundation-direct-clones/${MODULE}.rs"

# (a) Read boilerplate source (Read tool):
# (b) Adapt via Write tool to src/${MODULE}/ (~95% lift; rename habitat→workflow-trace types per cluster-A spec).
# (c) Per-module 4-stage gate (PIPESTATUS discipline):
cargo check 2>&1 | tee /tmp/p1-${MODULE}-check.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 11

cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/p1-${MODULE}-clippy.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 12

cargo clippy --all-targets -- -D warnings -W clippy::pedantic 2>&1 | tee /tmp/p1-${MODULE}-pedantic.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 13

CARGO_TARGET_DIR=./target cargo test --lib --release -- "${MODULE}" 2>&1 | tee /tmp/p1-${MODULE}-test.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 14

# Test-count assertion (≥50 per module per TEST_DISCIPLINE.md):
count=$(/usr/bin/grep -cE "^test .* ok$" /tmp/p1-${MODULE}-test.txt)
test "$count" -ge 50 || { echo "FAIL: ${MODULE} only ${count}<50 tests"; exit 14; }
```

**Day 0 commit (Hour 12):**

```bash
git add src/m01_foundation/ src/m02_error_taxonomy/ src/m03_config/ src/m05_metrics_collector/
git commit -m "feat(cat10): foundation direct clones — m01/m02/m03/m05 (4-stage PASS; ≥200 tests)"
```

**Day 0 verify-sync invariants check (1-3):**

```bash
./scripts/verify-sync.sh --invariants 1-3 2>&1 | tee /tmp/p1-d0-vs.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 15
```

**Watcher class:** F (AP-Hab-14 risk: pedantic dilution); F (AP-V7-05: module-plan-to-src LOC drift).

---

## Day 1 (Hours 12-36) — Cluster D Trust Layer (m8 + m9 + m10 + m11)

**Goal:** all 4 trust-layer modules shipped; `build.rs` CR-2 gate active; namespace guard callable; Ember CI test passing; **m11 pure-formula authoring only** (state machine deferred per G2 fix). Cluster D closes with ≥123 tests.

### Step 1.4 — m8 `povm_build_prereq` (Hours 12-16)

```bash
# Authoring (Write tool):
#   build.rs: emit cargo:rustc-cfg=povm_calibrated when POVM_CR2_DEPLOYED=1
#   src/m08_povm_build_prereq/mod.rs: cfg paired-annotation helpers
#   src/m08_povm_build_prereq/tests.rs: ≥50 tests (compile_error! tombstone behaviour, cfg-gate, env-var read)

# Verify gate fires:
POVM_CR2_DEPLOYED=1 cargo check 2>&1 | tail -5    # expect: Finished
unset POVM_CR2_DEPLOYED
cargo check 2>&1 | /usr/bin/grep -E "compile_error|povm_calibrated" || \
  echo "EXPECTED: warning(s) from cfg-emit; compile_error fires only on POVM-read sites"
export POVM_CR2_DEPLOYED=1

# Per-module gate (PIPESTATUS):
cargo check && cargo clippy -- -D warnings && \
cargo clippy -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m08 2>&1 | tee /tmp/p1-m8-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 16
```

**Inputs:** D-G9; CR-2 env var. **Outputs:** `build.rs` + m8 module + 50 tests. **Verification:** `POVM_CR2_DEPLOYED=1 cargo check` clean; unset → compile_error tombstone fires when a read-site stub is added. **Failure modes:** env-var leak in CI (set false-true mid-build) — Watcher Class-F. **Watcher class:** I (substrate-frame Hebbian-silence canary), F. **Mitigates:** AP-WT-F7 (CR-2 graceful-degrade pretend-fix).

### Step 1.5 — m9 `watcher_namespace_guard` (Hours 16-20)

```bash
# Author src/m09_watcher_namespace_guard/{mod.rs,tests.rs}:
#   pub const WORKFLOW_TRACE_NS_PREFIX: &str = "workflow_trace";
#   pub fn assert_workflow_trace_namespace(ns: &str) -> Result<(), WorkflowError>
#   tracing::error!(namespace = %ns, "namespace violation") on rejection

# Per-module gate:
cargo check && cargo clippy -- -D warnings && \
cargo clippy -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m09 2>&1 | tee /tmp/p1-m9-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 17
```

**Inputs:** m02 error taxonomy (NamespaceViolation variant). **Outputs:** guard module + ≥15 tests. **Verification:** `assert_workflow_trace_namespace("workflow_trace_test")` returns Ok; `("orac_learn")` returns Err. **Watcher class:** F. **Mitigates:** AP-Hab-03 (AP30), AP-WT-F8 (feedback-loop poisoning).

### Step 1.6 — m10 `ember_gate_test` (Hours 20-26)

```bash
# Author:
#   src/user_facing_strings.rs (ALL: &[(&str, &str)] registry — populated with m03/m05 strings)
#   tests/ember_gate.rs (CI test scoring each ALL entry against 7-trait rubric)
# W3 enforcement: Held → CI-FAIL until G4 §5.1 amendment (already green at runbook-00 close).

cargo test --lib --release -- ember_gate 2>&1 | tee /tmp/p1-m10-ember.txt
test "${PIPESTATUS[0]}" -eq 0 || { echo "FAIL: fix string before continuing"; exit 18; }

# Per-module 4-stage gate:
cargo check && cargo clippy -- -D warnings && \
cargo clippy -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m10 2>&1 | tee /tmp/p1-m10-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 18
```

**Inputs:** G4 §5.1 amendment (CI-FAIL on Held); m03+m05 strings registered. **Outputs:** ember_gate.rs CI test + ≥50 tests. **Failure modes:** Equanimity (all-caps status) / Honesty (round numbers without scope) — common first-run failures; fix strings per cluster-D spec. **Watcher class:** F. **Mitigates:** AP-WT-F7, AP-Hab-14.

### Step 1.7 — m11 `engine_sunset_lifecycle` PURE FORMULA ONLY (Hours 26-32)

**G2 FIX (V7 per KEYWORDS_20):** Day 1 ships **only `compute_decay_factor` pure function** + unit tests. SunsetPhase state machine deferred to Cluster F integration (Phase 2B). This decouples the math (testable in isolation) from the state machine (which requires m14 lift signal + m30 bank rows that don't exist yet).

```bash
# Author src/m11_engine_sunset_lifecycle/mod.rs (~80 LOC NOT 250):
#   pub fn compute_decay_factor(frequency: f64, fitness: f64, recency: f64, plain_decay_rate: f64) -> f64
#   Implementation: base_rate = 1.0 - plain_decay_rate
#                   compound = (frequency * fitness * recency).clamp(0.0, 1.0)
#                   base_rate.mul_add(1.0 - compound, compound)
#   Public surface: just compute_decay_factor + plain_decay_rate constant.

# Tests (≥50):
#   - zero signals → base_rate (12 tests: each axis zero, all zero, partial)
#   - all-ones → 1.0 (within f64 epsilon)
#   - clamp upper (frequency=2.0 → treated as 1.0)
#   - clamp lower (frequency=-0.5 → treated as 0.0)
#   - NaN guard via debug_assert!
#   - proptest: monotonic in each axis (10000 iters)
#   - calibration: plain_decay_rate=0.02 → 228 cycles to prune threshold 0.01
#   - 120-day sunset_at hard boundary fires before prune (calibration only, no state machine here)
#   - benches/criterion: <100ns per call (G15 hot-path constraint)

# Per-module gate:
cargo check && cargo clippy -- -D warnings && \
cargo clippy -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m11 2>&1 | tee /tmp/p1-m11-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 19

# G2 invariant verification: m11 mod.rs LOC ≤ 120 (state machine NOT present):
loc=$(wc -l < src/m11_engine_sunset_lifecycle/mod.rs)
test "$loc" -le 120 || { echo "FAIL G2 INVARIANT: m11 mod.rs ${loc}>120 — state machine landed early"; exit 19; }

# Verify no SunsetPhase enum present in Day 1 surface:
/usr/bin/grep -c "SunsetPhase" src/m11_engine_sunset_lifecycle/mod.rs 2>/dev/null && \
  { echo "FAIL G2 INVARIANT: SunsetPhase introduced too early"; exit 19; }
```

**Inputs:** m02 error taxonomy. **Outputs:** pure `compute_decay_factor` + ≥50 unit tests. **Verification:** LOC ≤120; no SunsetPhase symbol; gate green. **Failure modes:** NaN propagation → debug_assert + clamp; SunsetPhase landed early → revert per G2 fix; state machine code is **Phase 2B Wave 3 m11 integration** owned with m30 bank schema. **Watcher class:** I (Hebbian-silence proximity), G (substrate-frame: don't pre-compute "sunset" as anthropocentric verb). **Mitigates:** AP-V7-09 (substrate-frame confusion), AP-V7-05 (LOC drift).

### Step 1.8 — Day 1 commit

```bash
git add build.rs src/m08_povm_build_prereq/ src/m09_watcher_namespace_guard/ \
  src/m10_ember_gate/ src/m11_engine_sunset_lifecycle/ src/user_facing_strings.rs tests/ember_gate.rs

git commit -m "$(cat <<'EOF'
feat(cluster-d): trust layer (m8/m9/m10/m11 pure formula) — Day 1 gate-clean

m8 build.rs: POVM_CR2_DEPLOYED=1 emits cargo:rustc-cfg=povm_calibrated.
m9 watcher_namespace_guard: assert_workflow_trace_namespace + WORKFLOW_TRACE_NS_PREFIX.
m10 ember_gate_test: user_facing_strings registry + CI test (W3-as-FAIL active per G4).
m11 engine_sunset_lifecycle: compute_decay_factor PURE FUNCTION ONLY (NEW PRIMITIVE).
  G2 INVARIANT: SunsetPhase state machine deferred to Cluster F integration (Phase 2B).

Cluster D 4-stage gate: PASS. Tests: ≥123. CC-2 (Trust Layer Woven): ready for Day 2.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"

./scripts/verify-sync.sh --invariants 1-5 2>&1 | tee /tmp/p1-d1-vs.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 19
```

---

## Day 2 (Hours 36-60) — Cluster A Substrate Ingest (m1 + m2 + m3)

**Goal:** 3 substrate readers shipped; first stcortex consumer registered (real side-effect); ≥150 tests across cluster.

### Step 2.1 — Pre-conditions re-probe (Hour 36)

```bash
~/.local/bin/stcortex status | tee /tmp/p1-d2-stcortex.txt
test "${PIPESTATUS[0]}" -eq 0 || echo "WARN: stcortex DOWN; m2 registration deferred to mock-mode"

test "${POVM_CR2_DEPLOYED}" = "1" || { echo "FAIL: env var lost"; exit 20; }

sqlite3 -readonly ~/.local/share/povm-v2.db \
  "SELECT COUNT(*) FROM sessions WHERE fitness_delta IS NOT NULL" 2>&1 | tail -3
```

### Step 2.2 — m1 `atuin_consumer` (Hours 36-44)

```bash
# Author per cluster-A spec §m1: cursor-based pagination (NOVEL ~30 LOC),
# read-only WAL, PRAGMA query_only=ON, subprocess fallback, ToolCallRow typed.

cargo check && cargo clippy -- -D warnings && \
cargo clippy -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m1_atuin 2>&1 | tee /tmp/p1-m1-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 21

# Test-count + PRAGMA assertion:
count=$(/usr/bin/grep -cE "^test .* ok$" /tmp/p1-m1-gate.txt)
test "$count" -ge 50 || exit 21
```

### Step 2.3 — m2 `stcortex_consumer` (Hours 44-52)

```bash
# Author per cluster-A spec §m2: narrowed subscription (tool_call + consumption only).
# Real side-effect Day 2: register_consumer reducer call.

cargo check && cargo clippy -- -D warnings && \
cargo clippy -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m2_stcortex 2>&1 | tee /tmp/p1-m2-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 22

# Verify consumer registered (only if stcortex was UP):
~/.local/bin/stcortex consumers 2>/dev/null | /usr/bin/grep workflow-trace \
  | tee /tmp/p1-m2-consumer.txt
```

### Step 2.4 — m3 `injection_consumer` (Hours 52-60)

```bash
# Author per cluster-A spec §m3: CausalChainRow with ChainType/ConsentLevel typed enums,
# read_unresolved, read_recently_resolved.

cargo check && cargo clippy -- -D warnings && \
cargo clippy -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m3_injection 2>&1 | tee /tmp/p1-m3-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 23
```

### Step 2.5 — Day 2 commit

```bash
git add src/m1_atuin_consumer/ src/m2_stcortex_consumer/ src/m3_injection_consumer/
git commit -m "feat(cluster-a): substrate ingest — atuin/stcortex/injection (~150 tests; CC-1 data path ready)"
./scripts/verify-sync.sh --invariants 1-7 2>&1 | tee /tmp/p1-d2-vs.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 23
```

**Watcher class (Day 2):** B (first stcortex hand-off boundary crossing on register_consumer), H (atuin proprioception if WAL busy-timeout), F. **Mitigates:** AP-Hab-03 (AP30 — m9 guard at m2 boundary), AP-WT-F8.

---

## Day 3 (Hours 60-72) — Integration smoke + WCP notify

**Goal:** m1+m2+m3 read raw substrates and produce observable output; full 4-stage gate across all Day 0-2 modules; first WCP notify; hand-off state documented.

### Step 3.1 — Integration smoke tests (Hours 60-66)

```bash
# Author tests/integration_smoke.rs:
#   - m1::open_atuin_db → first page rows > 0
#   - m2::RegistrationHandle::is_fresh() true (within 30-day window)
#   - m3::read_unresolved returns ≥1 CausalChainRow
#   - m9::assert_workflow_trace_namespace("workflow_trace_test") Ok
#   - m9::assert_workflow_trace_namespace("orac_learn") Err(NamespaceViolation)
#   - m10::ember_gate_all_user_facing_strings PASS
#   - m11::compute_decay_factor(1.0,1.0,1.0,0.02) ≈ 1.0 (epsilon)

CARGO_TARGET_DIR=./target cargo test --lib --release -- integration 2>&1 | tee /tmp/p1-d3-integ.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 30

# Full 4-stage gate across all Day 0-2 modules (--workspace --all-targets):
cargo check --workspace --all-targets --all-features 2>&1 | tee /tmp/p1-d3-check.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 31

cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/p1-d3-clippy.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 32

cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tee /tmp/p1-d3-pedantic.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 33

cargo test --workspace --all-targets --all-features --release 2>&1 | tee /tmp/p1-d3-test.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 34
```

### Step 3.2 — verify-sync invariants 1-7 (Wave 1 subset)

```bash
./scripts/verify-sync.sh --invariants 1-7 2>&1 | tee /tmp/p1-d3-vs.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 35
# Invariants 1-7: src/ DAG matches ULTRAMAP; ≥50 tests/module; no unsafe; no unwrap;
#   public fns documented; clippy clean.
```

### Step 3.3 — Watcher namespace guard runtime spot-check (Hour 66)

```bash
~/.local/bin/stcortex sql "SELECT name, namespace, is_stale FROM consumer WHERE namespace LIKE 'workflow_trace%'" 2>&1 | tee /tmp/p1-d3-ns.txt
# Expect: workflow-trace-<sha> | workflow_trace_*| false
```

### Step 3.4 — First WCP notify (Hour 68)

```bash
WCP_TS=$(date -u +%Y-%m-%dT%H%M%SZ)
WCP_FILE="${HOME}/projects/shared-context/watcher-notices/${WCP_TS}_workflow_trace_phase1_day3.md"
# Write via Write tool (NOT cat <<EOF — Write tool mandatory). Content includes:
#   Cluster D + Cluster A + Cat-10 ship state; Class-E RESOLVED at commit <SHA>;
#   substrate state (stcortex consumer fresh; atuin readable; injection readable);
#   open items for Phase 2A (G9 stays gate; OI-7 Conductor still auto_start=false;
#   W3 status; LTP/LTD 0.043 — Class-I pre-positioned).

~/.local/bin/watcher notify phase1_day3_close "Cluster A+D shipped; ${count} tests; SHA $(git log --oneline -1 | cut -d' ' -f1)"
```

### Step 3.5 — Day 3 commit + Wave-1 merge tag

```bash
git add tests/integration_smoke.rs
git commit -m "test(integration): Phase 1 smoke — m1+m2+m3 substrate reads + cluster-D runtime checks"

# Wave-1 merge tag (per AGENT_VIEW_GITWORKTREES):
git tag -a "wave-1-complete" -m "Wave 1 close: L0 + L4 + L1 — ≥470 tests passing"
git push origin main --tags
git push gitlab main --tags 2>&1 | tee /tmp/p1-d3-push.txt
test "${PIPESTATUS[0]}" -eq 0 || echo "WARN: gitlab push failed; investigate per E1 PAT rotation"
```

---

## Phase-end gate (must be green before runbook-02)

| Check | Command | Pass criterion |
|---|---|---|
| Full 4-stage QG green | re-run Day 3 4-stage chain | exit 0 all stages |
| ≥470 tests passing | `/usr/bin/grep -c "^test .* ok$" /tmp/p1-d3-test.txt` | ≥470 |
| verify-sync invariants 1-7 | `./scripts/verify-sync.sh --invariants 1-7` | PASS |
| First commit SHA recorded | `git log --oneline | head -3` | 3 commits min |
| Wave-1 tag pushed both remotes | `git ls-remote origin wave-1-complete && git ls-remote gitlab wave-1-complete` | both SHAs match local |
| stcortex consumer registered (or skip documented) | `/tmp/p1-m2-consumer.txt` | match OR D-file skip note |
| WCP notify filed | `ls ~/projects/shared-context/watcher-notices/ | tail -5` | newest = Day 3 notice |
| Watcher Class-E resolved | `~/.local/bin/watcher list | /usr/bin/grep class_e_resolved` | match |

---

## Failure modes register

| # | Failure | Detection | Mitigation |
|---|---|---|---|
| 1 | **stcortex unreachable mid-Day-2 m2 registration** | `stcortex status` returns DOWN at Step 2.3 | document deferral in D-file; tests use mock path; re-attempt registration at start of Phase 2A |
| 2 | **m11 SunsetPhase state machine landed Day 1 (G2 violation)** | `/usr/bin/grep "SunsetPhase" src/m11_*/mod.rs` returns match | revert; SunsetPhase belongs to Phase 2B Wave 3 with m30 bank rows |
| 3 | **PIPESTATUS swallow in per-module gate** | gate prints PASS but stderr shows clippy errors | always `${PIPESTATUS[0]}` after each pipe; per-stage abort with explicit exit code (AP-Hab-05) |
| 4 | **`cp -f` alias trap if scripting binary placement** | binary copy fails or prompts interactively | NEVER bare `cp -f`; always `/usr/bin/cp -f` (AP-Hab-06) |
| 5 | **Test count over-report by subagent** | rust-pro subagent claims "60 tests" but `/usr/bin/grep -c "^test .* ok$"` shows 40 | Wave-end Command independently re-runs with `--message-format=json | jq` (AP-Drift-04) |
| 6 | **Ember gate W3 Held verdict on Day 1 string** | `cargo test -- ember_gate` shows EMBER-HELD | fix string per cluster-D spec §m10 HELD remediation; do not commit Held strings |
| 7 | **Pedantic dilution on doc-less pub items** | clippy::pedantic emits `missing_docs_in_private_items` cascading | doc all pub items per GOD_TIER_RUST rule 9; never `#![allow(missing_docs)]` permanently |
| 8 | **Worktree contamination** (if Wave 1 used worktrees: rare on Day 0-3) | `target/` symlinked between trees | per-worktree `target/`; no symlink (AP-V7-07) |

---

## Watcher flag pre-positioning (Phase 1)

| Class | Activates | Day |
|---|---|---|
| **A** activation | first commit (Class-E resolution); Wave-1 tag; first stcortex registration | 0, 2, 3 |
| **B** hand-off boundary | m2 registers consumer (first DB cross-substrate write) | 2 |
| **C** confidence-gate refusal | Ember CI rejects on Held verdict | 1 |
| **D** four-surface drift | Day 3 WCP notify must match plan.toml status fields | 3 |
| **F** AP24 boundary | every per-module commit checked for src/ growth = plan growth | 0-3 |
| **G** substrate-frame confusion | m11 must ship pure formula only (not "sunset" as verb) | 1 |
| **H** atuin proprioception | atuin WAL busy-timeout on m1 reads | 2 |
| **I** Hebbian silence | LTP/LTD < 0.05 visible in m8 startup check | 1, 3 |

---

## Atuin trajectory anchors (Phase 1)

```bash
atuin search "habitat-bootstrap" --before 4d
atuin search "cargo init.*workflow-trace" --before 4d
atuin search "cargo check" --before 4d | head -20
atuin search "cargo clippy" --before 4d | head -20
atuin search "cargo test --lib --release" --before 4d
atuin search "git commit.*genesis" --before 4d
atuin search "git tag.*wave-1-complete" --before 4d
atuin search "stcortex consumers" --before 4d
atuin scripts run hab-quality-gate --before 4d
```

Each must return ≥1 hit. Gaps = Class-H proprioception anomaly (commands fired outside atuin shell).

---

*runbook-01 authored 2026-05-17 by Command (V7 author wave subagent)*
