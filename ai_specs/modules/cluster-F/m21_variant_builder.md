---
title: m21 — variant_builder (Cluster F · L6 · KEYSTONE)
module_id: m21
module_name: variant_builder
cluster: F — Iteration (KEYSTONE)
layer: L6
binary: wf-crystallise
feature_gate: [intelligence]
loc_estimate: 200
test_count_min: 65
test_kinds: [property]
verb_class: active
gap_owner: [Gap 1]
mutation_kill_target: 75%
status: planning-only · HOLD-v2 active · pre-G7 Zen audit
authority: Luke @ node 0.A (single-phase override 2026-05-17)
date: 2026-05-17 (S1001982)
kind: per-module spec
decisions_applied: [D-A]
---

# m21 — `variant_builder`

> **Sister modules (Cluster F):** [m20](m20_prefixspan_miner.md) · [m21](m21_variant_builder.md) · [m22](m22_kmeans_feature.md) · [m23](m23_workflow_proposer.md)
>
> **Vault:** [[cluster-F-iteration]] · **V7 plan:** [cluster-F plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md) · **Cluster spec:** [`../../../the-workflow-engine-vault/module specs/cluster-F-iteration.md`](../../../the-workflow-engine-vault/module%20specs/cluster-F-iteration.md) · **Matrix row:** [MODULE_MATRIX m21](../../MODULE_MATRIX.md)
>
> **Back to:** [project CLAUDE.md](../../../CLAUDE.md) · [GENESIS v1.3 § 1 + § 3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · [PATTERNS](../../../PATTERNS.md) · [GOLD_STANDARDS](../../../GOLD_STANDARDS.md) · [ANTIPATTERNS](../../../ANTIPATTERNS.md)

---

## 1. Purpose

m21 is the **variant-construction half of the KEYSTONE**. For each canonical `Pattern` emitted by m20 (PrefixSpan), m21 builds a **deterministic top-K set of near-miss variants** ranked by **ascending normalized Levenshtein edit distance** computed on `StepToken` sequences.

Variants are the engine's **exploration surface** — they preserve solution-space topology around canonical patterns by surfacing the N structurally-nearest alternatives in step-token space, allowing m23 (proposer) and ultimately the human reviewer to see **gradient-shaped** alternatives rather than random walks. Per [GOD_TIER_CONSOLIDATION Part I § Cluster F](../../../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md):

> Same-pattern threshold: normalized Levenshtein **< 0.25**. Near-miss band **0.25-0.60**. Top-K-by-ascending-edit-distance (**N=3 default**). Random sampling rejected as non-reproducible.

m21 owns the second-largest share of Gap 1 (the keystone): ~150 LOC of fresh authorship for **Levenshtein-on-`StepToken`-sequences** (reference Levenshtein implementations operate on character sequences; m21's variant honours structural `StepToken` equality semantics, not byte equality).

---

## 2. Variant Construction Model

A **near-miss variant** of canonical `Pattern P` is any other observed `Pattern Q` whose normalized Levenshtein edit distance from P falls in the **near-miss band [0.25, 0.60]**.

- **< 0.25:** "same pattern" — Q is a structural duplicate of P; not a variant; collapsed at canonical layer (m20 already dedupes by `canonical_hash`).
- **[0.25, 0.60):** "near-miss" — Q differs from P by a small number of step-token edits; this is the gradient-information band.
- **>= 0.60:** "distinct" — Q is too structurally dissimilar from P to be informative as a variant.

The construction is **deterministic top-K-by-ascending-edit-distance**: given the band-filtered candidate set, m21 sorts by edit distance ascending (ties broken by canonical-id then lexicographic `Vec<StepToken>`), and keeps the first K (default K=3 per P0 #10 in [cluster-F-iteration spec](../../../the-workflow-engine-vault/module%20specs/cluster-F-iteration.md) § Gradient Preservation). **Random sampling is forbidden** — random sampling is non-reproducible and structurally violates F10 (gradient preservation).

**Why Levenshtein over n-gram similarity:**

n-gram similarity over tokenised sequences has poor behaviour at pattern boundaries (prefix/suffix effects inflate similarity for patterns that share a long prefix but differ at the end). Levenshtein:

1. Captures insertions, deletions, and substitutions uniformly.
2. Degrades gracefully as patterns lengthen.
3. Normalised Levenshtein (edit_distance / max(len(A), len(B))) gives a `[0.0, 1.0]` similarity score directly interpretable as "fraction of steps that differ."

For maximum pattern length 8 and typical cluster widths 6-20, the DP matrix is at most 20×20 — cheap. The DP is computed over `StepToken` slices (u32 comparisons), not strings; fast in practice.

**Why top-K-by-distance over top-K-by-similarity:**

The two are mirror images, but the distance phrasing makes the boundary semantics explicit: "the N closest patterns under the band cap." This is also the framing used in the cluster spec and survives Zen audit lock.

---

## 3. Inputs / Outputs / Substrates

**Reads from** (per [MODULE_MATRIX m21 row](../../MODULE_MATRIX.md)): `m20` — the full `Vec<Pattern>` mined by `prefixspan_miner`. m21 needs the full candidate set to compute pairwise edit distances; it cannot stream.

**Writes to:** `m22 input` (per MODULE_MATRIX) — but in practice m21 emits `Vec<PatternVariant>` consumed in-memory by m23 (proposer). m22 is parallel and orthogonal (consumes workflow-run features, not patterns).

**Substrates:** None. m21 is a pure transformation function — no SQLite writes, no atuin reads, no stcortex/POVM/injection.db interaction. The proposer (m23) is the single Cluster F substrate write surface.

**CC-3 gate:** Not directly enforced at m21. m21 inherits m20's CC-3 stabilization gate — if m20 returned an empty `Vec<Pattern>` due to gate, m21 has nothing to do and returns an empty `Vec<PatternVariant>`.

**CC-4 (F → G) contract:** m21's `PatternVariant` outputs are one of two inputs to `m23.WorkflowProposal` (the other being m20's canonical). Variants ride alongside the canonical to m30 review queue.

**Verb class:** `active` — single-phase override per [GENESIS v1.3 § 3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) table row m21 permits `propose` / `build variants`, bounded by `K` (default 3) and the constraint that m20 output is the only input.

---

## 4. Public API (planning-spec; markdown only)

```rust
// planning-spec only — m21_variant_builder public surface
// rationale: deterministic top-K-by-ascending-distance; never random sampling

use thiserror::Error;
use serde::{Deserialize, Serialize};
use crate::m20_prefixspan_miner::{Pattern, StepToken};

/// A near-miss variant of a canonical pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PatternVariant {
    /// Identifier of the canonical pattern this variant is near.
    pub canonical_id: u64,
    /// The variant's own step sequence.
    pub variant_steps: Vec<StepToken>,
    /// Normalised Levenshtein edit distance from canonical. In [0.0, 1.0].
    /// Always in the near-miss band [0.25, 0.60) for emitted variants.
    pub edit_distance: f64,
    /// Rank within the top-K (0 = nearest variant).
    pub top_k_rank: usize,
    /// Variant's own support count (inherited from m20).
    pub support: usize,
}

/// Newtype for top-K cap; prevents accidental u64 mixing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopK(pub usize);

#[derive(Debug, Error)]
pub enum VariantError {
    #[error("invalid similarity band: lo={lo}, hi={hi} (must satisfy 0.0 <= lo < hi <= 1.0)")]
    InvalidBand { lo: f64, hi: f64 },

    #[error("top-K must be >= 1")]
    TopKZero,

    #[error("canonical pattern empty: cannot compute edit distances")]
    EmptyCanonical,
}

/// Build variant set for a single canonical pattern.
///
/// # Determinism
///
/// Output is sorted by `edit_distance` ASC, then `canonical_id` ASC, then
/// `variant_steps` lexicographic. Identical inputs → identical outputs.
///
/// # Errors
///
/// `EmptyCanonical` if canonical's steps are empty. `InvalidBand` if band bounds
/// violate `0.0 <= lo < hi <= 1.0`. `TopKZero` if `k.0 == 0`.
pub fn build_variants(
    canonical: &Pattern,
    candidates: &[Pattern],
    k: TopK,
    band_lo: f64,
    band_hi: f64,
) -> Result<Vec<PatternVariant>, VariantError> { /* see § 5 */ }

/// Normalised Levenshtein edit distance on `StepToken` sequences.
/// Returns `[0.0, 1.0]` where 0.0 = identical, 1.0 = fully different.
pub fn normalized_levenshtein(a: &[StepToken], b: &[StepToken]) -> f64 { /* DP, see § 5 */ }
```

---

## 5. Algorithm Sketch (planning-spec only)

```rust
// planning-spec only — m21_variant_builder::levenshtein
// rationale: normalised to [0.0, 1.0]; StepToken equality is structural (Eq derived)
//            not byte-level. DP table is small (max 20×20 for typical inputs).

pub fn normalized_levenshtein(a: &[StepToken], b: &[StepToken]) -> f64 {
    let n = a.len();
    let m = b.len();
    if n == 0 && m == 0 { return 0.0; }
    let denom = n.max(m) as f64;
    let mut dp: Vec<Vec<usize>> = vec![vec![0; m + 1]; n + 1];
    for i in 0..=n { dp[i][0] = i; }
    for j in 0..=m { dp[0][j] = j; }
    for i in 1..=n {
        for j in 1..=m {
            let cost = if a[i-1] == b[j-1] { 0 } else { 1 };
            dp[i][j] = (dp[i-1][j] + 1)
                .min(dp[i][j-1] + 1)
                .min(dp[i-1][j-1] + cost);
        }
    }
    dp[n][m] as f64 / denom
}

// planning-spec only — m21_variant_builder::top_k
// rationale: deterministic ascending-distance selection; band-filter first to
//            avoid wasted sort work; stable sort with explicit tie-break

pub fn build_variants(
    canonical: &Pattern,
    candidates: &[Pattern],
    k: TopK,
    band_lo: f64,
    band_hi: f64,
) -> Result<Vec<PatternVariant>, VariantError> {
    if canonical.steps.is_empty() { return Err(VariantError::EmptyCanonical); }
    if !(0.0..1.0).contains(&band_lo) || !(band_lo..=1.0).contains(&band_hi) {
        return Err(VariantError::InvalidBand { lo: band_lo, hi: band_hi });
    }
    if k.0 == 0 { return Err(VariantError::TopKZero); }

    let mut scored: Vec<(f64, &Pattern)> = candidates.iter()
        .filter(|c| c.canonical_hash != canonical.canonical_hash) // exclude self
        .map(|c| (normalized_levenshtein(&canonical.steps, &c.steps), c))
        .filter(|(d, _)| (band_lo..band_hi).contains(d))           // near-miss band
        .collect();

    scored.sort_by(|(da, pa), (db, pb)| {
        da.partial_cmp(db).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| pa.canonical_hash.cmp(&pb.canonical_hash))
            .then_with(|| pa.steps.cmp(&pb.steps))
    });

    Ok(scored.into_iter().take(k.0).enumerate().map(|(rank, (d, p))| PatternVariant {
        canonical_id: canonical.canonical_hash,
        variant_steps: p.steps.clone(),
        edit_distance: d,
        top_k_rank: rank,
        support: p.support,
    }).collect())
}
```

**Tie-breaking rule:** when two variants have identical `edit_distance`, ties are broken by `canonical_hash` ASC, then by `variant_steps` lexicographic. This produces a total order — identical inputs always yield identical outputs, and the ordering survives serde roundtrip.

---

## 6. Boilerplate Lift / New Authorship

Per [GOD_TIER_CONSOLIDATION Part IV Category 04](../../../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md):

| Component | Source | Reuse | New |
|---|---|---|---|
| Sequence iteration scaffold | `m49_task_graph.rs` | 30% | adapted to `StepToken` slices |
| Levenshtein DP table | reference implementations | 0% | **~80 LOC fresh** (StepToken-specific equality) |
| Top-K-by-distance with tie-break | none | 0% | ~50 LOC fresh |
| Band-filter (`[lo, hi)`) | none | 0% | ~20 LOC fresh |
| Public surface + error taxonomy | conventional | 30% | ~30 LOC fresh |

**Total fresh authorship for m21:** ~150 LOC of the cluster's ~600-700 LOC Gap 1 budget. **Lifted:** ~50 LOC scaffolding.

The Levenshtein crate ecosystem (e.g., `strsim`, `edit-distance`) operates on `&str`. m21 cannot lift those because (a) `StepToken` equality is structural not byte-level, and (b) the normalisation divisor must be `max(len(A), len(B))` measured in tokens, not bytes. Authoring the DP from scratch is correct.

---

## 7. F2 Enforcement and Reproducibility

m21 does **not** enforce F2 (n >= 20) — that gate is at m23's `ProposalBuilder::build()`. m21 propagates `Pattern.support` into `PatternVariant.support` so m23 can read both canonical's and variant's support counts when constructing the proposal.

**Reproducibility is the m21 contract**, not sample-size enforcement. The test suite must include a **reproducibility test**: `build_variants(canonical, candidates, k, lo, hi)` called twice on identical inputs (including shuffled `candidates` order) must produce identical `Vec<PatternVariant>` output. This is the F10 mitigation: random variant sampling produces non-reproducible output and is structurally forbidden.

The tie-breaking rule (§ 5) is the mechanism: edit-distance ascending → canonical_hash ascending → steps lexicographic provides a total order. No `unwrap()` paths; no `HashMap` iteration (which would be non-deterministic).

---

## 8. Tests (target 65)

Per [MODULE_MATRIX m21 row](../../MODULE_MATRIX.md): `test_count_min: 65`, kind `property`.

Per [V7 cluster-F plan § m21 test budget](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md):

| Family | Count | Notes |
|---|---:|---|
| F-Unit | 35 | per-arm coverage of `build_variants`, `normalized_levenshtein`, tie-break, band-filter |
| F-Property | 10 | invariants (see below) |
| F-Fuzz | 0 | not in m21's budget (m20 owns fuzz lane) |
| F-Integration | 15 | m20 → m21 → m23 chain; varied K and band |
| F-Contract | 5 | `PatternVariant` serde roundtrip; `m23.ProposalBuilder` consumer contract |
| F-Regression | 4 | pre-seeded bug classes (see below) |
| F-Mutation | ≥75% kill | per G6 |
| **Total** | **≥65** | (70 allocated in V7 plan) |

**Property invariants (proptest):**

1. **Symmetry:** `normalized_levenshtein(a, b) == normalized_levenshtein(b, a)` always
2. **Identity:** `normalized_levenshtein(a, a) == 0.0` always
3. **Boundedness:** `normalized_levenshtein(a, b) ∈ [0.0, 1.0]` always
4. **Disjoint upper bound:** when `a` and `b` share no tokens and `len(a) == len(b)`, distance = 1.0
5. **Top-K cardinality:** `build_variants(_, _, k, _, _).len() <= k.0` always
6. **Band membership:** every emitted variant's `edit_distance ∈ [band_lo, band_hi)`
7. **Ordering:** `result.is_sorted_by(|a, b| a.edit_distance.partial_cmp(&b.edit_distance).unwrap())` always
8. **Reproducibility:** `build_variants(c, candidates, k, lo, hi) == build_variants(c, shuffled(candidates), k, lo, hi)`
9. **Self-exclusion:** canonical is never emitted as its own variant
10. **Monotonic K:** `build_variants(c, cands, K1, lo, hi)` is a prefix of `build_variants(c, cands, K2, lo, hi)` when K1 <= K2

**Pre-seeded regression bug classes:**

- Mutation of `min` to `max` in DP recurrence (must die — distance would be wrong direction)
- Off-by-one in `max(len(A), len(B))` divisor (must die — would produce >1.0 distances)
- Tie-break ordering swap (must die — would break reproducibility)
- Band-filter inclusive vs exclusive on upper bound (locked: `[lo, hi)`)

---

## 9. Antipatterns Avoided

| Antipattern | Mitigation in m21 |
|---|---|
| **F10** (random variant sampling) | Top-K-by-ascending-distance is deterministic; reproducibility property test enforces. |
| **AP-V7-03** (verb collapse) | m21 builds variants; never dispatches, promotes, or names with human-meaningful labels. |
| **AP-V7-07** (auto-promotion m23→m30) | m21 does not write to m30; output is in-memory `Vec<PatternVariant>` consumed by m23 only. |
| **AP-Test-01** (coverage theatre) | Mutation ≥75% on Levenshtein DP numerics; reproducibility property test. |
| **F11** (cascade monoculture) | m21 operates on opaque `StepToken(u32)`; never sees human-readable cascade names. |
| **AP30** (namespace string drift) | m21 has no string surface; consumes `Pattern` (typed) only. |
| **Class G** (substrate-frame confusion) | m21 builds **topologically-adjacent** variants — never "user-preferred" alternatives. |

---

## 10. Substrate / Watcher Class Pre-Position

- **Class G** (substrate-frame confusion) if variant generation drifts toward "what would the user prefer" rather than "what is topologically adjacent to the canonical in StepToken space." Watcher pre-positioned to flag any spec or implementation language drift at G7 audit.
- **Class A** (activation transition) at first `build_variants` invocation post-G9.
- **Class C** (refusal) at `InvalidBand` / `EmptyCanonical` / `TopKZero` — refusal IS correct behaviour.

**Atuin trajectory anchor:** `wt-variants-emit <canonical_id>` (proposed; T5.2) captures per-canonical `(variant_count, mean_edit_distance, min_edit_distance, max_edit_distance, build_duration_us)`. Trend of mean edit distance over time is a signal of solution-space diversity — narrowing distance indicates m20 patterns are converging (substrate is finding stable optima); widening indicates pattern-space is still expanding.

---

## 11. Open Questions (Zen G7 audit + G5 spec interview)

1. **Variant near-miss band [0.25, 0.60).** The 0.25 lower bound separates "same" from "near-miss"; 0.60 separates "near-miss" from "unrelated." Are these well-calibrated for workflow-trace's step vocabulary size (~40-80 step types)? G5 interview required to test against empirical data once first 30 days of m4/m5 observations exist.
2. **K default = 3.** From P0 #10 (gradient preservation requirement). Should K be configurable per cluster-type, or fixed engine-wide? Recommend fixed (deterministic substrate behaviour) but allow operator override via CLI `--top-k`.
3. **Tie-break ordering choice.** Currently distance → canonical_hash → steps lexicographic. Alternative: distance → support DESC → canonical_hash. Either gives total ordering; the proposed ordering favours stable hash identity over support count. Zen G7 to confirm.
4. **Pure-Rust Levenshtein library lift?** The `strsim` crate exists but operates on chars. Vendoring or authoring? **Recommend authoring** — keeps `StepToken` semantics explicit and avoids `strsim`'s string-conversion overhead.
5. **DP table allocation strategy.** `vec![vec![0; m+1]; n+1]` allocates per call. Should we use a thread-local scratch buffer for repeated calls? For ~20×20 tables this is premature; flag for post-soak optimisation if profiling shows allocator pressure.
6. **Band-filter inclusivity.** Locked at `[lo, hi)` — closed lower bound (0.25 admitted), open upper bound (0.60 excluded). Zen G7 to confirm convention is consistent with cluster-F spec text.

---

## 12. Synergy / Sister-Module Anchors

- **m20** ([m20_prefixspan_miner.md](m20_prefixspan_miner.md)) — primary input source; m21 consumes `Vec<Pattern>` from m20's `mine_patterns` (Pure Rust per D-A, Luke S1002127).
- **m22** ([m22_kmeans_feature.md](m22_kmeans_feature.md)) — **parallel, NOT consumed.** m21 does not consume m22 (variants are topological neighbours; feature clusters are orthogonal context). Crossing this boundary would create premature coupling.
- **m23** ([m23_workflow_proposer.md](m23_workflow_proposer.md)) — direct consumer; receives `Vec<PatternVariant>` and composes with m20 canonical + m22 feature context + m14 lift into `WorkflowProposal`.
- **m14** (Cluster E) — indirect: m21 inherits m20's CC-3 gate state; if m20 returned empty due to gate, m21 emits empty.
- **m23 → m30** (Cluster G) — **m23 NEVER auto-promotes to m30** (AP-V7-07). Variants traverse three module boundaries before any human review: m21 → m23 → m30 admission queue (human-reviewed).

---

## 13. Verification Trail

- **Frontmatter complete:** ✓ cluster, layer, binary, feature_gate, gap_owner, test_kinds, verb_class, test_count_min
- **Cluster ownership:** Cluster F · L6 · KEYSTONE
- **Gap ownership:** Gap 1 (variant-builder slice) — ~150 LOC fresh of cluster ~600-700 LOC budget
- **Cross-cluster contracts:** CC-3 (inherited from m20); CC-4 (downstream via m23)
- **F2 gate referenced:** § 7 (m21 does not enforce; m23 ProposalBuilder enforces n_samples >= 20)
- **AP-V7-07 (no auto-promote):** § 9, § 12
- **F10 (no random sampling):** § 2, § 7, § 8 (reproducibility property test)
- **Bidi anchors:** Sister modules (4) · Vault (1) · V7 plan (1) · Cluster spec (1) · Matrix row (1) · GENESIS v1.3 (1) · PATTERNS / GOLD_STANDARDS / ANTIPATTERNS (3) — ✓
- **Word count:** ~2,000 (within KEYSTONE 1,500-2,500 range)
- **No `.rs` files authored:** ✓ planning-only, HOLD-v2 respected
- **Rust fenced blocks are spec documentation:** ✓ labelled "planning-spec only" inline

*m21 spec authored 2026-05-17 (S1001982). Cluster F KEYSTONE — variant-builder slice of Gap 1. Planning-only per HOLD-v2 + AP24. Pre-G7 Zen audit.*
