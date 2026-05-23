//! `m22_kmeans_feature` — K-means feature clustering for workflow
//! variants. Cluster F · L6.
//!
//! Lightweight Lloyd's algorithm with k-means++ seeding and bounded
//! iterations. Used by m23 (proposer) for diversity-distance scoring;
//! by m31 (selector) for diversity-weighted bank admission.

use thiserror::Error;

use crate::m21_variant_builder::{MutationKind, WorkflowVariant};
use crate::m4_cascade::cluster_id::fnv1a_64;

/// Default maximum iterations of Lloyd's algorithm.
pub const DEFAULT_MAX_ITERATIONS: usize = 50;

/// Convergence threshold for the per-iteration centroid-shift L2-norm.
pub const DEFAULT_CONVERGENCE_EPSILON: f64 = 1e-6;

/// A clustered feature point.
#[derive(Debug, Clone, PartialEq)]
pub struct ClusteredPoint {
    /// Original point coordinates.
    pub coords: Vec<f64>,
    /// Assigned cluster index in `[0, k)`.
    pub cluster: usize,
}

/// K-means errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum KMeansError {
    /// `k` exceeded the number of points.
    #[error("k={k} exceeds point count {n}")]
    KExceedsN {
        /// Requested k.
        k: usize,
        /// Available points.
        n: usize,
    },
    /// `k == 0` or inputs were empty.
    #[error("empty input or zero k")]
    Empty,
    /// Points had inconsistent dimensionality.
    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimMismatch {
        /// Expected dim.
        expected: usize,
        /// Observed dim.
        got: usize,
    },
    /// An input coordinate was non-finite (NaN or infinity).
    #[error("non-finite coordinate at point {point}, dimension {dim}")]
    NonFiniteCoordinate {
        /// Index of the offending input point.
        point: usize,
        /// Index of the offending coordinate within that point.
        dim: usize,
    },
}

/// Configuration.
#[derive(Debug, Clone)]
pub struct KMeansConfig {
    /// Number of clusters.
    pub k: usize,
    /// Maximum iterations.
    pub max_iterations: usize,
    /// Convergence epsilon.
    pub convergence_epsilon: f64,
    /// Deterministic seed.
    pub seed: u64,
}

impl Default for KMeansConfig {
    fn default() -> Self {
        Self {
            k: 3,
            max_iterations: DEFAULT_MAX_ITERATIONS,
            convergence_epsilon: DEFAULT_CONVERGENCE_EPSILON,
            seed: 0xcbf2_9ce4_8422_2325,
        }
    }
}

/// Run K-means with k-means++ seeding.
///
/// # Errors
///
/// - [`KMeansError::Empty`] if `points.is_empty()` or `k == 0`.
/// - [`KMeansError::KExceedsN`] if `k > points.len()`.
/// - [`KMeansError::DimMismatch`] if points have inconsistent dimensions.
/// - [`KMeansError::NonFiniteCoordinate`] if any coordinate is NaN or infinite.
pub fn kmeans(
    points: &[Vec<f64>],
    config: &KMeansConfig,
) -> Result<(Vec<ClusteredPoint>, Vec<Vec<f64>>), KMeansError> {
    if points.is_empty() || config.k == 0 {
        return Err(KMeansError::Empty);
    }
    if config.k > points.len() {
        return Err(KMeansError::KExceedsN {
            k: config.k,
            n: points.len(),
        });
    }
    let dim = points[0].len();
    for (point, p) in points.iter().enumerate() {
        if p.len() != dim {
            return Err(KMeansError::DimMismatch {
                expected: dim,
                got: p.len(),
            });
        }
        // F-m22-02 hardening — non-finite coords (NaN / +∞ / -∞) produce
        // ill-defined centroids. Reported via the dedicated
        // `NonFiniteCoordinate` variant carrying the offending point and
        // coordinate indices: a value error is kept distinct from a
        // dimension-shape error (`DimMismatch`).
        for (d, v) in p.iter().enumerate() {
            if !v.is_finite() {
                return Err(KMeansError::NonFiniteCoordinate { point, dim: d });
            }
        }
    }
    let mut centroids = kmeans_plus_plus_seed(points, config.k, config.seed);
    // Capacity hint: per-iteration assignments are exactly `points.len()`.
    let mut assignments: Vec<usize> = Vec::with_capacity(points.len());
    for _ in 0..config.max_iterations {
        assignments.clear();
        assignments.extend(points.iter().map(|p| nearest_centroid(p, &centroids)));
        let new_centroids = recompute_centroids(points, &assignments, config.k, dim, &centroids);
        let shift = centroid_shift(&centroids, &new_centroids);
        centroids = new_centroids;
        if shift < config.convergence_epsilon {
            break;
        }
    }
    let final_assignments: Vec<usize> = points
        .iter()
        .map(|p| nearest_centroid(p, &centroids))
        .collect();
    let mut clustered: Vec<ClusteredPoint> = Vec::with_capacity(points.len());
    for (p, &c) in points.iter().zip(final_assignments.iter()) {
        clustered.push(ClusteredPoint {
            coords: p.clone(),
            cluster: c,
        });
    }
    Ok((clustered, centroids))
}

