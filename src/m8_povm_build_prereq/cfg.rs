//! Band thresholds + classifier — single source of truth for both build-time
//! (`build.rs`) and runtime ([`super::health::probe_band`]).
//!
//! Per m8 spec § 5: `build.rs` MAY `include!` this file to inherit the band
//! constants verbatim, so build-time and runtime cannot drift. For that
//! reason this file is intentionally self-contained — no `use` statements at
//! the top level — and is safe to inline into a build script.
//!
//! The thresholds are taken verbatim from [Hebbian v3 — Post-CR-2 Threshold
//! Reconciliation](https://example.invalid/hebbian-v3-reconciliation) (Phase
//! 1 substrate-LTP-density window). Phase 2 (>0.05) and Phase 3 (>0.10) are
//! provisional pending the 30-day baseline window that opens 2026-05-17.

/// POVM `learning_health` lower band (post-CR-2 magnitude-weighted floor).
///
/// Values below this are classified [`BandClassification::BelowFloor`] —
/// either CR-2 regression on the substrate side, or an uncalibrated POVM.
pub const POVM_LH_BAND_LOW: f64 = 0.05;

/// POVM `learning_health` upper band (post-CR-2 magnitude-weighted ceiling).
///
/// Values above this are classified [`BandClassification::AboveCeiling`] —
/// either pre-CR-2 binary inflation (the canonical 13.6× regression case) or
/// a substrate anomaly.
pub const POVM_LH_BAND_HIGH: f64 = 0.15;

/// Edge-tolerance for `cargo:warning=` precursor-signal emission. When a
/// probe value lands within this distance of either band edge, operators see
/// a non-fatal warning so they can investigate before the band is crossed.
pub const POVM_LH_EDGE_TOLERANCE: f64 = 0.01;

/// Classification of a POVM `learning_health` value vs the Phase-1 band.
///
/// Ordering reflects ascending severity for in-finite values
/// ([`Self::BelowFloor`] < [`Self::InBand`] < [`Self::AboveCeiling`]); the
/// [`Self::Nan`] variant is reserved for non-finite probe responses and is
/// not part of the linear ordering. See [`Self::ordinal`] for the integer
/// projection used in metrics emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BandClassification {
    /// `< POVM_LH_BAND_LOW` — CR-2 regression OR uncalibrated substrate.
    BelowFloor,
    /// `[POVM_LH_BAND_LOW, POVM_LH_BAND_HIGH]` — healthy magnitude-weighted
    /// Phase-1 band.
    InBand,
    /// `> POVM_LH_BAND_HIGH` — pre-CR-2 binary inflation OR substrate anomaly.
    AboveCeiling,
    /// Probe returned a non-finite value (`NaN`, `+∞`, or `-∞`).
    Nan,
}

impl BandClassification {
    /// Integer projection for metrics emission and per-band-arm Watcher
    /// class triggers. The values match the `m8_povm_band_classification`
    /// gauge documented in m8 spec § 9 Observability.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::BelowFloor => 0,
            Self::InBand => 1,
            Self::AboveCeiling => 2,
            Self::Nan => 3,
        }
    }

    /// Banner text for `tracing` log messages and `cargo:warning=` lines.
    /// Stable identifier — DO NOT rename without coordinating with the m05
    /// metrics collector and the Watcher Class-A activation pattern.
    #[must_use]
    pub const fn banner(self) -> &'static str {
        match self {
            Self::BelowFloor => "BELOW-FLOOR",
            Self::InBand => "IN-BAND",
            Self::AboveCeiling => "ABOVE-CEILING",
            Self::Nan => "NAN",
        }
    }

    /// `true` if the classification represents a usable substrate state. Only
    /// [`Self::InBand`] qualifies; every other variant must refuse downstream
    /// POVM reads per m8 spec § 8 failure modes.
    #[must_use]
    pub const fn is_healthy(self) -> bool {
        matches!(self, Self::InBand)
    }

    /// `true` if the classification is within [`POVM_LH_EDGE_TOLERANCE`] of
    /// either band edge — useful for emitting precursor warnings without
    /// failing the build.
    ///
    /// Returns `false` for [`Self::Nan`] (no edge defined) and for values
    /// strictly inside the band by more than the tolerance.
    #[must_use]
    pub fn is_band_edge(value: f64) -> bool {
        if !value.is_finite() {
            return false;
        }
        let near_low = (value - POVM_LH_BAND_LOW).abs() <= POVM_LH_EDGE_TOLERANCE;
        let near_high = (value - POVM_LH_BAND_HIGH).abs() <= POVM_LH_EDGE_TOLERANCE;
        near_low || near_high
    }
}

