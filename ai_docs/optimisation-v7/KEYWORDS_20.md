---
title: KEYWORDS — 20 Context Anchors for workflow-trace V7 Optimisation
date: 2026-05-17 (S1001982)
kind: planning-only · context-anchor · used by Claude across all V7 docs
count: exactly 20
purpose: load this list at session start; whenever a doc references a keyword, the full context is implied
---

# 20 KEYWORDS — Context Anchors for the V7 Optimisation Stack

> Back to: [[TASK_LIST_V7_OPTIMISATION.md]] · [[ULTRAMAP.md]] · [[../../CLAUDE.md]]
>
> **Usage:** every V7 doc references these by name. Loading this file gives Claude the full semantic scaffold for the entire end-to-end stack — no rediscovery needed mid-task. Each keyword's expansion lives below; treat it as a load-bearing primer.

---

## The 20 (alphabetical; load order is irrelevant)

| # | Keyword | One-line meaning |
|---|---|---|
| 1 | **AP24** | No code without explicit `start coding <project>` signal from Luke; G9 enforcement |
| 2 | **AP30** | stcortex namespace prefix discipline (`workflow_trace_*`); collision-proof |
| 3 | **Battern** | 6-step Battern protocol Design→Dispatch→Gate→Collect→Synthesize→Compose for fleet coordination |
| 4 | **bidi-anchor** | Bidirectional `> Back to:` link discipline; every doc round-trips to its parent + peers |
| 5 | **Cluster** | One of 8 (A-H) module-grouping units; 26 modules total across 9 layers L0-L8 |
| 6 | **Conductor** | HABITAT-CONDUCTOR; the only dispatch path for m32 (Waves 1B/1C/2/3 `auto_start=false` is B3 blocker) |
| 7 | **EscapeSurfaceProfile** | Ordinal enum `ReadOnly < HostWrite < Network < SandboxEscape < Destructive`; Gap 3 |
| 8 | **F2** | Hard sample-size gate (n≥20 + Wilson 95% CI); enforced at `ProposalBuilder::build()` construction |
| 9 | **four-surface** | ai_docs canonical + Obsidian vault mirror + stcortex namespace + CLAUDE.local.md anchor (G8-gated for this work) |
| 10 | **G1-G9** | Pre-genesis gates in Zen-prescribed serial order; ALL must green before `cargo init` |
| 11 | **gold-standard** | ME v2 + LCM + ORAC + (CodeSynthor V8) convergent patterns; 13 patterns cataloged |
| 12 | **HOLD-v2** | Current envelope: no code, no cargo, no scaffold, no rename, no stcortex writes |
| 13 | **KEYSTONE** | Cluster F m20-m23 (PrefixSpan + Levenshtein + Wilson CI); the engine's structural gap |
| 14 | **LTP/LTD-0.043** | Substrate condition (35× below healthy 1.5-4.0); Class-I Watcher flag pre-positioned |
| 15 | **PIPESTATUS** | Bash discipline `${PIPESTATUS[0]}` for cargo-pipe-tail gate chains; per-stage abort |
| 16 | **PrefixSpan** | Cluster F algorithm choice; gap-allowed matching; bounded right-gap MAX_GAP_STEPS=5 |
| 17 | **single-phase** | Luke 2026-05-17 override; 5 explicit waivers; risks on Command's head |
| 18 | **stcortex-only-m42** | m42 routes substrate-feedback to stcortex exclusively (POVM dual-path retired pre-deploy per 2026-05-17 ADR); outbox-first JSONL durability; fitness-delta constants preserved; CC-5 closure verified via stcortex pathway.weight delta |
| 19 | **verify-sync** | ≥18 invariants (Layer↔Module↔src/↔feature↔tests) checked every commit (synthex-v2 pattern) |
| 20 | **Watcher-class** | A-I flag taxonomy for deployment observation; pre-positioned per phase + per generation |

---

## Why exactly these 20

Five selection rules drove the cut:

