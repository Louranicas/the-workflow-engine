---
title: m20 — prefixspan_miner (Cluster F · L6 · KEYSTONE)
module_id: m20
module_name: prefixspan_miner
cluster: F — Iteration (KEYSTONE)
layer: L6
binary: wf-crystallise
feature_gate: [intelligence]
loc_estimate: 300
test_count_min: 75
test_kinds: [property, bench]
verb_class: active
gap_owner: [Gap 1]
mutation_kill_target: 80%
status: planning-only · HOLD-v2 active · pre-G7 Zen audit
authority: Luke @ node 0.A (single-phase override 2026-05-17)
date: 2026-05-17 (S1001982)
kind: per-module spec
decisions_applied: [D-A]
---

> **🏷 v0.1.0 — SD A/B reconciliation (S1004115 Phase 9 / § 15 D27):**
> The shipped m20 implementation is canonical. **SD12** (m20
> stabilization gate absent) is load-bearing: the m23 F2 evidence floor
> (`PROPOSAL_F2_THRESHOLD = 20`) is the actual stabilization mechanism
> the spec's "stabilization" refers to. m20 emits patterns above
> `min_support`; m23 is the F2 gatekeeper that refuses below-threshold
> proposals. Hardening Fleet W2 rewrote `project_after_prefix` from a
> greedy single-pass into a correct backtracking gap-bounded matcher
> with failure memoisation (KEYSTONE bug fix). Spec amendments mirror
> the shipped surface; no behavioural divergence remains. Full
> disposition: [`PHASE9_SD_RECONCILIATION_S1004115.md`](../../../ai_docs/PHASE9_SD_RECONCILIATION_S1004115.md).

# m20 — `prefixspan_miner`

> **Sister modules (Cluster F):** [m20](m20_prefixspan_miner.md) · [m21](m21_variant_builder.md) · [m22](m22_kmeans_feature.md) · [m23](m23_workflow_proposer.md)
>
> **Vault:** [[cluster-F-iteration]] · **V7 plan:** [cluster-F plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md) · **Cluster spec:** [`../../../the-workflow-engine-vault/module specs/cluster-F-iteration.md`](../../../the-workflow-engine-vault/module%20specs/cluster-F-iteration.md) · **Matrix row:** [MODULE_MATRIX m20](../../MODULE_MATRIX.md)
>
> **Back to:** [project CLAUDE.md](../../../CLAUDE.md) · [GENESIS v1.3 § 1](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · [PATTERNS](../../../PATTERNS.md) · [GOLD_STANDARDS](../../../GOLD_STANDARDS.md) · [ANTIPATTERNS](../../../ANTIPATTERNS.md)

---

## 1. Purpose

m20 is **the KEYSTONE of `workflow-trace`**. It mines **frequent sequential patterns with gap-allowed matching** from observed cascade and Battern step sequences, emitting typed `Pattern { steps: Vec<StepToken>, support: usize, gap_bounds: (usize, usize) }` artefacts for each pattern meeting the `min_support` floor under bounded right-gap `MAX_GAP_STEPS=5`.

This is the structural-gap authorship called out in the Boilerplate Hunt and again in [GOD_TIER_CONSOLIDATION Part III Gap 1](../../../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md): the N-step compositional sub-graph detection that cannot be lifted from existing habitat sources because the habitat's pairwise infrastructure (`CoActivationPair { a, b, ts_ms }` in POVM) only covers two elements at a time. Generalising from pairwise to N-step is fresh authorship (~250-280 LOC of m20's ~300 LOC budget) — m20 owns the largest share of the Cluster F ~600-700 LOC keystone fresh-authorship budget.

m20 is also the engine's **reason-for-existing**: without N-step pattern mining, `workflow-trace` is an observatory, not an engine. Every downstream module (m21 variants, m22 K-means feature context, m23 proposer composition, m30 bank admission, m32 dispatch) consumes m20's output transitively.

---

## 2. Algorithm Selection — Why PrefixSpan

Three candidates were evaluated for the N-step compositional sub-graph problem:

