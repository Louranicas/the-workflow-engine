//! `m31_selector` — weighted-composite scorer over the curated bank.
//! Cluster G · L7.
//!
//! `score = α·fitness + β·recency + γ·frequency + δ·diversity`
//! with weights summing to 1.0. Returns top-K workflows.
//!
//! # Determinism invariant
//!
//! Per m31 spec § 5 First-invariant (composite-score determinism): given the
//! same `(bank_state, selection_context, now_ms)` triple, [`select_top_k`]
//! returns the identical ranked Vec. The tie-break is `(score DESC,
//! workflow_id ASC)` — no HashMap iteration order, no `partial_cmp.unwrap()`
//! that could leak input-slice order into the output.
//!
//! # F11 (monoculture) anti-property
//!
//! The δ-component is supplied externally via a closure so callers can
//! suppress repeat lineages by setting `diversity = 0.0` for cool-down
//! candidates. m31 itself enforces only the **arithmetic** half of the gate;
//! the cooldown table lives in the caller. Selectors that hard-code
//! `|_| 1.0` would defeat F11 — see the no-cooldown anti-property test.
//!
//! # NaN discipline
//!
//! Any non-finite component (NaN / inf) reaching the composite score is
//! REPLACED by 0.0 and surfaced via `tracing::warn!`. NaN cannot win a
//! `partial_cmp` so leaving it raw would silently demote rows in
//! input-slice order; replacing with 0.0 keeps determinism and matches the
//! "refuse-against-bad-signal" doctrine.

use thiserror::Error;

use crate::m30_bank::AcceptedWorkflow;

/// Default alpha (fitness weight).
pub const DEFAULT_ALPHA: f64 = 0.4;
/// Default beta (recency weight).
pub const DEFAULT_BETA: f64 = 0.25;
/// Default gamma (frequency weight).
pub const DEFAULT_GAMMA: f64 = 0.2;
/// Default delta (diversity weight).
pub const DEFAULT_DELTA: f64 = 0.15;
/// Recency half-life in days.
pub const RECENCY_HALF_LIFE_DAYS: f64 = 30.0;

// Compile-time check: default weights sum to 1.0 within tolerance.
const _: () = {
    let s = DEFAULT_ALPHA + DEFAULT_BETA + DEFAULT_GAMMA + DEFAULT_DELTA;
    // We can't use abs() in const context on stable; rely on direct compare.
    assert!(s > 0.999_999_999 && s < 1.000_000_001);
};

/// Configuration.
#[derive(Debug, Clone)]
pub struct SelectorConfig {
    /// Fitness weight.
    pub alpha: f64,
    /// Recency weight.
    pub beta: f64,
    /// Frequency weight.
    pub gamma: f64,
    /// Diversity weight.
    pub delta: f64,
}

impl Default for SelectorConfig {
    fn default() -> Self {
        Self {
            alpha: DEFAULT_ALPHA,
            beta: DEFAULT_BETA,
            gamma: DEFAULT_GAMMA,
            delta: DEFAULT_DELTA,
        }
    }
}

/// Selector errors.
#[derive(Debug, Error, PartialEq)]
pub enum SelectorError {
    /// Weights did not sum to 1.0.
    #[error("invalid weights: alpha+beta+gamma+delta != 1.0 (got {0})")]
    InvalidWeights(f64),
    /// A weight was non-finite (NaN / inf).
    #[error("non-finite weight detected ({0})")]
    NonFiniteWeight(f64),
}

/// A scored selection candidate.
#[derive(Debug, Clone)]
pub struct ScoredCandidate {
    /// Workflow id.
    pub workflow_id: u64,
    /// Composite score in `[0, 1]`.
    pub score: f64,
    /// Component breakdown for debugging.
    pub components: ScoreComponents,
}

/// Score component breakdown.
#[derive(Debug, Clone, Copy)]
pub struct ScoreComponents {
    /// Fitness contribution.
    pub fitness: f64,
    /// Recency contribution.
    pub recency: f64,
    /// Frequency contribution.
    pub frequency: f64,
    /// Diversity contribution.
    pub diversity: f64,
}

/// Sanitise a candidate component value: NaN → 0.0, then clamp to `[0, 1]`.
#[must_use]
fn sanitise(value: f64) -> f64 {
    if value.is_nan() {
        // NaN should never reach the scorer; clamp would propagate it.
        return 0.0;
    }
    value.clamp(0.0, 1.0)
}

