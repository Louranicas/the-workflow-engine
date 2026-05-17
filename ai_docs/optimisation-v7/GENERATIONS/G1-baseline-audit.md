---
title: G1 — Baseline Audit (Generation 1 of 7)
date: 2026-05-17 (S1001982)
kind: planning-only · gap-identification pass
purpose: identify every gap in the current V1 corpus relative to V7 optimisation targets
inputs: ULTIMATE_DEPLOYMENT_FRAMEWORK (10 phase docs) + GOD_TIER_CONSOLIDATION + 8 cluster specs + this V7 foundation set
gap_count: 27 gaps named across 7 classes
---

# G1 — Baseline Audit

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · sibling: [[G2-consolidation.md]] (not yet written — G1 produces input for G2)
>
> **Function:** the first of 7 generations. Reads V1 corpus + V7 foundation; enumerates every gap that subsequent generations must close. Gap-class taxonomy: sync / bidi / test / integration / observability / standards / antipattern. Each gap gets ID + class + source-citation + which subsequent gen owns its closure.

---

## Gap-class taxonomy

| Class | Definition | Owning generation |
|---|---|---|
| **GAP-Sync** | Layer / Module / src/ allocation drift; missing verify-sync invariants | G2 consolidation |
| **GAP-Bidi** | Missing upstream / downstream edges; orphan modules in flow graph | G3 bidi flow |
| **GAP-Gold** | Divergent-from-gold-standard pattern; missing convergent-pattern adoption | G4 gold-standard |
| **GAP-Tool** | Missing integration with /scaffold, atuin, V3, CSv8, Zellij | G5 tooling |
| **GAP-Test** | Test count below 50 / missing pattern family / mutation kill-rate undefined | G6 test discipline |
| **GAP-AP** | Antipattern not registered / detection missing | sweep across all gens; G7 consolidates |
| **GAP-Substrate** | Anthropocentric-input assumption / substrate-frame missing | second-frame pass per gen |

---

## The 27 gaps

### GAP-Sync-01 — Cluster D `cargo:rustc-cfg=povm_calibrated` not mirrored in verify-sync
**Source:** GOD_TIER_CONSOLIDATION_S1001982.md Part X surprise #3.
**Problem:** m8 uses `cargo:rustc-cfg`, not Cargo feature. Verify-sync invariant set in GOD_TIER_RUST.md doesn't explicitly check this cannot-be-bypassed-by-`--features-full` property.
**Closure:** G2 — add invariant #21: "no `[features] povm_calibrated`" enforced; cargo `cfg(povm_calibrated)` only honored via env at build.

### GAP-Sync-02 — Workflow-core lib boundary undefined
**Source:** ULTIMATE_DEPLOYMENT_FRAMEWORK.md § 2-binary; Phase 1 Genesis runbook references shared `workflow-core` but no module-list spec.
**Problem:** which types/schemas/constants live in `workflow-core` vs per-module is unspecified.
**Closure:** G2 — define `workflow-core` lib contract: types (WorkflowId, StepToken, EscapeSurfaceProfile, Confidence), schemas (m7 hub, m30 bank, m40 NexusEvent), namespace constants (AP30 prefixes).

### GAP-Sync-03 — Wave-end verify-sync invariant 14-20 untested at Wave-1 close
**Source:** AGENT_VIEW_GITWORKTREES.md § Wave 1 merge gate references invariants 1-7; 8-13 at Wave 2; 14-20 at Wave 3.
**Problem:** invariants 14-20 reference active-verb modules (m20-m42) not in Wave 1. Without "graceful absent" check, Wave-1 gate fails on invariants 14-20.
**Closure:** G2 — verify-sync script supports `--invariants N-M` subset selection; per-Wave invariant subset documented.

### GAP-Sync-04 — m11 decay formula not exposed in m7 schema
**Source:** GOD_TIER_CONSOLIDATION Part I Cluster D + Cluster C.
**Problem:** m11 needs m7.last_run_at + m14.frequency + stcortex pathway.weight. m7 schema must surface `last_run_at REAL NOT NULL` at Cluster C build time, before m11 ships in Cluster D. **Dependency inversion vs ULTIMATE_DEPLOYMENT_FRAMEWORK Day 1 (Cluster D first)**.
**Closure:** G2 — clarify: m11 Day 1 implements PURE formula (no I/O); wiring to m7 happens at Cluster C build Day 3-4. Refactor Phase 1 runbook accordingly.