**Apriori** (REJECTED). Level-wise candidate generation. For a 26-module workflow-trace with an estimated 40-80 distinct `StepToken` alphabet, 4-step patterns would generate O(80⁴) ≈ 40M candidates. Each level requires a database pass. The candidate-explosion is structural to the algorithm and not tunable.

**N-gram sliding window** (REJECTED as primary). Treats sequences as strings, counts n-grams. Fast, simple, but **cannot skip gaps**. A cascade `[read_file → bash → edit → bash → cargo_check]` will not match the pattern of interest `[read_file → edit → cargo_check]` because the bash interleavings break the window. Gap-allowed matching is a hard requirement for cascade cluster detection (cascades regularly interleave monitoring calls between meaningful steps). Retained as an optional fingerprinting pre-filter inside PrefixSpan to accelerate candidate pruning.

**PrefixSpan** (CHOSEN — Pei et al. 2001 "PrefixSpan: Mining Sequential Patterns Efficiently"). Projection-based frequent sequential pattern mining. The algorithm:

1. Scans the database once to find frequent length-1 items (L1 candidates) meeting `min_support`.
2. For each frequent prefix P, **projects** the database — for each sequence containing P, retains only the suffix following the first match of P (under gap-allowed semantics).
3. Recurses on the projected database to find frequent length-(L+1) extensions.

**Why PrefixSpan over the alternatives:**

- **No candidate explosion.** Projection bounds the search space to actually-observed extensions.
- **Gap-allowed matching is intrinsic.** The projection step naturally handles gaps via post-occurrence semantics: given prefix P, the projected database contains suffixes following the first occurrence of P's last item.
- **Exact frequency support.** No probabilistic approximation; counts are exact.
- **Reference implementations exist** in academic literature, enabling **differential testing** against a naive O(n³) reference (see § 8).

**Complexity:** O(|D| × L × W) where |D| = number of sequences, L = max pattern length (capped at 8), W = average sequence width (typically 6-20 tool calls per cascade cluster). Bounded depth + bounded width → practical single-pass per recursion level.

---

## 3. Inputs / Outputs / Substrates

**Reads from** (per [MODULE_MATRIX m20 row](../../MODULE_MATRIX.md)): `m5 iter` (Battern step records ordered) and m4 cascade cluster step-lists. Concretely:

- `workflow_trace.db` table `cascade_clusters` (written by m4) — ordered `Vec<StepToken>` per cluster ID
- `workflow_trace.db` table `battern_step_records` (written by m5) — ordered Battern-step sequences
- `m14.Lift` (CC-3 gate) — Wilson-bounded `Option<Lift>` evidence stability check

**Writes to:** `m21 input` — `Vec<Pattern>` consumed in-memory by m21 (variant builder) and m23 (proposer). No SQLite write in this module; the proposer surfaces persist to `workflow_trace.db` table `proposals`.

**Substrates:** None at the H-cluster level; m20 is upstream of substrate-feedback. m20 NEVER writes to atuin, stcortex, injection.db, POVM, or any other habitat substrate. m13 (Cluster C) owns all stcortex writes.

**CC-3 (E → F) gate:** Before each invocation, m20 reads m14's `habitat_outcome_lift` metric and checks variance over the last 6 measurements is below `STABILIZATION_VARIANCE_THRESHOLD` (default proposed 0.05; final value G5 interview). If still trending, m20 returns `MinerError::StabilizationGateNotMet { variance, threshold }` and does not mine. This honours the F2 spirit beyond the bare n≥20 floor.

**Verb class:** `active` — single-phase override per [GENESIS v1.3 § 3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) table row m20 permits the verbs `mine` and `extract`, bounded by `MAX_GAP_STEPS=5` and the Wilson CI gate at the `ProposalBuilder` boundary (m23).

---

## 4. Public API (planning-spec; markdown only)

The following Rust fenced block is **planning-spec only** — NOT compileable source. Documents the intended public surface for Zen G7 audit and Command-2 build-executor reference at Wave 3.

