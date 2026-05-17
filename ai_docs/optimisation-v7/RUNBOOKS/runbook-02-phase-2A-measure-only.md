---
title: Runbook 02 — Phase 2A Measure-Only Build (Days 3-12)
date: 2026-05-17 (S1001982)
kind: planning-only operational runbook
phase: 2A
days: 3-12 (9 calendar days)
owner: Command-2 (build-executor) + rust-pro subagents + Command (orchestrator)
modules: m7 · m4 · m5 · m6 · m12 · m13 · m14 · m15
verb_budget: record · correlate · emit · refuse (passive Phase A — v1.2 locked)
loc_estimate: ~1,180
status: planning-only · activates after Wave-1 tag (runbook-01 close)
---

# Runbook 02 — Phase 2A Measure-Only Build (Days 3-12)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · sibling: [[runbook-01-phase-1-genesis]] · [[runbook-03-phase-2B-active]]
>
> Cites: `the-workflow-engine-vault/deployment framework/phase-2A-build-clusters-B-C-E.md` (narrative source). Cross-ref: [[../GENERATIONS/G3-bidi-flow.md]] (m7 hub edges), [[../MODULE_PLANS/cluster-B.md]] + [[../MODULE_PLANS/cluster-C.md]] + [[../MODULE_PLANS/cluster-E.md]] (per-module spec), [[../STANDARDS/GOD_TIER_RUST.md]], [[../KEYWORDS_20.md]] (F2, PIPESTATUS, verify-sync, single-phase).

---

## Overview

Phase 2A is the **observation and evidence layer**. 8 modules across Clusters B + C + E produce the data that Phase 2B (iteration + proposal + dispatch) operates on. m7 ships FIRST as the central correlation hub — its schema becomes the published contract every later 2A module writes to or reads from. By Day 12 close, the engine can — for the first time — tell whether any workflow produced measurable habitat lift (m14 Wilson CI signal). That signal is the precondition Phase 2B's m20 PrefixSpan iterator requires. The gate before this runbook is `wave-1-complete` tag (runbook-01 close). The gate after is `wave-2-complete` tag → control transfers to `runbook-03-phase-2B-active.md`.

**Verb-lock invariant:** Phase 2A is passive — only `record / correlate / emit / refuse` verbs are admitted. Active verbs (propose / dispatch / verify) are Cluster F/G in Phase 2B. If any 2A module starts to look like it's "selecting" or "deciding" — stop and re-read v1.2 §F2.

---

## Pre-flight checklist

```bash
set -o pipefail
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
WF_ROOT="/home/louranicas/claude-code-workspace/workflow-trace"
cd "$WF_ROOT"

# (1) Wave-1 tag present on both remotes:
git ls-remote --tags origin | /usr/bin/grep "wave-1-complete" | tee /tmp/p2a-pre-origin.txt
git ls-remote --tags gitlab | /usr/bin/grep "wave-1-complete" | tee /tmp/p2a-pre-gitlab.txt
test -s /tmp/p2a-pre-origin.txt && test -s /tmp/p2a-pre-gitlab.txt \
  || { echo "BLOCKED: wave-1-complete not on both remotes"; exit 1; }

# (2) verify-sync invariants 1-7 still pass:
./scripts/verify-sync.sh --invariants 1-7 2>&1 | tee /tmp/p2a-pre-vs.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 1

# (3) Cluster D + Cluster A modules still gate-clean:
cargo check --workspace --all-targets --all-features 2>&1 | tail -5
test "${PIPESTATUS[0]}" -eq 0 || exit 1

# (4) Habitat baseline still healthy:
for port in 8082 8083 8092 8111 8120 8125 8130 8132 8133 8180 10002; do
  code=$(curl -s -o /dev/null -w '%{http_code}' -m 1 "http://localhost:${port}/health" 2>/dev/null || echo "000")
  test "$code" = "200" || echo "WARN port=${port} code=${code}"
done

# (5) atuin baseline:
atuin scripts run habitat-density 2>&1 | tail -5

# (6) Worktree provisioning (post-G9 only — Wave 2):
# Luke @ terminal post-Wave-1-tag (or Command-2 in main if no worktree pressure):
#   for layer in l3-central l2-observe l5-evidence; do
#     branch="wt/${layer%-*}/wave2-${TS}"
#     git worktree add "../wt-${layer}" -b "${branch}"
#   done
```

