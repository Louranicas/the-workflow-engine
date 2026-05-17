---
title: m22 — kmeans_feature (Cluster F · L6 · KEYSTONE)
module_id: m22
module_name: kmeans_feature
cluster: F — Iteration (KEYSTONE)
layer: L6
binary: wf-crystallise
feature_gate: [intelligence]
loc_estimate: 170
test_count_min: 60
test_kinds: [property]
verb_class: active
gap_owner: [Gap 1]
mutation_kill_target: 70%
status: planning-only · HOLD-v2 active · pre-G7 Zen audit
authority: Luke @ node 0.A (single-phase override 2026-05-17)
date: 2026-05-17 (S1001982)
kind: per-module spec
decisions_applied: [D-A]
---

# m22 — `kmeans_feature`

> **Sister modules (Cluster F):** [m20](m20_prefixspan_miner.md) · [m21](m21_variant_builder.md) · [m22](m22_kmeans_feature.md) · [m23](m23_workflow_proposer.md)
>
> **Vault:** [[cluster-F-iteration]] · **V7 plan:** [cluster-F plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md) · **Cluster spec:** [`../../../the-workflow-engine-vault/module specs/cluster-F-iteration.md`](../../../the-workflow-engine-vault/module%20specs/cluster-F-iteration.md) · **Matrix row:** [MODULE_MATRIX m22](../../MODULE_MATRIX.md)
>
> **Back to:** [project CLAUDE.md](../../../CLAUDE.md) · [GENESIS v1.3 § 1 + § 3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · [PATTERNS](../../../PATTERNS.md) · [GOLD_STANDARDS](../../../GOLD_STANDARDS.md) · [ANTIPATTERNS](../../../ANTIPATTERNS.md)

---

## 1. Purpose

m22 is the **feature-space context provider** of Cluster F. It clusters observed workflows in a **numeric continuous feature space** (cluster size + step diversity + cost variance + fitness dimension + recency) using **K-means**, producing `FeatureCluster { centroid, members }` artefacts that give m23 (proposer) **context** about where each proposed workflow falls in the feature distribution.

Per [GENESIS v1.3 § 3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) m22 row: verb `cluster` permitted, bounded by `feature space is m6 + m7 only; not pattern-mining surface`. Per [V7 cluster-F plan § m22](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md): K-means is chosen over PrefixSpan **because the feature space is numeric and continuous** — PrefixSpan would be a category-error here (PrefixSpan mines categorical ordered sequences; K-means clusters numeric vectors).

m22 owns a small share of Gap 1 (~50 LOC fresh) — the K-means kernel itself is well-studied and ~70% liftable from `m39_fitness_tensor.rs` accumulator pattern + reference implementations. The workflow-trace-specific authorship is the **feature-extraction wiring** (`workflow_run → Vec<f64>`) and the **per-cluster F2 floor** (clusters with <20 members are dropped).

---

## 2. Feature Space Definition

A "prompt pattern" / workflow-run in this context is not the prompt text (which would require NLP and is out of scope per F9 zero-weight column rule). Instead, m22 extracts **observable structural attributes** from each workflow run record:

| Feature | Source | Type | Volatile? |
|---|---|---|---|
| `f0` cluster_size | m4 `cascade_clusters.step_count` | f64 | no |
| `f1` step_diversity | m4 distinct StepToken count | f64 | no |
| `f2` cost_variance | m6 `context_cost_records.cost_band` rolling SD | f64 | **yes** (EMA smoothed) |
| `f3` fitness_dimension | m7 `workflow_runs.fitness_dimension` | f64 (zero-weighted per F9) | no |
| `f4` recency | m7 `workflow_runs.generated_at` → exponentially-decayed | f64 | **yes** (EMA smoothed) |
| `f5` tool_type_count | m7 distinct tool_types per run | f64 | no |
| `f6` verification_present | m7 m33 call presence flag (0.0 / 1.0) | f64 | no |

The feature vector is `[f0, ..., f6]` — fixed-dimensional, normalised to unit variance per dimension before K-means. Volatile dimensions (`f2`, `f4`) use the **6-sample rolling-mean smoothing** lifted from `m39_fitness_tensor.rs` `VOLATILE_DIMENSIONS` mask pattern.

**Why K-means, not DBSCAN or hierarchical clustering:**

- **K-means is parametric** (fixed K) — predictable runtime and memory; substrate-stable.
- **DBSCAN requires `eps`** — radius selection is feature-scale-sensitive and would need re-tuning per feature change. Bad for substrate-stability.
- **Hierarchical clustering is O(n²)** memory — bad at scale for the projected workload of 10k+ runs.
- **K-means is well-studied** with mature convergence semantics (Lloyd's iteration) and well-understood failure modes.

**K selection:** K=5 default (per [V7 plan § m22 LOC budget](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md)). Auto-selection (elbow / silhouette) was considered and **rejected for substrate-stability** — auto-K introduces a session-to-session non-determinism that breaks Hebbian reinforcement (m42 → stcortex pathways need stable cluster IDs to reinforce). Fixed K is the conservative choice. Operator override via CLI `--k <n>` is permitted but logged.

**Initialisation:** **k-means++** seeding — provably better than uniform-random within an O(log k) factor and equally deterministic given a fixed RNG seed. The seed is `(workflow_trace_db_path_hash XOR run_count)` for reproducibility within a session.

---

## 3. Inputs / Outputs / Substrates

**Reads from** (per [MODULE_MATRIX m22 row](../../MODULE_MATRIX.md)): `m21, m14`.

- `workflow_trace.db` table `context_cost_records` (written by m6) — for `f2` cost_variance
- `workflow_trace.db` table `workflow_runs` (written by m7) — for `f3`, `f4`, `f5`, `f6`
- `workflow_trace.db` table `cascade_clusters` (written by m4) — for `f0`, `f1`
- m14 `habitat_outcome_lift` (CC-3 gate)

Note: m22 does **not** consume m21's `PatternVariant` despite MODULE_MATRIX listing `m21` as upstream. That row reads "m21, m14" to indicate m22 fires **after** m21 in cluster-F ordering. m22 operates on feature vectors, not patterns; the data-flow dependency is temporal (run-ordering) not value-consuming.

**Writes to:** `m23 input` — `Vec<FeatureCluster>` consumed in-memory by m23. No SQLite write.

**Substrates:** None. No atuin, stcortex, POVM, injection.db writes.

**CC-3 (E → F) gate:** Same as m20 — m22 reads m14's `habitat_outcome_lift` and checks stabilization variance before clustering. If gate open, returns `Err(ClusterError::StabilizationGateNotMet)`.

**Verb class:** `active` — permitted: `cluster` (K-means in feature space); bounded by feature_space restricted to numeric m6/m7 columns only (no pattern-mining surface).

---

## 4. Public API (planning-spec; markdown only)

```rust
// planning-spec only — m22_kmeans_feature public surface
// rationale: fixed-K clustering with k-means++ init + per-cluster F2 floor

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// A feature vector representing a single workflow run.
/// Fixed-dimensional: 7 features (f0..=f6). Normalised to unit variance per dim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureVector {
    pub workflow_id: u64,
    pub features: [f64; 7],
    /// Whether `f2` and `f4` were EMA-smoothed (always true after first 6 samples).
    pub volatile_smoothed: bool,
}

/// A K-means cluster produced by m22.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureCluster {
    /// 0..K cluster index (stable within a single mining run).
    pub cluster_id: u32,
    /// Centroid in 7-D feature space.
    pub centroid: [f64; 7],
    /// Workflow IDs assigned to this cluster.
    pub members: Vec<u64>,
    /// Within-cluster sum of squared distances to centroid.
    pub inertia: f64,
}

/// Newtype for K cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KValue(pub usize);

#[derive(Debug, Error)]
pub enum ClusterError {
    #[error("k must be >= 1; got {0}")]
    KZero(usize),

    #[error("not enough workflow runs to cluster: {n} < {required}")]
    InsufficientRuns { n: usize, required: usize },

    #[error("k-means did not converge within {max_iter} iterations")]
    DidNotConverge { max_iter: usize },

    #[error("m14 stabilization gate not met: variance {variance:.4} > threshold {threshold:.4}")]
    StabilizationGateNotMet { variance: f64, threshold: f64 },

    #[error("database read failed: {source}")]
    DatabaseRead { #[source] source: rusqlite::Error },
}

/// Run k-means feature clustering. Single public entry point.
///
/// # Determinism
///
/// k-means++ seeding uses RNG seeded with `(db_path_hash XOR run_count)`.
/// Same DB state → same clusters. Cluster IDs (0..K) are stable within a run
/// but NOT across mining cycles (centroid order may shift).
///
/// # F2 enforcement
///
/// Clusters with `members.len() < 20` are **dropped** from the output. A proposal
/// for cluster K is valid only if at least 20 runs have been assigned (per
/// cluster-F-iteration spec § m22 F2 enforcement).
pub fn cluster_features(
    db: &WorkflowDb,
    k: KValue,
    max_iter: usize,
) -> Result<Vec<FeatureCluster>, ClusterError> { /* see § 5 */ }
```

---

## 5. Algorithm Sketch (planning-spec only)

```rust
// planning-spec only — m22_kmeans_feature::kmeans
// rationale: Lloyd's iteration with k-means++ init + 6-sample rolling smooth on f2/f4

fn cluster_features(
    db: &WorkflowDb,
    k: KValue,
    max_iter: usize,
) -> Result<Vec<FeatureCluster>, ClusterError> {
    if k.0 == 0 { return Err(ClusterError::KZero(0)); }

    // Read workflow runs and extract feature vectors
    let vectors: Vec<FeatureVector> = extract_feature_vectors(db)?;
    const MIN_RUNS: usize = 20; // F2 floor for k=1 case
    if vectors.len() < MIN_RUNS {
        return Err(ClusterError::InsufficientRuns { n: vectors.len(), required: MIN_RUNS });
    }

    // Normalise to unit variance per dimension (zero-mean, unit-SD)
    let normalised = normalise_features(&vectors);

    // k-means++ seeding with deterministic RNG
    let seed = db_path_hash(db) ^ (vectors.len() as u64);
    let mut centroids: Vec<[f64; 7]> = kmeans_pp_seed(&normalised, k.0, seed);

    // Lloyd's iteration
    let mut assignments: Vec<u32> = vec![0; normalised.len()];
    for _iter in 0..max_iter {
        let mut changed = false;
        for (i, v) in normalised.iter().enumerate() {
            let new_a = nearest_centroid(&v.features, &centroids);
            if assignments[i] != new_a { assignments[i] = new_a; changed = true; }
        }
        // Re-compute centroids as mean of assigned points
        centroids = recompute_centroids(&normalised, &assignments, k.0);
        if !changed { break; }
    }

    // Build FeatureClusters and drop those below F2 floor
    let mut clusters: Vec<FeatureCluster> = (0..k.0 as u32).map(|cid| {
        let members: Vec<u64> = normalised.iter().zip(&assignments)
            .filter(|(_, a)| **a == cid)
            .map(|(v, _)| v.workflow_id)
            .collect();
        let inertia = compute_inertia(&normalised, &assignments, cid, &centroids[cid as usize]);
        FeatureCluster {
            cluster_id: cid,
            centroid: centroids[cid as usize],
            members,
            inertia,
        }
    }).collect();

    // F2 floor: drop clusters with < 20 members
    clusters.retain(|c| c.members.len() >= 20);
    clusters.sort_by_key(|c| c.cluster_id); // deterministic output order
    Ok(clusters)
}
```

**Convergence rule:** Lloyd's iteration terminates when no assignment changed (locked) or `max_iter` reached (returns `DidNotConverge` if max_iter without stability). `max_iter` default 100; in practice K-means on 7-D feature vectors with K=5 converges in <20 iterations.

**Volatile feature smoothing:** features `f2` (cost_variance) and `f4` (recency) are EMA-smoothed over the **6-sample window** (30s at 5s ticks per habitat convention) before being fed to K-means. The smoothing logic is lifted ~70% from `m39_fitness_tensor.rs` `VOLATILE_DIMENSIONS` mask pattern.

---

## 6. Boilerplate Lift / New Authorship

Per [GOD_TIER_CONSOLIDATION Part IV Category 04 + V7 plan § m22](../../../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md):

| Component | Source | Reuse | New |
|---|---|---|---|
| 6-sample rolling-mean smoothing | `m39_fitness_tensor.rs` | 70% | volatile feature mask |
| Idempotency dedup cache | `povm-v2_reinforcement.rs` | 60% | run-ID dedup |
| K-means kernel (Lloyd's) | reference / `linfa-clustering` style | 60% | adapted from m39 accumulator |
| k-means++ seeding | reference implementations | 40% | seeded with db_path_hash XOR run_count |
| Feature extraction (run → Vec<f64>) | none | 0% | **~80 LOC fresh** (workflow-trace-specific) |
| Per-cluster F2 floor (drop <20) | none | 0% | ~10 LOC fresh |
| Public surface + error taxonomy | conventional | 30% | ~30 LOC fresh |

**Total fresh authorship for m22:** ~80-100 LOC of the cluster's ~600-700 LOC Gap 1 budget. **Lifted:** ~70-90 LOC (K-means kernel + smoothing).

K-means is library-thin in the Rust ecosystem (`linfa-clustering`); m22 **vendors a workflow-trace-specific version** to (a) avoid the dep weight of `linfa`, (b) honour the deterministic seeding contract, and (c) keep the per-cluster F2 floor enforced at construction time without library plumbing.

---

## 7. F2 Enforcement (per-cluster floor)

F2 in m22 means **per-cluster** n >= 20 — a cluster is valid only if at least 20 workflow runs have been assigned to it. Clusters below the floor are **dropped from the output** (not raised as an error); the assumption is that small clusters represent edge cases that lack the statistical mass to support a Proposal under m23's `ProposalBuilder::build()` Wilson CI gate.

The drop is **silent at the m22 layer** but **logged via `tracing::info!`** so observers (Watcher) can see refusal events. This is Class C (refusal) behaviour and is the correct shape per [feedback_runbook_probe_freshness](../../../ANTIPATTERNS.md).

There is also a **global floor** at K-means entry: if `vectors.len() < 20`, `cluster_features` returns `Err(InsufficientRuns)` immediately without running K-means.

---

## 8. Tests (target 60)

Per [MODULE_MATRIX m22 row](../../MODULE_MATRIX.md): `test_count_min: 60`, kind `property`.

Per [V7 cluster-F plan § m22 test budget](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md):

| Family | Count | Notes |
|---|---:|---|
| F-Unit | 30 | per-arm coverage of feature extraction, normalisation, k-means++, Lloyd's |
| F-Property | 8 | invariants (see below) |
| F-Fuzz | 0 | not in m22's budget |
| F-Integration | 15 | m4/m6/m7 → m22 → m23 chain |
| F-Contract | 3 | `FeatureCluster` serde roundtrip; `m23.ProposalBuilder` consumer contract |
| F-Regression | 3 | pre-seeded bug classes |
| F-Mutation | ≥70% kill | per G6 standard floor |
| **Total** | **≥60** | |

**Property invariants (proptest):**

1. **Convergence:** for any input with `vectors.len() >= 20`, k-means terminates within `max_iter` (default 100)
2. **K=1 degenerate:** with K=1, all members assigned to single cluster; centroid = mean
3. **Identical-points handling:** if all input vectors are identical, K-1 clusters end up empty; only one cluster has members (dropped if <20)
4. **F2 floor:** every emitted cluster has `members.len() >= 20`
5. **Centroid stability:** for deterministic seed, two runs on identical input produce identical centroids
6. **Bounded inertia:** `inertia >= 0.0` always
7. **Cluster ID monotonic:** output `cluster_id` field is in `0..K` ascending after sort
8. **Normalisation invariance:** scaling all input features by constant C does not change cluster membership

**Pre-seeded regression bug classes:**

- Centroid recompute uses median instead of mean (mutation must die)
- Distance metric uses Manhattan instead of Euclidean (mutation must die)
- F2 floor uses `> 20` instead of `>= 20` (off-by-one must die)

---

## 9. Antipatterns Avoided

| Antipattern | Mitigation in m22 |
|---|---|
| **AP-V7-09** (substrate-frame confusion) | Features are **numeric and operationalised** (cluster_size, cost_variance, fitness_dimension) — never user-intent surrogates. Watcher Class G pre-positioned. |
| **AP-V7-03** (verb collapse) | m22 clusters; never recommends or dispatches. |
| **AP-V7-07** (auto-promotion m23→m30) | m22 does not write to m30; output is in-memory `Vec<FeatureCluster>` consumed by m23 only. |
| **AP-Test-01** (coverage theatre) | Mutation ≥70% on Lloyd's iteration + centroid recompute + distance metric. |
| **F2** (sample-size inflation) | Per-cluster floor: clusters with <20 members dropped at output. Global floor at entry. |
| **F11** (cascade monoculture) | Features are derived from m4 cluster IDs (opaque) and m6/m7 numeric columns; never pane-label semantics. |
| **AP29** (sync HTTP in tokio::spawn) | m22 is pure-CPU and synchronous; no HTTP / async surface. |
| **AP30** (namespace string drift) | All db reads use `workflow_trace_*` namespace constants. |

---

## 10. Substrate / Watcher Class Pre-Position

- **Class C** (refusal) at every per-cluster F2 drop (cluster < 20 members) — refusal IS correct behaviour; Watcher logs as healthy operation.
- **Class G** (substrate-frame confusion) if feature definitions drift toward user-intent surrogates. Watcher pre-positioned to flag at G7 audit if new features are introduced without "operationalised observable" justification.
- **Class A** (activation transition) at first `cluster_features` success post-G9.
- **Class I** (Hebbian silence) if K-means runs successfully but no FeatureCluster ever has >= 20 members — indicates substrate is too sparse for clustering; m22 emits empty for weeks. Surfaces a real problem (not enough observations) rather than a fault.

**Atuin trajectory anchor:** `wt-kmeans-snapshot` (proposed) captures per-invocation `(k_used, iterations_to_convergence, mean_inertia, n_clusters_dropped_by_f2, total_runtime_ms)`. Trend of `n_clusters_dropped_by_f2` declining over time = substrate has matured enough to support proposals.

---

## 11. Open Questions (Zen G7 audit + G5 spec interview)

1. **K default = 5.** Should this be auto-selected via elbow method or silhouette score? **Recommend fixed K=5** for substrate stability (auto-K breaks Hebbian reinforcement cluster-id mapping). Operator override via CLI `--k <n>` permitted. G5 to confirm.
2. **Feature set membership.** Currently 7 features (`f0..f6`). Should `f3` fitness_dimension be excluded per F9 zero-weight column rule? **Read** is permitted under F9; `f3` is included for clustering context only and is not used by m23 to score proposals.
3. **k-means++ vs uniform random seeding.** Locked at k-means++ (provably better, equally deterministic). Pure-Rust implementation needed (~30 LOC), or lift from `linfa-clustering`?
4. **Normalisation strategy.** Currently unit-variance per dim (z-score). Alternative: min-max to `[0, 1]`. z-score is more robust to outliers; min-max is more interpretable. **Recommend z-score**; revisit if cluster shapes look pathological at first run.
5. **EMA smoothing window for f2/f4.** 6 samples (30s at 5s ticks) lifted from m39. Confirm this is the right window for workflow-run cadence (slower than m39's fitness tensor tick rate).
6. **Empty-cluster handling.** When Lloyd's leaves a cluster empty, K-means++ should re-seed that centroid. Locked? Or accept the empty cluster (which would then be dropped by F2 floor)? Recommend re-seeding for cluster count stability.

---

## 12. Synergy / Sister-Module Anchors

- **m20** ([m20_prefixspan_miner.md](m20_prefixspan_miner.md)) — orthogonal upstream (Pure Rust per D-A, Luke S1002127). m22 reads workflow_runs (which include patterns mined by m20 transitively) but does not consume `Pattern` directly. Different algorithm shape (numeric clustering vs categorical sequence mining).
- **m21** ([m21_variant_builder.md](m21_variant_builder.md)) — **parallel, NOT consumed.** Variants are topologically-adjacent patterns; feature clusters are orthogonal context. Crossing this boundary would create premature coupling.
- **m23** ([m23_workflow_proposer.md](m23_workflow_proposer.md)) — direct consumer. m23 reads `Vec<FeatureCluster>` to attach `feature_centroid` context to each Proposal's `Evidence` payload.
- **m14** (Cluster E) — CC-3 upstream gate; m22 checks `habitat_outcome_lift` stabilization before clustering.
- **m4 / m6 / m7** (Cluster B + C observers) — primary input substrate via `workflow_trace.db` tables.
- **m23 → m30** (Cluster G) — **m23 NEVER auto-promotes to m30** (AP-V7-07). FeatureCluster context is purely informational; never directly causes admission.

---

## 13. Verification Trail

- **Frontmatter complete:** ✓ cluster, layer, binary, feature_gate, gap_owner, test_kinds, verb_class, test_count_min
- **Cluster ownership:** Cluster F · L6 · KEYSTONE
- **Gap ownership:** Gap 1 (feature-context slice) — ~80-100 LOC fresh of cluster ~600-700 LOC budget
- **Cross-cluster contracts:** CC-3 (E → F via m14) consumed; CC-4 (F → G) downstream via m23
- **F2 gate referenced:** § 7 (per-cluster floor at output + global floor at entry)
- **AP-V7-07 (no auto-promote):** § 9, § 12
- **AP-V7-09 (substrate-frame):** § 9 (features are operationalised observables only)
- **Bidi anchors:** Sister modules (4) · Vault (1) · V7 plan (1) · Cluster spec (1) · Matrix row (1) · GENESIS v1.3 (1) · PATTERNS / GOLD_STANDARDS / ANTIPATTERNS (3) — ✓
- **Word count:** ~1,900 (within KEYSTONE 1,500-2,500 range)
- **No `.rs` files authored:** ✓ planning-only, HOLD-v2 respected
- **Rust fenced blocks are spec documentation:** ✓ labelled "planning-spec only" inline

*m22 spec authored 2026-05-17 (S1001982). Cluster F KEYSTONE — feature-context slice of Gap 1. Planning-only per HOLD-v2 + AP24. Pre-G7 Zen audit.*