```rust
// planning-spec only — m20_prefixspan_miner public surface
// rationale: opaque StepToken alphabet (F11 cascade-monoculture mitigation)
//            + deterministic Pattern ordering (sort-by-support-DESC then length-DESC)
//            + Result-typed refusal on F2 / CC-3 / empty-DB conditions

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Opaque step token. Numeric, not human-readable. Display labels resolved at
/// report-emit time only by m12 via the StepTypeRegistry. F11 compliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StepToken(pub u32);

/// A gap-allowed sequential pattern emitted by m20.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pattern {
    pub steps: Vec<StepToken>,
    pub support: usize,
    /// (min_left_gap, max_right_gap) observed when matching this pattern.
    pub gap_bounds: (usize, usize),
    /// Stable hash for cross-module identity (used by m21, m23).
    pub canonical_hash: u64,
}

/// Newtype for min-support floor; prevents accidental u64 mixing with other counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinSupport(pub usize);

/// Newtype for max-gap config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxGap(pub usize);

#[derive(Debug, Error)]
pub enum MinerError {
    #[error("database read failed: {source}")]
    DatabaseRead { #[source] source: rusqlite::Error },

    #[error("empty database: no cascade clusters or battern step records to mine")]
    EmptyDatabase,

    #[error("pattern too long: {len} > {max}")]
    PatternTooLong { len: usize, max: usize },

    #[error("m14 stabilization gate not met: variance {variance:.4} > threshold {threshold:.4}")]
    StabilizationGateNotMet { variance: f64, threshold: f64 },

    #[error("min_support floor {0} below F2 hard minimum of 2")]
    MinSupportBelowFloor(usize),
}

/// Mine frequent sequential patterns. Single public entry point.
///
/// # Errors
///
/// - `EmptyDatabase` if no sequences were read.
/// - `StabilizationGateNotMet` if CC-3 gate is open.
/// - `DatabaseRead` on rusqlite failure.
///
/// # Determinism
///
/// Output is sorted by `support` DESC, then `steps.len()` DESC, then
/// `canonical_hash` ASC. Identical input → identical output across runs.
pub fn mine_patterns(
    db: &WorkflowDb,
    min_support: MinSupport,
    max_gap: MaxGap,
    max_length: usize,
) -> Result<Vec<Pattern>, MinerError> { /* algorithm body — see § 5 */ }
```

---

## 5. Algorithm Sketch (planning-spec only)

The PrefixSpan kernel is bounded-right-gap, leftmost-first match, with stable deterministic ordering. The pseudocode below is **planning-spec only** (markdown only; no `.rs` authored).

```rust
// planning-spec only — m20_prefixspan_miner::algorithm
// Reference: Pei et al. 2001 "PrefixSpan: Mining Sequential Patterns Efficiently"

fn prefix_span(
    db: &[Sequence],
    prefix: &[StepToken],
    min_support: MinSupport,
    max_gap: MaxGap,
    max_depth: usize,
    out: &mut Vec<Pattern>,
) {
    if prefix.len() >= max_depth { return; }
    // 1. Frequency of each item in the projected DB
    let counts: HashMap<StepToken, usize> = frequency_map(db);
    // 2. For each item meeting min_support, extend the prefix and recurse
    for (item, support) in counts.into_iter().filter(|(_, c)| *c >= min_support.0) {
        let mut new_prefix = prefix.to_vec();
        new_prefix.push(item);
        let projected = project(db, item, max_gap);
        out.push(Pattern {
            steps: new_prefix.clone(),
            support,
            gap_bounds: observed_gap_bounds(&projected),
            canonical_hash: fnv1a_xor_hash(&new_prefix),
        });
        prefix_span(&projected, &new_prefix, min_support, max_gap, max_depth, out);
    }
}

fn project(db: &[Sequence], item: StepToken, max_gap: MaxGap) -> Vec<Sequence> {
    // For each sequence: find leftmost occurrence of `item` within max_gap.0
    // positions of the cursor; return the suffix after that occurrence.
    db.iter().filter_map(|seq| {
        let upper = (seq.cursor + max_gap.0 + 1).min(seq.tokens.len());
        seq.tokens[seq.cursor..upper].iter().position(|t| *t == item)
            .map(|offset| seq.advance(offset + 1))
    }).collect()
}
```

**Gap-Allowed Matching Model (locked):**

