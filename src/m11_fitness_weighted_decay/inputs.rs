//! Signal normalisations for the Gap 2 compound decay formula.
//!
//! Per m11 spec § 5.2: three independently-tracked signals are each
//! normalised to the unit interval `[0, 1]` before composition in
//! [`super::formula::compute_decay_factor`]:
//!
//! - [`recency_factor`] — `exp(-lambda × days_since_last_run)`, half-life
//!   parameterised. Read from m7 `last_run_at`.
//! - [`frequency_factor`] — `run_count / cohort_max` clamped. Read from m14
//!   `evidence_aggregator`.
//! - [`fitness_factor`] — defensive clamp of stcortex `pathway.weight` which
//!   is already in `[0, 1]` by stcortex invariant. Read via m42 substrate
//!   route (post-2026-05-17 ADR).

/// Exponential recency factor: `exp(-lambda × days_since_last_run)`
/// where `lambda = ln(2) / half_life_days`.
///
/// At `days_since_last_run = 0` returns `1.0`; at
/// `days_since_last_run = half_life_days` returns `0.5`; at large `days`
/// asymptotes to `0.0`. Output clamped to `[0.0, 1.0]`.
///
/// `half_life_days = +inf` is the degenerate case → recency always `1.0`
/// (no time decay). `half_life_days <= 0.0` is treated the same as `+inf`
/// to avoid `lambda = inf` cascading into NaN.
///
/// # F7 — divergence from `m31_selector::recency_factor` is intentional
///
/// m31's selection-scoring `recency_factor` returns `0.5` for a workflow
/// that has **never run** (`last_run_ms == None`) — a neutral prior so an
/// unproven workflow is neither rewarded nor penalised when competing for
/// dispatch slots. This function does NOT model "never run": its input is
/// `days_since_last_run`, an *elapsed* quantity, and a value of `0.0` means
/// "ran today / just now" — which correctly earns full recency credit
/// (`1.0`, i.e. no time decay) inside the **decay** formula. The two
/// `0.5` vs `1.0` values answer different questions on different input
/// domains (`last_run_ms = None` ≠ `days_since_last_run = 0.0`) and must
/// not be reconciled. The consolidation cycle's clock-skew gate
/// ([`super::consolidation::run_consolidation_cycle`] Step 0) filters
/// future-dated `last_run_ms` before this function ever sees a negative
/// elapsed; `AcceptedWorkflowDecay::last_run_ms` is a non-`Option` `i64`,
/// so m11 has no "never run" sentinel to handle here.
#[must_use]
pub fn recency_factor(days_since_last_run: f64, half_life_days: f64) -> f64 {
    // F7: days == 0.0 → "ran today" → full recency credit (1.0). This is
    // NOT the same as m31's never-run prior (0.5); see the doc above.
    if !days_since_last_run.is_finite() || days_since_last_run <= 0.0 {
        return 1.0;
    }
    if !half_life_days.is_finite() || half_life_days <= 0.0 {
        return 1.0;
    }
    let lambda = std::f64::consts::LN_2 / half_life_days;
    let raw = (-lambda * days_since_last_run).exp();
    raw.clamp(0.0, 1.0)
}

/// Cohort-normalised frequency factor.
///
/// `run_count / cohort_max` clamped to `[0.0, 1.0]`. `cohort_max == 0`
/// returns `0.0` (no dispatches yet → no frequency signal).
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    reason = "u64 → f64 cast is safe for workflow run counts < 2^53; \
              precision irrelevant for ratio normalisation"
)]
pub fn frequency_factor(run_count: u64, cohort_max: u64) -> f64 {
    if cohort_max == 0 {
        return 0.0;
    }
    (run_count as f64 / cohort_max as f64).clamp(0.0, 1.0)
}

