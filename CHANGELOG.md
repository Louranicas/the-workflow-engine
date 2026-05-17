# CHANGELOG — workflow-trace

All notable changes to the spec, structure, and decisions for `workflow-trace` are recorded here. Versioning is **spec-versioned** pre-G9 (no Cargo SemVer until first commit + `cargo check` green).

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) but versions track binding-spec revisions, not code.

---

## [Unreleased]

### Pending (Wave 4 candidates — post-S1002127)
- ai_docs/INDEX + ai_specs/INDEX status markers `TBD Wave 2` → `LIVE` (cosmetic)
- ai_specs/INDEX heading-form variance documentation (3 canonical forms: `## N.` / `## N —` / `## §N`)
- Vault HOME.md wikilinks to Wave-2B deep docs (`ARCHITECTURE_DEEP_DIVE`, `CODE_MODULE_MAP`, `CARGO_LAYOUT_SPEC`, etc.) — currently bidi anchor present in HOME but no per-doc wikilinks
- Cluster scaffold vault notes (A-H) → ai_specs/modules/cluster-X/ back-links (currently one-way: vault→ai_specs missing)
- m32 cooldown ladder defaults table → back-propagate from ultramap schematic to m32 spec § 2 DispatcherConfig
- m11 compound decay worked-examples → back-propagate to m11 spec test fixtures
- m42 CC-5 closure-test 5-step ritual → back-propagate to m42 spec § tests
- Cluster D Day-1 gantt → back-propagate to V7 runbook-01 Phase-1 Genesis
- NA-GAP-01..11 remediation (Frame-A substrate-as-primary; ~8h work; **HIGH-VALUE pre-G9** per na-gap-analyst)
  - Consider authoring `ai_specs/substrates/{atuin,stcortex,injection_db,synthex,lcm,conductor,watcher,operator}.md` with lifecycle / refusal / drift models
  - Introduce first-class `RefusalToken` taxonomy distinguishing substrate-authored vs engine-authored refusal
  - Add substrate fixtures to `tests/` planning (post-G9)
  - Make m32 refusals engine-emitted (Class-C wire events), not Watcher-inferred from absence
  - Document operator-as-substrate dynamics (consent-fatigue cap, attention budget)
- agent-claim-verifier checks 6 + 16 in CI regression slot (post-G9)
- Bottom-anchor decision on 11 specs (Cluster B/C/E/F missing trailing `Back to:`) — Command accepted top-anchor-sufficient; can re-author if Luke disagrees
- **Workspace-root CLAUDE.local.md "The Workflow Engine" row amendment** — flagged stale by 4-surface verifier; project charter forbids; **Luke action required**

