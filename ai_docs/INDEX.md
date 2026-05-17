# ai_docs/INDEX — workflow-trace documentation map

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)
> **Sister index:** [`../ai_specs/INDEX.md`](../ai_specs/INDEX.md) (per-module Rust specs) · [`../ultramap/README.md`](../ultramap/README.md) (flow maps)

---

## What lives here

`ai_docs/` carries **descriptive** documentation — what the system IS, how it WORKS at runtime, how to OPERATE it. Compare with `ai_specs/` which carries **prescriptive** specs (what each module MUST be).

---

## Root-level docs (Wave-2 author target — TBD)

| Doc | Purpose | Status |
|---|---|---|
| `ARCHITECTURE_DEEP_DIVE.md` | Cross-cluster topology, message flows, state machines (dev-ops-engine-v3 mirror) | TBD Wave 2 |
| `CODE_MODULE_MAP.md` | All 26 modules with public exports, dependencies, hot paths | TBD Wave 2 |
| `DEPLOYMENT_GUIDE.md` | End-to-end deploy recipe (synthesis of vault deployment framework) | TBD Wave 2 |
| `ERROR_TAXONOMY.md` | thiserror taxonomy across clusters | TBD Wave 2 |
| `MESSAGE_FLOWS.md` | Cross-cluster + cross-service message envelopes | TBD Wave 2 |
| `META_TREE_MIND_MAP.md` | Concept tree (dev-ops-engine-v3 mirror) | TBD Wave 2 |
| `ONBOARDING.md` | New contributor cold-start (post-G9) | TBD Wave 2 |
| `PERFORMANCE.md` | Hot-path performance budgets, benchmarks, profiling guide | TBD Wave 2 |
| `QUICKSTART.md` | 5-minute developer quickstart | TBD Wave 2 |
| `STATE_MACHINES.md` | Sunset lifecycle, dispatch flow, verifier state machines | TBD Wave 2 |
| `CARGO_LAYOUT_SPEC.md` | Workspace structure spec (until G9; replaces real Cargo.toml) | TBD Wave 2 |
| `GENESIS_PROMPT_V1_3.md` | Binding spec v1.3 amendment | **LIVE** |

## Subdirs

| Dir | Contents | Status |
|---|---|---|
| [`layers/`](layers/) | Per-cluster docs (8 files) | TBD Wave 2 |
| [`modules/`](modules/) | Per-module operational docs (26 files) | TBD Wave 2 (mirror of `ai_specs/modules/`) |
| [`decisions/`](decisions/) | ADRs (single-phase override, m42 pivot, …) | Seeded; growing |
| [`schematics/`](schematics/) | Mermaid diagrams (per cluster + cross-cluster) | TBD Wave 2 |
| [`runbooks/`](runbooks/) | Operational runbooks (per phase) | Mirror to `optimisation-v7/RUNBOOKS/` |
| [`reflections/`](reflections/) | Session reflections (S1001982+) | TBD as work lands |
| [`optimisation-v7/`](optimisation-v7/) | V7 framework (45 deliverables; canonical for many docs) | **LIVE** |

---

## Canonical V7 framework (already live in `optimisation-v7/`)

| File | Role |
|---|---|
| [`OPTIMISATION_FRAMEWORK_V7_FINAL.md`](optimisation-v7/OPTIMISATION_FRAMEWORK_V7_FINAL.md) | Table of contents for all 44 V7 deliverables |
| [`KEYWORDS_20.md`](optimisation-v7/KEYWORDS_20.md) | 20 keyword anchors |
| [`ULTRAMAP.md`](optimisation-v7/ULTRAMAP.md) | View 1 (layer) + View 2 (module table) — **the canonical structural map** |
| [`ANTIPATTERNS_REGISTER.md`](optimisation-v7/ANTIPATTERNS_REGISTER.md) | Canonical antipattern register |
| [`DECISION_REGISTER.md`](optimisation-v7/DECISION_REGISTER.md) | 61 decisions (13 V7 + 48 grilling) |
| [`HANDSHAKE_PROTOCOL_TAB1.md`](optimisation-v7/HANDSHAKE_PROTOCOL_TAB1.md) | Tab 1 orchestrator triad comms |
| [`AGENT_VIEW_GITWORKTREES.md`](optimisation-v7/AGENT_VIEW_GITWORKTREES.md) | Agent View + worktrees protocol |
| [`TASK_LIST_V7_OPTIMISATION.md`](optimisation-v7/TASK_LIST_V7_OPTIMISATION.md) | V7 author task list |
| [`VERIFICATION_T0.md`](optimisation-v7/VERIFICATION_T0.md) | Watcher T0 baseline + 3 yellow signals |

### `optimisation-v7/MODULE_PLANS/` (cluster-level Rust planning specs — newer than vault module specs)