### GAP-Sync-05 — No invariant on `cargo doc --no-deps` clean
**Source:** GOD_TIER_RUST.md rule 4.
**Problem:** rule says `#![warn(missing_docs)]` but no verify-sync check that `cargo doc` actually compiles clean per Wave-end.
**Closure:** G2 — add invariant #22: `cargo doc --no-deps --workspace 2>&1` returns 0 warnings.

---

### GAP-Bidi-01 — m22 K-means upstream-edges incomplete
**Source:** ULTRAMAP.md View 2.
**Problem:** ULTRAMAP shows m22 ← m6 + m7. Module spec cluster-F implies m22 also needs m4 cluster IDs for feature vector. Bidi diagram inconsistent.
**Closure:** G3 — reconcile cluster-F module plan with ULTRAMAP; m22 upstream = m4 + m6 + m7.

### GAP-Bidi-02 — CC-7 (Pressure-Driven Evolution) feedback edge missing in flow diagram
**Source:** GOD_TIER_CONSOLIDATION Part II CC table.
**Problem:** CC-7 says "m15 reservation register surfaces in agent-cross-talk; Watcher + Zen observe scope-pressure; accumulation triggers spec amendment interview." This is a feedback loop BACK to spec / G5 interview. ULTRAMAP V2 flow diagram only shows m15 → agent-cross-talk/ (one-way).
**Closure:** G3 — add CC-7 feedback edge: m15 → agent-cross-talk/ → Watcher/Zen → (manual) spec interview → m1.config update.

### GAP-Bidi-03 — m40 / m41 / m42 fan-out lacks circuit-breaker shared-state contract
**Source:** GOD_TIER_CONSOLIDATION Part I Cluster H: "Circuit breaker (Closed → Open → HalfOpen) shared across all three modules; 5 failures → Open; 60s → HalfOpen."
**Problem:** Shared across THREE modules — but spec doesn't specify whether breaker is per-peer (3 breakers) or shared (1 breaker for all bridges). Bidi diagram doesn't show breaker state-machine.
**Closure:** G3 — explicit: 1 breaker per substrate peer (3 total); shared module `m40_42_common::Breaker`; state-transition diagram in cluster-H plan.

### GAP-Bidi-04 — m10 Ember CI lacks bidi anchor with `m10_ember_gate::test_harness`
**Source:** GOD_TIER_RUST.md rule 4 + TEST_DISCIPLINE.md.
**Problem:** m10 specifies "fails CI on Held verdicts unless `tests/ember_held_approvals.tsv` allowlist entry". The test-harness module is not bidi-named in ULTRAMAP V2.
**Closure:** G3 — add bidi: m10 ↔ tests/ember_held_approvals.tsv; m10 ↔ Watcher rubric `~/projects/claude_code/Ember 7-Trait Gate Rubric.md`.

### GAP-Bidi-05 — Cluster A m1/m2/m3 → m9 namespace guard edge missing
**Source:** ANTIPATTERNS_REGISTER.md AP-WT-F3 (substrate-input poisoning).
**Problem:** m9 is named as mitigation, but ULTRAMAP doesn't show m9 inspecting m1/m2/m3 outputs.
**Closure:** G3 — m9 is aspect-layer (per Cluster D); add aspect-arrow from m9 → m1, m2, m3 at write-boundary.

---

### GAP-Gold-01 — Cargo workspace organisation not aligned with gold-standard recommendation
**Source:** GOD_TIER_CONSOLIDATION Part IV § Divergent decisions.
**Problem:** S1002029 Synthesis recommends "ORAC pattern (single crate + features) unless ≥3 independent release cadences." Workflow-trace ULTRAMAP shows two binaries + one lib (single workspace). **Risk:** treating this as 3-crate workspace when ORAC pattern says single-crate+features unless cadences diverge.
**Closure:** G4 — confirm: single workspace; `[workspace] members = [".", "wf-crystallise", "wf-dispatch", "workflow-core"]` OR single-crate with `[bin]` entries. Recommendation: single-crate single-`Cargo.toml` with two `[[bin]]` + lib (no workspace). Aligns with ORAC.

### GAP-Gold-02 — Spec authority pattern unspecified
**Source:** GOD_TIER_CONSOLIDATION Part IV § Divergent decisions: LCM `plan.toml` recommended.
**Problem:** v1.2/v1.3 are markdown narrative specs. ORAC pattern uses `ORAC_PLAN.md`. LCM uses `plan.toml`. Workflow-trace has not chosen.
**Closure:** G4 — adopt LCM `plan.toml` declarative + supplementary markdown narrative; both at `ai_docs/`. Plan.toml drives `/scaffold` consistency-check.

