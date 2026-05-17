---
title: Runbook 03 — Phase 2B Active Build (Days 12-21) — KEYSTONE m20 4-internal-pass
date: 2026-05-17 (S1001982)
kind: planning-only operational runbook
phase: 2B
days: 12-21 (9 calendar days; densest build phase)
owner: Command-2 (Cluster F + H lead) + Command-3 (Cluster G librarian-lane) + rust-pro subagents
modules: m20 m21 m22 m23 m30 m31 m32 m33 m40 m41 m42
binaries: wf-crystallise (m20-23, m40-42) + wf-dispatch (m30-33)
loc_estimate: ~2,250
status: planning-only · activates after wave-2-complete (runbook-02 close)
---

# Runbook 03 — Phase 2B Active Build (Days 12-21) — KEYSTONE m20 4-internal-pass

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · sibling: [[runbook-02-phase-2A-measure-only]] · [[runbook-04-phase-3-integration]]
>
> Cites: `the-workflow-engine-vault/deployment framework/phase-2B-build-clusters-F-G-H.md` (narrative source). Cross-ref: [[../MODULE_PLANS/cluster-F.md]] (KEYSTONE), [[../MODULE_PLANS/cluster-G.md]] (dispatch + 4-agent verifier), [[../MODULE_PLANS/cluster-H.md]] (substrate feedback), [[../STANDARDS/GOD_TIER_RUST.md]], [[../KEYWORDS_20.md]] (KEYSTONE, PrefixSpan, EscapeSurfaceProfile, Conductor, AP30, single-phase).

---

## Overview

Phase 2B is the densest build phase: 11 modules across 3 clusters in 9 days. **Cluster F m20 is the KEYSTONE** — the PrefixSpan-based N-step sequential pattern miner with no boilerplate ancestor beyond ~30%. **Cluster G** closes CC-4 (Proposal → Bank → Dispatch) and introduces the **HUMAN-IN-THE-LOOP** gate at m30 + 4-agent verifier at m33. **Cluster H** closes CC-5 (substrate learning loop). By Day 21 close, three structural gaps are owned by working code: N-step sub-graph detection, verification-gated dispatch (part of Gap 3 — EscapeSurfaceProfile schema), and CC-5 substrate-feedback close. The gate before this runbook is `wave-2-complete` (runbook-02 close). The gate after is `wave-3-complete` (= phase-2-complete) → control transfers to `runbook-04-phase-3-integration.md`.

**m20 4-INTERNAL-PASS:** the KEYSTONE is too large for one author session. It splits across **4 internal passes Days 12-15** — skeleton → algorithm → Wilson CI gate → variant selection. Each pass has its own gate. This is the V7 KEYWORDS_20 KEYSTONE rule.

---

## Pre-flight checklist

```bash
set -o pipefail
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
WF_ROOT="/home/louranicas/claude-code-workspace/workflow-trace"
cd "$WF_ROOT"

# (1) wave-2-complete tag on both remotes:
git ls-remote --tags origin | /usr/bin/grep "wave-2-complete" | tee /tmp/p2b-pre-origin.txt
git ls-remote --tags gitlab | /usr/bin/grep "wave-2-complete" | tee /tmp/p2b-pre-gitlab.txt
test -s /tmp/p2b-pre-origin.txt && test -s /tmp/p2b-pre-gitlab.txt || exit 1

# (2) verify-sync invariants 1-13:
./scripts/verify-sync.sh --invariants 1-13 2>&1 | tee /tmp/p2b-pre-vs.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 1

# (3) Conductor maturity probe (m32 dependency):
curl -s -m 1 http://localhost:8141/health 2>&1 | tee /tmp/p2b-pre-conductor.txt
# Outcomes:
#   200 + enforcer_active=true → m32 can wire to live mode
#   non-200 OR auto_start=false → m32 ships in refuse-mode (still authorised; Phase 3 wires live)

# (4) m14 first lift snapshot reachable:
# (target/release/wf-crystallise lift --raw)
# n may still be <20 — that's OK; m20 will refuse to propose; document.

# (5) Worktree provisioning (Wave 3 — Luke @ terminal):
#   git worktree add ../wt-l6-keystone -b wt/l6/wave3-${TS}
#   git worktree add ../wt-l7-dispatch -b wt/l7/wave3-${TS}
#   git worktree add ../wt-l8-feedback -b wt/l8/wave3-${TS}
# Per-worktree: CARGO_TARGET_DIR=./target (NO symlink; AP-V7-07)
```