1. **High-frequency reference rate** — appears in ≥3 of the 10 phase docs OR every cluster spec
2. **Load-bearing semantic gravity** — its absence would force re-derivation from upstream
3. **Coverage of all 8 clusters + all 9 phases + 5 waivers + 11 failure modes** — no surface left unaddressed
4. **Anchors a specific concrete artefact** — every keyword maps to a file, command, formula, or rule
5. **Disambiguates a common drift point** — e.g., AP24 vs AP30 collision, F2 sample-size vs F8 feedback-poisoning, four-surface deferred-vs-active

---

## Expansion register (load-on-demand)

### 1. AP24 — Activation discipline
Long-form: No `cargo init`, no `src/*.rs`, no scaffold, no execution of any kind without Luke typing the exact text `start coding workflow-trace` (or unambiguous canonical-name variant). G9 IS this signal. The signal that arrived 2026-05-17T08:43Z is held as queued intent only (Zen URGENT block per D6). When G8 greens, Luke must **re-issue** the signal explicitly.
Anti-pattern caught: pre-G9 src/ creation. Watcher Class-F flag.
Source: MEMORY.md `feedback_wait_for_start_coding.md` + `~/projects/claude_code/Sessions/Session 080 — Activation Discipline.md`.

### 2. AP30 — Namespace prefix discipline
Long-form: stcortex memories under workflow-trace MUST use prefix `workflow_trace_*` (NOT `workflow_engine_*` legacy or unprefixed). Avoids collision with V3 P01..P16, ORAC POVM space, and the existing `the_workflow_engine` planning namespace (baseline memories 16477/16479).
Anti-pattern caught: silent namespace collision producing pathway-merge between unrelated systems.
Source: CLAUDE.md § Memory Systems #8 stcortex.

### 3. Battern — Patterned batch dispatch
Long-form: 6-step protocol — Design (compose task), Dispatch (fan-out to Fleet-ALPHA/BETA/GAMMA), Gate (verify each pane received), Collect (gather artefacts), Synthesize (Command merges), Compose (file outbound). Used for the V1 ULTIMATE_DEPLOYMENT_FRAMEWORK's 9-agent author wave. Used for the V7 module-plans + runbooks parallel author waves.
Anti-pattern caught: ad-hoc shell dispatch + no audit trail.
Source: `.claude/skills/battern-protocol/SKILL.md` + `~/projects/claude_code/habitat/reflections/2026-05-03-weaver-battern-verification-cascade.md`.

### 4. bidi-anchor — Bidirectional vault linking
Long-form: Every authored markdown file begins with `> Back to: [[parent]]` referencing both the canonical file path AND the vault mirror. Every parent's index lists the child. Round-trip navigation in one hop.
Anti-pattern caught: orphan docs that auto-prune when their navigation parent rots.
Source: CLAUDE.local.md § Bidirectional anchors.

### 5. Cluster — 26 modules across 8 clusters (A-H)
| Cluster | Modules | Role | Layer |
|---|---|---|---|
| A | m1, m2, m3 | substrate ingest | L1 |
| B | m4, m5, m6 | habitat observation | L2 |
| C | m7, m12, m13 | central correlation + output | L3 |
| D | m8, m9, m10, m11 | trust (aspect layer) | L4 |
| E | m14, m15 | evidence + pressure | L5 |
| F | m20-m23 | iteration (KEYSTONE) | L6 |
| G | m30-m33 | bank + select + dispatch + verify | L7 |
| H | m40-m42 | substrate feedback | L8 |

### 6. Conductor — HABITAT-CONDUCTOR
Long-form: Habitat's authoritative dispatch coordinator. m32 NEVER executes workflows directly — only routes via Conductor (P0 #3). Waves 1B/1C/2/3 currently `auto_start=false` (B3 blocker). Refuse-mode is the m32 behaviour when Conductor unreachable: ERROR log + typed `DispatchError::ConductorDispatchDisabled`, not silent no-op.

### 7. EscapeSurfaceProfile — Gap 3 ordinal schema
Long-form: `enum EscapeSurfaceProfile { ReadOnly, HostWrite, Network, SandboxEscape, Destructive }`. Ordinal comparison via `Ord` derive. Display-before-step in m32 (stdout banner mandatory). m30 owns the schema; m9 namespace guard at write boundaries; m32 dispatcher banner. Replaces scattered classifiers in `.claude/skills/forge`, `genesis`, `pre-deploy-hardening`, `silent-swallow-detect`, `hookify.preserve-blanket-guard.local.md` (S102 scar tissue).