/// Classify a POVM `learning_health` value against the Phase-1 band.
///
/// The band is inclusive on both ends — `classify(POVM_LH_BAND_LOW)` and
/// `classify(POVM_LH_BAND_HIGH)` both return [`BandClassification::InBand`].
/// Non-finite values (NaN, ±∞) return [`BandClassification::Nan`].
///
/// # Examples
///
/// ```
/// use workflow_core::m8_povm_build_prereq::{classify, BandClassification};
///
/// assert_eq!(classify(0.10),       BandClassification::InBand);
/// assert_eq!(classify(0.04),       BandClassification::BelowFloor);
/// assert_eq!(classify(0.20),       BandClassification::AboveCeiling);
/// assert_eq!(classify(f64::NAN),   BandClassification::Nan);
/// ```
#[must_use]
pub fn classify(value: f64) -> BandClassification {
    if !value.is_finite() {
        BandClassification::Nan
    } else if value < POVM_LH_BAND_LOW {
        BandClassification::BelowFloor
    } else if value > POVM_LH_BAND_HIGH {
        BandClassification::AboveCeiling
    } else {
        BandClassification::InBand
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify, BandClassification, POVM_LH_BAND_HIGH, POVM_LH_BAND_LOW, POVM_LH_EDGE_TOLERANCE,
    };

    // ---- F-Unit (25 tests) ----------------------------------------------

    #[test]
    fn classify_zero_is_below_floor() {
        assert_eq!(classify(0.0), BandClassification::BelowFloor);
    }

    #[test]
    fn classify_just_below_floor_is_below_floor() {
        assert_eq!(classify(0.0499), BandClassification::BelowFloor);
    }

    #[test]
    fn classify_exact_floor_is_in_band() {
        // Band is inclusive on the floor edge.
        assert_eq!(classify(POVM_LH_BAND_LOW), BandClassification::InBand);
    }

    #[test]
    fn classify_just_above_floor_is_in_band() {
        assert_eq!(classify(0.0501), BandClassification::InBand);
    }

    #[test]
    fn classify_band_midpoint_is_in_band() {
        let mid = (POVM_LH_BAND_LOW + POVM_LH_BAND_HIGH) / 2.0;
        assert_eq!(classify(mid), BandClassification::InBand);
    }

    #[test]
    fn classify_just_below_ceiling_is_in_band() {
        assert_eq!(classify(0.1499), BandClassification::InBand);
    }

    #[test]
    fn classify_exact_ceiling_is_in_band() {
        // Band is inclusive on the ceiling edge.
        assert_eq!(classify(POVM_LH_BAND_HIGH), BandClassification::InBand);
    }

    #[test]
    fn classify_just_above_ceiling_is_above_ceiling() {
        assert_eq!(classify(0.1501), BandClassification::AboveCeiling);
    }

    #[test]
    fn classify_far_above_ceiling_is_above_ceiling() {
        // Mirrors the pre-CR-2 inflated `learning_health=0.9114` canonical case.
        assert_eq!(classify(0.9114), BandClassification::AboveCeiling);
    }

    #[test]
    fn classify_unity_is_above_ceiling() {
        assert_eq!(classify(1.0), BandClassification::AboveCeiling);
    }

    #[test]
    fn classify_negative_is_below_floor() {
        assert_eq!(classify(-0.01), BandClassification::BelowFloor);
    }

    #[test]
    fn classify_large_negative_is_below_floor() {
        assert_eq!(classify(-1000.0), BandClassification::BelowFloor);
    }

    #[test]
    fn classify_nan_is_nan() {
        assert_eq!(classify(f64::NAN), BandClassification::Nan);
    }

    #[test]
    fn classify_positive_infinity_is_nan() {
        assert_eq!(classify(f64::INFINITY), BandClassification::Nan);
    }

    #[test]
    fn classify_negative_infinity_is_nan() {
        assert_eq!(classify(f64::NEG_INFINITY), BandClassification::Nan);
    }

    #[test]
    fn classify_smallest_positive_is_below_floor() {
        // f64::MIN_POSITIVE is far smaller than POVM_LH_BAND_LOW.
        assert_eq!(classify(f64::MIN_POSITIVE), BandClassification::BelowFloor);
    }

    #[test]
    fn classify_post_cr2_canonical_value_is_in_band() {
        // `learning_health=0.067` is the post-CR-2 magnitude-weighted value
        // observed live across the habitat (~0.067 ≈ TLV2 + Hebbian-v3 row).
        assert_eq!(classify(0.067), BandClassification::InBand);
    }

    #[test]
    fn ordinal_below_floor() {
        assert_eq!(BandClassification::BelowFloor.ordinal(), 0);
    }

    #[test]
    fn ordinal_in_band() {
        assert_eq!(BandClassification::InBand.ordinal(), 1);
    }

    #[test]
    fn ordinal_above_ceiling() {
        assert_eq!(BandClassification::AboveCeiling.ordinal(), 2);
    }

    #[test]
    fn ordinal_nan() {
        assert_eq!(BandClassification::Nan.ordinal(), 3);
    }

    #[test]
    fn banner_strings_are_stable() {
        // Stable identifiers — m05 metrics collector + Watcher Class-A
        // depend on these literal strings. Do not rename without coordinated
        // amendment across both consumers.
        assert_eq!(BandClassification::BelowFloor.banner(), "BELOW-FLOOR");
        assert_eq!(BandClassification::InBand.banner(), "IN-BAND");
        assert_eq!(BandClassification::AboveCeiling.banner(), "ABOVE-CEILING");
        assert_eq!(BandClassification::Nan.banner(), "NAN");
    }

    #[test]
    fn is_healthy_only_for_in_band() {
        assert!(BandClassification::InBand.is_healthy());
        assert!(!BandClassification::BelowFloor.is_healthy());
        assert!(!BandClassification::AboveCeiling.is_healthy());
        assert!(!BandClassification::Nan.is_healthy());
    }

    #[test]
    fn is_band_edge_detects_proximity_to_floor() {
        assert!(BandClassification::is_band_edge(POVM_LH_BAND_LOW));
        assert!(BandClassification::is_band_edge(POVM_LH_BAND_LOW - 0.005));
        assert!(BandClassification::is_band_edge(POVM_LH_BAND_LOW + 0.005));
        assert!(!BandClassification::is_band_edge(POVM_LH_BAND_LOW - 0.02));
    }

    #[test]
    fn is_band_edge_detects_proximity_to_ceiling() {
        assert!(BandClassification::is_band_edge(POVM_LH_BAND_HIGH));
        assert!(BandClassification::is_band_edge(POVM_LH_BAND_HIGH - 0.005));
        assert!(BandClassification::is_band_edge(POVM_LH_BAND_HIGH + 0.005));
        assert!(!BandClassification::is_band_edge(POVM_LH_BAND_HIGH + 0.02));
    }

    #[test]
    fn is_band_edge_rejects_non_finite() {
        assert!(!BandClassification::is_band_edge(f64::NAN));
        assert!(!BandClassification::is_band_edge(f64::INFINITY));
        assert!(!BandClassification::is_band_edge(f64::NEG_INFINITY));
    }

    #[test]
    fn is_band_edge_rejects_deep_in_band() {
        let mid = (POVM_LH_BAND_LOW + POVM_LH_BAND_HIGH) / 2.0;
        assert!(!BandClassification::is_band_edge(mid));
    }

    // ---- F-Property (5 tests) -------------------------------------------

    #[test]
    fn classify_is_monotonic_on_band_ordinal() {
        // Iterate from 0.0 to 0.2 in steps of 0.001; classify(v).ordinal()
        // should be non-decreasing across the finite range (NaN excluded).
        let mut prev: u8 = 0;
        let mut value = 0.0_f64;
        while value <= 0.2 {
            let ord = classify(value).ordinal();
            assert!(
                ord >= prev,
                "non-monotonic at value={value} (ord={ord} prev={prev})"
            );
            prev = ord;
            value += 0.001;
        }
    }

    #[test]
    fn classify_is_idempotent_for_finite_input() {
        // Same input → same classification across 1000 repeats.
        let probes: [f64; 5] = [0.0, 0.05, 0.10, 0.15, 0.20];
        for &p in &probes {
            let first = classify(p);
            for _ in 0..1000 {
                assert_eq!(classify(p), first, "non-idempotent at value={p}");
            }
        }
    }

    #[test]
    fn classify_is_deterministic_for_nan() {
        for _ in 0..100 {
            assert_eq!(classify(f64::NAN), BandClassification::Nan);
        }
    }

    #[test]
    fn band_partition_is_total_for_finite_input() {
        // Every finite input gets exactly one classification (no overlap, no
        // gap). Sampled across 0.0 .. 1.0 step 0.001.
        let mut value = 0.0_f64;
        while value <= 1.0 {
            let c = classify(value);
            assert!(
                c == BandClassification::BelowFloor
                    || c == BandClassification::InBand
                    || c == BandClassification::AboveCeiling,
                "finite input {value} classified as {c:?}"
            );
            value += 0.001;
        }
    }

    #[test]
    fn ordinal_ordering_matches_band_ordering() {
        // Below < In < Above for ordinals; NaN reserved at 3.
        assert!(
            BandClassification::BelowFloor.ordinal() < BandClassification::InBand.ordinal()
        );
        assert!(
            BandClassification::InBand.ordinal() < BandClassification::AboveCeiling.ordinal()
        );
        assert!(
            BandClassification::AboveCeiling.ordinal() < BandClassification::Nan.ordinal()
        );
    }

    // ---- F-Regression (2 tests) -----------------------------------------

    #[test]
    fn regression_band_edge_precision_drift() {
        // F7 regression slot — band-edge value 0.050_000_1 must NOT silently
        // classify InBand if precision drift sneaks in. 0.050_000_1 > 0.05
        // strictly, so classify returns InBand. The regression we guard
        // against is reversing the comparison to `<=` instead of `<`.
        assert_eq!(classify(0.050_000_1), BandClassification::InBand);
        // And the symmetric high-edge:
        assert_eq!(classify(0.149_999_9), BandClassification::InBand);
    }

    #[test]
    fn regression_band_constants_match_hebbian_v3_thresholds() {
        // If these constants drift the entire reason-for-being of m8 collapses.
        // Hebbian v3 Phase 1: substrate-LTP-density > 0.015 (passing); the
        // POVM learning_health band is the magnitude-weighted projection of
        // that into the [0.05, 0.15] window per the reconciliation note.
        // `black_box` defeats const-propagation so the assertions are
        // evaluated at runtime, not compile-time.
        let low = std::hint::black_box(POVM_LH_BAND_LOW);
        let high = std::hint::black_box(POVM_LH_BAND_HIGH);
        let tol = std::hint::black_box(POVM_LH_EDGE_TOLERANCE);
        assert!((low - 0.05).abs() < f64::EPSILON);
        assert!((high - 0.15).abs() < f64::EPSILON);
        assert!((tol - 0.01).abs() < f64::EPSILON);
    }
}
