---
title: Cluster F — ITERATION Module Spec
date: 2026-05-17 (S1001982)
cluster: F
modules: m20, m21, m22, m23
status: planning-only · HOLD-v2 active · pre-G7 Zen audit
authority: Luke @ node 0.A (single-phase override 2026-05-17)
kind: module-spec
---

# Cluster F — ITERATION Module Spec

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Modules Synergy Clusters and Feature Verification S1001982]] · [[Genesis Prompt v1.2 S1001982]] · [[BOILERPLATE_INDEX]]
>
> Related: [[cluster-B-observation]] (inputs) · [[cluster-C-correlation]] (inputs) · [[cluster-E-evidence]] (inputs from m14) · [[cluster-G-bank-dispatch]] (output target via human review)

---

## Cluster Overview

Cluster F is the **engine keystone** of `workflow-trace`. It is the structural gap acknowledged in the Boilerplate Hunt — the N-step compositional sub-graph detection algorithm that no existing habitat source covers beyond 30% reuse. The four modules develop and iterate (active verbs are permitted in single-phase per Luke 2026-05-17).

The cluster's defining constraint: **PROPOSE, never act.** Every output of m20-m23 is a `Proposal` — a structured record carrying a canonical variant, N near-miss variants, confidence bands, sample counts, and CI bars. No proposal auto-promotes to m30 (the bank). Human review is required at every transition from m23's proposal surface to m30.

All four modules are read-only toward habitat data. None writes to atuin, stcortex, or injection.db (m13 owns those paths). The only write surface in Cluster F is the `proposals` table in the local `workflow_trace.db`.

---

## Cross-Cluster Contracts (Cluster F)

| Contract | Direction | Content |
|---|---|---|
| CC-3 | E → F | m14 `habitat_outcome_lift` metric informs which iterator focuses; iterators only propose where evidence supports n ≥ 20 |
| CC-4 | F → G | m23 proposals → human review → m30 bank (NEVER auto-promoted) |
| CC-5 | H → F (feedback loop) | Pathway weights from m40/m41/m42 reinforce/LTD outputs shift over time → m20-m22 inputs re-weighted |

### F2 Hard Gate (from spec, applies to all m20-m22 outputs)

Every `Proposal` emitted by m20, m21, and m22 MUST carry:
- `n_samples: usize` — number of observed executions supporting the pattern
- `ci_lower: f64`, `ci_upper: f64` — Wilson score interval bounds (see F2 Enforcement section)

A proposal with `n_samples < 20` is illegal. The check fires at construction time; the module returns `Err(ProposalError::InsufficientSamples { n, required: 20 })`.

---

## The KEYSTONE GAP — N-Step Compositional Sub-Graph Detection

### Why This Is New Authorship

The Boilerplate Hunt documents this gap explicitly at Cat 04 row for `m20_heat_source_hebbian.rs`:

> **30% reuse.** `CoActivationPair` provides pairwise (2-step) co-activation. The KEYSTONE: generalize from pairwise to N-step sub-graphs is fresh authorship (~600-1,000 LOC).

The existing pairwise infrastructure in POVM (v2 reinforcement signals, `CoActivationPair { a, b, ts_ms }`) covers exactly two elements at a time. Cascade detection in workflow-trace requires detecting that tool sequence `[A → B → C → D]` recurs across sessions, possibly with gaps (A, B, _, D is still a match if the gap is known). The transition from 2-element to N-element is not incremental — it requires a different data structure and a different algorithm.

### Algorithm Selection: PrefixSpan (chosen over Apriori and n-gram sliding window)

Three candidates were evaluated:

**Apriori** — generates all frequent itemsets by level-wise candidate expansion. Problem: it requires multiple database passes and the candidate explosion for sequences (versus sets) is quadratic in the number of distinct items. For workflow-trace, the "alphabet" is the set of tool-call step types (~40-80 distinct steps in practice). Apriori would generate O(80^4) candidates for 4-step patterns. Rejected.

