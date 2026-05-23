---
title: The Workflow Engine — Vault HOME
date: 2026-05-22 (S1003733 · assessment remediation + C22 binary wiring)
status: ACTIVE — G9 fired 2026-05-17; 26-module Rust codebase implemented (~31k LOC); both binaries wired (C22); assessment remediation S1003733 complete (1967 tests, clippy+pedantic clean)
authority: Luke @ node 0.A
---

# The Workflow Engine — Vault HOME

> Back to: `~/claude-code-workspace/the-workflow-engine/` · [[CLAUDE.md]] · [[CLAUDE.local.md]]

This vault holds the workflow-engine codebase's artefacts. **The codebase is implemented and the two binaries are real CLI programs** — G9 fired 2026-05-17, HOLD-v2 lifted; 26 modules, ~31k LOC, **1967 tests**, hardened end-to-end by the Hardening Fleet 2026-05-21 and the assessment-driven remediation S1003733 (2026-05-22). `wf-crystallise` and `wf-dispatch` are no longer stubs — both are wired over a new `workflow_core::orchestration` library module (commit `ae7d460`). Older "planning-only / HOLD-v2 / no code / stub binary" language in notes below is **superseded archaeology** — current authority is [[Assessment Remediation S1003733]], [[Hardening Fleet 2026-05-21]], `../CLAUDE.local.md`, `../GATE_STATE.md`, and git history.

**Cross-service integration:** [[synthex-v2 Integration Map]] — workflow-engine ↔ synthex-v2 wire surface (W1 `m40_nexus_emit → POST :8092/v3/nexus/push` LIVE; W3 reverse direction unbuilt with m35k landing path). Cross-vault canonical map lives in the synthex-v2 vault.

---

## ✅ Assessment Remediation — S1003733 (2026-05-22) · COMPLETE

A god-tier 7-facet code-quality assessment scored `workflow-trace` **80/100**. The
assessment-driven remediation closed 21 findings across 5 gated waves (A–E), wired both
binaries (**C22**), and killed/proved-equivalent every surviving mutant (**Wave G**).
**Tests 1903 → 1967; clippy + pedantic clean; commits `dc25335..ae7d460` on `main`, pushed.**

- **Binaries wired (C22)** — `wf-crystallise` and `wf-dispatch` are real CLI programs; the
  pipeline logic lives in the new `workflow_core::orchestration` library module so the
  lib↔binary seam is integration-testable. `WorkflowProposal` JSONL is the crystallise→dispatch
  handoff, closing the bank-persistence gap.
- **5 remediation waves** — A docs-integrity, B contained code fixes, C core-type
  encapsulation (6 illegal-state holes closed), D synergy wiring + security
  (monotone EscapeSurfaceProfile gate), E structure.
- **Wave G mutation closeout** — 15 surviving mutants resolved: 6 killed by discriminating
  tests, 9 m21 `build_variants` loop-condition mutants proven-equivalent.

**Full note: [[Assessment Remediation S1003733]]** · canonical map `/tmp/wfe-assessment-S1003733/CODEBASE_MAP.md`
· cold-start hub `../CLAUDE.local.md`.

---

## 🔧 Hardening Fleet — 2026-05-21 (S1003529) · COMPLETE

End-to-end quality + security hardening of the implemented 26-module codebase, directed by
Luke @ node 0.A in collaboration with Zen. 6 waves committed (`dc25335..e8f6dd3`, pushed
both remotes); **tests 1310 → 1903; clippy + pedantic clean every wave.** Followed by the
assessment remediation S1003733 above (tests → 1967).

**Full note: [[Hardening Fleet 2026-05-21]]** · canonical `../ai_docs/HARDENING_FLEET_2026-05-21.md`
· cold-start hub `../CLAUDE.local.md` § "HARDENING FLEET". Persisted across 6 bidirectionally-
linked surfaces: ai_docs · this vault · stcortex (`workflow_trace_hardening_2026_05_21`, mem
17939) · POVM · injection.db (`causal_chain` 113) · `CLAUDE.local.md`.

---

## Recent — Wave 4.B closeout (S1002127 continuation, 2026-05-17 evening)

