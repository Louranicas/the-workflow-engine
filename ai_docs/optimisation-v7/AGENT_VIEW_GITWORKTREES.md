---
title: AGENT VIEW + GIT WORKTREES — workflow-trace V7 isolation plan
date: 2026-05-17 (S1001982)
kind: planning-only · per-layer worktree + per-cluster agent allocation
purpose: each layer L0-L8 builds in isolated worktree; merge at Wave-end; no shared mutable state
inheritance: worktree-mastery skill + ULTRAPLATE swarm-orchestrator patterns
---

# AGENT VIEW + GIT WORKTREES — workflow-trace V7

> Back to: [[TASK_LIST_V7_OPTIMISATION.md]] · [[HANDSHAKE_PROTOCOL_TAB1.md]] · [[ULTRAMAP.md]]
>
> **Function:** define exactly which worktree + which agent owns each layer build, with isolation invariants and merge protocol. Activated POST-G9; HOLD-v2 prohibits `git worktree add` now.

---

## Worktree topology (9 layers → 9 worktrees → 3 build waves)

```
main branch
├── wt-l0-core            (L0 — workflow-core lib + types + schemas)              Wave 1
├── wt-l1-ingest          (L1 — m1 + m2 + m3 substrate ingest)                     Wave 1
├── wt-l4-trust           (L4 — m8 + m9 + m10 + m11 aspect-layer)                  Wave 1
├── wt-l2-observe         (L2 — m4 + m5 + m6 observation)                          Wave 2
├── wt-l3-central         (L3 — m7 + m12 + m13 central correlation)                Wave 2
├── wt-l5-evidence        (L5 — m14 + m15 evidence/pressure)                       Wave 2
├── wt-l6-keystone        (L6 — m20 + m21 + m22 + m23 KEYSTONE iteration)          Wave 3
├── wt-l7-dispatch        (L7 — m30 + m31 + m32 + m33 dispatch)                    Wave 3
└── wt-l8-feedback        (L8 — m40 + m41 + m42 substrate feedback)                Wave 3
```

**Branch naming:** `wt/{layer}/{wave}-{ts}` — e.g., `wt/l6/wave3-20260615T0900Z`.

---

## Wave-by-Wave allocation (post-G9, Days 0-21)