**N-gram sliding window** (Command-2's lean, per [[Modules Synergy Clusters and Feature Verification S1001982]]) — treats tool-call sequences as strings and counts n-gram occurrences. Fast and simple. Problem: sliding windows cannot skip gaps. If a session runs `[read_file → bash → edit → bash → cargo_check]` and the pattern of interest is `[read_file → edit → cargo_check]` (with the bash calls elided), a sliding window misses it. Gap-allowed matching is a hard requirement for cascade cluster detection (cascades frequently interleave monitoring calls between the "meaningful" steps). Rejected as primary; retained as a fingerprinting step inside PrefixSpan to accelerate candidate pruning.

**PrefixSpan** (chosen) — a projection-based frequent sequential pattern mining algorithm. It scans the database once to find frequent length-1 patterns, then projects the database recursively under each prefix to find frequent extensions. It is well-suited to sparse, gap-allowed sequence matching because the projection step naturally handles gaps via a "post-occurrence" model: given prefix P, the projected database for extension item X contains only the suffix of each sequence that follows the first occurrence of X after the last prefix item. This naturally captures gap-allowed matching without generating explicit gap candidates.

Complexity: O(|D| × L × W) where |D| = number of sequences, L = maximum pattern length (capped at 8 in this spec), W = average sequence width (tool calls per cascade cluster, typically 6-20). Practical: single-pass per recursion level with bounded depth.

### Gap-Allowed Matching Model

A sequence database row is a tool-call cluster: an ordered list of step-type tokens for one cascade execution. The pattern `[A, B, C]` matches a cluster `[A, X, B, Y, C]` under gap-allowed semantics because B follows A (possibly non-adjacent) and C follows B.

The gap model used in this spec: **unbounded left-gap, bounded right-gap.** After matching prefix item k, the matcher scans forward in the cluster sequence for the next prefix item k+1. It will match the first occurrence of k+1 within a right-gap window of `MAX_GAP_STEPS` (default 5). Steps that appear more than `MAX_GAP_STEPS` after the last matched step break the match. This prevents spurious matches where A and B are separated by half the session.

### PrefixSpan Algorithm Sketch

```
# Input: sequence_db = [(cluster_id, [step_type_0, step_type_1, ...]), ...]
#        min_support: usize (default 20 per F2)
#        max_length: usize (default 8)
# Output: [(pattern: Vec<StepType>, support: usize, cluster_ids: Vec<u64>)]

fn prefix_span(db, prefix, min_support, max_depth):
    if depth > max_depth: return []
    
    # Count frequency of each item in the projected database
    item_counts = frequency_map(db)  # O(|db| × avg_seq_len)
    
    results = []
    for each (item, support) in item_counts where support >= min_support:
        new_prefix = prefix + [item]
        new_db = project(db, item)  # keep only suffixes after item
        results.push((new_prefix, support, cluster_ids(new_db)))
        results.extend(prefix_span(new_db, new_prefix, min_support, max_depth-1))
    
    return results

fn project(db, item):
    # For each sequence: find the first occurrence of item within MAX_GAP_STEPS
    # of the last matched position; return the remainder after that occurrence.
    result = []
    for (cluster_id, seq) in db:
        for i in 0..seq.len():
            if seq[i] == item:
                # item found; suffix is seq[i+1..]
                result.push((cluster_id, seq[i+1..]))
                break  # leftmost-first match
    return result
```

The gap constraint is enforced inside `project`: when scanning for `item`, the scan is limited to `MAX_GAP_STEPS` positions forward from the last matched position (tracked per sequence in the projected database).

### Rust Trait Sketch for the Keystone Engine

```rust
/// A token in the step sequence. Opaque by default (F11 compliance — no human-readable labels
/// at the pattern level). The display name is resolved via `StepTypeRegistry` at report-emit
/// time only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepToken(u32);

/// A gap-allowed sequential pattern: an ordered list of step tokens with
/// observed support and the set of cluster IDs where it was found.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequentialPattern {
    pub steps: Vec<StepToken>,
    pub support: usize,
    pub cluster_ids: Vec<u64>,
    /// Per-dimension confidence from the fitness tensor.
    pub confidence: PatternConfidence,
}

/// Per-dimension confidence derived from the 12D fitness tensor on observed clusters.
/// Mirrors `m39_fitness_tensor.rs` structure adapted for workflow-trace dimensions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PatternConfidence {
    /// Weighted geometric mean over relevant dimensions (frequency, fitness_delta, recency).
    pub score: f64,
    /// Smoothed over a 6-sample window (30s at 5s ticks) for volatile dimensions.
    pub is_smoothed: bool,
    /// Sample count backing this estimate. Must be >= 20 for a Proposal to be valid.
    pub n_samples: usize,
}

/// The pattern mining engine.
pub trait SequentialPatternMiner {
    /// Mine gap-allowed frequent sequential patterns from the sequence database.
    ///
    /// # Errors
    ///
    /// Returns `MinerError::DatabaseRead` on SQLite failure or `MinerError::EmptyDatabase`
    /// if no clusters have been recorded yet.
    fn mine(
        &self,
        db: &ClusterDatabase,
        min_support: usize,
        max_length: usize,
        max_gap: usize,
    ) -> Result<Vec<SequentialPattern>, MinerError>;

    /// Project the database under prefix `item` for the next recursion level.
    fn project(
        &self,
        db: &ProjectedDatabase,
        item: StepToken,
        max_gap: usize,
    ) -> ProjectedDatabase;
}
```

---

## Sub-Graph Isomorphism and Edit Distance

Two cascade patterns are considered "the same" if their sequence structures are sufficiently similar. This drives the gradient-preservation step in m23 (identifying near-miss variants) and the diversity-enforcement step in m31 (preventing monoculture).

### Comparison Function: Levenshtein Edit Distance on Step Token Sequences

n-gram similarity (Command-2's lean) was considered. It is fast and captures approximate matches well for short sequences. However, n-gram similarity over tokenized sequences has poor behavior at pattern boundaries (prefix/suffix effects inflate similarity for patterns that share a long prefix but differ at the end). Levenshtein edit distance on the full token sequence is chosen as the primary metric because:

1. It captures insertions, deletions, and substitutions uniformly.
2. It degrades gracefully as patterns lengthen.
3. Normalized Levenshtein distance (edit distance / max(len(A), len(B))) gives a [0,1] similarity score that is directly interpretable as "fraction of steps that differ."

Two patterns are considered "the same" if their normalized edit distance is below `SIMILARITY_THRESHOLD` (default 0.25, configurable via `ClusterConfig::similarity_threshold`). Two patterns are "near-misses" if their normalized edit distance is in the range `[0.25, 0.60]`.

```rust
/// Normalized Levenshtein distance between two step-token sequences.
/// Returns a value in [0.0, 1.0] where 0.0 = identical, 1.0 = fully different.
pub fn normalized_edit_distance(a: &[StepToken], b: &[StepToken]) -> f64 {
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return 0.0;
    }
    let dist = levenshtein(a, b);  // standard DP, O(|a| * |b|)
    dist as f64 / max_len as f64
}

/// Whether two patterns are similar enough to be considered the "same" pattern.
pub fn are_same_pattern(a: &SequentialPattern, b: &SequentialPattern, threshold: f64) -> bool {
    normalized_edit_distance(&a.steps, &b.steps) < threshold
}

/// Whether pattern b is a near-miss of pattern a.
pub fn is_near_miss(a: &SequentialPattern, b: &SequentialPattern, lo: f64, hi: f64) -> bool {
    let d = normalized_edit_distance(&a.steps, &b.steps);
    d >= lo && d < hi
}
```

The Levenshtein DP is computed over `StepToken` slices (u32 comparisons), not strings, so it is fast in practice. For maximum pattern length 8 and typical cluster widths 6-20, the matrix is at most 20x20.

---

## F2 Enforcement — Wilson Score CI

Every proposal carries `n_samples`, `ci_lower`, and `ci_upper`. The CI is a **Wilson score interval** for a Bernoulli proportion, where the "success" event is defined per iterator:

- m20 (`cascade_iterator`): success = cascade cluster matched a canonical pattern
- m21 (`battern_iterator`): success = battern execution met its wallclock budget
- m22 (`prompt_pattern_iterator`): success = prompt-template produced a positive fitness delta

The Wilson score interval is preferred over Wald (`p ± z * sqrt(p*(1-p)/n)`) because it is accurate for small samples and remains valid when the proportion is near 0 or 1. The Wald interval can produce negative lower bounds or bounds outside [0,1] for small n; Wilson does not.

### Wilson Score Formula

```
p_hat = successes / n
center = (p_hat + z^2/(2n)) / (1 + z^2/n)
margin = z * sqrt(p_hat*(1-p_hat)/n + z^2/(4n^2)) / (1 + z^2/n)
ci_lower = center - margin
ci_upper = center + margin
```

where `z = 1.96` (95% confidence interval, two-tailed).

### Rust Sketch

```rust
/// Wilson score 95% CI for a proportion.
///
/// # Arguments
/// - `successes`: number of successes observed
/// - `n`: total trials
///
/// # Returns
/// `(ci_lower, ci_upper)` both in `[0, 1]`.
///
/// # Errors
/// Returns `ProposalError::InsufficientSamples` if `n < MIN_SAMPLES`.
pub fn wilson_95(successes: usize, n: usize) -> Result<(f64, f64), ProposalError> {
    const MIN_SAMPLES: usize = 20;
    const Z: f64 = 1.959_964;  // z_{0.975}
    const Z2: f64 = Z * Z;

    if n < MIN_SAMPLES {
        return Err(ProposalError::InsufficientSamples { n, required: MIN_SAMPLES });
    }
    let p = successes as f64 / n as f64;
    let n_f = n as f64;
    let denom = 1.0 + Z2 / n_f;
    let center = (p + Z2 / (2.0 * n_f)) / denom;
    let margin = Z * ((p * (1.0 - p) / n_f) + Z2 / (4.0 * n_f * n_f)).sqrt() / denom;
    Ok(((center - margin).max(0.0), (center + margin).min(1.0)))
}
```

The check is structural: `ProposalBuilder::build()` calls `wilson_95` and fails at construction if `n < 20`. There is no bypass path. The 20-sample requirement is from F2 and is non-negotiable per spec.

---

## Gradient Preservation — N=3 Near-Miss Variants

P0 #10 from the Genesis spec requires that m23 surface N near-miss variants alongside a canonical proposal. The default N=3 is from the NA-Gap gradient preservation constraint. The rule is: do not discard information by presenting only the single best pattern; show the topology of the nearby solution space.

### Schema: `Proposal`

```rust
/// The canonical output unit of Cluster F.
///
/// A Proposal is NEVER auto-promoted to m30 (the bank).
/// It is a read-only record presented to a human for evaluation.
/// Promotion requires explicit human action (CLI: `wf-crystallise propose accept <id>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique proposal ID. Opaque (F11 compliance).
    pub id: ProposalId,
    /// Which iterator generated this proposal.
    pub source: IteratorKind,
    /// The canonical (highest-confidence) pattern.
    pub canonical: SequentialPattern,
    /// N near-miss variants. len <= MAX_VARIANTS (default 3).
    /// Variants are ordered by ascending normalized edit distance from canonical.
    pub variants: Vec<SequentialPattern>,
    /// Confidence band across canonical + all variants.
    pub confidence_band: ConfidenceBand,
    /// Sample count for the canonical pattern. Must be >= 20.
    pub n_samples: usize,
    /// Wilson 95% CI for the canonical.
    pub ci_lower: f64,
    pub ci_upper: f64,
    /// Lineage: which cluster IDs contributed to this proposal.
    pub cluster_lineage: Vec<u64>,
    /// Timestamp when this proposal was generated.
    pub generated_at: u64,  // Unix epoch ms
    /// Deviation rationale (populated by m23 from m32 bypass events).
    pub deviation_evidence: Vec<DeviationEvent>,
    /// Whether the pattern's canonical form was shaped by deviation evidence.
    pub deviation_shaped: bool,
}

/// How similar/different the variants are from the canonical.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConfidenceBand {
    /// Minimum confidence across canonical + variants.
    pub min: f64,
    /// Maximum confidence across canonical + variants.
    pub max: f64,
    /// Standard deviation of confidence scores.
    pub std_dev: f64,
    /// Whether the band is narrow (std_dev < 0.05) — indicates convergent evidence.
    pub is_narrow: bool,
}