### 8. F2 — Sample-size hard gate
Long-form: `n≥20 + Wilson 95% CI (z=1.96)` per report type. Not Wald (negative lower bounds at small n). Enforced at `ProposalBuilder::build()` — rejection at construction, no runtime bypass. Returns `Option<Confidence>::None` not `0.0` when n<20 (distinguishes "no signal" from "zero signal").

### 9. four-surface — Persistence discipline
Long-form: For G8-and-beyond plans: write to (1) ai_docs canonical, (2) Obsidian vault mirror, (3) stcortex namespace, (4) CLAUDE.local.md anchor. Each surface bidi-linked to the other three. "One surface survives = plan survives" (CLAUDE.local.md Working Mode). **DEFERRED for V7 optimisation** — surfaces 3 (stcortex) and 4 (workspace-root CLAUDE.local.md edits) are HOLD-v2-blocked.

### 10. G1-G9 — Pre-genesis gates
Serial DAG (no parallelisation). Current state 2026-05-17: 0 of 9 green. G9 fired out-of-sequence and is Zen URGENT-blocked. Two unblock paths: drive G1-G8 in order OR file explicit per-gate Luke waivers. **Phase 0 runbook owns the operational detail.**

### 11. gold-standard — Convergent-pattern catalogue
13 patterns ME v2 + LCM + ORAC + (CodeSynthor V8) all share: `src/mN_<theme>/` DAG, workspace+feature matrix, CLAUDE.md split, MASTER_INDEX root, ai_docs/+ai_specs/ partition, co-located vault, SQLite+migrations/, 50+ tests/module, 4-stage QG, `mN_bridges/<peer>.rs` with circuit breaker, `forbid(unsafe_code)+deny(unwrap)`, live drift register, layered TOML config.

### 12. HOLD-v2 — Current envelope
Forbids: code, cargo, scaffold, rename `the-workflow-engine/→workflow-trace/`, stcortex writes under `workflow_trace_*`, workspace-root CLAUDE.local.md edits (one exception: Hebbian v3 reconciliation citation update — already applied).
Allows: file-drop comms, vault writes, project-tree planning docs (which is what V7 is).

### 13. KEYSTONE — Cluster F m20-m23
Engine's structural keystone. ~850 LOC total; ~600-700 LOC fresh authorship (PrefixSpan + variant builder + similarity scorer + gradient-preservation proposer). m20 splits across 4 internal passes (skeleton → algorithm → Wilson CI gate → variant selection). m22 K-means NOT PrefixSpan (different feature space).

### 14. LTP/LTD-0.043 — Substrate condition
Current ratio: 0.043 (2,547 LTP / 58,772 LTD). Target healthy band: 1.5-4.0. **35× below target — LTD-dominant.** CR-2 fixed measurement (was 0.911 inflated; now 0.067 magnitude-weighted), not condition. Engine ships onto this substrate. Watcher tick·16 (2026-05-17T02:59Z): sustained Hebbian pause confirmed across 4+ ticks. Class-I flag actively firing.

### 15. PIPESTATUS — Bash gate-chain discipline
Long-form: `cargo … | tail` makes `$?` capture `tail`'s exit (always 0). Use `${PIPESTATUS[0]}` + explicit per-stage abort, or the gate prints green while clippy was screaming.
Anti-pattern caught: S1001882 near-miss — almost committed against a broken gate (clippy::doc_markdown on un-backticked `tracing::error`).
Source: MEMORY.md `feedback_pipestatus_for_gate_chains.md`.

### 16. PrefixSpan — Cluster F algorithm choice
Long-form: Sequential pattern mining; projection-based; avoids candidate-explosion of Apriori at N>2. Gap-allowed matching is the killer feature absent from n-gram sliding window. `MAX_GAP_STEPS = 5` default; unbounded left-gap, bounded right-gap.