### GAP-Gold-03 — Evolution layer policy unstated
**Source:** GOD_TIER_CONSOLIDATION Part IV: "**LCM pattern** (none at M0); defer to M2+".
**Problem:** Workflow-trace m31 selector includes fitness-weighted scoring → uses RALPH-like primitives. No evolution-layer rollout policy.
**Closure:** G4 — declare M0-only at Phase 5 deploy: m31 selector active but selection weights derive only from m14 measured lift + m11 decay (no RALPH integration at M0). Optional M2+ RALPH integration via `feature = "ralph-integration"`.

### GAP-Gold-04 — V8 ↔ V3 wire reuse not yet planned at integration phase
**Source:** GOD_TIER_CONSOLIDATION Part IV § 5 substrate-level learnings #1.
**Problem:** "V8 and V3 already speak bidirectionally — POST :8082/api/v8/confidence + /api/v8/learning exist. Hebbian feedback exists at protocol level. *Don't reinvent.*" Workflow-trace V7 doesn't yet plan how m32 dispatch results feed back into V8 confidence updates.
**Closure:** G4 + G5 — Phase 3 Track 5 (DevOps V3 integration) defines: m32 PassVerified outcomes → POST :8082/api/v8/learning with confidence delta.

### GAP-Gold-05 — LCM Drift #11 generalisation not transposed
**Source:** GOD_TIER_CONSOLIDATION Part IV § 5 substrate-level learnings #5.
**Problem:** "orchestrator MUST independently re-exercise" — adopted in V7 verify-sync but not specifically transposed to workflow-trace Wave-end orchestration discipline.
**Closure:** G4 — explicit Wave-end orchestrator (Command) checklist: re-exercise `--workspace --all-targets --all-features`, `git log -1`, integration-smoke per cluster; reject Wave merge on any drift.

---

### GAP-Tool-01 — `/scaffold` integration timing unspecified
**Source:** TASK_LIST T5.1.
**Problem:** GOD_TIER_CONSOLIDATION says `/scaffold` is V8's bound. Workflow-trace V7 doesn't specify WHEN `/scaffold` is invoked (Genesis? per-Wave-end? per-PR?).
**Closure:** G5 — `/scaffold` invoked at: (a) Genesis Day 0 (initial 8-layer skeleton); (b) per-Wave-end (consistency drift check); (c) on-demand by Command (manual audit).

### GAP-Tool-02 — Atuin scripts proposed for workflow-trace undefined
**Source:** TASK_LIST T5.2 mentions "≥5 new scripts" but doesn't name them.
**Problem:** What scripts? When invoked? Owner?
**Closure:** G5 — propose: `wt-gate-status` (current gate state G1-G9 + Wave merge SHA), `wt-soak-pulse` (Phase 5C 30s-interval probe of m14 lift), `wt-substrate-pulse` (LTP/LTD + RALPH + Watcher class flag count), `wt-bridge-check` (5 substrate peers — V8/V3/SYNTHEX/LCM/POVM live), `wt-wave-status` (which worktrees active, lock files, time-budget consumption), `wt-cc5-trace` (Cluster H first-closure trace), `wt-keystone-bench` (m20 PrefixSpan criterion bench delta).

### GAP-Tool-03 — V3 T1-T6 mapping to wf-* binaries unspecified
**Source:** ULTIMATE_DEPLOYMENT_FRAMEWORK § View 4 Tooling Integration Graph.
**Problem:** T1-T6 pipeline emits NAM-03 confidence; workflow-trace 2-binary split doesn't map to T1-T6 explicitly.
**Closure:** G5 — explicit mapping: T1 Specify → wf-crystallise propose; T2 Scaffold → /scaffold; T3 Implement → cargo build; T4 Harden → 4-stage QG; T5 Document → cargo doc; T6 Deploy → wf-dispatch + Conductor.

### GAP-Tool-04 — CodeSynthor V8 plugin integration deferred to "Phase 3 Track 5" but not detailed
**Source:** ULTRAMAP V4 + GOD_TIER_CONSOLIDATION Part I Cluster H.
**Problem:** CSv8 is Elixir OTP at :8111. Workflow-trace V7 needs explicit Rust ↔ Elixir wire spec.
**Closure:** G5 — codesynthor-v8-integration.md defines: HTTP REST contract; per-endpoint payload spec; m32 PassVerified → CSv8 `/api/v8/learning` confidence delta; sphere registration via PV2 (m32 dispatch results published to PV2 sphere).