/// A deviation event: operator bypassed a step during dispatch (fed back from m32).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviationEvent {
    /// The step that was bypassed.
    pub bypassed_step: StepToken,
    /// Operator-supplied rationale (free text from m32 display-before-step prompt).
    pub rationale: String,
    /// Cluster ID where the deviation occurred.
    pub cluster_id: u64,
    /// Timestamp of the deviation.
    pub ts_ms: u64,
}
```

### Variant Selection Rule: Top-K by Edit Distance from Canonical

Given the PrefixSpan output (a set of `SequentialPattern` items), the canonical is the pattern with the highest `confidence.score`. The N near-miss variants are selected as follows:

1. Filter out all patterns that `are_same_pattern` as the canonical (normalized edit distance < 0.25).
2. Sort remaining patterns by normalized edit distance ascending.
3. Keep the top N (default 3) nearest patterns within the near-miss band [0.25, 0.60].
4. Patterns with edit distance > 0.60 are excluded (too dissimilar to be informative near-misses).

This is not random sampling. The rule is deterministic given the same input — it always picks the N structurally nearest alternatives. This preserves gradient information about the solution space because the nearest alternatives differ in the smallest number of steps from the canonical.

```rust
pub fn select_variants(
    canonical: &SequentialPattern,
    candidates: &[SequentialPattern],
    max_variants: usize,
    near_miss_lo: f64,
    near_miss_hi: f64,
) -> Vec<SequentialPattern> {
    let mut near_misses: Vec<(f64, &SequentialPattern)> = candidates
        .iter()
        .filter(|p| {
            let d = normalized_edit_distance(&canonical.steps, &p.steps);
            d >= near_miss_lo && d < near_miss_hi
        })
        .map(|p| (normalized_edit_distance(&canonical.steps, &p.steps), p))
        .collect();

    near_misses.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    near_misses
        .into_iter()
        .take(max_variants)
        .map(|(_, p)| p.clone())
        .collect()
}
```

---

## Module Specifications

### m20 — `cascade_iterator`

**Job:** Read m4 cluster data and m6 cost data; mine N-step sequential patterns across cascade executions; propose cascade variants for human evaluation with CI bars per F2.

**LOC estimate:** ~200 (new authorship ~170 + lifted pattern ~30)

**Inputs:**
- `workflow_trace.db` table `cascade_clusters` (written by m4)
- `workflow_trace.db` table `context_cost_records` (written by m6)
- m14 `habitat_outcome_lift` metric (read via shared metric store; CC-3 gate)

**Outputs:**
- `Vec<Proposal>` written to `workflow_trace.db` table `cascade_proposals`
- Proposals include `source: IteratorKind::Cascade`

**Reuse map:**
- Kahn's topological sort from `m49_task_graph.rs` (~50% reuse) — used to determine valid step orderings within a cascade cluster before feeding the sequence to PrefixSpan. The task graph already handles cycle detection; cascade clusters are guaranteed DAGs by m4.
- `ProjectedDatabase` struct design informed by m49's adjacency list pattern.

**New authorship (~170 LOC):** The PrefixSpan engine itself (recursive projection, gap-allowed matching, frequency counting), the `StepToken` opaque type, the `PatternConfidence` computation from the 12D fitness tensor dimensions adapted for cascade-specific axes (frequency D3, recency approximated via D9, fitness_delta from m6 cost records).

**Trait sketch:**

```rust
pub struct CascadeIterator {
    db: Arc<WorkflowDb>,
    config: CascadeIteratorConfig,
    miner: Box<dyn SequentialPatternMiner>,
}