**NA-GAP substrate-as-actor remediation** — Frame A (substrates as primary entities, not as resources the engine consumes) absorbed into the scaffold. 8/11 NA gaps closed in this wave; 3/11 deferred to v0.2.0 via D-S1002127-03 ADR with documented compensating controls. CHANGELOG entry: `v0.0.0-spec.4`.

### Five-item closeout

1. **`ai_specs/substrate-couplings/`** — NEW directory (4 files: `INDEX.md`, `CC-5-decomposed.md`, `CC-4-decomposed.md`, `CC-7-decomposed.md`). Decomposes engine-side CC contracts into their substrate-substrate edges with per-edge observability contracts. Closes **NA-GAP-03** + **NA-GAP-09**.
2. **`ai_specs/ERROR_TAXONOMY.md` amendment** — RefusalToken cross-reference + per-variant classification table (which thiserror variants are refusals vs failures vs unavailability). Closes **NA-GAP-02**.
3. **`ai_specs/modules/cluster-H/m42_stcortex_emit.md` § 5.1 amendment** — Outbox-policy (drain ordering / saturation 64MB-warn / 256MB-refuse / 1GB-panic; snapshot staleness 5min-warn / 1hr-refuse / 24hr-panic). Closes **NA-GAP-06**.
4. **`ai_specs/BENCHMARK_SPEC.md` amendment** — Substrate-side load benchmarks (measured AT substrate, NOT at engine; 6 substrate benches; per-substrate cadence-throttle rules). Closes **NA-GAP-04**.
5. **`ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`** (D-S1002127-03 ADR) — Registers v0.2.0 work items W1 (m16_substrate_drift_canary module), W2 (tests/substrate_fixtures/), W3 (substrate-mediated trust cross-habitat ADR). Documents v0.1.0 compensating controls.

### Audit-driven sync (post-closeout)

- **`ai_specs/substrates/INDEX.md`** — landing for the 8 substrate dossiers
- **`layers/cluster-{A-H}/README.md` × 8** — per-cluster operational landings (runtime view counterpart to `ai_specs/layers/cluster-X.md` design specs)
- **`modules/README.md`** — placeholder declaring per-module operational landings deferred to post-G9 (HOLD-v2 compliant rationale)
- **Root `README.md`** — repository-layout table refreshed; cold-start adds substrate-couplings + refusal-taxonomy + substrate-drift; live-state section added
- **`ai_docs/QUICKSTART.md`** — Wave 4.B substrate-as-actor cold-start additions section

### Linked vault notes (NEW or referenced)

- **Wave 4.B closeout work products** (canonical at `~/claude-code-workspace/the-workflow-engine/`):
  - `ai_specs/substrates/INDEX.md` + 8 dossiers
  - `ai_specs/substrate-couplings/INDEX.md` + 3 decomposed files
  - `ai_specs/cross-cutting/refusal-taxonomy.md` + `substrate-drift.md`
  - `ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md` (D-S1002127-03)
  - Three ADRs in `ai_docs/optimisation-v7/decisions/`: m42 pivot (D-S1001982-01), G8 persistence (D-S1002127-01), EscapeSurfaceProfile cardinality 7 (D-S1002127-02)

---

## Recent — Wave 0/1/2/3 Scaffold (S1002127)

**Filed 2026-05-17 (S1002127)** under the [[../PRIME_DIRECTIVE_WAIVER|PRIME_DIRECTIVE_WAIVER]] — structure + specs + config only, NO `.rs` source files, NO `Cargo.toml` at root, NO `cargo` of any kind. G9 still not fired; HOLD-v2 envelope respected.