- **Unbounded left-gap** on the first prefix item — the pattern can begin anywhere in the sequence.
- **Bounded right-gap** of `MAX_GAP_STEPS` (default 5) on each subsequent prefix item — the next prefix item must appear within 5 positions of the previous match, else the match breaks.
- **Leftmost-first** — when multiple positions match, the leftmost is taken (deterministic).

The gap constraint is enforced inside `project`: when scanning for `item`, the scan is limited to `MAX_GAP_STEPS` positions forward from the cursor. This prevents spurious matches where two prefix items are separated by half the session.

---

## 6. Boilerplate Lift / New Authorship

Per [GOD_TIER_CONSOLIDATION Part IV Category 04](../../../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md):

| Component | Source | Reuse | New |
|---|---|---|---|
| Kahn's topological sort scaffold | `m49_task_graph.rs` | 50% | adapted to sequence-projection |
| Rolling-mean accumulator | `m39_fitness_tensor.rs` | 30% | support-counting only |
| Pairwise co-activation API shape | POVM v2 `CoActivationPair` | 30% | generalised pairwise → N-step (KEYSTONE) |
| PrefixSpan kernel (projection + gap_bounds) | none | 0% | **~250 LOC fresh** |
| Differential reference (naive O(n³)) | none | 0% | ~50 LOC test-only |
| StepToken opaque type + canonical_hash | none | 0% | ~30 LOC fresh |

**Total fresh authorship for m20:** ~250-280 LOC of the cluster's ~600-700 LOC Gap 1 budget. **Lifted:** ~70-100 LOC scaffolding only.

The ratio is inverted from most habitat modules. This is structural to the keystone: the pairwise→N-step generalisation cannot be lifted because the source pairwise infrastructure is structurally a different algorithm shape.

---

## 7. F2 Enforcement and Gap-Bounds Semantics

F2 hard gate (from [cluster-F-iteration spec](../../../the-workflow-engine-vault/module%20specs/cluster-F-iteration.md) § F2 Hard Gate): every `Pattern` emitted with `support >= min_support` and `min_support >= 2`. m20's responsibility ends at the pattern; m23's `ProposalBuilder::build()` enforces the harder `n_samples >= 20` Wilson CI gate before any `Proposal` reaches the human review queue.

m20 returns `MinSupportBelowFloor(n)` if a caller requests `min_support < 2` — there is no bypass. The 2-floor is structural; downstream Wilson CI computation has different bounds (n >= 20 for proposal admission, n >= 5 for deviation-shaped variants) but the mining-time floor is always 2.

`gap_bounds: (min_left_gap, max_right_gap)` are **observed** per pattern across the projected database. m21 (variant builder) consumes these to inform variant construction. m23 includes them in the `Proposal.canonical` payload for human review context.

---

## 8. Tests (target 75 — KEYSTONE allowance above default 50)

Per [MODULE_MATRIX m20 row](../../MODULE_MATRIX.md): `test_count_min: 75`, kinds `property+bench`.

Per [V7 cluster-F plan § m20 test budget](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md):

| Family | Count | Notes |
|---|---:|---|
| F-Unit | 45 | per-arm: 3 per public fn × 15 fns; per-StepToken-variant coverage |
| F-Property | 10 | invariants (see below) |
| F-Fuzz | 1 target / 24h budget | `m20_prefixspan` fuzz target; arbitrary `Vec<Vec<StepToken>>` must never panic |
| F-Integration | 20 | m4/m5/m14 upstream wiring; m23 downstream handshake; CC-3 closure exercise |
| F-Contract | 5 | Pattern serde roundtrip stable across versions; `ProposalBuilder` consumer contract |
| F-Regression | 8 | pre-seeded bug classes (see below) |
| F-Differential | embedded | naive O(n³) reference; randomised input equality modulo output ordering |
| F-Mutation | ≥80% kill rate | KEYSTONE threshold per G6 |
| **Total** | **≥75** | (90 allocated for keystone) |

**Property invariants (proptest):**