impl CascadeIterator {
    /// Run one iteration cycle. Reads from m4 + m6, mines patterns, writes proposals.
    ///
    /// Called on a configurable cadence (default: once per hour by the crystalliser sweep).
    ///
    /// # Errors
    ///
    /// `IteratorError::InsufficientData` if fewer than `min_support` clusters exist.
    pub fn iterate(&self) -> Result<Vec<Proposal>, IteratorError> { ... }
}
```

**CC-3 gate:** Before running, m20 checks whether m14's `habitat_outcome_lift` metric has stabilized (variance < threshold over last 6 measurements). If the metric is still trending, m20 waits — proposing patterns on unstable evidence violates the spirit of F2 even if n >= 20.

**F2 enforcement:** Every proposal emitted by `iterate()` has `n_samples >= 20` or is not emitted. The check is at `ProposalBuilder::build()`.

**Test coverage targets (50 minimum):**
- PrefixSpan correctness (linear sequence, diamond, with-gaps, deep chain)
- Gap constraint enforcement (patterns beyond MAX_GAP_STEPS not matched)
- F2 gate (proposals with n < 20 rejected at build time)
- Wilson CI bounds (non-negative, in [0,1], correct for known proportions)
- Variant selection (top-K by edit distance, near-miss band enforced)
- DB read failure handling (empty database, corrupted row)
- Empty cluster database returns InsufficientData error
- Confidence band computation (min/max/std_dev/is_narrow)
- m14 gate (stabilization check blocks early iteration)
- Opaque ID discipline (no human-readable labels in proposal steps)
- Property tests: CI lower <= CI upper always; confidence in [0,1]; edit distance symmetric

---

### m21 — `battern_iterator`

**Job:** Read m5 battern step records and m6 cost data; propose battern execution variants that minimize either wallclock duration or token cost; CI bars per F2.

**LOC estimate:** ~200

**Inputs:**
- `workflow_trace.db` table `battern_step_records` (written by m5)
- `workflow_trace.db` table `context_cost_records` (written by m6)
- m14 `habitat_outcome_lift` metric (CC-3 gate, same as m20)

**Outputs:**
- `Vec<Proposal>` written to `workflow_trace.db` table `battern_proposals`
- Proposals include `source: IteratorKind::Battern`

**What m21 mines:** The 6-step Battern protocol (Design/Dispatch/Gate/Collect/Synthesize/Compose) produces step-duration and step-outcome records. m21 mines two things:
1. **Step-ordering variants** — are there valid reorderings of the 6 steps (that still satisfy the Battern protocol's hard ordering constraints) that reduce wallclock?
2. **Step-duration outliers** — which Battern instances ran unusually long at a particular step, and what was the cluster context? This surfaces patterns like "Gate always runs long when the cascade cluster contains more than 4 services."

The Battern protocol's ordering constraints (Design must precede Dispatch; Gate must precede Collect) are encoded as a DAG identical to `m49_task_graph.rs`'s DependsOn edges. Only topological orderings that respect these edges are valid reorderings. The task graph's `topological_order()` function provides the enumeration of valid orderings.

**Reuse map:**
- `m49_task_graph.rs` topological sort (~50% reuse) — topological enumeration of valid Battern orderings
- `povm-v2_reinforcement.rs` dedup pattern (~60% reuse) — idempotency cache so the same battern-id is not double-counted across two iterator cycles

**New authorship (~150 LOC):** Step-duration outlier detection (IQR-based), wallclock-vs-cost Pareto frontier computation, Battern-specific success definition for Wilson CI.

**Optimization objective:** Two separate proposals are generated per mining cycle: one optimizing for minimum wallclock (`ProposalKind::WallclockOptimized`) and one for minimum token cost (`ProposalKind::CostOptimized`). The human reviewer sees both and chooses.

**F2 enforcement:** Same as m20. `n_samples` counts the number of Battern executions that contributed to the pattern, not the number of individual steps.

**Test coverage targets (50 minimum):**
- Topological enumeration of valid Battern orderings (correct count for standard 6-step DAG)
- Ordering constraint enforcement (invalid orderings rejected)
- Wallclock and cost proposals generated independently
- Step-duration outlier detection (IQR thresholds correct)
- Dedup prevents same battern-id counted twice
- F2 gate enforced
- Wilson CI bounds correct for Battern success rates
- Empty step records returns InsufficientData
- CC-3 m14 gate blocks premature iteration
- Proposal carries correct `IteratorKind::Battern` variant

---

### m22 — `prompt_pattern_iterator`

**Job:** Read m6 cost records and m7 workflow arc records; identify recurring prompt-template structures (invocation patterns, context-window usage patterns) alongside their measured cost/outcome ratios; propose variants with CI bars per F2.

**LOC estimate:** ~200

**Inputs:**
- `workflow_trace.db` table `context_cost_records` (written by m6)
- `workflow_trace.db` table `workflow_runs` (written by m7)
- m14 `habitat_outcome_lift` metric (CC-3 gate)

**Outputs:**
- `Vec<Proposal>` written to `workflow_trace.db` table `prompt_proposals`
- Proposals include `source: IteratorKind::PromptPattern`

**What m22 mines:** Prompt patterns are extracted from workflow run records. A "prompt pattern" in this context is a structural feature of how a workflow was invoked — not the content of the prompt (which would require NLP), but observable structural attributes:
- Token cost tier (low / medium / high, based on context_cost_records)
- Number of tool calls in the run
- Number of distinct tool types used
- Whether the run included a verification step (m33 call present)
- Run outcome (from m7's fitness_dimension, zero-weighted per F9 but the outcome column is still readable)

These structural attributes form a feature vector that can be clustered. m22 applies a simple K-means variant (K=5 by default, configurable) to the feature vectors to find recurring prompt-pattern clusters. Each cluster is a candidate proposal.

**Why K-means and not PrefixSpan here:** Prompt patterns are not naturally ordered sequences — they are feature vectors. The sequential structure that PrefixSpan exploits is absent. K-means on feature vectors is appropriate and is much simpler to implement correctly for this use case.

**Reuse map:**
- `m39_fitness_tensor.rs` rolling-mean smoothing (~70% reuse) — the 6-sample smoothing window for volatile dimensions is directly applicable to the cost-tier feature, which fluctuates within a session. The `VOLATILE_DIMENSIONS` mask pattern is adapted: token cost tier is volatile, tool-call count is stable.
- `povm-v2_reinforcement.rs` dedup pattern (~60% reuse) — prevents double-counting runs that span multiple context_cost_records rows.

**New authorship (~130 LOC):** Feature vector construction from workflow run records, K-means clustering (online variant with fixed K), cluster-to-Proposal mapping with cost/outcome ratio computation.

**Success definition for Wilson CI:** A run is a "success" if its fitness_delta (from m7 `workflow_runs.fitness_delta`) is positive.

**F2 enforcement:** A proposal for cluster K is valid only if at least 20 runs have been assigned to that cluster. Clusters with fewer than 20 members are dropped.

**Test coverage targets (50 minimum):**
- Feature vector construction (correct fields, correct normalization)
- K-means convergence (terminates within MAX_ITER)
- Cluster membership stability (same inputs yield same clusters)
- Small-cluster filtering (< 20 runs → cluster dropped)
- Wilson CI bounds for prompt-pattern success rates
- Volatile feature smoothing (6-sample window applied to cost_tier)
- Dedup on run IDs
- CC-3 m14 gate
- Proposal carries correct `IteratorKind::PromptPattern`
- Cost/outcome ratio monotonic: higher cost clusters have measurably different ratios

---

### m23 — `workflow_proposer`

**Job:** Aggregate proposals from m20, m21, m22; apply gradient-preservation (surface N=3 near-miss variants alongside each canonical); capture deviation-as-evidence; write final `Proposal` records to the proposals table. This is the single output surface of Cluster F.

**LOC estimate:** ~250 (largest module in cluster — aggregation + gradient-preservation logic + deviation-evidence system)

**Inputs:**
- `workflow_trace.db` tables `cascade_proposals`, `battern_proposals`, `prompt_proposals` (written by m20-m22)
- `workflow_trace.db` table `deviation_events` (written by m32 when operator bypasses a step)

**Outputs:**
- `workflow_trace.db` table `proposals` — the final aggregated proposals visible to the human reviewer

**NEVER writes to:**
- m30's bank (proposals go to humans, not to the bank)
- stcortex (m13 owns all stcortex writes)
- atuin or injection.db

**Reuse map:**
- `SKILL-pre-deploy-hardening.md` deviation-rationale capture pattern (~80% reuse for schema) — the `Verdict { agent, APPROVE/REJECT, evidence }` pattern in pre-deploy hardening is adapted as `DeviationEvent { bypassed_step, rationale, cluster_id, ts_ms }`. The structural idea is identical: capture the reason for the departure at the point of departure.
- `m39_fitness_tensor.rs` per-dimension confidence weighting (~70% reuse) — the `DIMENSION_WEIGHTS` sum-to-1.0 and rolling-mean smoothing infrastructure is adapted for the `ConfidenceBand` computation across variant patterns.

**New authorship (~180 LOC):** The aggregation loop (merging proposals across three iterator types), the cross-type deduplication (a cascade proposal and a prompt-pattern proposal that reference the same cluster should not both appear as canonical in the same output), the deviation-evidence joining, and the gradient-preservation variant selection (see Near-Miss Variant Selection section above).

**Deviation-as-Evidence System:**

When m32 (the dispatcher) presents a step to the operator before execution (per P0 #11 escape-surface display) and the operator chooses to bypass a step with an explicit rationale, m32 writes a row to `deviation_events`. m23 reads these events and joins them to relevant proposals:

- A deviation event naming `step X` as bypassed is joined to any proposal whose canonical pattern contains `step X`.
- If the same step is bypassed multiple times with consistent rationale (similarity threshold on rationale strings, default 0.8 cosine), m23 interprets this as positive signal toward a "step X removed" variant.
- The deviation-shaped variant (canonical pattern with `step X` removed) is added to the `variants` list if it meets a relaxed support threshold (n >= 5, not 20, because deviation events are rare by nature).
- The proposal's `deviation_shaped: bool` field is set to `true` and the `deviation_evidence` array is populated.

This closes the feedback loop: operator behavior during dispatch shapes future proposals. The loop goes: m32 dispatch → operator bypass + rationale → m23 deviation_events read → m23 proposal shaped → human reviews → m30 bank (if accepted). Never auto-promotes.

**Trait sketch:**

```rust
pub struct WorkflowProposer {
    db: Arc<WorkflowDb>,
    config: ProposerConfig,
}

