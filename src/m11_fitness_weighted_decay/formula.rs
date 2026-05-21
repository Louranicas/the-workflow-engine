//! The Gap 2 **NEW PRIMITIVE** — `frequency × fitness × recency` compound
//! decay formula.
//!
//! Per m11 spec § 5.1 and [CLAUDE.md § structural-gap authorship]:
//!
//! ```text
//! decay_factor = base_rate + (1.0 - base_rate) × clamp(frequency × fitness × recency, 0.0, 1.0)
//! ```
//!
//! where `base_rate = 1.0 - plain_decay_rate` (default `0.98` →
//! `plain_decay_rate = 0.02`).
//!
//! # Interpretation
//!
//! - All three signals at `1.0` → `decay_factor = 1.0` (no decay; thriving).
//! - All three signals at `0.0` → `decay_factor = base_rate` (fastest legal
//!   decay; the floor).
//! - **High frequency, low fitness** (used but not producing good outcomes)
//!   → middle of range; decay slows relative to an unused workflow but does
//!   NOT stop. This is the spec's compositional integrity check: usage
//!   alone never grants immortality.
//!
//! # Source lineage
//!
//! - `base_rate` shape: lifted from `povm-v2_lifecycle.rs::decay_pathways`
//!   (`weight *= (1 - rate)` per cycle).
//! - `frequency` signal: m14 `evidence_aggregator` normalised
//!   `run_count` (see [`super::inputs::frequency_factor`]).
//! - `fitness` signal: stcortex `pathway.weight` (m42 substrate route,
//!   post-2026-05-17 ADR; see [`super::inputs::fitness_factor`]).
//! - `recency` signal: `exp(-lambda × days_since_last_run)` from m7
//!   `last_run_at` (see [`super::inputs::recency_factor`]).
//! - **Composition (the NEW PRIMITIVE):** this function — no upstream
//!   ancestor.

use super::error::DecayError;

/// Compound decay factor in `[base_rate, 1.0]`. Applied multiplicatively
/// to m31 selector composite scores: `weight_next = weight × DecayFactor`.
/// A factor of `1.0` means no decay this cycle; a factor of `base_rate` is
/// the fastest decay the law allows (the floor).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecayFactor(f64);

impl DecayFactor {
    /// Inner `f64` value.
    #[must_use]
    pub const fn as_f64(self) -> f64 {
        self.0
    }

    /// Construct from a raw `f64`. Returns [`DecayError::OutOfRange`] when
    /// `value` is non-finite or outside `[0.0, 1.0]`.
    ///
    /// # Errors
    ///
    /// [`DecayError::OutOfRange`] for non-finite or out-of-range input.
    pub fn new(value: f64) -> Result<Self, DecayError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(DecayError::OutOfRange { value });
        }
        Ok(Self(value))
    }
}

impl std::fmt::Display for DecayFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.6}", self.0)
    }
}

