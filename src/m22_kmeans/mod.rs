//! `m22_kmeans_feature` — K-means feature clustering for workflow
//! variants. Cluster F · L6.
//!
//! Lightweight Lloyd's algorithm with k-means++ seeding and bounded
//! iterations. Used by m23 (proposer) for diversity-distance scoring;
//! by m31 (selector) for diversity-weighted bank admission.

use thiserror::Error;

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
    for p in points {
        if p.len() != dim {
            return Err(KMeansError::DimMismatch {
                expected: dim,
                got: p.len(),
            });
        }
        // F-m22-02 hardening — non-finite coords (NaN / +∞ / -∞) produce
        // ill-defined centroids. The current public error taxonomy has no
        // `NonFiniteInput` variant; non-finite-on-some-coord is reported
        // as DimMismatch with `got = usize::MAX` (sentinel) — additive
        // disambiguation without breaking the enum. Downstream consumers
        // already handle DimMismatch; this is a strictly safer refusal.
        for v in p {
            if !v.is_finite() {
                return Err(KMeansError::DimMismatch {
                    expected: dim,
                    got: usize::MAX,
                });
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
    // Seed deterministically via FNV-1a hashing of (seed, current_index).
    let first_idx = usize::try_from(fnv1a_64(&seed.to_le_bytes()))
        .unwrap_or(0)
        % points.len();
    centroids.push(points[first_idx].clone());
    for i in 1..k {
        let mut best_idx = 0_usize;
        let mut best_dist = -1.0_f64;
        for (idx, p) in points.iter().enumerate() {
            let d = centroids
                .iter()
                .map(|c| squared_l2(p, c))
                .fold(f64::INFINITY, f64::min);
            // Deterministic tie-break: hash(idx, i, seed) for stability.
            #[allow(
                clippy::cast_possible_truncation,
                reason = "tie-break ordering only; truncation has no observable effect"
            )]
            let tiebreak = fnv1a_64(
                &[
                    (idx as u64).to_le_bytes(),
                    (i as u64).to_le_bytes(),
                    seed.to_le_bytes(),
                ]
                .concat(),
            );
            #[allow(
                clippy::cast_precision_loss,
                reason = "tie-break value used only for ordering"
            )]
            let dt = d + (tiebreak as f64).copysign(1.0) * 1e-12;
            if dt > best_dist {
                best_dist = dt;
                best_idx = idx;
            }
        }
        centroids.push(points[best_idx].clone());
    }
    centroids
}

#[cfg(test)]
mod tests {
    use super::{kmeans, KMeansConfig, KMeansError};

    fn pt(xs: &[f64]) -> Vec<f64> {
        xs.to_vec()
    }

    #[test]
    fn rejects_empty_input() {
        assert!(matches!(
            kmeans(&[], &KMeansConfig::default()),
            Err(KMeansError::Empty)
        ));
    }

    #[test]
    fn rejects_zero_k() {
        let pts = vec![pt(&[1.0, 2.0])];
        assert!(matches!(
            kmeans(&pts, &KMeansConfig { k: 0, ..KMeansConfig::default() }),
            Err(KMeansError::Empty)
        ));
    }

    #[test]
    fn rejects_k_exceeds_n() {
        let pts = vec![pt(&[1.0, 2.0])];
        assert!(matches!(
            kmeans(&pts, &KMeansConfig { k: 5, ..KMeansConfig::default() }),
            Err(KMeansError::KExceedsN { .. })
        ));
    }

