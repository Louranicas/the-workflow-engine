# workflow-trace — Deployment Guide

> **Back to:** [`README.md`](../README.md) · [`CLAUDE.md`](../CLAUDE.md) · [`CLAUDE.local.md`](../CLAUDE.local.md) · [`ai_docs/INDEX.md`](INDEX.md) · canonical (66,576 words) [`ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md`](../the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md) · [`GENESIS_PROMPT_V1_3.md`](GENESIS_PROMPT_V1_3.md) · [`ai_docs/optimisation-v7/RUNBOOKS/`](optimisation-v7/RUNBOOKS/)
>
> **Function:** Navigable runbook condensation of the canonical 10-phase, 66,576-word `ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md`. This file IS the deploy spine — refer to phase-specific docs in `the-workflow-engine-vault/deployment framework/phase-{0..8}-*.md` for full detail. Status: planning-only · HOLD-v2 active · G9 not fired.

---

## 0. Status & gating preface

This is the **recipe**, not the execution. Execution is gated on:

1. **G1-G8 GREEN** (currently NOT GREEN — see [`CLAUDE.local.md`](../CLAUDE.local.md) Pending Luke decisions B1-B6)
2. **G9 fire** — Luke explicit signal `start coding workflow-trace`. The 2026-05-17T08:43Z arrival is held as queued intent only (Zen URGENT block until B1 resolved)
3. **v1.3 patch ratification** — Genesis v1.3 awaiting Zen G7 re-audit verdict

Until then: **no `cargo init`, no `cargo new`, no `Cargo.toml`, no `src/*.rs`**. All deploy-shaped commands below are reference text, NOT live invocations.

---

## 1. Phase map (Day 0 → D120+)

| Days | Phase | Owner | Output | Phase doc |
|---|---|---|---|---|
| pre-G9 | Phase 0 — G1-G9 gates | Watcher / Command / Luke / Zen | 9 gates GREEN + G9 signal | [`phase-0-pre-genesis-gates`](../the-workflow-engine-vault/deployment%20framework/phase-0-pre-genesis-gates.md) |
| 0-3 | Phase 1 — Genesis | Command + Command-2 | cargo crate + Cluster D + Cluster A + ~600 LOC | [`phase-1-genesis-day-0-3`](../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md) |
| 3-12 | Phase 2A — B/C/E (measure-only) | Command-2 | 8 modules m4-m7 + m12-m15 (~870 LOC) | [`phase-2A-build-clusters-B-C-E`](../the-workflow-engine-vault/deployment%20framework/phase-2A-build-clusters-B-C-E.md) |
| 12-21 | Phase 2B — F/G/H (KEYSTONE + active) | Command-2 + Command-3 | 11 modules m20-m23 + m30-m33 + m40-m42 (~2,200 LOC); 3 structural gaps | [`phase-2B-build-clusters-F-G-H`](../the-workflow-engine-vault/deployment%20framework/phase-2B-build-clusters-F-G-H.md) |
| 21-26 | Phase 3 — Integration | Command + Command-3 + Zen | 5 tracks; CC-5 FIRST CLOSURE | [`phase-3-integration-conductor-wiring`](../the-workflow-engine-vault/deployment%20framework/phase-3-integration-conductor-wiring.md) |
| 26-28 | Phase 4 — Pre-deploy hardening | Zen + 4-agent gate | PASS/FAIL/DEGRADED verdict | [`phase-4-pre-deploy-hardening`](../the-workflow-engine-vault/deployment%20framework/phase-4-pre-deploy-hardening.md) |
| 28-30 | Phase 5A — Binary deploy | Command + Luke @ terminal | `~/.local/bin/wf-{crystallise,dispatch}` | [`phase-5-deploy-and-soak`](../the-workflow-engine-vault/deployment%20framework/phase-5-deploy-and-soak.md) |
| 30 | Phase 5B — Cutover ceremony | Command + Watcher | WCP carriage handoff | [`phase-5`](../the-workflow-engine-vault/deployment%20framework/phase-5-deploy-and-soak.md) |
| 30→D120 | Phase 5C — 120-day soak | Watcher + system | continuous lift + decay + weekly synthesis | [`phase-5`](../the-workflow-engine-vault/deployment%20framework/phase-5-deploy-and-soak.md) |
| D120 | Phase 6 — Sunset evaluation | m11 + Luke | PASS continue / FAIL retire / DEGRADED | [`phase-6-sunset-and-cross-cutting`](../the-workflow-engine-vault/deployment%20framework/phase-6-sunset-and-cross-cutting.md) |
| continuous | Phase 7 — Security + compliance | security-auditor | 7 domains × 7 phases | [`phase-7-security-compliance`](../the-workflow-engine-vault/deployment%20framework/phase-7-security-compliance.md) |
| continuous | Phase 8 — Observability + ops | observability-engineer | 5 tracks | [`phase-8-observability-operations`](../the-workflow-engine-vault/deployment%20framework/phase-8-observability-operations.md) |