If any check fails, return to runbook-01 phase-end gate and re-verify.

---

## Day 3-4 — m7 `workflow_arc_record` (Cluster C HUB; SHIPS FIRST)

**Goal:** m7 schema locked; `WorkflowRunRow` struct + `workflow_runs` DDL published as the stable contract. All downstream 2A modules write to or read from this. After Day 4 lock, schema change requires explicit migration bump.

### Step 7.1 — Test-first authorship (Day 3)

```bash
# Author per cluster-C spec §m7 (TEST FILE FIRST per LCM gold-standard pattern):
#   tests/m7_workflow_arc_record.rs (failing tests drive impl order:
#     pragma block → DDL → open_memory → configure_connection → create_all_tables →
#     insert_run → close_run → merge_observation → find_open → find_by_outcome)
# Boilerplate lifts:
#   - 03-sqlite-multi-db/m06_schema.rs : WAL pragma + busy_timeout=5000 + wal_autocheckpoint=100 (~90% lift)
#   - 03-sqlite-multi-db/m07_causal_chain.rs : parse_row + find_by_outcome adaptation (~70% lift)

cargo test --lib --release -- m7_workflow_arc_record 2>&1 | tee /tmp/p2a-d3-m7-test.txt
# Expected initial: tests FAIL (no impl); test names + count visible
```

### Step 7.2 — Impl + Day 4 4-stage gate

```bash
# Author src/m7_workflow_arc_record/{mod.rs, schema.rs, db.rs}:
#   - open_database(path), open_memory(), configure_connection() (WAL + foreign_keys + synchronous=NORMAL)
#   - create_all_tables() (CREATE TABLE workflow_runs with fitness_dimension REAL NOT NULL DEFAULT 0.0)
#   - insert_run() / close_run() / merge_observation(ClusterBObservation::{Cascade,BatternStep,ContextCost})
#   - find_open(limit) / find_by_outcome(outcome, limit)

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m7 2>&1 | tee /tmp/p2a-d4-m7-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 70

# ≥50 tests asserted (F9 zero-weight invariant; WAL pragma; round-trip):
count=$(/usr/bin/grep -cE "^test .* ok$" /tmp/p2a-d4-m7-gate.txt)
test "$count" -ge 50 || { echo "FAIL m7 ${count}<50 tests"; exit 70; }

# Day 4 commit — m7 schema lock:
git add src/m7_workflow_arc_record/ migrations/0001_init.sql tests/m7_workflow_arc_record.rs
git commit -m "feat(cluster-c): m7 workflow_arc_record SHIP — schema lock contract (~50 tests)"
```

**Inputs:** Wave-1 modules; `migrations/0001_init.sql`. **Outputs:** m7 module + 6 public fns + workflow_runs DDL (LOCKED Day 4). **Verification:** 4-stage PASS; ≥50 tests; `find_open(0)` returns empty; `find_by_outcome("ok", 10)` returns closed rows. **Failure modes:** WAL pragma divergence between open_memory and open_database → Watcher Class-D; F9 zero-weight written non-zero → Watcher Class-G; `wal_autocheckpoint` omitted → Class-I (write amplification under Day 10+ stress, not observable Day 4). **Watcher class:** A (schema lock = activation), D, G. **Mitigates:** AP-WT-F9 (fitness distortion), AP-Drift-05 (migration applied but schema unchanged — catch via `sqlite3 db.sqlite ".schema"` diff).

---

## Day 5-6 — m4 `cascade_correlator` (Cluster B)

**Goal:** atuin → m7 cascade path live; F11 opaque-id discipline enforced (FNV-1a XOR, never label substrings).

### Step 4.1 — Test-first F11 + gap-boundary

```bash
# Author tests FIRST per cluster-B spec §m4:
#   - F11 compliance: cluster_id matches /^cascade_cluster_[0-9a-f]{16}$/
#   - NO ALPHA/BETA/GAMMA/LEFT/RIGHT/TR/BR substring leakage
#   - Gap boundary: 29,999ms together; 30,001ms split
#   - min_pane_count filter (single-session → 0 clusters)
#   - ID stability + collision <0.01% over 10k pairs
#   - Kahn dag_depth correctness (3-step chain → depth=2)
#   - Cycle detection logged
#   - Empty atuin → empty vec
#   - Pagination correctness
#   - cc-* binary detection
#   - Overlap boundary

cargo test --lib --release -- m4_cascade_correlator 2>&1 | tee /tmp/p2a-m4-test1.txt
# Expected: tests FAIL (no impl)
```