impl WorkflowProposer {
    /// Aggregate proposals from m20-m22, apply gradient-preservation,
    /// join deviation evidence, write to `proposals` table.
    ///
    /// # Returns
    ///
    /// Number of proposals written.
    ///
    /// # Errors
    ///
    /// `ProposerError::DatabaseWrite` on SQLite failure.
    /// `ProposerError::NoIteratorOutput` if all three iterators returned empty.
    pub fn aggregate_and_write(&self) -> Result<usize, ProposerError> { ... }

    /// Select N near-miss variants for a canonical pattern.
    /// Returns at most `self.config.max_variants` (default 3).
    fn select_near_misses(
        &self,
        canonical: &SequentialPattern,
        candidates: &[SequentialPattern],
    ) -> Vec<SequentialPattern> { ... }

    /// Join deviation events to a proposal, marking deviation_shaped if
    /// enough consistent bypass evidence exists for a step in the canonical.
    fn apply_deviation_evidence(
        &self,
        proposal: &mut Proposal,
        events: &[DeviationEvent],
    ) { ... }
}
```

**Human Review Gate Enforcement:**

m23's `aggregate_and_write` marks each written proposal with `status: ProposalStatus::AwaitingReview`. The CLI command `wf-crystallise proposals list` surfaces these. The CLI command `wf-crystallise propose accept <id>` transitions the status to `Accepted` and triggers a write to m30 (by m30's own handler, not m23). m23 never writes to m30 directly.

The transition is owned by m30. m23 is blind to what happens after the proposal is written. This is the architectural guarantee that proposals never auto-promote.

**Test coverage targets (50 minimum):**
- Aggregation correctly merges proposals from all three iterator types
- Cross-type deduplication (same cluster, different iterators → one canonical)
- Variant selection (top-3 near-misses, correct edit distance ordering)
- Deviation event join (correct step matching, consistent rationale detection)
- Deviation-shaped flag set only when sufficient bypass evidence exists
- Relaxed support threshold (n >= 5) applied correctly to deviation-shaped variants
- Empty iterator input → ProposerError::NoIteratorOutput
- Proposals written with status AwaitingReview
- m23 never writes to m30 (verified by test that asserts m30 bank table unchanged)
- ConfidenceBand computation (min/max/std_dev/is_narrow)
- Property test: variants always ordered by ascending edit distance from canonical
- Deviation evidence rationale similarity threshold (0.8 cosine, not exact match)

---

## Lift vs. New Authorship Summary (Cluster F)

| Component | Source | Reuse % | New authorship |
|---|---|---|---|
| DAG cycle detection + topological sort | `m49_task_graph.rs` | 50% | Gap-allowed projection in PrefixSpan |
| Pairwise co-activation API shape | `m20_heat_source_hebbian.rs` | 30% | N-step generalization (~600 LOC keystone) |
| Idempotency dedup cache | `povm-v2_reinforcement.rs` | 60% | Domain-specific success definitions per iterator |
| 12D fitness tensor rolling mean | `m39_fitness_tensor.rs` | 70% | Per-pattern confidence weighting, volatile mask |
| Deviation capture schema | `SKILL-pre-deploy-hardening.md` | 80% | Rationale similarity dedup, deviation-shaped flag |
| Wilson CI implementation | (none — fresh) | 0% | Full implementation (~40 LOC) |
| PrefixSpan engine | (none — fresh) | 0% | Full implementation (~350 LOC across m20+m23) |
| Levenshtein distance on StepToken | (none — fresh) | 0% | ~30 LOC |
| Near-miss variant selection | (none — fresh) | 0% | ~50 LOC |
| Gradient preservation system | (none — fresh) | 0% | ~80 LOC in m23 |

**Total new authorship: ~600-700 LOC (the KEYSTONE gap).** Total lift: ~300-400 LOC. The ratio is inverted from most clusters — Cluster F has more new authorship than lifted code, which is why the Boilerplate Hunt called it out explicitly.

---

## Invariants (all of Cluster F)

1. **PROPOSE, never act.** No module in Cluster F writes to m30, executes tool calls, dispatches to Conductor, or modifies atuin history.
2. **F2 hard gate.** Every `Proposal` emitted by m20-m22 carries `n_samples >= 20`. The `ProposalBuilder::build()` method is the single enforcement point. There is no bypass.
3. **Opaque IDs (F11).** Step tokens are `StepToken(u32)` — numeric, not human-readable. Display names are resolved only at report-emit time (m12), not in the pattern layer.
4. **CC-3 stabilization gate.** m20-m22 check m14 evidence stability before running. An unstable habitat_outcome_lift metric indicates the observational basis is still changing; proposing on changing evidence wastes human attention.
5. **Gradient preservation.** m23 always surfaces N=3 near-miss variants alongside a canonical. The `max_variants` config can reduce this floor but never below 1; the canonical alone without alternatives is not a Proposal, it is a recommendation (forbidden in Phase A verb-discipline language; the structure enforces the discipline).
6. **Human-in-the-loop for bank promotion.** m23's output table `proposals` is the only path to m30. The transition requires `wf-crystallise propose accept <id>` — an explicit human command. There is no automated promotion even if every metric looks good.
7. **Wilson CI, not Wald.** The CI method is not configurable per proposal. All proposals use Wilson score intervals. This prevents later regressions where "Wald was used for this batch" creates inconsistency in archived proposals.
8. **Deviation evidence is additive, not reductive.** Joining deviation events to a proposal adds `deviation_evidence` entries and may add a deviation-shaped variant, but it does not remove the canonical or its standard variants. The human sees everything.

---

## Dependency Graph Within Cluster F

```
m4 (cascade_correlator)     m5 (battern_step_record)    m6 (context_cost_record)
         │                           │                           │
         └──────────────►  m20 (cascade_iterator) ◄────────────►─┘
                                     │