---

## 2. Phase 0 — Pre-Genesis Gates G1-G9 (PRE-DEPLOY HARD GATE)

The 9 gates that must clear before any code can be written. See [`phase-0-pre-genesis-gates`](../the-workflow-engine-vault/deployment%20framework/phase-0-pre-genesis-gates.md) for full criteria.

| Gate | Owner | Resolution path | Status |
|---|---|---|---|
| G1 | Watcher | Close-notice issued via `~/.local/bin/watcher notify` | NOT GREEN |
| G2 | Command | Directory rename `the-workflow-engine/` → `workflow-trace/` (gated on G1) | NOT GREEN |
| G3 | Luke @ terminal | ~~`devenv restart povm-engine` + verify `learning_health ∈ [0.05, 0.15]`~~ — **DROPPED per m42 stcortex-only ADR**; G3 now reduces to stcortex `:3000` reach + reducer registry check | DROPPED → reduced |
| G4 | Watcher | Ember §5.1 Held-semantics amendment authored + Luke approves | NOT GREEN (B4) |
| G5 | Command | F2 spec interview signed off | NOT GREEN |
| G6 | Command | Dual-frame gap analysis (conventional + non-anthropocentric) authored | partial |
| G7 | Zen | Spec audit of v1.3 patch (amendment-only delta) — verdict PASS/AMEND/REFUSE | PENDING (B1, B2) |
| G8 | Command | All persistence surfaces ratified (stcortex + ai_docs + vault + CLAUDE.local + injection.db) | NOT GREEN |
| G9 | Luke @ node 0.A | Explicit signal `start coding workflow-trace` after G1-G8 GREEN | queued-intent only (Zen URGENT block) |

**Phase 0 exit criterion:** 9/9 GREEN + Luke re-issues G9.

---

## 3. Phase 1 — Genesis (Day 0-3)

### 3.1 Prerequisites (Luke @ terminal)

- Rust 1.83 toolchain (rust-version in `plan.toml`)
- `~/.local/bin/devenv` configured
- HABITAT-CONDUCTOR `:8141` reachable OR `auto_start=false` accepted (B3 blocker)
- stcortex `:3000` reachable (G3 reduced surface)

### 3.2 Bootstrap commands (post-G9 only)

```bash
cd /home/louranicas/claude-code-workspace/workflow-trace   # post-G2 rename
# Scaffold from plan.toml (single-crate, two-binary, in-crate lib):
scaffold-gen --from-plan plan.toml .
# CARGO_TARGET_DIR per-shell to avoid worktree contamination (AP-V7-07):
export CARGO_TARGET_DIR=./target
cargo check 2>&1 | tee /tmp/wf-day0-check.log
test "${PIPESTATUS[0]}" -eq 0 || { echo "ABORT — check failed"; exit 1; }
```

### 3.3 Cluster D ships Day 1 (BEFORE Cluster A)

Order-of-operations invariant per [`plan.toml`](../plan.toml) `[[layers]] L4 ship_first=true`:

```
Day 0-1: workflow_core lib + types + schemas + namespace + errors + m8 build.rs + m9 + m10 + m11 stubs
Day 1-2: Cluster A (m1, m2, m3) with m8 povm_calibrated already gated
Day 2-3: Day-0 commit SHA + first `cargo test --lib` GREEN (Class-E resolution)
```

m8 must install `cargo:rustc-cfg=povm_calibrated` BEFORE any module that touches POVM-derived signals can compile. This is **not negotiable** — it is CC-2's compile-time checkpoint.

---

## 4. Phase 2A — Build B/C/E (Day 3-12)

### 4.1 Module order

