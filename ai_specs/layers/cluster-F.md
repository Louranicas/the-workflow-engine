---
title: Cluster F — Iteration KEYSTONE (Layer L6) — Layer Spec
cluster: F
layer: L6
module_count: 4
modules: [m20_prefixspan_miner, m21_variant_builder, m22_kmeans_feature, m23_workflow_proposer]
binary: wf-crystallise
feature_gates: [intelligence]
cc_owns: [CC-4 (Proposal→Bank→Dispatch entry; m23 owns)]
cc_consumes: [CC-1, CC-2, CC-3, CC-5]
ship_priority: Day 3 Wave 3 (KEYSTONE — after Cluster E evidence lands)
status: SPEC
date: 2026-05-17
hold_v2_compliant: true
---

# Cluster F — Iteration KEYSTONE (L6)

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · vault [[cluster-F-iteration]] · [`../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md)

## Role

Cluster F IS **the iteration KEYSTONE** — the four-module pipeline that turns Cluster B observations + Cluster E evidence into typed `WorkflowProposal` artefacts for human review. This is the cluster Genesis v1.3 § 1 flags as "structural-gap authorship (cannot be lifted)": the N-step compositional sub-graph detection (PrefixSpan + Levenshtein + Wilson CI) is novel arithmetic, ~600-1,000 LOC, the largest engineering surface in workflow-trace. It is also the most fragile — F2 (sample-size inflation), F11 (cascade monoculture), and F8 (feedback-loop poisoning) all attack here. The cluster's structural mitigations are CC-3 (n≥20 gate at construction), F11 (opaque IDs from m4), and m23's `ProposalBuilder::build()` returning `Result<WorkflowProposal, ProposalError::LiftEvidenceMissing>` — the gate is at *construction*, not at runtime; you cannot build a proposal without evidence.

The four modules form a strict pipeline. **m20** runs the PrefixSpan sequential-pattern miner over m5 battern trajectories with `MAX_GAP_STEPS=5` and emits frequent-pattern candidates (n≥20 gated). **m21** takes the candidates and builds Levenshtein-distance variants top-K, expanding the search surface without losing the gradient. **m22** runs K-means feature clustering over `(m21 variant features, m14 lift signal)` to group structurally-similar variants. **m23** is the `ProposalBuilder` — the construction-time gate where Wilson-CI evidence is required (`lift.ok_or(ProposalError::LiftEvidenceMissing)?`) and a `WorkflowProposal` artefact emerges. m23 does NOT promote into m30; the human-review boundary is the F5 mitigation explicitly preserved in Genesis v1.3 § 2.

## Modules

| Module | Spec | LOC | Tests | Verb |
|---|---|---:|---:|---|
| `m20_prefixspan_miner` | [`../modules/cluster-F/m20_prefixspan_miner.md`](../modules/cluster-F/m20_prefixspan_miner.md) | 300 | **90** (KEYSTONE) | mine / extract |
| `m21_variant_builder` | [`../modules/cluster-F/m21_variant_builder.md`](../modules/cluster-F/m21_variant_builder.md) | 200 | 70 | propose / build variants |
| `m22_kmeans_feature` | [`../modules/cluster-F/m22_kmeans_feature.md`](../modules/cluster-F/m22_kmeans_feature.md) | 170 | 60 | cluster |
| `m23_workflow_proposer` | [`../modules/cluster-F/m23_workflow_proposer.md`](../modules/cluster-F/m23_workflow_proposer.md) | 180 | 60 | aggregate / preserve gradient |

## Cross-cluster contracts

- **OWNS:**
  - **CC-4 (Proposal→Bank→Dispatch Pipeline, entry-point)** — m23's emitted `WorkflowProposal` is the input to the human-review boundary; downstream m30 (Cluster G) admits only the survivors. See [`../synergies/CC-4.md`](../synergies/CC-4.md).
- **CONSUMES:**
  - **CC-1** — m20 reads m5 trajectories that surfaced through m7's JSONB join; m22 reads m6 cost-bands the same way.
  - **CC-2** — m9 wraps any namespace-bearing string m23 emits (proposal-id namespace is `workflow_trace_proposal_*`).
  - **CC-3 (PRIMARY consumer)** — m14's `Option<Lift>` is the construction-time gate. `m23::ProposalBuilder::build()` performs `let lift = self.lift.ok_or(ProposalError::LiftEvidenceMissing)?;` — there is no runtime bypass.
  - **CC-5 (read-back side)** — over weeks, the substrate-feedback loop changes which cluster_ids have lift; m20-m22 inputs shift; m23 emits substrate-shaped proposals. See [`../synergies/CC-5.md`](../synergies/CC-5.md).

## Binary placement

All four modules in **`wf-crystallise`**. m23 emits proposal artefacts to a review queue (CLI surface in m12); humans then run `wf-dispatch bank accept` to admit. Cluster F is read+propose only.

## Feature-gate posture

**`feature = "intelligence"`** gated. Cluster F is the iteration KEYSTONE — turning intelligence off produces a "passive recorder" build with no proposal generation. Useful for debug, mandatory for production.

## Ship priority

**Day 3 Wave 3 — KEYSTONE.** Implementation order: m20 (the largest, most-tested module — 90 tests including KEYSTONE bench + property + fuzz) → m21 (Levenshtein top-K, depends on m20 output schema) → m22 (K-means, depends on m21 variant features + m14 lift) → m23 (the final aggregator + construction-time gate; smallest LOC but largest contract surface). m20 must land first so m21's test corpus can use m20-emitted patterns.

## Operational invariants

1. **n≥20 gate at construction (CC-3 / F2 mitigation).** `m23::ProposalBuilder::build()` requires a non-`None` `Lift`; below n=20 m14 returns `None`; m23 returns `LiftEvidenceMissing`. There is no runtime bypass; no `force_build` method.
2. **MAX_GAP_STEPS=5 hard cap in m20.** PrefixSpan with unbounded gap explodes combinatorially; the cap is a Genesis v1.3 § 3 m20 row binding constant. Tests assert that gap=6 patterns are silently rejected from the iterator.
3. **Opaque IDs propagate end-to-end (F11).** m4's opaque `cluster_id` flows through m20-m23; no module re-names with a human-meaningful label. Test invariant: `rg '"[a-z]+_[a-z]+"' src/m23_workflow_proposer/ | grep -v 'cluster_id\|battern_id'` audit.
4. **Gradient preservation in m23 (F10).** m23 emits N near-miss variants alongside the canonical proposal; collapsing to a single canonical proposal is F10 violation. The `WorkflowProposal { canonical, variants: Vec<Variant> }` shape is contract-binding.
5. **No auto-promote (F5 / AP-V7-07).** m23 produces `WorkflowProposal` artefacts; emission is to a CLI review queue (m12 surface). m23 NEVER calls `m30::accept` directly; the human-review boundary is structural — there is no code path bridging m23 → m30.

## Failure modes the cluster structurally refuses

- **PrefixSpan with unbounded gap.** Combinatorial explosion at gap=10+; m20 hard-caps at 5.
- **Wilson-CI bypass via "if no lift, use prior".** m23 returns `LiftEvidenceMissing` instead of falling back to a Bayesian prior; falling back would silently feed Cluster G with un-evidenced proposals.
- **m23 calling `m30::accept` directly.** Hard refusal — `m30::accept` requires `HumanAcceptanceSignature` and m23 cannot construct one (constructor is private to the `wf-dispatch` CLI).
- **Variant collapse to single canonical.** Emitting only `Variant { score: highest }` and dropping near-misses violates F10 gradient preservation. Tests assert N≥3 variants emitted when m22 produces ≥3 cluster-mates.
- **Pattern-naming.** Any string-literal pattern label outside `tests/` is a F11 violation.

## Performance envelope

| Operation | Target | Notes |
|---|---|---|
| m20 PrefixSpan (10k rows) | < 500 ms | criterion bench; KEYSTONE — see [`../BENCHMARK_SPEC.md`](../BENCHMARK_SPEC.md) |
| m20 PrefixSpan (100k stress) | < 8 s | criterion bench; max-realistic input size |
| m21 Levenshtein top-K (K=10) | < 100 ms | per pattern; K bounded |
| m22 K-means (k=5, 200 features) | < 200 ms | convergence at iter≤20 |
| m23 `ProposalBuilder::build` | < 50 ms | mostly serde + Wilson-CI check |

## Verify-sync invariants

- **#4** — m20 has ≥75 tests, m21/m22/m23 ≥60 each per [`../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) matrix.
- **#16** — F2 mitigation test asserts `ProposalBuilder::build()` returns `LiftEvidenceMissing` when m14 produced `None`.
- **#17** — `tests/integration/cc3_evidence_driven_iteration.rs` and `tests/integration/cc4_proposal_bank_dispatch.rs` exist.
- **#20** — m23 has no `m30::accept` call path; verified by `rg 'BankDb::accept\|bank\.accept' src/m23_workflow_proposer/` returning 0.

