---
title: MODULE PLAN — Cluster F (Iteration · KEYSTONE) · m20 PrefixSpan / m21 variant builder / m22 K-means / m23 proposer
date: 2026-05-17
kind: planning-only · per-module spec · no code authorised (HOLD-v2 + AP24)
cluster: F
layer: L6
modules: [m20, m21, m22, m23]
loc_budget: ~850
test_budget: 280
mutation_kill_targets: { m20: 80%, m21: 75%, m22: 70%, m23: 75% }
structural_gap: "Gap 1 — N-step compositional sub-graph detection (~600-700 LOC fresh)"
authority: Command · workflow-trace V7 optimisation
status: KEYSTONE — the engine's structural gap
---

# Cluster F — Iteration (L6) · KEYSTONE

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · [[../GENERATIONS/G3-bidi-flow.md]] · sibling [[cluster-E.md]] · [[cluster-G.md]] · [[CROSS_CLUSTER_SYNERGIES.md]]
>
> **Function:** Cluster F is the engine's **structural keystone** — the four-module composition (PrefixSpan miner + Levenshtein-based variant builder + K-means feature clusterer + gradient-preservation proposer) that turns evidence-gated cascades into **typed `WorkflowProposal` artefacts** ready for human accept and bank admission. Per [[../KEYWORDS_20.md]] § KEYSTONE: ~850 LOC total, ~600-700 LOC fresh authorship (Gap 1 owner per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part III). Without Cluster F there is no "engine" — only an observatory.

---

## Overview

Cluster F sits at L6, downstream of Cluster B (cascade + battern observations) + Cluster E (evidence gate). It owns four modules in a tight DAG:

```
m4 ─┐                                              m22 (K-means feature)
m5 ─┤                                                   │
m14 ┤── m20 (PrefixSpan) ── m21 (variant builder) ── m23 (proposer) ── human accept ── m30 (bank)
m6 ─┤
m7 ─┘
```

m20 is the canonical-pattern miner; m21 generates near-miss variants around each canonical (Levenshtein top-K-by-edit-distance); m22 clusters in feature-space (cost-variance + step-diversity + fitness-dimension) to provide *context* for proposer decisions; m23 composes the four signals (canonical patterns + near-miss variants + feature clusters + Wilson-bounded lift) into typed `WorkflowProposal` artefacts gated at construction by F2.

**Why KEYSTONE:** the cluster owns Gap 1 (Part III consolidation) — *N-step compositional sub-graph detection*, which is the engine's reason-for-existing. PrefixSpan is chosen over Apriori (candidate-explosion at N>2 in 80-symbol step alphabet) and over n-gram sliding window (no gap-allowed matching). Levenshtein top-K-by-ascending-edit-distance is chosen over random variant sampling (random sampling rejected as non-reproducible — see [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part III Gap 1).

**Test density:** 280 tests across 4 modules (avg 70/module; m20 = 90 KEYSTONE allowance). Mutation kill ≥80% on m20 (G6 cluster F threshold per [[../GENERATIONS/G6-test-discipline.md]]) — algorithm correctness paramount.

---

## m20 — PrefixSpan sequential pattern miner (L6 · `src/m20_prefixspan/`)

### Purpose

Mine **frequent sequential patterns** with gap-allowed matching from observed cascade + battern sequences. Emits typed `Pattern { steps: Vec<StepToken>, support: usize, gap_bounds: (usize, usize) }` for each frequent pattern at min_support ≥2 and bounded right-gap MAX_GAP_STEPS=5. The output is the *canonical-pattern catalogue* that m21 (variants) and m23 (proposer) consume.

PrefixSpan is the algorithm because:
1. Projection-based; avoids Apriori's candidate-explosion O(|Σ|^L) at length L
2. Gap-allowed matching is intrinsic (Pattern [A, B, C] matches sequence [A, X, B, Y, C] with gap_bounds (1, 1))
3. Frequency support is exact (no probabilistic approximation)
4. Reference implementations exist in academic literature (Pei et al. 2001) for differential testing

