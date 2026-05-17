---
title: Atuin Integration Deep-Dive — workflow-trace V7
date: 2026-05-17 (S1001982)
kind: planning-only · integration deep-dive · expands G5 § Atuin
parent: GENERATIONS/G5-tooling.md
owner: Command (per-phase invocation matrix); subagents file via their own atuin sessions
provenance: atuin is THE only cross-tool ledger per S1002029 learning #4
---

# Atuin Integration — workflow-trace V7

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../GENERATIONS/G5-tooling.md]] · [[../ULTRAMAP.md]]
>
> Siblings: [[scaffold-integration.md]] · [[devops-v3-integration.md]] · [[codesynthor-v8-integration.md]] · [[json-claude-code-optimisation.md]] · [[progressive-disclosure-obsidian.md]]

---

## Overview

Atuin is the workflow-trace ecosystem's **proprioception layer + cross-tool provenance ledger** (per G4 Axis 3 reasoning: CLI-first decision specifically because atuin already provides what an HTTP service would provide for free, and per S1002029 learning #4 cited in G5-tooling.md GAP-Tool-02 closure). Three planes of value: (1) **command-history audit** — every workflow-trace shell invocation persists with cwd + exit code + duration in `~/.local/share/atuin/history.db`, giving a single ledger that cross-correlates with `/scaffold`'s SQLite, V3's deploy DB, V8's confidence DB, and Obsidian's vault mtimes by timestamp; (2) **script registry** — 7 new `wt-*` scripts proposed below complement ~40 existing `habitat-*` / `hab-*` / `cc-*` workflow primitives (per the live `atuin scripts list` taken during G5 authoring); (3) **KV store** — already used habitat-wide for cross-pane fleet state (`atuin kv get --namespace habitat habitat.last_session` per the project CLAUDE.local.md Resume protocol), extended by workflow-trace for Wave + worktree lock + gate-status coordination across Tab 1 Command / Command-2 / Command-3 panes. Activation timing: Phase 0 session-start (`habitat-bootstrap`) onward through D120 sunset; the `wt-*` scripts are authored post-G9 only.

---

## 7 proposed `wt-*` workflow-trace atuin scripts

Each script is authored post-G9 via the `crystallize` skill (per G5 § G5 close: "will be authored as separate `crystallize` task post-G9"). Specs below; implementation deferred.

### 1. `wt-gate-status` — G1-G9 + Wave + worktree dashboard
- **Purpose:** single-call snapshot of G-gate state (G1 green / G2 pending / …), current Wave SHA on `main`, active worktrees with their lock files
- **Invocation:** `atuin scripts run wt-gate-status`
- **Output (~12 lines stdout):**
  ```text
  Gates: G1✅ G2✅ G3✅ G4✅ G5⏳ G6⬜ G7⬜ G8⬜ G9⬜
  Wave on main: wave-1-complete (sha 4f3a…)
  Active worktrees:
    wt-l0-core   branch=wt/l0/wave1-20260615T0900Z  agent=Command-2 created=2h ago budget=Day1
    wt-l4-trust  branch=wt/l4/wave1-20260615T0900Z  agent=Command-2 created=2h ago budget=Day2
  Pending: G5 (per Wave-1 merge SHA verified, awaiting G6 author)
  ```
- **Phase:** continuous (Phase 0 onward)
- **Implementation:** wraps `git log --decorate --oneline -1`, `git worktree list`, `cat .atuin-worktree-lock`, parses local gate-state file at `the-workflow-engine/.gate-state.json` (V7-defined; updated by per-gate authors)

### 2. `wt-soak-pulse` — Phase 5C lift signal probe
- **Purpose:** 30s-interval snapshot of m14 habitat_outcome_lift (mean + n + Wilson CI lower-bound) for soak monitoring
- **Invocation:** `atuin scripts run wt-soak-pulse` (typically launched in long-running loop via Monitor tool or zellij background pane)
- **Output (per-pulse line, append-only to `/tmp/wt-soak-pulse-{startTS}.tsv`):**
  ```text
  2026-06-15T09:00:00Z  m14_lift_mean=0.073  n=47  wilson_lower=0.041  decay_floor=0.018
  2026-06-15T09:00:30Z  m14_lift_mean=0.075  n=49  wilson_lower=0.043  decay_floor=0.018
  ```
