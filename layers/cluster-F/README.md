---
cluster_id: F
name: Iteration (KEYSTONE)
modules: [m20, m21, m22, m23]
binary: wf-crystallise
loc_estimate: ~850
substrates_touched: [engine-internal SQLite (workflow_runs read), S-C stcortex (m23 → m13 write)]
date: 2026-05-17
status: SCAFFOLD (pre-G9; LIVE-on-G9-fire-days-5-7 — KEYSTONE)
keystone: true
---

# layers/cluster-F — Iteration KEYSTONE (operational landing)

> **Back to:** [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · sister design spec [`../../ai_specs/layers/cluster-F.md`](../../ai_specs/layers/cluster-F.md) · per-module specs [`../../ai_specs/modules/cluster-F/`](../../ai_specs/modules/cluster-F/) · synergy [`../../ai_specs/synergies/CC-3.md`](../../ai_specs/synergies/CC-3.md) · structural gap registry [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) § structural-gap authorship

## What this cluster IS

Cluster F is the **KEYSTONE iteration engine** — four modules that compose the engine's defining capability:

- **m20 PrefixSpan miner** — N-step compositional sub-graph detection (PrefixSpan algorithm; ~280 LOC pure-Rust per D-A Luke S1002127); the engine's "what patterns are emerging?" surface
- **m21 variant builder** — generate candidate workflow variants from m20 patterns + m14 lift evidence
- **m22 k-means feature** — feature-space clustering on variant candidates for diversity-aware selection
- **m23 workflow proposer** — gradient-preservation final author; emits proposals to m30 acceptance flow

Cluster F is the **KEYSTONE** because it carries the engine's only **structural-gap authorship** — the three primitives that cannot be lifted from existing boilerplate:

1. N-step compositional sub-graph detection (PrefixSpan + Levenshtein + Wilson CI)
2. `frequency × fitness × recency` compound decay (lives in m11, consumed here)
3. Unified destructiveness / EscapeSurfaceProfile schema (declared in m30 / m32 / m9; emitted here via m23)

## Modules

| Module | Concern | LOC | Spec |
|---|---|---|---|
| **m20** prefixspan_miner | PrefixSpan sub-graph mining; pure-Rust impl per D-A Luke S1002127; bench target 10k rows < 500ms / 100k < 8s | ~280 | [`m20_prefixspan_miner.md`](../../ai_specs/modules/cluster-F/m20_prefixspan_miner.md) |
| **m21** variant_builder | top-K candidate generator from PrefixSpan patterns | ~180 | [`m21_variant_builder.md`](../../ai_specs/modules/cluster-F/m21_variant_builder.md) |
| **m22** kmeans_feature | feature-space k-means for diversity scoring | ~200 | [`m22_kmeans_feature.md`](../../ai_specs/modules/cluster-F/m22_kmeans_feature.md) |
| **m23** workflow_proposer | gradient-preservation final author; emits proposals carrying AP30 namespace + EscapeSurfaceProfile + lift evidence | ~190 | [`m23_workflow_proposer.md`](../../ai_specs/modules/cluster-F/m23_workflow_proposer.md) |

## Cross-cluster contracts

- **CC-3 (Evidence-Driven Iteration, E → F)** — m14 lift evidence is m20's input weighting; m23 proposals carry m14 evidence forward to m30
- **CC-4 (Proposal → Bank → Dispatch)** — m23 is the entry point to CC-4; flows to m30 (Cluster G); substrate-coupling decomposition in [`../../ai_specs/substrate-couplings/CC-4-decomposed.md`](../../ai_specs/substrate-couplings/CC-4-decomposed.md)
- **CC-5 (substrate learning loop — receiver-end)** — Cluster F is downstream of CC-5: m31's selection cycle (Cluster G) reads updated stcortex pathway weights and shifts m20-m22's iterator inputs over days/weeks (per [`../../ai_specs/substrate-couplings/CC-5-decomposed.md`](../../ai_specs/substrate-couplings/CC-5-decomposed.md))

## Watcher class pre-position

- **Class C (refusal)** — fires on m23 emitting `OperatorRefusal` token (no acceptance signature available)
- **Class A (activation)** — fires on first m23 → m30 admission
- **Class G (gradient signal)** — fires on m20 corpus convergence stall

## Bench targets (engine's tightest)

m20 carries the engine's most expensive surface:

| Bench | Target | Schedule |
|---|---|---|
| m20 PrefixSpan 10k rows / gap=5 / n_min=20 | **< 500 ms** (commit-blocking) | PR-CI |
| m20 PrefixSpan 100k stress | **< 8 s** (Wave-end-blocking) | Wave-end + nightly |

Per [`../../ai_specs/BENCHMARK_SPEC.md`](../../ai_specs/BENCHMARK_SPEC.md) § m20 PrefixSpan bench (KEYSTONE). Flamegraph profiling mandatory at Wave-end.

## Mutation budget

Per V7 G6 § mutation budget, Cluster F carries **≥ 75% kill** mutation budget (m20 PrefixSpan is the engine's mostalgorithm-dense module; surviving mutations indicate test gaps in pattern-detection logic).

## Substrate-drift awareness (S-C indirect)

Cluster F is downstream of CC-5 substrate learning; per [`../../ai_specs/cross-cutting/substrate-drift.md`](../../ai_specs/cross-cutting/substrate-drift.md), stcortex CR-2-class drift propagates to Cluster F as **iterator-input distribution shift**. m20-m22's input weighting comes from m14 (which reads m7 which depends on m13/m42 write-side correctness). A drift episode in S-C silently changes Cluster F's behaviour without any Cluster F module returning an Err.

## Runtime concerns (post-G9; placeholder)

| Concern | Pre-G9 status |
|---|---|
| Metrics emitted | DEFERRED — `prefixspan_corpus_size`, `variant_candidates_emitted`, `kmeans_iterations_converged`, `proposals_emitted` |
| Failure-mode | EmptyCorpus / GapExceeded / TopKExceedsCorpus / ConvergenceFailed / LiftEvidenceMissing / LineageMismatch (per [`../../ai_specs/ERROR_TAXONOMY.md`](../../ai_specs/ERROR_TAXONOMY.md) § Cluster F) |
| Capacity envelope | substrate-side benches (NA-GAP-04) include S-C read-rate cap |

## Wave-1 build order (post-G9)

Cluster F ships **Days 5-7** post-G9 (KEYSTONE — most complex; depends on all prior clusters being live and tested). Order within Cluster F:

1. m20 prefixspan_miner (Day 5 — pure algorithm; isolation testable)
2. m22 kmeans_feature (Day 5-6 — also algorithmic; independent of m21)
3. m21 variant_builder (Day 6 — depends on m20 output shape)
4. m23 workflow_proposer (Day 7 — final composition; depends on all three above + m14 + m11)

## HOLD-v2 compliance

This README is markdown only. **0** `.rs` files, **0** `Cargo.toml`, **0** code under `layers/cluster-F/`. The KEYSTONE designation is the post-G9 build priority; the PrefixSpan pure-Rust implementation choice (per Luke D-A) is a planning decision that does NOT pre-author code.

---

> **Back to:** [`../../README.md`](../../README.md) · sister [`../../ai_specs/layers/cluster-F.md`](../../ai_specs/layers/cluster-F.md) · [`../../ai_specs/BENCHMARK_SPEC.md`](../../ai_specs/BENCHMARK_SPEC.md)

*Filed 2026-05-17 (S1002127 Wave 4.B audit) · Command · planning-only · HOLD-v2 compliant · KEYSTONE.*