### GAP-Tool-05 — Zellij plugin proposal for workflow-trace status surface undefined
**Source:** Luke prompt clause "code synthor v8 zellij plugin".
**Problem:** "Code synthor v8 zellij plugin" implies CSv8 also has a Zellij plugin. Need to specify how workflow-trace surfaces (m14 lift, Watcher class-flag count, gate state) appear in Zellij plugin pane.
**Closure:** G5 — propose `wf-status` Zellij plugin pane displaying: gate state (G1-G9), Wave merge SHA, current Watcher class-flag count, m14 lift mean, m11 decay floor, last m32 dispatch outcome.

### GAP-Tool-06 — JSON settings.json schema for workflow-trace `.claude/settings.json` undefined
**Source:** TASK_LIST T5.5.
**Problem:** Luke wants "consistency optimisation in .json for claude code use". What JSON files? settings.json (workspace), .mcp.json, hook payloads, plugin manifests?
**Closure:** G5 — propose: `.claude/settings.json` per workflow-trace project (PRE-G2-rename in current dir, then move on rename); proposed hook config (PostToolUse: rust-postedit gate; Stop: rust-gate-on-stop full QG); `.mcp.json` adds workflow-trace MCP tools (if any planned post-G9).

---

### GAP-Test-01 — Mutation test budget per module unstated for some modules
**Source:** TEST_DISCIPLINE.md table.
**Problem:** Per-module budget gives 1 mutation budget but doesn't specify acceptable kill rate threshold per module (only the global ≥70%).
**Closure:** G6 — per-module threshold: m20 (KEYSTONE) ≥80%; m32 (dispatch security) ≥85%; aspect-layer m8-m11 ≥75%; others ≥70%.

### GAP-Test-02 — Fuzz target enumeration incomplete
**Source:** TEST_DISCIPLINE.md table (only m4 + m7 + m20 listed for fuzz).
**Problem:** parsers / decoders include m40 JSONL serde + m13 stcortex writer schema + m41 JSON-RPC framing. None have fuzz targets.
**Closure:** G6 — add fuzz targets: m13_stcortex_schema_fuzz, m40_outbox_jsonl_fuzz, m41_jsonrpc_frame_fuzz.

### GAP-Test-03 — Integration-test live-services requirement undocumented
**Source:** TEST_DISCIPLINE.md F-Integration family says "real local services or testcontainer".
**Problem:** Which services? `devenv start` requirement for which subset?
**Closure:** G6 — explicit: m32+m33 integration tests require Conductor (post-B3 resolution); m40 requires synthex-v2 :8092; m41 requires LCM RPC; m42 requires povm-v2 :8125. `#[ignore = "requires live X"]` discipline.

### GAP-Test-04 — Property-test seed reproducibility unstated
**Source:** TEST_DISCIPLINE.md proptest pattern.
**Problem:** When proptest fails, regression-saved test must use specific seed to reproduce. Convention unstated.
**Closure:** G6 — convention: proptest `failure_persistence = Some(Box::new(FileFailurePersistence::SourceParallel("regressions")))`; per-module `tests/regressions/` directory; CI runs at fixed seed for reproducibility.

---

### GAP-AP-01 — AP-V7-10 (mutation-test budget skip) not registered
**Source:** TEST_DISCIPLINE.md mutation discipline.
**Problem:** New antipattern: "Mutation budget cleared without re-running cargo mutants since spec changed." Not in ANTIPATTERNS_REGISTER.
**Closure:** G7 — add AP-V7-10 to register.

### GAP-AP-02 — AP-V7-11 (Wave-merge SHA divergence between local + remote) not registered
**Source:** AP-Drift-08 + AGENT_VIEW_GITWORKTREES.md merge protocol step 8.
**Problem:** Merge succeeds locally; push to one remote succeeds; push to other fails (or PAT scope changed). Wave declared complete on local but inconsistent on remotes.
**Closure:** G7 — add AP-V7-11; mitigation: per-Wave-end `git ls-remote` both remotes verification before tag-complete.

---

