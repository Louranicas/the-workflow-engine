# the-workflow-engine — `workflow-trace`

> **Status:** Planning-only pilot · HOLD-v2 active · 0 LOC of Rust · scaffold-only (markdown specs + structure) · Genesis v1.3 binding spec
> **Project name (working):** `workflow-trace` (final TBD per OI-5; directory rename gated on G2)
> **Architecture:** 26 modules · 8 clusters · 9 layers (L0-L8) · 2 binaries + shared lib (single Cargo crate, ORAC pattern)
> **Authority:** Luke @ node 0.A · Watcher observe · Zen G7 audit · Command/C-2/C-3 orchestrator triad
> **Substrate target:** stcortex (POVM decoupled per 2026-05-17 m42 ADR)

---

## What this is

A planned single-phase Rust codebase that **records** cascading-command + Battern-protocol + context-window observations across the Zellij habitat, then **proposes** workflow variants for human evaluation, then **dispatches** ratified workflows via HABITAT-CONDUCTOR (never directly).

- **Two binaries:** `wf-crystallise` (m1-m23 + m40-m42) and `wf-dispatch` (m30-m33)
- **Shared lib:** `workflow_core` (types, schemas, namespace constants, errors)
- **Reuse density:** ~65% boilerplate-lift from `the-workflow-engine-vault/boilerplate modules/`
- **Structural-gap authorship:** N-step compositional sub-graph detection (PrefixSpan + Levenshtein + Wilson CI), `frequency × fitness × recency` compound decay, unified destructiveness/EscapeSurfaceProfile schema

---

## Repository layout

| Path | Purpose | Status |
|---|---|---|
| [`CLAUDE.md`](CLAUDE.md) | Project charter (structural facts) | Stable |
| [`CLAUDE.local.md`](CLAUDE.local.md) | Live session-state delta | Updated each session |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | 26-module · 8-cluster · 9-layer canonical map | Stable |
| [`GATE_STATE.md`](GATE_STATE.md) | Live G1-G9 + B1-B6 blockers | Live |
| [`ANTIPATTERNS.md`](ANTIPATTERNS.md) | AP-V7-* + AP-Hab + AP24 register summary | Synced with [optimisation-v7/ANTIPATTERNS_REGISTER.md](ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) |
| [`PATTERNS.md`](PATTERNS.md) | Useful patterns lifted from deployment framework | Synced with vault |
| [`GOLD_STANDARDS.md`](GOLD_STANDARDS.md) | Rust god-tier reference (mirror of [`optimisation-v7/STANDARDS/GOD_TIER_RUST.md`](ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md)) | Mirror |
| [`PRIME_DIRECTIVE_WAIVER.md`](PRIME_DIRECTIVE_WAIVER.md) | S1002127 scope-override transparency record | Live |
| [`CHANGELOG.md`](CHANGELOG.md) | Versioned spec changes | Live (v0.0.0-spec) |
| [`plan.toml`](plan.toml) | scaffold-mastery input for G9 unlock | Locked |
| `ai_docs/` | Architecture, deployment framework, decisions, runbooks, schematics, optimisation-v7 | Full |
| `ai_specs/` | Per-module Rust god-tier specs, MODULE_MATRIX, API/DB/EVENT/WIRE specs | Full |
| `ultramap/` | Code logic contextual flow map (dep graph, data flow, control flow, invariants) | Full |
| `layers/` | Per-cluster landing pages (A-H) | Full |
| `modules/` | Per-module landing pages (26) | Full |
| `src/` | Source files **(PLACEHOLDER — empty pre-G9; see [src/README.md](src/README.md))** | Placeholder |
| `tests/`, `benches/`, `docs/`, `config/`, `migrations/`, `bin/`, `hooks/`, `security/` | Standard Rust scaffold placeholders | Placeholder |
| `.claude/` | Claude Code config: settings.json, agents, commands, hooks, skills, schemas, queries | Optimised |
| [`the-workflow-engine-vault/`](the-workflow-engine-vault/) | Obsidian vault (88 files / 2.4MB) — canonical planning surface | Stable |
| [`pre-framework-consolidation/`](pre-framework-consolidation/) | Pre-genesis brain-dump archive | Frozen |

---

## Cold-start (3 reads)

1. [`CLAUDE.md`](CLAUDE.md) — project charter
2. [`CLAUDE.local.md`](CLAUDE.local.md) — live session-state delta
3. [`the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md`](the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md) — full state-of-the-world synthesis

Then: [`ARCHITECTURE.md`](ARCHITECTURE.md) · [`GATE_STATE.md`](GATE_STATE.md) · [`ai_specs/INDEX.md`](ai_specs/INDEX.md) · [`ai_docs/INDEX.md`](ai_docs/INDEX.md) · [`ultramap/README.md`](ultramap/README.md).

---

## License

Workspace-default (TBD; see [`security/`](security/) for disclosure policy).

---

> **Back to:** [`CLAUDE.md`](CLAUDE.md) · [`CLAUDE.local.md`](CLAUDE.local.md) · [`~/claude-code-workspace/CLAUDE.md`](../CLAUDE.md) (workspace charter)