### 17. single-phase — 2026-05-17 Luke override
Waives 5 prior P0 considerations: Watcher R6 frame-separation (partial), Fossil scope-discipline (FULL — **Phase 4 Zen audit is the non-negotiable compensating control**), RALPH selector-without-measurement safety (partial), Skeptic pain-source verification (FULL), Substrate exploration-protection (partial). G1-G9 NOT waived. v1.2 verb-locked invariant relaxes for active-verb modules m20-m33 + m40-m42, but Zen G7 spec audit on v1.3 still required.

### 18. stcortex-only-m42 — Substrate-feedback routing
Long-form: m42 (renamed `src/m42_stcortex_emit/`) writes substrate-feedback exclusively to stcortex `:3000` under `workflow_trace_*` namespace from M0. POVM dual-path retired pre-deployment per 2026-05-17 ADR (live probe found POVM `:8125` health-200 but serving pre-CR-2 binary `learning_health=0.9146`). Featureset preserved: fitness-delta constants (PassVerified +0.25 / Pass +0.15 / Blocked -0.05 / Fail -0.10), outbox-first JSONL durability, shared `m40_42_common::Breaker` (now 2 peers — synthex-v2 + stcortex), CC-5 closure-test (assertion: stcortex pathway.weight delta observable). Watcher Class-I extended to monitor stcortex pathway.weight delta over rolling 7d. Offline JSONL snapshot fallback per workspace charter — never silent POVM fallback. Substrate-condition: no dependency on POVM CR-2 deployment. Anchor: [[ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md]].

(Crate-layout knowledge formerly anchored here is preserved in: v1.3 § 6 Axis 1 + ULTRAMAP V2 + G2-consolidation Cargo contract. Two-binary split `wf-crystallise` (m1-m23, m40-m42) + `wf-dispatch` (m30-m33) + shared `workflow-core` lib remains the structural decision.)

### 19. verify-sync — Sync-invariant gate
Long-form: synthex-v2 pattern — every commit must satisfy ≥18 verify-sync invariants. Each invariant: predicate + check-command + failure-class. Workflow-trace target ≥18. Example: "every L1 module has an entry in `src/lib.rs::layer_l1::mod`", "every cluster file in vault has a corresponding `MODULE_PLANS/cluster-N.md`", "every feature-gated module has tests in both `feature=full` and `default` configurations". Mechanically enforced via `scripts/verify-sync.sh` + CI.

### 20. Watcher-class — A-I flag taxonomy
| Class | Captures | Pre-positioned at |
|---|---|---|
| A | activation transition (gate flip) | every G1-G9 + Cluster H first activation + G7 verdict moment |
| B | hand-off boundary crossing | Phase 3 cross-substrate calls; Phase 5B cutover |
| C | confidence-gate refusal (NAM-03) | Phase 3 Track 2 Conductor refuse; Phase 4 4-agent REJECT |
| D | four-surface drift | Phase 5+ ongoing |
| E | ancestor-rhyme (planning sprawl) | T0 (41,508w / 0 LOC) — demoted at tick·15 since project inducted |
| F | AP24 violation (code before G9) | pre-Phase-1 only |
| G | substrate-frame confusion | Phase 2B onward (active-verb cluster F/G/H risk) |
| H | atuin proprioception anomaly | Phase 5+ |
| I | Hebbian silence | Phase 3 first POVM write through Phase 5+; **currently firing live per tick·16** |

---

## Quick-use guide (for next-session Claude)

1. Open this file when entering workflow-trace context.
2. Treat each `**keyword**` mention in V7 docs as an implicit reference to the full expansion here.
3. If a doc uses a keyword in a way inconsistent with its expansion — flag a Class-D drift (four-surface) and propose amendment.
4. The 20 are stable; if the framework matures and new load-bearing terms emerge (e.g., a new failure mode or a new substrate), update this file and bump the count discussion — do NOT silently grow past 20 without explicit acknowledgment.

---

*KEYWORDS_20 authored 2026-05-17 by Command as foundational anchor for the V7 optimisation stack. Referenced by all 42 V7 deliverables.*
