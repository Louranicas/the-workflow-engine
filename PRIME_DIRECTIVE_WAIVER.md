# PRIME_DIRECTIVE_WAIVER — S1002127 Scaffold-Only Scope Override

> **Filed:** 2026-05-17 (S1002127)
> **Authoriser:** Luke @ node 0.A — direct prompt
> **Recipient:** Command (Tab 1 Orchestrator top-left)
> **Audit lane:** Zen G7 (notified via this record; verdict pending)
> **Watcher lane:** Class-A flag candidate (sprawl-vs-substrate-state ratio; tracked at [`the-workflow-engine-vault/Watcher Deployment Watch Journal S1001982.md`](the-workflow-engine-vault/Watcher%20Deployment%20Watch%20Journal%20S1001982.md))

---

## What this waiver waives

Per [`CLAUDE.md`](CLAUDE.md) § PRIME DIRECTIVE, the prior clause read:

> **Planning-only pilot.** No code. No `cargo init`. No `cargo new`. No source files under `src/`. No `Cargo.toml`. **No scaffold.** Markdown spec documents only.

Luke's S1002127 instruction (verbatim — file drop in conversation):

> "refer to /home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.local.md and use /scaffold and the dev ops engine V3 to fully scaffold the code base **(you are not to start coding until i type 'start coding')** ensure the .claude folder is fully optimised for the full end to end stack of the code base and claude code including the use of .json … ensure fully deployed ai_docs folder fully deployed ai_specs folder fully deployed and in sync layers, modules, src, ultramap, code logic contextual flow map … proceed seamlessly"

---

## Interpretation (Command's tight reading)

The waiver applies to **structure + specs + config** only. It does NOT waive the no-code / no-cargo-build / no-G9-fire clauses.

| Action class | Allowed under this waiver? | Why |
|---|---|---|
| `mkdir` directory structure | ✅ YES | Structure is not code |
| Markdown specs (per-module Rust godtier spec) | ✅ YES | Spec ≠ implementation |
| `.claude/` config (JSON, agents, commands, hooks, skills, schemas) | ✅ YES | Claude Code tooling, not Rust |
| `plan.toml` (scaffold-mastery input metadata) | ✅ YES | Build-tool config, not Rust source |
| `ai_docs/` deep authoring (architecture, runbooks, decisions) | ✅ YES | Documentation |
| `ai_specs/` per-module specs (26 files) | ✅ YES | Markdown specs sourced from existing vault canonical |
| `ultramap/` flow maps (Mermaid) | ✅ YES | Diagrams, not Rust |
| `layers/`, `modules/` landing pages | ✅ YES | Navigation, not Rust |
| Gold-standard files (LICENSE, CHANGELOG, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY.md, .gitignore) | ✅ YES | Project metadata |
| **`.rs` source files** under `src/` | ❌ NO | "you are not to start coding until i type 'start coding'" |
| **`Cargo.toml` at root** | ❌ NO | Implies imminent `cargo build`; G9 not fired; metadata captured in [`plan.toml`](plan.toml) + [`ai_docs/CARGO_LAYOUT_SPEC.md`](ai_docs/CARGO_LAYOUT_SPEC.md) instead |
| `cargo init` / `cargo new` / `cargo build` | ❌ NO | HOLD-v2 hard refusal carries forward |
| Directory rename `the-workflow-engine/` → `workflow-trace/` | ❌ NO | G2 not fired |
| stcortex writes under `workflow_trace_*` namespace | ❌ NO | G8 not green |

---

## Wider-interpretation escape

If Luke wants Cargo.toml at root + empty `src/m*/mod.rs` + `src/lib.rs` stubs (which would compile but not implement anything), tell Command and the waiver widens. Default-tight interpretation chosen because:

1. The phrase "not to start coding" is most safely read as "no `.rs` source files of any kind"
2. A `Cargo.toml` at root with stub `src/lib.rs` would let any subagent or hook trigger `cargo build` and silently begin implementation
3. G9 has not fired; HOLD-v2 envelope was specifically designed to forbid Cargo + src/

---

## Watcher notice

