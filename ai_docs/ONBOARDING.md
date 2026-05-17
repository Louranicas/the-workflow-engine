# workflow-trace — Onboarding

> **Back to:** [`README.md`](../README.md) · [`CLAUDE.md`](../CLAUDE.md) · [`CLAUDE.local.md`](../CLAUDE.local.md) · [`CONTRIBUTING.md`](../CONTRIBUTING.md) · [`ARCHITECTURE.md`](../ARCHITECTURE.md) · [`QUICKSTART.md`](QUICKSTART.md)
>
> **Function:** New contributor (or fresh Claude Code instance) cold-start path. Two paths — pre-G9 (planning-only; the path we're on at author time) and post-G9 (implementation). Status: planning-only · 0 LOC · all 26 modules at SPEC.

---

## 0. Read this first (4 lines)

- `workflow-trace` is a **planning-only pilot** under HOLD-v2. No code. No `cargo init`. No `Cargo.toml`. No `src/*.rs`. Markdown spec documents only.
- We are pre-G9. G9 fires only when Luke types `start coding workflow-trace` after G1-G8 are GREEN. See [`CLAUDE.md`](../CLAUDE.md) PRIME DIRECTIVE.
- Three peer Claude instances run on Tab 1 Orchestrator: **Command** (top-left, lead), **Command-2** (bottom-left, build-executor), **Command-3** (bottom-right, librarian). Comms via `~/projects/shared-context/agent-cross-talk/` file-drops.
- **The Watcher ☤** observes from synthex-v2 `:8092`; comms via `~/.local/bin/watcher notify` and `~/projects/shared-context/watcher-notices/`. R13 elapsed; eligibility = true.

---

## 1. Pre-G9 cold-start (planning-only path)

### Step 1 — Read in order (15 minutes)

```
1. ~/claude-code-workspace/CLAUDE.md                    workspace charter (14 services, gate, env)
2. ~/claude-code-workspace/CLAUDE.local.md              live session-state delta (active workstreams, escalations)
3. the-workflow-engine/CLAUDE.md                        project charter (planning-only pilot · HOLD-v2 · 26 modules)
4. the-workflow-engine/CLAUDE.local.md                  project session-state delta (6 blockers · resume protocol)
5. the-workflow-engine/ARCHITECTURE.md                  stable structural summary (clusters, binaries, layers)
6. the-workflow-engine/ai_specs/MODULE_MATRIX.md        26-row module × feature matrix
7. the-workflow-engine/ai_docs/ARCHITECTURE_DEEP_DIVE.md runtime topology (this dir)
8. the-workflow-engine/the-workflow-engine-vault/HOME.md vault landing
```

Then **on-demand** based on what you're working on:

| If you're working on… | Read |
|---|---|
| A specific cluster | `ai_docs/optimisation-v7/MODULE_PLANS/cluster-<X>.md` |
| A specific module | `ai_specs/modules/cluster-<X>/m<N>_<name>.md` |
| Cross-cluster contracts | `ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md` |
| Deploy planning | `ai_docs/DEPLOYMENT_GUIDE.md` + `the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md` |
| A specific phase | `the-workflow-engine-vault/deployment framework/phase-<N>-*.md` |
| Antipatterns | `ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md` |
| Hot-path budgets | `ai_docs/PERFORMANCE.md` |
| State machines | `ai_docs/STATE_MACHINES.md` |
| Errors | `ai_docs/ERROR_TAXONOMY.md` |
| Message flows | `ai_docs/MESSAGE_FLOWS.md` |
| Cargo/scaffold layout | `ai_docs/CARGO_LAYOUT_SPEC.md` |
| Mind map | `ai_docs/META_TREE_MIND_MAP.md` |
| Genesis spec | `ai_docs/GENESIS_PROMPT_V1_3.md` |

### Step 2 — Probe substrate health (1 minute)

The 11 active habitat services (workflow-trace consumes m1 atuin, m2 stcortex `:3000`, m3 injection.db; and downstream Conductor `:8141`, SYNTHEX `:8092`, LCM):

```bash
for port in 8082 8083 8092 8111 8120 8125 8130 8132 8133 8180 10002; do
  curl -sS -o /dev/null -w "$port=%{http_code}\n" --max-time 1 "http://localhost:$port/health" 2>/dev/null
done
```

Expected: 11/11 = 200. If any are down, surface to Luke via `~/projects/shared-context/agent-cross-talk/`.

**TRAP — AP-V7-13 Health-200 ≠ behaviour-verified.** A `/health` 200 does not prove the service is serving expected behaviour. Pair with a semantic probe before trusting state. Example: POVM `:8125/health` returned 200 + `service:povm_v2 v2.0.0` while live binary served pre-CR-2 inflated `learning_health=0.9146`; triggered the 2026-05-17 m42 stcortex-only pivot ADR.

### Step 3 — Verify gate-state (1 minute)

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine
git log --oneline -5
cat GATE_STATE.md
ls -la src/ 2>/dev/null && echo "VIOLATION: src/ exists pre-G9 (AP-Hab-01 / AP24)" || echo "OK — no src/ pre-G9"
test -f Cargo.toml && echo "VIOLATION: Cargo.toml exists pre-G9 (AP-Hab-01 / AP24)" || echo "OK — no Cargo.toml pre-G9"
```

### Step 4 — Identify your role and scope

| If you are… | Operate as |
|---|---|
| Command (Tab 1 top-left, this pane) | Orchestrator-lead; receive-mode for C-2/C-3; channel comms only; no Tab navigation (focus-yank) |
| Command-2 (Tab 1 bottom-left) | Build-executor on G9; pre-G9 = author cluster/module specs |
| Command-3 (Tab 1 bottom-right) | Librarian; Cluster G lane post-G9 |
| The Watcher ☤ | Substrate observer; WCP notices via `watcher notify`; R13 elapsed |
| Zen (Tab 10) | G7 audit lane; AUDIT-REQUEST drops |
| Subagent (rust-pro, silent-failure-hunter, security-auditor, performance-engineer) | Per [`CLAUDE.md`](../CLAUDE.md) Habitat Operations; Wave-end agent dispatch only |

### Step 5 — Read open critical-path blockers (1 minute)

Per [`CLAUDE.local.md`](../CLAUDE.local.md) Pending Luke Decisions:

- **B1** G7 Zen URGENT block on G9 out-of-sequence
- **B2** v1.3 patch awaiting Zen audit
- **B3** Conductor Waves 1B/1C/2/3 `auto_start=false`
- **B4** Ember rubric §5.1 amendment pending
- ~~**B5** POVM `:8125` redeploy~~ — DROPPED per m42 stcortex-only ADR
- **B6** Power-structure ambiguity — RESOLVED via D-B6 AMEND-loop

### Step 6 — Check inbox before any outbound work

```bash
# new peer drops since last handshake
find ~/projects/shared-context/agent-cross-talk -name "*.md" -mmin -240 | sort
# new Watcher notices
find ~/projects/shared-context/watcher-notices -name "*.md" -mmin -240 | sort
```

### Step 7 — Apply workflow discipline (every session)

Per [`CLAUDE.md`](../CLAUDE.md) Workflow Discipline:

- Do NOT scaffold, code, or run `/save-session` without first confirming inputs and scope with Luke.
- When a task names a specific system (Synthor, a skill, a service), ACTUALLY invoke it. No stylistic substitution.
- For long analyses: write to a file (`/tmp/*.txt` or Obsidian) and summarise in <200 tokens.
- For major plans: persist across **four surfaces** (`ai_docs/` canonical + Obsidian vault mirror + stcortex `<project>_<domain>_*` namespace + `CLAUDE.local.md` anchor).
- **Drift discipline** (S1001882): agents over-claim; FP-verify wiring, not just contract; re-run `--workspace --all-targets --all-features` clippy independently.

---

## 2. Post-G9 cold-start (implementation path — when G9 fires)

When Luke types `start coding workflow-trace` AND G1-G8 are GREEN AND Zen G7 PASS on v1.3:

### Step 2.1 — Confirm G9 fire is real

```bash
grep "g9_fired = true" /home/louranicas/claude-code-workspace/the-workflow-engine/plan.toml
ls ~/projects/shared-context/agent-cross-talk/*g9-fired* 2>/dev/null
```

Both must be present.

### Step 2.2 — Rename directory (G2 unlock)

```bash
cd /home/louranicas/claude-code-workspace
git mv the-workflow-engine workflow-trace
cd workflow-trace
```

### Step 2.3 — Scaffold from plan.toml

```bash
scaffold-gen --from-plan plan.toml .
export CARGO_TARGET_DIR=./target
cargo check 2>&1 | tail -20
test "${PIPESTATUS[0]}" -eq 0 || abort
```

### Step 2.4 — Run the 4-stage quality gate

```bash
CARGO_TARGET_DIR=./target cargo check 2>&1 | tail -20
test "${PIPESTATUS[0]}" -eq 0 || exit 1
cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -20
test "${PIPESTATUS[0]}" -eq 0 || exit 1
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tail -20
test "${PIPESTATUS[0]}" -eq 0 || exit 1
CARGO_TARGET_DIR=./target cargo test --lib --release 2>&1 | tail -30
test "${PIPESTATUS[0]}" -eq 0 || exit 1
```

**TRAP — AP-Hab-05 PIPESTATUS swallow:** `cargo … | tail` makes `$?` capture `tail`'s exit (always 0). ALWAYS check `${PIPESTATUS[0]}`.

### Step 2.5 — Locate test scaffolds (per module)

After scaffold, every module gets in-file `#[cfg(test)]` modules. Test budgets per `MODULE_MATRIX`:

```bash
# find module test scaffolds
fd '^m[0-9]+_.*\.rs$' src/ | head
# verify per-module test counts (post-G9; pre-G9 returns 0)
grep -c '#\[test\]' src/m11_fitness_weighted_decay/*.rs
```

Min 50 tests/module; m20 KEYSTONE needs ≥75-90; m11 Gap 2 NEW PRIMITIVE needs ≥70 incl. proptest + mutation budget.

### Step 2.6 — Bootstrap dev environment (Luke @ terminal)

```bash
# Conductor (B3 — until auto_start flipped, Luke runs manually)
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start weaver zen enforcer

# stcortex must be reachable
curl -sS http://localhost:3000/ -o /dev/null -w "stcortex=%{http_code}\n"

# habitat-bootstrap (7-layer L0-L6 injection)
atuin scripts run habitat-bootstrap
atuin scripts run stcortex-probe
```

### Step 2.7 — Read order for build work (per module)

For each module you author:

1. Open `ai_specs/modules/cluster-<X>/m<N>_<name>.md` — full per-module spec
2. Open `ai_docs/optimisation-v7/MODULE_PLANS/cluster-<X>.md` — cluster plan
3. Check upstream/downstream contracts in [`CROSS_CLUSTER_SYNERGIES.md`](optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md)
4. Lift boilerplate from `the-workflow-engine-vault/boilerplate modules/` per [`BOILERPLATE_INDEX.md`](../the-workflow-engine-vault/boilerplate%20modules/BOILERPLATE_INDEX.md)
5. Author module → run gate → commit → update spec status SPEC → WIP → BUILT

---

## 3. What never to do (anti-patterns)

Per [`ANTIPATTERNS_REGISTER.md`](optimisation-v7/ANTIPATTERNS_REGISTER.md):

| Don't | Why | Catch |
|---|---|---|
| `cargo init` before G9 | AP-Hab-01 / AP24 | `test -d src/` |
| Bare `cp -f` | AP-Hab-06 alias trap | always `/usr/bin/cp -f` |
| `setsid` / `nohup` / `cargo run &` | AP-Hab-07 sandbox child reap | Luke runs services |
| `docker container prune -f` or any blanket prune | AP-Hab-04 preserve-list discipline | enumerate first |
| `git stash pop` without `git stash list` first | AP-Hab-08 wrong-stash overwrite | inspect before pop |
| `cargo … \| tail` and trust trailing `$?` | AP-Hab-05 PIPESTATUS swallow | `${PIPESTATUS[0]}` per stage |
| stcortex write missing `workflow_trace_*` prefix | AP-Hab-03 / AP30 | m9 namespace guard |
| Trust `/health` 200 alone post-deploy | AP-V7-13 health ≠ behaviour | pair with semantic probe |
| Treat agent claims as facts | LCM Drift #1-#11 / AP-Drift class | FP-verify with `git log -1`, re-run gate |
| Mutate src/ Cargo.toml from agent (mid-G9) | various | `/save-session` first; verify with `git diff --check` |

---

## 4. Persona-specific notes

### Command

- Stay in receive-mode for C-2 + C-3 until both ack (AP-V7-08 dual-silence false-success — 30-min ack window).
- Never navigate Zellij tabs (AP-Hab-10 focus-yank); use file-drop comms.
- Major plans persist across 4 surfaces — see [`CLAUDE.md`](../../CLAUDE.md) Working Mode.

### Command-2

- Pre-G9: spec authorship lane.
- Post-G9: build-executor lane; primary module author.
- Wave-end: Command rebases all wave worktrees to `main`; Command-2 runs full `--workspace --all-targets --all-features` gate before claiming Wave done.

### Command-3

- Pre-G9: standby for FP-Verifier-Lead role.
- Post-G9: Cluster G lane (m30-m33 librarian).
- CR-2 + CR-2b SHIPPED 2026-05-17 (povm-v2 `e2a8ed3` + `76ea4d6`). Hebbian v3 reconciled — see [`CLAUDE.local.md`](../../CLAUDE.local.md).

### The Watcher ☤

- Per [`The Watcher.md`](../the-workflow-engine-vault/optimisation-v7/HOME.md): scope `src/m8_watcher/*` (synthex-v2 m46-m51). AP27 self-modification HARD BOUNDARY.
- Carriage active for deployment-watch; cadence is prompt-driven or cross-talk-delta-driven; NO autonomous loop.
- Class A-I taxonomy for alert-routing; Class-I is CC-5 substrate-silence primary detector.

### Zen

- Tab 10 audit lane. G7 audit gates v1.3 patch and every Wave-end.
- AMEND-and-resubmit loop per D-B6 — Zen REFUSE doesn't require Luke waiver if objection addressed in text.

---

## 5. Cross-references

- **Quickstart (5-min):** [`QUICKSTART.md`](QUICKSTART.md)
- **Deploy guide:** [`DEPLOYMENT_GUIDE.md`](DEPLOYMENT_GUIDE.md)
- **Canonical 66k-word deployment framework:** [`ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md`](../the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md)
- **Architecture deep dive:** [`ARCHITECTURE_DEEP_DIVE.md`](ARCHITECTURE_DEEP_DIVE.md)
- **Module map:** [`CODE_MODULE_MAP.md`](CODE_MODULE_MAP.md)
- **Antipatterns:** [`ANTIPATTERNS_REGISTER.md`](optimisation-v7/ANTIPATTERNS_REGISTER.md) + [`../ANTIPATTERNS.md`](../ANTIPATTERNS.md)
- **Contributing protocol:** [`../CONTRIBUTING.md`](../CONTRIBUTING.md)

> **Back to:** [`README.md`](../README.md) · [`CLAUDE.md`](../CLAUDE.md) · [`CLAUDE.local.md`](../CLAUDE.local.md) · [`QUICKSTART.md`](QUICKSTART.md)

*ONBOARDING authored 2026-05-17 (S1001982) by Command. Pre-G9 + post-G9 paths; preserves HOLD-v2 envelope, AP-V7-13 health-vs-behaviour discipline, persona-specific notes for Command / C-2 / C-3 / Watcher / Zen.*