/// Compute the Gap 2 compound decay factor.
///
/// FMA arithmetic: `base_rate.mul_add(1.0 - cs, cs)`
/// = `base_rate × (1 - cs) + cs`
/// = `base_rate + (1 - base_rate) × cs`
///
/// where `cs = clamp(frequency × fitness × recency, 0, 1)`.
///
/// Inputs are debug-asserted to be in `[0.0, 1.0]` (release builds drop
/// the debug_assert and rely on the `clamp` for safety).
///
/// # Errors
///
/// [`DecayError::OutOfRange`] cannot occur via this function's well-typed
/// arithmetic on `[0,1]` inputs, but is propagated from
/// [`DecayFactor::new`] for compile-time exhaustiveness in callers.
pub fn compute_decay_factor(
    frequency: f64,
    fitness: f64,
    recency: f64,
    plain_decay_rate: f64,
) -> Result<DecayFactor, DecayError> {
    debug_assert!(
        (0.0..=1.0).contains(&frequency),
        "frequency must be in [0,1]"
    );
    debug_assert!(
        (0.0..=1.0).contains(&fitness),
        "fitness must be in [0,1]"
    );
    debug_assert!(
        (0.0..=1.0).contains(&recency),
        "recency must be in [0,1]"
    );
    debug_assert!(
        (0.0..=1.0).contains(&plain_decay_rate),
        "plain_decay_rate must be in [0,1]"
    );

    let base_rate = 1.0 - plain_decay_rate;
    let compound_signal = (frequency * fitness * recency).clamp(0.0, 1.0);
    let value = base_rate.mul_add(1.0 - compound_signal, compound_signal);
    DecayFactor::new(value)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::super::error::DecayError;
    use super::{compute_decay_factor, DecayFactor};

    fn factor(f: f64, g: f64, r: f64, p: f64) -> f64 {
        compute_decay_factor(f, g, r, p).expect("happy").as_f64()
    }

    // ---- DecayFactor newtype (5) ----------------------------------------

    #[test]
    fn decay_factor_new_accepts_zero() {
        let d = DecayFactor::new(0.0).unwrap();
        assert!((d.as_f64() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn decay_factor_new_accepts_one() {
        let d = DecayFactor::new(1.0).unwrap();
        assert!((d.as_f64() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn decay_factor_new_rejects_above_one() {
        let err = DecayFactor::new(1.5).unwrap_err();
        assert!(matches!(err, DecayError::OutOfRange { .. }));
    }

    #[test]
    fn decay_factor_new_rejects_below_zero() {
        let err = DecayFactor::new(-0.5).unwrap_err();
        assert!(matches!(err, DecayError::OutOfRange { .. }));
    }

    #[test]
    fn decay_factor_new_rejects_nan_and_infinity() {
        assert!(matches!(
            DecayFactor::new(f64::NAN),
            Err(DecayError::OutOfRange { .. })
        ));
        assert!(matches!(
            DecayFactor::new(f64::INFINITY),
            Err(DecayError::OutOfRange { .. })
        ));
        assert!(matches!(
            DecayFactor::new(f64::NEG_INFINITY),
            Err(DecayError::OutOfRange { .. })
        ));
    }

    // ---- Formula corner cases (15) --------------------------------------

    #[test]
    fn all_zero_signals_returns_base_rate() {
        // base_rate = 1 - 0.02 = 0.98
        assert!((factor(0.0, 0.0, 0.0, 0.02) - 0.98).abs() < 1e-12);
    }

    #[test]
    fn all_one_signals_returns_one() {
        assert!((factor(1.0, 1.0, 1.0, 0.02) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn signals_at_one_half_each_returns_predictable_value() {
        // cs = 0.125, base_rate = 0.98
        // value = 0.98 * 0.875 + 0.125 = 0.8575 + 0.125 = 0.9825
        let v = factor(0.5, 0.5, 0.5, 0.02);
        assert!((v - 0.9825).abs() < 1e-12, "got {v}");
    }

    #[test]
    fn plain_decay_rate_zero_returns_one_always() {
        for &f in &[0.0, 0.5, 1.0] {
            for &g in &[0.0, 0.5, 1.0] {
                for &r in &[0.0, 0.5, 1.0] {
                    let v = factor(f, g, r, 0.0);
                    assert!((v - 1.0).abs() < 1e-12, "f={f} g={g} r={r} v={v}");
                }
            }
        }
    }

    #[test]
    fn plain_decay_rate_one_returns_compound_signal() {
        // base_rate = 0, so value = 0 * (1-cs) + cs = cs
        let v_zero = factor(0.0, 0.0, 0.0, 1.0);
        assert!((v_zero - 0.0).abs() < 1e-12);
        let v_full = factor(1.0, 1.0, 1.0, 1.0);
        assert!((v_full - 1.0).abs() < 1e-12);
        let v_half = factor(0.5, 0.5, 0.5, 1.0);
        assert!((v_half - 0.125).abs() < 1e-12);
    }

    #[test]
    fn high_frequency_low_fitness_does_not_stop_decay() {
        // Compositional integrity check: frequency alone is not enough.
        // f=1.0, g=0.0, r=1.0 → cs = 0 → value = base_rate = 0.98
        let v = factor(1.0, 0.0, 1.0, 0.02);
        assert!((v - 0.98).abs() < 1e-12);
    }

    #[test]
    fn vary_frequency_only_monotone() {
        let v0 = factor(0.0, 1.0, 1.0, 0.02);
        let v5 = factor(0.5, 1.0, 1.0, 0.02);
        let v1 = factor(1.0, 1.0, 1.0, 0.02);
        assert!(v0 < v5);
        assert!(v5 < v1);
    }

    #[test]
    fn vary_fitness_only_monotone() {
        let v0 = factor(1.0, 0.0, 1.0, 0.02);
        let v5 = factor(1.0, 0.5, 1.0, 0.02);
        let v1 = factor(1.0, 1.0, 1.0, 0.02);
        assert!(v0 < v5);
        assert!(v5 < v1);
    }

    #[test]
    fn vary_recency_only_monotone() {
        let v0 = factor(1.0, 1.0, 0.0, 0.02);
        let v5 = factor(1.0, 1.0, 0.5, 0.02);
        let v1 = factor(1.0, 1.0, 1.0, 0.02);
        assert!(v0 < v5);
        assert!(v5 < v1);
    }

    #[test]
    fn permutation_of_signals_yields_same_factor() {
        // f*g*r is symmetric under permutation of factors.
        let permutations: [(f64, f64, f64); 6] = [
            (1.0, 0.5, 0.25),
            (1.0, 0.25, 0.5),
            (0.5, 1.0, 0.25),
            (0.5, 0.25, 1.0),
            (0.25, 1.0, 0.5),
            (0.25, 0.5, 1.0),
        ];
        let reference = factor(permutations[0].0, permutations[0].1, permutations[0].2, 0.02);
        for (a, b, c) in permutations {
            let v = factor(a, b, c, 0.02);
            assert!((v - reference).abs() < 1e-12, "perm ({a},{b},{c})");
        }
    }

    #[test]
    fn signal_of_zero_anywhere_collapses_to_base_rate() {
        // Any zero factor in f*g*r drives cs to 0 → value = base_rate.
        for (f, g, r) in [
            (0.0, 1.0, 1.0),
            (1.0, 0.0, 1.0),
            (1.0, 1.0, 0.0),
            (0.0, 0.5, 1.0),
        ] {
            let v = factor(f, g, r, 0.02);
            assert!((v - 0.98).abs() < 1e-12, "f={f} g={g} r={r} v={v}");
        }
    }

    #[test]
    fn factor_for_default_plain_decay_rate_at_post_cr2_band_is_valid() {
        // Smoke: default plain_decay_rate (0.02) + post-CR-2 band fitness
        // (0.067 → fitness_factor passes through to 0.067) + middle freq/rec
        // produces a sensible value > base_rate.
        let v = factor(0.5, 0.067, 0.7, 0.02);
        assert!(v > 0.98 - 1e-12, "v={v}");
        assert!(v < 1.0 + 1e-12, "v={v}");
    }

    #[test]
    fn display_format_is_six_decimals() {
        let d = DecayFactor::new(0.123_456_789).unwrap();
        assert_eq!(format!("{d}"), "0.123457");
    }

    #[test]
    fn implements_copy_and_partial_eq() {
        let d = DecayFactor::new(0.5).unwrap();
        let e = d;
        assert_eq!(d, e);
    }

    #[test]
    fn fma_form_matches_naive_form_for_pinned_inputs() {
        // FMA-vs-naive parity for pinned inputs. Property test version
        // below covers 10k random pairs.
        for &(f, g, r, p) in &[
            (0.0_f64, 0.0_f64, 0.0_f64, 0.02_f64),
            (1.0, 1.0, 1.0, 0.02),
            (0.5, 0.5, 0.5, 0.02),
            (0.3, 0.7, 0.9, 0.05),
        ] {
            let base = 1.0 - p;
            let cs = (f * g * r).clamp(0.0, 1.0);
            let naive = base * (1.0 - cs) + cs;
            let fma = factor(f, g, r, p);
            assert!((fma - naive).abs() < 1e-12, "(f,g,r,p)=({f},{g},{r},{p})");
        }
    }

    // ---- F-Property (10 invariants via proptest) ------------------------

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 10_000,
            .. ProptestConfig::default()
        })]

        // Invariant 1: decay ∈ [base_rate, 1.0] for all finite inputs.
        #[test]
        fn prop_decay_in_base_to_one(
            f in 0.0_f64..=1.0,
            g in 0.0_f64..=1.0,
            r in 0.0_f64..=1.0,
            p in 0.0_f64..=1.0,
        ) {
            let base_rate = 1.0 - p;
            let v = factor(f, g, r, p);
            prop_assert!(v >= base_rate - 1e-12, "v={v} < base_rate={base_rate}");
            prop_assert!(v <= 1.0 + 1e-12, "v={v} > 1.0");
        }

        // Invariant 9: FMA precision — equals naive within 1 ulp (~1e-15).
        #[test]
        fn prop_fma_matches_naive_within_1_ulp(
            f in 0.0_f64..=1.0,
            g in 0.0_f64..=1.0,
            r in 0.0_f64..=1.0,
            p in 0.0_f64..=1.0,
        ) {
            let base = 1.0 - p;
            let cs = (f * g * r).clamp(0.0, 1.0);
            let naive = base * (1.0 - cs) + cs;
            let fma = factor(f, g, r, p);
            // 1 ulp at f64 magnitude ~1 is ~2.22e-16; allow 1e-12 for stack-up.
            prop_assert!((fma - naive).abs() < 1e-12);
        }

        // Invariant 10: base-floor — decay_factor ≥ base_rate always.
        #[test]
        fn prop_base_floor_invariant(
            f in 0.0_f64..=1.0,
            g in 0.0_f64..=1.0,
            r in 0.0_f64..=1.0,
            p in 0.0_f64..=1.0,
        ) {
            let base_rate = 1.0 - p;
            prop_assert!(factor(f, g, r, p) >= base_rate - 1e-12);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 1_000,
            .. ProptestConfig::default()
        })]

        // Invariant 2: monotone non-decreasing in frequency
        // (fix fitness/recency/p).
        #[test]
        fn prop_monotone_in_frequency(
            g in 0.0_f64..=1.0,
            r in 0.0_f64..=1.0,
            p in 0.0_f64..=1.0,
            f_lo in 0.0_f64..=0.5,
            delta in 0.0_f64..=0.5,
        ) {
            let f_hi = (f_lo + delta).min(1.0);
            let v_lo = factor(f_lo, g, r, p);
            let v_hi = factor(f_hi, g, r, p);
            prop_assert!(v_hi >= v_lo - 1e-12, "v_hi={v_hi} v_lo={v_lo}");
        }

        // Invariant 3: monotone non-decreasing in fitness.
        #[test]
        fn prop_monotone_in_fitness(
            f in 0.0_f64..=1.0,
            r in 0.0_f64..=1.0,
            p in 0.0_f64..=1.0,
            g_lo in 0.0_f64..=0.5,
            delta in 0.0_f64..=0.5,
        ) {
            let g_hi = (g_lo + delta).min(1.0);
            prop_assert!(factor(f, g_hi, r, p) >= factor(f, g_lo, r, p) - 1e-12);
        }

        // Invariant 4: monotone non-decreasing in recency.
        #[test]
        fn prop_monotone_in_recency(
            f in 0.0_f64..=1.0,
            g in 0.0_f64..=1.0,
            p in 0.0_f64..=1.0,
            r_lo in 0.0_f64..=0.5,
            delta in 0.0_f64..=0.5,
        ) {
            let r_hi = (r_lo + delta).min(1.0);
            prop_assert!(factor(f, g, r_hi, p) >= factor(f, g, r_lo, p) - 1e-12);
        }

        // Invariant 5: idempotent under recency=1.0 (degenerate
        // half_life=+inf case is upstream in inputs::recency_factor).
        #[test]
        fn prop_recency_one_is_no_op_modifier(
            f in 0.0_f64..=1.0,
            g in 0.0_f64..=1.0,
            p in 0.0_f64..=1.0,
        ) {
            // With recency=1.0, cs = f*g; value = base + (1-base)*f*g.
            let base = 1.0 - p;
            let cs = (f * g).clamp(0.0, 1.0);
            let expected = base + (1.0 - base) * cs;
            let actual = factor(f, g, 1.0, p);
            prop_assert!((actual - expected).abs() < 1e-12);
        }

        // Invariant 6: base_rate=1.0 (plain_decay_rate=0.0) → 1.0 always.
        #[test]
        fn prop_base_one_is_one_always(
            f in 0.0_f64..=1.0,
            g in 0.0_f64..=1.0,
            r in 0.0_f64..=1.0,
        ) {
            let v = factor(f, g, r, 0.0);
            prop_assert!((v - 1.0).abs() < 1e-12);
        }

        // Invariant 7: base_rate=0.0 + any axis zero → value ∈ [0,1].
        #[test]
        fn prop_zero_axis_with_zero_base_stays_in_range(
            g in 0.0_f64..=1.0,
            r in 0.0_f64..=1.0,
        ) {
            // f=0; base=0 (plain=1.0); value = 0 * (1-0) + 0 = 0.
            let v = factor(0.0, g, r, 1.0);
            prop_assert!((0.0..=1.0).contains(&v));
        }

        // Invariant 8: symmetric under timestamp-shift (recency depends
        // only on Δt). We test via the formula directly — since recency
        // is already a scalar input here, this invariant lives at the
        // `inputs::recency_factor` boundary. Here we re-confirm: the
        // formula commutes with input substitution.
        #[test]
        fn prop_symmetric_under_signal_swap_when_one_signal_is_one(
            f in 0.0_f64..=1.0,
            g in 0.0_f64..=1.0,
            p in 0.0_f64..=1.0,
        ) {
            // f*g*1 == g*f*1; symmetry on the compound_signal product.
            prop_assert!((factor(f, g, 1.0, p) - factor(g, f, 1.0, p)).abs() < 1e-12);
        }
    }

    // ====================================================================
    // W4 mutation-kill pass (S1003529) — `formula.rs:49` is the body of
    // `DecayFactor::as_f64`, a const accessor returning `self.0`. The
    // surviving mutant replaces the whole function body with `0.0`. The
    // tests below pin `as_f64()` to non-zero, distinct values so the
    // constant-`0.0` mutant fails loudly.
    // ====================================================================

    #[test]
    // rationale: Accessor mutant kill — `as_f64()` must return the EXACT
    // inner value, not the constant `0.0`. Construct a DecayFactor from a
    // value that is provably non-zero and assert round-trip identity. A
    // `-> 0.0` body mutant collapses this to 0.0 and fails the `> 0.0`
    // and the `(v - 0.75)` equality checks.
    fn as_f64_returns_inner_value_not_constant_zero() {
        let d = DecayFactor::new(0.75).expect("0.75 in range");
        let v = d.as_f64();
        assert!(
            v > 0.0,
            "as_f64() returned {v} — a `-> 0.0` body mutant collapsed the \
             accessor to a constant",
        );
        assert!(
            (v - 0.75).abs() < 1e-12,
            "as_f64() must round-trip the constructed value 0.75, got {v}",
        );
    }

    #[test]
    // rationale: Accessor mutant kill — second witness at a different
    // non-zero magnitude (0.5) AND at the upper bound 1.0. Two distinct
    // expected values guard against any constant-body mutant: a `-> 0.0`
    // mutant cannot satisfy both `== 0.5` and `== 1.0` simultaneously.
    fn as_f64_round_trips_distinct_non_zero_magnitudes() {
        let half = DecayFactor::new(0.5).expect("0.5 in range");
        assert!((half.as_f64() - 0.5).abs() < 1e-12, "0.5 round-trip");
        let one = DecayFactor::new(1.0).expect("1.0 in range");
        assert!((one.as_f64() - 1.0).abs() < 1e-12, "1.0 round-trip");
        // The two values must differ — a constant-`0.0` mutant makes them
        // both 0.0, collapsing this inequality.
        assert!(
            (half.as_f64() - one.as_f64()).abs() > 0.4,
            "as_f64() must distinguish 0.5 from 1.0; a constant-body \
             mutant makes both return the same value",
        );
    }

    #[test]
    // rationale: Accessor mutant kill via the formula path — a healthy
    // compute_decay_factor result (all signals 1.0 → factor 1.0) read
    // back through as_f64() MUST be 1.0. The `factor()` helper depends on
    // as_f64(); a `-> 0.0` mutant would make `factor(1,1,1,0.02)` return
    // 0.0 instead of 1.0.
    fn compute_decay_factor_result_read_via_as_f64_is_non_zero() {
        let df = compute_decay_factor(1.0, 1.0, 1.0, 0.02).expect("happy");
        let v = df.as_f64();
        assert!(
            (v - 1.0).abs() < 1e-12,
            "all-one signals must yield decay factor 1.0 via as_f64(), \
             got {v} — a :49 `as_f64 -> 0.0` mutant survived",
        );
    }

    // ---- F-Regression (4) -----------------------------------------------

    #[test]
    fn regression_gap_2_pinned_input_vectors() {
        // Snapshot of canonical input → output mappings. If the formula
        // shape changes, these break loudly.
        let cases = [
            (0.0_f64, 0.0_f64, 0.0_f64, 0.02_f64, 0.98_f64),
            (1.0, 1.0, 1.0, 0.02, 1.0),
            (0.5, 0.5, 0.5, 0.02, 0.9825),
            (0.0, 1.0, 1.0, 0.02, 0.98),
            (1.0, 1.0, 1.0, 0.50, 1.0),
            (0.0, 0.0, 0.0, 0.50, 0.5),
        ];
        for (f, g, r, p, expected) in cases {
            let v = factor(f, g, r, p);
            assert!(
                (v - expected).abs() < 1e-12,
                "f={f} g={g} r={r} p={p} expected={expected} got={v}"
            );
        }
    }

    #[test]
    fn regression_decay_factor_clamp_safety() {
        // If a future refactor removes the `clamp` in compute_decay_factor,
        // values outside [0,1] could leak through. The newtype DecayFactor
        // still enforces invariant at construction. This test guards the
        // construction boundary.
        for v in [-0.000_000_1, 1.000_000_1, f64::NAN] {
            assert!(DecayFactor::new(v).is_err(), "value {v} should be rejected");
        }
    }

    #[test]
    fn regression_compositional_integrity_high_freq_low_fit() {
        // F1 (bank/name ossification) regression: a workflow with high
        // frequency but zero fitness MUST decay at base_rate. Usage alone
        // never grants immortality.
        let v = factor(1.0, 0.0, 1.0, 0.02);
        assert!((v - 0.98).abs() < 1e-12);
        // Conversely, high fitness alone with zero usage also collapses
        // (the product semantics).
        let v2 = factor(0.0, 1.0, 1.0, 0.02);
        assert!((v2 - 0.98).abs() < 1e-12);
    }

    #[test]
    fn regression_thriving_workflow_no_decay() {
        // R5 mitigation: a workflow with all signals at 1.0 has decay
        // factor exactly 1.0 → multiplicative no-op → weight preserved.
        let v = factor(1.0, 1.0, 1.0, 0.02);
        assert!((v - 1.0).abs() < 1e-12);
    }
}