### Step 4.2 — Impl + 4-stage gate

```bash
# Author src/m4_cascade_correlator/{mod.rs, fnv1a.rs, kahn.rs, sliding_window.rs}:
#   - FNV-1a 64 (~15 LOC fresh, NO external crate; seed 0xcbf29ce484222325u64)
#   - Kahn topological sort (lift from m49_task_graph.rs ~50% reuse)
#   - sliding window with 30,000ms gap rule
#   - assign_cluster_id(window_range, sorted_pane_labels, step_count) -> CascadeClusterId
#   - read_atuin_since(last_id, page_size) -> Vec<AtuinStep>

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m4 2>&1 | tee /tmp/p2a-d6-m4-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 41

# F11 grep audit (AP-WT-F11):
/usr/bin/grep -nE "ALPHA|BETA|GAMMA|LEFT|RIGHT|TR|BR" src/m4_cascade_correlator/ 2>&1 | tee /tmp/p2a-m4-f11.txt
test ! -s /tmp/p2a-m4-f11.txt || { echo "AP-WT-F11 VIOLATION"; exit 41; }

git add src/m4_cascade_correlator/ tests/m4_cascade_correlator.rs
git commit -m "feat(cluster-b): m4 cascade_correlator — FNV-1a opaque ids (≥50 tests; F11 enforced)"
```

