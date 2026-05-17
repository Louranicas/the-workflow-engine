# the-workflow-engine — `workflow-trace`

> **Status:** Planning-only pilot · HOLD-v2 active · **0 LOC of Rust** · scaffold-only (markdown specs + structure + .claude config) · Genesis v1.3 binding spec · Wave 4.B closeout LIVE
> **Project name (working):** `workflow-trace` (final TBD per OI-5; directory rename gated on G2)
> **Architecture:** 26 modules · 8 clusters · 9 layers (L0-L8) · 2 binaries + shared lib (single Cargo crate, ORAC pattern)
> **Authority:** Luke @ node 0.A · Watcher observe (R13 eligible · 48,723 obs) · Zen G7 audit · Command/C-2/C-3 orchestrator triad
> **Substrate target:** stcortex (POVM-decoupled per 2026-05-17 m42 ADR · D-S1001982-01)
> **Cross-cutting frames:** anthropocentric module-primary (default) + substrates-as-actors (Frame A per NA gap analysis Wave 4.B)

---

## What this is

A planned single-phase Rust codebase that **records** cascading-command + Battern-protocol + context-window observations across the Zellij habitat, then **proposes** workflow variants for human evaluation, then **dispatches** ratified workflows via HABITAT-CONDUCTOR (never directly).

- **Two binaries:** `wf-crystallise` (m1-m23 + m40-m42) and `wf-dispatch` (m30-m33)
- **Shared lib:** `workflow_core` (types, schemas, namespace constants, errors)
- **Reuse density:** ~65% boilerplate-lift from `the-workflow-engine-vault/boilerplate modules/`
- **Structural-gap authorship:** N-step compositional sub-graph detection (PrefixSpan + Levenshtein + Wilson CI), `frequency × fitness × recency` compound decay, unified destructiveness / EscapeSurfaceProfile schema (7-variant per D-S1002127-02)

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
| [`CHANGELOG.md`](CHANGELOG.md) | Versioned spec changes | Live (`v0.0.0-spec.4` Wave 4.B closeout) |
| [`plan.toml`](plan.toml) | scaffold-mastery input for G9 unlock; declares 4 persistence surfaces | Locked |
| [`ai_docs/`](ai_docs/) | Architecture, deployment framework, decisions, runbooks, schematics, optimisation-v7, QUICKSTART, NA gap analysis | Full (63 files) |
| [`ai_docs/decisions/`](ai_docs/decisions/) | NA-remediation ADRs (D-S1002127-03 substrate-as-actor deferrals) | Live |
| [`ai_docs/optimisation-v7/decisions/`](ai_docs/optimisation-v7/decisions/) | V7-authored ADRs (m42 pivot D-S1001982-01, G8 persistence D-S1002127-01, EscapeSurfaceProfile cardinality-7 D-S1002127-02) | Live |
| [`ai_specs/`](ai_specs/) | Per-module Rust god-tier specs, MODULE_MATRIX, API/DB/EVENT/WIRE specs, cross-cutting, synergies | Full (75 files) |
| [`ai_specs/modules/cluster-{A-H}/`](ai_specs/modules/) | 26 per-module prescriptive specs | Full |
| [`ai_specs/layers/`](ai_specs/layers/) | 8 per-cluster design specs | Full |
| [`ai_specs/synergies/`](ai_specs/synergies/) | 7 cross-cluster engine-side contracts (CC-1..7 + CC-1.subA) | Full |
| [`ai_specs/cross-cutting/`](ai_specs/cross-cutting/) | 7 cross-cutting axes (error-handling, concurrency, persistence, observability, feature-gating, **refusal-taxonomy**, **substrate-drift**) | Full |
| [`ai_specs/substrates/`](ai_specs/substrates/) | 8 substrate-as-actor dossiers (S-A..S-G + S-watcher) — Frame A per NA-GAP remediation | Full (Wave 4 + 4.B) |
| [`ai_specs/substrate-couplings/`](ai_specs/substrate-couplings/) | 4 substrate-substrate edge decompositions (CC-5, CC-4, CC-7) — closes NA-GAP-03 / NA-GAP-09 | Full (Wave 4.B) |
| [`ultramap/`](ultramap/) | Code logic contextual flow map (dep graph, data flow, control flow, contextual flow, invariants, master) | Full (14 files / 16 Mermaid diagrams) |
| [`layers/`](layers/) | Per-cluster operational landing pages (8 sub-dirs, README each) — runtime view counterpart to `ai_specs/layers/` | Full (Wave 4.B audit) |
| [`modules/`](modules/) | Per-module operational landing pages (reserved for post-G9; placeholder README documents the intent) | Placeholder (HOLD-v2; post-G9 author-pass) |
| [`src/`](src/) | Source files **(PLACEHOLDER — empty pre-G9; see [`src/README.md`](src/README.md))** | Placeholder |
| [`tests/`](tests/), [`benches/`](benches/), [`docs/`](docs/), [`scripts/`](scripts/), [`migrations/`](migrations/), [`hooks/`](hooks/), [`security/`](security/) | Standard Rust scaffold placeholders (README each) | Placeholder |
| [`bin/wf-crystallise/`](bin/wf-crystallise/), [`bin/wf-dispatch/`](bin/wf-dispatch/) | Two-binary split per-binary landings | Placeholder (README each; no Cargo.toml pre-G9) |
| [`config/`](config/) | Config templates (default, production, devenv-service) | Templates |
| [`.claude/`](.claude/) | Claude Code config: settings.json, agents, commands, hooks, skills, schemas, queries | Optimised (29 files) |
| [`the-workflow-engine-vault/`](the-workflow-engine-vault/) | Obsidian vault (103 files) — canonical planning surface; bidi anchors to ai_docs / ai_specs | Stable (Wave 4.B mirrored) |
| [`pre-framework-consolidation/`](pre-framework-consolidation/) | Pre-genesis brain-dump archive | Frozen |

