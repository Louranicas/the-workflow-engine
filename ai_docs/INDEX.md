# ai_docs/INDEX — workflow-trace documentation map

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)
> **Sister index:** [`../ai_specs/INDEX.md`](../ai_specs/INDEX.md) (per-module Rust specs) · [`../ultramap/README.md`](../ultramap/README.md) (flow maps)

---

## What lives here

`ai_docs/` carries **descriptive** documentation — what the system IS, how it WORKS at runtime, how to OPERATE it. Compare with `ai_specs/` which carries **prescriptive** specs (what each module MUST be).

---

## Operator + developer docs (post-G9 — LIVE)

> Authored during the S1003733 assessment-driven remediation. These are the
> day-to-day operator/developer surface for the implemented 26-module codebase.

| Doc | Purpose | Status |
|---|---|---|
| [`../QUICKSTART.md`](../QUICKSTART.md) | 5-minute developer quickstart — build, run both binaries, read the gate | **LIVE** |
| [`../docs/DIAGNOSTICS.md`](../docs/DIAGNOSTICS.md) | God-tier diagnostics + troubleshooting — 4-stage gate, `wf-crystallise` / `wf-dispatch` symptoms, external-service matrix, m8 build warnings, logging, build/env issues | **LIVE** |
| [`../docs/COMMAND_MAPPING.md`](../docs/COMMAND_MAPPING.md) | `wf-crystallise` / `wf-dispatch` CLI flag reference + command mapping | **LIVE** |
| [`API_MAP.md`](API_MAP.md) | Public-API surface across `wf-crystallise` + `wf-dispatch` + `workflow_core` lib (the authoritative `pub use` inventory) | **LIVE** |
| [`HARDENING_FLEET_2026-05-21.md`](HARDENING_FLEET_2026-05-21.md) | Hardening Fleet remediation record — W0–W5 waves, baseline, results, S1003733 resolution (m8 KEEP-DORMANT, W3 type-design portfolio) | **LIVE** |
| [`HARDENING_W2_FINDINGS.md`](HARDENING_W2_FINDINGS.md) | Wave-2 security findings detail (19 findings) | **LIVE** |
| [`HARDENING_W3_TYPE_DESIGN_PORTFOLIO.md`](HARDENING_W3_TYPE_DESIGN_PORTFOLIO.md) | Wave-3 type-design portfolio (`#[non_exhaustive]`, newtype encapsulation) | **LIVE** |

## Root-level docs (descriptive reference set — LIVE)

| Doc | Purpose | Status |
|---|---|---|
| [`ARCHITECTURE_DEEP_DIVE.md`](ARCHITECTURE_DEEP_DIVE.md) | Cross-cluster topology, message flows, state machines | **LIVE** |
| [`CODE_MODULE_MAP.md`](CODE_MODULE_MAP.md) | All 26 modules with public exports, dependencies, hot paths | **LIVE** (descriptive companion to [`API_MAP.md`](API_MAP.md)) |
| [`DEPLOYMENT_GUIDE.md`](DEPLOYMENT_GUIDE.md) | End-to-end deploy recipe (synthesis of vault deployment framework) | **LIVE** |
| [`ERROR_TAXONOMY.md`](ERROR_TAXONOMY.md) | thiserror taxonomy across clusters | **LIVE** |
| [`MESSAGE_FLOWS.md`](MESSAGE_FLOWS.md) | Cross-cluster + cross-service message envelopes | **LIVE** |
| [`META_TREE_MIND_MAP.md`](META_TREE_MIND_MAP.md) | Concept tree | **LIVE** |
| [`ONBOARDING.md`](ONBOARDING.md) | New contributor cold-start | **LIVE** |
| [`PERFORMANCE.md`](PERFORMANCE.md) | Hot-path performance budgets, benchmarks, profiling guide | **LIVE** |
| [`STATE_MACHINES.md`](STATE_MACHINES.md) | Sunset lifecycle, dispatch flow, verifier state machines | **LIVE** |
| [`QUICKSTART.md`](QUICKSTART.md) | Quickstart (ai_docs copy; root-level [`../QUICKSTART.md`](../QUICKSTART.md) is the operator-facing canonical) | **LIVE** |
| [`CARGO_LAYOUT_SPEC.md`](CARGO_LAYOUT_SPEC.md) | Workspace structure spec (historical — superseded by the real `Cargo.toml` post-G9) | Superseded |
| [`GENESIS_PROMPT_V1_3.md`](GENESIS_PROMPT_V1_3.md) | Binding spec v1.3 amendment | **LIVE** |
| [`CONVENTIONAL_GAP_ANALYSIS_S1002209.md`](CONVENTIONAL_GAP_ANALYSIS_S1002209.md) | Conventional-frame gap analysis (G6) | **LIVE** |
| [`NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) | Non-anthropocentric (Frame-A) gap analysis (G6) | **LIVE** |
| [`HARDENING_FLEET_CARRY_FORWARD_S1002600.md`](HARDENING_FLEET_CARRY_FORWARD_S1002600.md) · [`MUTATION_TEST_REPORT_S1002600.md`](MUTATION_TEST_REPORT_S1002600.md) | Carry-forward register + mutation-test report (S1002600) | **LIVE** |

## Subdirs

| Dir | Contents | Status |
|---|---|---|
| [`layers/`](layers/) | Per-cluster docs | Seeded |
| [`modules/`](modules/) | Per-module operational docs (mirror of `ai_specs/modules/`) | Empty — use [`ai_specs/modules/`](../ai_specs/modules/) + [`CODE_MODULE_MAP.md`](CODE_MODULE_MAP.md) + [`API_MAP.md`](API_MAP.md) |
| [`decisions/`](decisions/) | ADRs — `2026-05-17-substrate-as-actor-deferrals.md` (D-S1002127-03); see also [`optimisation-v7/decisions/`](optimisation-v7/decisions/) | Seeded; growing |
| [`schematics/`](schematics/) | Mermaid diagrams (per cluster + cross-cluster) | Seeded |
| [`runbooks/`](runbooks/) | Operational runbooks (per phase) — mirror of `optimisation-v7/RUNBOOKS/` | Seeded |
| [`reflections/`](reflections/) | Session reflections (S1001982+) | Seeded |
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

## Wave-16 — workflow-trace as habitat service (S1005032, 2026-05-25)

- [`WAVE_16_WF_DAEMON_DESIGN_S1005032.md`](WAVE_16_WF_DAEMON_DESIGN_S1005032.md) — `wf-daemon` design: habitat-service shape (`/health` on `:8142` + embedded `wf-poller` subsystem); port 8141→8142 re-port story (collision with HABITAT-CONDUCTOR); convention compliance; honest residuals; cross-refs to HTTP spec + lifecycle ultramap.

## Vault canonical (for source-of-truth on planning artefacts)

- [`the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md`](../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md) — 11 parts state-of-world synthesis
- [`the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md`](../the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md) — canonical end-to-end deployment recipe
- [`the-workflow-engine-vault/module specs/`](../the-workflow-engine-vault/module%20specs/) — 8 cluster-level planning specs (41,508 words)
- [`the-workflow-engine-vault/deployment framework/`](../the-workflow-engine-vault/deployment%20framework/) — 10 phase docs

---

> **Back to:** [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../GATE_STATE.md`](../GATE_STATE.md) · sister [`../ai_specs/INDEX.md`](../ai_specs/INDEX.md)