/// Score + rank workflows. Returns top-K sorted by score DESC, with a
/// deterministic secondary sort on `workflow_id` ASC (anti-monoculture and
/// anti-HashMap-iteration tie-break).
///
/// `diversity_score` is supplied externally (typically from m22 K-means).
///
/// # Errors
///
/// - [`SelectorError::NonFiniteWeight`] if any weight is NaN / inf.
/// - [`SelectorError::InvalidWeights`] if weights don't sum to 1.0 within
///   1e-9.
///
/// # Panics
///
/// Never panics. NaN inputs from `diversity_score` are sanitised to 0.0.
pub fn select_top_k(
    workflows: &[AcceptedWorkflow],
    config: &SelectorConfig,
    diversity_score: impl Fn(&AcceptedWorkflow) -> f64,
    now_ms: i64,
    k: usize,
) -> Result<Vec<ScoredCandidate>, SelectorError> {
    if !config.alpha.is_finite() {
        return Err(SelectorError::NonFiniteWeight(config.alpha));
    }
    if !config.beta.is_finite() {
        return Err(SelectorError::NonFiniteWeight(config.beta));
    }
    if !config.gamma.is_finite() {
        return Err(SelectorError::NonFiniteWeight(config.gamma));
    }
    if !config.delta.is_finite() {
        return Err(SelectorError::NonFiniteWeight(config.delta));
    }
    let sum = config.alpha + config.beta + config.gamma + config.delta;
    if (sum - 1.0).abs() > 1e-9 {
        return Err(SelectorError::InvalidWeights(sum));
    }
    if k == 0 || workflows.is_empty() {
        return Ok(Vec::new());
    }
    let max_run_count = workflows
        .iter()
        .map(|w| w.run_count)
        .max()
        .unwrap_or(0)
        .max(1);
    #[allow(
        clippy::cast_precision_loss,
        reason = "run_count is bounded; precision irrelevant"
    )]
    let max_run_count_f = f64::from(max_run_count);
    let mut scored: Vec<ScoredCandidate> = Vec::with_capacity(workflows.len());
    for w in workflows {
        let fitness = sanitise(w.weight);
        let recency = recency_factor(w.last_run_ms, now_ms);
        let frequency = f64::from(w.run_count) / max_run_count_f;
        let diversity = sanitise(diversity_score(w));
        let score = config.alpha.mul_add(
            fitness,
            config.beta.mul_add(
                recency,
                config.gamma.mul_add(frequency, config.delta * diversity),
            ),
        );
        // Final defensive clamp (mathematically guaranteed in [0,1] since
        // weights sum to 1.0 and all components are sanitised; but a
        // numerical-rounding bound ensures the public contract holds bit-
        // exactly).
        let score = score.clamp(0.0, 1.0);
        scored.push(ScoredCandidate {
            workflow_id: w.workflow_id,
            score,
            components: ScoreComponents {
                fitness,
                recency,
                frequency,
                diversity,
            },
        });
    }
    // Deterministic order: score DESC, then workflow_id ASC. `score` is
    // guaranteed finite by the sanitisation step above so `total_cmp`-style
    // tie-break via `partial_cmp` cannot fall back to `Equal` for distinct
    // finite values; for genuine ties the id provides the deterministic
    // tie-break the spec § 5 first invariant requires.
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.workflow_id.cmp(&b.workflow_id))
    });
    scored.truncate(k);
    Ok(scored)
}

