---
title: MASTER INDEX — the-workflow-engine vault
date: 2026-05-22 (S1003733 · assessment remediation + C22 binary wiring)
kind: index
status: ACTIVE — G9 fired; 26-module codebase implemented; both binaries wired (C22); 1967 tests, clippy+pedantic clean
---

# MASTER INDEX

> Back to: [[HOME]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]

Comprehensive catalogue of every artefact in the the-workflow-engine vault, organised by purpose.

> **Current reality (2026-05-24):** the project has **shipped v0.2.0** (substrate-safety
> milestone). G9 fired 2026-05-17; v0.1.0 / M0 tagged 2026-05-23 (`df00fd2`); v0.2.0 tagged
> 2026-05-24 (`5d92248`). 27 modules (m16 added in v0.2.0 Phase 9 KEYSTONE); both binaries
> (`wf-crystallise`, `wf-dispatch`) are real CLI programs after C22; **2163 tests, clippy +
> pedantic clean**, three stacked SemVer-breaks at v0.2.0 wire level (`WorkflowProposal` 6 →
> 12 fields). Any "planning-only / HOLD-v2 / stub binary" language remaining below is
> superseded archaeology. See [[Session S1004377 — v0.2.0 SHIPPED]], [[Workflow-Trace v0.2.0 Plan v2 S1004377]],
> [[Assessment Remediation S1003733]], and [[Hardening Fleet 2026-05-21]] for the live state.

---

## 0a. SYNTHEX-V2 wiring schematics + gap analysis + Plan v2 (S1004590, 2026-05-24)

- **[[Wiring Plan v2 — Source-Verified Integration S1004590]]** — **🟢 RATIFICATION-CANDIDATE.** Source-grounded synthesis of all 4 evidence streams. Read FIRST. Supersedes wiring schematics' headline claims; renders NA-1' REFUTED + escalates CONV-3/4 + adds NA-1''.
- [[Wiring Gap Analysis — S1004590 Dual-Frame]] — VERDICT: AMEND. Dual-pass conventional 6/10 verified + NA pass 4 HIGH. **Plan v2 supersedes the remediation roadmap in this note.**
- [[SYNTHEX-V2 Integration Master Schematic]] — umbrella; cross-vault to `synthex-v2/MASTER_INDEX`
- [[Wiring 01 — m16 Heartbeat Consumer (NA-4 Closure)]] — **highest leverage**; closes OP-6 / NA-4 self-canary loop
- [[Wiring 02 — NexusEvent Bidirectional Push]] — m40/m41/m42 outbound to `:8092`
- [[Wiring 03 — stcortex Pathway Namespace Alignment]] — `workflow_trace_*` prefix + cross-loop Hebbian pair discipline (P30)
- [[Wiring 04 — Watcher (m46–m51) Integration Hooks]] — optional; substrate-side Watcher proposal flow post-Wiring 01

> Source: `/tmp/synthex-v2-wiring-discovery-for-workflow-trace.md` (read-only Explore mission, 551 lines, 24-row wiring surface table)

---

## 0b. Current-state notes (read these first for live reality)

- [[Session S1004377 — v0.2.0 SHIPPED]] — **🟢 v0.2.0 SHIPPED 2026-05-24 — substrate-safety milestone · tag `v0.2.0` at `5d92248` · 2163 tests (+115 from v0.1.0) · 12 phases · 4 new modules + 1 test suite + 3 ADRs + Genesis v1.4 · 21 §15 decisions · 12 honest residuals · 6 operator hand-offs (OP-1..OP-6)**
- [[Workflow-Trace v0.2.0 Plan v2 S1004377]] — v0.2.0 Plan v2 RATIFIED 2026-05-23 (vault mirror; 25 gap-analysis findings folded; 21 Phase 4 interview decisions locked)
- [[Session S1004115 — v0.1.1 + v0.2.0 Prep Save]] — post-M0 hygiene + v0.2.0 prep save (2026-05-23)
- [[Session S1004115 — Completion Plan v2 Locked]] — v0.1.0 / M0 ship record (2026-05-23)
- [[Assessment Remediation S1003733]] — **7-facet assessment (80/100), 5-wave remediation, C22 binary wiring, Wave G mutation closeout, per-facet deltas, Bugs & Known Issues (all assessment findings CLOSED), Diagnostics, open follow-ups**
- [[Hardening Fleet 2026-05-21]] — 6-wave end-to-end quality + security hardening (W0–W5)