[`cluster-A.md`](optimisation-v7/MODULE_PLANS/cluster-A.md) · [`cluster-B.md`](optimisation-v7/MODULE_PLANS/cluster-B.md) · [`cluster-C.md`](optimisation-v7/MODULE_PLANS/cluster-C.md) · [`cluster-D.md`](optimisation-v7/MODULE_PLANS/cluster-D.md) · [`cluster-E.md`](optimisation-v7/MODULE_PLANS/cluster-E.md) · [`cluster-F.md`](optimisation-v7/MODULE_PLANS/cluster-F.md) · [`cluster-G.md`](optimisation-v7/MODULE_PLANS/cluster-G.md) · [`cluster-H.md`](optimisation-v7/MODULE_PLANS/cluster-H.md) · [`CROSS_CLUSTER_SYNERGIES.md`](optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md)

### `optimisation-v7/GENERATIONS/` (G1-G7 V7 generation history)

[`G1-baseline-audit.md`](optimisation-v7/GENERATIONS/G1-baseline-audit.md) · [`G2-consolidation.md`](optimisation-v7/GENERATIONS/G2-consolidation.md) (canonical src/ layout) · [`G3-bidi-flow.md`](optimisation-v7/GENERATIONS/G3-bidi-flow.md) · [`G4-gold-standard.md`](optimisation-v7/GENERATIONS/G4-gold-standard.md) · [`G5-tooling.md`](optimisation-v7/GENERATIONS/G5-tooling.md) · [`G6-test-discipline.md`](optimisation-v7/GENERATIONS/G6-test-discipline.md) · [`G7-final-synthesis.md`](optimisation-v7/GENERATIONS/G7-final-synthesis.md)

### `optimisation-v7/STANDARDS/`

[`GOD_TIER_RUST.md`](optimisation-v7/STANDARDS/GOD_TIER_RUST.md) · [`TEST_DISCIPLINE.md`](optimisation-v7/STANDARDS/TEST_DISCIPLINE.md)

### `optimisation-v7/RUNBOOKS/` (Phase 0-8 + cross-cutting + emergency rollback)

[`runbook-00-pre-genesis-gates.md`](optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md) · [`runbook-01-phase-1-genesis.md`](optimisation-v7/RUNBOOKS/runbook-01-phase-1-genesis.md) · [`runbook-02-phase-2A-measure-only.md`](optimisation-v7/RUNBOOKS/runbook-02-phase-2A-measure-only.md) · [`runbook-03-phase-2B-active.md`](optimisation-v7/RUNBOOKS/runbook-03-phase-2B-active.md) · [`runbook-04-phase-3-integration.md`](optimisation-v7/RUNBOOKS/runbook-04-phase-3-integration.md) · [`runbook-05-phase-4-hardening.md`](optimisation-v7/RUNBOOKS/runbook-05-phase-4-hardening.md) · [`runbook-06-phase-5-deploy-soak.md`](optimisation-v7/RUNBOOKS/runbook-06-phase-5-deploy-soak.md) · [`runbook-07-phase-6-sunset.md`](optimisation-v7/RUNBOOKS/runbook-07-phase-6-sunset.md) · [`runbook-08-phase-7-security.md`](optimisation-v7/RUNBOOKS/runbook-08-phase-7-security.md) · [`runbook-09-phase-8-observability.md`](optimisation-v7/RUNBOOKS/runbook-09-phase-8-observability.md) · [`runbook-10-cross-cutting.md`](optimisation-v7/RUNBOOKS/runbook-10-cross-cutting.md) · [`runbook-11-emergency-rollback.md`](optimisation-v7/RUNBOOKS/runbook-11-emergency-rollback.md)

### `optimisation-v7/INTEGRATION/` (per-service integration)

[`devops-v3-integration.md`](optimisation-v7/INTEGRATION/devops-v3-integration.md) · [`scaffold-integration.md`](optimisation-v7/INTEGRATION/scaffold-integration.md) · [`atuin-integration.md`](optimisation-v7/INTEGRATION/atuin-integration.md) · [`codesynthor-v8-integration.md`](optimisation-v7/INTEGRATION/codesynthor-v8-integration.md) · [`progressive-disclosure-obsidian.md`](optimisation-v7/INTEGRATION/progressive-disclosure-obsidian.md) · [`json-claude-code-optimisation.md`](optimisation-v7/INTEGRATION/json-claude-code-optimisation.md)

### `optimisation-v7/decisions/` (ADRs)

[`2026-05-17-m42-stcortex-only-pivot.md`](optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md)

---

## Vault canonical (for source-of-truth on planning artefacts)

- [`the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md`](../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md) — 11 parts state-of-world synthesis
- [`the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md`](../the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md) — canonical end-to-end deployment recipe
- [`the-workflow-engine-vault/module specs/`](../the-workflow-engine-vault/module%20specs/) — 8 cluster-level planning specs (41,508 words)
- [`the-workflow-engine-vault/deployment framework/`](../the-workflow-engine-vault/deployment%20framework/) — 10 phase docs

---

> **Back to:** [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../GATE_STATE.md`](../GATE_STATE.md) · sister [`../ai_specs/INDEX.md`](../ai_specs/INDEX.md)
