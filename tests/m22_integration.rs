//! Integration tests for m22 `kmeans` (Wave-D1).
//!
//! Exercises the m22 surface at its public-API call boundary:
//!
//! - Determinism — the same `seed` over the same input yields
//!   bit-identical centroids (compared via `f64::to_bits()`, not an
//!   epsilon — m22's contract is exact reproducibility).
//! - Boundary — `k == n` places every point in its own cluster.
//! - Adversarial — NaN / +∞ / -∞ coordinates are refused with a typed
//!   error (the public taxonomy reports these as `DimMismatch` with the
//!   `usize::MAX` sentinel — see m22 mod docstring F-m22-02).
//! - Wave-1 fix regression — an empty cluster mid-iteration retains its
//!   prior centroid rather than recentering on the origin (no NaN drift).
//! - Wave-A4 H7 fix regression — the k-means++ tiebreak bias is bounded
//!   relative to distance and never flips materially-different ordering.
//! - Chain-proof — the algorithm converges within
//!   `DEFAULT_MAX_ITERATIONS`.
//! - Adversarial — all-identical points (zero variance) are handled
//!   without panic.
//! - Contract regression — the `KMeansError` taxonomy is exhaustively
//!   pinned.
//!
//! Day-1 surface note (flagged for orchestrator): `kmeans` does not
//! expose an iteration counter in its return value, so the
//! within-`DEFAULT_MAX_ITERATIONS` convergence test asserts via the
//! observable consequence — a clearly-separable input reaches a stable
//! correct partition using the *default* config (whose `max_iterations`
//! IS `DEFAULT_MAX_ITERATIONS`). A future revision exposing an
//! `iterations_run` field would let this test count directly.

#![allow(clippy::doc_markdown)]

use workflow_core::m22_kmeans::{
    kmeans, KMeansConfig, KMeansError, DEFAULT_CONVERGENCE_EPSILON, DEFAULT_MAX_ITERATIONS,
};

// ---- fixtures ------------------------------------------------------------

/// Two clearly-separable groups of 2D points — used by convergence and
/// determinism tests.
fn separable_2d() -> Vec<Vec<f64>> {
    vec![
        vec![0.0, 0.0],
        vec![0.2, 0.1],
        vec![0.1, 0.2],
        vec![20.0, 20.0],
        vec![20.2, 19.9],
        vec![19.8, 20.1],
    ]
}

// ---- determinism ---------------------------------------------------------

// rationale: Determinism — m22's first invariant is exact
// reproducibility: the same seed over the same input MUST produce
// bit-identical centroids. Compared via `to_bits()` so a sub-ULP drift
// (e.g. an FMA-vs-add reorder) fails loud.
#[test]
fn m22_same_seed_yields_bit_identical_centroids() {
    // rationale: Determinism (bit-exact centroid reproducibility)
    let pts = separable_2d();
    let cfg = KMeansConfig {
        k: 2,
        seed: 0x5EED_C0DE_1234_5678,
        ..KMeansConfig::default()
    };
    let (_, c1) = kmeans(&pts, &cfg).expect("first run");
    let (_, c2) = kmeans(&pts, &cfg).expect("second run");
    assert_eq!(c1.len(), c2.len(), "centroid count diverged");
    for (a, b) in c1.iter().zip(c2.iter()) {
        assert_eq!(a.len(), b.len(), "centroid dimension diverged");
        for (av, bv) in a.iter().zip(b.iter()) {
            assert_eq!(
                av.to_bits(),
                bv.to_bits(),
                "centroid not bit-identical: {av} vs {bv}"
            );
        }
    }
}

// ---- boundary ------------------------------------------------------------

// rationale: Boundary — when `k == n` every point becomes its own
// cluster; the partition must contain exactly `n` distinct cluster ids.
#[test]
fn m22_k_equals_n_each_point_own_cluster() {
    // rationale: Boundary (k == n)
    let pts = vec![vec![0.0], vec![10.0], vec![20.0], vec![30.0]];
    let cfg = KMeansConfig {
        k: 4,
        ..KMeansConfig::default()
    };
    let (clustered, centroids) = kmeans(&pts, &cfg).expect("k==n run");
    assert_eq!(centroids.len(), 4, "k centroids expected");
    let mut ids: Vec<usize> = clustered.iter().map(|c| c.cluster).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 4, "every point must occupy a distinct cluster");
}

// ---- adversarial: non-finite input ---------------------------------------