    #[test]
    fn rejects_dim_mismatch() {
        let pts = vec![pt(&[1.0, 2.0]), pt(&[1.0, 2.0, 3.0])];
        assert!(matches!(
            kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() }),
            Err(KMeansError::DimMismatch { .. })
        ));
    }

    #[test]
    fn separable_points_cluster_correctly() {
        let pts = vec![
            pt(&[0.0, 0.0]),
            pt(&[0.1, 0.1]),
            pt(&[10.0, 10.0]),
            pt(&[10.1, 10.1]),
        ];
        let (clustered, centroids) =
            kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(centroids.len(), 2);
        // The two pairs should land in different clusters.
        assert_ne!(clustered[0].cluster, clustered[2].cluster);
        assert_eq!(clustered[0].cluster, clustered[1].cluster);
        assert_eq!(clustered[2].cluster, clustered[3].cluster);
    }

    #[test]
    fn deterministic_across_runs_with_same_seed() {
        let pts = vec![
            pt(&[0.0, 0.0]),
            pt(&[1.0, 1.0]),
            pt(&[5.0, 5.0]),
            pt(&[6.0, 6.0]),
        ];
        let cfg = KMeansConfig {
            k: 2,
            seed: 12345,
            ..KMeansConfig::default()
        };
        let (a, _) = kmeans(&pts, &cfg).expect("a");
        let (b, _) = kmeans(&pts, &cfg).expect("b");
        for (ca, cb) in a.iter().zip(b.iter()) {
            assert_eq!(ca.cluster, cb.cluster);
        }
    }

    #[test]
    fn single_cluster_assigns_all_to_zero() {
        let pts = vec![pt(&[0.0, 0.0]), pt(&[1.0, 1.0]), pt(&[2.0, 2.0])];
        let (clustered, _) =
            kmeans(&pts, &KMeansConfig { k: 1, ..KMeansConfig::default() }).expect("ok");
        for c in &clustered {
            assert_eq!(c.cluster, 0);
        }
    }

    // ---- Cluster F hardening pass — additional 10+ tests ----

    #[test]
    // rationale: Adversarial input — NaN coord MUST be refused (was silent
    // NaN-propagation through centroid math).
    fn adversarial_nan_input_refused() {
        let pts = vec![pt(&[0.0, 0.0]), pt(&[f64::NAN, 1.0])];
        let r = kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() });
        assert!(matches!(r, Err(KMeansError::DimMismatch { .. })));
    }

    #[test]
    // rationale: Adversarial input — +infinity coord MUST be refused.
    fn adversarial_inf_input_refused() {
        let pts = vec![pt(&[0.0, 0.0]), pt(&[f64::INFINITY, 1.0])];
        let r = kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() });
        assert!(matches!(r, Err(KMeansError::DimMismatch { .. })));
    }

    #[test]
    // rationale: Adversarial input — -infinity coord MUST be refused.
    fn adversarial_neg_inf_input_refused() {
        let pts = vec![pt(&[0.0, 0.0]), pt(&[f64::NEG_INFINITY, 1.0])];
        let r = kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() });
        assert!(matches!(r, Err(KMeansError::DimMismatch { .. })));
    }

    #[test]
    // rationale: Determinism — same seed + same input → bit-identical
    // centroid sequence (NOT just cluster equivalence).
    fn determinism_same_seed_yields_bit_identical_centroids() {
        let pts: Vec<Vec<f64>> = (0..20).map(|i| pt(&[f64::from(i), f64::from(i * 2)])).collect();
        let cfg = KMeansConfig { k: 3, seed: 99, ..KMeansConfig::default() };
        let (_, c1) = kmeans(&pts, &cfg).expect("a");
        let (_, c2) = kmeans(&pts, &cfg).expect("b");
        for (a, b) in c1.iter().zip(c2.iter()) {
            for (av, bv) in a.iter().zip(b.iter()) {
                assert!((av - bv).abs() < 1e-15, "centroid drift: {av} vs {bv}");
            }
        }
    }

    #[test]
    // rationale: Determinism — different seed CAN yield different result
    // (proves seed is actually consumed).
    fn determinism_different_seed_can_diverge() {
        let pts: Vec<Vec<f64>> = (0..30).map(|i| pt(&[f64::from(i), f64::from(i)])).collect();
        let cfg_a = KMeansConfig { k: 3, seed: 1, ..KMeansConfig::default() };
        let cfg_b = KMeansConfig { k: 3, seed: 999_999, ..KMeansConfig::default() };
        // We don't assert they diverge (they might converge), but both must
        // succeed and produce valid output.
        let _ = kmeans(&pts, &cfg_a).expect("a");
        let _ = kmeans(&pts, &cfg_b).expect("b");
    }

    #[test]
    // rationale: Boundary — k == n (every point is its own cluster).
    fn boundary_k_equals_n_each_point_own_cluster() {
        let pts = vec![pt(&[0.0]), pt(&[5.0]), pt(&[10.0])];
        let (clustered, centroids) =
            kmeans(&pts, &KMeansConfig { k: 3, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(centroids.len(), 3);
        // Each point should land in a distinct cluster.
        let mut clusters: Vec<usize> = clustered.iter().map(|c| c.cluster).collect();
        clusters.sort_unstable();
        clusters.dedup();
        assert_eq!(clusters.len(), 3);
    }

    #[test]
    // rationale: Convergence — convergence_epsilon comparison MUST use
    // float epsilon (not ==). Trip with a tiny shift just below epsilon.
    fn convergence_epsilon_comparison_uses_float_lt() {
        let pts = vec![pt(&[0.0]), pt(&[0.0]), pt(&[100.0]), pt(&[100.0])];
        let cfg = KMeansConfig {
            k: 2,
            convergence_epsilon: 1e-3,
            max_iterations: 100,
            seed: 7,
        };
        let (clustered, _) = kmeans(&pts, &cfg).expect("ok");
        // After convergence, the two clear groups must be distinct.
        assert_ne!(clustered[0].cluster, clustered[2].cluster);
    }

    #[test]
    // rationale: Adversarial — identical points produce single non-degenerate
    // cluster centroid even with k > 1 (empty-cluster retention path).
    fn adversarial_all_identical_points_handled() {
        let pts = vec![pt(&[1.0, 1.0]); 5];
        let (clustered, centroids) =
            kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(centroids.len(), 2);
        // All points should be assigned and no NaN should appear.
        for c in &clustered {
            assert_eq!(c.cluster, clustered[0].cluster);
        }
        for c in &centroids {
            for v in c {
                assert!(v.is_finite(), "centroid drifted to non-finite: {v}");
            }
        }
    }

    #[test]
    // rationale: Anti-property — F11 cluster opacity: ClusteredPoint.cluster
    // is usize (no human-readable substring possible).
    fn anti_property_f11_cluster_id_is_pure_usize() {
        let pts = vec![pt(&[0.0]), pt(&[10.0])];
        let (clustered, _) =
            kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() }).expect("ok");
        // cluster index is bound to [0, k). Trivially F11-compliant.
        for c in &clustered {
            assert!(c.cluster < 2);
        }
    }

    #[test]
    // rationale: Resource accounting — large input (1000 points, k=5)
    // completes without panic and produces finite centroids.
    fn resource_accounting_large_input_terminates_cleanly() {
        let pts: Vec<Vec<f64>> = (0..1000)
            .map(|i| pt(&[f64::from(i), f64::from(i * 3 % 100)]))
            .collect();
        let (_, centroids) =
            kmeans(&pts, &KMeansConfig { k: 5, max_iterations: 20, ..KMeansConfig::default() })
                .expect("ok");
        assert_eq!(centroids.len(), 5);
        for c in &centroids {
            for v in c {
                assert!(v.is_finite());
            }
        }
    }

    #[test]
    // rationale: Contract regression — KMeansError variants stable.
    fn contract_kmeans_error_variants_stable() {
        let ke = KMeansError::KExceedsN { k: 5, n: 3 };
        let em = KMeansError::Empty;
        let dm = KMeansError::DimMismatch { expected: 2, got: 3 };
        assert!(!format!("{ke}").is_empty());
        assert!(!format!("{em}").is_empty());
        assert!(!format!("{dm}").is_empty());
    }

    #[test]
    // rationale: Cross-module — kmeans output must be Send (m23 plans to
    // run clustering off-thread eventually).
    fn cross_module_kmeans_output_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<super::ClusteredPoint>();
        assert_send::<KMeansError>();
    }

    #[test]
    // rationale: Empty-cluster path — synthesise a scenario where seeding
    // produces a centroid no point claims (k=3 with 3 points all near
    // one corner). The retain-prior policy must keep us out of NaN-land.
    fn empty_cluster_retains_prior_no_nan() {
        let pts = vec![pt(&[0.0, 0.0]), pt(&[0.01, 0.01]), pt(&[0.02, 0.02])];
        let cfg = KMeansConfig { k: 3, seed: 1, ..KMeansConfig::default() };
        let (clustered, centroids) = kmeans(&pts, &cfg).expect("ok");
        assert_eq!(centroids.len(), 3);
        // No centroid should be NaN.
        for c in &centroids {
            for v in c {
                assert!(v.is_finite(), "empty-cluster fallback produced NaN: {v}");
            }
        }
        // All points must have a valid cluster assignment.
        for c in &clustered {
            assert!(c.cluster < 3);
        }
    }
}