```
m7 hub schema FIRST (Day 3-4)   # central contract surface (CC-1 owner)
   ├─► m4 cascade_correlator (Day 4-6)   # F11 opaque IDs via FNV-1a XOR
   ├─► m5 battern_step_record (Day 4-6)  # step_label: Option preserves unlabelled
   ├─► m6 context_cost (Day 4-6)         # 20-session EMA, EXCLUDES Converged (F10)
   ├─► m12 cli_reports (Day 6-8)
   ├─► m13 stcortex_writer (Day 6-8)     # 3-band LTP/LTD gate
m14 habitat_outcome_lift (Day 8-10)      # Wilson CI not Wald; Option<Lift>::None at n<20
m15 pressure_register (Day 10-12)        # JSONL one-event-per-file
```

### 4.2 Quality gate (run AFTER EACH MODULE; per [`CLAUDE.md`](../CLAUDE.md))

```bash
CARGO_TARGET_DIR=./target cargo check 2>&1 | tail -20
test "${PIPESTATUS[0]}" -eq 0 || abort
CARGO_TARGET_DIR=./target cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -20
test "${PIPESTATUS[0]}" -eq 0 || abort
CARGO_TARGET_DIR=./target cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tail -20
test "${PIPESTATUS[0]}" -eq 0 || abort
CARGO_TARGET_DIR=./target cargo test --lib --release 2>&1 | tail -30
test "${PIPESTATUS[0]}" -eq 0 || abort
```

**TRAP — AP-Hab-05 PIPESTATUS swallow:** `cargo … | tail` captures `tail`'s exit (always 0). ALWAYS check `${PIPESTATUS[0]}` per stage. S1001882 near-miss: gate printed GREEN while clippy was screaming `doc_markdown` on un-backticked `tracing::error`.

---

## 5. Phase 2B — Build F/G/H (Day 12-21, KEYSTONE)

| Sub-wave | Modules | Lead | Notes |
|---|---|---|---|
| 2B-1 KEYSTONE | m20 PrefixSpan (split across 4 internal passes: skeleton → algorithm → Wilson-CI gate → variant selection), m21 variant_builder, m22 K-means feature (NOT PrefixSpan — feature vectors vs sequences), m23 proposer | Command-2 | Gap 1 owner; 250+ tests for m20 alone |
| 2B-2 Cluster G | m30 curated_bank (NEVER auto-promote — F5), m31 selector (composite α/β/γ/δ), m32 dispatcher (5-check), m33 verifier (4-agent · 7-day TTL) | Command-3 (librarian-lane) | Gap 3 EscapeSurfaceProfile; m32 refuse-mode is NOT panic NOT exit |
| 2B-3 Cluster H | m40 NexusEvent emit, m41 LCM RPC, m42 stcortex emit (POVM DECOUPLED per 2026-05-17 ADR) | Command-2 + rust-pro | outbox-first JSONL durability |

**630+ tests across 11 modules.** Worktree pattern: `wt-l6-keystone`, `wt-l7-dispatch`, `wt-l8-feedback` per [ULTRAMAP View 5](optimisation-v7/ULTRAMAP.md). Wave-end merge after Command rebases all wave worktrees to `main`.

---

## 6. Phase 3 — Integration (Day 21-26)

5 integration tracks (parallel where possible):

1. **stcortex track** — m2 reducer registry + m13 writer + m42 emit; verify `workflow_trace_*` namespace honoured (AP30); `bridge-contract` skill pre-merge
2. **Conductor track** — m32 dispatch contract; BLOCKED by B3 (`auto_start=false` for Waves 1B/1C/2/3) — Luke @ terminal `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start weaver/zen/enforcer`
3. **SYNTHEX v2 track** — m40 `/v3/nexus/push` outbox-first; reconnect-per-call; serde `rename = "type"` trap (most likely silent failure mode)
4. **LCM track** — m41 `lcm.loop.create {max_iters: 1}`; reconnect-per-call (no persistent UDS connection)
5. **CC-5 first closure** — Day 26 milestone — first measurable substrate delta `learning_health(post) > learning_health(pre)` after a known-good dispatch

**Critical** — CC-5 FIRST CLOSURE measured by:

```
1. Capture baseline stcortex pathway.weight for workflow_trace_test_pathway
2. wf-dispatch dispatch <known-good-workflow>
3. Assert DispatchOutcome::PassVerified
4. Sleep 2s (substrate propagation)
5. Re-read pathway.weight; assert post > pre
```