// rationale: Adversarial input — a NaN coordinate produces ill-defined
// centroid arithmetic; m22 MUST refuse it with a typed error rather than
// silently propagate NaN through the centroid math.
#[test]
fn m22_rejects_nan_input() {
    // rationale: Adversarial input (NaN coordinate)
    let pts = vec![vec![0.0, 0.0], vec![f64::NAN, 1.0]];
    let cfg = KMeansConfig {
        k: 2,
        ..KMeansConfig::default()
    };
    let err = kmeans(&pts, &cfg).expect_err("NaN input must be refused");
    assert!(
        matches!(err, KMeansError::DimMismatch { .. }),
        "NaN must be reported via the typed taxonomy, got {err:?}"
    );
}

// rationale: Adversarial input — a +∞ coordinate is non-finite and MUST
// be refused identically to NaN.
#[test]
fn m22_rejects_infinite_input() {
    // rationale: Adversarial input (+infinity coordinate)
    let pts = vec![vec![0.0, 0.0], vec![f64::INFINITY, 1.0]];
    let cfg = KMeansConfig {
        k: 2,
        ..KMeansConfig::default()
    };
    let err = kmeans(&pts, &cfg).expect_err("infinite input must be refused");
    assert!(
        matches!(err, KMeansError::DimMismatch { .. }),
        "infinity must be reported via the typed taxonomy, got {err:?}"
    );
    // -infinity is the symmetric case and must also be refused.
    let neg = vec![vec![0.0, 0.0], vec![f64::NEG_INFINITY, 1.0]];
    assert!(
        matches!(kmeans(&neg, &cfg), Err(KMeansError::DimMismatch { .. })),
        "-infinity must also be refused"
    );
}

// ---- Wave-1 fix regression: empty-cluster retention ----------------------

// rationale: Wave-1 fix regression — when seeding produces a centroid no
// point claims, m22 retains the prior centroid rather than recentering
// on the origin. The regression guard is: no centroid component goes
// non-finite, and every point keeps a valid in-range cluster assignment.
#[test]
fn m22_empty_cluster_retains_prior_centroid_no_nan() {
    // rationale: Wave-1 fix regression (empty-cluster → retain prior)
    // Three points clustered tightly at one corner with k=3 forces at
    // least one seeded centroid to claim no points mid-iteration.
    let pts = vec![vec![0.0, 0.0], vec![0.01, 0.01], vec![0.02, 0.02]];
    let cfg = KMeansConfig {
        k: 3,
        seed: 1,
        ..KMeansConfig::default()
    };
    let (clustered, centroids) = kmeans(&pts, &cfg).expect("empty-cluster run");
    assert_eq!(centroids.len(), 3, "k centroids must still be returned");
    for c in &centroids {
        for v in c {
            assert!(
                v.is_finite(),
                "empty-cluster fallback produced a non-finite centroid: {v}"
            );
        }
    }
    for c in &clustered {
        assert!(c.cluster < 3, "cluster index out of range: {}", c.cluster);
    }
}

// ---- Wave-A4 H7 fix regression: bounded tiebreak bias --------------------

// rationale: Wave-A4 H7 fix regression — the k-means++ tiebreak bias is
// bounded by ~d·1024·ε and must NEVER flip the ordering of
// materially-different distances. With three points (near pair + clear
// far point) k-means++ MUST consistently seed toward the far point; the
// pre-fix bias (~1e7 magnitude) would have made that choice
// hash-determined. Across many seeds the far point must be picked
// effectively every time — bias ≤ d·1e-10 in practical terms.
#[test]
fn m22_tiebreak_bias_bounded_relative_to_distance() {
    // rationale: Wave-A4 H7 fix regression (bounded tiebreak bias)
    let pts = vec![vec![0.0], vec![1.0], vec![100.0]];
    let mut far_chosen = 0_usize;
    let trials = 60_u64;
    for seed in 0..trials {
        let cfg = KMeansConfig {
            k: 2,
            seed,
            ..KMeansConfig::default()
        };
        let (_, centroids) = kmeans(&pts, &cfg).expect("tiebreak run");
        if centroids.iter().any(|c| (c[0] - 100.0).abs() < 1e-6) {
            far_chosen += 1;
        }
    }
    // The bounded bias (~2.3e-13·max(d,1)) cannot flip the d=1 vs d=100
    // ordering, so the far point is chosen on effectively every seed.
    assert!(
        far_chosen >= 55,
        "tiebreak bias appears to dominate distance: far point chosen \
         only {far_chosen}/{trials} times (expected >= 55)"
    );
}