---

## 0. Start here

- [[HOME]] — landing page · gate state · team table · waiver record
- [[workflow-engine-code-base]] — **workflow tracker** · chronological audit · decision log · open issues
- [[Vault Save Status S1001982]] — what lives where (audit trail)

## 1. Current architecture (binding direction)

- [[Modules Synergy Clusters and Feature Verification S1001982]] — **the 26-module single-phase architecture · 8 clusters · 7 cross-cluster synergies · 30 features verified · waiver record**

## 2. Binding spec (current)

- [[Genesis Prompt v1.2 S1001982]] ↔ canonical [[THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982]] — Zen-audit-locked spec · v1.0 → v1.1 → v1.2 evolution · 9 pre-genesis gates · hard refusals · failure-mode table

> **Note:** v1.2 listed 11 modules (Phase A only). The single-phase override was absorbed
> into **v1.3** (`../ai_docs/GENESIS_PROMPT_V1_3.md`, 46K) — the binding spec for the realised
> 26-module architecture. v1.3 cleared its G7 Zen audit; G9 fired 2026-05-17 and the codebase
> is now implemented + hardened.

## 3. Deliberation artefacts (how we got here)

Chronological order — each row pairs vault mirror ↔ canonical working-dir doc:

1. [[Circle of Experts S1001982]] ↔ [[THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982]] — 8-persona disputation · 3 syntheses (A/B/C) · Synthesis A recommended
2. [[Town Hall S1001982]] ↔ [[THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982]] — 12-persona finals · 15 P0 constraints · vote tally 11/1/0
3. [[Boilerplate Hunt S1001982]] ↔ [[THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982]] — 9-Explore-fleet · 63 candidates · 9 top-picks · 3 structural gaps · Conductor blocker
4. [[Convergence Command x Command-3 S1001982]] ↔ [[CONVERGENCE_COMMAND_X_COMMAND3_S1001982]] — peer synthesis · 8 convergences · 5 extensions · 2 tensions
5. [[Module Structure S1001982]] ↔ [[THE_WORKFLOW_ENGINE_MODULE_STRUCTURE_S1001982]] — 9-layer model · 3-phase architecture (superseded by single-phase) · Phase C placeholder preserved
6. [[Modules Synergy Clusters and Feature Verification S1001982]] — single-phase 26-module architecture (current; vault-native, no canonical mirror)

## 4. Prompt versions