This waiver narrows but does NOT close Watcher's Class-E flag (sprawl-vs-substrate ratio). Class-E resolves on first `git commit` with `cargo check` exit 0 — i.e. at G9-fire, not at scaffold-completion. Scaffold-only output still counts as planning sprawl by the Watcher's metric, but is now structure-bearing planning sprawl (per Luke's directive that the structure pre-positions a faster G9 fire when it arrives).

Watcher channel notice should be filed: [`scripts/watcher`](../scripts/) `notify` after scaffold completion.

---

## Zen G7 notice

The pending G7 audit (filed 2026-05-17T160500Z) is on the v1.3 amendment, not on this scaffold waiver. If Zen reads this waiver and objects, the AMEND-and-resubmit loop (D-B6) applies: Command amends scaffold scope (e.g. tighten further or relax further) and resubmits. No Luke waiver of Zen REFUSE required.

---

## Record of scaffold actions taken under this waiver

| Wave | Actions | Status |
|---|---|---|
| Wave 0 | mkdir skeleton; root anchor files (README, ARCHITECTURE, GATE_STATE, ANTIPATTERNS, PATTERNS, GOLD_STANDARDS, CONTRIBUTING, SECURITY, CODE_OF_CONDUCT, CHANGELOG, .gitignore, plan.toml); ai_docs/INDEX, ai_specs/INDEX, ai_specs/MODULE_MATRIX, ultramap/README, src/README, tests/README, benches/README, .claude/settings.json | **LIVE** (22 files) |
| Wave 1 | 8 parallel cluster-spec-author agents (one per cluster A-H); 26 per-module god-tier Rust specs written to `ai_specs/modules/cluster-{X}/m<N>_<name>.md`; ~70k words total | **LIVE** (26 files; HOLD-v2 compliant; no `.rs`) |
| Wave 2 | 5 parallel infrastructure agents: (2A) `.claude/` JSON optimisation 28 files, (2B) ai_docs deep authoring 11 files ~19k words, (2C) ai_specs cross-cutting + layers + synergies 33 files ~29k words (CC-1b resolved as `CC-1.subA`), (2D) ultramap deep authoring 13 files 16 Mermaid diagrams, (2E) Obsidian vault sync + remaining gold-standard 30 files (16 vault + 14 repo) | **LIVE** (115 files) |
| Wave 3 | **agent-claim-verifier**: PASS-WITH-AMENDMENTS (20/20 hard checks PASS; 3 cosmetic — bottom-anchor on 11 specs, heading-form variance across 3 forms, CI regression slot); confidence 0.94. Receipt at `~/projects/shared-context/agent-cross-talk/2026-05-17T064906Z_agent-claim-verifier_workflow_trace_wave1_2_verification.md`. **four-surface-persistence-verifier**: PARTIAL (Surfaces 1+2 strong; Surface 3 correctly reserved pre-G8; Surface 4 anchor added concurrent with verifier run via CLAUDE.local.md edit); 5 gaps including stale workspace-CLAUDE.local.md row (project-charter-forbidden to amend; Luke action required) + new ADR `2026-05-17-g8-stcortex-persistence-plan.md` authored to close gap #3. **na-gap-analyst**: Frame A (substrate-as-primary) chosen; 11 NA gaps surfaced at [`ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md); ~8h Wave 4 remediation recommended before G9. | **LIVE — PASS-WITH-AMENDMENTS** |
| Wave 4 | **5 Luke S1002127 decisions** applied: D-A m23 PrefixSpan **Pure Rust**, D-B m1 page_size **2_000**, D-C m10 Ember §5.1 **Hybrid CI-FAIL+allowlist**, D-D m11 **dual-read soak**, D-E m13 threshold **>0.015** (W4.0 agent). **EscapeSurfaceProfile cardinality 6 → 7** via new variant **`PrivilegeEscalation`** at ordinal 30 (W4.A agent; ~12 file amendments + new ADR `2026-05-17-escape-surface-cardinality-7-privilege-escalation.md`; D-S1002127-02). **NA-GAP-01..11 substrate-as-primary remediation** ~8h (W4.B agent; new `ai_specs/substrates/` with 8 dossiers + 2 cross-cutting specs `refusal-taxonomy.md` + `substrate-drift.md` + 3-5 surgical amendments). | IN PROGRESS |

**Scaffold totals across Waves 0+1+2:** ~163 files / ~140k words. NO `.rs` source files. NO `Cargo.toml`. HOLD-v2 envelope respected throughout. G9 `start coding workflow-trace` NOT fired.

This row updates as scaffold work lands.

---

> **Back to:** [`README.md`](README.md) · [`CLAUDE.md`](CLAUDE.md) · [`CLAUDE.local.md`](CLAUDE.local.md) · [`GATE_STATE.md`](GATE_STATE.md)