### Pending (binding-spec gating; not scaffold scope)
- Luke `start coding workflow-trace` (G9) — gated on G1-G8 sequence
- Zen G7 verdict on v1.3 amendment + this scaffold (AUDIT-REQUEST v2 filed 2026-05-17T160500Z)
- B4 Ember §5.1 Held-semantics amendment (Watcher's lane; awaits Luke direction)
- B3 Conductor `auto_start=false` (Luke @ terminal `devenv start weaver/zen/enforcer`)

---

## [v0.0.0-spec.3] — 2026-05-17 (S1002127 — Wave 3 verification + closure)

### Added
- **Wave 3 verifier reports landing** — 3 parallel agents:
  - `agent-claim-verifier` — **PASS-WITH-AMENDMENTS** (20/20 hard checks PASS; 3 cosmetic; confidence 0.94); receipt at `~/projects/shared-context/agent-cross-talk/2026-05-17T064906Z_agent-claim-verifier_workflow_trace_wave1_2_verification.md` (cross-talk: `broadcast: clean_verified`)
  - `four-surface-persistence-verifier` — **PARTIAL** (Surfaces 1+2 strong; Surface 3 correctly reserved pre-G8; Surface 4 anchor added concurrent with verifier via CLAUDE.local.md edit); 5 gaps surfaced
  - `na-gap-analyst` — **Frame A (substrate-as-primary)** chosen; 11 NA gaps at [`ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md); ~8h Wave 4 remediation recommended **HIGH-VALUE pre-G9**
- **New ADR D-S1002127-01** — [`ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md`](ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md) — pre-specifies Surface 3 writes for G8-fire (~46 memories + ~60 pathways under `workflow_trace_*` namespace with reverse-anchor embedding rule); closes four-surface-verifier gap #3
- **plan.toml `[scaffold_meta].four_surfaces`** array — machine-readable enumeration of the 4 persistence surfaces (closes four-surface-verifier gap #4)
- **Project CLAUDE.local.md** S1002127 scaffold section added (closes four-surface-verifier gap #1 via concurrent edit)

### Resolved
- **PRIME_DIRECTIVE_WAIVER.md Wave 3 row** — IN PROGRESS → **LIVE — PASS-WITH-AMENDMENTS**
- Template-variance heading-form decision: ACCEPT all 3 forms (`## N.` / `## N —` / `## §N`) as canonical; document in `ai_specs/INDEX.md` (Wave 4 cosmetic)
- Bottom-anchor decision: ACCEPT top-anchor-sufficient (round-trip works without trailing `Back to:` on 11 affected specs)
- HOLD-v2 verified intact: 0 `.rs` files in active scope; 0 `Cargo.toml`; 38 `.rs` files in `the-workflow-engine-vault/boilerplate modules/` are intentional paste-templates (not code tree)

### Flagged for Luke
- **Workspace-root CLAUDE.local.md "The Workflow Engine" row stale** — project charter forbids workspace-CLAUDE.local edits for workflow-trace spec; Luke must amend manually OR grant explicit waiver
- **NA-GAP-01..11 (substrate-as-primary frame)** — author NA gap analysis recommends ~8h Wave 4 remediation before G9; substrate-naive risk = CR-2-class shift lands via Luke spot-check rather than engine detection
- **EscapeSurfaceProfile cardinality drift** (5 vs 6) — V7 GOD_TIER_RUST.md amendment recommended
- **Test budget drift** (1,562 / 1,594 / 1,599) — TEST_STRATEGY locked at 1,594
- 5 cluster-spec open questions to G7 (m1 page_size, m23 PrefixSpan implementation, m10 Ember §5.1, m11 re-calibration, m13 LTP/LTD scale)

---

## [v0.0.0-spec.2] — 2026-05-17 (S1002127 — Wave 2)

### Added
- **`.claude/` deep optimisation (28 files)** — `anti_patterns.json` (24 entries: AP-V7-01..13 + AP24/27/29/30 + AP32-37), `patterns.json` (35 entries), `context.json`, `status.json`, `ALIGNMENT_VERIFICATION.md`, 6 project-specific subagents (`agents/`), 6 project slash commands (`commands/`), 4 executable hooks (`hooks/`), 4 JSON schemas (`schemas/`), 3 canned SQL queries (`queries/`)
- **`ai_docs/` deep authoring (11 files, ~19k words)** — `ARCHITECTURE_DEEP_DIVE`, `CODE_MODULE_MAP`, `DEPLOYMENT_GUIDE`, `ERROR_TAXONOMY`, `MESSAGE_FLOWS`, `META_TREE_MIND_MAP`, `ONBOARDING`, `PERFORMANCE`, `QUICKSTART`, `STATE_MACHINES`, `CARGO_LAYOUT_SPEC`
- **`ai_specs/` cross-cutting + layers + synergies (33 files, ~29k words)** — 8 cluster-level layer specs (`layers/cluster-{A-H}.md`); 12 cross-cutting specs (API/DATABASE/EVENT/WIRE/IPC/DESIGN_CONSTRAINTS/CONSENT/SECURITY/ERROR_TAXONOMY/OBSERVABILITY/TEST_STRATEGY/BENCHMARK); 7 synergy contracts + README (`synergies/CC-{1-7}.md` + README — **CC-1b resolved as `CC-1.subA` sub-contract**); 5 cross-cutting axis specs (`cross-cutting/`)
- **`ultramap/` deep authoring (13 files, 16 Mermaid diagrams)** — `MODULE_DEPENDENCY_GRAPH`, `DATA_FLOW`, `CONTROL_FLOW`, `CONTEXTUAL_FLOW`, `INVARIANT_MAP`, `ULTRAMAP` master synthesis; 7 schematics (`cc4-pipeline`, `cc5-loop`, `m32-5check`, `cluster-d-day1`, `gap{1,2,3}-*`)
- **Obsidian vault sync (16 file changes)** — 6 audited (`> Back to:` anchors include `[[CLAUDE.md]] · [[CLAUDE.local.md]]`), 2 updated (HOME.md + MASTER_INDEX.md additions), 9 new (`Scaffold Wave 0-2 — Session S1002127.md` + 8 per-cluster scaffold notes)
- **Remaining gold-standard (14 files)** — `LICENSE` (placeholder, TBD), 8 placeholder dir READMEs (docs/config/scripts/migrations/bin/hooks/security/schematics), 2 per-binary READMEs (wf-crystallise/wf-dispatch), 3 config templates (default/production/devenv-service)

### Resolved
- **CC-1b reconciliation:** documented as `CC-1.subA` sub-contract in `synergies/CC-1.md` (preserves canonical 7-CC list discipline; AP-V7-02 Ultramap-rot avoided)

### Flagged
- **EscapeSurfaceProfile cardinality drift** — V7 GOD_TIER_RUST.md invariant #19 says 5, v1.3 + m30 spec say 6 (DataExfil added for openclaw scar tissue). Documented in `ai_specs/DESIGN_CONSTRAINTS.md` + `SECURITY_SPEC.md`. V7 amendment recommended.
- **Test budget drift** — V7 docs vary 1,562 / 1,594 / 1,599; `TEST_STRATEGY.md` locks at **1,594** per G6 latest matrix
- **`povm_calibrated` cfg name** — historical post-2026-05-17 m42 ADR; rename/retire deferred to post-G9 spec revision

## [v0.0.0-spec.1] — 2026-05-17 (S1002127 — Wave 1)

### Added
- **26 per-module god-tier Rust spec files** (~70k words, ~2,700 words/spec) written by 8 parallel cluster-spec-author agents
  - Cluster A (3): m1_atuin_consumer, m2_stcortex_consumer, m3_injection_db_consumer
  - Cluster B (3): m4_cascade_correlator, m5_battern_step_record, m6_context_cost
  - Cluster C (3): m7_workflow_runs, m12_cli_reports, m13_stcortex_writer
  - Cluster D (4): m8_povm_build_prereq, m9_watcher_namespace_guard, m10_ember_ci_gate, m11_fitness_weighted_decay (**Gap 2 owner**)
  - Cluster E (2): m14_habitat_outcome_lift, m15_pressure_register
  - Cluster F (4): m20_prefixspan_miner, m21_variant_builder, m22_kmeans_feature, m23_workflow_proposer (**KEYSTONE, Gap 1 owner**)
  - Cluster G (4): m30_curated_bank, m31_selector, m32_conductor_dispatcher, m33_verifier (**Gap 3 owner with m9**)
  - Cluster H (3): m40_nexusevent_emit, m41_lcm_rpc, m42_stcortex_emit (**POVM DECOUPLED per 2026-05-17 ADR**)
- Each spec: YAML frontmatter (14 fields) + 13-section body (Purpose/Public surface/Internal data/Data flow/Algorithm/Boilerplate lifts/ME v2 patterns/Test strategy/Antipatterns/Useful patterns/CC contracts/Open questions/Implementation order) + bidi anchors top+bottom

### Open questions to G7 / Luke / Watcher (consolidated; full per-spec lists in §12 of each)
- m1 page_size: V7 plan `1_000` vs vault `2_000` — needs Zen G7 reconciliation
- m23 PrefixSpan implementation: pure-Rust vs C-FFI vs Python-port (Cluster F agent recommends pure-Rust; **#1 G7 question**)
- m10 Ember §5.1 amendment (B4 blocker) — biggest Cluster D dependency
- m11 re-calibration post-m42 ADR (fitness signal POVM `learning_health` → stcortex `pathway.weight`); dual-read soak proposed
- m13 LTP/LTD scale reconciliation (vault `>0.15` vs workspace S1002127 `0.018`)
- EscapeSurfaceProfile ordinal stability across versions (m30/m32/m33 + m9) — reserve numeric gaps?

## [v0.0.0-spec.0] — 2026-05-17 (S1002127 — Wave 0)

### Added
- Scaffold-only scope-override waiver ([`PRIME_DIRECTIVE_WAIVER.md`](PRIME_DIRECTIVE_WAIVER.md)) — Luke @ S1002127
- Wave 0 scaffold skeleton: `src/`, `tests/`, `benches/`, `docs/`, `config/`, `migrations/`, `bin/{wf-crystallise,wf-dispatch}/`, `hooks/`, `security/`, `ai_docs/{layers,modules,decisions,schematics,runbooks,reflections}`, `ai_specs/{layers,modules/cluster-A..H,patterns,schematics,synergies,cross-cutting}`, `ultramap/schematics`, `layers/cluster-A..H`, `modules/`, `.claude/{agents,commands,hooks,skills,schemas,queries,worktrees}`
- Root anchor files: [`README.md`](README.md), [`ARCHITECTURE.md`](ARCHITECTURE.md), [`GATE_STATE.md`](GATE_STATE.md), [`ANTIPATTERNS.md`](ANTIPATTERNS.md), [`PATTERNS.md`](PATTERNS.md), [`GOLD_STANDARDS.md`](GOLD_STANDARDS.md), [`CONTRIBUTING.md`](CONTRIBUTING.md), [`SECURITY.md`](SECURITY.md), [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md), [`plan.toml`](plan.toml), [`.gitignore`](.gitignore)

### Pre-existing (carried over)
- [`CLAUDE.md`](CLAUDE.md) project charter
- [`CLAUDE.local.md`](CLAUDE.local.md) session-state delta
- [`the-workflow-engine-vault/`](the-workflow-engine-vault/) (88 files / 2.4MB)
- [`ai_docs/optimisation-v7/`](ai_docs/optimisation-v7/) (45 V7 deliverables: framework, generations G1-G7, integration, runbooks, standards, module plans, decisions, ultramap)
- [`ai_docs/GENESIS_PROMPT_V1_3.md`](ai_docs/GENESIS_PROMPT_V1_3.md) (binding spec v1.3 amendment; Zen G7 verdict pending)
- [`pre-framework-consolidation/`](pre-framework-consolidation/) brain-dump archive

---

## [v1.3-amendment] — 2026-05-17 (S1001982 → S1002127)

### Spec patch (binding; awaits Zen G7 verdict)
- Single-phase override absorbed (Luke override; D-B6 AMEND-loop adopted)
- m42 stcortex-only pivot ([ADR](ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md)) — POVM decoupled
- 26 modules locked (OI-3 resolved: was 28/11/25/26 across artefacts)
- m1-m42 unpadded naming locked (OI-4 resolved)
- m33 additive (`workflow_verifier`; required by Town Hall P0 #9)
- Two-binary split locked: `wf-crystallise` + `wf-dispatch` + `workflow_core` lib in same crate (ORAC pattern, not LCM workspace)
- Cluster D early-ship locked (Day 1; before any Cluster A reader)
- Feature gate matrix locked (default/full/api/intelligence/monitoring/evolution; D NOT gated)

---

## [v1.2] — 2026-05-15 (S1001982)

- Zen-audit-locked binding spec for 11-module Phase-A-only deployment (superseded by v1.3)

---

## [v1.1] — 2026-05-14
## [v1.0] — 2026-05-13
## [v0] — 2026-05-12 (Genesis Prompt v0 sketch)

> Earlier versions are in [`the-workflow-engine-vault/Genesis Prompt v0 S1001982.md`](the-workflow-engine-vault/Genesis%20Prompt%20v0%20S1001982.md) and [`the-workflow-engine-vault/Genesis Prompt v1.2 S1001982.md`](the-workflow-engine-vault/Genesis%20Prompt%20v1.2%20S1001982.md).

---

> **Back to:** [`README.md`](README.md) · [`PRIME_DIRECTIVE_WAIVER.md`](PRIME_DIRECTIVE_WAIVER.md) · [`GATE_STATE.md`](GATE_STATE.md)