### GAP-Substrate-01 — m31 selector activation against degraded LTP/LTD=0.043 substrate
**Source:** GOD_TIER_CONSOLIDATION Part V; Watcher tick·16 (sustained Hebbian pause).
**Problem:** m31 selector reads stcortex pathway.weight → if substrate is LTD-dominant, selector reads near-uniform-low weights → selection becomes effectively random. Whether this corrupts loop convergence is unknown pre-build.
**Closure:** G6 + G7 — Phase 5C 30-day post-deploy monitor BEFORE allowing m31 active selection; OR explicit Luke acceptance of risk; OR defer m31 to M2 post-substrate-rehab.

### GAP-Substrate-02 — Skeptic pain-source verification waived; engine may solve imagined pain
**Source:** GOD_TIER_CONSOLIDATION Part VI Waiver #4.
**Problem:** No Luke-articulated pain evidence in injection.db / MEMORY.md / sessions. Risk: engine builds and ships but nobody actually uses `wf-crystallise propose` because nobody has the pain.
**Closure:** G6 + G7 — Phase 5C weekly Watcher synthesis must report `wf-crystallise propose accept` invocation count. <1/week sustained over 4 weeks = Class-E trigger.

---

## Cross-cutting observations (not gaps, but principles)

### Principle G1-P-01 — V7 itself should not exceed 90,000 words
Rationale: 41,508 words / 0 LOC was the Class-E ancestor-rhyme leading indicator. V7 ADDS planning. Ceiling: V7 deliverables ≤90,000 words total. Current pace ~25,000 words for 9 foundation docs — 7 generations × ~6,000 each + 8 cluster plans × 3,000 + 12 runbooks × 2,000 + 6 integrations × 2,500 + final synthesis ~5,000 = ~108,000. **Over budget.** Mitigation: aggressive de-duplication via keyword references; runbooks tight (focus on commands + verification); cluster plans focus on bidi flow not narrative repetition.

### Principle G1-P-02 — Citation discipline mandatory per gen
Rationale: AP-V7-01 (7-gen drift). Each gen must cite source-line for new claims. Gen-N+1 audits Gen-N citations before building on them.

### Principle G1-P-03 — Watcher pre-positioning per gen
Per Generation, explicitly state which Watcher classes (A-I) the gen's content activates:
- G1 baseline: Class E (this very audit ratifies the ancestor-rhyme risk register)
- G2 consolidation: Class D (four-surface drift if sync invariants break)
- G3 bidi flow: Class G (substrate-frame confusion in bidi edges)
- G4 gold-standard: Class B (hand-off boundary if gold-standard adoption breaks workflow-trace contracts)
- G5 tooling: Class H (atuin proprioception if tool integration breaks command provenance)
- G6 test discipline: Class C (confidence-gate refusal if mutation kill-rate fails)
- G7 final synthesis: Class A (activation transition — V7 closure timestamp)

---

## G1 Watcher pre-positioning

**Class E activated.** This baseline audit IS the ratification of ancestor-rhyme risk. By naming GAP-Substrate-02 (Skeptic pain waiver) and Principle G1-P-01 (word ceiling), V7 takes structural mitigation. Watcher Class-E demoted at tick·15 (project inducted into habitat); G1 keeps it demoted by explicitly bounding V7 planning volume.

---

## G1 substrate-frame pass

**Second-frame question:** "what does this baseline audit ITSELF look like from substrate-frame?"

From substrate-frame, the V1 corpus is **77 vault files / 1.9MB of planning** = a particular kind of substrate signal: anthropocentric coordination output. V7 is its **reinforcement pulse**. The substrate-frame question: does V7 risk **over-reinforcement** (Hebbian saturation of planning-pathway weights at the expense of code-pathway weights)?

**Substrate-frame mitigation:** V7 explicitly couples planning deliverables to **executable Phase 1 actions** (runbooks contain bash commands; module plans contain test allocations; verify-sync invariants are checkable). Planning weight is bounded by executable referent — purely descriptive deliverables are rejected. **G2 must enforce this:** any consolidation that produces narrative without executable referent is rejected by G2 gate.

---

## G1 close

✅ **G1 PASS.** 27 gaps named with class + source-citation + closure-owner. 3 principles seeded (word ceiling, citation discipline, Watcher pre-positioning). 1 substrate-frame mitigation rule (executable-referent requirement) added to G2.

**Output for G2:** this file. G2 reads gaps GAP-Sync-* + Principle G1-P-01 (word ceiling) + substrate-frame executable-referent rule. G2 produces consolidation pass closing 5 GAP-Sync entries + adding sync invariants 21-22.

---

*G1 authored 2026-05-17 by Command. Gaps 1-27 + 3 principles + substrate-frame mitigation. G2 begins immediately.*