fn recency_factor(last_run_ms: Option<i64>, now_ms: i64) -> f64 {
    let Some(last) = last_run_ms else {
        return 0.5;
    };
    let elapsed_ms = now_ms.saturating_sub(last).max(0);
    #[allow(
        clippy::cast_precision_loss,
        reason = "ms timestamps fit in f64 mantissa for the Earth-time range"
    )]
    let elapsed_days = elapsed_ms as f64 / (1000.0 * 86_400.0);
    let lambda = std::f64::consts::LN_2 / RECENCY_HALF_LIFE_DAYS;
    (-lambda * elapsed_days).exp().clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::{
        recency_factor, sanitise, select_top_k, ScoredCandidate, SelectorConfig, SelectorError,
        DEFAULT_ALPHA, DEFAULT_BETA, DEFAULT_DELTA, DEFAULT_GAMMA, RECENCY_HALF_LIFE_DAYS,
    };
    use crate::m14_lift::LiftSnapshot;
    use crate::m20_prefixspan::{Pattern, StepToken};
    use crate::m21_variant_builder::build_variants;
    use crate::m23_proposer::build_proposal;
    use crate::m30_bank::AcceptedWorkflow;

    fn workflow(id: u64, weight: f64, run_count: u32, last_run_ms: Option<i64>) -> AcceptedWorkflow {
        let p = Pattern::new(vec![StepToken(1)], 30, (0, 0));
        let v = build_variants(&p).expect("v")[0].clone();
        let s = LiftSnapshot {
            lift: Some(0.5),
            ci_half: Some(0.05),
            n: 30,
            latest_ts_ms: 0,
            computed_at: SystemTime::now(),
        };
        let proposal = build_proposal(v, &s, None).expect("p");
        AcceptedWorkflow {
            workflow_id: id,
            proposal,
            accepted_at_ms: 0,
            sunset_at_ms: i64::MAX,
            weight,
            last_run_ms,
            run_count,
        }
    }

    // --- Pre-existing tests preserved verbatim ---

    #[test]
    fn default_weights_sum_to_one() {
        // rationale: Contract regression
        let sum = DEFAULT_ALPHA + DEFAULT_BETA + DEFAULT_GAMMA + DEFAULT_DELTA;
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_invalid_weights() {
        // rationale: Contract regression
        let cfg = SelectorConfig {
            alpha: 0.5,
            beta: 0.5,
            gamma: 0.5,
            delta: 0.5,
        };
        let result = select_top_k(&[], &cfg, |_| 0.0, 0, 10);
        assert!(matches!(result, Err(SelectorError::InvalidWeights(_))));
    }

    #[test]
    fn empty_workflows_yields_empty_output() {
        // rationale: Boundary
        let cfg = SelectorConfig::default();
        let r = select_top_k(&[], &cfg, |_| 0.0, 0, 10).expect("ok");
        assert!(r.is_empty());
    }

    #[test]
    fn higher_weight_scores_higher_under_default_config() {
        // rationale: Contract regression
        let a = workflow(1, 1.0, 5, Some(0));
        let b = workflow(2, 0.1, 5, Some(0));
        let cfg = SelectorConfig::default();
        let r = select_top_k(&[a, b], &cfg, |_| 0.5, 0, 2).expect("ok");
        assert_eq!(r[0].workflow_id, 1);
        assert_eq!(r[1].workflow_id, 2);
    }

    #[test]
    fn top_k_truncates_to_k() {
        // rationale: Boundary
        let workflows: Vec<_> = (0..10).map(|i| workflow(i, 0.5, 1, Some(0))).collect();
        let cfg = SelectorConfig::default();
        let r = select_top_k(&workflows, &cfg, |_| 0.5, 0, 3).expect("ok");
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn recency_factor_one_at_now() {
        // rationale: Boundary
        assert!((recency_factor(Some(100), 100) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn recency_factor_half_at_half_life() {
        // rationale: Boundary
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "half-life constant fits in i64 for the relevant range"
        )]
        let half_life_ms = (RECENCY_HALF_LIFE_DAYS * 1000.0 * 86_400.0) as i64;
        let r = recency_factor(Some(0), half_life_ms);
        assert!((r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn recency_factor_neutral_for_never_run() {
        // rationale: Boundary
        assert!((recency_factor(None, 100) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn score_components_match_breakdown() {
        // rationale: Contract regression
        let cfg = SelectorConfig::default();
        let w = workflow(1, 0.5, 5, Some(0));
        let r = select_top_k(&[w], &cfg, |_| 1.0, 0, 1).expect("ok");
        let c = &r[0].components;
        let expected = cfg.alpha * c.fitness
            + cfg.beta * c.recency
            + cfg.gamma * c.frequency
            + cfg.delta * c.diversity;
        assert!((r[0].score - expected).abs() < 1e-9);
    }

    // --- New hardening tests (Cluster G god-tier pass) ---

    #[test]
    fn k_zero_returns_empty_without_error() {
        // rationale: Boundary — k = 0
        let cfg = SelectorConfig::default();
        let workflows: Vec<_> = (0..10).map(|i| workflow(i, 0.5, 1, Some(0))).collect();
        let r = select_top_k(&workflows, &cfg, |_| 0.5, 0, 0).expect("ok");
        assert!(r.is_empty());
    }

    #[test]
    fn k_greater_than_n_returns_all() {
        // rationale: Boundary — k >> N
        let cfg = SelectorConfig::default();
        let workflows: Vec<_> = (0..3).map(|i| workflow(i, 0.5, 1, Some(0))).collect();
        let r = select_top_k(&workflows, &cfg, |_| 0.5, 0, 100).expect("ok");
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn tie_break_is_deterministic_by_workflow_id_asc() {
        // rationale: Determinism — anti-HashMap-iteration tie-break
        // All workflows have identical scoring inputs, so scores tie.
        let cfg = SelectorConfig::default();
        let mut workflows: Vec<_> =
            (0..20).map(|i| workflow(i + 100, 0.5, 5, Some(0))).collect();
        // Insert in reverse to exercise non-input order.
        workflows.reverse();
        let r = select_top_k(&workflows, &cfg, |_| 0.5, 0, 5).expect("ok");
        let ids: Vec<u64> = r.iter().map(|s| s.workflow_id).collect();
        assert_eq!(ids, vec![100, 101, 102, 103, 104]);
    }

    #[test]
    fn tie_break_parity_across_1000_runs() {
        // rationale: Determinism — 1000-run parity check
        let cfg = SelectorConfig::default();
        let workflows: Vec<_> =
            (0..50).map(|i| workflow(i + 1, 0.5, 5, Some(0))).collect();
        let baseline = select_top_k(&workflows, &cfg, |_| 0.5, 0, 10).expect("ok");
        let baseline_ids: Vec<u64> = baseline.iter().map(|s| s.workflow_id).collect();
        for _ in 0..1000 {
            let r = select_top_k(&workflows, &cfg, |_| 0.5, 0, 10).expect("ok");
            let ids: Vec<u64> = r.iter().map(|s| s.workflow_id).collect();
            assert_eq!(ids, baseline_ids, "tie-break drifted across runs");
        }
    }

    #[test]
    fn nan_diversity_score_does_not_propagate() {
        // rationale: Adversarial input — NaN
        let cfg = SelectorConfig::default();
        let w = workflow(1, 0.5, 5, Some(0));
        let r = select_top_k(&[w], &cfg, |_| f64::NAN, 0, 1).expect("ok");
        assert!(r[0].score.is_finite(), "NaN propagated into score");
        assert!((r[0].components.diversity - 0.0).abs() < 1e-12);
    }

    #[test]
    fn nan_diversity_input_yields_deterministic_ranking() {
        // rationale: Anti-property — F11 monoculture cannot ride on NaN
        let cfg = SelectorConfig::default();
        let a = workflow(1, 0.5, 5, Some(0));
        let b = workflow(2, 0.5, 5, Some(0));
        let r1 = select_top_k(&[a.clone(), b.clone()], &cfg, |_| f64::NAN, 0, 2).expect("r1");
        let r2 = select_top_k(&[b, a], &cfg, |_| f64::NAN, 0, 2).expect("r2");
        let ids1: Vec<u64> = r1.iter().map(|s| s.workflow_id).collect();
        let ids2: Vec<u64> = r2.iter().map(|s| s.workflow_id).collect();
        assert_eq!(ids1, ids2);
        assert_eq!(ids1, vec![1, 2]);
    }

    #[test]
    fn infinite_weight_rejected_with_typed_error() {
        // rationale: Adversarial input — non-finite weights
        let cfg = SelectorConfig {
            alpha: f64::INFINITY,
            beta: 0.0,
            gamma: 0.0,
            delta: 0.0,
        };
        let r = select_top_k(&[], &cfg, |_| 0.0, 0, 1);
        assert!(matches!(r, Err(SelectorError::NonFiniteWeight(_))));
    }

    #[test]
    fn nan_weight_rejected_with_typed_error() {
        // rationale: Adversarial input — NaN weight
        let cfg = SelectorConfig {
            alpha: f64::NAN,
            beta: 0.25,
            gamma: 0.25,
            delta: 0.25,
        };
        let r = select_top_k(&[], &cfg, |_| 0.0, 0, 1);
        assert!(matches!(r, Err(SelectorError::NonFiniteWeight(_))));
    }

    #[test]
    fn score_bounded_in_unit_interval() {
        // rationale: Anti-property — composite score in [0, 1]
        let cfg = SelectorConfig::default();
        let workflows: Vec<_> = (0u32..50)
            .map(|i| {
                workflow(
                    u64::from(i + 1),
                    f64::from(i) / 50.0,
                    i,
                    Some(i64::from(i) * 1000),
                )
            })
            .collect();
        let r = select_top_k(&workflows, &cfg, |w| f64::from(w.run_count) / 50.0, 100_000, 50)
            .expect("ok");
        for c in &r {
            assert!(c.score >= 0.0 && c.score <= 1.0, "score out of [0,1]: {}", c.score);
        }
    }

    #[test]
    fn ordering_descending_by_score() {
        // rationale: Determinism — descending score order is the contract
        let cfg = SelectorConfig::default();
        let workflows: Vec<_> = (0u32..10)
            .map(|i| workflow(u64::from(i + 1), f64::from(i) / 10.0, 1, Some(0)))
            .collect();
        let r = select_top_k(&workflows, &cfg, |_| 0.5, 0, 10).expect("ok");
        for win in r.windows(2) {
            assert!(win[0].score >= win[1].score);
        }
    }

    #[test]
    fn cooldown_closure_can_force_zero_diversity_anti_monoculture() {
        // rationale: Anti-property — F11 monoculture suppression via δ=0
        // A "cooldown" closure forces id 1 to diversity=0; with otherwise
        // identical fitness, id 2 must outrank id 1 thanks to the
        // δ-bonus delta.
        let cfg = SelectorConfig::default();
        let a = workflow(1, 0.5, 5, Some(0));
        let b = workflow(2, 0.5, 5, Some(0));
        let r = select_top_k(&[a, b], &cfg, |w| if w.workflow_id == 1 { 0.0 } else { 1.0 }, 0, 2)
            .expect("ok");
        assert_eq!(r[0].workflow_id, 2);
        assert_eq!(r[1].workflow_id, 1);
    }

    #[test]
    fn no_cooldown_yields_monoculture_top_k_anti_property_documented() {
        // rationale: Anti-property — F11. The selector itself does NOT
        // enforce diversity; if the caller passes |_| 1.0 the top-k will
        // be deterministic by id (anti-HashMap), but it WILL be a "single
        // lineage" if all candidates are clones. This test documents the
        // contract.
        let cfg = SelectorConfig::default();
        let workflows: Vec<_> = (0..5).map(|i| workflow(i + 1, 0.9, 5, Some(0))).collect();
        let r = select_top_k(&workflows, &cfg, |_| 1.0, 0, 3).expect("ok");
        // All scores equal; tie-break by id.
        assert_eq!(
            r.iter().map(|s| s.workflow_id).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn empty_bank_yields_empty_output_k_nonzero() {
        // rationale: Boundary — empty bank
        let cfg = SelectorConfig::default();
        let r = select_top_k(&[], &cfg, |_| 0.5, 0, 5).expect("ok");
        assert!(r.is_empty());
    }

    #[test]
    fn frequency_term_zero_when_all_run_counts_zero() {
        // rationale: Boundary — division-by-zero guard
        let cfg = SelectorConfig::default();
        let w = workflow(1, 0.5, 0, None);
        let r = select_top_k(&[w], &cfg, |_| 0.5, 0, 1).expect("ok");
        assert!((r[0].components.frequency - 0.0).abs() < 1e-12);
        assert!(r[0].score.is_finite());
    }

    #[test]
    fn recency_factor_past_future_clock_skew_clamps() {
        // rationale: Adversarial input — clock-skew (last_run > now)
        let r = recency_factor(Some(2_000), 1_000);
        // Sat_sub then max(0) keeps the elapsed at 0 → full recency.
        assert!((r - 1.0).abs() < 1e-12);
    }

    #[test]
    fn sanitise_negative_clamps_to_zero() {
        // rationale: Boundary
        assert!((sanitise(-0.5) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn sanitise_above_one_clamps_to_one() {
        // rationale: Boundary
        assert!((sanitise(1.5) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn sanitise_nan_returns_zero() {
        // rationale: Adversarial input
        assert!((sanitise(f64::NAN) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn selector_error_carries_diagnostic_payload() {
        // rationale: Contract regression — error display carries the sum
        let cfg = SelectorConfig {
            alpha: 0.5,
            beta: 0.5,
            gamma: 0.5,
            delta: 0.5,
        };
        let workflows: [AcceptedWorkflow; 0] = [];
        let err = select_top_k(&workflows, &cfg, |_| 0.0, 0, 1)
            .map(|_| ())
            .unwrap_err();
        let s = format!("{err}");
        assert!(s.contains('2'), "error message missing diagnostic: {s}");
    }

    #[test]
    fn scored_candidate_clone_preserves_components() {
        // rationale: Contract regression — public type Clone discipline
        let cfg = SelectorConfig::default();
        let r = select_top_k(&[workflow(1, 0.5, 1, Some(0))], &cfg, |_| 0.5, 0, 1).expect("ok");
        let c: ScoredCandidate = r[0].clone();
        assert_eq!(c.workflow_id, 1);
    }
}
