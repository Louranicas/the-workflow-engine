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
    }
    let mut centroids = kmeans_plus_plus_seed(points, config.k, config.seed);
    for _ in 0..config.max_iterations {
        let assignments: Vec<usize> = points
            .iter()
            .map(|p| nearest_centroid(p, &centroids))
            .collect();
        let new_centroids = recompute_centroids(points, &assignments, config.k, dim);
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
    let clustered: Vec<ClusteredPoint> = points
        .iter()
        .zip(final_assignments.iter())
        .map(|(p, &c)| ClusteredPoint {
            coords: p.clone(),
            cluster: c,
        })
        .collect();
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
        let n = counts[i].max(1);
        #[allow(
            clippy::cast_precision_loss,
            reason = "n is bounded by point count"
        )]
        let n_f = n as f64;
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
}