## Per-module cross-links

- [`../modules/cluster-F/m20_prefixspan_miner.md`](../modules/cluster-F/m20_prefixspan_miner.md) — PrefixSpan sequential pattern miner (KEYSTONE)
- [`../modules/cluster-F/m21_variant_builder.md`](../modules/cluster-F/m21_variant_builder.md) — Levenshtein top-K variant generator
- [`../modules/cluster-F/m22_kmeans_feature.md`](../modules/cluster-F/m22_kmeans_feature.md) — K-means feature clusterer
- [`../modules/cluster-F/m23_workflow_proposer.md`](../modules/cluster-F/m23_workflow_proposer.md) — ProposalBuilder + Wilson-CI gate

## Antipatterns specific to Cluster F

- **AP-WT-F2** (sample-size inflation) — m23 construction-time gate; not runtime.
- **AP-WT-F5** (bank creep) — m23 NEVER auto-promotes; the m30 admission path requires interactive human signature.
- **AP-WT-F10** (gradient preservation collapse) — m23 emits N variants.
- **AP-WT-F11** (cascade monoculture) — opaque IDs end-to-end.
- **AP-V7-07** (auto-promote m23 → m30) — Genesis v1.3 § 2 forbids; structural refusal.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · vault [[cluster-F-iteration]]