/// Defensive clamp of stcortex pathway weight to `[0.0, 1.0]`.
///
/// stcortex `pathway.weight` is in `[0, 1]` by substrate invariant; this
/// clamp is defense-in-depth against substrate-side drift or NaN.
#[must_use]
pub fn fitness_factor(pathway_weight: f64) -> f64 {
    if !pathway_weight.is_finite() {
        return 0.0;
    }
    pathway_weight.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{fitness_factor, frequency_factor, recency_factor};

    // ---- recency_factor (5) ---------------------------------------------

    #[test]
    fn recency_zero_days_is_one() {
        let r = recency_factor(0.0, 30.0);
        assert!((r - 1.0).abs() < 1e-12);
    }

    #[test]
    fn recency_one_half_life_is_one_half() {
        let r = recency_factor(30.0, 30.0);
        assert!((r - 0.5).abs() < 1e-12);
    }

    #[test]
    fn recency_six_half_lives_is_about_two_to_neg_six() {
        // 180d @ half-life 30d = 6 half-lives → 2^-6 ≈ 0.015625
        let r = recency_factor(180.0, 30.0);
        assert!((r - 0.015_625).abs() < 1e-6);
    }

    #[test]
    fn recency_infinite_half_life_returns_one() {
        assert!((recency_factor(100.0, f64::INFINITY) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn recency_non_finite_or_negative_days_returns_one() {
        assert!((recency_factor(f64::NAN, 30.0) - 1.0).abs() < 1e-12);
        assert!((recency_factor(-1.0, 30.0) - 1.0).abs() < 1e-12);
        assert!((recency_factor(-1000.0, 30.0) - 1.0).abs() < 1e-12);
    }

    // ---- frequency_factor (4) -------------------------------------------

    #[test]
    fn frequency_cohort_zero_returns_zero() {
        assert!(frequency_factor(5, 0).abs() < 1e-12);
    }

    #[test]
    fn frequency_at_cohort_max_is_one() {
        assert!((frequency_factor(100, 100) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn frequency_half_cohort_max_is_one_half() {
        assert!((frequency_factor(50, 100) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn frequency_zero_run_count_is_zero() {
        assert!(frequency_factor(0, 100).abs() < 1e-12);
    }

    // ---- fitness_factor (3) ---------------------------------------------

    #[test]
    fn fitness_in_range_pass_through() {
        let v = std::hint::black_box(0.42_f64);
        assert!((fitness_factor(v) - 0.42).abs() < 1e-12);
    }

    #[test]
    fn fitness_clamps_above_one() {
        assert!((fitness_factor(2.5) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn fitness_clamps_below_zero_or_nan() {
        assert!(fitness_factor(-0.5).abs() < 1e-12);
        assert!(fitness_factor(f64::NAN).abs() < 1e-12);
        assert!(fitness_factor(f64::NEG_INFINITY).abs() < 1e-12);
    }

    // ====================================================================
    // W4 FINAL mutation-kill pass (S1003529) — `recency_factor` survivors.
    //
    // recency_factor's two early-return guards:
    //   if !days_since_last_run.is_finite() || days_since_last_run <= 0.0  // L46
    //       { return 1.0; }
    //   if !half_life_days.is_finite() || half_life_days <= 0.0            // L49
    //       { return 1.0; }
    //
    // The pre-existing tests only probed the early-return SHAPE (days=0,
    // NaN-days, negative-days, infinite-half-life) — all of which return
    // 1.0 under BOTH the real guard and several mutated guards, so the
    // mutants survive. The kill discriminator is a NORMAL, finite,
    // positive input (`days = 30`, `half_life = 30`) which the real code
    // must carry past BOTH guards into the exponential, yielding exactly
    // 0.5 — a value no early-return-1.0 mutant can produce.
    // ====================================================================

    // rationale: KILLS `inputs.rs:46:8` delete `!` in the L46 guard.
    // Real guard: `if !days.is_finite() || days <= 0.0`. For days=30
    // (finite, positive): `!is_finite(30)`=false, `30<=0`=false -> guard
    // false -> falls through -> exp -> returns 0.5.
    // `!`-deletion mutant: `if days.is_finite() || days <= 0.0` ->
    // `is_finite(30)`=true -> guard true -> `return 1.0` (WRONG).
    //
    // ALSO KILLS `inputs.rs:46:64` `<=` -> `>`.
    // Real guard's second clause `days <= 0.0`: for days=30 it is false.
    // `>` mutant: `days > 0.0` -> `30 > 0` = true -> guard true ->
    // `return 1.0` (WRONG). Real code returns 0.5.
    #[test]
    fn mutkill_46_final_finite_positive_days_falls_through_to_exp() {
        // One half-life of elapsed time => exactly 0.5. A guard that
        // wrongly early-returns 1.0 (either the `!`-deletion or the
        // `<=`->`>` mutant) fails this exact-value assertion.
        let r = recency_factor(30.0, 30.0);
        assert!(
            (r - 0.5).abs() < 1e-12,
            "30 days at a 30-day half-life must decay to exactly 0.5; \
             an L46-guard mutant early-returns 1.0 instead, got {r}"
        );
        // Second anchor: a non-half-life positive elapsed must ALSO reach
        // the exponential (not 1.0). 60 days @ 30-day half-life => 0.25.
        let r2 = recency_factor(60.0, 30.0);
        assert!(
            (r2 - 0.25).abs() < 1e-12,
            "60 days at a 30-day half-life must decay to 0.25, got {r2}"
        );
    }

    // rationale: KILLS `inputs.rs:49:8` delete `!` in the L49 guard.
    // Real guard: `if !half_life.is_finite() || half_life <= 0.0`. For
    // half_life=30 (finite, positive): `!is_finite(30)`=false,
    // `30<=0`=false -> guard false -> falls through -> exp -> 0.5.
    // `!`-deletion mutant: `if half_life.is_finite() || ...` ->
    // `is_finite(30)`=true -> guard true -> `return 1.0` (WRONG).
    #[test]
    fn mutkill_49_8_final_finite_positive_half_life_falls_through_to_exp() {
        let r = recency_factor(30.0, 30.0);
        assert!(
            (r - 0.5).abs() < 1e-12,
            "a finite positive half-life must NOT trigger the L49 early \
             return; deleting `!` makes it early-return 1.0, got {r}"
        );
    }

    // rationale: KILLS `inputs.rs:49:36` `||` -> `&&` in the L49 guard.
    // Real guard: `if !half_life.is_finite() || half_life <= 0.0` returns
    // 1.0 when half_life is non-finite (NaN/±inf) OR non-positive.
    // The `&&` mutant returns 1.0 only when half_life is non-finite AND
    // non-positive. The discriminating input is `half_life = NaN` with a
    // finite positive `days`:
    //   real: `!is_finite(NaN)`=true -> guard true -> `return 1.0`.
    //   `&&` mutant: `!is_finite(NaN)`(true) && `NaN <= 0.0`(false, all
    //     NaN comparisons are false) = false -> guard false -> falls
    //     through -> `lambda = LN_2 / NaN = NaN` -> `raw = exp(NaN) =
    //     NaN` -> `NaN.clamp(0.0, 1.0) = NaN`.
    // So real=1.0, mutant=NaN. `(NaN - 1.0).abs() < 1e-12` is false, so
    // the mutant fails this assertion. (`f64::clamp` returns NaN when
    // `self` is NaN — verified.)
    #[test]
    fn mutkill_49_36_final_nan_half_life_short_circuits_to_one_not_nan() {
        let r = recency_factor(30.0, f64::NAN);
        assert!(
            r.is_finite(),
            "NaN half-life must hit the L49 early return (1.0), not fall \
             through to a NaN exponential; an `||`->`&&` mutant yields NaN"
        );
        assert!(
            (r - 1.0).abs() < 1e-12,
            "NaN half-life => degenerate no-decay => exactly 1.0, got {r}"
        );
        // Symmetric guard: a NON-positive but FINITE half-life must ALSO
        // early-return 1.0 (the other `||` arm). Under `&&` this input
        // (`!is_finite(-5)`=false) would fall through; with a negative
        // half-life `lambda` is negative so `raw = exp(positive)` > 1 and
        // clamps to 1.0 — coincidentally still 1.0, so this arm alone is
        // not discriminating; the NaN case above is the true killer.
        let neg = recency_factor(30.0, -5.0);
        assert!((neg - 1.0).abs() < 1e-12, "non-positive half-life => 1.0");
    }
}
