# CONTRIBUTING — workflow-trace

> **Status:** Pre-G9 (HOLD-v2 active). Contribution surface is **spec-only** until Luke types `start coding workflow-trace`.
> **Authority:** Luke @ node 0.A (project owner) · Zen (G7 audit lane) · Watcher ☤ (observe + propose; AP27 self-mod forbidden) · Command/C-2/C-3 (Tab 1 Orchestrator triad)

---

## Who can contribute, and how

### Pre-G9 (now)

| Contributor | Surface | Channel |
|---|---|---|
| **Luke** | All gates, all decisions, all waivers | Direct prompt |
| **Command / C-2 / C-3** | Spec authoring, gap analysis, scaffold orchestration | Tab 1 panes + file-drop `~/projects/shared-context/agent-cross-talk/` |
| **The Watcher ☤** | Substrate observation, Class-A/E/I flags, deployment watch journal | `~/.local/bin/watcher notify` + `~/projects/shared-context/watcher-notices/` |
| **Zen** | G7 audit, AMEND-loop verdicts, gold-standard cross-checks | File-drop AUDIT-REQUEST |
| **subagents** | Wave-2/3 author + verify (cluster spec author, .claude optimiser, ai_docs deep author, ultramap author, verifier) | Spawned by Command via Agent tool |

### Post-G9 (G1-G9 sequence GREEN)

| Contributor | Surface |
|---|---|
| Anyone | Per-cluster module implementation, tests, benchmarks, integration tests, runbook updates |

---

## How to add a new module spec (pre-G9 path)

1. Confirm the module isn't already listed in [`ai_specs/MODULE_MATRIX.md`](ai_specs/MODULE_MATRIX.md). If it is, you're editing not adding.
2. Add a row to MODULE_MATRIX with the 14 columns populated.
3. Add an entry to [`plan.toml`](plan.toml) `[[modules]]` array.
4. Write the per-module spec at `ai_specs/modules/cluster-<X>/m<N>_<name>.md` using the 13-section template from any existing module spec.
5. Cross-link from `ai_specs/INDEX.md` and the appropriate `ai_specs/layers/cluster-<X>.md`.
6. Update [`ARCHITECTURE.md`](ARCHITECTURE.md) cluster table if module count changes.
7. Update [`CHANGELOG.md`](CHANGELOG.md) under `[Unreleased]`.
8. Add Obsidian vault note at `the-workflow-engine-vault/module specs/m<N>_<name>.md` (or amend the cluster spec).

---

## How to add a new antipattern

See [`ANTIPATTERNS.md`](ANTIPATTERNS.md) § "How to add a new antipattern":
1. Add canonical entry at [`ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md`](ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md)
2. Update machine-readable [`.claude/anti_patterns.json`](.claude/anti_patterns.json)
3. Update the one-line row in [`ANTIPATTERNS.md`](ANTIPATTERNS.md)
4. If refusal-mode, add a Bash-pattern block hook in [`.claude/hooks/`](.claude/hooks/)

---

## How to fly a decision (D-B6 AMEND-loop)

Per the D-B6 precedence decision (Genesis v1.3 § 7):

1. **Author** the proposal as a markdown ADR in `ai_docs/optimisation-v7/decisions/YYYY-MM-DD-<slug>.md`
2. **File** an AUDIT-REQUEST to Zen at `~/projects/shared-context/agent-cross-talk/YYYY-MM-DDThhmmssZ_command_g7_audit_request_<slug>.md`
3. **Wait** for Zen verdict: ACCEPT / REFUSE / AMEND
4. **If REFUSE:** amend per Zen's objection, file an amended ADR, return to step 2 (no Luke waiver required if objection addressed)
5. **If ACCEPT:** persist across 4 surfaces (canonical + Obsidian + stcortex + CLAUDE.local anchor), update CHANGELOG, notify peers

---

## Quality gate (post-G9 only)

Per [`GOLD_STANDARDS.md`](GOLD_STANDARDS.md):

```bash
CARGO_TARGET_DIR=./target cargo check 2>&1 | tail -20 && \
cargo clippy -- -D warnings 2>&1 | tail -20 && \
cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=./target cargo test --lib --release 2>&1 | tail -30
```

Zero tolerance at every stage. `${PIPESTATUS[0]}` for the chained-pipe gates so `tail`'s exit doesn't mask earlier failures (AP-V7-13 cousin).

---

## Commit conventions

Pre-G9: spec commits only. Format:
- `spec(<scope>): <imperative summary>` (e.g. `spec(m20): add PrefixSpan algorithm sketch + bench targets`)
- `docs(<scope>): <imperative>`
- `decisions(<id>): <imperative>`
- `scaffold(<area>): <imperative>` (e.g. `scaffold(ai_specs): seed cluster-A module spec dir`)

Post-G9: standard Rust:
- `feat(<scope>): <imperative>`
- `fix(<scope>): <imperative>`
- `refactor(<scope>): <imperative>`
- `test(<scope>): <imperative>`
- `perf(<scope>): <imperative>`

Always include test counts post-G9 in commit body (e.g. "1830/1830 passing, zero warnings").

---

## Code-of-conduct

See [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

---

> **Back to:** [`README.md`](README.md) · [`CLAUDE.md`](CLAUDE.md) · [`GATE_STATE.md`](GATE_STATE.md) · [`GOLD_STANDARDS.md`](GOLD_STANDARDS.md)