/// Normalisation cap for step-count feature: pattern lengths above this saturate to 1.0.
pub const FEATURE_STEP_COUNT_NORM: f64 = 20.0;

/// Normalisation cap for the Levenshtein-from-identity proxy: the canonical max
/// per-variant edit distance is 2.0 (Swap = two transpositions / pair of substitutions).
pub const FEATURE_LEVENSHTEIN_NORM: f64 = 2.0;

/// Soft maximum on `k` returned by [`recommended_k_for_variant_count`].
pub const RECOMMENDED_K_MAX: usize = 8;

/// Extract a 5-dimensional feature vector from a [`WorkflowVariant`] for K-means clustering.
///
/// Per Completion Plan v2 § 15 D17 (S1004115), the feature dimensions are:
///
/// - **dim 0** — `step-count-norm`: `min(steps.len() / FEATURE_STEP_COUNT_NORM, 1.0)`.
///   Pattern lengths above [`FEATURE_STEP_COUNT_NORM`] saturate to `1.0` so an
///   outlier long pattern does not dominate L2 distance.
/// - **dims 1-3** — mutation kind as a one-hot triple `[identity_hot, swap_hot,
///   skip_hot]` (exactly one is `1.0`, the other two are `0.0`).
/// - **dim 4** — `levenshtein-from-identity-norm`: a deterministic closed-form
///   proxy for the edit distance from the [`MutationKind::Identity`] baseline,
///   divided by [`FEATURE_LEVENSHTEIN_NORM`] to land in `[0.0, 1.0]`:
///   - `MutationKind::Identity` → `0.0` (the baseline)
///   - `MutationKind::Skip { .. }` → `0.5` (one deletion = 1 edit / 2.0)
///   - `MutationKind::Swap { .. }` → `1.0` (one swap ≈ two transpositions / 2.0)
///
/// All five dimensions land in `[0.0, 1.0]` so K-means' L2 distance is not
/// dominated by any one axis. The function is **deterministic** (no PRNG, no
/// floating-point equality on input) and totally defined for every
/// `WorkflowVariant`.
///
/// # Why this proxy for Levenshtein
///
/// A true Levenshtein-from-Identity would require reconstructing the source
/// pattern's step sequence to compare against, but `WorkflowVariant` carries
/// only the `source_pattern_hash` (an opaque FNV-1a u64), not the steps. The
/// closed-form proxy maps each [`MutationKind`] variant to its canonical
/// minimum edit-distance cost (Identity = 0, Skip = 1 deletion, Swap = 2
/// substitutions), preserving the ordering and adding finer separation on
/// dim 4 than the mutation one-hot alone would.
#[must_use]
pub fn extract_variant_features(variant: &WorkflowVariant) -> Vec<f64> {
    #[allow(
        clippy::cast_precision_loss,
        reason = "step lengths are tiny (<= MAX_STEPS_NORM expected ~ 20); precision loss \
                  on the cast is below f64 mantissa range"
    )]
    let step_norm = (variant.steps.len() as f64 / FEATURE_STEP_COUNT_NORM).min(1.0);
    let (identity_hot, swap_hot, skip_hot) = match variant.mutation {
        MutationKind::Identity => (1.0_f64, 0.0_f64, 0.0_f64),
        MutationKind::Swap { .. } => (0.0_f64, 1.0_f64, 0.0_f64),
        MutationKind::Skip { .. } => (0.0_f64, 0.0_f64, 1.0_f64),
    };
    let levenshtein_proxy = match variant.mutation {
        MutationKind::Identity => 0.0_f64,
        MutationKind::Skip { .. } => 1.0_f64,
        MutationKind::Swap { .. } => 2.0_f64,
    };
    let lev_norm = (levenshtein_proxy / FEATURE_LEVENSHTEIN_NORM).min(1.0);
    vec![step_norm, identity_hot, swap_hot, skip_hot, lev_norm]
}