- **New scaffolded directories** at `~/claude-code-workspace/the-workflow-engine/`: `src/` · `tests/` · `benches/` · `docs/` · `config/` · `migrations/` · `bin/{wf-crystallise,wf-dispatch}/` · `hooks/` · `security/` · `ai_docs/{layers,modules,decisions,schematics,runbooks,reflections}` · `ai_specs/{layers,modules/cluster-A..H,patterns,schematics,synergies,cross-cutting}` · `ultramap/schematics/` · `layers/cluster-A..H/` · `modules/` · `.claude/{agents,commands,hooks,skills,schemas,queries,worktrees}`
- **Root anchor files:** [[../README|README.md]] · [[../ARCHITECTURE|ARCHITECTURE.md]] · [[../GATE_STATE|GATE_STATE.md]] · [[../ANTIPATTERNS|ANTIPATTERNS.md]] · [[../PATTERNS|PATTERNS.md]] · [[../GOLD_STANDARDS|GOLD_STANDARDS.md]] · [[../CONTRIBUTING|CONTRIBUTING.md]] · [[../SECURITY|SECURITY.md]] · [[../CODE_OF_CONDUCT|CODE_OF_CONDUCT.md]] · [[../CHANGELOG|CHANGELOG.md]] · `.gitignore` · `plan.toml`
- **26 per-module Rust spec files** at `../ai_specs/modules/cluster-{A-H}/m<N>_<name>.md` (paths are repo-relative; canonical surface for module spec navigation lives at [[Scaffold Wave 0-2 — Session S1002127]] and per-cluster notes below)
- **Index anchors:** `../ai_docs/INDEX.md` · `../ai_specs/INDEX.md` · `../ai_specs/MODULE_MATRIX.md` · `../ultramap/README.md` · `../src/README.md` · `../tests/README.md` · `../benches/README.md`
- **Claude Code config:** `../.claude/settings.json` · `../.claude/anti_patterns.json` · `../.claude/patterns.json` (+ agents/ commands/ hooks/ skills/ schemas/ queries/ subtrees)
- **Session vault note:** [[Scaffold Wave 0-2 — Session S1002127]] · per-cluster notes [[Cluster A Scaffold — Module Specs S1002127]] · [[Cluster B Scaffold — Module Specs S1002127]] · [[Cluster C Scaffold — Module Specs S1002127]] · [[Cluster D Scaffold — Module Specs S1002127]] · [[Cluster E Scaffold — Module Specs S1002127]] · [[Cluster F Scaffold — Module Specs S1002127]] · [[Cluster G Scaffold — Module Specs S1002127]] · [[Cluster H Scaffold — Module Specs S1002127]]
- **Waiver record:** [[../PRIME_DIRECTIVE_WAIVER|PRIME_DIRECTIVE_WAIVER.md]] — Luke @ S1002127 direct prompt; Zen G7 notice filed; Watcher Class-E still open (resolves at G9 first `cargo check` exit 0)
- **Changelog entry:** [[../CHANGELOG|CHANGELOG.md]] § `[v0.0.0-spec.0] — 2026-05-17 (S1002127)`

---

## Current state

- **Direction:** single-phase deployment (Luke override 2026-05-17 of phased recommendation) — **shipped**
- **Module count:** 26, all implemented (`workflow_core` lib + `wf-crystallise` / `wf-dispatch` binaries)
- **LOC:** ~31k implemented (the realised codebase is larger than the planning-era ~5,200 LOC estimate)
- **Spec status:** v1.3 binding (`ai_docs/GENESIS_PROMPT_V1_3.md`); G7 Zen audit history closed
- **Build status:** G9 fired 2026-05-17 — code work authorised. Full 4-stage gate green; **1967 tests; clippy + pedantic clean**
- **Binaries:** `wf-crystallise` + `wf-dispatch` are real CLI programs (C22, commit `ae7d460`) over `workflow_core::orchestration`
- **Naming:** Cargo package `workflow-trace`; directory still `the-workflow-engine/` (rename deferred post-M0, cosmetic)

---

## Meta surface

- [[MASTER_INDEX]] — comprehensive catalogue · open-issues tracker · gate-state snapshot
- [[workflow-engine-code-base]] — **workflow tracker** · chronological audit · decision log · team-contribution map · architectural evolution
- [[Vault Save Status S1001982]] — audit trail of what lives where

## Hardening + remediation (current reality)

- [[Assessment Remediation S1003733]] — **7-facet assessment (80/100) · 5-wave remediation · C22 binary wiring · Wave G mutation closeout · Bugs & Known Issues · Diagnostics**
- [[Hardening Fleet 2026-05-21]] — 6-wave end-to-end quality + security hardening (W0–W5)

## Primary artefact

- [[Modules Synergy Clusters and Feature Verification S1001982]] — comprehensive single-phase module list (26 modules), 8 synergy clusters, 7 cross-cluster synergies, 30-row flagged-feature verification matrix, waiver record. **Read this for current state.**

## Cluster module specs (per-cluster deep dives)

