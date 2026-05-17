---
title: MODULE PLAN — Cluster B (Habitat Observation)
date: 2026-05-17
kind: planning-only · per-module deep plan · V7 T3.B deliverable
cluster: B
layer: L2
modules: [m4, m5, m6]
status: V7 author-wave subagent draft (Command)
---

# Cluster B — Habitat Observation (L2)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · [[../GENERATIONS/G3-bidi-flow.md]]
>
> Peers: [[cluster-A.md]] (upstream substrate ingest) · [[cluster-C.md]] (m7 hub joins B outputs) · [[cluster-D.md]] (aspect-wrap) · [[cluster-E.md]] (m14 lift consumes m7 rows derived from B)

---

## Overview

Cluster B is L2 habitat-observation — three correlator modules that transform the raw substrate streams from Cluster A (atuin/stcortex/injection.db) into typed observational records: cascade clusters (m4), battern step records (m5), and context-cost bands (m6). Per ULTRAMAP View 2, B accounts for **~460 LOC** (m4 ~180 + m5 ~140 + m6 ~140) and **170 tests** (60 + 55 + 55 per TEST_DISCIPLINE matrix). All three modules emit into the m7 central hub (Cluster C) which joins them via the `consumer_inputs` JSONB column.

Cluster B is the canonical site for three habitat-specific failure modes (per ANTIPATTERNS_REGISTER § AP-WT): **F11 (cascade monoculture)** is m4's exclusive remit (opaque cluster IDs via FNV-1a XOR); **F1 (name ossification)** lands on m5 (battern step_label must remain `Option`, never forcing 6-step shape); **F10 (exploration-cost preservation)** is m6's responsibility (20-session EMA excludes Converged outcomes). Failing any one collapses the cluster's evidence into anthropocentric noise.

Substrate-frame note (per G3 substrate-frame pass): m4/m5/m6 sit at the boundary where raw trajectory becomes typed observation. The aspect-layer L4 wraps every write-out (m9 namespace prefix on every emitted `cluster_id`/`battern_id` derived for downstream m13 stcortex write). CC-1 Cascade-Cost Coupling lives entirely inside Cluster B — m4 ↔ m6 via m7's JSONB join column; no direct module ↔ module coupling permitted.

---

## m4 — cascade

**Purpose.** Derive opaque cascade-cluster IDs from raw atuin/stcortex/injection rows; never leak pane-label semantics into downstream evidence.

**Upstream-IN** (per G3 § m4).
- `m1.AtuinHistoryRow` — raw history rows with verbatim pane labels.
- `m2.StcortexRow` — narrowed-scope memory consumption events.
- `m3.InjectionEvent` — causal-chain anchors framing cascade boundaries.

**Downstream-OUT.**
- `Vec<CascadeCluster { cluster_id: ClusterId, session_id: SessionId, step_count: usize, window_range: (Timestamp, Timestamp) }>` → m7 central hub (join target) + m31 selector (diversity-check input).

**Aspect-IN.**
- m8 — compile-time `cfg(povm_calibrated)` gate.
- m9 — write-time namespace prefix validation on the derived `cluster_id` payload before m13 stcortex write derived from this cluster.

**src/ path:** `src/m4_cascade/` with `mod.rs` (public API), `cluster_id.rs` (the FNV-1a XOR derivation — the F11 mitigation core), `window.rs` (window-range computation), `derive.rs` (the cascade-derivation algorithm), `error.rs`.

**LOC budget** (per ULTRAMAP View 2): **~180 LOC**. Boilerplate-lift density moderate (~40%); the FNV-1a XOR `cluster_id` derivation is fresh authorship per cluster-B spec.

**Test budget** (per TEST_DISCIPLINE matrix row m4): **60 tests**.

**Test-pattern allocation.**
- F-Unit 30 — per-arm coverage: `derive_cluster_id` for each input variant, `window_range` for sorted/unsorted/empty/single-row inputs, `step_count` accumulator, `ClusterId` `Display`/`Debug` non-leakage.
- F-Property 8 — opaque-ID invariants: `for_all (window: WindowRange, labels: Vec<String>) -> !derive_cluster_id(window, labels).to_string().contains(label)` for any input label; `cluster_id` deterministic under sorted-pane-label permutation; XOR-fold associative.
- F-Fuzz 1 — `m4_cascade_id_fuzz` (per TEST_DISCIPLINE § Fuzz pattern): random UTF-8 bytes into `derive_cluster_id`; assertion = no ALPHA/BETA/GAMMA substring leak.
- F-Integration 15 — m4 ↔ m1/m2/m3 wiring; m4 ↔ m7 join; m4 ↔ m31 diversity-check; concurrent cascade derivation across multiple sessions.
- F-Contract 3 — `CascadeCluster` schema snapshot (insta); `ClusterId` Display stability.
- F-Regression 3 — F11 regression slot (any future leak); cluster-A schema-change regression; XOR commutativity regression.
- F-Mutation 1 budget — ≥70% kill on `cluster_id.rs` (the F11 core).