m5 ───────────────────────► m21 (battern_iterator) ◄───────────► m6
                                     │
m6 ───────────────────────► m22 (prompt_pattern_iterator) ◄───► m7
                                     │
                 m14 (evidence_aggregator) ─────► [CC-3 gate applied to all]
                                     │
m20 ──────► m21 ──────► m22 ──────► m23 (workflow_proposer) ◄── deviation_events (m32)
                                     │
                             [human review gate]
                                     │
                             m30 (workflow_bank)
```

---

## SQLite Schema (Cluster F additions to `workflow_trace.db`)

```sql
-- Proposals from cascade_iterator
CREATE TABLE IF NOT EXISTS cascade_proposals (
    id          TEXT PRIMARY KEY,          -- ProposalId (UUID)
    canonical   BLOB NOT NULL,             -- JSON-encoded SequentialPattern
    variants    BLOB NOT NULL,             -- JSON array of SequentialPattern
    n_samples   INTEGER NOT NULL,          -- >= 20
    ci_lower    REAL NOT NULL,
    ci_upper    REAL NOT NULL,
    confidence  BLOB NOT NULL,             -- JSON-encoded ConfidenceBand
    cluster_lineage TEXT NOT NULL,         -- JSON array of u64 cluster IDs
    generated_at INTEGER NOT NULL,         -- Unix epoch ms
    status      TEXT NOT NULL DEFAULT 'awaiting_review'
);