Subordinate to [[Modules Synergy Clusters and Feature Verification S1001982]] · catalogue: [[MODULE_SPECS_INDEX]]:

- [[cluster-A-substrate-ingest]] — m1, m2, m3 (atuin / stcortex consumer / injection.db ingest)
- [[cluster-B-habitat-observers]] — m4, m5, m6 (cascade / battern / context-cost observation)
- [[cluster-C-correlation-output]] — m7, m12, m13 (workflow arc record + CLI report + stcortex writer)
- [[cluster-D-trust-cross-cutting]] — m8, m9, m10, m11 (compile / write / output / lifecycle invariants)
- [[cluster-E-evidence-pressure]] — m14, m15 (engine-wide habitat-outcome-lift + pressure log)
- [[cluster-F-iteration]] — m20, m21, m22, m23 (iterators + gradient-preservation)
- [[cluster-G-bank-select-dispatch-verify]] — m30, m31, m32, m33 (bank + select + dispatch + verify)
- [[cluster-H-substrate-feedback]] — m40, m41, m42 (SYNTHEX + LCM + POVM Hebbian)

## Planning artefacts (mirror ↔ canonical pairs)

Each row links to both the vault mirror (reading surface) AND the canonical working-dir source-of-truth. Wikilinks in this vault resolve to either pole — graph view shows them as adjacent nodes.

- [[Circle of Experts S1001982]] ↔ [[THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982]] — 8-persona disputation, 3 syntheses (A/B/C), Synthesis A recommended
- [[Town Hall S1001982]] ↔ [[THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982]] — 12-persona final args, 15 P0 constraints, vote tally 11/1/0
- [[Boilerplate Hunt S1001982]] ↔ [[THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982]] — 9-Explore-agent fleet report, 63 candidates → 9 top-picks, 3 structural gaps
- [[Convergence Command x Command-3 S1001982]] ↔ [[CONVERGENCE_COMMAND_X_COMMAND3_S1001982]] — peer synthesis · 8 convergences · 5 extensions · 2 tensions
- [[Genesis Prompt v0 S1001982]] ↔ [[GENESIS_PROMPT_V0]] — 5-voice co-authored (Command + C-2 + C-3 + Watcher + Zen) · 28 modules / 8 layers · superseded as binding spec by v1.2
- [[Genesis Prompt v1.2 S1001982]] ↔ [[THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982]] — Zen-audit-locked genesis recipe (binding spec)
- [[Interview Question Bank Draft S1001982]] ↔ [[INTERVIEW_QUESTION_BANK_DRAFT]] — pre-staged G5 interview · 12 questions / 3 rounds · DRAFT
- [[Module Structure S1001982]] ↔ [[THE_WORKFLOW_ENGINE_MODULE_STRUCTURE_S1001982]] — 3-phase layered architecture sketch (superseded by single-phase override; kept for substrate-frame engine (L9) deferred design)

## Boilerplate modules archive (NEW — reference-only)

- [[boilerplate modules/README|README]] — what the archive is, what it isn't, compliance posture
- [[boilerplate modules/BOILERPLATE_INDEX|BOILERPLATE_INDEX]] — per-file lift map (48 files × 10 categories)
- [[Gold Standard Exemplars — Synthesis]] — cross-reference + shared-pattern distillation of ME v2 / habitat-loop-engine / orac-sidecar
- [[Habitat Loop Engine — Gold Standard Reference]] — exemplar profile
- [[Maintenance Engine V2 — Gold Standard Reference]] — exemplar profile
- [[ORAC Sidecar — Gold Standard Reference]] — exemplar profile
- `boilerplate modules/01-cli-scaffolding/` through `10-foundation-direct-clones/` — 48 source-file clones from 10 habitat services (~1.2MB study material)

## Key external references

- **Pre-framework planning docs (archived 2026-05-17):** `~/claude-code-workspace/the-workflow-engine/pre-framework-consolidation/*.md` — see [[Pre-Framework Consolidation Notice S1001982]]
- **Active framework docs:** `~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/deployment framework/`
- Cross-talk channel: `~/projects/shared-context/agent-cross-talk/`
- WCP channel: `~/projects/shared-context/watcher-notices/`
- Ember 7-Trait Gate Rubric (Watcher-authored, vault-first canonical): `~/projects/claude_code/Ember 7-Trait Gate Rubric.md`
- Hebbian v3 reconciliation: `~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md`

