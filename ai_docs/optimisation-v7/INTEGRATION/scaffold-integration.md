---
title: /scaffold Integration Deep-Dive — workflow-trace V7
date: 2026-05-17 (S1001982)
kind: planning-only · integration deep-dive · expands G5 § /scaffold
parent: GENERATIONS/G5-tooling.md
owner: Command (Genesis Day 0); Command per-Wave-end (drift check); on-demand any session
status: planning-only — invoked POST-G9 only
---

# `/scaffold` Integration — workflow-trace V7

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../GENERATIONS/G5-tooling.md]] · [[../ULTRAMAP.md]]
>
> Siblings: [[atuin-integration.md]] · [[devops-v3-integration.md]] · [[codesynthor-v8-integration.md]] · [[json-claude-code-optimisation.md]] · [[progressive-disclosure-obsidian.md]]

---

## Overview

`/scaffold` is **CodeSynthor V8's structural bound** — its corrective function whenever V8's generative authoring drifts from the declared plan.toml shape (per G4 Axis 5 + S1002029 learning #3, cited by G5-tooling.md § Gap closure GAP-Tool-01). For workflow-trace it serves three roles: (1) the Genesis Day 0 generator that materialises the 26-module / 9-layer / two-binary src/ tree from `plan.toml` (per ULTRAMAP View 2 Cluster × Module × src/path table); (2) the per-Wave-end consistency check that fails-loud when an authored module diverges from plan; (3) the on-demand audit when any agent suspects layer drift. It is invoked via the `scaffold-mastery` skill (Claude Code) or the `~/.local/bin/scaffold` direct CLI; it is read-only against `plan.toml` and write-allowed against `src/`. Activation is post-G9 only — under HOLD-v2 (per project CLAUDE.md PRIME DIRECTIVE) all scaffolding is forbidden until Luke re-issues `start coding workflow-trace` after G1-G8 green.

---

## When invoked

| Trigger | Cadence | Owner | Mode | Output |
|---|---|---|---|---|
| **Genesis Day 0** (post-G9 first build) | once | Command (foreground) | write (create src/ tree) | full `src/m1_…` through `src/m42_…` skeleton + `Cargo.toml` + `build.rs` + `migrations/` per ULTRAMAP § Layer Map |
| **Wave-end consistency check** | per Wave (1, 2, 3) | Command (Wave-merge orchestrator) | read-only diff | drift report `/tmp/wt-scaffold-drift-wave-{N}.txt` |
| **On-demand audit** | any session | any agent (Command / C-2 / C-3 / Zen) | read-only diff | drift report stdout |
| **Post-deploy weekly soak** (D30→D120) | weekly | Watcher (carriage) | read-only diff | weekly drift entry in `Watcher Deployment Watch Journal` |
| **D120 sunset retrospective** | once | m11 + Luke | read-only diff | final drift report for IC-N candidate seeding |

**Cadence aligns with G5 Tooling Integration Time Matrix (G5-tooling.md § Tooling integration time matrix):** Day 0 (initial) + per-Wave-end (drift) + Phase 5C weekly (retrospective) + D120 (sunset).

---

## plan.toml ↔ src/ tree mapping (the contract)

Per G4 Axis 5 decision: **LCM `plan.toml` is spec authority + supplementary markdown narrative**. `plan.toml` lives at project root; `/scaffold` reads it as the single source of structural truth.

### plan.toml top-level shape (workflow-trace specialisation)

The `plan.toml` declares — for `/scaffold` to materialise:

| Section | Drives | Source of values |
|---|---|---|
| `[crate]` | top-level `Cargo.toml` `[package]` | G4 Axis 1 (single crate + features); name = `workflow-trace`; version = `0.1.0`; edition = `2024` |
| `[binaries]` | `src/bin/wf_crystallise.rs` + `src/bin/wf_dispatch.rs` stubs | KEYWORDS_20 #18 two-binary; per `[[bin]]` entry with feature-gate |
| `[features]` | `Cargo.toml` `[features]` matrix | G2 consolidation feature strategy: `default`, `full`, `lib-only`, `cli-only`, `dispatch-only`, `serve` (feature-gated post-D60 HTTP per G4 Axis 3 exception), `ralph-integration` (G4 Axis 4 M2+ optional) |
| `[layers]` | layer module discovery roots | one entry per L0..L8 (ULTRAMAP View 1); each entry lists permitted parent + permitted children |
| `[modules.mN]` (×26) | `src/mN_<theme>/mod.rs` + `src/mN_<theme>/tests.rs` skeletons | ULTRAMAP View 2 — per-module purpose + LOC budget + test budget + cluster + layer |
| `[bridges]` | `mN_bridges/<peer>.rs` skeleton with circuit-breaker hook | G4 convergent pattern #10; m40/m41/m42 each declare their peer host:port |
| `[migrations]` | `migrations/NNNN_<name>.sql` skeleton | G4 convergent pattern #7 SQLite + migrations/ |
| `[verify_sync]` | scripts/verify-sync.sh invariant catalogue | ≥18 invariants per KEYWORDS_20 #19 |
| `[stcortex]` | namespace constants in `src/workflow_core/namespace.rs` | KEYWORDS_20 #2 AP30 `workflow_trace_*` prefix |
| `[atuin]` | atuin script declarations (7 proposed per G5) | per [[atuin-integration.md]] |

### Mapping invariants (`/scaffold` enforces; verify-sync gates)

| # | Invariant | Failure → |
|---|---|---|
| S1 | Every `[modules.mN]` has matching `src/m{N}_<theme>/mod.rs` | drift report MISSING |
| S2 | Every `src/m{N}_<theme>/mod.rs` has matching `[modules.mN]` | drift report ORPHAN |
| S3 | `[layers.LK]` permitted children equals actual `mN` set | drift report LAYER_MISMATCH |
| S4 | Every `[[bin]]` has a `src/bin/<name>.rs` stub | drift report BIN_MISSING |
| S5 | Every `[bridges.<peer>]` has `src/m{N}_bridges/<peer>.rs` skeleton | drift report BRIDGE_MISSING |
| S6 | Every `[migrations.NNNN]` has a `migrations/NNNN_<name>.sql` file | drift report MIGRATION_MISSING |
| S7 | Feature matrix in `Cargo.toml` matches `[features]` set in plan.toml | drift report FEATURE_DRIFT |
| S8 | Namespace constant in `src/workflow_core/namespace.rs` matches `[stcortex].prefix = "workflow_trace_"` | drift report NAMESPACE_DRIFT (AP30 risk) |

Invariants S1-S8 are a strict subset of the ≥18 verify-sync invariants enumerated in G2 (per KEYWORDS_20 #19); `/scaffold` owns S1-S8, the remaining ≥10 invariants belong to `scripts/verify-sync.sh` proper.

---

## Drift detection algorithm

`/scaffold check` runs the algorithm; `/scaffold sync` applies non-destructive additions only (never deletes).

```text
ALGORITHM scaffold-drift-check(plan.toml, src/):
  D := []                                       # drift list
  PLAN := parse_toml(plan.toml)
  FS   := walk(src/)

  # S1 — every plan module exists on disk
  for module in PLAN.modules:
    expected := "src/m{module.id}_{module.theme}/mod.rs"
    if expected not in FS:
      D += MISSING(module, expected)

  # S2 — every on-disk module is in plan
  for path in FS where matches "src/m\\d+_\\w+/mod.rs":
    if not PLAN.modules.includes(extract_id(path)):
      D += ORPHAN(path)

  # S3 — layer membership consistency
  for layer in PLAN.layers:
    declared := set(layer.children)
    actual   := set(FS where module belongs to layer K)
    if declared ≠ actual:
      D += LAYER_MISMATCH(layer, declared XOR actual)

  # S4..S8 — analogous

  # Cluster integrity (V7 addition)
  for cluster in CLUSTERS (A..H):
    members := PLAN.modules.where(cluster == cluster.id)
    if !members.is_layer_contiguous():
      D += CLUSTER_NON_CONTIGUOUS(cluster)

  emit_report(D)
  return D.is_empty()
```

The algorithm is **idempotent** and **read-only by default**. `/scaffold sync` writes ONLY missing skeletons (never overwrites authored code) — and only with operator confirmation. Per G4 LCM Drift #3 (scaffold-without-binary-wiring), `/scaffold` will refuse to mark a module "wired" — that's the Wave-end integration smoke's job (per [[devops-v3-integration.md]] T4).

---

## Failure modes (≥3)

| ID | Failure | Detection | Mitigation |
|---|---|---|---|
| SC-01 | plan.toml drifts from src/ silently (Wave-2 author adds m7b alongside m7) | `/scaffold check` reports ORPHAN(m7b) | Command files D-drift in `~/projects/shared-context/agent-cross-talk/{TS}_command_scaffold_drift.md`; module dropped or plan.toml amended via decision-record |
| SC-02 | Cluster non-contiguous (someone moves m5 from Cluster B into Cluster C) | `/scaffold check` reports CLUSTER_NON_CONTIGUOUS | hard-block at Wave-end checklist (per G4 § Wave-end orchestrator checklist step 4); ULTRAMAP View 2 amendment required |
| SC-03 | Namespace constant drifts (S102-class AP30 violation; someone hard-codes `"workflow_engine_*"`) | `/scaffold check` reports NAMESPACE_DRIFT | Class-D Watcher flag fires (KEYWORDS_20 #20 + #2); m9 namespace guard refuses runtime writes; immediate fix-forward |
| SC-04 | Bridge skeleton missing for declared peer (e.g., `[bridges.synthex_v2]` declared but `src/m40_bridges/synthex_v2.rs` absent) | `/scaffold check` reports BRIDGE_MISSING | per-Wave-end checklist refuses tag-complete; rust-pro subagent dispatched to author skeleton |
| SC-05 | `/scaffold sync` overwrites authored code (catastrophic) | git status after `sync` shows modified m20_prefixspan/algorithm.rs | **mitigation: `/scaffold sync` operates write-only on missing files; refuses to overwrite (per algorithm)**; if mitigation fails, `git restore` from main; root-cause to scaffold-mastery skill bug |
| SC-06 | Two agents run `/scaffold sync` simultaneously in two worktrees | `Cargo.toml` merge conflict at Wave-end | per AGENT_VIEW_GITWORKTREES.md WT-04 (file ownership); `/scaffold sync` reserved for Command in main worktree only — subagents call `/scaffold check` (read-only) |

---

## Atuin trajectory (provenance)

Per G5 § Atuin integration provenance principle (S1002029 #4: atuin is the ONLY cross-tool ledger), every `/scaffold` invocation lands in `~/.local/share/atuin/history.db`. The CLI form is preferred over the skill invocation when audit-trail matters.

```bash
# At Genesis Day 0:
atuin search "scaffold" --before 1d                # initial materialisation present
# Per Wave-end:
atuin search "scaffold check" --before 7d          # drift checks per Wave
# Investigation:
atuin search "scaffold" --before 30d | grep -i drift  # historical drift events
```

The atuin row format embeds the working directory + exit code + duration; this is the substrate-frame provenance per [[atuin-integration.md]] § Provenance principle.

---

## Source skill citation

Per G5 § /scaffold integration footer (G5-tooling.md line 40):

- **`scaffold-mastery` skill** — generates plan-driven Rust microservice scaffolds from `plan.toml`; supports custom layers, modules, test kinds, feature gates, consent configuration, per-module dependencies. Workflow-trace's plan.toml is its driver.
- **`genesis` skill** — end-to-end Habitat new-service creation (plan.toml → scaffold → CLAUDE.md → git init → devenv.toml registration → release build → deploy → health verify). For workflow-trace **only steps 1-3 apply** (plan.toml + scaffold + CLAUDE.md) because per G4 Axis 3 the project is CLI-first, not a devenv-registered service.

---

## Verification commands

Run from project root after Genesis Day 0 and at every Wave-end:

```bash
# Drift check (read-only)
~/.local/bin/scaffold check --plan plan.toml --src src/

# Drift detail with per-invariant breakdown
~/.local/bin/scaffold check --plan plan.toml --src src/ --verbose --invariants S1,S2,S3,S4,S5,S6,S7,S8

# Sync missing skeletons (writes only, never overwrites) — Command in main worktree only
~/.local/bin/scaffold sync --plan plan.toml --src src/ --dry-run     # always dry-run first
~/.local/bin/scaffold sync --plan plan.toml --src src/ --confirm

# Wave-end integration with checklist (per G4 Wave-end orchestrator)
./scripts/wave-end-checklist.sh <wave-N>          # invokes scaffold check as step 0

# Cross-check against ULTRAMAP View 2 table (manual reading + grep)
rg -n '^\| \*\*[A-H] L\d+\*\*' ../ai_docs/optimisation-v7/ULTRAMAP.md | wc -l       # should be ≥ 26 module rows
```

---

## Sign-off

✅ scaffold-integration spec complete. Genesis Day 0 + per-Wave-end + on-demand cadence locked. plan.toml ↔ src/ contract defined with S1-S8 invariants. Drift detection algorithm specified read-only-by-default. ≥3 failure modes (6 enumerated) with mitigations. Atuin provenance trajectory. Source skills cited. Verification commands deterministic.

*Authored 2026-05-17 (S1001982) — Command for V7 G5 expansion. Subject to G7 Zen spec audit. HOLD-v2 respected: planning-only.*