**Inputs:** m1 atuin reader + m7 schema. **Outputs:** opaque cluster IDs; `merge_observation::Cascade` call site to m7. **Verification:** F11 grep clean; collision rate <0.01% asserted in tests. **Watcher class:** G (substrate-frame: don't preserve human-readable pane semantics in IDs). **Mitigates:** AP-WT-F11.

---

## Day 7 — m5 `battern_step_record` (Cluster B)

```bash
# Author src/m5_battern_step_record/ per cluster-B spec §m5:
#   - step_label: Option<String> (NEVER required — passive recording)
#   - step_index: u32 + duration_ms: i64
#   - merge_observation::BatternStep call to m7 with step_index + duration_ms round-trip

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m5 2>&1 | tee /tmp/p2a-d7-m5-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 51

git add src/m5_battern_step_record/ && git commit -m "feat(cluster-b): m5 battern_step_record — Option label, JSONB round-trip"
```

**Watcher class:** F. **Mitigates:** active-verb leak (step_label required would be selecting — must stay Option).

---

## Day 8 — m6 `context_cost_record` (Cluster B)

```bash
# Author src/m6_context_cost_record/ per cluster-B spec §m6:
#   - 20-session rolling EMA; F10 invariant: window EXCLUDES Converged outcomes
#   - cost_tokens column update on m7 row (not just JSONB)
#   - baseline computed AFTER filter, not before

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m6 2>&1 | tee /tmp/p2a-d8-m6-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 61

# F10 invariant assertion (AP-WT-F10):
/usr/bin/grep -E "exclude.*Converged|filter.*Converged" src/m6_context_cost_record/mod.rs \
  | tee /tmp/p2a-m6-f10.txt
test -s /tmp/p2a-m6-f10.txt || { echo "AP-WT-F10 VIOLATION: m6 baseline includes Converged"; exit 61; }

git add src/m6_context_cost_record/ && git commit -m "feat(cluster-b): m6 context_cost_record — F10 exclude-Converged EMA baseline"
```

**Watcher class:** F. **Mitigates:** AP-WT-F10.

---

## Day 9 — m12 `report_emitter` (Cluster C)

```bash
# Author src/m12_report_emitter/ per cluster-C spec §m12:
#   - human CLI reports reading from m7 (find_open / find_by_outcome)
#   - subcommands: `wf-crystallise status`, `wf-crystallise lift --raw`, `wf-crystallise lift --cost`
#   - render only; NEVER writes back; NEVER dispatches (passive verb-lock)
#   - all output strings registered in user_facing_strings::ALL for m10 Ember CI

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m12 2>&1 | tee /tmp/p2a-d9-m12-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 12

# Re-run Ember gate over the expanded user_facing_strings::ALL:
cargo test --lib --release -- ember_gate 2>&1 | tee /tmp/p2a-d9-ember.txt
test "${PIPESTATUS[0]}" -eq 0 || { echo "FAIL: ember on m12 strings"; exit 12; }

git add src/m12_report_emitter/ src/bin/wf_crystallise.rs src/user_facing_strings.rs \
  && git commit -m "feat(cluster-c): m12 report_emitter — CLI status/lift surface, Ember-clean"
```

**Watcher class:** C (Ember refusal risk on human-facing copy). **Mitigates:** AP-WT-F7 (Ember bypass).

---

## Day 10 — m13 `stcortex_writer_narrowed` (Cluster C — first WRITE)

**This is the first stcortex WRITE under `workflow_trace_*` namespace. AP30 enforced via m9.**

```bash
# Author src/m13_stcortex_writer_narrowed/ per cluster-C spec §m13:
#   - reads m7 closed runs + m11 compute_decay_factor → builds CorrelationMemory payload
#   - 3-band LTP/LTD gate (per CR-2 reconciliation): write only if confidence in band
#   - calls m9::assert_workflow_trace_namespace() BEFORE every write (defense in depth)
#   - subprocess: ~/.local/bin/stcortex call write_memory ...
#   - on stcortex DOWN: log ERROR; do NOT fall back to POVM (CLAUDE.md memory row 8)

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m13 2>&1 | tee /tmp/p2a-d10-m13-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 13

# Live write smoke (only if stcortex UP):
~/.local/bin/stcortex status 2>&1 | tee /tmp/p2a-d10-stcortex.txt
if grep -q "UP" /tmp/p2a-d10-stcortex.txt 2>/dev/null; then
  # Run integration test that writes a single workflow_trace_smoke memory:
  cargo test --release --test m13_live_smoke 2>&1 | tee /tmp/p2a-d10-m13-live.txt
  test "${PIPESTATUS[0]}" -eq 0 || exit 13

  # Verify it landed:
  ~/.local/bin/stcortex sql \
    "SELECT id, namespace FROM memory WHERE namespace LIKE 'workflow_trace_%' ORDER BY id DESC LIMIT 5" \
    | tee /tmp/p2a-d10-m13-verify.txt
fi

git add src/m13_stcortex_writer_narrowed/ && git commit -m "feat(cluster-c): m13 stcortex_writer_narrowed — AP30 enforced, 3-band gate"
```

**Watcher class:** B (first DB write hand-off), A. **Mitigates:** AP-Hab-03 (AP30), AP-WT-F3 (substrate-input poisoning).

---

## Day 11 — m14 `evidence_aggregator` (Cluster E — F2 hard gate)

**The F2 keystone:** Phase 2A's structural enforcement of n≥20 + Wilson 95% CI. If F2 isn't right here, every downstream m20-m22 inherits a broken signal.

```bash
# Author src/m14_evidence_aggregator/ per cluster-E spec §m14:
#   - parking_lot::RwLock<Option<LiftSnapshot>> + RwLock<Vec<WorkflowLiftContribution>> + RwLock<VecDeque<WorkflowRunRow>>
#   - 5-minute timer in wf-crystallise main loop calls run_cycle()
#   - run_cycle: decay window → ingest new m7 rows → compute lift → emit
#   - F2 HARD GATE in run_cycle:
#       if window.len() < 20 { return LiftSnapshot { lift: None, ci_half: None, n: window.len(), ... }; }
#   - wilson_95(p_hat, n) -> Option<(f64,f64)> for n>=20 else None
#   - individually_significant: bool per workflow (per-workflow F2 flag)
#   - aggregator pattern lifted ~85% from habitat-nerve-center_m3_aggregator_mod.rs

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m14 2>&1 | tee /tmp/p2a-d11-m14-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 14

# F2 grep audit — verify the n<20 path is structurally None, not 0.0:
/usr/bin/grep -nE "lift: None|ci_half: None" src/m14_evidence_aggregator/mod.rs \
  | tee /tmp/p2a-d11-f2.txt
test -s /tmp/p2a-d11-f2.txt || { echo "AP-WT-F2 VIOLATION: m14 returns 0.0 not None"; exit 14; }

# Property test (proptest, 10000 iters):
#   - For all (p, n): if n<20 then wilson_95(p,n) == None
#   - For all (p, n): if n>=20 then ci_lower ≤ p ≤ ci_upper and both in [0,1]

git add src/m14_evidence_aggregator/ && git commit -m "feat(cluster-e): m14 evidence_aggregator — F2 hard gate (Wilson 95% CI, Option<f64> below n=20)"
```

**Watcher class:** C (F2 confidence refusal), I (Hebbian backstop). **Mitigates:** AP-WT-F2 (sample-size inflation).

---

## Day 12 — m15 `pressure_register` + Integration smoke + Wave-2 close

### Step 15 — m15 (~Hour 60-66)

```bash
# Author src/m15_pressure_register/ per cluster-E spec §m15:
#   - JSONL one-event-per-file emit to ~/projects/shared-context/agent-cross-talk/
#   - naming: PHASE-B-RESERVATION-NOTICE-{iso8601_date}-{session_prefix_8}-{event_id:05}.jsonl
#   - atomic write via .tmp + rename
#   - if fires >3 times/session → Class-E candidate

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m15 2>&1 | tee /tmp/p2a-d12-m15-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 15

git add src/m15_pressure_register/ && git commit -m "feat(cluster-e): m15 pressure_register — JSONL one-event-per-file emit"
```

### Step 12.1 — Wave-2 integration smoke

```bash
# Author tests/integration/wave2_smoke.rs covering:
#   - m1 → m4 → m7 path (cascade write)
#   - m1 → m5 → m7 path (battern step write)
#   - m1 → m6 → m7 path (cost update)
#   - m7 → m14 path (evidence aggregation with F2 None when n<20)
#   - m7 → m12 path (CLI rendering)
#   - m7 → m13 → stcortex path (live write to workflow_trace_* namespace)
#   - m9 namespace guard called on every m13 write (mock confirms)
#   - F11 opaque IDs cluster-id-only present (no human-readable substrings)
#   - F10 m6 baseline excludes Converged outcomes
#   - F2 m14 returns None for n<20

cargo test --workspace --all-targets --all-features --release -- wave2_smoke 2>&1 | tee /tmp/p2a-d12-smoke.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 80

# Full --workspace 4-stage gate:
cargo check --workspace --all-targets --all-features 2>&1 | tee /tmp/p2a-d12-check.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 81

cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/p2a-d12-clippy.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 82

cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tee /tmp/p2a-d12-pedantic.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 83

cargo test --workspace --all-targets --all-features --release 2>&1 | tee /tmp/p2a-d12-test.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 84
```

### Step 12.2 — verify-sync invariants 8-13 + Wave-end orchestrator checklist

```bash
./scripts/verify-sync.sh --invariants 1-13 2>&1 | tee /tmp/p2a-d12-vs.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 85

./scripts/wave-end-checklist.sh 2 2>&1 | tee /tmp/p2a-d12-wave-end.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 86
```

### Step 12.3 — Wave-2 merge tag + push + WCP notify

```bash
git tag -a "wave-2-complete" -m "Wave 2 close: Clusters B + C + E — m14 F2 enforced; m7 schema locked"
git push origin main --tags
git push gitlab main --tags 2>&1 | tee /tmp/p2a-d12-push.txt

WCP_TS=$(date -u +%Y-%m-%dT%H%M%SZ)
# Write via Write tool:
#   ~/projects/shared-context/watcher-notices/${WCP_TS}_workflow_trace_phase2A_close.md
# Body: per-module ship state; m14 first lift sample (may be None if n<20 still);
#   stcortex workflow_trace_* memory count (Day 10 first write onwards);
#   open items for Phase 2B (Conductor still OI-7; Watcher §5.1 should be green from runbook-00).

~/.local/bin/watcher notify phase2A_close "Wave 2 tag; F2 gate live; m7 schema locked; SHA $(git log --oneline -1 | cut -d' ' -f1)"
```

---

## Phase-end gate (must be green before runbook-03)

| Check | Command | Pass criterion |
|---|---|---|
| Full --workspace 4-stage QG | re-run Day 12 chain | exit 0 |
| ≥860 cumulative tests | `/usr/bin/grep -c "^test .* ok$" /tmp/p2a-d12-test.txt` | ≥860 |
| verify-sync invariants 1-13 | `./scripts/verify-sync.sh --invariants 1-13` | PASS |
| F2 None path enforced | grep audit | non-empty match |
| F11 opaque-id grep clean | `/usr/bin/grep -E "ALPHA\|BETA\|GAMMA" src/m4_*/` | empty |
| F10 EMA exclude-Converged | grep audit | match |
| m7 schema locked (no migration bump in Days 5-12) | `ls migrations/` | only 0001_init.sql |
| First m14 lift snapshot taken | `wf-crystallise lift --raw` (smoke or actual) | runs without panic |
| Wave-2 tag on both remotes | `git ls-remote --tags origin/gitlab` | match |
| WCP notify filed | `ls ~/projects/shared-context/watcher-notices/` newest | matches Wave-2 close |

---

## Failure modes register

| # | Failure | Detection | Mitigation |
|---|---|---|---|
| 1 | **m7 schema bumped mid-Phase-2A** (any 0002_*.sql appears) | `ls migrations/ | wc -l > 1` | hard refusal; revert; coordinate on D4 lock; AP-Drift-05 risk |
| 2 | **m14 returns 0.0 for n<20 instead of None** | grep `lift: 0\.0` in m14/ | AP-WT-F2 violation — patch ProposalBuilder::build pattern at construction |
| 3 | **m6 EMA baseline includes Converged outcomes** | grep absence of "filter.*Converged" | AP-WT-F10 violation |
| 4 | **m4 cluster_id contains human substring** (ALPHA/BETA/GAMMA/TR/BR) | grep audit | AP-WT-F11 violation; rewrite to FNV-1a XOR |
| 5 | **m13 falls back to POVM on stcortex down** | grep `povm.*write` in m13 | AP-Hab silent-fallback violation; log ERROR + return SubstrateUnavailable |
| 6 | **PIPESTATUS swallow in per-module gate** | gate prints PASS but stderr shows clippy errors | always `${PIPESTATUS[0]}` (AP-Hab-05) |
| 7 | **Worktree contamination** (Wave 2 uses 3 worktrees) | `ln -s ../wt-X/target` visible in any wt dir | per-worktree `target/` (AP-V7-07) |
| 8 | **m12 strings fail Ember gate** | `cargo test -- ember_gate` shows REJECT | fix per cluster-D spec §m10 remediation; common: Equanimity all-caps |
| 9 | **n<20 still at Day 12 close** | atuin history yields <20 closed workflows | NOT a failure — Phase 2A measure-only; m20 PrefixSpan will refuse to propose; document in WCP notify |
| 10 | **active-verb leak** (e.g., m12 "selects" rather than "renders") | code review + grep `propose\|dispatch\|verify` in 2A src/ | hard refusal; verb-lock invariant |

---

## Watcher flag pre-positioning (Phase 2A)

| Class | Activates | When |
|---|---|---|
| **A** activation | m7 schema lock; first m14 lift snapshot; Wave-2 tag | Day 4, 11, 12 |
| **B** hand-off boundary | m13 first stcortex write under workflow_trace_* | Day 10 |
| **C** confidence-gate refusal | m14 returns None on n<20; Ember rejects m12 string | Day 9, 11 |
| **D** four-surface drift | Wave-2 WCP notify vs plan.toml status | Day 12 |
| **F** AP24 boundary; god-tier dilution | per-module commits | every day |
| **G** substrate-frame confusion | m4 opaque-id discipline (no semantic ALPHA/BETA leak) | Day 5-6 |
| **H** atuin proprioception | m1 WAL busy-timeout under m4 stress | Day 5-12 |
| **I** Hebbian silence | LTP/LTD stays below floor; sustained no LTP signal from CR-2 substrate | continuous |

---

## Atuin trajectory anchors (Phase 2A)

```bash
atuin search "cargo test.*m7" --before 12d
atuin search "cargo test.*m4_cascade" --before 12d
atuin search "cargo test.*m14" --before 12d
atuin search "stcortex sql.*workflow_trace_" --before 12d
atuin search "wf-crystallise lift --raw" --before 12d
atuin search "git tag.*wave-2-complete" --before 12d
atuin scripts run habitat-density --before 12d   # quality-density baseline
atuin scripts run habitat-evolution-delta --before 12d  # RALPH delta during Phase 2A
```

Each must return ≥1 hit. Provenance ground truth for orchestrator re-verification (LCM Drift #11).

---

*runbook-02 authored 2026-05-17 by Command (V7 author wave subagent)*
