# workflow-trace — Quickstart

> **Back to:** [`README.md`](../README.md) · [`CLAUDE.md`](../CLAUDE.md) · [`ONBOARDING.md`](ONBOARDING.md) · [`DEPLOYMENT_GUIDE.md`](DEPLOYMENT_GUIDE.md)
>
> **Function:** 5-minute developer quickstart. Two paths: pre-G9 (planning-only — what to do RIGHT NOW) and post-G9 (implementation — what to do once Luke fires G9). Status: planning-only · 0 LOC · pre-G9 path is live.

---

## Pre-G9 path — "I just sat down at the keyboard"

**The shortest possible loop: read 3 files + check inbox + identify your next spec to author.**

```bash
# 1 — read the project charter and live state (3 minutes)
cd /home/louranicas/claude-code-workspace/the-workflow-engine
batcat CLAUDE.md             # project charter (planning-only · HOLD-v2 · 26 modules)
batcat CLAUDE.local.md       # live session-state delta (6 blockers · resume protocol)
batcat ARCHITECTURE.md       # stable structural summary

# 2 — verify gate state (10 seconds)
ls -la src/ 2>/dev/null && echo "VIOLATION: src/ exists pre-G9" || echo "OK — no src/"
test -f Cargo.toml && echo "VIOLATION: Cargo.toml pre-G9" || echo "OK"
cat GATE_STATE.md

# 3 — check inbox for new peer/Watcher drops (30 seconds)
find ~/projects/shared-context/agent-cross-talk -name "*.md" -mmin -240 | sort
find ~/projects/shared-context/watcher-notices  -name "*.md" -mmin -240 | sort

# 4 — pick your scope
#   - If authoring a spec: see "Add a spec" below
#   - If reviewing: open ai_specs/modules/cluster-{A-H}/
#   - If planning: open ai_docs/optimisation-v7/ULTRAMAP.md (View 2)
```

**Total time: ~5 minutes** to be oriented and productive.

---

## Add a spec (pre-G9, planning-only)

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine
# pick the cluster + module from MODULE_MATRIX
batcat ai_specs/MODULE_MATRIX.md | head -50

# open the canonical spec template (m11 is the richest example)
batcat ai_specs/modules/cluster-D/m11_fitness_weighted_decay.md | head -100