-- Proposals from battern_iterator (same shape)
CREATE TABLE IF NOT EXISTS battern_proposals (
    id          TEXT PRIMARY KEY,
    canonical   BLOB NOT NULL,
    variants    BLOB NOT NULL,
    proposal_kind TEXT NOT NULL,           -- 'wallclock_optimized' | 'cost_optimized'
    n_samples   INTEGER NOT NULL,
    ci_lower    REAL NOT NULL,
    ci_upper    REAL NOT NULL,
    confidence  BLOB NOT NULL,
    cluster_lineage TEXT NOT NULL,
    generated_at INTEGER NOT NULL,
    status      TEXT NOT NULL DEFAULT 'awaiting_review'
);

-- Proposals from prompt_pattern_iterator
CREATE TABLE IF NOT EXISTS prompt_proposals (
    id          TEXT PRIMARY KEY,
    canonical   BLOB NOT NULL,
    variants    BLOB NOT NULL,
    n_samples   INTEGER NOT NULL,
    ci_lower    REAL NOT NULL,
    ci_upper    REAL NOT NULL,
    confidence  BLOB NOT NULL,
    cluster_lineage TEXT NOT NULL,
    generated_at INTEGER NOT NULL,
    status      TEXT NOT NULL DEFAULT 'awaiting_review'
);