### Edge contract (from [[../GENERATIONS/G3-bidi-flow.md]] § Cluster F m20)

- **Upstream-IN:** `m4.CascadeCluster` step-lists, `m5.BatternStepRow` ordered, `m14.Lift` evidence-gate (n≥20 enforced)
- **Downstream-OUT:** `Vec<Pattern { steps, support, gap_bounds }>` → m21 (variants) + m23 (proposer)
- **Aspect-IN:** m8 build_prereq, m9 namespace_guard (validates `cluster_id` namespacing before consumption)
- **Failure-mode mitigated:** F2 (sample-size n≥20 enforced at output; patterns with support<min_support never emitted)

### src/ path (planning-spec only)

```
src/m20_prefixspan/
├── mod.rs                # public: pub struct Pattern, pub fn mine_patterns
├── algorithm/
│   ├── mod.rs
│   ├── projection.rs     # prefix-projection core (the PrefixSpan kernel)
│   ├── gap_bounds.rs     # MAX_GAP_STEPS=5 bounded-right-gap enforcement
│   ├── support.rs        # support counting + min_support pruning
│   └── frequent_items.rs # initial L1 frequent item discovery
├── types/
│   ├── mod.rs
│   ├── step_token.rs     # StepToken enum (canonical symbolic representation of a step)
│   ├── pattern.rs        # Pattern struct + Ord/Hash for dedup
│   └── sequence.rs       # Sequence type alias + iterator helpers
├── differential.rs       # naive O(n³) reference implementation (test-only; behind #[cfg(test)])
└── tests/                # 90 tests (see allocation)
```

### LOC budget

~350 LOC (per ULTRAMAP View 2 — highest single-module budget in the engine). Composition:
- algorithm/ ~200 LOC (PrefixSpan kernel + gap_bounds + support)
- types/ ~80 LOC
- differential.rs ~50 LOC (test-only naive reference for F-differential)
- mod.rs + glue ~20 LOC

### Test budget (90 tests; KEYSTONE allowance per [[../STANDARDS/TEST_DISCIPLINE.md]] § m20)

| Family | Count | Notes |
|---|---:|---|
| F-Unit | 45 | per-arm coverage; 3 per public fn × 15 fns; per-StepToken-variant test |
| F-Property | 10 | invariants: gap_bounds monotonic in MAX_GAP_STEPS · idempotent re-mine · support_count = exact match count · output sorted-by-support |
| F-Fuzz | 1 target (24h budget) | `m20_prefixspan` fuzz target per [[../GENERATIONS/G6-test-discipline.md]] § Fuzz enumeration — input arbitrary `Vec<Vec<StepToken>>`, must never panic |
| F-Integration | 20 | m4/m5/m14 upstream wiring; m23 downstream handshake; CC-3 closure exercise |
| F-Contract | 5 | Pattern schema parity (serde roundtrip stable across versions); ProposalBuilder consumer contract |
| F-Regression | 8 | reserved (5 bug-classes pre-seeded from spec: zero-support emit, gap_bounds overflow, empty-sequence panic, duplicate-pattern emit, StepToken UTF-8 munge) |
| F-Mutation | 1 budget (≥**80%** kill rate per G6 KEYSTONE) | algorithm correctness paramount |