**Mutation kill threshold:** ≥70%.

**Boilerplate-lift source** (per cluster-B spec).
- FNV-1a hash primitive — fresh authorship (~20 LOC) following the standard FNV-1a 64-bit constant table; no upstream Rust crate dep required (matches god-tier "no unneeded deps" rule).
- Cascade-window derivation — adapted from `memory-injection/m07_causal_chain.rs::find_unresolved` query-result-windowing pattern (~30% reuse).
- Sorted-pane-label canonicalisation — fresh authorship.

**Structural-gap LOC.** None. F11 mitigation is novel-but-bounded (~50 LOC); not Gap-class structural primitive.

**Failure-modes covered** (per ANTIPATTERNS_REGISTER § AP-WT-F11).
- **F11 (cascade monoculture) — exclusive owner.** `cluster_id = ClusterId(fnv1a_64(window_range) XOR fnv1a_64(sorted_pane_labels) XOR step_count)` per G3 § m4. Display impl prints hex digest only; never the source labels. F11 fuzz target asserts no ALPHA/BETA/GAMMA in output.
- F3 (substrate-input poisoning) — m4 typed all inputs into `CascadeCluster` enum before downstream emit; corrupted inputs surface as `derive_cluster_id` returning `Err(DeriveFailure::EmptyInput)`.
- AP-V7-09 (substrate-frame engine confusion) — opaque-ID is precisely the substrate-frame discipline that prevents anthropocentric "this cluster is about X" rationalisation.