1. `output.is_sorted_by(|a, b| b.support.cmp(&a.support).then(b.steps.len().cmp(&a.steps.len())))` always
2. `mine_patterns(db, k, g, l) == mine_patterns(db, k, g, l)` (idempotent re-mine)
3. Every pattern's `support == count_matches(db, &pattern.steps, max_gap)` (exact match count)
4. Increasing `min_support` is monotonic — `mine(k1) ⊇ mine(k2)` when `k1 <= k2`
5. `gap_bounds.1 <= MAX_GAP_STEPS` always
6. Empty database → `Err(EmptyDatabase)` always, never panic

**Pre-seeded regression bug classes:**

- Zero-support pattern emit (must not happen — `min_support >= 2` floor)
- Gap-bounds overflow (large sequences must not panic)
- Empty-sequence handling (one empty sequence among many — must not skew counts)
- Duplicate-pattern emit (same `canonical_hash` twice — must dedupe)
- StepToken u32 wraparound (counts must not overflow at 4B-step input)
- Off-by-one in projection cursor (catches single-position miss)
- Differential equality (naive vs PrefixSpan output equal modulo ordering)
- Cancellation safety (mining is `Send + 'static`; cancellable via cooperative `tokio::select!`)

**Bench:** `criterion` target for `mine_patterns` at:

- 10k atuin rows (typical workload — must complete in < 500ms)
- 100k stress (must complete in < 10s; document scaling factor)
- 1M synthetic stress (smoke; document any panics — should be none)

---

## 9. Antipatterns Avoided

| Antipattern | Mitigation in m20 |
|---|---|
| **AP-V7-03** (verb collapse) | m20 mines / extracts; never recommends / dispatches / promotes. Output is `Vec<Pattern>` only. |
| **AP-V7-07** (auto-promotion m23→m30) | m20 does not write to m30 at all; cannot bypass human review even by accident. |
| **AP-V7-13** (diagnostics theatre) | bench output is paired with live `cargo bench` evidence at G6; no refresh-stamp without re-run. |
| **AP-Drift-03** (scaffold without binary-wiring) | Wave 3 integration test exercises m20 → m21 → m23 → m30 full chain. |
| **AP-Test-01** (coverage theatre) | mutation ≥80% kill + differential test against naive reference. |
| **F11** (cascade monoculture) | m20 consumes `StepToken(u32)` only; never sees human-readable cascade names. |
| **AP30** (namespace string drift) | m20 reads from `workflow_trace_*` namespace constants; never literals. |
| **AP-V7-11** (Phase A active verbs) | m20 is Phase B (Cluster F) under single-phase override; verb permitted per [GENESIS v1.3 § 3](../../../ai_docs/GENESIS_PROMPT_V1_3.md). |

---

## 10. Substrate / Watcher Class Pre-Position

- **Class A** (activation transition) at first PrefixSpan invocation post-G9 — the moment the KEYSTONE goes live is the highest-leverage observation moment per [GOD_TIER_CONSOLIDATION Part VII](../../../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md).
- **Class G** (substrate-frame confusion) if m20 spec language or implementation drifts toward "search for what the user wants" rather than "mine frequent sub-graphs of observed substrate sequences." Watcher pre-positioned to flag anthropocentric framing in m20 docs at G7 Zen audit.
- **Class I** (Hebbian silence) if m20 emits ≥5 patterns per week but stcortex `learning_health` does not trend up — CC-5 broken downstream of m20 (not m20's fault, but m20's emission rate is the upstream signal).
- **Class C** (refusal) at every `EmptyDatabase` or `StabilizationGateNotMet` — refusal IS the correct behaviour; Watcher logs as healthy.

**Atuin trajectory anchor:** `wt-prefixspan-replay` (proposed; T5.2) captures `(input_sequence_hash, output_pattern_count, max_gap_observed, mining_duration_ms)` per invocation. Trend of `output_pattern_count / input_sequence_count` is the engine's **pattern-discovery rate** — a primary substrate signal monitored at Phase 5C synthesis.

---

## 11. Open Questions (Zen G7 audit + G5 spec interview)

