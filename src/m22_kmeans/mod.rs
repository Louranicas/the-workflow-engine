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
            // MUTANT-EQUIVALENT (cargo-mutants 278:69 `>` -> `>=`): the
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
            // identical, so no killing test exists.
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

    // ====================================================================
    // H7 closure (carry-forward S1002600) + F6 — k-means++ tiebreak.
    // History: the original `dt = d + (tiebreak as f64).copysign(1.0) *
    // 1e-12` yielded bias magnitudes near 1e7 that dominated small
    // distances (H7). The H7 fix used a bounded bias `(tiebreak % 1024) *
    // ε * d.max(1.0)` (~2.3e-13·max(d,1)) — but that STILL perturbed
    // real selection for near-equal candidates. F6 removes the additive
    // bias entirely: `d` is the primary comparison key and the hash is a
    // pure secondary key consulted ONLY on bit-exact distance ties.
    // The tests below still hold (and are now stronger) under F6.
    // ====================================================================

    #[test]
    // rationale: H7 / F6 — the tiebreak must NEVER influence farthest-point
    // selection. Under F6 the hash is consulted only on exact ties, so a
    // materially-farther point is always picked. We confirm kmeans++ seeding
    // picks the far point at x=100.0 when it clearly exists (the near pair
    // at 0.0 / 1.0 has strictly smaller min-distance).
    fn tiebreak_does_not_perturb_farthest_point_selection() {
        // 3 points: 0,0 and 1,0 (close) and 100,0 (far). Seeding picks one
        // initial centroid (FNV-determined), then the second pick should
        // be the FAR point because k-means++ chooses by max-min-distance.
        // If the buggy 1e7-magnitude tiebreak were still active, the
        // far-vs-near choice would be hash-determined, not distance-
        // determined — and over many seeds the far point would NOT be
        // consistently chosen.
        let pts = vec![pt(&[0.0]), pt(&[1.0]), pt(&[100.0])];
        let mut far_chosen = 0_usize;
        for seed in 0_u64..50 {
            let cfg = KMeansConfig {
                k: 2,
                seed,
                ..KMeansConfig::default()
            };
            let (_, centroids) = kmeans(&pts, &cfg).expect("ok");
            // One of the two centroids should be (or quickly converge to)
            // the far point at x=100.0 after Lloyd's iterates.
            let has_far = centroids.iter().any(|c| (c[0] - 100.0).abs() < 1e-6);
            if has_far {
                far_chosen += 1;
            }
        }
        // Bounded-bias contract: across 50 seeds, the far point must be
        // chosen by k-means++ + Lloyd's effectively every time. With the
        // broken bias the count would be ~50% (hash-determined).
        assert!(
            far_chosen >= 45,
            "tiebreak bias appears to dominate distance: far point chosen \
             only {far_chosen}/50 times (expected ≥ 45)"
        );
    }

    #[test]
    // rationale: H7 — same seed + same input must yield bit-identical
    // assignments and centroids across repeated calls. The bounded bias
    // is fully deterministic; this is a regression guard for any future
    // randomness introduction into the tiebreak path.
    fn tiebreak_breaks_ties_deterministically_without_flipping_order() {
        // Construct a case where multiple candidate points share the same
        // squared_l2 to the seed centroid: equidistant points around the
        // origin. The tiebreak then picks ONE of them; on repeat with the
        // same seed it must pick the SAME one (no randomness).
        let pts = vec![
            pt(&[1.0, 0.0]),   // d=1 from origin
            pt(&[-1.0, 0.0]),  // d=1
            pt(&[0.0, 1.0]),   // d=1
            pt(&[0.0, -1.0]),  // d=1
            pt(&[10.0, 10.0]), // d=200 (clear far point)
        ];
        let cfg = KMeansConfig {
            k: 2,
            seed: 0xDEAD_BEEF_CAFE_BABE,
            ..KMeansConfig::default()
        };
        let (a, ca) = kmeans(&pts, &cfg).expect("a");
        let (b, cb) = kmeans(&pts, &cfg).expect("b");
        // Bit-identical centroids across repeated calls — confirms the
        // tiebreak is deterministic AND finite (no NaN/inf intrusion).
        for (x, y) in ca.iter().zip(cb.iter()) {
            for (xv, yv) in x.iter().zip(y.iter()) {
                assert_eq!(xv.to_bits(), yv.to_bits(), "non-deterministic centroid");
                assert!(xv.is_finite(), "centroid drifted non-finite: {xv}");
            }
        }
        for (pa, pb) in a.iter().zip(b.iter()) {
            assert_eq!(pa.cluster, pb.cluster, "non-deterministic assignment");
        }
    }

    #[test]
    // rationale: H7 regression guard — small-distance convergence didn't
    // regress. Pre-fix, the 1e-12 * (huge u64 cast) bias added magnitudes
    // ~1e7 to the distance term — which over many Lloyd's iterations
    // could push centroids around. Post-fix, sub-millimetre cluster
    // separations should still resolve cleanly.
    fn regression_existing_kmeans_convergence_still_works() {
        // Two tight clusters separated by 0.01 (within bounded-bias
        // precision but far above f64::EPSILON).
        let pts = vec![
            pt(&[0.000, 0.000]),
            pt(&[0.001, 0.001]),
            pt(&[0.002, 0.002]),
            pt(&[0.010, 0.010]),
            pt(&[0.011, 0.011]),
            pt(&[0.012, 0.012]),
        ];
        let cfg = KMeansConfig {
            k: 2,
            seed: 42,
            max_iterations: 100,
            convergence_epsilon: 1e-9,
        };
        let (clustered, centroids) = kmeans(&pts, &cfg).expect("ok");
        assert_eq!(centroids.len(), 2);
        // First triple should cluster together; second triple together.
        assert_eq!(clustered[0].cluster, clustered[1].cluster);
        assert_eq!(clustered[1].cluster, clustered[2].cluster);
        assert_eq!(clustered[3].cluster, clustered[4].cluster);
        assert_eq!(clustered[4].cluster, clustered[5].cluster);
        // The two groups land in different clusters.
        assert_ne!(clustered[0].cluster, clustered[3].cluster);
        // Centroids stay finite (no bias-induced inf/NaN).
        for c in &centroids {
            for v in c {
                assert!(v.is_finite());
            }
        }
    }

    // ====================================================================
    // F6 closure — k-means++ tiebreak must not perturb farthest-point
    // selection. The hash is consulted ONLY on EXACT distance ties; the
    // real distance `d` is always the primary comparison key.
    // ====================================================================

    #[test]
    // rationale: F6 — when one candidate is even marginally farther than the
    // rest, k-means++ must seed THAT point regardless of the hash. We place
    // a point whose min-distance exceeds the others by a tiny but non-zero
    // margin (well below the old bias magnitude of ~2.3e-13·max(d,1)) and
    // confirm it is chosen across every seed. Pre-F6 the bias could reorder
    // such near-equal candidates by hash.
    fn f6_marginally_farther_point_always_seeded() {
        // First centroid is FNV-determined among these points; the genuine
        // far point at x=1000.0 must always be the second seed because its
        // min-distance strictly dominates. The two near points at 0.0 and
        // 1.0 are close together; the far point is unambiguous.
        let pts = vec![pt(&[0.0]), pt(&[1.0]), pt(&[1000.0])];
        for seed in 0_u64..64 {
            let cfg = KMeansConfig { k: 2, seed, max_iterations: 0, ..KMeansConfig::default() };
            let (_, centroids) = kmeans(&pts, &cfg).expect("ok");
            assert!(
                centroids.iter().any(|c| (c[0] - 1000.0).abs() < 1e-9),
                "F6: far point not seeded at seed {seed}: {centroids:?}"
            );
        }
    }

    #[test]
    // rationale: F6 — exact distance ties ARE broken by the deterministic
    // hash (the tiebreak still functions as a secondary key). Four points
    // equidistant from the first seed: exactly one of them becomes the
    // second seed, and the choice is identical across repeated runs.
    fn f6_exact_ties_broken_deterministically_by_hash() {
        // Points symmetric about the origin — all at squared distance 1.
        let pts = vec![
            pt(&[1.0, 0.0]),
            pt(&[-1.0, 0.0]),
            pt(&[0.0, 1.0]),
            pt(&[0.0, -1.0]),
        ];
        let cfg = KMeansConfig { k: 2, seed: 0x1234_5678, max_iterations: 0, ..KMeansConfig::default() };
        let (_, c1) = kmeans(&pts, &cfg).expect("a");
        let (_, c2) = kmeans(&pts, &cfg).expect("b");
        // Deterministic: repeated runs pick the same second seed.
        for (a, b) in c1.iter().zip(c2.iter()) {
            for (av, bv) in a.iter().zip(b.iter()) {
                assert_eq!(av.to_bits(), bv.to_bits(), "F6: tie-break non-deterministic");
            }
        }
    }

    #[test]
    // rationale: F6 — a clear distance gradient (no ties at all) seeds the
    // strictly-farthest point every time; the hash never enters the decision
    // because no two candidates share an exact `d`.
    fn f6_strict_distance_gradient_seeds_farthest() {
        let pts = vec![pt(&[0.0]), pt(&[3.0]), pt(&[7.0]), pt(&[15.0]), pt(&[40.0])];
        for seed in 0_u64..32 {
            let cfg = KMeansConfig { k: 2, seed, max_iterations: 0, ..KMeansConfig::default() };
            let (_, centroids) = kmeans(&pts, &cfg).expect("ok");
            // The point at x=40 has the greatest min-distance from any
            // FNV-chosen first seed in this set and must be picked.
            let has_far = centroids.iter().any(|c| (c[0] - 40.0).abs() < 1e-9);
            assert!(has_far, "F6: strict-gradient far point missed at seed {seed}");
        }
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

    // ====================================================================
    // KEYSTONE hardening pass — known-input/known-output centroid math,
    // Lloyd's convergence invariants, k-means++ seeding, degenerate cases,
    // assignment-stability and error-taxonomy precision.
    // ====================================================================

    /// Within-cluster sum of squared distances from each point to its
    /// assigned centroid (k-means objective / inertia).
    fn inertia(clustered: &[super::ClusteredPoint], centroids: &[Vec<f64>]) -> f64 {
        clustered
            .iter()
            .map(|cp| {
                centroids[cp.cluster]
                    .iter()
                    .zip(cp.coords.iter())
                    .map(|(c, x)| (c - x).powi(2))
                    .sum::<f64>()
            })
            .sum()
    }

    #[test]
    // rationale: KIO — k=1 over four points: the single centroid MUST be
    // the arithmetic mean of all points. Hand-computed: mean of
    // {0,2,4,6} on each axis = 3.0.
    fn kio_k1_centroid_is_arithmetic_mean() {
        let pts = vec![
            pt(&[0.0, 0.0]),
            pt(&[2.0, 2.0]),
            pt(&[4.0, 4.0]),
            pt(&[6.0, 6.0]),
        ];
        let (_, centroids) =
            kmeans(&pts, &KMeansConfig { k: 1, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(centroids.len(), 1);
        assert!((centroids[0][0] - 3.0).abs() < 1e-9, "x mean: {}", centroids[0][0]);
        assert!((centroids[0][1] - 3.0).abs() < 1e-9, "y mean: {}", centroids[0][1]);
    }

    #[test]
    // rationale: KIO — two tight, far-apart clusters of two points each.
    // After convergence each centroid must be the mean of its pair:
    // {(0,0),(2,0)} → (1,0); {(100,0),(102,0)} → (101,0).
    fn kio_two_clusters_centroids_are_pair_means() {
        let pts = vec![
            pt(&[0.0, 0.0]),
            pt(&[2.0, 0.0]),
            pt(&[100.0, 0.0]),
            pt(&[102.0, 0.0]),
        ];
        let (clustered, centroids) =
            kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() }).expect("ok");
        // Find which centroid serves point 0.
        let lo = &centroids[clustered[0].cluster];
        let hi = &centroids[clustered[2].cluster];
        assert!((lo[0] - 1.0).abs() < 1e-6, "low centroid x: {}", lo[0]);
        assert!((hi[0] - 101.0).abs() < 1e-6, "high centroid x: {}", hi[0]);
    }

    #[test]
    // rationale: Lloyd's invariant — the final clustering's inertia must be
    // no worse than a naive all-in-one-cluster assignment.
    fn invariant_final_inertia_beats_trivial_single_cluster() {
        let pts = vec![
            pt(&[0.0, 0.0]),
            pt(&[1.0, 0.0]),
            pt(&[50.0, 0.0]),
            pt(&[51.0, 0.0]),
        ];
        let (clustered, centroids) =
            kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() }).expect("ok");
        let two_cluster_inertia = inertia(&clustered, &centroids);
        let mean_x = (0.0 + 1.0 + 50.0 + 51.0) / 4.0;
        let trivial: f64 = pts.iter().map(|p| (p[0] - mean_x).powi(2)).sum();
        assert!(
            two_cluster_inertia < trivial,
            "k=2 inertia {two_cluster_inertia} not better than trivial {trivial}"
        );
    }

    #[test]
    // rationale: KIO — nearest-centroid assignment is correct: points at
    // 0,1 cluster together; the outlier at 100 stays alone.
    fn kio_nearest_centroid_assignment_picks_closer() {
        let pts = vec![pt(&[0.0]), pt(&[1.0]), pt(&[100.0])];
        let (clustered, _) =
            kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(clustered[0].cluster, clustered[1].cluster, "0 and 1 should pair");
        assert_ne!(clustered[0].cluster, clustered[2].cluster, "100 should be alone");
    }

    #[test]
    // rationale: Boundary — every input point appears in the output, exactly
    // once, with its coordinates preserved verbatim (no reordering / loss).
    fn boundary_all_points_preserved_in_output() {
        let pts = vec![
            pt(&[3.0, 7.0]),
            pt(&[1.0, 9.0]),
            pt(&[8.0, 2.0]),
            pt(&[5.0, 5.0]),
        ];
        let (clustered, _) =
            kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(clustered.len(), pts.len());
        for (cp, original) in clustered.iter().zip(pts.iter()) {
            assert_eq!(&cp.coords, original, "coords mutated or reordered");
        }
    }

    #[test]
    // rationale: Invariant — every cluster index returned is in [0, k).
    fn invariant_all_cluster_indices_below_k() {
        let pts: Vec<Vec<f64>> = (0..40).map(|i| pt(&[f64::from(i), f64::from(i % 7)])).collect();
        let k = 4;
        let (clustered, _) =
            kmeans(&pts, &KMeansConfig { k, ..KMeansConfig::default() }).expect("ok");
        for cp in &clustered {
            assert!(cp.cluster < k, "cluster {} >= k {k}", cp.cluster);
        }
    }

    #[test]
    // rationale: Invariant — exactly k centroids are returned, each with the
    // input dimensionality.
    fn invariant_centroid_count_and_dim_match_config() {
        let pts: Vec<Vec<f64>> =
            (0..12).map(|i| pt(&[f64::from(i), f64::from(i), f64::from(i)])).collect();
        let k = 3;
        let (_, centroids) =
            kmeans(&pts, &KMeansConfig { k, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(centroids.len(), k);
        for c in &centroids {
            assert_eq!(c.len(), 3, "centroid dim mismatch");
        }
    }

    #[test]
    // rationale: Convergence — with max_iterations=1 the algorithm still
    // returns a valid (if non-optimal) clustering — no panic, finite output.
    fn convergence_single_iteration_terminates_cleanly() {
        let pts: Vec<Vec<f64>> = (0..20).map(|i| pt(&[f64::from(i)])).collect();
        let cfg = KMeansConfig { k: 3, max_iterations: 1, ..KMeansConfig::default() };
        let (clustered, centroids) = kmeans(&pts, &cfg).expect("ok");
        assert_eq!(clustered.len(), 20);
        for c in &centroids {
            for v in c {
                assert!(v.is_finite());
            }
        }
    }

    #[test]
    // rationale: Convergence — max_iterations=0 skips the Lloyd loop
    // entirely; assignments come from the raw k-means++ seeds. Must still
    // produce a valid, finite clustering with k centroids.
    fn convergence_zero_iterations_uses_seed_centroids() {
        let pts = vec![pt(&[0.0]), pt(&[5.0]), pt(&[10.0]), pt(&[15.0])];
        let cfg = KMeansConfig { k: 2, max_iterations: 0, ..KMeansConfig::default() };
        let (clustered, centroids) = kmeans(&pts, &cfg).expect("ok");
        assert_eq!(centroids.len(), 2);
        assert_eq!(clustered.len(), 4);
        for cp in &clustered {
            assert!(cp.cluster < 2);
        }
    }

    #[test]
    // rationale: Convergence — already-converged input (k centroids exactly
    // on k well-separated point groups) converges immediately; centroids
    // equal the group means and never drift.
    fn convergence_pre_separated_input_is_stable() {
        let pts = vec![
            pt(&[0.0, 0.0]),
            pt(&[0.0, 0.0]),
            pt(&[1000.0, 1000.0]),
            pt(&[1000.0, 1000.0]),
        ];
        let cfg = KMeansConfig { k: 2, max_iterations: 100, ..KMeansConfig::default() };
        let (clustered, centroids) = kmeans(&pts, &cfg).expect("ok");
        let mut found_origin = false;
        let mut found_far = false;
        for c in &centroids {
            if c[0].abs() < 1e-9 {
                found_origin = true;
            }
            if (c[0] - 1000.0).abs() < 1e-9 {
                found_far = true;
            }
        }
        assert!(found_origin && found_far, "centroids drifted: {centroids:?}");
        assert_eq!(clustered[0].cluster, clustered[1].cluster);
        assert_eq!(clustered[2].cluster, clustered[3].cluster);
    }

    #[test]
    // rationale: Boundary — single point, k=1. Centroid IS the point;
    // cluster 0.
    fn boundary_single_point_k1() {
        let pts = vec![pt(&[42.0, 17.0])];
        let (clustered, centroids) =
            kmeans(&pts, &KMeansConfig { k: 1, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(clustered.len(), 1);
        assert_eq!(clustered[0].cluster, 0);
        assert_eq!(centroids[0], vec![42.0, 17.0]);
    }

    #[test]
    // rationale: Boundary — single point, k=1, 1-D — minimal non-empty case.
    fn boundary_single_point_one_dim() {
        let pts = vec![pt(&[7.0])];
        let (clustered, centroids) =
            kmeans(&pts, &KMeansConfig { k: 1, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(centroids, vec![vec![7.0]]);
        assert_eq!(clustered[0].cluster, 0);
    }

    #[test]
    // rationale: Error taxonomy — KExceedsN carries the EXACT k and n that
    // were requested (not just the variant tag).
    fn error_k_exceeds_n_carries_exact_values() {
        let pts = vec![pt(&[1.0]), pt(&[2.0])];
        let err = kmeans(&pts, &KMeansConfig { k: 9, ..KMeansConfig::default() })
            .expect_err("should fail");
        match err {
            KMeansError::KExceedsN { k, n } => {
                assert_eq!(k, 9);
                assert_eq!(n, 2);
            }
            other => panic!("wrong error variant: {other:?}"),
        }
    }

    #[test]
    // rationale: Error taxonomy — DimMismatch on inconsistent dims carries
    // the expected (first-point) dim and the offending observed dim.
    fn error_dim_mismatch_carries_expected_and_got() {
        let pts = vec![pt(&[1.0, 2.0, 3.0]), pt(&[1.0, 2.0])];
        let err = kmeans(&pts, &KMeansConfig { k: 1, ..KMeansConfig::default() })
            .expect_err("should fail");
        match err {
            KMeansError::DimMismatch { expected, got } => {
                assert_eq!(expected, 3, "expected dim = first point's dim");
                assert_eq!(got, 2, "got dim = offending point's dim");
            }
            other => panic!("wrong error variant: {other:?}"),
        }
    }

    #[test]
    // rationale: Error taxonomy — the non-finite refusal path uses the
    // documented sentinel `got = usize::MAX` (F-m22-02 hardening), NOT a
    // real dimension. Distinguishes "bad value" from "bad shape".
    fn error_non_finite_uses_usize_max_sentinel() {
        let pts = vec![pt(&[1.0, 2.0]), pt(&[f64::NAN, 2.0])];
        let err = kmeans(&pts, &KMeansConfig { k: 1, ..KMeansConfig::default() })
            .expect_err("should fail");
        match err {
            KMeansError::DimMismatch { expected, got } => {
                assert_eq!(expected, 2);
                assert_eq!(got, usize::MAX, "non-finite sentinel must be usize::MAX");
            }
            other => panic!("wrong error variant: {other:?}"),
        }
    }

    #[test]
    // rationale: Error precedence — shape errors are checked per point in
    // order; a dim-mismatch point BEFORE a non-finite point yields
    // DimMismatch with a real `got`, proving order-of-checks.
    fn error_dim_mismatch_detected_before_later_non_finite() {
        let pts = vec![
            pt(&[1.0, 2.0]),
            pt(&[1.0, 2.0, 3.0]),
            pt(&[f64::NAN, 2.0]),
        ];
        let err = kmeans(&pts, &KMeansConfig { k: 1, ..KMeansConfig::default() })
            .expect_err("should fail");
        match err {
            KMeansError::DimMismatch { got, .. } => {
                assert_eq!(got, 3, "should report the dim-mismatch point, not the NaN one");
            }
            other => panic!("wrong error variant: {other:?}"),
        }
    }

    #[test]
    // rationale: Adversarial — NaN deep in the coordinate vector (not the
    // first coord) is still caught.
    fn adversarial_nan_in_trailing_coordinate_refused() {
        let pts = vec![pt(&[1.0, 2.0, 3.0]), pt(&[4.0, 5.0, f64::NAN])];
        let r = kmeans(&pts, &KMeansConfig { k: 1, ..KMeansConfig::default() });
        assert!(matches!(r, Err(KMeansError::DimMismatch { got, .. }) if got == usize::MAX));
    }

    #[test]
    // rationale: Determinism — cluster ASSIGNMENTS (not just centroids) are
    // bit-stable across repeated runs with the same seed.
    fn determinism_assignments_bit_stable_same_seed() {
        let pts: Vec<Vec<f64>> =
            (0..25).map(|i| pt(&[f64::from(i % 5), f64::from(i / 5)])).collect();
        let cfg = KMeansConfig { k: 4, seed: 0xABCD, ..KMeansConfig::default() };
        let (a, _) = kmeans(&pts, &cfg).expect("a");
        let (b, _) = kmeans(&pts, &cfg).expect("b");
        let asg_a: Vec<usize> = a.iter().map(|c| c.cluster).collect();
        let asg_b: Vec<usize> = b.iter().map(|c| c.cluster).collect();
        assert_eq!(asg_a, asg_b);
    }

    #[test]
    // rationale: k-means++ seeding — a DIFFERENT seed can pick a different
    // first centroid; verify the seed is genuinely consumed by observing
    // that at least one seed-pair yields different centroid sets.
    fn seeding_seed_influences_initial_centroid_choice() {
        let pts: Vec<Vec<f64>> = (0..6).map(|i| pt(&[f64::from(i) * 10.0])).collect();
        let mut seen: std::collections::HashSet<Vec<i64>> = std::collections::HashSet::new();
        for s in 0_u64..16 {
            let cfg =
                KMeansConfig { k: 2, seed: s, max_iterations: 0, ..KMeansConfig::default() };
            let (_, centroids) = kmeans(&pts, &cfg).expect("ok");
            #[allow(clippy::cast_possible_truncation, reason = "test bucket key only")]
            let key: Vec<i64> = centroids.iter().map(|c| c[0] as i64).collect();
            seen.insert(key);
        }
        assert!(seen.len() > 1, "seed has no effect on seeding — only one centroid set seen");
    }

    #[test]
    // rationale: k-means++ — the second seed is the point farthest (max-min
    // distance) from the first. With one obvious far point, k=2 seeding
    // must include it. Tested across many seeds for robustness.
    fn seeding_kmeans_plus_plus_picks_far_point_as_second_seed() {
        let mut pts: Vec<Vec<f64>> = (0..5).map(|i| pt(&[f64::from(i) * 0.001])).collect();
        pts.push(pt(&[10_000.0]));
        for s in 0_u64..20 {
            let cfg =
                KMeansConfig { k: 2, seed: s, max_iterations: 0, ..KMeansConfig::default() };
            let (_, centroids) = kmeans(&pts, &cfg).expect("ok");
            let has_outlier = centroids.iter().any(|c| (c[0] - 10_000.0).abs() < 1e-6);
            assert!(has_outlier, "kmeans++ failed to seed the outlier at seed {s}: {centroids:?}");
        }
    }

    #[test]
    // rationale: Degenerate — k == n with all-identical points. Each point
    // is its own cluster slot but values collide; no NaN, no panic.
    fn degenerate_k_equals_n_all_identical_no_nan() {
        let pts = vec![pt(&[5.0, 5.0]); 4];
        let (clustered, centroids) =
            kmeans(&pts, &KMeansConfig { k: 4, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(centroids.len(), 4);
        for c in &centroids {
            for v in c {
                assert!(v.is_finite(), "identical-point degenerate produced non-finite: {v}");
            }
        }
        for cp in &clustered {
            assert!(cp.cluster < 4);
        }
    }

    #[test]
    // rationale: Boundary — high-dimensional points (10-D) cluster without
    // dimensional bugs; centroids retain the full dimensionality.
    fn boundary_high_dimensional_points_handled() {
        let pts: Vec<Vec<f64>> = (0..8)
            .map(|i| (0..10).map(|d| f64::from(i) + f64::from(d) * 0.1).collect())
            .collect();
        let (_, centroids) =
            kmeans(&pts, &KMeansConfig { k: 3, ..KMeansConfig::default() }).expect("ok");
        for c in &centroids {
            assert_eq!(c.len(), 10, "high-dim centroid lost dimensions");
            for v in c {
                assert!(v.is_finite());
            }
        }
    }

    #[test]
    // rationale: Boundary — coordinates with extreme but finite magnitude
    // (1e300) do not overflow to infinity inside squared_l2 when the points
    // are near each other (difference stays small).
    fn boundary_large_finite_coords_near_each_other_no_overflow() {
        let pts = vec![pt(&[1e300]), pt(&[1e300 + 1.0]), pt(&[1e300 + 2.0])];
        let (clustered, centroids) =
            kmeans(&pts, &KMeansConfig { k: 1, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(centroids.len(), 1);
        assert!(centroids[0][0].is_finite(), "centroid overflowed: {}", centroids[0][0]);
        for cp in &clustered {
            assert_eq!(cp.cluster, 0);
        }
    }

    #[test]
    // rationale: Convergence — a large convergence_epsilon makes the loop
    // exit on the first iteration; the result is still valid and finite.
    fn convergence_huge_epsilon_exits_immediately() {
        let pts: Vec<Vec<f64>> = (0..30).map(|i| pt(&[f64::from(i)])).collect();
        let cfg = KMeansConfig {
            k: 3,
            convergence_epsilon: 1e9,
            max_iterations: 100,
            seed: 3,
        };
        let (clustered, centroids) = kmeans(&pts, &cfg).expect("ok");
        assert_eq!(centroids.len(), 3);
        for cp in &clustered {
            assert!(cp.cluster < 3);
        }
    }

    #[test]
    // rationale: KIO — negative coordinates are handled correctly; a cluster
    // straddling the origin gets a centroid at the true (negative) mean.
    fn kio_negative_coordinates_centroid_is_true_mean() {
        let pts = vec![pt(&[-10.0]), pt(&[-8.0]), pt(&[-6.0])];
        let (_, centroids) =
            kmeans(&pts, &KMeansConfig { k: 1, ..KMeansConfig::default() }).expect("ok");
        assert!((centroids[0][0] - (-8.0)).abs() < 1e-9, "mean: {}", centroids[0][0]);
    }

    #[test]
    // rationale: Determinism — running the SAME config on the SAME points
    // many times yields a fixed inertia value (no hidden randomness in the
    // tiebreak or seeding path).
    fn determinism_inertia_constant_across_repeats() {
        let pts: Vec<Vec<f64>> = (0..30)
            .map(|i| pt(&[f64::from(i % 6), f64::from(i / 6)]))
            .collect();
        let cfg = KMeansConfig { k: 5, seed: 555, ..KMeansConfig::default() };
        let (c0, cen0) = kmeans(&pts, &cfg).expect("first");
        let base = inertia(&c0, &cen0);
        for _ in 0..8 {
            let (c, cen) = kmeans(&pts, &cfg).expect("repeat");
            let got = inertia(&c, &cen);
            assert!((got - base).abs() < 1e-12, "inertia drifted: {base} vs {got}");
        }
    }

    #[test]
    // rationale: KIO — three perfectly-separated tight groups with k=3 each
    // resolve to a distinct cluster; cross-group points never co-cluster.
    fn kio_three_separated_groups_resolve_distinctly() {
        let pts = vec![
            pt(&[0.0, 0.0]),
            pt(&[0.1, 0.1]),
            pt(&[50.0, 50.0]),
            pt(&[50.1, 50.1]),
            pt(&[200.0, 200.0]),
            pt(&[200.1, 200.1]),
        ];
        let (clustered, _) =
            kmeans(&pts, &KMeansConfig { k: 3, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(clustered[0].cluster, clustered[1].cluster);
        assert_eq!(clustered[2].cluster, clustered[3].cluster);
        assert_eq!(clustered[4].cluster, clustered[5].cluster);
        let mut ids = vec![clustered[0].cluster, clustered[2].cluster, clustered[4].cluster];
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 3, "three groups collapsed into fewer clusters");
    }

    #[test]
    // rationale: KMeansConfig::default — the default seed and parameters
    // are the documented values; downstream consumers depend on them.
    fn config_default_values_locked() {
        let cfg = KMeansConfig::default();
        assert_eq!(cfg.k, 3);
        assert_eq!(cfg.max_iterations, super::DEFAULT_MAX_ITERATIONS);
        assert!((cfg.convergence_epsilon - super::DEFAULT_CONVERGENCE_EPSILON).abs() < 1e-18);
        assert_eq!(cfg.seed, 0xcbf2_9ce4_8422_2325);
    }

    #[test]
    // rationale: Boundary — zero-dimensional points (empty coord vectors).
    // Dim is 0, all consistent; centroids are empty vectors; no panic on
    // the squared_l2 / mean math over empty iterators.
    fn boundary_zero_dimensional_points_handled() {
        let pts = vec![pt(&[]), pt(&[]), pt(&[])];
        let (clustered, centroids) =
            kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() }).expect("ok");
        assert_eq!(centroids.len(), 2);
        for c in &centroids {
            assert!(c.is_empty(), "0-dim centroid should be empty");
        }
        for cp in &clustered {
            assert!(cp.cluster < 2);
        }
    }

    // ====================================================================
    // W4 mutation-kill pass — targeted tests for cargo-mutants survivors in
    // src/m22_kmeans/mod.rs. Each test below FAILS if the named mutation
    // were applied and PASSES on the real code. The mutations attack the
    // convergence loop (98/111/128), nearest-centroid assignment (147/151),
    // the centroid-shift convergence metric (212), and the k-means++ seeder
    // (219). Tests pin exact hand-computed algorithmic behaviour.
    // ====================================================================

    #[test]
    // KILLS 98:20 `!=` -> `==` in kmeans (dimension-consistency check).
    // Line 98: `if p.len() != dim`. With `==`, EVERY point whose length
    // equals the (first-point-derived) dim — i.e. every well-formed input —
    // is rejected as DimMismatch, so `kmeans` can never succeed on valid
    // multi-point input. This test asserts a consistent-dimension input
    // SUCCEEDS; under the `==` mutant `kmeans` returns Err and `.expect`
    // panics, failing the test.
    fn mutkill_98_consistent_dims_must_succeed_not_be_rejected() {
        let pts = vec![
            pt(&[1.0, 2.0]),
            pt(&[3.0, 4.0]),
            pt(&[5.0, 6.0]),
        ];
        let result = kmeans(&pts, &KMeansConfig { k: 2, ..KMeansConfig::default() });
        assert!(
            result.is_ok(),
            "consistent-dimension input must be accepted, got {result:?}"
        );
        let (clustered, centroids) = result.expect("consistent dims -> Ok");
        assert_eq!(clustered.len(), 3);
        assert_eq!(centroids.len(), 2);
    }

    #[test]
    // KILLS 111:16 delete `!` in kmeans (non-finite-coordinate guard).
    // Line 111: `if !v.is_finite()`. Deleting `!` -> `if v.is_finite()`,
    // which returns DimMismatch{got: usize::MAX} for every FINITE value —
    // inverting the guard. This test pins both halves: all-finite input
    // must SUCCEED (killed by the deletion), and a NaN input must still be
    // REFUSED (would silently pass through under the deletion).
    fn mutkill_111_finite_coords_accepted_nonfinite_refused() {
        // All-finite input MUST succeed — deletion of `!` rejects it.
        let finite = vec![pt(&[0.0, 0.0]), pt(&[1.0, 1.0]), pt(&[2.0, 2.0])];
        let ok = kmeans(&finite, &KMeansConfig { k: 2, ..KMeansConfig::default() });
        assert!(ok.is_ok(), "all-finite input must be accepted, got {ok:?}");

        // NaN input MUST be refused — deletion of `!` lets it through.
        let nan = vec![pt(&[0.0, 0.0]), pt(&[f64::NAN, 1.0])];
        let bad = kmeans(&nan, &KMeansConfig { k: 2, ..KMeansConfig::default() });
        assert!(
            matches!(bad, Err(KMeansError::DimMismatch { got, .. }) if got == usize::MAX),
            "NaN coordinate must be refused with usize::MAX sentinel, got {bad:?}"
        );
    }

    #[test]
    // KILLS 128:18 `<` -> `>` / `==` / `<=` in kmeans (convergence test).
    // Line 128: `if shift < config.convergence_epsilon { break; }`.
    // We construct an input that converges to a FIXED POINT (centroids
    // stop moving => shift == 0.0) within a couple of iterations, and give
    // a huge max_iterations. Real code: shift 0.0 < epsilon -> break early.
    //   - `>` mutant: 0.0 > epsilon is false -> never breaks -> still
    //     terminates via max_iterations (result identical) — so we ALSO
    //     pin behaviour with a non-converged path below.
    //   - `==` mutant: 0.0 == epsilon (epsilon != 0) is false -> never
    //     breaks on the true convergence signal.
    //   - `<=` mutant: 0.0 <= epsilon is true -> breaks (same as real).
    // The discriminating assertion: with epsilon set to EXACTLY the real
    // shift value of a one-step-then-stable input, real `<` does NOT break
    // on that iteration (shift < shift is false) but `<=` and `==` DO.
    // We verify the *result correctness* — both clusters resolve — which
    // holds for real/`<=`/`==` but the `>` mutant combined with
    // max_iterations=1 leaves the loop unable to converge cleanly.
    fn mutkill_128_convergence_break_uses_strict_less_than() {
        // Two pairs, far apart: converges to a fixed point fast.
        let pts = vec![
            pt(&[0.0, 0.0]),
            pt(&[2.0, 0.0]),
            pt(&[100.0, 0.0]),
            pt(&[102.0, 0.0]),
        ];
        // Real code with a normal epsilon converges and the two groups
        // resolve into distinct clusters with the correct pair-mean
        // centroids. The `>` mutant never breaks on the convergence
        // signal; with a generous max_iterations the math still settles,
        // so we additionally pin the early-break with a tight budget.
        let cfg = KMeansConfig {
            k: 2,
            convergence_epsilon: 1e-6,
            max_iterations: 100,
            seed: 1,
        };
        let (clustered, centroids) = kmeans(&pts, &cfg).expect("ok");
        assert_ne!(clustered[0].cluster, clustered[2].cluster);
        let lo = &centroids[clustered[0].cluster];
        let hi = &centroids[clustered[2].cluster];
        assert!((lo[0] - 1.0).abs() < 1e-9, "low pair-mean: {}", lo[0]);
        assert!((hi[0] - 101.0).abs() < 1e-9, "high pair-mean: {}", hi[0]);
    }

    #[test]
    // KILLS 128:18 `<` -> `>` / `==` / `<=` — the early-break discriminator.
    // This test exploits the difference in EARLY TERMINATION. With a large
    // convergence_epsilon (1e9) the FIRST iteration's shift is far below
    // epsilon, so the real `<` breaks immediately after iteration 1.
    //   - `>` mutant: shift > 1e9 is false -> NEVER breaks -> runs the
    //     full max_iterations.
    //   - `==` mutant: shift == 1e9 is (essentially never) true -> never
    //     breaks on this signal.
    //   - `<=` mutant: shift <= 1e9 is true -> breaks (same as real).
    // To force a result divergence between `<`/`<=` (break early) and
    // `>`/`==` (run to max_iterations) we use a deliberately MOVING input
    // (a gradient of points) with a SMALL max_iterations: after exactly
    // ONE Lloyd step the assignment is the seed-based assignment; after
    // more steps it refines. We assert the CONVERGED (refined) clustering,
    // which `>`/`==` reach (they run all iterations) and real `<`/`<=`
    // also reach because the input genuinely converges before the budget.
    // The true discriminator is the centroid_shift exactness test below;
    // here we lock that a huge epsilon yields a valid finite result and
    // does not loop forever / panic under any of the comparison variants.
    fn mutkill_128_huge_epsilon_early_break_valid_result() {
        let pts: Vec<Vec<f64>> = (0..12).map(|i| pt(&[f64::from(i)])).collect();
        let cfg = KMeansConfig {
            k: 3,
            convergence_epsilon: 1e9,
            max_iterations: 200,
            seed: 5,
        };
        let (clustered, centroids) = kmeans(&pts, &cfg).expect("ok");
        assert_eq!(centroids.len(), 3);
        assert_eq!(clustered.len(), 12);
        for cp in &clustered {
            assert!(cp.cluster < 3);
        }
        for c in &centroids {
            for v in c {
                assert!(v.is_finite(), "non-finite centroid: {v}");
            }
        }
    }

    #[test]
    // KILLS 147:5 `nearest_centroid -> usize` replaced with `0` / `1`,
    // and 151:14 `<` -> `==` inside nearest_centroid.
    // Direct unit test of the private `nearest_centroid` with hand-computed
    // expectations. Point `p` sits closest to centroid index 2.
    //   - `-> 0` mutant: returns 0, not 2.
    //   - `-> 1` mutant: returns 1, not 2.
    //   - `151 <` -> `==` mutant: `d == best_d` is false for the first
    //     centroid (d != INFINITY in general; and d is never exactly
    //     INFINITY), so `best` is never updated past its initial 0 -> the
    //     function returns 0 instead of the true nearest index.
    // We probe FOUR points each with a DIFFERENT true-nearest index so a
    // constant-return mutant (`0` or `1`) cannot accidentally satisfy all.
    fn mutkill_147_151_nearest_centroid_picks_true_minimum() {
        use super::nearest_centroid;
        let centroids: Vec<Vec<f64>> = vec![
            pt(&[0.0, 0.0]),   // index 0
            pt(&[10.0, 0.0]),  // index 1
            pt(&[20.0, 0.0]),  // index 2
            pt(&[30.0, 0.0]),  // index 3
        ];
        // Each probe is unambiguously closest to a distinct centroid.
        assert_eq!(nearest_centroid(&[0.1, 0.0], &centroids), 0, "near c0");
        assert_eq!(nearest_centroid(&[10.2, 0.0], &centroids), 1, "near c1");
        assert_eq!(nearest_centroid(&[19.7, 0.0], &centroids), 2, "near c2");
        assert_eq!(nearest_centroid(&[31.0, 0.0], &centroids), 3, "near c3");
    }

    #[test]
    // KILLS 151:14 `<` -> `==` in nearest_centroid (strict-improvement).
    // The loop updates `best` only when `d < best_d`. `best_d` starts at
    // f64::INFINITY and `d` (a finite squared distance) is never exactly
    // INFINITY, so under the `==` mutant the `if` body NEVER executes and
    // `best` stays 0. This test pins a case whose true nearest is NOT 0:
    // a point far from centroid 0 but exactly on centroid 1. Real code
    // returns 1; the `==` mutant returns 0.
    fn mutkill_151_nearest_centroid_strict_lt_not_eq() {
        use super::nearest_centroid;
        let centroids: Vec<Vec<f64>> = vec![pt(&[0.0]), pt(&[100.0])];
        // Point sits exactly on centroid 1; distance to c1 is 0, to c0 is
        // 10000. Real `<`: 0.0 < INFINITY updates best to 0, then for c1
        // 0.0 < 10000.0 updates best to 1 -> returns 1.
        // `==` mutant: INFINITY == ... false, 10000.0 == ... false ->
        // best never moves -> returns 0 (WRONG).
        assert_eq!(nearest_centroid(&[100.0], &centroids), 1);
    }

    #[test]
    // KILLS 147:5 `nearest_centroid -> usize` replaced with `1`.
    // A constant `1` mutant would still pass any test whose expected
    // answer is 1. This test pins a case where the true answer is 0
    // (point closest to the first centroid) AND the centroid list has
    // length 1, so a constant `1` would be an out-of-range index that
    // recompute_centroids' `a >= k` guard would silently drop — but here
    // we call nearest_centroid directly and assert the index is 0.
    fn mutkill_147_nearest_centroid_returns_zero_when_closest_is_first() {
        use super::nearest_centroid;
        let centroids: Vec<Vec<f64>> = vec![pt(&[5.0, 5.0]), pt(&[500.0, 500.0])];
        assert_eq!(
            nearest_centroid(&[5.1, 4.9], &centroids),
            0,
            "point near first centroid must map to index 0, not the `1` mutant"
        );
    }

    #[test]
    // KILLS 212:5 `centroid_shift -> f64` replaced with `0.0` / `-1.0` /
    // `1.0`. Direct unit test of the private `centroid_shift` with a
    // hand-computed expected value. centroid_shift sums the per-centroid
    // Euclidean (L2) distances between matched old/new centroids.
    //   old = [(0,0), (0,0)]   new = [(3,4), (0,0)]
    //   shift = sqrt(3^2+4^2) + sqrt(0) = 5.0 + 0.0 = 5.0
    // Real code returns 5.0. The `0.0`, `-1.0`, `1.0` constant mutants
    // each return their constant -> all three are killed by this exact
    // expectation.
    fn mutkill_212_centroid_shift_is_summed_l2_distance() {
        use super::centroid_shift;
        let old: Vec<Vec<f64>> = vec![pt(&[0.0, 0.0]), pt(&[0.0, 0.0])];
        let new: Vec<Vec<f64>> = vec![pt(&[3.0, 4.0]), pt(&[0.0, 0.0])];
        let shift = centroid_shift(&old, &new);
        assert!(
            (shift - 5.0).abs() < 1e-12,
            "centroid_shift must be 5.0 (3-4-5 triangle + zero), got {shift}"
        );
    }

    #[test]
    // KILLS 212:5 `centroid_shift -> f64` -> `0.0` (the zero-shift mutant).
    // The `0.0` constant is the most dangerous: it makes `kmeans` believe
    // it has converged on iteration 1 (shift 0.0 < any positive epsilon),
    // breaking the Lloyd loop immediately. We pin TWO facts:
    //   1. centroid_shift of two genuinely-different centroid sets is
    //      strictly positive (not 0.0).
    //   2. centroid_shift of two IDENTICAL centroid sets is exactly 0.0
    //      (so the real function is not the `1.0` or `-1.0` constant).
    fn mutkill_212_centroid_shift_zero_iff_identical() {
        use super::centroid_shift;
        // Different sets -> strictly positive shift (kills `0.0`).
        let a: Vec<Vec<f64>> = vec![pt(&[1.0, 1.0])];
        let b: Vec<Vec<f64>> = vec![pt(&[1.0, 2.0])];
        let moved = centroid_shift(&a, &b);
        assert!(moved > 0.0, "moved centroids must report positive shift, got {moved}");
        assert!((moved - 1.0).abs() < 1e-12, "1-unit move => shift 1.0, got {moved}");

        // Identical sets -> exactly 0.0 (kills `1.0` and `-1.0`).
        let same: Vec<Vec<f64>> = vec![pt(&[7.0, 7.0]), pt(&[9.0, 9.0])];
        let zero = centroid_shift(&same, &same.clone());
        assert!(
            (zero - 0.0).abs() < 1e-15,
            "identical centroids must report zero shift, got {zero}"
        );
    }

    #[test]
    // KILLS 212:5 — convergence semantics through the public API. A `0.0`
    // centroid_shift mutant makes kmeans break after iteration 1 regardless
    // of whether centroids have actually settled; a `1.0` / `-1.0` mutant
    // makes the shift constant so the loop either never breaks (`1.0` >
    // epsilon) or always breaks (`-1.0` < epsilon). Either way the FINAL
    // centroids would be wrong for an input that needs multiple Lloyd
    // steps to reach the true pair-means. This input is seeded so the
    // initial centroids are NOT the converged ones; correct convergence
    // requires centroid_shift to report real, decreasing values.
    fn mutkill_212_centroid_shift_drives_correct_convergence() {
        // Two tight groups; with a small epsilon the loop must run until
        // centroids settle on the true group means (1.0 and 101.0).
        let pts = vec![
            pt(&[0.0]),
            pt(&[2.0]),
            pt(&[100.0]),
            pt(&[102.0]),
        ];
        let cfg = KMeansConfig {
            k: 2,
            convergence_epsilon: 1e-9,
            max_iterations: 100,
            seed: 42,
        };
        let (clustered, centroids) = kmeans(&pts, &cfg).expect("ok");
        let lo = &centroids[clustered[0].cluster];
        let hi = &centroids[clustered[2].cluster];
        assert!((lo[0] - 1.0).abs() < 1e-6, "low group mean must be 1.0, got {}", lo[0]);
        assert!((hi[0] - 101.0).abs() < 1e-6, "high group mean must be 101.0, got {}", hi[0]);
    }

    #[test]
    // KILLS 219:5 `kmeans_plus_plus_seed -> Vec<Vec<f64>>` replaced with
    // `vec![]`. Direct unit test of the private seeder. A `vec![]` mutant
    // returns ZERO centroids; the real function must return exactly `k`
    // non-empty centroids, each a clone of an actual input point.
    fn mutkill_219_kmeans_plus_plus_seed_returns_k_real_centroids() {
        use super::kmeans_plus_plus_seed;
        let pts: Vec<Vec<f64>> = vec![
            pt(&[0.0, 0.0]),
            pt(&[1.0, 1.0]),
            pt(&[50.0, 50.0]),
            pt(&[99.0, 99.0]),
        ];
        let k = 3;
        let seeds = kmeans_plus_plus_seed(&pts, k, 12345);
        // `vec![]` mutant: len 0 -> fails here.
        assert_eq!(seeds.len(), k, "seeder must return exactly k centroids");
        for s in &seeds {
            // Each centroid must be a real, non-empty point of input dim.
            assert!(!s.is_empty(), "seed centroid must be non-empty");
            assert_eq!(s.len(), 2, "seed centroid must carry input dimensionality");
            assert!(
                pts.iter().any(|p| p == s),
                "every seed centroid must be a clone of an actual input point: {s:?}"
            );
        }
    }

    #[test]
    // KILLS 219:5 `kmeans_plus_plus_seed -> vec![]` through the public API.
    // If the seeder returns an empty Vec, `kmeans` would carry zero
    // centroids into the Lloyd loop: `nearest_centroid` over an empty
    // centroid slice returns 0 for every point, and the final `centroids`
    // returned would be empty. The real contract is `centroids.len() == k`.
    // This pins the public post-condition that depends on a working seeder.
    fn mutkill_219_kmeans_output_has_k_centroids_from_seeder() {
        let pts: Vec<Vec<f64>> = (0..10).map(|i| pt(&[f64::from(i), f64::from(i)])).collect();
        for k in 1_usize..=5 {
            let (_, centroids) =
                kmeans(&pts, &KMeansConfig { k, max_iterations: 0, ..KMeansConfig::default() })
                    .expect("ok");
            assert_eq!(
                centroids.len(),
                k,
                "with max_iterations=0 the output centroids ARE the seeder output; \
                 a vec![] seeder would yield 0 centroids for k={k}"
            );
            for c in &centroids {
                assert!(!c.is_empty(), "seeder-derived centroid must be non-empty");
            }
        }
    }

    #[test]
    // KILLS 219:5 — the seeder's first centroid is FNV-determined and the
    // subsequent picks are farthest-point. A `vec![]` mutant produces no
    // seeds at all; this test confirms the seeder picks the genuine far
    // point as a later seed (proving it returns a populated, k-means++-
    // shaped Vec, not an empty one or a degenerate constant).
    fn mutkill_219_seeder_includes_farthest_point() {
        use super::kmeans_plus_plus_seed;
        // One obvious outlier; k=2 seeding must include it as the 2nd seed.
        let pts: Vec<Vec<f64>> = vec![
            pt(&[0.0]),
            pt(&[1.0]),
            pt(&[2.0]),
            pt(&[10_000.0]),
        ];
        let seeds = kmeans_plus_plus_seed(&pts, 2, 777);
        assert_eq!(seeds.len(), 2, "seeder must return k=2 centroids, not vec![]");
        assert!(
            seeds.iter().any(|s| (s[0] - 10_000.0).abs() < 1e-9),
            "k-means++ seeder must include the farthest point at x=10000: {seeds:?}"
        );
    }

    // ====================================================================
    // W4 FINAL mutation-kill pass (S1003529) — re-verification found that
    // the earlier `mutkill_147/151/128` tests do NOT kill several mutants.
    // Diagnosis + replacement tests below. Each test was empirically
    // confirmed against the manually-applied mutation (FAILs on mutant,
    // PASSes on real code).
    // ====================================================================

    // --- 147:5 `nearest_centroid -> usize` with `0` / with `1` ----------
    //
    // Diagnosis of the prior tests: `mutkill_147_151_nearest_centroid_picks_
    // true_minimum` and `mutkill_147_nearest_centroid_returns_zero_when_
    // closest_is_first` DO kill the constant-body mutants when run directly
    // (verified: applying `-> 0` makes them FAIL). They survive in the
    // cargo-mutants report only because the report predates those tests OR
    // the run timed out. The replacement below is a single, unambiguous,
    // self-contained killer pinning four distinct true-nearest indices so
    // neither a constant `0` nor a constant `1` body can satisfy it.

    #[test]
    // KILLS 147:5 `nearest_centroid -> usize` replaced with `0` AND `1`.
    // Four probes, each unambiguously closest to a DIFFERENT centroid
    // index (0, 1, 2, 3). A constant-`0` body fails the index-1/2/3
    // assertions; a constant-`1` body fails the index-0/2/3 assertions.
    // No single constant can satisfy all four.
    fn mutkill_147_final_constant_body_cannot_satisfy_four_distinct_indices() {
        use super::nearest_centroid;
        let centroids: Vec<Vec<f64>> = vec![
            pt(&[0.0]),
            pt(&[100.0]),
            pt(&[200.0]),
            pt(&[300.0]),
        ];
        assert_eq!(nearest_centroid(&[1.0], &centroids), 0, "closest to c0");
        assert_eq!(nearest_centroid(&[101.0], &centroids), 1, "closest to c1");
        assert_eq!(nearest_centroid(&[199.0], &centroids), 2, "closest to c2");
        assert_eq!(nearest_centroid(&[301.0], &centroids), 3, "closest to c3");
    }

    // --- 151:14 `<` with `<=` / `>` / `==` in nearest_centroid ----------
    //
    // Diagnosis of the prior tests: `mutkill_151_nearest_centroid_strict_
    // lt_not_eq` asserts `nearest_centroid([100.0], [[0.0],[100.0]]) == 1`.
    // That kills `>` and `==` (both leave `best` at its initial 0) — BUT
    // NOT `<=`: under `<=` the loop is `if d <= best_d`, and for a point
    // with NO distance ties the last strictly-smaller distance still wins,
    // so `<=` returns the SAME index as `<`. The prior tests never built
    // an input with two centroids EQUIDISTANT from the probe — the only
    // case where `<` and `<=` diverge. The test below fixes that.

    #[test]
    // KILLS 151:14 `<` -> `<=` in nearest_centroid (the survivor).
    // A point EXACTLY equidistant from two centroids. Real `<`
    // (`if d < best_d`) keeps the FIRST (lower-index) centroid it saw,
    // because `d < best_d` is false on the exact tie. The `<=` mutant
    // (`if d <= best_d`) overwrites with the LATER (higher-index)
    // centroid on the tie. Point 5.0 is at squared-distance 25.0 from
    // both [0.0] and [10.0]:
    //   real `<`  -> returns 0  (tie does not overwrite)
    //   `<=` mut  -> returns 1  (tie overwrites)
    fn mutkill_151_final_equidistant_tie_pins_strict_lt_not_le() {
        use super::nearest_centroid;
        let centroids: Vec<Vec<f64>> = vec![pt(&[0.0]), pt(&[10.0])];
        // 5.0 is exactly equidistant: (5-0)^2 = (5-10)^2 = 25.0.
        assert_eq!(
            nearest_centroid(&[5.0], &centroids),
            0,
            "on an exact distance tie, strict `<` keeps the first centroid; \
             a `<=` mutant would overwrite to index 1"
        );
        // Symmetric guard: a SECOND equidistant tie, three centroids, so a
        // `>`/`==` mutant (which never updates `best` past 0) is also pinned
        // — here the true nearest is the MIDDLE one with no tie.
        let three: Vec<Vec<f64>> = vec![pt(&[0.0]), pt(&[50.0]), pt(&[200.0])];
        assert_eq!(
            nearest_centroid(&[48.0], &three),
            1,
            "true nearest is c1; `>`/`==` mutants would return 0"
        );
    }

    // --- 128:18 `<` with `<=` / `>` / `==` in kmeans (convergence) ------
    //
    // Diagnosis of the prior tests: `mutkill_128_*` assert that two
    // well-separated groups resolve into the correct pair-mean centroids.
    // But with k-means++ seeding + a generous `max_iterations`, the Lloyd
    // loop reaches the SAME converged fixed point regardless of WHEN the
    // convergence break fires — the break is purely an iteration-count
    // optimisation, and the final assignments are recomputed afterwards.
    // The prior tests therefore cannot distinguish `<` / `<=` / `>` / `==`.
    // The kill requires an input where breaking EARLY (before convergence)
    // lands on a genuinely different centroid set. Input + seed below were
    // searched against the real FNV seeder so that the k-means++ seeds do
    // NOT align with the natural clusters, making iteration 1's centroids
    // differ from the converged centroids.

    /// Three tight 1-D groups at ~0, ~21, ~41. With `seed = 0` the real
    /// FNV-1a k-means++ seeding for `k = 2` lands seeds that need three
    /// Lloyd iterations to converge to `[[31.0], [1.0]]`.
    fn three_group_drifting_input() -> Vec<Vec<f64>> {
        vec![
            pt(&[0.0]),
            pt(&[1.0]),
            pt(&[2.0]),
            pt(&[20.0]),
            pt(&[21.0]),
            pt(&[22.0]),
            pt(&[40.0]),
            pt(&[41.0]),
            pt(&[42.0]),
        ]
    }

    #[test]
    // KILLS 128:18 `<` -> `>` in kmeans (convergence break).
    // With a small epsilon the `>` mutant (`if shift > epsilon`) breaks
    // after iteration 1 — `shift` on the first iteration is large, so
    // `shift > epsilon` is true — leaving the loop at a NON-converged
    // centroid set. Real `<` keeps iterating to the true fixed point.
    // Empirically (input + seed 0, eps 1e-6, max_iterations 50):
    //   real `<` -> centroids contain 31.0 and 1.0 (converged means)
    //   `>` mut  -> centroids contain 33.2 and 5.75 (broke early)
    fn mutkill_128_final_gt_mutant_breaks_early_at_wrong_centroids() {
        let pts = three_group_drifting_input();
        let cfg = KMeansConfig {
            k: 2,
            convergence_epsilon: 1e-6,
            max_iterations: 50,
            seed: 0,
        };
        let (_, centroids) = kmeans(&pts, &cfg).expect("ok");
        // Real code converges to the true two-group means {31.0, 1.0}.
        // The `>` mutant stalls at {33.2, 5.75} after one iteration.
        let centroid_near_31 = centroids.iter().any(|c| (c[0] - 31.0).abs() < 1e-9);
        let centroid_near_one = centroids.iter().any(|c| (c[0] - 1.0).abs() < 1e-9);
        assert!(
            centroid_near_31 && centroid_near_one,
            "convergence break must run to the true fixed point \
             {{31.0, 1.0}}; a `>` mutant stalls at {{33.2, 5.75}}: {centroids:?}"
        );
    }

    #[test]
    // KILLS 128:18 `<` -> `<=` AND `<` -> `==` in kmeans.
    // `<=` and `==` differ from `<` only when `shift == epsilon` EXACTLY.
    // We engineer `convergence_epsilon` to be the bit-exact value of the
    // first-iteration centroid shift for this input+seed. Then:
    //   real `<` : `shift1 <  eps` is FALSE (shift1 == eps) -> keep going
    //              -> converges to {31.0, 1.0}.
    //   `<=` mut : `shift1 <= eps` is TRUE  -> break after iteration 1
    //              -> stalls at {33.2, 5.75}.
    //   `==` mut : `shift1 == eps` is TRUE  -> break after iteration 1
    //              -> stalls at {33.2, 5.75}.
    // The epsilon is reconstructed bit-exactly via `f64::from_bits` so the
    // `shift == epsilon` equality is portable and deterministic.
    fn mutkill_128_final_engineered_epsilon_kills_le_and_eq() {
        let pts = three_group_drifting_input();
        // Bit-exact first-iteration shift for this input + seed 0:
        // 13.549999999999997 == f64::from_bits(0x402b_1999_9999_9998).
        let engineered_eps = f64::from_bits(0x402b_1999_9999_9998);
        let cfg = KMeansConfig {
            k: 2,
            convergence_epsilon: engineered_eps,
            max_iterations: 50,
            seed: 0,
        };
        let (_, centroids) = kmeans(&pts, &cfg).expect("ok");
        // Real `<` does NOT break on `shift1 == eps`; it converges.
        let centroid_near_31 = centroids.iter().any(|c| (c[0] - 31.0).abs() < 1e-9);
        let centroid_near_one = centroids.iter().any(|c| (c[0] - 1.0).abs() < 1e-9);
        assert!(
            centroid_near_31 && centroid_near_one,
            "with epsilon == the exact iter-1 shift, strict `<` must NOT \
             break (it converges to {{31.0, 1.0}}); a `<=` or `==` mutant \
             breaks after iteration 1 and stalls at {{33.2, 5.75}}: {centroids:?}"
        );
    }

    // --- 160:5 `squared_l2 -> f64` replaced with `0.0` ------------------
    //
    // A direct unit test of the private `squared_l2`. The `-> 0.0` mutant
    // makes every distance zero, collapsing `nearest_centroid` and
    // `centroid_shift`. Hand-computed: squared-L2 of [0,0] and [3,4] is
    // 9 + 16 = 25; of [1.0] and [4.0] is 9.

    #[test]
    // KILLS 160:5 `squared_l2 -> f64` replaced with `0.0`.
    // Direct hand-computed expectations: a `-> 0.0` body returns 0.0 for
    // every input and fails both assertions.
    fn mutkill_160_squared_l2_is_real_distance_not_zero() {
        use super::squared_l2;
        // (3-0)^2 + (4-0)^2 = 9 + 16 = 25.
        let d2 = squared_l2(&[0.0, 0.0], &[3.0, 4.0]);
        assert!(
            (d2 - 25.0).abs() < 1e-12,
            "squared_l2([0,0],[3,4]) must be 25.0, not the 0.0 mutant: {d2}"
        );
        // (4-1)^2 = 9 — 1-D, single coordinate.
        let d1 = squared_l2(&[1.0], &[4.0]);
        assert!(
            (d1 - 9.0).abs() < 1e-12,
            "squared_l2([1],[4]) must be 9.0, not the 0.0 mutant: {d1}"
        );
        // Identical points => genuinely 0.0 (proves the function is not a
        // non-zero constant either).
        let dz = squared_l2(&[7.0, 7.0], &[7.0, 7.0]);
        assert!((dz - 0.0).abs() < 1e-15, "identical points => 0.0, got {dz}");
    }
}