---

## Days 12-15 — Cluster F KEYSTONE (m20 4-INTERNAL-PASS + m21 + m22 + m23)

### Day 12 — m20 PASS 1: Skeleton

```bash
# Worktree: wt-l6-keystone (or main if no contention)
# Author src/m20_cascade_iterator/{mod.rs,types.rs,miner.rs,builder.rs}:
#   - types: StepToken, SequentialPattern, PatternConfidence, ClusterFError taxonomy
#   - trait: SequentialPatternMiner
#   - structs: CascadeIterator, ProposalBuilder
#   - ALL methods: todo!() stubs
#   - Serde derives, lifetime annotations, trait bounds — get these RIGHT here, not later

# Mechanical gate on skeleton:
cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic 2>&1 | tee /tmp/p2b-d12-m20-skel.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 201

git add src/m20_cascade_iterator/ && git commit -m "feat(cluster-f): m20 PASS-1 skeleton — types, trait, structs, todo() bodies (gate-clean)"
```

**Inputs:** wave-2 modules; cluster-F spec. **Outputs:** compilable skeleton. **Verification:** clippy/pedantic clean on todo!() bodies. **Failure modes:** trait bound mismatch surfaces NOW not Day 13; lifetime over-borrow appears NOW. **Watcher class:** A (KEYSTONE activation), G (substrate-frame: PrefixSpan is sub-graph mining not "intent" inference). **Mitigates:** AP-V7-09.

### Day 13 — m20 PASS 2: PrefixSpan core

```bash
# Author src/m20_cascade_iterator/miner.rs:
#   - fn project(prefix: &[StepToken], db: &SequenceDatabase, max_gap: usize) -> ProjectedDatabase
#   - fn prefix_span(prefix: &[StepToken], projected: &ProjectedDatabase, min_support: usize, max_length: usize) -> Vec<SequentialPattern>
#   - recursion with bounded right-gap (MAX_GAP_STEPS=5), unbounded left
#   - Lift Kahn topological sort scaffold from m49_task_graph.rs (~50% reuse)

# Tests on synthetic sequence databases (write FIRST per TDD):
#   - linear sequence: a→b→c (k=1 → 3 patterns; k=2 → 2; k=3 → 1)
#   - diamond: a→{b|c}→d
#   - gap-allowed: max_gap=2 admits a..b but rejects a...b (3 gaps)
#   - patterns longer than max_length: rejected
#   - empty db → empty output
#   - min_support boundary (n=19 vs n=20)

cargo test --lib --release -- m20_miner 2>&1 | tee /tmp/p2b-d13-m20-miner.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 202

git add src/m20_cascade_iterator/miner.rs tests/m20_miner.rs \
  && git commit -m "feat(cluster-f): m20 PASS-2 PrefixSpan — projection + recursion + gap-allowed matching"
```