1. **PrefixSpan implementation choice — pure-Rust vs C-FFI? — DECIDED.** **DECIDED 2026-05-17 (S1002127, Luke):** Pure Rust authored from scratch (~280 LOC fresh). Rationale: no mature pure-Rust PrefixSpan crate exists; C-FFI introduces AP29-class hazards (sync HTTP / FFI in tokio::spawn); Python-port adds deployment complexity. KEYSTONE deserves owned implementation. Lift status (§ 6) confirmed at 0%.
2. **MAX_GAP_STEPS default (currently 5).** Cascade traces typically interleave 1-3 monitoring calls between meaningful steps; gap of 5 gives margin. But may permit spurious matches in long sessions. Should this be per-cluster-type configurable, or one global default? G5 interview required.
3. **Differential reference scope.** Naive O(n³) reference must implement the same gap-allowed semantics. Should it be included in production binary behind `#[cfg(test)]` only, or behind a `feature = "differential"` flag for runtime self-check?
4. **CC-3 stabilization variance threshold (proposed 0.05).** What variance on `habitat_outcome_lift` constitutes stable? The metric's natural variance is not yet known; this requires empirical calibration after the first 30-120d of m14 observation.
5. **Cancellation discipline.** Long mining runs must be cancellable cooperatively. Should `mine_patterns` take `&CancellationToken`, or rely on caller wrapping in `tokio::select!`? Recommendation: explicit `&CancellationToken` parameter for ergonomic call sites.
6. **Max pattern length cap (currently 8).** Bounded for complexity; but cascades can in principle be longer. Should there be an overflow signal (`PatternTooLong` warning vs error)?

---

## 12. Synergy / Sister-Module Anchors

- **m21** ([m21_variant_builder.md](m21_variant_builder.md)) — consumes m20's `Vec<Pattern>` to build top-K near-miss variants by Levenshtein edit distance.
- **m22** ([m22_kmeans_feature.md](m22_kmeans_feature.md)) — orthogonal consumer; uses workflow run features (cost variance, step diversity, fitness dimension) for cluster context — does NOT consume m20 patterns directly.
- **m23** ([m23_workflow_proposer.md](m23_workflow_proposer.md)) — composes m20 canonical + m21 variants + m22 feature context + m14 lift into typed `WorkflowProposal` artefacts under F2 construction-time gate.
- **m14** (Cluster E `habitat_outcome_lift`) — CC-3 upstream gate; m20 reads `Option<Lift>` for stabilization check.
- **m4 / m5** (Cluster B observers) — primary input substrate via `workflow_trace.db` cluster_clusters + battern_step_records tables.
- **m23 → m30** (Cluster G) — **m23 NEVER auto-promotes to m30** (AP-V7-07). m20's output is two transitive hops from the bank.

---

## 13. Verification Trail

- **Frontmatter complete:** ✓ cluster, layer, binary, feature_gate, gap_owner, test_kinds, verb_class, test_count_min
- **Cluster ownership:** Cluster F · L6 · KEYSTONE
- **Gap ownership:** Gap 1 (N-step compositional sub-graph detection) — ~250-280 LOC fresh of cluster ~600-700 LOC budget
- **Cross-cluster contracts:** CC-3 (E → F upstream) consumed; CC-4 (F → G) downstream via m21/m23
- **F2 gate referenced:** § 7 (m20 emits `support >= 2` floor; m23 ProposalBuilder enforces `n_samples >= 20`)
- **AP-V7-07 (no auto-promote):** § 9, § 12
- **Bidi anchors:** Sister modules (4) · Vault (1) · V7 plan (1) · Cluster spec (1) · Matrix row (1) · GENESIS v1.3 (1) · PATTERNS / GOLD_STANDARDS / ANTIPATTERNS (3) — ✓
- **Word count:** ~2,300 (within KEYSTONE 1,500-2,500 range)
- **No `.rs` files authored:** ✓ planning-only, HOLD-v2 respected
- **Rust fenced blocks are spec documentation:** ✓ labelled "planning-spec only" inline

*m20 spec authored 2026-05-17 (S1001982). Cluster F KEYSTONE — owns the largest share of Gap 1 fresh-authorship budget. Planning-only per HOLD-v2 + AP24. Pre-G7 Zen audit.*