/// Adaptive choice of `k` (cluster count) for a given variant count.
///
/// Per Completion Plan v2 § 15 D19 (S1004115), `k` is derived from the variant
/// count rather than configured externally. Heuristic:
///
/// ```text
/// k = round(sqrt(n / 2.0)).clamp(1, RECOMMENDED_K_MAX).min(n)
/// ```
///
/// Rationale: `sqrt(n / 2.0)` keeps clusters meaningfully populated (≥ 2
/// points/cluster expected on uniform data) without degenerating to a single
/// cluster for small N or to N clusters for large N. The clamp ensures `k`
/// stays sane for outlier counts; the `min(n)` floor satisfies
/// [`kmeans`]'s precondition `k <= points.len()`.
///
/// - `n == 0` returns `1` (never `0`, to avoid tripping [`KMeansError::Empty`]
///   when the caller forwards into [`kmeans`]; the caller is expected to
///   skip [`kmeans`] entirely when `n == 0`).
/// - `n == 1` returns `1` (a degenerate single-cluster).
/// - `n in [2..=4]` returns `1` or `2` per the sqrt rule.
/// - large `n` saturates at [`RECOMMENDED_K_MAX`].
#[must_use]
pub fn recommended_k_for_variant_count(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "variant counts are bounded by MAX_VARIANTS_PER_PATTERN * pattern_count, \
                  well within f64 mantissa precision for any realistic workflow-trace input"
    )]
    let raw_f = ((n as f64) / 2.0_f64).sqrt().round();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "sqrt of a non-negative finite f64 is non-negative finite; round() is finite; \
                  the clamp+min below caps the usize cast at RECOMMENDED_K_MAX which is tiny"
    )]
    let raw = raw_f as usize;
    raw.clamp(1, RECOMMENDED_K_MAX).min(n)
}

fn nearest_centroid(p: &[f64], centroids: &[Vec<f64>]) -> usize {
    let mut best = 0_usize;
    let mut best_d = f64::INFINITY;
    for (i, c) in centroids.iter().enumerate() {
        let d = squared_l2(p, c);
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}

fn squared_l2(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum()
}

fn recompute_centroids(
    points: &[Vec<f64>],
    assignments: &[usize],
    k: usize,
    dim: usize,
    prior: &[Vec<f64>],
) -> Vec<Vec<f64>> {
    let mut sums: Vec<Vec<f64>> = vec![vec![0.0; dim]; k];
    let mut counts: Vec<usize> = vec![0; k];
    for (p, &a) in points.iter().zip(assignments.iter()) {
        if a >= k {
            continue;
        }
        for (s, v) in sums[a].iter_mut().zip(p.iter()) {
            *s += v;
        }
        counts[a] += 1;
    }
    let mut out: Vec<Vec<f64>> = Vec::with_capacity(k);
    for (i, s) in sums.into_iter().enumerate() {
        if counts[i] == 0 {
            // F-m22-01 fix — empty cluster mid-iteration: retain the prior
            // centroid rather than recentering on the origin. This preserves
            // determinism, avoids NaN/origin drift, and keeps the algorithm
            // convergent. Per Cluster F spec invariant: "empty-cluster
            // scenario … must be handled (typed error or re-seed, NOT
            // NaN/panic)." Retaining the prior is the canonical Lloyd's
            // recovery action.
            if let Some(p) = prior.get(i) {
                out.push(p.clone());
            } else {
                // Degenerate: prior was shorter than k. Use origin as
                // last-resort fallback (only reachable if seeding was
                // pathologically incomplete — defensive).
                out.push(vec![0.0_f64; dim]);
            }
            continue;
        }
        #[allow(
            clippy::cast_precision_loss,
            reason = "counts[i] is bounded by point count which is well within f64 mantissa precision for any realistic workflow-trace input"
        )]
        let n_f = counts[i] as f64;
        out.push(s.into_iter().map(|v| v / n_f).collect());
    }
    out
}

fn centroid_shift(a: &[Vec<f64>], b: &[Vec<f64>]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| squared_l2(x, y).sqrt())
        .sum()
}