---

## Cold-start (3 reads + 2 deep-dives)

### Quickstart path (5 minutes)

1. [`CLAUDE.md`](CLAUDE.md) — project charter (structural facts; planning-only; HOLD-v2; 26 modules)
2. [`CLAUDE.local.md`](CLAUDE.local.md) — live session-state delta (6 blockers; In-flight; cold-start pointer)
3. [`ai_docs/QUICKSTART.md`](ai_docs/QUICKSTART.md) — 5-minute pre-G9 + post-G9 paths

### Deep-dive path (when you need the full architecture)

4. [`the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md`](the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md) — 9-agent fleet synthesis of all 77 vault files; 11 parts + 2 appendices (~7,000 words)
5. [`ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) — Frame A (substrates-as-actors) dual-pass analysis; 11 NA gaps; 8/11 closed via Wave 4.B; 3/11 deferred to v0.2.0

### Then by topic

| Topic | Entry point |
|---|---|
| **Architecture** | [`ARCHITECTURE.md`](ARCHITECTURE.md) → [`ai_specs/INDEX.md`](ai_specs/INDEX.md) → [`ai_specs/MODULE_MATRIX.md`](ai_specs/MODULE_MATRIX.md) |
| **Gates / blockers** | [`GATE_STATE.md`](GATE_STATE.md) |
| **Module specs (prescriptive)** | [`ai_specs/modules/cluster-{A-H}/`](ai_specs/modules/) — 26 files |
| **Cluster design specs** | [`ai_specs/layers/cluster-{A-H}.md`](ai_specs/layers/) — 8 files |
| **Cluster operational landings** | [`layers/cluster-{A-H}/README.md`](layers/) — 8 files (runtime view) |
| **Cross-cluster contracts (engine-side)** | [`ai_specs/synergies/`](ai_specs/synergies/) — CC-1..7 + CC-1.subA |
| **Substrate dossiers (Frame A)** | [`ai_specs/substrates/INDEX.md`](ai_specs/substrates/INDEX.md) → 8 dossiers (S-A..S-G + S-watcher) |
| **Substrate-substrate couplings** | [`ai_specs/substrate-couplings/INDEX.md`](ai_specs/substrate-couplings/INDEX.md) → CC-5 / CC-4 / CC-7 decomposed |
| **Refusal-token taxonomy** | [`ai_specs/cross-cutting/refusal-taxonomy.md`](ai_specs/cross-cutting/refusal-taxonomy.md) |
| **Substrate-drift detection** | [`ai_specs/cross-cutting/substrate-drift.md`](ai_specs/cross-cutting/substrate-drift.md) |
| **Antipattern catalogue** | [`ANTIPATTERNS.md`](ANTIPATTERNS.md) + [`ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md`](ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) (canonical) |
| **Useful patterns** | [`PATTERNS.md`](PATTERNS.md) |
| **God-tier Rust standards** | [`GOLD_STANDARDS.md`](GOLD_STANDARDS.md) + [`ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md`](ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md) |
| **Decision register** | [`ai_docs/optimisation-v7/DECISION_REGISTER.md`](ai_docs/optimisation-v7/DECISION_REGISTER.md) + ADRs in [`ai_docs/decisions/`](ai_docs/decisions/) + [`ai_docs/optimisation-v7/decisions/`](ai_docs/optimisation-v7/decisions/) |
| **Runtime flow maps** | [`ultramap/README.md`](ultramap/README.md) → 5 flow docs (DATA / CONTROL / CONTEXTUAL / INVARIANT / dep graph) + master ULTRAMAP |
| **Operational quickstart** | [`ai_docs/QUICKSTART.md`](ai_docs/QUICKSTART.md) (pre-G9 + post-G9 paths) |
| **Vault home** | [`the-workflow-engine-vault/HOME.md`](the-workflow-engine-vault/HOME.md) → [`MASTER_INDEX.md`](the-workflow-engine-vault/MASTER_INDEX.md) |

---

## State at this README revision (2026-05-17 · post-Wave-4.B-closeout)

### Live invariants

- **HOLD-v2 envelope:** 0 `.rs` / 0 `Cargo.toml` in active scope; markdown specs only
- **Spec version:** `v0.0.0-spec.4` (Wave 4.B closeout — NA-GAP substrate-as-actor remediation: 8/11 closed, 3/11 deferred to v0.2.0)
- **Binding spec:** v1.3 amendment (single-phase override + m42 stcortex-only pivot); awaits Zen G7 verdict on AUDIT-REQUEST v3 (per D-B6 AMEND-loop)
- **EscapeSurfaceProfile:** 7-variant ordinal locked (`PrivilegeEscalation` at ord 30 per D-S1002127-02)
- **G9:** BLOCKED; Luke types `start coding workflow-trace` to unlock; Cluster D ships Day 1 (m8→m9→m10→m11) per non-negotiable framework
- **Substrate health (last probe):** 11/12 healthy at session-start; RALPH fitness 0.6987 trending up; `substrate_LTP_density 0.018` PASSING Phase 1
- **Substrate target:** stcortex `:3000`; POVM `:8125` DECOUPLED per m42 ADR

### Active blockers (Luke physical actions remaining — 3, per [`GATE_STATE.md`](GATE_STATE.md) Luke actions)

1. **B1 / B2:** Luke green-lights v1.3 patch authoring OR drives G1-G8 sequence to clear G7 Zen URGENT block
2. **B3:** Luke @ terminal: `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start weaver/zen/enforcer` (Conductor wave panes 1B/1C/2/3 currently `auto_start=false`)
3. **Wake C-2 + C-3 panes** (5 silent handshakes filed; receive-mode v2 standing)

### Resolved this Wave 4.B

- ✅ NA-GAP-01, 04, 05, 10 (partial) — 8 substrate dossiers including operator-as-substrate
- ✅ NA-GAP-02, 11 — RefusalToken introduction (refusal-taxonomy.md + ERROR_TAXONOMY.md amendment)
- ✅ NA-GAP-03, 09 — substrate-couplings/ directory (4 files: CC-5, CC-4, CC-7 decomposed + INDEX)
- ✅ NA-GAP-04, 06 — m42 § 5.1 outbox-policy + BENCHMARK_SPEC.md substrate-side load benchmarks
- ✅ NA-GAP-07 (cross-cutting half) — substrate-drift.md canary contract; m16 module deferred to v0.2.0 W1 per D-S1002127-03
- ⏳ NA-GAP-08, 10 — deferred to v0.2.0 W2, W3 per D-S1002127-03 ADR with compensating controls

### Four-surface persistence status

Per workspace [`CLAUDE.md`](../CLAUDE.md) Working Mode + [`plan.toml`](plan.toml) `[scaffold_meta].four_surfaces`:

1. **`ai_docs/` canonical** — LIVE (63 files; ADRs in `decisions/` and `optimisation-v7/decisions/`)
2. **Obsidian vault mirror** — LIVE (103 files; pre-existing 88 + Wave 0/1/2/3/4 additions)
3. **stcortex `workflow_trace_*` namespace** — RESERVED pre-G8 per [G8 persistence ADR](ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md); writes fire mechanically on G8-green
4. **`CLAUDE.local.md` anchor** — LIVE (this directory + workspace-root row pending Luke amendment)

---

## Genesis / build order on G9 fire

When Luke types `start coding workflow-trace`, the implementation order is fixed:

| Day | Cluster(s) | Modules | Concern |
|---|---|---|---|
| **Day 1** | D (ship-first) | m8 → m9 → m10 → m11 | Trust scaffolding; namespace guard live before any substrate touch |
| **Day 2** | A | m1, m3, m2 | Read-side substrate ingest |
| **Day 3** | B + C (parallel) | m4, m5, m6, m7, m13, m12 | Observers + persistence + writer + reports |
| **Day 4** | E | m15, m14 | Pressure + lift evidence |
| **Day 5-7** | F (KEYSTONE) | m20 → m22 → m21 → m23 | Iteration engine; structural-gap authorship |
| **Day 8-10** | G | m30 → m31 → m33 → m32 | Bank + select + verify + dispatch |
| **Day 11-12** | H | m40, m41, m42 | Substrate feedback (CC-5 emit) |
| **Day 13+** | integration | CC-* closure tests; soak | Cross-cluster contract verification; live-substrate integration |

Per-cluster operational landings: [`layers/cluster-{A-H}/README.md`](layers/) — each carries the cluster's runtime concerns + substrate-side discipline + build-order detail.

---

## License

Workspace-default (TBD — see [`security/`](security/) for disclosure policy).

---

> **Back to:** [`CLAUDE.md`](CLAUDE.md) · [`CLAUDE.local.md`](CLAUDE.local.md) · [`ai_docs/QUICKSTART.md`](ai_docs/QUICKSTART.md) · [`ai_docs/INDEX.md`](ai_docs/INDEX.md) · [`ai_specs/INDEX.md`](ai_specs/INDEX.md) · [`the-workflow-engine-vault/HOME.md`](the-workflow-engine-vault/HOME.md) · [`~/claude-code-workspace/CLAUDE.md`](../CLAUDE.md) (workspace charter)

*README last updated 2026-05-17 (S1002127 · Wave 4.B closeout + audit-driven cross-surface sync). Repository layout claims verified against `find`/`ls` for accuracy.*