If this fails, Watcher Class-I fires (engine appears functional but substrate isn't moving — the most important silent failure mode).

---

## 7. Phase 4 — Pre-Deploy Hardening (Day 26-28)

4-agent parallel gate on the staged diff:

| Agent | Owns | Catches |
|---|---|---|
| silent-failure-hunter | F7 (POVM graceful-degrade pretend-fix — DECOUPLED post-ADR), F3 (substrate-input poisoning) | Result-default collapse, `unwrap_or(true)` masks |
| security-auditor | F8 (Watcher feedback-loop poisoning), F11 (cascade monoculture leak), AP-Hab-04 (preserve-list discipline) | secret leaks, blanket commands |
| performance-engineer | F2 (sample-size inflation), F10 (exploration-cost preservation collapse) | hot-path budget violations |
| zen | F1 (bank ossification), F9 (workflow-grain fitness distortion), BUG-035 class | spec-coherence audit |

**Wave-1 mechanical gate uses `${PIPESTATUS[0]}` per stage** (S1001882 discipline). APPROVE from all four required before `/usr/bin/cp -f` to `~/.local/bin/`.

---

## 8. Phase 5 — Deploy + 120-day Soak

### 8.1 Release build

```bash
CARGO_TARGET_DIR=./target cargo build --release --features full 2>&1 | tail -10
ls -la ./target/release/wf-crystallise ./target/release/wf-dispatch
```

### 8.2 Binary deployment (TRAP: cp alias)

**AP-Hab-06 `cp -f` alias trap** — `cp` aliased to `cp -i`. ALWAYS `/usr/bin/cp -f`:

```bash
/usr/bin/cp -f ./target/release/wf-crystallise ~/.local/bin/wf-crystallise
/usr/bin/cp -f ./target/release/wf-dispatch    ~/.local/bin/wf-dispatch
ls -la ~/.local/bin/wf-{crystallise,dispatch}
```

### 8.3 DevEnv registration (CLI-not-service)

workflow-trace is a **CLI tool**, not a long-running daemon. It does NOT register a port in `~/.config/devenv/devenv.toml`. (TBD port 8190 reserved in `plan.toml` for future telemetry endpoint; not used in Phase 5A.)

This **eliminates 4 of 8 forge traps** (no SIGPIPE, no health-path bikeshed, no port conflict, no batch ordering).

### 8.4 Cutover ceremony (Day 30)

- Command emits WCP carriage handoff notice via `~/.local/bin/watcher notify`
- Watcher takes over Phase 5C synthesis cadence (weekly)
- T+30d production observation begins

### 8.5 Phase 5C 120-day soak (Day 30 → D120)

Continuous observation:

- **m14 lift** — weekly rolling 7-day window; Watcher independent recompute (AP-Drift-07 mitigation)
- **m11 decay trajectory** — healthy band 0.97-0.99 at D60; warning band 0.30 at D60 (never-dispatched)
- **CC-5 `learning_health` delta** — Watcher Class-I primary detector; must move post-dispatch
- **POVM cutover (~D25 mid-soak)** — DROPPED per m42 stcortex-only ADR; no longer a soak risk surface

### 8.6 Probe verification (paired with `/health`)

**AP-V7-13 Health-200 ≠ behaviour-verified.** Never trust `/health` alone. Pair with a semantic probe:

```bash
# Day-of-deploy verify (workflow-trace has no daemon port; substitute semantic probe):
wf-crystallise observe --dry-run --json | jq '.schema_version, .git_sha'
wf-dispatch verify --self-test
```

(POVM-style example for context: `:8125/health` returned 200 + `service:povm_v2 v2.0.0` but `:8125/stats` showed `learning_health=0.9146` pre-CR-2 inflated. Source had CR-2 at `e2a8ed3`; live binary did not.)

---

## 9. Phase 6 — Sunset (D120)

m11 produces D120 sunset evaluation per workflow. Three outcomes:

| Outcome | Action |
|---|---|
| PASS continue | Workflow remains in bank with renewed `sunset_at` |
| FAIL retire | m13 emits stcortex delete-marker via m9 namespace guard; entry pruned from m30 |
| DEGRADED Luke-decide | m15 emits pressure event; Luke deliberation; spec amendment possibility |

**D120 immutability** — `sunset_at` encoded at deploy time, NOT recalculated at runtime. Runtime recalculation is how ancestors drifted.

**Bounded extension** — 60-day max × 2 cycles before formal spec amendment required.

---

## 10. Phases 7 + 8 (continuous, every phase)

### Phase 7 Security + compliance (7 domains × 7 phases matrix)

- W1 namespace discipline (AP30)
- AP30 stcortex namespace guard (m9)
- m10 Ember rubric CI gate
- Cipher EscapeSurfaceProfile (Gap 3)
- Secret scanning (AP-Drift-09) — `rg -n '(glpat-|github_pat_|gho_)' --hidden -g '!.git' .` pre-commit
- HMAC-SHA256 multi-user upgrade path (FNV-1a adequate single-user)
- Waiver 2 (Fossil scope discipline) → Zen G7 as non-negotiable compensating control

### Phase 8 Observability + ops (5 tracks)

- Structured logs (`tracing` + `tracing-subscriber`)
- Prometheus metrics via Pushgateway (CLI-first; binaries exit after work)
- OTel (fails open — never blocks dispatch)
- SLOs + alerts (Watcher class A-I = alert-routing taxonomy)
- Dashboards (Habitat Nerve Center :8083)

**Cardinality trap** — `wf_m31_selection_weight` cardinality is single highest Prometheus risk → FNV-1a u32 hashing + overflow bucket. `wf_m14_lift = -1.0` as sentinel (NOT zero) distinguishes Class-I from warm-up.

---

## 11. Rollback (3 commands per phase)

Per [`phase-5-deploy-and-soak`](../the-workflow-engine-vault/deployment%20framework/phase-5-deploy-and-soak.md) §Rollback:

```bash
/usr/bin/cp -f ~/.local/bin/wf-crystallise.prev ~/.local/bin/wf-crystallise
/usr/bin/cp -f ~/.local/bin/wf-dispatch.prev    ~/.local/bin/wf-dispatch
wf-dispatch verify --self-test   # confirm previous binary healthy
```

Worktree rollback: `git checkout main && git reset --hard <pre-phase-merge-SHA>`.

---

## 12. Open critical-path blockers at deploy-guide-author time

Per [`CLAUDE.local.md`](../CLAUDE.local.md):

- **B1** G7 Zen URGENT block on G9 out-of-sequence
- **B2** v1.3 patch awaiting Zen audit verdict
- **B3** Conductor Waves 1B/1C/2/3 `auto_start=false`
- **B4** Ember rubric §5.1 Held-semantics amendment pending
- ~~**B5** POVM `:8125` redeploy verify~~ — DROPPED per m42 stcortex-only ADR
- **B6** Power-structure ambiguity (Luke override vs Zen G7 audit precedence) — RESOLVED via D-B6 AMEND-loop

3 Luke physical actions remain (down from 4 pre-ADR). See [`watcher-notices/2026-05-17T160300Z_command_luke_action_needed_v2.md`](file:///home/louranicas/projects/shared-context/agent-cross-talk/2026-05-17T160300Z_command_luke_action_needed_v2.md).

---

## 13. Cross-references

- **Canonical full deployment framework (66,576 words):** [`ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md`](../the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md)
- **Per-phase docs:** [`the-workflow-engine-vault/deployment framework/phase-{0..8}-*.md`](../the-workflow-engine-vault/deployment%20framework/)
- **Runbooks:** [`ai_docs/optimisation-v7/RUNBOOKS/`](optimisation-v7/RUNBOOKS/) (R-00 through R-11)
- **Antipatterns referenced:** [`ANTIPATTERNS_REGISTER.md`](optimisation-v7/ANTIPATTERNS_REGISTER.md)
- **Cross-cluster synergies (CC-1..CC-7):** [`CROSS_CLUSTER_SYNERGIES.md`](optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md)
- **Performance budgets:** [`PERFORMANCE.md`](PERFORMANCE.md)
- **Onboarding (cold-start path):** [`ONBOARDING.md`](ONBOARDING.md)
- **5-minute quickstart:** [`QUICKSTART.md`](QUICKSTART.md)

> **Back to:** [`README.md`](../README.md) · [`CLAUDE.md`](../CLAUDE.md) · [`ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md`](../the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md) · [`GENESIS_PROMPT_V1_3.md`](GENESIS_PROMPT_V1_3.md)

*DEPLOYMENT_GUIDE authored 2026-05-17 (S1001982) by Command. Condensation of 10-phase ULTIMATE framework into navigable runbook; preserves m42 POVM-decoupled fact, AP-V7-13 health-vs-behaviour discipline, B3+B4 critical-path blockers; status = planning-only · HOLD-v2 · awaiting G9 fire.*