**Inputs:** PASS-1 skeleton. **Outputs:** working PrefixSpan core; 5+ synthetic-db tests green. **Verification:** PrefixSpan correctness on each scenario; recursion termination proven by max_length. **Failure modes:** projection over-scans without MAX_GAP_STEPS guard → infinite candidates; right-gap not enforced — write boundary test FIRST. **Watcher class:** A. **Mitigates:** AP-V7-09 (substrate-frame: this mines sequences, doesn't infer user goals).

### Day 14 — m20 PASS 3: Wilson CI + F2 hard gate

```bash
# Author src/m20_cascade_iterator/builder.rs:
#   - fn wilson_95(p_hat: f64, n: usize) -> Option<(f64, f64)>
#       returns None if n < 20; else (lower, upper) bounded [0,1]
#   - ProposalBuilder::build() — F2 ENFORCEMENT AT CONSTRUCTION
#       if pattern.support < 20 { return Err(ClusterFError::SampleSizeBelowF2) }
#       NEVER returns 0.0 fitness on n<20
#   - CC-3 m14 stabilisation gate: if m14 evidence variance >= threshold → defer

# Property tests (proptest 10000 iters):
#   - For all (p, n): if n<20 then wilson_95 returns None
#   - For all (p, n): if n>=20 then ci_lower <= p_hat <= ci_upper AND both in [0,1]
#   - ProposalBuilder::build with pattern.support=19 returns Err; =20 succeeds

cargo test --lib --release -- m20_builder 2>&1 | tee /tmp/p2b-d14-m20-builder.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 203

# F2 grep audit:
/usr/bin/grep -nE "if pattern.support < 20|SampleSizeBelowF2|Option<.*Confidence>" \
  src/m20_cascade_iterator/builder.rs | tee /tmp/p2b-d14-f2.txt
test -s /tmp/p2b-d14-f2.txt || { echo "AP-WT-F2 VIOLATION"; exit 203; }

git add src/m20_cascade_iterator/builder.rs tests/m20_builder.rs \
  && git commit -m "feat(cluster-f): m20 PASS-3 Wilson CI + F2 hard gate at ProposalBuilder::build"
```

**Inputs:** PASS-2; m14 evidence reader. **Outputs:** F2-enforced ProposalBuilder. **Verification:** F2 enforcement at construction (not at runtime); ci bounds non-negative + within [0,1]. **Watcher class:** C (refusal at n<20). **Mitigates:** AP-WT-F2.

### Day 15 — m20 PASS 4: Variant selection + m21 + m22 start

```bash
# Author src/m20_cascade_iterator/variants.rs + workflow_core::similarity:
#   - fn normalized_edit_distance(a: &[StepToken], b: &[StepToken]) -> f64  (Levenshtein normalised by max length)
#   - fn select_variants(patterns: Vec<SequentialPattern>, k: usize) -> Vec<Variant>  (top-K by ASCENDING edit distance from canonical)
#   - near-miss band [0.25, 0.60] preserved per cluster-F spec
#   - wire CascadeIterator::iterate() end-to-end

cargo test --lib --release -- m20 2>&1 | tee /tmp/p2b-d15-m20-full.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 204
count=$(/usr/bin/grep -cE "^test .* ok$" /tmp/p2b-d15-m20-full.txt)
test "$count" -ge 90 || { echo "FAIL m20 ${count}<90 tests (KEYSTONE budget)"; exit 204; }

# Per-module 4-stage gate:
cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m20 2>&1 | tee /tmp/p2b-d15-m20-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 204

# m21 + m22 skeletons start (full impl on Days 15-17 — overlap):
# Author src/m21_variant_builder/mod.rs (skeleton) — IQR outlier detection (Pareto frontier wallclock vs cost)
# Author src/m22_kmeans_clusterer/mod.rs (skeleton) — K=5 K-means on feature vectors

git add src/m20_cascade_iterator/ src/m21_variant_builder/ src/m22_kmeans_clusterer/ \
  workflow_core/src/similarity/ \
  && git commit -m "feat(cluster-f): m20 PASS-4 variant selection + Levenshtein similarity (~90 tests)"
```

**Watcher class:** A (KEYSTONE first end-to-end fire). **Mitigates:** AP-WT-F2 (transitive).

---

## Day 15-17 — m21 + m22 + m23 (parallel with G start)

### Day 16 — m21 `variant_builder`

```bash
# Author src/m21_variant_builder/ per cluster-F spec §m21:
#   - IQR-based step-duration outlier detection
#   - wallclock-vs-cost Pareto frontier
#   - Lift topological sort (~50%) from m49_task_graph.rs + dedup (~60%) from povm-v2_reinforcement.rs

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m21 2>&1 | tee /tmp/p2b-d16-m21-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 211
git add src/m21_variant_builder/ && git commit -m "feat(cluster-f): m21 variant_builder — IQR + Pareto (≥70 tests)"
```

### Day 17 — m22 `kmeans_clusterer` + m23 `proposer`

```bash
# m22: K-means K=5 on feature vectors (cost, lift, latency, deviation, success_rate)
#   Lift rolling-mean from m39_fitness_tensor (~70%) + dedup from povm-v2_reinforcement (~60%)
cargo test --lib --release -- m22 2>&1 | tee /tmp/p2b-d17-m22-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 221

# m23: gradient-preservation proposer; n≥5 deviation-relaxed (NOT n≥20 — m20 already enforces F2)
#   Composes m20+m21+m22 outputs → Proposal with status=AwaitingReview
#   NEVER calls m30::accept — emits Proposal only; CC-4 contract
cargo test --lib --release -- m23 2>&1 | tee /tmp/p2b-d17-m23-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 222

git add src/m22_kmeans_clusterer/ src/m23_proposer/ \
  && git commit -m "feat(cluster-f): m22 K-means + m23 proposer — CC-4 emit Proposal only"
```

**Watcher class:** A (Cluster F close), G. **Mitigates:** AP-WT-F5 (bank-creep: m23 NEVER calls m30::accept directly).

---

## Day 15-19 — Cluster G (m30 + m31 + m32 + m33) — Command-3 librarian-lane

### Day 15 — m30 `workflow_bank` (overlap with m20 PASS-4)

```bash
# Worktree: wt-l7-dispatch (Command-3)
# Author src/m30_workflow_bank/ per cluster-G spec:
#   - SQLite WAL workflow_bank.db
#   - BankDb::accept(workflow, accepted_by: HumanIdentity) — REJECTS accepted_by=auto OR null
#   - BankDb::eligible(escape_profile: EscapeSurfaceProfile) — filters by surface ordinal
#   - BankDb::apply_decay_tick(now: i64) — calls m11::compute_decay_factor
#   - EscapeSurfaceProfile enum: ReadOnly < HostWrite < Network < SandboxEscape < Destructive (Ord derive)
#   - Lift conductor_state.rs WAL constructor (~70%) + conductor_divergence.rs Rule trait (~50%)

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m30 2>&1 | tee /tmp/p2b-d15-m30-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 301

# EscapeSurfaceProfile cardinality + Ord assertion (verify-sync invariant #19):
/usr/bin/grep -cE "ReadOnly|HostWrite|Network|SandboxEscape|Destructive" \
  src/m30_workflow_bank/types.rs | tee /tmp/p2b-d15-esp.txt

# AP-WT-F5 audit (bank creep):
/usr/bin/grep -nE "accepted_by:.*\"auto\"|accepted_by.*None" src/m30_workflow_bank/ \
  | tee /tmp/p2b-d15-f5.txt
test ! -s /tmp/p2b-d15-f5.txt || { echo "AP-WT-F5 VIOLATION"; exit 301; }

git add src/m30_workflow_bank/ && git commit -m "feat(cluster-g): m30 workflow_bank — HUMAN-IN-LOOP accept; EscapeSurfaceProfile schema"
```

**Watcher class:** A (Bank activation), F (AP24-style human gate). **Mitigates:** AP-WT-F5 (bank creep), AP-WT-F1 (bank ossification — m11 decay wired).

### Day 16 — m31 `selector` + m33 `verifier`

```bash
# m31: composite score = α·fitness + β·recency + γ·frequency + δ·diversity (0.40/0.25/0.20/0.15)
#   Bigram Jaccard similarity for diversity
#   10-gen cooldown + 50% mono-parameter rejection + round-robin
#   Lifts m40_mutation_selector (~70%)
cargo test --lib --release -- m31 2>&1 | tee /tmp/p2b-d16-m31-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 311

# m33 4-agent verifier (gold-standard):
#   Lift SKILL-pre-deploy-hardening.md 4-agent pattern (~80%)
#   Mechanical Wave 1 → parallel-agent Wave 2 → boolean AND Wave 3 → audit-first write
#   definition_hash via FNV-1a 64 of steps_json
#   TTL: 7 days
#   CC-6 contract: update last_verified_at BEFORE returning PASS
cargo test --lib --release -- m33 2>&1 | tee /tmp/p2b-d16-m33-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 331

git add src/m31_selector/ src/m33_verifier/ \
  && git commit -m "feat(cluster-g): m31 selector + m33 4-agent verifier — CC-6 contract"
```

**Watcher class:** A, C, F. **Mitigates:** AP-WT-F4 (premature dispatch — m33 must verify first).

### Day 17 — m32 `dispatcher` (5-check pre-dispatch + Conductor-only)

```bash
# Author src/m32_dispatcher/ per cluster-G spec:
#   - 5-check pre-dispatch sequence IN ORDER:
#       (1) probe_conductor() — http get :8141/health (RAW socket addr; NO http:// prefix per BUG-033)
#       (2) m33 TTL fresh (last_verified_at + 7d > now)
#       (3) definition_hash match (current steps_json hash == verified hash)
#       (4) sunset guard (m11 weight > prune threshold)
#       (5) dispatch_cooldown (kv-tracked, prevent re-dispatch within window)
#   - Refuse-mode: CONDUCTOR_DISPATCH_ENABLED!=1 → ERROR + DispatchError::ConductorDispatchDisabled
#       (NEVER silent no-op; full message to stdout)
#   - Display-before-step BANNER on stdout (mandatory, not optional log) — EscapeSurfaceProfile shown
#   - Lift conductor_enforcement.rs (~80%) + conductor_state.rs WAL (~70%) + m24_povm_bridge.rs (gold standard wire shape)

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m32 2>&1 | tee /tmp/p2b-d17-m32-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 321

# verify-sync invariant #20: m32 NEVER invokes workflow exec directly:
/usr/bin/grep -nE "dispatch_direct|exec_local|execute_workflow\(" src/m32_dispatcher/ \
  | tee /tmp/p2b-d17-direct.txt
test ! -s /tmp/p2b-d17-direct.txt || { echo "VS-20 VIOLATION: m32 has direct exec path"; exit 321; }

# AP-Hab-12 trap: raw socket addr in HTTP, NO http:// prefix
/usr/bin/grep -nE "\"http://" src/m32_dispatcher/ | tee /tmp/p2b-d17-prefix.txt
test ! -s /tmp/p2b-d17-prefix.txt || { echo "AP-Hab BUG-033 RISK: http:// prefix in m32"; exit 321; }

# Manual refuse-mode legibility check:
CONDUCTOR_DISPATCH_ENABLED= cargo test --lib --release -- m32_refuse_mode 2>&1 | tail -10

git add src/m32_dispatcher/ && git commit -m "feat(cluster-g): m32 dispatcher — Conductor-only + 5-check + refuse-mode (VS-20 enforced)"
```

**Watcher class:** A (refuse-mode → live-mode transition pre-positioned for Phase 3), C, F. **Mitigates:** AP-WT-F4, AP-WT-F6 (self-dispatch refusal), verify-sync VS-20.

---

## Day 17-21 — Cluster H Substrate Feedback (m40 + m41 + m42)

### Day 17 — m40 `nexus_event_emitter` skeleton (overlap with m32)

```bash
# Worktree: wt-l8-feedback
# Author src/m40_synthex_emit/ per cluster-H spec:
#   - dual-transport outbox-first + HTTP fire-and-forget
#   - circuit breaker FSM (Closed → Open → HalfOpen)
#   - NexusEvent re-declared LOCALLY (Option A untyped JSON) — DO NOT import synthex_v2
#   - posted=true only after HTTP 2xx
#   - outbox sweep task: retry up to max_attempts=5 with exponential backoff ±25% jitter
#   - compaction removes posted=true entries older than 24h
#   - Lift m22_synthex_bridge.rs (~90%) + m22_synthex_async.rs circuit breaker (~95%)

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m40 2>&1 | tee /tmp/p2b-d17-m40-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 401

git add src/m40_synthex_emit/ && git commit -m "feat(cluster-h): m40 nexus_event_emitter — outbox-first; circuit breaker; jitter"
```

### Day 18 — m41 `lcm_rpc_client`

```bash
# Author src/m41_lcm_router/ per cluster-H spec:
#   - JSON-RPC 2.0 newline-framed over Unix domain socket
#   - 30s read timeout
#   - ONLY lcm.loop.create with max_iters: 1 (NO hypothetical lcm.deploy)
#   - Deploy-shape detection in m32 (NOT m41); m41 only fires when StepKind::Deploy flag set
#   - Lift m22_synthex_async.rs circuit breaker (~95%) + lcm_supervisor.rs client-side mirror

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m41 2>&1 | tee /tmp/p2b-d18-m41-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 411

git add src/m41_lcm_router/ && git commit -m "feat(cluster-h): m41 lcm_rpc_client — JSON-RPC 2.0 over UDS, 30s timeout"
```

### Day 19 — m42 `povm_hebbian_dual_path` + CC-5 fan-out

```bash
# Author src/m42_povm_dual/ per cluster-H spec:
#   - dual-path: povm_overlap_active=true (default → 2026-07-10) routes to POVM
#                                    =false routes to stcortex via m13
#   - post-cutover stcortex unavailability: log ERROR + return SubstrateUnavailable
#       (NO silent POVM fallback — CLAUDE.md memory row 8 stcortex policy)
#   - Fitness-delta constants WIRED AS NAMED CONSTANTS (NOT magic numbers):
#       PASS_VERIFIED = +0.10, PASS = +0.05, BLOCKED = -0.05, FAIL = -0.10
#   - Lift m24_povm_bridge.rs (~85% gold standard)
#   - AP30 enforcement: every retrieval_id prefixed workflow_trace_

cargo check && cargo clippy --workspace --all-targets -- -D warnings && \
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic && \
cargo test --lib --release -- m42 2>&1 | tee /tmp/p2b-d19-m42-gate.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 421

# AP30 audit on retrieval_ids:
/usr/bin/grep -nE "retrieval_id.*=" src/m42_povm_dual/ | /usr/bin/grep -vE "workflow_trace_" \
  | tee /tmp/p2b-d19-ap30.txt
test ! -s /tmp/p2b-d19-ap30.txt || { echo "AP-Hab-03 / AP30 VIOLATION in m42 retrieval_ids"; exit 421; }

# Wire CC-5 fan-out in m32 (m32 already shipped Day 17; this is the edit):
#   After Conductor accepts request, m32 fires WorkflowDispatchEvent to m40/m41/m42 channel.
#   Fan-out failure non-fatal (log + continue).
# Re-run m32 tests after the wiring:
cargo test --lib --release -- m32 2>&1 | tail -5

git add src/m42_povm_dual/ src/m32_dispatcher/cc5_fanout.rs \
  && git commit -m "feat(cluster-h): m42 povm_hebbian_dual_path + CC-5 m32 fan-out (AP30 enforced)"
```

**Watcher class:** A (CC-5 first close), B (cross-substrate boundary), I (Hebbian — first reinforce write through CC-5 loop). **Mitigates:** AP-Hab-03 (AP30), AP-Drift-06 (bridge contract drift — run `bridge-contract` skill).

---

## Day 19-21 — Cross-Cluster Integration Testing

### Day 20 — CC-4 + CC-6 integration

```bash
# Author tests/integration/cc4_proposal_pipeline.rs:
#   - m23 emits Proposal → proposals table with status=AwaitingReview
#   - Simulate `wf-crystallise propose accept <id>` (m12 CLI subcommand)
#   - m30::accept called — RECORD this; m23 NEVER called it directly
#   - Proposal row transitions AwaitingReview → Accepted only AFTER human command
cargo test --workspace --all-targets --all-features --release -- cc4 2>&1 | tee /tmp/p2b-d20-cc4.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 501

# Author tests/integration/cc6_definition_drift.rs:
#   - m33 verifies workflow → last_verified_at recorded
#   - m30 row retrieved
#   - m32 5-check gate passes
#   - Edit steps_json → re-run m32 5-check → DispatchError::DefinitionDrifted returned
cargo test --workspace --all-targets --all-features --release -- cc6 2>&1 | tee /tmp/p2b-d20-cc6.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 502
```

### Day 21 — CC-5 integration + KEYSTONE gap demonstration

```bash
# Author tests/integration/cc5_substrate_feedback.rs:
#   - m32 dispatch (mock Conductor accepting) → WorkflowDispatchEvent emitted
#   - m40 outbox contains event (posted may be false until http)
#   - m42 ReinforcePayload contains correct fitness_delta per outcome (use named constants)
#   - AP30 namespace prefix asserted on all pathway IDs
cargo test --workspace --all-targets --all-features --release -- cc5 2>&1 | tee /tmp/p2b-d21-cc5.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 503

# KEYSTONE gap demonstration test:
#   - Synthetic sequence DB → m20 PrefixSpan
#   - top-K variants ordered ASCENDING by edit distance from canonical
#   - Wilson CI bounds non-negative
#   - m23 emits Proposal with deviation_shaped: false on undeviated proposal
cargo test --workspace --all-targets --all-features --release -- keystone_gap 2>&1 | tee /tmp/p2b-d21-keystone.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 504

# Full Wave-3 4-stage gate (--workspace --all-targets --all-features):
cargo check --workspace --all-targets --all-features 2>&1 | tee /tmp/p2b-d21-check.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 510
cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/p2b-d21-clippy.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 511
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tee /tmp/p2b-d21-pedantic.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 512
cargo test --workspace --all-targets --all-features --release 2>&1 | tee /tmp/p2b-d21-test.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 513

# verify-sync invariants 14-20 (Wave 3 subset):
./scripts/verify-sync.sh --invariants 1-20 2>&1 | tee /tmp/p2b-d21-vs.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 514

# Mutation test on KEYSTONE (target ≥70% kill-rate):
cargo +nightly mutants --package workflow-trace --filter "m20_cascade_iterator::" 2>&1 | tee /tmp/p2b-d21-mut.txt
# Record: kill-rate must be ≥70% or document gap

# Wave-end orchestrator checklist:
./scripts/wave-end-checklist.sh 3 2>&1 | tee /tmp/p2b-d21-wave-end.txt
test "${PIPESTATUS[0]}" -eq 0 || exit 515

# Wave-3 tag + push:
git tag -a "wave-3-complete" -m "Wave 3 / phase-2-complete: Clusters F+G+H — KEYSTONE m20 shipped; CC-5 first close"
git tag -a "phase-2-complete" -m "Phase 2 close (Wave 1+2+3 integrated)"
git push origin main --tags
git push gitlab main --tags 2>&1 | tee /tmp/p2b-d21-push.txt

# WCP notify Watcher:
~/.local/bin/watcher notify phase2_close \
  "Wave 3 tag; KEYSTONE shipped; CC-5 first close ready for Phase 3; SHA $(git log --oneline -1 | cut -d' ' -f1)"
```

---

## Phase-end gate (must be green before runbook-04)

| Check | Command | Pass criterion |
|---|---|---|
| Full --workspace 4-stage QG | re-run Day 21 chain | exit 0 |
| ≥1,594 cumulative tests | `/usr/bin/grep -c "^test .* ok$" /tmp/p2b-d21-test.txt` | ≥1594 |
| verify-sync 1-20 | `./scripts/verify-sync.sh --invariants 1-20` | PASS |
| KEYSTONE m20 mutation kill-rate | `/tmp/p2b-d21-mut.txt` | ≥70% |
| F2 enforced at ProposalBuilder::build | grep audit | match |
| AP30 namespace prefix on m42 retrieval_ids | grep audit | clean |
| EscapeSurfaceProfile cardinality = 5 | `/usr/bin/grep -cE "ReadOnly\|HostWrite\|Network\|SandboxEscape\|Destructive" src/m30_*/types.rs` | =5 |
| m32 NO direct exec path (VS-20) | grep audit | clean |
| m32 NO http:// prefix in URLs | grep audit | clean |
| Wave-3 tag + phase-2-complete both remotes | `git ls-remote --tags origin/gitlab` | both match |

---

## Failure modes register

| # | Failure | Detection | Mitigation |
|---|---|---|---|
| 1 | **m20 PASS-1 skeleton lifetime explosion** | trait-bound errors cascade through PASS-2 | author skeleton WITH all lifetime annotations correct Day 12; don't defer |
| 2 | **PrefixSpan over-scans without right-gap bound** | tests hang on diamond DB | MAX_GAP_STEPS=5 enforced in projection loop; write boundary test FIRST |
| 3 | **F2 enforced only at runtime not at ProposalBuilder::build** | builder produces NaN/0.0 on n<19 | enforce at construction; return Err; never produce a Proposal that bypasses (AP-WT-F2) |
| 4 | **m30 admits workflow with accepted_by=auto or null** | grep audit shows pattern | hard refusal; `wf-crystallise propose accept <id>` is the only path (AP-WT-F5) |
| 5 | **m32 contains direct exec path (VS-20 violation)** | grep `execute_workflow\|dispatch_direct` | revert; Conductor-only is non-negotiable |
| 6 | **m32 URL contains http:// prefix** | grep audit | use raw socket addr (BUG-033 gold-standard fix) |
| 7 | **m42 silent POVM fallback on stcortex down** | grep absence of `SubstrateUnavailable` return | per CLAUDE.md row-8 policy: log ERROR + return; never silent fallback |
| 8 | **m42 retrieval_id without workflow_trace_ prefix** | AP30 grep audit | enforce m9 namespace guard at write boundary |
| 9 | **Worktree contamination** (Wave 3 = 3 worktrees) | `ln -s ../wt-X/target` | per-worktree target (AP-V7-07) |
| 10 | **Mutation kill-rate < 70% on KEYSTONE m20** | cargo-mutants report | author more invariant assertions in tests; AP-Test-01 (coverage theatre) |
| 11 | **PIPESTATUS swallow** in 4-stage chain | gate green; stderr screaming | always `${PIPESTATUS[0]}` (AP-Hab-05) |
| 12 | **CC-5 fan-out failure treated as fatal** | dispatch fails when m40/m41/m42 unreachable | fan-out failure is log+continue, not error return |

---

## Watcher flag pre-positioning (Phase 2B)

| Class | Activates | Day |
|---|---|---|
| **A** activation | m20 PASS-1 (KEYSTONE start); m30 bank live; CC-5 first close; Wave-3 tag | 12, 15, 21 |
| **B** hand-off boundary | m32 → Conductor probe; m42 → POVM/stcortex write | 17, 19 |
| **C** confidence-gate refusal | m20 F2 refuses n<20; m32 5-check refuses; m33 4-agent REJECT | 14, 17 |
| **D** four-surface drift | Wave-3 WCP notify vs plan.toml status fields | 21 |
| **F** AP24 boundary; god-tier dilution | per-module commits | every day |
| **G** substrate-frame confusion | m20 PrefixSpan as sub-graph mining (not user-intent inference) | 12-15 |
| **H** atuin proprioception | Conductor probe latency; HTTP retry storms | 17-19 |
| **I** Hebbian silence | CC-5 first close — LTP signal expected; if still silent at Day 21 → Class-I escalation | 19-21 |

---

## Atuin trajectory anchors (Phase 2B)

```bash
atuin search "cargo test.*m20" --before 10d
atuin search "cargo test.*keystone" --before 10d
atuin search "cargo +nightly mutants" --before 10d
atuin search "cargo test.*cc4\|cc5\|cc6" --before 10d
atuin search "curl.*8141/health" --before 10d
atuin search "stcortex.*workflow_trace_" --before 10d
atuin search "git tag.*wave-3-complete\|phase-2-complete" --before 10d
atuin scripts run habitat-density --before 10d
atuin scripts run habitat-cross-tensor --before 10d
```

Each must return ≥1 hit. Gaps = Class-H proprioception anomaly.

---

*runbook-03 authored 2026-05-17 by Command (V7 author wave subagent)*