### Wave 1 — Foundation (Days 0-3)
**Goal:** L0 + L4 + L1 ship before anything else. L4 must compile before L1 (aspect layer's compile-time gates).

| Worktree | Branch | Agent | Modules | LOC budget | Test budget | Time budget |
|---|---|---|---|---|---|---|
| `wt-l0-core` | `wt/l0/wave1-{ts}` | Command-2 (foreground) | workflow-core lib | ~200 | (lib tests in module owners) | Day 0-1 |
| `wt-l4-trust` | `wt/l4/wave1-{ts}` | Command-2 (foreground) + rust-pro subagent (m11 decay math) | m8, m9, m10, m11 | ~450 | 230 | Day 1-2 |
| `wt-l1-ingest` | `wt/l1/wave1-{ts}` | Command-2 (foreground) | m1, m2, m3 | ~230 | 150 | Day 2-3 |

**Wave-1 merge gate (Day 3):**
- All 3 worktrees rebased onto main
- Integrated branch passes 4-stage QG
- Verify-sync invariants 1-7 PASS
- ≥430 tests passing
- Command tags `wave-1-complete`

### Wave 2 — Measure-only (Days 3-12)
**Goal:** L2 + L3 + L5 measure-only modules ship; m7 central hub schema first; everything else reads it.

| Worktree | Branch | Agent | Modules | LOC budget | Test budget | Time budget |
|---|---|---|---|---|---|---|
| `wt-l3-central` | `wt/l3/wave2-{ts}` | Command-2 (foreground) + rust-pro subagent (m7 schema design) | m7, m12, m13 | ~370 | 180 | Day 3-6 (m7 FIRST) |
| `wt-l2-observe` | `wt/l2/wave2-{ts}` | Command-2 + rust-pro subagents (m4 FNV-1a, m5 Option labels, m6 EMA exclude-Converged) | m4, m5, m6 | ~460 | 170 | Day 6-10 |
| `wt-l5-evidence` | `wt/l5/wave2-{ts}` | Command-2 + rust-pro subagent (m14 Wilson CI) | m14, m15 | ~200 | 110 | Day 10-12 |

**Wave-2 merge gate (Day 12):**
- Integration test: m1→m4→m7 + m1→m5→m7 + m1→m6→m7 + m7→m14 paths all green
- Verify-sync invariants 8-13 PASS
- ≥860 tests passing (cumulative)
- First m14 lift signal observable (even if n<20 NoneVariant)
- Command tags `wave-2-complete`

### Wave 3 — Active + Feedback (Days 12-21)
**Goal:** KEYSTONE m20-m23 ships; m30-m33 dispatch+verify ships; m40-m42 substrate feedback ships.

| Worktree | Branch | Agent | Modules | LOC budget | Test budget | Time budget |
|---|---|---|---|---|---|---|
| `wt-l6-keystone` | `wt/l6/wave3-{ts}` | Command-2 (PrefixSpan author) + Command-3 (variant builder + K-means) + rust-pro subagent (proposer) | m20, m21, m22, m23 | ~850 | 280 | Day 12-17 (LONGEST — KEYSTONE) |
| `wt-l7-dispatch` | `wt/l7/wave3-{ts}` | Command-3 (foreground, librarian-lane) + rust-pro subagent (m33 4-agent gate scaffolding) | m30, m31, m32, m33 | ~950 | 290 | Day 14-19 (overlaps wt-l6) |
| `wt-l8-feedback` | `wt/l8/wave3-{ts}` | Command-2 + rust-pro subagent (outbox-first JSONL) | m40, m41, m42 | ~450 | 180 | Day 17-21 |

**Wave-3 merge gate (Day 21):**
- 5-integration-track smoke (m32 dispatch → m40/m41/m42 emit → SYNTHEX/LCM/POVM observable)
- CC-5 substrate learning loop FIRST CLOSURE measured (Watcher Class-I clears)
- Verify-sync invariants 14-20 PASS
- ≥1,594 tests passing (cumulative — meets budget)
- m20 mutation kill-rate ≥70%
- Command tags `wave-3-complete` → `phase-2-complete`

---

## Isolation invariants (per worktree-mastery skill)

### Resource sharing table

| Resource | Shared? | Reason |
|---|---|---|
| `Cargo.lock` | YES — symlink to main | dep version consistency |
| `target/` | NO — per-worktree | concurrent compilation; symlink causes phantom rebuilds (AP-V7-07) |
| `node_modules/` | N/A | Rust only |
| `.cargo/config.toml` | YES — symlink to main | build profile consistency |
| stcortex namespace | YES — `workflow_trace_*` | shared substrate; coordination via m9 namespace guard |
| Test fixtures `tests/fixtures/` | YES — symlink to main | shared captured payloads |
| `.bacon-locations` | NO — per-worktree | bacon-ls export per editor pane |

### Worktree creation discipline (post-G9 only)

```bash
# Wave 1 worktree creation (Luke @ terminal post-G9):
cd ~/claude-code-workspace/workflow-trace/  # G2-renamed
for layer in l0-core l4-trust l1-ingest; do
  branch="wt/${layer%-*}/wave1-$(date -u +%Y%m%dT%H%MZ)"
  git worktree add "../wt-${layer}" -b "${branch}"
done

# Per-worktree shell init:
cd ../wt-l0-core
# .cargo/config.toml symlink
ln -sf ../workflow-trace/.cargo .cargo
# Cargo.lock symlink (only after wave 1 first stabilises)
ln -sf ../workflow-trace/Cargo.lock Cargo.lock
# target/ stays per-worktree (NO symlink)
export CARGO_TARGET_DIR=./target
```

### Lock discipline

Each worktree holds a `.atuin-worktree-lock` file containing:
- worktree branch name
- agent identifier (Command-2 / Command-3 / rust-pro)
- creation timestamp UTC
- expected completion timestamp (Wave time budget)

```bash
# Per worktree at activation:
echo "branch=$(git branch --show-current)" > .atuin-worktree-lock
echo "agent=Command-2" >> .atuin-worktree-lock
echo "created=$(date -u +%Y-%m-%dT%H%M%SZ)" >> .atuin-worktree-lock
echo "wave_budget_end=$(date -u -d '+3 days' +%Y-%m-%dT%H%M%SZ)" >> .atuin-worktree-lock
```

Supervisor (Command) re-reads every 30s; flags stale locks (>budget overflow).

### Merge protocol (per-Wave-end)

```bash
# Wave-end (Command runs):
cd ~/claude-code-workspace/workflow-trace/

# 1. Pull each worktree's branch
for wt in wt-l0-core wt-l4-trust wt-l1-ingest; do
  branch=$(cd "../$wt" && git branch --show-current)
  git fetch "../$wt" "$branch":"refs/heads/integrate/$branch"
done

# 2. Create integration branch
git checkout -b "wave-1-integration-$(date -u +%Y%m%dT%H%MZ)"

# 3. Merge each (rebase preferred for linear history)
git merge --no-ff "integrate/wt/l0/wave1-..." -m "merge L0 wave 1"
git merge --no-ff "integrate/wt/l4/wave1-..." -m "merge L4 wave 1"
git merge --no-ff "integrate/wt/l1/wave1-..." -m "merge L1 wave 1"

# 4. Run 4-stage QG on integrated branch
./scripts/gate.sh  # see GOD_TIER_RUST.md

# 5. Run verify-sync (invariants 1-7 for Wave 1)
./scripts/verify-sync.sh --invariants 1-7

# 6. If both green → merge to main
git checkout main
git merge --no-ff "wave-1-integration-..." -m "Wave 1 complete: L0 + L4 + L1 — 430 tests passing"
git tag -a "wave-1-complete" -m "Wave 1 close"

# 7. Cleanup worktrees
git worktree remove ../wt-l0-core
git worktree remove ../wt-l4-trust
git worktree remove ../wt-l1-ingest
git branch -d wt/l0/wave1-... wt/l4/wave1-... wt/l1/wave1-...

# 8. Push both remotes
git push origin main --tags
git push gitlab main --tags
```

### Cleanup discipline

After Wave-end merge SHA verified on `main`:
1. `git worktree remove` per layer
2. `git branch -d` per layer branch
3. `git worktree prune` cleanup
4. atuin trajectory entry: `wave-{N}-complete`

---

## Agent View allocation (per layer, per role)

### Per-layer agent stack

| Layer | Foreground agent | Background agents (parallel) | Tool list |
|---|---|---|---|
| L0 core | Command-2 | (none) | full Rust toolchain; rust-analyzer |
| L1 ingest | Command-2 | rust-pro (per-module tests) | + sqlx for injection.db; atuin CLI for m1 |
| L2 observe | Command-2 | rust-pro × 3 (one per module) | + criterion for m4 PrefixSpan |
| L3 central | Command-2 | rust-pro × 3 | + stcortex CLI for m13 |
| L4 trust | Command-2 | rust-pro × 2 (m11 math, m10 Ember) | + Cargo build-script tooling for m8 |
| L5 evidence | Command-2 | rust-pro × 2 | + proptest for m14 Wilson CI |
| L6 KEYSTONE | Command-2 (PrefixSpan) + Command-3 (variant builder) | rust-pro × 4 (one per module) | + proptest + cargo-mutants |
| L7 dispatch | Command-3 (librarian-lane) | rust-pro × 4 | + security-auditor for m32 EscapeSurfaceProfile |
| L8 feedback | Command-2 | rust-pro × 3 | + reqwest + serde for outbox JSONL |

### Persistent agents (across all waves)

| Agent | Role | Cadence | Output |
|---|---|---|---|
| **Watcher ☤** | observer | continuous tick-based | per-tick journal entries; Class A-I flags |
| **Zen** | audit | per-Wave-end | APPROVE/REFUSE/AMEND on integrated branch |
| **stcortex-reviewer** | anchor verification | per-Wave-end | confirms `workflow_trace_*` namespace anchors landed (G8+) |
| **obsidian-vault-librarian** | vault sweep | per-Wave-end | bidi anchor audit + MOC update |
| **security-auditor** | security continuous | Phase 7 every phase | per-domain audit per Phase 7 runbook |
| **performance-engineer** | observability + perf | Phase 8 every phase | Pushgateway metrics audit + flamegraph if hot path identified |
| **silent-failure-hunter** | Phase 4 | Phase 4 only | F1-F11 failure-mode hunt + AP-Drift sweep |

### Subagent dispatch pattern (Battern-coordinated)

```bash
# At Wave-N start, Command files Battern Wave plan:
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_wave_${N}_plan.md <<EOF
WAVE ${N} PLAN
Worktrees: <list>
Per-worktree foreground agent + module list
Per-module rust-pro subagent assignment (parallel dispatch via Battern Step 2)
Time budget: <days>
Gate: per-worktree 4-stage QG + integration smoke + verify-sync invariants
Wave-end Zen audit
EOF
```

Command-2 and Command-3 then dispatch their own rust-pro subagents in their respective panes (Agent tool, isolation worktree). Each subagent gets:
- worktree path
- module spec (from MODULE_PLANS/cluster-N.md)
- test budget
- antipattern list (relevant subset from ANTIPATTERNS_REGISTER.md)
- 4-stage QG enforcement requirement

---

## Concurrency invariants

| Invariant | Enforcement |
|---|---|
| At most ONE foreground agent per worktree | Lock file + supervisor 30s re-check |
| Background subagents may run in parallel WITHIN a worktree | per-module file ownership; no shared file edits |
| Background subagents may NOT run in parallel ACROSS worktrees in the same Wave | atomic Wave dispatch |
| Worktrees of the SAME wave may build in parallel | per-worktree `target/`; no shared cargo cache |
| Wave-N+1 worktrees may NOT be created until Wave-N merge SHA on main | Wave-end gate enforces |

---

## Failure-mode register

| ID | Failure | Detection | Mitigation |
|---|---|---|---|
| WT-01 | Two agents in same worktree | conflicting edits in `git status` | lock file + supervisor; abort second agent on collision |
| WT-02 | `target/` symlink causes phantom rebuilds | atuin shows ln -s ../wt-X/target | per-worktree `target/` rule (AP-V7-07) |
| WT-03 | Wave-N+1 starts before Wave-N merged | branch tree shows non-linear history | hard refusal at Wave dispatch — verify merge SHA first |
| WT-04 | Background subagent edits file owned by sibling | merge conflict at module facade | per-module file ownership in dispatch plan |
| WT-05 | Lock file stale (agent crashed without releasing) | supervisor sees lock older than wave_budget | manual cleanup + re-dispatch; root cause investigation |
| WT-06 | Stale `Cargo.lock` symlink (main updated mid-Wave) | build error in worktree | re-symlink at Wave-start; do NOT update mid-Wave |

---

## Atuin trajectory (provenance)

```bash
# Per-Wave atuin queries:
atuin search "git worktree add" --before 1d
atuin search "wave-.-complete" --before 30d
atuin search "verify-sync" --before 1d

# Per-day worktree health:
atuin stats --period day | grep -E "(worktree|cargo)"
```

---

## V7 self-applicability

**V7 is planning-only.** No worktrees during V7 authoring. This document describes the **post-G9 build** worktree topology. V7's own author-wave for runbooks + cluster plans + integrations uses **subagents in main worktree** (Agent tool with `isolation: "worktree"` parameter where appropriate, foreground/background per Command's dispatch).

---

*AGENT_VIEW_GITWORKTREES authored 2026-05-17 by Command. 9 worktrees × 3 waves × isolation invariants × failure-mode register operational post-G9.*