-- Aggregated proposals from workflow_proposer (m23)
CREATE TABLE IF NOT EXISTS proposals (
    id                  TEXT PRIMARY KEY,
    source              TEXT NOT NULL,     -- 'cascade' | 'battern' | 'prompt_pattern'
    canonical           BLOB NOT NULL,
    variants            BLOB NOT NULL,
    confidence_band     BLOB NOT NULL,
    n_samples           INTEGER NOT NULL,
    ci_lower            REAL NOT NULL,
    ci_upper            REAL NOT NULL,
    cluster_lineage     TEXT NOT NULL,
    deviation_evidence  BLOB NOT NULL DEFAULT '[]',
    deviation_shaped    INTEGER NOT NULL DEFAULT 0,  -- boolean
    generated_at        INTEGER NOT NULL,
    status              TEXT NOT NULL DEFAULT 'awaiting_review'
);

-- Deviation events written by m32, read by m23
CREATE TABLE IF NOT EXISTS deviation_events (
    id              TEXT PRIMARY KEY,      -- UUID
    bypassed_step   INTEGER NOT NULL,      -- StepToken value
    rationale       TEXT NOT NULL,
    cluster_id      INTEGER NOT NULL,
    ts_ms           INTEGER NOT NULL
);
```

---

## Error Taxonomy (Cluster F)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ClusterFError {
    #[error("insufficient samples: {n} < {required}")]
    InsufficientSamples { n: usize, required: usize },

    #[error("database read failed: {source}")]
    DatabaseRead { #[source] source: rusqlite::Error },

    #[error("database write failed: {source}")]
    DatabaseWrite { #[source] source: rusqlite::Error },

    #[error("no iterator output: all three iterators returned empty")]
    NoIteratorOutput,

    #[error("empty cluster database: cannot mine patterns")]
    EmptyDatabase,

    #[error("pattern too long: {len} > {max}")]
    PatternTooLong { len: usize, max: usize },

    #[error("m14 stabilization gate not met: habitat_outcome_lift variance {variance:.4} > threshold {threshold:.4}")]
    StabilizationGateNotMet { variance: f64, threshold: f64 },

    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

---

## Test Count Targets per Module

| Module | Minimum tests |
|---|---|
| m20 `cascade_iterator` | 50 |
| m21 `battern_iterator` | 50 |
| m22 `prompt_pattern_iterator` | 50 |
| m23 `workflow_proposer` | 50 |
| Cluster F shared types (StepToken, Proposal, CI, edit distance) | 30 |
| **Total Cluster F minimum** | **230** |

Property-based tests (proptest) are required for:
- Wilson CI: `ci_lower <= ci_upper` always; both in `[0, 1]` always
- Normalized edit distance: symmetric; returns 0.0 for equal inputs; returns 1.0 when sequences share no elements and are same length
- `select_variants`: result length always `<= max_variants`; all variants within near-miss band

---

## What This Cluster Does Not Do

These responsibilities are explicitly out of scope for Cluster F:

- Cluster F does not name patterns with human-meaningful labels. Step tokens are opaque (F11). m12 resolves display names from the registry at report time.
- Cluster F does not auto-promote to m30. Full stop. The proposal is presented to a human. The human acts.
- Cluster F does not execute tool calls. It reads from tables that record tool calls but never dispatches to Conductor, LCM, or any service.
- Cluster F does not write to stcortex. m13 handles stcortex writes with LTP/LTD backpressure checks. m23 writes to `workflow_trace.db` only.
- Cluster F does not implement the RALPH fitness-weighted decay for the bank. That is m11 (sunset lifecycle) and m31 (selector). Cluster F's fitness tensor work is used only for pattern confidence scoring, not for bank decay.

---

## Open Questions (for G5 Spec Interview)

The following questions were not resolved during planning and require the G5 structured interview (Watcher + Zen + Luke synchronous participants) before build:

1. **MAX_GAP_STEPS default (currently 5).** Is 5 the right default for cascade gap-allowed matching? A cascade that runs `read_file`, monitoring calls, then `bash`, typically has 1-3 interleaved monitoring calls. A gap of 5 gives margin but may allow spurious matches. Should this be per-cluster-type configurable?

2. **Variant near-miss band [0.25, 0.60].** The lower bound 0.25 separates "same pattern" from "near-miss." The upper bound 0.60 separates "near-miss" from "unrelated." Are these bounds well-calibrated for workflow-trace's step vocabulary size (~40-80 step types)?

3. **K for K-means in m22.** Default K=5. Should this be auto-selected (elbow method) or fixed? Auto-selection adds complexity; fixed K is simpler but may overfit or underfit.

4. **Deviation rationale cosine similarity threshold (0.8).** The text similarity threshold for grouping bypass rationales into a consistent pattern. Is this threshold appropriate, or should it use exact string matching for simplicity?

5. **CC-3 stabilization gate variance threshold.** What variance threshold on `habitat_outcome_lift` constitutes "stable"? The metric's natural variance is not yet known. This may need to be calibrated empirically after the first 120d of observation.

---

*Cluster F spec authored 2026-05-17 (S1001982). Status: planning-only. HOLD-v2 active. Pre-G7 Zen audit.*
*Back to: [[HOME]] · [[MASTER_INDEX]] · [[Modules Synergy Clusters and Feature Verification S1001982]]*