**Plus F-Differential (not in G6's 7-family list but added per [[../GENERATIONS/G6-test-discipline.md]] § Top-1% practice #3):** `tests/differential/prefixspan_vs_naive.rs` — randomised input compared against naive O(n³) reference; equality required modulo output ordering. Catches subtle off-by-one + edge cases.

### Mutation kill threshold

**80%** (G6 cluster F m20 = KEYSTONE threshold per [[../GENERATIONS/G6-test-discipline.md]] § Per-module mutation table). Rationale: PrefixSpan is the engine's reason-for-existing; mutations of projection logic, support counting, or gap_bounds enforcement must die. Wave-end `cargo mutants --regex 'm20_prefixspan::.*'` enforces. Surviving mutations require either (a) added test catching the mutation OR (b) `// IGNORE: cosmetic only` annotation with rationale.

### Boilerplate-lift source

Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part IV Category 04 (Pattern detection):
- **m49 task_graph (Kahn's topological sort)** ~50% reuse — graph-traversal scaffolding adapted to sequence-projection
- **m39 fitness tensor rolling smoothing** ~30% reuse — accumulator pattern for support counting

Fresh authorship: ~250-280 LOC (PrefixSpan kernel + gap_bounds + differential reference). Lifted: ~70-100 LOC scaffolding. Per Part III Gap 1: *core algorithm ~300-500 LOC is fresh*.

### Structural-gap LOC

**Gap 1 KEYSTONE owner** (per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part III). m20 owns ~250-280 LOC of the ~600-700 fresh-authorship Gap 1 budget. The rest distributes across m21 (variant builder, ~150 LOC) + m23 (proposer composition, ~100 LOC) + m22 (K-means context, ~100 LOC of the K-means is mostly reused but the Cluster F feature-space wiring is fresh).

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| F2 (sample-size) | min_support ≥2 floor; n<min_support never emitted as Pattern; downstream m23 honours per Wilson CI gate |
| AP-Drift-03 (scaffold without binary-wiring) | integration test exercising m20 → m23 call chain at Wave-end |
| AP-Test-01 (coverage theatre) | mutation ≥80% + differential test against naive reference |
| F11 (cascade monoculture leakage) | m20 consumes opaque `m4.CascadeCluster` IDs only; no pane-label semantic visible |

### Atuin trajectory anchor

Per [[../INTEGRATION/atuin-integration.md]] § wt-prefixspan-replay (proposed; T5.2): atuin script captures `(input_sequence_hash, output_pattern_count, max_gap_observed, mining_duration_ms)` for every m20 invocation. Provenance for Phase 5C synthesis: trend of `output_pattern_count / input_sequence_count` is the engine's *pattern-discovery rate*.

### Watcher class pre-position

- **Class A** (activation transition) at first PrefixSpan invocation post-G9 — the moment the KEYSTONE goes live is the highest-leverage observation moment per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part VII.
- **Class G** (substrate-frame confusion) if m20 is treated as "search for what the user wants" rather than "mine frequent sub-graphs of observed substrate sequences". Watcher pre-positioned to flag any spec language drifting toward anthropocentric framing.
- **Class I** (Hebbian silence) if m20 emits ≥5 patterns per week but stcortex `learning_health` doesn't trend up — CC-5 broken downstream.

### KEYSTONE algorithm pseudocode (planning-spec only)

The following Rust fenced block is **planning-spec only** — NOT source code. Documents the intended algorithm shape for Zen G7 audit and Command-2 build-executor reference at Wave 3.

```rust
// planning-spec only — m20_prefixspan::algorithm::projection
// Reference: Pei et al. 2001 "PrefixSpan: Mining Sequential Patterns Efficiently"
// Adapted for: bounded-right-gap matching (MAX_GAP_STEPS=5 default) + opaque StepToken alphabet

pub fn mine_patterns(
    sequences: Vec<Sequence>,
    min_support: MinSupport,
    max_gap: MaxGap,
) -> Vec<Pattern> {
    // L1: discover frequent items (single-step patterns with support ≥ min_support)
    let frequent_items: Vec<StepToken> = sequences
        .iter()
        .flat_map(|s| s.iter().cloned())
        .fold(HashMap::<StepToken, usize>::new(), |mut acc, t| { *acc.entry(t).or_insert(0) += 1; acc })
        .into_iter()
        .filter(|(_, c)| *c >= min_support.0)
        .map(|(t, _)| t)
        .collect();

    let mut output = Vec::new();
    for item in frequent_items {
        // For each frequent L1 item, build projected database and recurse
        let prefix = Pattern { steps: vec![item.clone()], support: 0, gap_bounds: (0, 0) };
        let projected = project(&sequences, &prefix, max_gap);
        if projected.len() >= min_support.0 {
            let mut prefix = prefix;
            prefix.support = projected.len();
            output.push(prefix.clone());
            extend(&projected, prefix, &mut output, min_support, max_gap);
        }
    }
    // Stable sort by support DESC then pattern length DESC for deterministic output
    output.sort_by(|a, b| b.support.cmp(&a.support).then(b.steps.len().cmp(&a.steps.len())));
    output
}

fn project(sequences: &[Sequence], prefix: &Pattern, max_gap: MaxGap) -> Vec<Sequence> {
    // For each sequence containing prefix (with bounded-right-gap), emit suffix following last prefix match
    sequences.iter().filter_map(|s| {
        let mut cursor = 0usize;
        let mut last_match = None;
        for (pi, p_step) in prefix.steps.iter().enumerate() {
            // Find p_step in s[cursor..], honouring max_gap on subsequent steps
            if pi == 0 {
                // Unbounded left-gap on first step
                cursor = s.iter().position(|t| t == p_step)?;
            } else {
                // Bounded right-gap on subsequent steps
                let upper = (cursor + max_gap.0 + 1).min(s.len());
                let offset = s[cursor..upper].iter().position(|t| t == p_step)?;
                cursor += offset;
            }
            last_match = Some(cursor);
            cursor += 1;
        }
        let suffix = s[last_match.unwrap() + 1..].to_vec();
        Some(suffix)
    }).collect()
}

fn extend(
    projected: &[Sequence],
    prefix: Pattern,
    output: &mut Vec<Pattern>,
    min_support: MinSupport,
    max_gap: MaxGap,
) {
    // Recurse: find frequent items in projected DB, extend prefix
    // (full recursion body omitted in planning-spec; ~50 LOC implementation)
}
```

```rust
// planning-spec only — m20_prefixspan::types::pattern
// rationale: deterministic ordering for downstream m21 + m23 consumption

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Pattern {
    pub steps: Vec<StepToken>,
    pub support: usize,
    pub gap_bounds: (usize, usize),
}
```

---

## m21 — variant builder (L6 · `src/m21_variant_builder/`)

### Purpose

For each canonical `Pattern` emitted by m20, build a deterministic **top-K** set of near-miss variants ranked by **ascending normalized Levenshtein edit distance**. Variants are the engine's *exploration surface* — they preserve solution-space topology around canonical patterns, allowing the proposer to suggest *gradient-shaped* alternatives rather than random walks. Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part I § Cluster F: *Same-pattern threshold: normalized Levenshtein < 0.25; near-miss band 0.25-0.60. Top-K-by-ascending-edit-distance (N=3 default). Random sampling rejected as non-reproducible.*

### Edge contract

- **Upstream-IN:** `m20.Pattern` (canonical) + m20 near-miss `Pattern`s (those at 0.25 ≤ Levenshtein ≤ 0.60 vs canonical)
- **Downstream-OUT:** `Vec<PatternVariant { canonical_id, variant_steps, edit_distance: f64, top_k_rank: usize }>` → m23 (proposer)
- **Aspect-IN:** m8, m9
- **Failure-mode mitigated:** F10 (variant selection deterministic top-K-by-edit-distance — not random; preserves topology); F2 (variants inherit n≥20 enforcement from m20's canonical patterns)

### src/ path (planning-spec only)

```
src/m21_variant_builder/
├── mod.rs                # public: pub struct PatternVariant, pub fn build_variants
├── levenshtein.rs        # normalized Levenshtein on StepToken sequences
├── top_k.rs              # deterministic ascending-distance top-K selection
├── band_filter.rs        # near-miss band [0.25, 0.60] filter
└── tests/
```

### LOC budget

~200 LOC (per ULTRAMAP View 2). Levenshtein ~80 LOC (dense numerics with memoisation); top_k + band_filter ~80 LOC; mod.rs + types ~40 LOC.

### Test budget (70 tests; per [[../STANDARDS/TEST_DISCIPLINE.md]] § m21)

| Family | Count |
|---|---:|
| F-Unit | 35 |
| F-Property | 10 |
| F-Fuzz | 0 |
| F-Integration | 15 |
| F-Contract | 5 |
| F-Regression | 4 |
| F-Mutation | 1 budget (≥**75%** per G6) |

### Mutation kill threshold

**75%** (G6 m21). Levenshtein numerics critical (mutations of memoisation table or normalization divisor must die); top-K determinism critical (mutations of sort ordering must die).

### Boilerplate-lift source

Per Part IV Category 04: m49 task_graph adaptation (~30% reuse for sequence iteration scaffolding). Levenshtein algorithm has many reference implementations in `crates.io` but Cluster F authors a workflow-specific version to honour StepToken equality semantics (canonical token equality is not byte equality).

Fresh authorship: ~150 LOC. Lifted: ~50 LOC.

### Structural-gap LOC

Part of Gap 1 KEYSTONE allocation (~150 LOC). The Levenshtein-on-StepToken-sequences is fresh; reference Levenshtein implementations operate on character sequences.

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| F10 (random variant sampling) | top-K-by-ascending-edit-distance is deterministic; reproducibility test asserts identical output across runs given identical input |
| AP-Test-01 | mutation ≥75% on Levenshtein numerics |

### Atuin trajectory anchor

`wt-variants-emit <canonical_id>` (proposed): captures per-canonical variant set + edit-distances for trajectory grain.

### Watcher class pre-position

- **Class G** (substrate-frame) if variant generation drifts toward "what would the user prefer" rather than "what is topologically adjacent to the canonical in StepToken space".

### Levenshtein normalized pseudocode (planning-spec only)

```rust
// planning-spec only — m21_variant_builder::levenshtein
// rationale: normalized to [0.0, 1.0] for threshold comparison; uses dynamic programming
// StepToken equality is structural (Eq + Hash derived), not byte-level

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
```

```rust
// planning-spec only — m21_variant_builder::top_k
// rationale: deterministic ascending-distance selection; ties broken by canonical-pattern-id then variant-steps lexicographic

pub fn select_top_k(canonical: &Pattern, candidates: Vec<Pattern>, k: usize) -> Vec<PatternVariant> {
    let mut scored: Vec<(f64, Pattern)> = candidates.into_iter()
        .map(|c| (normalized_levenshtein(&canonical.steps, &c.steps), c))
        .filter(|(d, _)| (0.25..=0.60).contains(d))  // band filter (near-miss band per spec)
        .collect();
    // Stable sort: ascending edit distance, ties → canonical id, ties → variant steps lex
    scored.sort_by(|(da, pa), (db, pb)| {
        da.partial_cmp(db).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| pa.steps.cmp(&pb.steps))
    });
    scored.into_iter().take(k).enumerate().map(|(rank, (d, p))| PatternVariant {
        canonical_id: canonical.hash_id(),
        variant_steps: p.steps,
        edit_distance: d,
        top_k_rank: rank,
    }).collect()
}
```

---

## m22 — K-means feature clusterer (L6 · `src/m22_kmeans/`)

### Purpose

Cluster observed workflows in **feature space** (cluster size + step diversity + cost variance + fitness dimension) using K-means, producing `FeatureCluster` artefacts that give m23 (proposer) *context* about where proposed workflows fall in the feature distribution. K-means is chosen over PrefixSpan here because the feature space is *numeric and continuous* (per [[../GENERATIONS/G3-bidi-flow.md]] § m22) — PrefixSpan would be category-error.

### Edge contract (GAP-Bidi-01 closure per G3)

- **Upstream-IN:** `m4.cluster_id` (feature: cluster size, step diversity), `m6.ContextCostBand` (feature: cost variance), `m7.fitness_dimension` (feature)
- **Downstream-OUT:** `Vec<FeatureCluster { centroid: Vec<f64>, members: Vec<WorkflowId> }>` → m23 (proposer feature-context)
- **Aspect-IN:** m8, m9
- **Failure-mode mitigated:** F2 (per-cluster n≥20 enforcement before K-means runs)

### src/ path (planning-spec only)

```
src/m22_kmeans/
├── mod.rs                # public: pub struct FeatureCluster, pub fn cluster_features
├── kmeans.rs             # k-means++ initialization + Lloyd's iteration
├── feature_extract.rs    # workflow → Vec<f64> feature vector
└── tests/
```

### LOC budget

~150 LOC. K-means itself is well-studied; most LOC is feature-extraction wiring + per-cluster n≥20 enforcement.

### Test budget (60 tests)

| Family | Count |
|---|---:|
| F-Unit | 30 |
| F-Property | 8 (convergence invariants; k=1 degenerate; identical-points handling) |
| F-Fuzz | 0 |
| F-Integration | 15 |
| F-Contract | 3 |
| F-Regression | 3 |
| F-Mutation | 1 budget (≥**70%** per G6) |

### Mutation kill threshold

**70%** (G6 standard floor; K-means is well-studied so mutations of Lloyd's iteration produce obvious test failures via convergence tests).

### Boilerplate-lift source

K-means is library-thin in Rust ecosystem (e.g., `linfa-clustering`); but vendoring a workflow-specific version avoids dep weight + honours StepToken-derived feature semantics. ~60% from m39 fitness tensor accumulator pattern.

Fresh authorship: ~80 LOC (feature_extract specific to workflow-trace inputs). Lifted: ~70 LOC (k-means kernel adapted from m39 + reference).

### Structural-gap LOC

Small Gap 1 contribution (~50 LOC) — feature-space adaptation for workflow-trace is fresh; K-means core is not.

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| F2 | per-cluster n≥20 floor; K-means refuses to emit clusters below threshold |
| AP-V7-09 (substrate-frame) | features are numeric (operationalised) not user-intent surrogates |

### Atuin trajectory anchor

`wt-kmeans-snapshot` (proposed): captures `(k, iterations_to_convergence, sse)` per invocation.

### Watcher class pre-position

- **Class C** (refusal) if per-cluster n<20 — K-means refuses to emit; Watcher logs as correct behaviour.

---

## m23 — gradient-preservation proposer (L6 · `src/m23_proposer/`)

### Purpose

Compose m20's canonical patterns + m21's variants + m22's feature context + m14's Wilson-bounded lift into typed `WorkflowProposal` artefacts. The **construction-time gate** is F2 — `ProposalBuilder::build()` returns `Err(SampleSizeBelowF2)` if `Option<Lift>::None` from m14 OR if canonical pattern support is below threshold. Deviation-shaped variants get relaxed n≥5 explicit flag (per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part I § Cluster F).

### Edge contract

- **Upstream-IN:** `m20.Pattern` + `m21.PatternVariant` + `m22.FeatureCluster` + `m14.Lift`
- **Downstream-OUT:** `Vec<WorkflowProposal { steps, evidence, deviation_relaxed_n: bool }>` → human accept via `wf-crystallise propose accept <id>` → m30 (bank)
- **Aspect-IN:** m8, m9, m10 (Ember 7-trait CI gate on the proposal text)
- **Failure-mode mitigated:** F2 (n≥20 default; deviation-shaped variants relaxed n≥5 with explicit `deviation_relaxed_n: true` flag in serialized output)

### src/ path (planning-spec only)

```
src/m23_proposer/
├── mod.rs                # public: pub struct WorkflowProposal, pub struct ProposalBuilder
├── builder.rs            # ProposalBuilder::build() — F2 gate enforcement at construction
├── evidence.rs           # Evidence quad: { canonical_support, wilson_low, wilson_high, feature_centroid }
├── deviation.rs          # deviation-shaped variant handling (relaxed n≥5 explicit flag)
└── tests/
```

### LOC budget

~150 LOC. Composition logic + F2 gate enforcement; small surface but high stakes.

### Test budget (60 tests)

| Family | Count |
|---|---:|
| F-Unit | 30 |
| F-Property | 5 |
| F-Fuzz | 0 |
| F-Integration | 15 |
| F-Contract | 5 |
| F-Regression | 4 |
| F-Mutation | 1 budget (≥**75%** per G6) |

### Mutation kill threshold

**75%** (G6 m23). F2 enforcement at construction must be mutation-tight — any surviving mutation that lets n<20 through silently is a critical bug.

### Boilerplate-lift source

Per Part IV Category 09 (Trap/verify/escape): SKILL-pre-deploy-hardening 4-agent-gate pattern adapted (~80% reuse) for the proposal-construction gate. povm-v2 dedup (~60% reuse) for canonical-id collision handling.

Fresh authorship: ~100 LOC (ProposalBuilder + Evidence quad + deviation handling). Lifted: ~50 LOC.

### Structural-gap LOC

Part of Gap 1 KEYSTONE (~100 LOC). Composition of canonical + variant + feature + lift into single typed proposal is fresh authorship.

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| F2 | construction-time refusal; `Result<WorkflowProposal, ProposalError>` API forces caller to handle refusal |
| F5 (bank creep) | proposal NEVER auto-promotes; explicit `wf-crystallise propose accept <id>` required at m30 admission |
| AP-Drift-04 (test count over-report) | every proposal carries `evidence.n` field; auditable post-construction |

### Atuin trajectory anchor

`wt-proposals-list` (proposed; T5.2): every emitted proposal lands in atuin with `(canonical_id, variant_count, evidence_n, wilson_low, wilson_high, deviation_relaxed_n)`.

### Watcher class pre-position

- **Class C** (refusal) at every F2 rejection — refusal IS the correct behaviour per [[../KEYWORDS_20.md]] § F2 keyword.
- **Class A** (activation) at first ProposalBuilder::build() success post-G9 — the moment a typed proposal exists is the moment the engine has *output*.

### KEYSTONE ProposalBuilder pseudocode (planning-spec only)

```rust
// planning-spec only — m23_proposer::builder
// rationale: F2 gate enforced at CONSTRUCTION, not runtime; refusal returns typed Err
// per [[../KEYWORDS_20.md]] § F2 keyword: n≥20 + Wilson 95% CI; deviation-shaped variants relaxed n≥5

pub struct ProposalBuilder {
    canonical: Pattern,
    variants: Vec<PatternVariant>,
    feature_cluster: Option<FeatureCluster>,
    lift: Option<Lift>,
    deviation_relaxed: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ProposalError {
    #[error("F2 violation: lift evidence missing (n<20 from m14)")]
    LiftEvidenceMissing,
    #[error("F2 violation: canonical support {0} below floor")]
    SupportBelowFloor(usize),
    #[error("deviation-relaxed flag set but no variants provided")]
    DeviationFlagWithoutVariants,
}

impl ProposalBuilder {
    pub fn build(self) -> Result<WorkflowProposal, ProposalError> {
        let lift = self.lift.ok_or(ProposalError::LiftEvidenceMissing)?;
        // Wilson CI bounds already enforced in m14; here we honour the Option discipline
        let floor = if self.deviation_relaxed { 5 } else { 20 };
        if self.canonical.support < floor {
            return Err(ProposalError::SupportBelowFloor(self.canonical.support));
        }
        if self.deviation_relaxed && self.variants.is_empty() {
            return Err(ProposalError::DeviationFlagWithoutVariants);
        }
        Ok(WorkflowProposal {
            steps: self.canonical.steps,
            evidence: Evidence {
                canonical_support: self.canonical.support,
                wilson_low: lift.wilson_low,
                wilson_high: lift.wilson_high,
                feature_centroid: self.feature_cluster.map(|fc| fc.centroid),
            },
            deviation_relaxed_n: self.deviation_relaxed,
        })
    }
}
```

---

## Cluster-level synergies

### CC-3 — Evidence-Driven Iteration (E → F)

Cluster F is the consumer half of CC-3. m14's `Option<Lift>` is the construction-time gate for m23; m20's min_support is the iteration-time gate. The two gates are independent and additive — neither alone is sufficient.

### CC-4 — Proposal → Bank → Dispatch (F → G → Conductor)

Cluster F is the producer half of CC-4. m23 emits `WorkflowProposal` artefacts; human `wf-crystallise propose accept <id>` is the boundary between Cluster F and Cluster G m30. No auto-promotion (F5 mitigation).

### CC-5 — Substrate Learning Loop (G → H → back to F)

Cluster F is the eventual *consumer* of CC-5's substrate-grain feedback. As Cluster G m32 dispatches and Cluster H m40-m42 propagate to substrate, stcortex pathway weights update; next session's Cluster F invocations see updated inputs (via m1/m2/m3 → m4-m7 → m14 → m20/m22). Loop is **intentionally slow** (days/weeks per [[../GENERATIONS/G3-bidi-flow.md]] § CC-5).

### Intra-cluster

m20 → m21 → m23: linear (variants feed proposer).
m20 → m23: direct (canonical feeds proposer).
m22 → m23: parallel (feature context feeds proposer alongside canonical + variants).
m14 → m23: parallel (lift feeds proposer alongside canonical).

m21 does NOT consume m22 (variants are topological neighbours of canonical patterns; feature clusters are orthogonal context). Crossing this boundary would create premature coupling.

---

## Cluster-level antipatterns (Cluster F specific)

| ID | Antipattern | Mitigation |
|---|---|---|
| AP-WT-F2 | sample-size inflation | construction-time refusal in m23 ProposalBuilder; m14 Option::None propagation |
| AP-WT-F10 | random variant sampling | m21 top-K-by-ascending-edit-distance deterministic; reproducibility test |
| AP-WT-F11 | cascade monoculture leakage | m20 consumes opaque cluster_id; never pane-label semantic |
| AP-Drift-03 | scaffold without binary-wiring | Wave 3 integration test exercises m20→m21→m23→m30 full chain |
| AP-V7-09 | substrate-frame confusion | spec language audit at G7 for anthropocentric framing in m20/m21/m22/m23 docs |
| AP-V7-05 | module-plan-to-src drift | per-Wave-end `wc -l src/m20_prefixspan/**.rs ≤ 2 × 350` check |

---

## Citation discipline

Every claim cites ULTRAMAP View 2 (LOC + tests), G3 § Cluster F (edge contracts), GOD_TIER_CONSOLIDATION Part I + Part III Gap 1 (algorithm choices + thresholds), G6 § Per-module mutation (kill targets), KEYWORDS_20 § KEYSTONE / F2 / PrefixSpan, ANTIPATTERNS_REGISTER (failure mappings). No uncited claims.

---

## Sign-off

Cluster F (KEYSTONE) plan authored 2026-05-17 by Command (parallel author for V7 optimisation). Planning-only per HOLD-v2 + AP24; pseudocode blocks are spec documentation NOT source. ~280 tests across 4 modules; mutation kill targets 80%/75%/70%/75% with m20 at G6 KEYSTONE threshold. Owns Gap 1 (~600-700 LOC fresh authorship of PrefixSpan + Levenshtein-on-StepToken + ProposalBuilder F2 gate). CC-3 consumer + CC-4 producer + CC-5 eventual consumer. Watcher Class A pre-positioned for first PrefixSpan invocation moment. Read with [[cluster-E.md]] (m14 upstream) + [[cluster-G.md]] (m30 downstream) + [[CROSS_CLUSTER_SYNERGIES.md]] (CC-3/4/5 deep contracts).

*Luke @ node 0.A | Command @ Orchestrator | Watcher ☤ @ observing | Zen @ audit-lane | 2026-05-17 (S1001982)*