- **Phase:** Phase 5C only (Day 30 → D120)
- **Implementation:** queries `~/.local/share/workflow-trace/db.sqlite` via `wf-crystallise stats lift --json`; computes Wilson CI per KEYWORDS_20 #8 (z=1.96 — not Wald per F2 hard gate)

### 3. `wt-substrate-pulse` — habitat substrate health one-liner
- **Purpose:** LTP/LTD ratio + RALPH gen + fitness + per-class Watcher flag counts; mirrors `habitat-intel` shape but extended for the 9 Watcher classes (A-I per KEYWORDS_20 #20)
- **Invocation:** `atuin scripts run wt-substrate-pulse`
- **Output:**
  ```text
  LTP/LTD=0.043 (Phase1 substrate_LTP_density=0.018 PASSING)
  RALPH gen=7,622 fitness=0.6987
  Watcher flags 5m EMA: A=0 B=0 C=0 D=0 E=0 F=0 G=0 H=0 I=4
  Bridges UP: 6/7 (SX :8090 retired)
  ```
- **Phase:** continuous (Phase 0 onward; particularly during Phase 5C soak)
- **Implementation:** parallel curls to `:8090/api/health` + `:8125/health` + `:8092/v3/health` + watcher journal grep + ratio calc with accumulation guard (per existing `habitat-ratio-tracker` script pattern)

### 4. `wt-bridge-check` — 5-substrate-peer probe
- **Purpose:** per-Wave-end + post-deploy probe of the 5 substrate peers workflow-trace touches (V8 :8111, V3 :8082, SYNTHEX v2 :8092, LCM RPC, POVM :8125)
- **Invocation:** `atuin scripts run wt-bridge-check`
- **Output:**
  ```text
  V8       :8111  200 OK   (CodeSynthor Elixir OTP — Holy Trinity)
  V3       :8082  200 OK   (DevOps Engine V3 — T1-T6)
  SYNTHEX  :8092  200 OK   (v2 — m40 NexusEvent target)
  LCM      RPC    200 OK   (lcm.health — m41 router target)
  POVM     :8125  200 OK   (deprecated 2026-07-10 — m42 dual-path overlap)
  All bridges UP — workflow-trace can ship.
  ```
- **Phase:** per-Wave-end (Wave 1, 2, 3) + Phase 4 hardening + Phase 5A pre-deploy + Phase 5C weekly
- **Implementation:** extends existing `habitat-bridge-check` script with the 5 specific peers; uses CARGO_TARGET_DIR-free probes (curl only)

### 5. `wt-wave-status` — Wave isolation health
- **Purpose:** per-Wave snapshot of active worktrees, their lock files, time-budget consumption, and any stale-lock violations (per AGENT_VIEW_GITWORKTREES.md WT-05)
- **Invocation:** `atuin scripts run wt-wave-status`
- **Output:**
  ```text
  Wave 3 active. 3 worktrees:
    wt-l6-keystone  branch=wt/l6/wave3-20260620T1100Z  agent=Command-2  age=14h  budget_used=23%  STATUS=ACTIVE
    wt-l7-dispatch  branch=wt/l7/wave3-20260621T0900Z  agent=Command-3  age=2h   budget_used=4%   STATUS=ACTIVE
    wt-l8-feedback  branch=wt/l8/wave3-20260620T1500Z  agent=Command-2  age=10h  budget_used=15%  STATUS=ACTIVE
  Stale locks: 0
  ```
- **Phase:** Wave 1, 2, 3 build (Days 0-21)
- **Implementation:** parses each worktree's `.atuin-worktree-lock`; cross-checks against `git worktree list`; budget computed against wave_budget_end timestamp

### 6. `wt-cc5-trace` — substrate learning loop closure trace
- **Purpose:** trace CC-5 first closure (m32 dispatch → m40 emit → SYNTHEX → stcortex pathway weight delta) — the key Phase 3 Day 26 verification per AGENT_VIEW_GITWORKTREES.md Wave-3 merge gate "CC-5 substrate learning loop FIRST CLOSURE measured"
- **Invocation:** `atuin scripts run wt-cc5-trace <workflow_id>`
- **Output:**
  ```text
  Tracing CC-5 closure for workflow_id=wf_a3f29e…
    m32 dispatch_at:       2026-06-22T14:03:11Z  outcome=PassVerified
    m40 emit_at:           2026-06-22T14:03:11.412Z  outbox=wf_a3f29e_event_001.json
    SYNTHEX ingest_at:     2026-06-22T14:03:11.847Z  nexus_event_id=ne_8f3d…
    stcortex pathway pre_weight=0.612  post_weight=0.687  delta=+0.075  band=LTP
  CC-5 CLOSED in 0.836s — Class-I Watcher flag should clear within tick·3
  ```
- **Phase:** Phase 3 Day 26 (first closure measure); Phase 5C weekly verification
- **Implementation:** chained SQL across SQLite `workflow-trace/db.sqlite` (m32 outcome), JSONL outbox tail (m40), HTTP GET `:8092/v3/nexus/events/<id>` (SYNTHEX), HTTP GET `:3000/pathways/<pre>/<post>` (stcortex)

### 7. `wt-keystone-bench` — m20 PrefixSpan benchmark delta
- **Purpose:** capture criterion benchmark for m20 PrefixSpan across generations; delta vs prior captures (m20 is KEYSTONE per KEYWORDS_20 #13; performance regressions here cascade through CC-3/CC-4/CC-5)
- **Invocation:** `atuin scripts run wt-keystone-bench`
- **Output:**
  ```text
  Bench captured 2026-06-22T15:00Z (commit 4f3a…)
    prefixspan_small_seq:  43.2µs  (Δ vs prior -1.7% — improvement)
    prefixspan_medium:     1.42ms  (Δ vs prior +0.3% — within noise)
    prefixspan_large_gap5: 18.7ms  (Δ vs prior +2.1% — REGRESSION watch)
  Stored: /tmp/wt-keystone-bench-20260622T1500Z.json
  ```
- **Phase:** post-Wave-3 + Phase 5C weekly
- **Implementation:** `cargo bench -p workflow-trace --bench m20_prefixspan -- --save-baseline current`; diff against prior `target/criterion/baseline/`

---

## Existing atuin scripts reused (full citations)

Per G5 § Atuin integration "Existing reuse" line. Each script preserves its current function; workflow-trace runbooks reference by name:

| Script | Tags | Workflow-trace use |
|---|---|---|
| **`habitat-intel`** | habitat, monitoring | 17ms baseline pulse before each Wave-end; cited in runbook-02/03/04 |
| **`habitat-sweep`** | habitat, health | 11-service health sweep before/after deploy (Phase 5A); per runbook-05 |
| **`habitat-bridge-check`** | habitat, bridges, health | superseded by `wt-bridge-check` for workflow-trace-specific 5-peer subset, but kept as universal fallback |
| **`hab-qg`** | habitat, quality | Wave-end multi-repo QG (ORAC + ME); cited in `scripts/wave-end-checklist.sh` per G4 |
| **`habitat-evolution-delta`** | habitat, evolution, ralph, delta | RALPH gen + fitness delta between Wave snapshots; pre-positioned at Phase 5C weekly retrospective |
| **`habitat-metabolic`** | habitat, composite | composite ME × ORAC × PV2 health pre-deploy gate per Phase 4 |
| **`habitat-fingerprint`** | habitat, diagnostics | per-pulse fingerprint to detect substrate drift during long-running Phase 5C soak |
| **`habitat-12d-burst`** | habitat, ultimate | deep-diagnostic invocation when `wt-substrate-pulse` flags Class-I sustained pause |
| **`memory-search`** | memory, federation | unified RM + POVM + ORAC blackboard search during Phase 2A/2B authoring (boilerplate-lift research) |
| **`hab-investigate`** | habitat, debug | root-cause invocation when Wave-end checklist fails on any of 8 sources |

**Provenance lift:** these 10 scripts (of the ~40 in the live registry) are explicitly named in the V7 runbooks; the remaining ~30 are reachable as situational tools without being scripted in.

---

## Atuin KV store usage (cross-pane state)

Per the project CLAUDE.local.md Resume protocol (`atuin kv get --namespace habitat habitat.last_session`) — the same pattern extends for workflow-trace. KV namespace: `wt` (avoids collision with `habitat`).

| Key | Owner (writer) | Reader | Purpose |
|---|---|---|---|
| `wt.gate.G1` … `wt.gate.G9` | per-gate author (Command / Watcher / Zen / Luke) | all panes | gate-state JSON (`{state: "green"|"pending", verified_at, verifier}`) |
| `wt.wave.current` | Command at Wave start | all panes | current Wave number + worktree branches |
| `wt.wave.{N}.merge_sha` | Command at Wave-end | all panes | merge SHA into main after Wave N |
| `wt.worktree.{name}.lock` | foreground agent at worktree open | supervisor (Command) | agent + ts + budget triple |
| `wt.cc5.first_closure_at` | m32 dispatcher on first PassVerified | Watcher | UTC timestamp of CC-5 first closure (clears Class-I) |
| `wt.last_session` | save-session hook | next session | mirrors `habitat.last_session` but project-scoped |
| `wt.spec.version` | Zen on G7 audit | all panes | current locked spec version (v1.2 → v1.3 …) |
| `wt.deploy.last_binary_sha` | Phase 5A binary cp | health monitor | binary SHA placed at `~/.local/bin/wf-*` |

**Anti-pattern caught:** project CLAUDE.local.md "Receive-mode v2 standing" depends on writing-side never silently overwriting. KV namespace `wt` is **append-with-overwrite**; readers MUST timestamp-compare, not last-write-wins (per the CLAUDE.local.md `feedback_verify_dispatch_landed.md` discipline).

---

## Provenance principle (S1002029 #4)

**atuin is the ONLY cross-tool ledger.** Per G5 § Atuin integration provenance line + G5 § G5 substrate-frame pass:

- `/scaffold` writes to `~/.local/share/scaffold/history.db` (if any) — local-only
- DevOps Engine V3 writes to its own deploy DB at `:8082` — local-to-V3
- CodeSynthor V8 writes to its own confidence DB at `:8111` — local-to-V8
- Obsidian vault writes manifest as file mtimes — local-to-FS
- workflow-trace writes to `~/.local/share/workflow-trace/db.sqlite` — local-to-workflow-trace

**None of these cross-correlate by content.** They DO cross-correlate by timestamp **only if** every invocation lands in atuin first. Therefore: **every workflow-trace shell command MUST run in an atuin-active shell**, and every internal tool dispatch SHOULD use the `atuin scripts run` wrapper (not direct binary invocation).

This is enforced by:
1. Project CLAUDE.md operational rule "All cross-pane comms via file-drop" — file-drop scripts are atuin-wrapped
2. AGENT_VIEW_GITWORKTREES.md § Atuin trajectory (provenance) — per-Wave queries
3. Phase 8 observability runbook (R-09 per ULTRAMAP View 3) — Pushgateway pattern annotated with atuin session IDs
4. Per-Wave-end audit: `atuin search "workflow-trace OR wt-*" --before 7d` confirms zero blind invocations

---

## Per-phase atuin script invocation matrix

Per G5 § Tooling integration time matrix, the Atuin column expanded with script names:

| Phase | atuin invocations |
|---|---|
| Phase 0 (HOLD-v2) | session-start `habitat-bootstrap` + `stcortex-probe` (per workspace CLAUDE.md) + `habitat-intel` (baseline) |
| Phase 1 Genesis Day 0 | session-start + `wt-gate-status` (post-G9 verify) + `wt-substrate-pulse` baseline + initial `/scaffold` (logged) |
| Phase 1 Day 1-3 | `wt-gate-status` per-day + `wt-wave-status` |
| Phase 2A Days 3-12 | `wt-wave-status` 2× daily + `wt-substrate-pulse` per-day + `hab-qg` per-module-merge |
| Phase 2B Days 12-21 | `wt-wave-status` + `wt-keystone-bench` post-m20 build + `wt-substrate-pulse` |
| Phase 3 Days 21-26 | `wt-cc5-trace <id>` at first closure (Day 26 KEY EVENT) + `wt-bridge-check` per-Track-end |
| Phase 4 Days 26-28 | `wt-bridge-check` + `habitat-metabolic` pre-deploy gate + `hab-investigate` if 4-agent flags REJECT |
| Phase 5A Days 28-30 | `wt-substrate-pulse` 2× daily + `habitat-sweep` pre-deploy + post-deploy |
| Phase 5B Day 30 cutover | `wt-substrate-pulse` 4× during cutover + `wt-bridge-check` post-cutover |
| Phase 5C Day 30→D120 | `wt-soak-pulse` continuous (30s loop) + `wt-substrate-pulse` weekly + `wt-cc5-trace` weekly + `wt-keystone-bench` weekly + `habitat-evolution-delta` weekly |
| Phase 6 Sunset D120 | full `wt-*` matrix one final invocation + `habitat-fingerprint` final + retrospective `atuin search` against full history |

---

## Verification commands

```bash
# Confirm all proposed wt-* scripts exist (post-G9)
atuin scripts list | grep -E '^- wt-'             # should list 7 (or N green) post-author

# Confirm habitat- scripts named in V7 runbooks still exist
for s in habitat-intel habitat-sweep habitat-bridge-check hab-qg habitat-evolution-delta \
         habitat-metabolic habitat-fingerprint habitat-12d-burst memory-search hab-investigate; do
  atuin scripts list | grep -q "^- $s " && echo "$s ✅" || echo "$s ❌ MISSING"
done

# KV namespace audit
atuin kv list --namespace wt 2>/dev/null | wc -l     # should be ≥ 8 once Wave 1 starts

# Provenance audit per Wave-end
atuin search "workflow-trace OR wt-" --before 7d | wc -l    # gives weekly invocation count
atuin search --cwd "$HOME/claude-code-workspace/workflow-trace" --before 7d | head -50

# Cross-tool correlation by timestamp (the substrate-frame proof)
ts=$(atuin search "wf-crystallise propose accept" --before 1d --limit 1 --format json | jq -r '.[0].timestamp')
# Then look up V3 deploy row with timestamp ≈ ts (within ± 1s)
# And V8 confidence delta row at same timestamp
# This is the proof that atuin IS the cross-tool ledger
```

---

## Failure modes

| ID | Failure | Detection | Mitigation |
|---|---|---|---|
| AT-01 | Luke runs `cargo build` outside atuin-active shell (Class-H proprioception anomaly per G5 § Watcher pre-positioning) | per-Wave-end `atuin search` returns fewer rows than `cargo build` log expects | every runbook prefixed with explicit atuin context note; PreToolUse hook (per [[json-claude-code-optimisation.md]]) refuses non-atuin Bash on critical commands |
| AT-02 | KV race condition (two panes write `wt.gate.G5` simultaneously) | last-write-wins silently drops one update | timestamp-compare on read; writer pattern uses `atuin kv set --if-empty` semantics (or local file-lock with kv mirror) |
| AT-03 | Stale lock file in worktree (agent crashed without releasing) | `wt-wave-status` reports lock older than budget | manual cleanup per AGENT_VIEW_GITWORKTREES.md WT-05; root-cause investigation |
| AT-04 | atuin history DB corruption (S100-class) | `atuin search` returns garbage / empty | restore from `~/.local/share/atuin/history.db.bak` snapshot; freeze cross-tool correlation until restored |
| AT-05 | `wt-soak-pulse` 30s loop drifts (system load) | tsv interval >35s | acceptable — Wilson CI absorbs jitter; flag only if interval >60s |

---

## Sign-off

✅ atuin-integration spec complete. 7 `wt-*` scripts proposed with purpose/invocation/output/phase. 10 existing scripts reused with explicit per-runbook citation. KV namespace `wt` defined with 8 cross-pane keys + receiver discipline. Provenance principle reasserted (S1002029 #4). Per-phase invocation matrix complete (Phase 0 through D120). 5 failure modes with mitigations. Verification commands deterministic.

*Authored 2026-05-17 (S1001982) — Command for V7 G5 expansion. `wt-*` scripts deferred to post-G9 crystallize. HOLD-v2 respected: planning-only.*