// ---- chain-proof: convergence within DEFAULT_MAX_ITERATIONS --------------

// rationale: Chain-proof — m22 converges within `DEFAULT_MAX_ITERATIONS`.
// `kmeans` exposes no iteration counter, so this asserts via the
// observable consequence: under the *default* config (whose
// `max_iterations` IS `DEFAULT_MAX_ITERATIONS` and whose
// `convergence_epsilon` IS `DEFAULT_CONVERGENCE_EPSILON`) a clearly-
// separable input reaches the correct stable partition. If convergence
// needed more than the default budget the partition would be wrong.
#[test]
fn m22_converges_within_default_max_iterations() {
    // rationale: Chain-proof (convergence inside the default iteration cap)
    // Sanity-pin the default constants the contract depends on.
    assert_eq!(
        KMeansConfig::default().max_iterations,
        DEFAULT_MAX_ITERATIONS,
        "default config must use DEFAULT_MAX_ITERATIONS"
    );
    assert!(
        (KMeansConfig::default().convergence_epsilon - DEFAULT_CONVERGENCE_EPSILON).abs()
            < f64::EPSILON,
        "default config must use DEFAULT_CONVERGENCE_EPSILON"
    );
    let pts = separable_2d();
    let cfg = KMeansConfig {
        k: 2,
        seed: 7,
        ..KMeansConfig::default()
    };
    let (clustered, _) = kmeans(&pts, &cfg).expect("convergence run");
    // The first triple (near origin) and the second triple (near 20,20)
    // must each land in a single, distinct cluster — only reachable if
    // Lloyd's converged inside DEFAULT_MAX_ITERATIONS.
    assert_eq!(clustered[0].cluster, clustered[1].cluster);
    assert_eq!(clustered[1].cluster, clustered[2].cluster);
    assert_eq!(clustered[3].cluster, clustered[4].cluster);
    assert_eq!(clustered[4].cluster, clustered[5].cluster);
    assert_ne!(
        clustered[0].cluster, clustered[3].cluster,
        "the two separable groups must converge to distinct clusters"
    );
}

// ---- adversarial: zero variance ------------------------------------------

// rationale: Adversarial input — all-identical points (zero variance)
// drive every empty-cluster recovery path at once; m22 must complete
// without panic and return only finite centroids.
#[test]
fn m22_all_identical_points_handled_without_panic() {
    // rationale: Adversarial input (zero variance)
    let pts = vec![vec![3.5, 3.5]; 6];
    let cfg = KMeansConfig {
        k: 3,
        seed: 11,
        ..KMeansConfig::default()
    };
    let (clustered, centroids) = kmeans(&pts, &cfg).expect("zero-variance run");
    assert_eq!(centroids.len(), 3);
    for c in &centroids {
        for v in c {
            assert!(v.is_finite(), "zero-variance produced non-finite centroid: {v}");
        }
    }
    // Every identical point lands in the same cluster.
    let first = clustered[0].cluster;
    for c in &clustered {
        assert_eq!(c.cluster, first, "identical points must co-cluster");
    }
}

// ---- contract regression -------------------------------------------------

// rationale: Contract regression — the `KMeansError` taxonomy is pinned.
// The match is exhaustive: a new variant added without a test arm fails
// to compile, forcing the taxonomy to stay covered.
#[test]
fn m22_kmeans_error_variants_exhaustive() {
    // rationale: Contract regression (error taxonomy exhaustiveness)
    let cases = [
        KMeansError::Empty,
        KMeansError::KExceedsN { k: 5, n: 3 },
        KMeansError::DimMismatch {
            expected: 2,
            got: 3,
        },
    ];
    for err in cases {
        // Display payload must be non-empty for every variant.
        assert!(!format!("{err}").is_empty(), "empty Display for {err:?}");
        match err {
            KMeansError::Empty
            | KMeansError::KExceedsN { .. }
            | KMeansError::DimMismatch { .. } => {}
        }
    }
    // Pin the empty-input and k>n refusals through the real surface.
    assert!(matches!(
        kmeans(&[], &KMeansConfig::default()),
        Err(KMeansError::Empty)
    ));
    let one = vec![vec![1.0, 2.0]];
    assert!(matches!(
        kmeans(
            &one,
            &KMeansConfig {
                k: 5,
                ..KMeansConfig::default()
            }
        ),
        Err(KMeansError::KExceedsN { .. })
    ));
}