fn kmeans_plus_plus_seed(points: &[Vec<f64>], k: usize, seed: u64) -> Vec<Vec<f64>> {
    let mut centroids: Vec<Vec<f64>> = Vec::with_capacity(k);
    // Seed deterministically via FNV-1a hashing of `seed`. The 64-bit hash
    // is reduced into `[0, points.len())` by `u64`-modulo BEFORE the
    // `usize` cast: `hash % len` is always strictly less than `len`, which
    // is itself a `usize` value, so the result is a valid index that fits
    // `usize` on every target — no fallback is needed or possible.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "`hash % len_u64` is < len, which originated as a usize, so the value fits usize losslessly"
    )]
    let first_idx = (fnv1a_64(&seed.to_le_bytes()) % points.len() as u64) as usize;
    centroids.push(points[first_idx].clone());
    for i in 1..k {
        let mut best_idx = 0_usize;
        let mut best_dist = -1.0_f64;
        let mut best_tiebreak = 0_u64;
        for (idx, p) in points.iter().enumerate() {
            let d = centroids
                .iter()
                .map(|c| squared_l2(p, c))
                .fold(f64::INFINITY, f64::min);
            // F6 fix — the tiebreak hash is consulted ONLY to break exact
            // distance ties, never to perturb farthest-point selection.
            //
            // The prior implementation (H7 carry-forward S1002600) folded a
            // bounded bias `(tiebreak % 1024) * ε * d.max(1.0)` into EVERY
            // candidate's distance and compared the perturbed value `dt`.
            // Even though the bias was small (~2.3e-13 · max(d,1)) it still
            // perturbed real selection: two points whose true squared-L2
            // distances differed by less than the bias magnitude could be
            // reordered by the hash. That made k-means++ "pick the farthest
            // point" subtly hash-determined for near-equal candidates.
            //
            // The correct k-means++ invariant is: pick the point with the
            // strictly greatest min-distance `d`; only when two candidates
            // have EXACTLY-equal `d` (bit-identical) does the deterministic
            // hash decide. `d` is therefore compared first; the hash-based
            // `tiebreak` is consulted purely as a secondary key. This keeps
            // the seeding deterministic without ever flipping the ordering
            // of materially-different distances.
            let tiebreak = fnv1a_64(
                &[
                    (idx as u64).to_le_bytes(),
                    (i as u64).to_le_bytes(),
                    seed.to_le_bytes(),
                ]
                .concat(),
            );
            // Primary key: real distance `d` (strictly greater wins).
            // Secondary key: hash `tiebreak` (greater wins) — reached ONLY
            // when `d == best_dist` exactly.
            //
            // The `d == best_dist` comparison is deliberately EXACT (not an
            // epsilon margin): the F6 contract is that the hash only ever
            // decides bit-identical distances. An epsilon band would
            // re-introduce the very perturbation F6 removes — points whose
            // true distances differ by less than ε would again be
            // hash-reordered. Exact equality is therefore correct here.
            #[allow(
                clippy::float_cmp,
                reason = "F6: the hash tiebreak must apply ONLY to bit-exact \
                          distance ties; an epsilon band would re-introduce \
                          the hash-perturbation of near-equal distances that \
                          F6 exists to eliminate"
            )]
            // mutant-equivalent (cargo-mutants 303:69 `>` -> `>=`): the
            // `tiebreak >= best_tiebreak` mutant differs from `>` ONLY when
            // a later candidate satisfies `tiebreak == best_tiebreak`. Both
            // `tiebreak` and `best_tiebreak` are 64-bit FNV-1a hashes of
            // distinct `[idx, i, seed]` triples computed in the SAME seeding
            // round (`i` and `seed` fixed, `idx` distinct). Equality there
            // requires an FNV-1a-64 collision on two distinct 24-byte inputs
            // — not constructible without a 2^64 brute-force search, and not
            // reachable by any workflow-trace input. (The `best_tiebreak`
            // initial `0u64` is also unreachable as a real candidate value
            // for the same 1-in-2^64 reason, and idx=0 always wins via the
            // first `d > best_dist` clause before the tiebreak is consulted.)
            // For every constructible input the two operators are observably
            // identical, so no killing test exists. NOTE: only the SECOND
            // `>` here (`tiebreak > best_tiebreak`, col 69) is equivalent
            // under `>=`. The FIRST `>` (`d > best_dist`, col 26) is NOT
            // equivalent — an exact distance tie reorders `d == best_dist`
            // observably; killed by `mutkill_303_*` in tests.rs. The
            // col-69 `>`→`==` and `>`→`<` mutants are likewise killable
            // (the tiebreak path still fires on real ties) and are also
            // killed by `mutkill_303_*`.
            let wins = d > best_dist || (d == best_dist && tiebreak > best_tiebreak);
            if wins {
                best_dist = d;
                best_tiebreak = tiebreak;
                best_idx = idx;
            }
        }
        centroids.push(points[best_idx].clone());
    }
    centroids
}

#[cfg(test)]
mod tests;