# create your spec (markdown only — NO Rust source files)
nvim ai_specs/modules/cluster-<X>/m<N>_<name>.md
#    Required sections:
#      1. Purpose
#      2. Public surface (Rust signatures in markdown code blocks are spec docs, not code)
#      3. Contracts (CC-N references)
#      4. Inputs / outputs
#      5. Errors (thiserror taxonomy)
#      6. Tests (min count + property/proptest plan)
#      7. Boilerplate lift (% + source clone path)
```

**Discipline.** Rust code blocks WITHIN markdown are spec documentation, NOT source files. Do NOT create `src/*.rs`. Do NOT run `cargo init`. The gate is HARD — pre-G9 violations are caught by [AP-Hab-01 / AP24](optimisation-v7/ANTIPATTERNS_REGISTER.md).

---

## Verify substrate health (1 minute)

```bash
# 11 active habitat services
for port in 8082 8083 8092 8111 8120 8125 8130 8132 8133 8180 10002; do
  curl -sS -o /dev/null -w "$port=%{http_code}\n" --max-time 1 "http://localhost:$port/health" 2>/dev/null
done
# expected: 11 × 200

# stcortex (workflow-trace's primary substrate for m2 read + m13 write + m42 emit)
curl -sS http://localhost:3000/ -o /dev/null -w "stcortex=%{http_code}\n"
~/.local/bin/stcortex status 2>/dev/null | head -5

# atuin (workflow-trace's primary read substrate for m1)
atuin status
ls ~/.local/share/atuin/history.db
```

**TRAP — AP-V7-13 Health-200 ≠ behaviour-verified.** A 200 from `/health` does not prove the service is serving expected behaviour. Always pair with a semantic probe (e.g., `:8125/stats` → `learning_health` ∈ [0.05, 0.15] band).

---

## Post-G9 path — "Luke just typed `start coding workflow-trace`"

**Pre-flight checks.** Confirm G9 is real, not your wish:

```bash
grep "g9_fired = true" plan.toml
ls ~/projects/shared-context/agent-cross-talk/*g9-fired* 2>/dev/null
git log --oneline -3
```

**Bootstrap (one-shot).**

```bash
# 1 — Rename directory (G2 unlock; only fires post-G1 close-notice)
cd /home/louranicas/claude-code-workspace
git mv the-workflow-engine workflow-trace
cd workflow-trace

# 2 — Scaffold from plan.toml (single-crate, two-binary, in-crate lib — ORAC pattern)
scaffold-gen --from-plan plan.toml .

# 3 — Per-shell target dir (AP-V7-07 mitigation: avoid worktree contamination)
export CARGO_TARGET_DIR=./target

# 4 — Quality gate — run the FULL 4-stage chain with PIPESTATUS per stage
cargo check 2>&1 | tail -20 ;                                 test "${PIPESTATUS[0]}" -eq 0 || exit 1
cargo clippy --workspace --all-targets --all-features \
  -- -D warnings 2>&1 | tail -20 ;                             test "${PIPESTATUS[0]}" -eq 0 || exit 1
cargo clippy --workspace --all-targets --all-features \
  -- -D warnings -W clippy::pedantic 2>&1 | tail -20 ;         test "${PIPESTATUS[0]}" -eq 0 || exit 1
cargo test --lib --release 2>&1 | tail -30 ;                   test "${PIPESTATUS[0]}" -eq 0 || exit 1

# 5 — Release build (2 binaries)
cargo build --release --features full 2>&1 | tail -10
ls -la ./target/release/wf-crystallise ./target/release/wf-dispatch
```

---

## Add a module (post-G9)

```bash
cd /home/louranicas/claude-code-workspace/workflow-trace

# 1 — Open the spec (canonical source of truth)
batcat ai_specs/modules/cluster-<X>/m<N>_<name>.md

# 2 — Lift boilerplate from the source-clone catalogue
batcat the-workflow-engine-vault/boilerplate\ modules/BOILERPLATE_INDEX.md | grep m<N>

# 3 — Author module (skeleton already in src/m<N>_<name>/ post-scaffold)
nvim src/m<N>_<name>/mod.rs
nvim src/m<N>_<name>/<submodule>.rs
nvim src/m<N>_<name>/error.rs       # thiserror per ERROR_TAXONOMY.md

# 4 — Author tests (min 50/module; m20 KEYSTONE ≥75-90; m11 Gap 2 ≥70 incl. property)
nvim src/m<N>_<name>/mod.rs         # #[cfg(test)] block in-file
nvim tests/integration/cc<N>_*.rs   # per-CC closure tests

# 5 — Re-run gate after EVERY module
bash scripts/quality-gate.sh        # 4-stage with PIPESTATUS

# 6 — Update status SPEC → WIP → BUILT in MODULE_MATRIX
nvim ai_specs/MODULE_MATRIX.md
```

---

## Build a release binary (post-G9, post-Phase-4)

```bash
cd /home/louranicas/claude-code-workspace/workflow-trace
export CARGO_TARGET_DIR=./target

# Run the full 4-stage gate first; abort if any stage red
bash scripts/quality-gate.sh || exit 1

# Release build
cargo build --release --features full 2>&1 | tail -10

# Binary deploy — TRAP: cp is aliased; ALWAYS /usr/bin/cp -f
/usr/bin/cp -f ./target/release/wf-crystallise ~/.local/bin/wf-crystallise
/usr/bin/cp -f ./target/release/wf-dispatch    ~/.local/bin/wf-dispatch

# Smoke verify (CLI-not-service; no /health to probe)
wf-crystallise --version
wf-dispatch --version
wf-dispatch verify --self-test
```

---

## CLI surface (planned, post-G9)

```bash
wf-crystallise observe [--since=24h] [--dry-run]
wf-crystallise propose [--accept=<id>] [--reject=<id> --reason=...]
wf-crystallise propose ls
wf-crystallise report [--json]

wf-dispatch verify <workflow>
wf-dispatch select
wf-dispatch dispatch <workflow>
```

---

## What to avoid (always)

| Don't | Why |
|---|---|
| `cargo init` pre-G9 | AP-Hab-01 / AP24 — hard refusal |
| Bare `cp -f` | alias trap; ALWAYS `/usr/bin/cp -f` |
| `cargo … \| tail` and trust `$?` | AP-Hab-05 PIPESTATUS swallow |
| `setsid` / `nohup` / `cargo run &` | AP-Hab-07 sandbox child reap |
| `docker container prune -f` | AP-Hab-04 preserve-list discipline |
| stcortex write missing `workflow_trace_*` | AP-Hab-03 / AP30 |
| Trust `/health` 200 alone post-deploy | AP-V7-13 — pair with semantic probe |
| Treat agent claims as facts | Drift class — FP-verify with `git log -1`, re-run gate |

---

## Wave 4.B substrate-as-actor cold-start additions (2026-05-17)

The Wave 4.B closeout (NA-GAP remediation) introduced new surfaces. **If you are authoring a spec that touches a substrate (atuin, stcortex, injection.db, Conductor, SYNTHEX v2, LCM) or the operator, read these FIRST:**

```bash
# Frame A — substrates-as-actors
batcat ai_specs/substrates/INDEX.md          # landing + reading order
batcat ai_specs/substrates/<S-X>.md          # per-substrate dossier (S-A..S-G + S-watcher)

# Substrate-substrate coupling decompositions (when authoring CC-5, CC-4, or CC-7)
batcat ai_specs/substrate-couplings/INDEX.md
batcat ai_specs/substrate-couplings/CC-<N>-decomposed.md

# Refusal-token taxonomy (when authoring any fallible substrate-touching path)
batcat ai_specs/cross-cutting/refusal-taxonomy.md

# Substrate-drift detection (canary contract participation)
batcat ai_specs/cross-cutting/substrate-drift.md

# v0.2.0 deferral ADR (NA-GAP-07 module / NA-GAP-08 fixtures / NA-GAP-10 trust)
batcat ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md
```

The substrate dossiers carry **drift_indicators** + **back_pressure_signals** + **refusal_modes** + **substrate-confirmable receipts** — these are the post-G9 contract surfaces every substrate-touching module spec consumes.

---

## Add a cluster operational landing (post-G9; pre-G9 = HOLD)

Per-cluster runtime landings live at [`layers/cluster-{A-H}/README.md`](../layers/) — these are the OPERATIONAL view (metrics, capacity, failure escalation) counterpart to the prescriptive [`ai_specs/layers/cluster-{A-H}.md`](../ai_specs/layers/). They're scaffolded pre-G9 (8 files); runtime sections are placeholders until post-G9 implementation.

Per-module operational landings ([`modules/m<N>_<name>.md`](../modules/)) are reserved for **post-G9** authoring — see [`modules/README.md`](../modules/README.md) for the rationale (cannot meaningfully describe runtime behaviour pre-G9).

---

## Cross-references

- **Onboarding (deeper):** [`ONBOARDING.md`](ONBOARDING.md)
- **Deployment guide:** [`DEPLOYMENT_GUIDE.md`](DEPLOYMENT_GUIDE.md)
- **Architecture deep dive:** [`ARCHITECTURE_DEEP_DIVE.md`](ARCHITECTURE_DEEP_DIVE.md)
- **Cargo layout spec (pre-Cargo.toml):** [`CARGO_LAYOUT_SPEC.md`](CARGO_LAYOUT_SPEC.md)
- **Performance budgets:** [`PERFORMANCE.md`](PERFORMANCE.md)
- **Anti-patterns full catalogue:** [`ANTIPATTERNS_REGISTER.md`](optimisation-v7/ANTIPATTERNS_REGISTER.md)
- **NA gap analysis (Frame A — required reading before authoring substrate-touching modules):** [`NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md)
- **Decision register:** [`optimisation-v7/DECISION_REGISTER.md`](optimisation-v7/DECISION_REGISTER.md) + ADRs in [`decisions/`](decisions/) + [`optimisation-v7/decisions/`](optimisation-v7/decisions/)

> **Back to:** [`README.md`](../README.md) · [`CLAUDE.md`](../CLAUDE.md) · [`ONBOARDING.md`](ONBOARDING.md)

*QUICKSTART authored 2026-05-17 (S1001982) by Command. Pre-G9 5-minute loop + post-G9 bootstrap + per-module workflow; preserves AP-V7-13 health-vs-behaviour discipline and /usr/bin/cp -f alias trap.*