- [[Genesis Prompt v0 S1001982]] ↔ canonical [[GENESIS_PROMPT_V0]] — 5-voice co-authored (Command + C-2 + C-3 + Watcher + Zen) · 28 modules / 8 layers / ~5,600 LOC · **superseded as binding spec by v1.2**
- [[Genesis Prompt v1.2 S1001982]] ↔ canonical [[THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982]] — current binding spec (Command's Zen-audit-locked version)
- v1.3 — **pending** (single-phase Luke override absorption)

## 5. Interview / spec materials

- [[Interview Question Bank Draft S1001982]] ↔ canonical [[INTERVIEW_QUESTION_BANK_DRAFT]] — pre-staged G5 interview · 12 questions / 3 rounds · AskUserQuestion-compatible · DRAFT pre-Action-1 + Action-2

## 5b. Cluster module specs (per-cluster deep dives)

Subordinate to [[Modules Synergy Clusters and Feature Verification S1001982]] · catalogue: [[MODULE_SPECS_INDEX]]:

- [[cluster-A-substrate-ingest]] — m1, m2, m3 (atuin / stcortex consumer / injection.db ingest)
- [[cluster-B-habitat-observers]] — m4, m5, m6 (cascade / battern / context-cost)
- [[cluster-C-correlation-output]] — m7, m12, m13 (workflow arc record + CLI + stcortex writer)
- [[cluster-D-trust-cross-cutting]] — m8, m9, m10, m11 (compile / write / output / lifecycle invariants)
- [[cluster-E-evidence-pressure]] — m14, m15 (habitat-outcome-lift + pressure log)
- [[cluster-F-iteration]] — m20, m21, m22, m23 (iterators + gradient-preservation)
- [[cluster-G-bank-select-dispatch-verify]] — m30, m31, m32, m33 (bank + select + dispatch + verify)
- [[cluster-H-substrate-feedback]] — m40, m41, m42 (SYNTHEX + LCM + POVM Hebbian) — **m42 POVM-decoupled per 2026-05-17 D-S1001982-01 ADR**

## 5c. Wave 4 + 4.B — NA-GAP substrate-as-actor remediation (Frame A)

Per the NA gap analysis at `~/claude-code-workspace/the-workflow-engine/ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`, the scaffold's anthropocentric module-primary view was complemented by a Frame A (substrates-as-actors) pass in Wave 4 + Wave 4.B closeout. 8/11 NA gaps closed; 3/11 deferred to v0.2.0 via ADR D-S1002127-03.

### Substrate dossiers (Wave 4 — 8 substrate-as-actor profiles)

Canonical at `~/claude-code-workspace/the-workflow-engine/ai_specs/substrates/`:

- `ai_specs/substrates/INDEX.md` — landing + reading order
- `ai_specs/substrates/atuin.md` (S-A — shell-history SQLite)
- `ai_specs/substrates/injection_db.md` (S-B — causal-chain SQLite)
- `ai_specs/substrates/stcortex.md` (S-C — SpacetimeDB pioneer memory; CANONICAL substrate-drift case CR-2)
- `ai_specs/substrates/conductor.md` (S-D — HABITAT-CONDUCTOR enforcement)
- `ai_specs/substrates/synthex.md` (S-E — SYNTHEX v2 NexusEvent; Hebbian coordinator since S226)
- `ai_specs/substrates/lcm.md` (S-F — LCM loop create/cancel MCP)
- `ai_specs/substrates/watcher.md` (S-watcher — The Watcher ☤ persona; AP27-bounded)
- `ai_specs/substrates/operator.md` (S-G — operator-as-substrate per NA-GAP-05)

### Substrate-substrate couplings (Wave 4.B — 4 decomposition files)

Canonical at `~/claude-code-workspace/the-workflow-engine/ai_specs/substrate-couplings/`:

- `ai_specs/substrate-couplings/INDEX.md` — verification-discipline pattern + substrate-confirmable-receipt convention
- `ai_specs/substrate-couplings/CC-5-decomposed.md` — 5 substrate-substrate edges in the substrate learning loop (PRIMARY; closes NA-GAP-03 + NA-GAP-09)
- `ai_specs/substrate-couplings/CC-4-decomposed.md` — 3 edges (m32→S-D Conductor dispatch + m30→S-G operator AP-V7-07; AP-V7-13 enrichment)
- `ai_specs/substrate-couplings/CC-7-decomposed.md` — 4 edges (operator-as-substrate pressure → spec amendment fanout → S-watcher Ember gate → fatigue feedback)

### Cross-cutting NA-remediation specs

Canonical at `~/claude-code-workspace/the-workflow-engine/ai_specs/cross-cutting/`:

- `ai_specs/cross-cutting/refusal-taxonomy.md` — `RefusalToken` taxonomy (SubstrateAuthored / EngineAuthored / OperatorRefusal) + `WireEvent::Refusal` Class-C envelope (closes NA-GAP-02 + NA-GAP-11)
- `ai_specs/cross-cutting/substrate-drift.md` — first-class substrate-drift detection (canary contract + `SubstrateDriftDetected` event + CR-2 POVM canonical case; closes NA-GAP-07 cross-cutting half)

### Decision register (Wave 4.B ADRs)

- **D-S1001982-01** — `ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md` (m42 POVM decoupling; 48-decision grilling outcome)
- **D-S1002127-01** — `ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md` (G8 stcortex persistence pre-spec; ~46 memories + ~60 pathways planned at G8-green)
- **D-S1002127-02** — `ai_docs/optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md` (EscapeSurfaceProfile 6→7 with `PrivilegeEscalation`)
- **D-S1002127-03** — `ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md` (v0.2.0 work items W1/W2/W3 for NA-GAP-07 module / NA-GAP-08 fixtures / NA-GAP-10 trust)

### Per-cluster operational landings (Wave 4.B audit)

Operational view counterpart to `ai_specs/layers/cluster-X.md` design specs, scaffolded at `~/claude-code-workspace/the-workflow-engine/layers/cluster-{A-H}/README.md`. Per-module operational landings (`modules/m<N>_<name>.md` × 26) are reserved for post-G9 per `modules/README.md` rationale (cannot meaningfully describe runtime behaviour pre-G9).

### Surface amendments (Wave 4.B)

- `ai_specs/ERROR_TAXONOMY.md` — added `RefusalToken` section + per-variant classification table
- `ai_specs/modules/cluster-H/m42_stcortex_emit.md` § 5.1 — outbox-policy (drain ordering / saturation 64MB-warn / 256MB-refuse / 1GB-panic; snapshot staleness 5min/1hr/24hr thresholds)
- `ai_specs/BENCHMARK_SPEC.md` — substrate-side load benchmarks section (6 substrate benches; per-substrate cadence-throttle rules)
- `ai_specs/INDEX.md` — substrate-couplings/ section + footer back-refs
- `README.md` (root) — repository-layout table refreshed; cold-start adds Wave 4.B surfaces
- `ai_docs/QUICKSTART.md` — Wave 4.B substrate-as-actor cold-start additions

## 6. Boilerplate modules archive (reference-only)

- [[boilerplate modules/README|README (boilerplate modules)]] — archive purpose, compliance posture, REFERENCE-ONLY warning
- [[boilerplate modules/BOILERPLATE_INDEX|BOILERPLATE_INDEX]] — per-file lift map with target-module mapping

**Gold-standard exemplar profiles** (synthesis: [[Gold Standard Exemplars — Synthesis]]):

- [[Habitat Loop Engine — Gold Standard Reference]]
- [[Maintenance Engine V2 — Gold Standard Reference]]
- [[ORAC Sidecar — Gold Standard Reference]]

**Markdown clones (skills + onboarding + feedback):**

- [[CONSUMER-ONBOARDING]] · [[stcortex_API]] (cat 02)
- [[SKILL-forge]] · [[SKILL-genesis]] · [[SKILL-pre-deploy-hardening]] · [[SKILL-quality-gate]] · [[SKILL-silent-swallow-detect]] · [[feedback_preserve_list_discipline]] · [[hookify.preserve-blanket-guard.local]] (cat 09)

- 48 source files across 10 category subdirectories (~1.2MB):

| Subdir | Files | Source crates |
|---|---:|---|
| `01-cli-scaffolding/` | 4 | habitat-conductor + loop-engine-v2 |
| `02-stcortex-consumer/` | 4 | stcortex (clients + docs) |
| `03-sqlite-multi-db/` | 6 | memory-injection + habitat-buoy |
| `04-pattern-detection/` | 3 | orac-sidecar + synthex-v2 + povm-v2 |
| `05-decay-ttl-ltd/` | 4 | povm-v2 + orac-sidecar + synthex-v2 + memory-injection |
| `06-daemon-scaffolding/` | 6 | synthex-v2 + habitat-nerve-center + habitat-buoy |
| `07-conductor-dispatch/` | 5 | habitat-conductor + dev-ops-engine-v3 |
| `08-nexus-lcm-rpc/` | 4 | orac-sidecar + dev-ops-engine-v3 |
| `09-trap-verify-escape-skills/` | 7 | .claude/skills/* + hookify + feedback memory |
| `10-foundation-direct-clones/` | 4 | synthex-v2 + dev-ops-engine-v3 (95% reuse) |

## 7. Convention notes (vault navigation)

### Bidirectional linking

Every note starts with `> Back to: [[HOME]] · [[MASTER_INDEX]] · canonical: <fs-path-if-mirror>` per habitat convention.

HOME.md cross-links forward to every primary + mirror note.
MASTER_INDEX.md (this note) categorises all artefacts by purpose.
workflow-engine-code-base.md provides chronological view + decision log.

### Mirror vs canonical

Mirror notes in vault carry **substantive summary** + link to canonical fs path. Full content for pre-framework planning artefacts now lives at `~/claude-code-workspace/the-workflow-engine/pre-framework-consolidation/` (consolidated 2026-05-17 — see [[Pre-Framework Consolidation Notice S1001982]]). Framework-era artefacts live in `the-workflow-engine-vault/deployment framework/`. Vault mirrors are for **navigation + reference**, not duplication.

### Naming convention

Vault notes use **Title Case with S1001982 session-id suffix** for the post-town-hall planning batch. Older session artefacts (none yet in this vault) would use their own session-id.

---

## ⚠️ Known open issues

The original 6 planning-era open issues are **all resolved** — the gate sequence is closed,
v1.3 is the binding spec, and the 26-module codebase is implemented. The honest **residuals**
after the S1003733 remediation + C22 wiring are tracked in [[Assessment Remediation S1003733]]
§ "Bugs & Known Issues" — summarised here:

| # | Residual | Location | Status |
|---|---|---|---|
| R1 | m33 verifiers are conservative-default (`Approve`) placeholders — real per-kind policy logic needs inputs the binary does not yet receive | `src/m33_verifier/`, `src/orchestration/dispatch.rs` | OPEN — honest scope note from C22 |
| R2 | m22 K-means diversity not assembled on the CLI batch paths — `wf-crystallise` passes an honest `\|_\| None` closure, not a faked signal | `src/orchestration/crystallise.rs` | OPEN — honest scope note from C22 |
| R3 | 9 m21 `build_variants` loop-condition mutants proven output-equivalent (not killable; defense-in-depth iteration cap makes the mutation inert) | `src/m21_variant_builder/` | CLOSED-AS-EQUIVALENT — `// mutant-equivalent:` proof comments in source |
| R4 | m8 POVM trust gate is a dormant build.rs tripwire (KEEP-DORMANT decision), not a runtime pipeline stage | `src/m8_povm_build_prereq/` | RESOLVED — architecture decision (keep-dormant) |

**Resolved planning-era issues** (historical): module-count / naming-convention inconsistency
(reconciled at 26 modules, `m<N>` unpadded); v1.3 patch (authored, G7-cleared); Ember rubric
§5.1 (resolved); Conductor `auto_start=false` (Luke terminal action, non-blocking — `--dry-run`
is the default-safe `wf-dispatch` mode); G9 out-of-sequence (G9 fired 2026-05-17).

See [[Assessment Remediation S1003733]] for the full Bugs & Known Issues / Diagnostics tables
and [[workflow-engine-code-base]] for the historical open-issue tracker.

---

## Gate states snapshot — ALL RESOLVED (G9 fired 2026-05-17)

| # | Gate | State |
|---:|---|---|
| G1 | Watcher ratification close-notice | ✅ resolved |
| G2 | Directory rename `the-workflow-engine/` → `workflow-trace/` | deferred post-M0 (cosmetic) |
| G3 | `:8125` redeploy verify (povm-v2 live) | ✅ resolved (m42 stcortex-only pivot) |
| G4 | Watcher notes (Hebbian v3 / Ember rubric) | ✅ resolved |
| G5 | Genesis interview + F2 hard gate | ✅ resolved |
| G6 | Dual-frame gap analysis | ✅ resolved |
| G7 | Zen spec audit (APPROVE/REFUSE/AMEND) | ✅ resolved (v1.3 binding) |
| G8 | Four-surface persistence | ✅ resolved |
| G9 | Luke explicit start-coding signal | 🔥 **FIRED 2026-05-17** |

---

---

## 7b. Scaffold Wave 0/1/2 (S1002127 — 2026-05-17)

Under [[../PRIME_DIRECTIVE_WAIVER|PRIME_DIRECTIVE_WAIVER.md]] (structure + specs + config only; NO `.rs`; NO `Cargo.toml`; G9 still gated). See [[Scaffold Wave 0-2 — Session S1002127]] for the full session note.

**Per-cluster scaffold notes (8):**

- [[Cluster A Scaffold — Module Specs S1002127]] — m1, m2, m3 (substrate ingest)
- [[Cluster B Scaffold — Module Specs S1002127]] — m4, m5, m6 (habitat observers)
- [[Cluster C Scaffold — Module Specs S1002127]] — m7, m12, m13 (correlation + output hub)
- [[Cluster D Scaffold — Module Specs S1002127]] — m8, m9, m10, m11 (trust cross-cutting)
- [[Cluster E Scaffold — Module Specs S1002127]] — m14, m15 (evidence + pressure)
- [[Cluster F Scaffold — Module Specs S1002127]] — m20, m21, m22, m23 (iteration KEYSTONE)
- [[Cluster G Scaffold — Module Specs S1002127]] — m30, m31, m32, m33 (bank + select + dispatch + verify)
- [[Cluster H Scaffold — Module Specs S1002127]] — m40, m41, m42 (substrate feedback)

**Canonical repo surfaces (paths repo-relative from vault):**

- `../ai_specs/modules/cluster-{A-H}/` — 26 per-module Rust spec markdown files (one per module, sourced from the existing cluster specs under `module specs/`)
- `../ai_specs/INDEX.md` · `../ai_specs/MODULE_MATRIX.md` — top-level catalogue + 26×30 capability matrix
- `../ai_docs/INDEX.md` — top-level docs index (architecture deep dive, runbooks, decisions, schematics, reflections)
- `../ultramap/README.md` (+ `../ultramap/schematics/`) — DATA / CONTROL / CONTEXTUAL / INVARIANT / master flow maps
- `../src/README.md` · `../tests/README.md` · `../benches/README.md` — directory landing pages (no code under any of them)
- `../.claude/` — Claude Code config: [`settings.json`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/.claude/settings.json) · [`anti_patterns.json`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/.claude/anti_patterns.json) · [`patterns.json`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/.claude/patterns.json) (+ `agents/` `commands/` `hooks/` `skills/` `schemas/` `queries/` `worktrees/` subtrees)
- `../bin/wf-crystallise/README.md` · `../bin/wf-dispatch/README.md` — per-binary READMEs (planning surface only)
- `../config/*.toml.template` · `../docs/README.md` · `../scripts/README.md` · `../migrations/README.md` · `../hooks/README.md` · `../security/README.md` · `../schematics/README.md` — placeholder READMEs (one paragraph each)

**Waiver / decision provenance:** [[../PRIME_DIRECTIVE_WAIVER|PRIME_DIRECTIVE_WAIVER.md]] · [[../CHANGELOG|CHANGELOG.md]] § `[v0.0.0-spec.0]` · [[../CLAUDE|../CLAUDE.md]] § PRIME DIRECTIVE (waiver overrides "no scaffold" clause only).

---

## 8. V7 Optimisation subtree (added 2026-05-17 post-grilling)

Canonical: `ai_docs/optimisation-v7/` (44 markdown deliverables / ~115k words / planning-only / HOLD-v2 respected). 3 vault mirrors of the most-load-bearing artefacts:

- [[optimisation-v7/HOME|V7 vault HOME]] — Tier-1 landing for V7 subtree
- [[optimisation-v7/V7 Optimisation Framework]] — canonical V7 framework mirror (Tier-2)
- [[optimisation-v7/m42 stcortex-only pivot ADR]] — m42 pivot ADR mirror (Tier-2; 48-decision grilling outcome)
- [[optimisation-v7/Session S1001982 m42 pivot grilling]] — reusable substrate-pivot doctrine (Tier-2)

**Substrate bidi:** [[stcortex — Pioneer Capability Dossier 2026-05-10]] (m42 routes-to) · [[POVM Engine]] (DECOUPLED post 2026-05-17 ADR)

**m42 pivot summary:** module renamed `src/m42_povm_dual/` → `src/m42_stcortex_emit/`. POVM dependency removed pre-deployment. Featureset preserved 1:1 via stcortex. Risk surface reduced. Triggered AP-V7-13 crystallisation ("Health-200 ≠ behaviour-verified"). Decision register: 61 made / 0 deferred.

---

*MASTER_INDEX last refreshed: 2026-05-17 S1001982 (vault save v2 + V7 subtree added + m42 pivot landed)*