**Atuin trajectory anchor.**
- `wt-cascade-pulse` (proposed per T5.2; reads m4's last `CascadeCluster` count for engine-aliveness probe).
- `habitat-bootstrap` (downstream consumer pattern — m4's opaque IDs propagate via m7 into the L4 bootstrap injection).

**Watcher class pre-position.**
- **Class A** — first `derive_cluster_id` invocation post-Genesis.
- **Class G** — substrate-frame confusion if a downstream module starts decoding `cluster_id` back to pane labels.
- **Class D** — four-surface drift if Display impl reveals source semantics in any of stcortex/vault/ai_docs/CLAUDE.local.md.

---

## m5 — battern

**Purpose.** Record battern step-execution rows where the step shape is **optional** — never force the canonical 6-step Battern protocol shape onto unlabelled cascades.

**Upstream-IN** (per G3 § m5).
- `m1.AtuinHistoryRow` — raw history with battern-tag candidate strings.
- `m3.InjectionEvent` — chain labels that may seed `step_label`.

**Downstream-OUT.**
- `Vec<BatternStepRow { battern_id: BatternId, step_label: Option<BatternStepLabel>, step_count: usize, started_at: Timestamp, ended_at: Timestamp }>` → m7 central hub.

**Aspect-IN.**
- m8 — compile-time gate.
- m9 — write-time prefix validation (downstream m13 write).

**src/ path:** `src/m5_battern/` with `mod.rs`, `step_label.rs` (the `Option<BatternStepLabel>` enum + parse from `m3` label hint), `derive.rs` (the battern-row construction), `error.rs`.

**LOC budget** (per ULTRAMAP View 2): **~140 LOC**. ~50% boilerplate-lift (from m4 cascade-window pattern); ~50% fresh (step_label option-discipline).

**Test budget** (per TEST_DISCIPLINE matrix row m5): **55 tests**.

**Test-pattern allocation.**
- F-Unit 28 — `BatternStepRow` construction with all step_label arms (None/Design/Dispatch/Gate/Collect/Synthesize/Compose/Other), `parse_from_hint` per known + unknown labels, step_count monotonic.
- F-Property 6 — step_label round-trip: parse(emit(label)) == label; `None` preserved through serialization; `step_count` non-decreasing within a battern_id.
- F-Fuzz 0.
- F-Integration 15 — m5 ↔ m1/m3 upstream; m5 ↔ m7 join with `consumer_inputs`; concurrent batterns across sessions.
- F-Contract 3 — `BatternStepRow` schema snapshot.
- F-Regression 3 — F1 regression (any code path forcing label=Other when None was correct); 6-step-shape regression.
- F-Mutation 1 budget — ≥70%.

**Mutation kill threshold:** ≥70%.

**Boilerplate-lift source** (per cluster-B spec).
- Window-derivation pattern — adapted from m4's `window.rs` (~50% reuse).
- `BatternStepLabel` enum — fresh authorship aligned with `.claude/skills/battern-protocol/SKILL.md` 6-step naming.
- Timestamp normalisation — adapted from `memory-injection/m11_parallel_query.rs::elapsed_ms` helper (~40% reuse).

**Structural-gap LOC.** None.

**Failure-modes covered.**
- **F1 (bank/name ossification) — exclusive owner at the observation layer.** `step_label: Option<BatternStepLabel>` is the structural guarantee; m5 NEVER substitutes `Other` for `None`. Per G3 § m5 contract.
- AP-Hab-11 (hyphen-slug munge) — `BatternStepLabel::parse_from_hint` converts hyphens to underscores in slug shape before downstream m13 write.
- AP-Test-05 (assertion drift) — every test asserts a behaviour with `// rationale: F1 …` linking back; per TEST_DISCIPLINE § rationale comment rule.

**Atuin trajectory anchor.**
- `battern-protocol` (skill) — m5 is the typed-record counterpart to the skill's prose protocol; m5 ingests atuin scripts that *invoke* `battern` dispatches.
- `wt-battern-pulse` (proposed per T5.2; counts open vs closed batterns).

**Watcher class pre-position.**
- **Class A** — first `BatternStepRow` post-Genesis with `step_label = Some(Design)` (the canonical first step).
- **Class B** — hand-off boundary crossing — every battern Gate-step is a hand-off; pre-positioned per Phase 3 cross-substrate calls.
- **Class G** — substrate-frame confusion if `step_label` is back-decoded to ascribe agentic intent to substrate steps.

---

## m6 — context-cost

**Purpose.** Maintain a 20-session EMA of context-window cost per session-type, **excluding Converged outcomes from the baseline** (the F10 mitigation that prevents exploration from looking expensive).

**Upstream-IN** (per G3 § m6).
- `m1.AtuinHistoryRow` — token-count + session-type hints from atuin metadata.
- `m3.InjectionEvent` — outcome tags (`Converged | Exploring | Failed | Abandoned`) that gate EMA inclusion.

**Downstream-OUT.**
- `Vec<ContextCostBand { session_type: SessionType, ema_mean: f64, ema_variance: f64, n: usize }>` → m7 central hub.

**Aspect-IN.**
- m8 — compile-time gate.
- m9 — write-time prefix validation downstream.

**src/ path:** `src/m6_cost/` with `mod.rs`, `ema.rs` (the 20-session EMA + variance accumulator), `gate.rs` (the F10 Converged-exclusion filter), `band.rs` (`ContextCostBand` construction), `error.rs`.

**LOC budget** (per ULTRAMAP View 2): **~140 LOC**. ~30% boilerplate-lift (EMA primitive from ME v2 telemetry helpers); ~70% fresh (the F10 exclusion gate + session-type bucketing).

**Test budget** (per TEST_DISCIPLINE matrix row m6): **55 tests**.

**Test-pattern allocation.**
- F-Unit 28 — EMA initial-value handling, EMA update per `n`, variance convergence, F10 gate per outcome arm, `SessionType` parse per known arm, empty-input handling.
- F-Property 6 — EMA invariants: `ema_mean ∈ [min_input, max_input]` for all finite inputs; variance ≥ 0; EMA converges to true mean as n→∞ on stationary input; F10 gate idempotent (`gate(gate(x)) = gate(x)`).
- F-Fuzz 0.
- F-Integration 15 — m6 ↔ m1/m3 upstream; m6 ↔ m7 join; multi-session-type interleave; F10 gate end-to-end with Converged/Exploring/Failed mix.
- F-Contract 3 — `ContextCostBand` schema snapshot.
- F-Regression 3 — F10 regression (any commit that includes Converged in baseline EMA); session-type bucketing regression.
- F-Mutation 1 budget — ≥70%.

**Mutation kill threshold:** ≥70%.

**Boilerplate-lift source** (per cluster-B spec).
- EMA primitive — adapted from ME v2 telemetry exponential-moving-average helper (typical signature `fn ema_update(prev: f64, sample: f64, alpha: f64) -> f64`) — ~70% reuse, ~30% adaptation for variance accumulator.
- Session-type bucketing — adapted from `memory-injection/m07_causal_chain.rs::parse_chain_type` enum-parse pattern (~50% reuse).
- F10 gate — fresh authorship (~20 LOC); no upstream equivalent because F10 is workflow-trace-specific.

**Structural-gap LOC.** None.

**Failure-modes covered.**
- **F10 (exploration-cost preservation collapse) — exclusive owner.** EMA excludes Converged outcomes by construction; gated at `band.rs` boundary. Per G3 § m6 contract.
- F9 (workflow-grain fitness distortion) — m6's `ema_mean`/`ema_variance` are real-valued, never null; m7's `fitness_dimension REAL NOT NULL DEFAULT 0.0` schema invariant complements this.
- AP-Test-02 (property-test stub) — m6's property tests run ≥10k iters per invariant per TEST_DISCIPLINE § Property-test pattern.

**Atuin trajectory anchor.**
- `wt-cost-pulse` (proposed per T5.2; reads m6's last `ContextCostBand` for session-type cost trend).
- `habitat-bootstrap` (m6's cost bands inform the L4 bootstrap context-budget allocation).

**Watcher class pre-position.**
- **Class A** — first `ContextCostBand` emit post-Genesis.
- **Class G** — substrate-frame confusion if `ema_mean` is back-interpreted as user-effort rather than context-window cost.
- **Class H** — atuin proprioception anomaly if m6's per-session cost diverges >2σ from atuin's per-command token totals.

---

## Cluster-level synergies (which CC-1..CC-7 Cluster B participates in)

Per G3 § Cross-cluster synergies:

- **CC-1 Cascade-Cost Coupling (B internal via m7 join).** This is Cluster B's defining synergy. m4 ↔ m6 NEVER directly coupled; the join lives in m7's `consumer_inputs` JSONB column. m5 participates as a third join-input. Cluster B's contract guarantee: every emitted row carries a `session_id` that survives the m7 join.
- **CC-2 Trust Layer Woven (D → all).** m4/m5/m6 are aspect-wrapped at write-time by m9 (prefix on `cluster_id`/`battern_id`/`session_type` payloads when those propagate into m13 stcortex writes). m8 compile-gate applies. m10 Ember CI does not directly gate m4/m5/m6 (those are non-PR-text modules), but the trait audit runs over docstrings.
- **CC-3 Evidence-Driven Iteration (E → F).** Cluster B is one hop upstream of evidence: B → C (m7) → E (m14 lift) → F (m20 PrefixSpan gate). Cluster B's contract guarantee: every emitted row carries the trajectory information m20 needs to reconstruct n≥20 cohorts at the m14 sample-size gate.
- **CC-7 Pressure-Driven Evolution (E → spec).** Cluster B's failure-modes (F1/F10/F11) are the most likely trigger of m15 reservation events: if a cascade leaks pane labels, m15 fires `PHASE-B-RESERVATION-NOTICE-cascade-monoculture-leak.jsonl`; if battern step_label gets coerced from None to Other, m15 fires `PHASE-B-RESERVATION-NOTICE-battern-ossification.jsonl`; if F10 gate flips state, m15 fires `PHASE-B-RESERVATION-NOTICE-exploration-cost-collapse.jsonl`. The closure loops back to Cluster B's config (m4 hash inputs, m5 step_label parse hints, m6 F10 gate criteria) next session.

Cluster B does NOT directly participate in CC-4 (proposal→bank→dispatch), CC-5 (substrate learning loop), or CC-6 (verification-gated dispatch).

---

## Cluster-level antipatterns (subset of ANTIPATTERNS_REGISTER relevant to Cluster B)

- **AP-WT-F1 (bank/name ossification) — m5 owner.** Mitigation: `step_label: Option`; tested by F-Regression slot.
- **AP-WT-F10 (exploration-cost preservation collapse) — m6 owner.** Mitigation: EMA excludes Converged; tested by F-Property invariant + F-Regression slot.
- **AP-WT-F11 (cascade monoculture) — m4 owner.** Mitigation: FNV-1a XOR derivation; tested by F-Fuzz target + F-Property invariant + F-Regression slot.
- **AP-Hab-11 (hyphen-slug munge)** — m5's `BatternStepLabel::parse_from_hint` converts hyphens to underscores; mitigated at construction.
- **AP-V7-09 (substrate-frame engine confusion)** — Cluster B is the canonical risk zone (substrate→typed-observation boundary); Watcher Class-G pre-positioned for all three modules.
- **AP-V7-05 (module-plan-to-src-drift)** — Cluster B's LOC budget (180/140/140) must not exceed 2× at implementation; verify-sync invariant per Wave-end.
- **AP-Test-01 (coverage theatre)** — Cluster B's F-Property invariants are designed to defeat coverage theatre: opaque-ID non-leakage, EMA convergence, step_label round-trip are all properties that mutation testing would surface absence of.
- **AP-Drift-04 (test count over-report)** — Cluster B's 170-test count must verify-count at Wave-end per `cargo test --no-run --message-format=json | jq | wc -l`.

The cluster's overall risk surface is **typed-observation discipline** — every byte that crosses L2→L3 must be enum-typed-or-opaque-hashed, never carry verbatim substrate semantics that could be back-decoded downstream.

---

*cluster-B authored 2026-05-17 by Command (V7 author wave subagent)*