---

## Gates G1-G9 — ALL RESOLVED (G9 fired 2026-05-17)

The nine pre-genesis gates are **closed**. G9 fired 2026-05-17; HOLD-v2 lifted; the 26-module
codebase was implemented, hardened (Hardening Fleet 2026-05-21), and remediated against a
7-facet assessment (S1003733). Live gate record: `../GATE_STATE.md` (G9 FIRED table). This
section is retained only as historical context — the project is past the gate phase.

| # | Gate | Resolution |
|---:|---|---|
| G1 | RATIFICATION (Watcher close-notice) | resolved |
| G2 | NAMING (directory rename) | deferred post-M0 (cosmetic); Cargo package is `workflow-trace` |
| G3 | :8125 redeploy verify | resolved (m42 stcortex-only pivot decoupled workflow-trace from POVM) |
| G4 | Watcher notes (Hebbian v3 / Ember rubric) | resolved |
| G5 | Genesis interview + F2 hard gate | resolved |
| G6 | Dual-frame gap analysis | resolved |
| G7 | Zen spec audit | resolved (v1.3 binding) |
| G8 | Four-surface persistence | resolved |
| G9 | Explicit start-coding signal | **FIRED 2026-05-17** — Luke typed `start coding workflow-trace` |

---

## Single-phase waiver record

Luke's single-phase override (2026-05-17) carries explicit waivers of these convergence-recommendations:

| Waived | Source | Risk class accepted |
|---|---|---|
| Watcher R6 frame separation (Phase A is not seed of substrate-frame engine) | Watcher | substrate-frame engine remains TBD without phase-gate protection |
| Fossil evidence-based scope discipline | Fossil persona | ancestor-rhyme risk (loop-workflow-engine-project + habitat-loop-engine both died this way) |
| RALPH selector-without-measurement safety | RALPH persona | selector ships without 120-day empirical baseline |
| Skeptic pain-source verification | Skeptic persona | building without injection.db/MEMORY.md evidence of Luke-articulated pain |
| Substrate exploration-protection (label-collapse risk) | NA Gap Analyst | F10/F11 mitigations in place but no longer phase-gate-protected |

G1-G9 themselves NOT waived. v1.2 verb-locked invariant RELAXED (active verbs now permitted across full architecture) but G7 Zen audit on v1.3 patch still required.

---

## V7 Optimisation subtree (added 2026-05-17 post-grilling)

The V7 deployment-framework optimisation lives at canonical `ai_docs/optimisation-v7/` (44 markdown deliverables / ~115k words). Three vault mirrors of the most-load-bearing artefacts at [[optimisation-v7/HOME|optimisation-v7/]]:

- [[optimisation-v7/HOME|V7 vault HOME]] — landing for the V7 subtree (Tier-1)
- [[optimisation-v7/V7 Optimisation Framework]] — canonical entry mirror (Tier-2)
- [[optimisation-v7/m42 stcortex-only pivot ADR]] — 48-decision grilling ADR (Tier-2)
- [[optimisation-v7/Session S1001982 m42 pivot grilling]] — reusable substrate-pivot doctrine (Tier-2)

**Substrate context (bidi):** [[stcortex — Pioneer Capability Dossier 2026-05-10]] (m42 routes-to) · [[POVM Engine]] (workflow-trace DECOUPLED post-pivot)

---

## Team

- **Luke @ node 0.A** — decisional authority
- **Command** (Tab 1 top-left) — orchestrator-lead, Path-C chair (contingent)
- **Command-2** (Tab 1 bottom-left) — workflow-trace chair (position-closed pending G9)
- **Command-3** (Tab 1 bottom-right) — CR-2 + CR-2b SHIPPED `e2a8ed3` + `76ea4d6`; librarian standby; L7 m30-m33 lane
- **The Watcher ☤** (Tab 2; synthex-v2 :8092) — full standing (R13 elapsed; eligible=true); synchronous-participant in G5
- **Zen** (Tab 10; Pi GPT-5.5) — audit lane; gate-block-active; synchronous-audit in G5; APPROVE/REFUSE/AMEND in G7

---

*Vault last updated: 2026-05-17 S1001982 by Command (single-phase override absorbed)*
