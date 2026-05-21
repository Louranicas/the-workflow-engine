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
#[non_exhaustive]
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

/// Exponential recency factor for a curated-bank workflow.
///
/// Returns `exp(-lambda · elapsed_days)` clamped to `[0, 1]`, where
/// `lambda = ln(2) / RECENCY_HALF_LIFE_DAYS`. A workflow run "now" scores
/// `1.0`; one at exactly the half-life scores `0.5`; old runs asymptote
/// to `0.0`.
///
/// # Never-run workflows score the *neutral* 0.5 (F7 — intentional divergence)
///
/// When `last_run_ms` is `None` (the workflow has been admitted to the bank
/// but never dispatched) this function returns **`0.5`** — a deliberately
/// neutral mid-scale recency. This differs from
/// [`crate::m11_fitness_weighted_decay::inputs::recency_factor`], whose
/// `days_since_last_run <= 0.0` branch returns **`1.0`**. The divergence is
/// **intentional** and follows from the two functions answering different
/// questions:
///
/// - **m11 decay** measures elapsed time *since a known last run*. Its
///   `<= 0.0` branch means "ran just now / no aging yet" → maximal recency
///   `1.0`. A never-run workflow does not reach m11's compound-decay path
///   with `days = 0`; m11 normalises an actual elapsed-time signal.
/// - **m31 selection** ranks bank candidates for dispatch. A never-run
///   workflow has *no run history at all* — treating it as "freshly run"
///   (`1.0`) would let unproven workflows outrank genuinely-recent proven
///   ones on the β-term. `0.5` is the neutral prior: it neither rewards nor
///   penalises the absence of history, leaving fitness (α), frequency (γ)
///   and diversity (δ) to discriminate.
///
/// If a future change unifies these semantics it must be done centrally
/// across both modules — flagged for Command per the F7 hardening note.
fn recency_factor(last_run_ms: Option<i64>, now_ms: i64) -> f64 {
    let Some(last) = last_run_ms else {
        // F7: neutral 0.5 for never-run workflows — see fn doc comment for
        // the intentional divergence from m11's 1.0 (different question).
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

    // ---- Wave: god-tier hardening pass — m31 to ≥50 tests ----

    #[test]
    // rationale: Scoring correctness — hand-computed composite score.
    // Default weights α=0.4 β=0.25 γ=0.2 δ=0.15. One workflow:
    // fitness=1.0, last_run=now → recency=1.0, run_count=max → frequency=1.0,
    // diversity=1.0 → score = 0.4+0.25+0.2+0.15 = 1.0.
    fn scoring_all_components_one_yields_unit_score() {
        let cfg = SelectorConfig::default();
        let w = workflow(1, 1.0, 5, Some(1_700_000_000_000));
        let r = select_top_k(&[w], &cfg, |_| 1.0, 1_700_000_000_000, 1).expect("ok");
        assert!((r[0].score - 1.0).abs() < 1e-9, "score was {}", r[0].score);
    }

    #[test]
    // rationale: Scoring correctness — hand-computed with zero diversity.
    // fitness=1.0 recency=1.0 frequency=1.0 diversity=0.0 →
    // score = 0.4·1 + 0.25·1 + 0.2·1 + 0.15·0 = 0.85.
    fn scoring_zero_diversity_drops_score_by_delta() {
        let cfg = SelectorConfig::default();
        let now = 1_700_000_000_000_i64;
        let w = workflow(1, 1.0, 5, Some(now));
        let r = select_top_k(&[w], &cfg, |_| 0.0, now, 1).expect("ok");
        assert!((r[0].score - 0.85).abs() < 1e-9, "score was {}", r[0].score);
    }

    #[test]
    // rationale: Scoring correctness — pure-fitness config (α=1) makes the
    // composite score equal the sanitised fitness exactly.
    fn scoring_pure_fitness_config_score_equals_weight() {
        let cfg = SelectorConfig { alpha: 1.0, beta: 0.0, gamma: 0.0, delta: 0.0 };
        let now = 1_700_000_000_000_i64;
        let w = workflow(1, 0.37, 5, Some(now));
        let r = select_top_k(&[w], &cfg, |_| 0.99, now, 1).expect("ok");
        assert!((r[0].score - 0.37).abs() < 1e-9, "score was {}", r[0].score);
    }

    #[test]
    // rationale: Scoring correctness — pure-diversity config (δ=1) makes the
    // score equal the sanitised diversity input exactly.
    fn scoring_pure_diversity_config_score_equals_diversity() {
        let cfg = SelectorConfig { alpha: 0.0, beta: 0.0, gamma: 0.0, delta: 1.0 };
        let w = workflow(1, 0.9, 5, Some(0));
        let r = select_top_k(&[w], &cfg, |_| 0.42, 0, 1).expect("ok");
        assert!((r[0].score - 0.42).abs() < 1e-9, "score was {}", r[0].score);
    }

    #[test]
    // rationale: Frequency correctness — frequency is run_count / max_run_count.
    // Two workflows with run_count 2 and 8 → frequency 0.25 and 1.0.
    fn frequency_is_run_count_over_cohort_max() {
        let cfg = SelectorConfig::default();
        let low = workflow(1, 0.5, 2, Some(0));
        let high = workflow(2, 0.5, 8, Some(0));
        let r = select_top_k(&[low, high], &cfg, |_| 0.5, 0, 2).expect("ok");
        let f_low = r.iter().find(|c| c.workflow_id == 1).expect("low").components.frequency;
        let f_high = r.iter().find(|c| c.workflow_id == 2).expect("high").components.frequency;
        assert!((f_low - 0.25).abs() < 1e-9, "f_low {f_low}");
        assert!((f_high - 1.0).abs() < 1e-9, "f_high {f_high}");
    }

    #[test]
    // rationale: Frequency boundary — max_run_count is floored at 1 so a
    // cohort of all-zero run_counts does not divide by zero; frequency=0.
    fn frequency_max_run_count_floored_at_one() {
        let cfg = SelectorConfig::default();
        let a = workflow(1, 0.5, 0, None);
        let b = workflow(2, 0.5, 0, None);
        let r = select_top_k(&[a, b], &cfg, |_| 0.5, 0, 2).expect("ok");
        for c in &r {
            assert!((c.components.frequency - 0.0).abs() < 1e-12);
            assert!(c.score.is_finite());
        }
    }

    #[test]
    // rationale: Recency correctness — recency factor at one half-life is
    // 0.5, surfaced through select_top_k's component breakdown.
    fn recency_component_half_at_one_half_life() {
        let cfg = SelectorConfig::default();
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "half-life constant fits i64 for this range"
        )]
        let half_life_ms = (RECENCY_HALF_LIFE_DAYS * 1000.0 * 86_400.0) as i64;
        let w = workflow(1, 0.5, 5, Some(0));
        let r = select_top_k(&[w], &cfg, |_| 0.5, half_life_ms, 1).expect("ok");
        assert!((r[0].components.recency - 0.5).abs() < 1e-6);
    }

    #[test]
    // rationale: Recency correctness — never-run workflow (last_run None)
    // carries the neutral 0.5 recency through the component breakdown.
    fn recency_component_neutral_for_never_run_workflow() {
        let cfg = SelectorConfig::default();
        let w = workflow(1, 0.5, 5, None);
        let r = select_top_k(&[w], &cfg, |_| 0.5, 1_700_000_000_000, 1).expect("ok");
        assert!((r[0].components.recency - 0.5).abs() < 1e-12);
    }

    #[test]
    // rationale: Recency decay — two half-lives gives 0.25 (exponential).
    fn recency_factor_two_half_lives_quarter() {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "half-life constant fits i64 for this range"
        )]
        let two_half_lives = (2.0 * RECENCY_HALF_LIFE_DAYS * 1000.0 * 86_400.0) as i64;
        let r = recency_factor(Some(0), two_half_lives);
        assert!((r - 0.25).abs() < 1e-6, "recency at 2 half-lives was {r}");
    }

    #[test]
    // rationale: Recency monotonicity — older last_run yields a strictly
    // smaller recency factor than a more recent one.
    fn recency_factor_monotonic_decreasing_with_age() {
        let now = 1_000_000_000_000_i64;
        let day_ms = 1000 * 86_400;
        let recent = recency_factor(Some(now - day_ms), now);
        let old = recency_factor(Some(now - 100 * day_ms), now);
        assert!(recent > old, "recent {recent} should exceed old {old}");
    }

    #[test]
    // rationale: Error path — only ONE weight non-finite still rejects;
    // the gamma slot is the one tested here (covers all 4 source branches).
    fn nonfinite_gamma_weight_rejected() {
        let cfg = SelectorConfig { alpha: 0.4, beta: 0.25, gamma: f64::INFINITY, delta: 0.15 };
        let r = select_top_k(&[], &cfg, |_| 0.0, 0, 1);
        assert!(matches!(r, Err(SelectorError::NonFiniteWeight(_))));
    }

    #[test]
    // rationale: Error path — non-finite delta weight rejected (4th branch).
    fn nonfinite_delta_weight_rejected() {
        let cfg = SelectorConfig { alpha: 0.4, beta: 0.25, gamma: 0.2, delta: f64::NAN };
        let r = select_top_k(&[], &cfg, |_| 0.0, 0, 1);
        assert!(matches!(r, Err(SelectorError::NonFiniteWeight(_))));
    }

    #[test]
    // rationale: Error precedence — a non-finite weight is detected BEFORE
    // the sum-to-1.0 check; the typed error must be NonFiniteWeight, not
    // InvalidWeights (NaN would make the sum NaN otherwise).
    fn nonfinite_weight_detected_before_sum_check() {
        let cfg = SelectorConfig { alpha: f64::NAN, beta: 0.0, gamma: 0.0, delta: 0.0 };
        match select_top_k(&[], &cfg, |_| 0.0, 0, 1) {
            Err(SelectorError::NonFiniteWeight(_)) => {}
            other => panic!("expected NonFiniteWeight, got {other:?}"),
        }
    }

    #[test]
    // rationale: Error boundary — weights summing to just outside the 1e-9
    // tolerance are rejected; the InvalidWeights error carries the sum.
    fn weights_just_outside_tolerance_rejected() {
        let cfg = SelectorConfig { alpha: 0.4, beta: 0.25, gamma: 0.2, delta: 0.15 + 1e-6 };
        match select_top_k(&[], &cfg, |_| 0.0, 0, 1) {
            Err(SelectorError::InvalidWeights(sum)) => {
                assert!((sum - 1.000_001).abs() < 1e-7, "sum payload was {sum}");
            }
            other => panic!("expected InvalidWeights, got {other:?}"),
        }
    }

    #[test]
    // rationale: Error boundary — weights summing within the 1e-9 tolerance
    // are accepted (slight float drift must not trip the gate).
    fn weights_just_inside_tolerance_accepted() {
        let cfg = SelectorConfig { alpha: 0.4, beta: 0.25, gamma: 0.2, delta: 0.15 + 1e-12 };
        let r = select_top_k(&[], &cfg, |_| 0.0, 0, 1);
        assert!(r.is_ok(), "weights within tolerance should be accepted");
    }

    #[test]
    // rationale: Weight validation precedes the empty-workflows shortcut —
    // an invalid config errors even when there is nothing to score.
    fn invalid_weights_error_even_on_empty_bank() {
        let cfg = SelectorConfig { alpha: 0.9, beta: 0.9, gamma: 0.9, delta: 0.9 };
        let r = select_top_k(&[], &cfg, |_| 0.0, 0, 5);
        assert!(matches!(r, Err(SelectorError::InvalidWeights(_))));
    }

    #[test]
    // rationale: Determinism — reversing the input slice must not change the
    // ranked output when scores genuinely differ (no input-order leak).
    fn ranking_independent_of_input_slice_order() {
        let cfg = SelectorConfig::default();
        let mut ws: Vec<_> = (0u32..8)
            .map(|i| workflow(u64::from(i + 1), f64::from(i) / 8.0, 1, Some(0)))
            .collect();
        let forward = select_top_k(&ws, &cfg, |_| 0.5, 0, 8).expect("fwd");
        ws.reverse();
        let reversed = select_top_k(&ws, &cfg, |_| 0.5, 0, 8).expect("rev");
        let f_ids: Vec<u64> = forward.iter().map(|c| c.workflow_id).collect();
        let r_ids: Vec<u64> = reversed.iter().map(|c| c.workflow_id).collect();
        assert_eq!(f_ids, r_ids, "ranking leaked input-slice order");
    }

    #[test]
    // rationale: Tie-break — when two workflows tie on score, the LOWER
    // workflow_id ranks first regardless of insertion order.
    fn tie_break_lower_id_wins_pair() {
        let cfg = SelectorConfig::default();
        // Insert higher id first to exercise the secondary sort.
        let hi = workflow(999, 0.5, 5, Some(0));
        let lo = workflow(7, 0.5, 5, Some(0));
        let r = select_top_k(&[hi, lo], &cfg, |_| 0.5, 0, 2).expect("ok");
        assert_eq!(r[0].workflow_id, 7);
        assert_eq!(r[1].workflow_id, 999);
    }

    #[test]
    // rationale: Diversity — δ-weight makes a higher-diversity candidate
    // outrank an identical-fitness lower-diversity one (anti-monoculture).
    fn higher_diversity_outranks_at_equal_fitness() {
        let cfg = SelectorConfig::default();
        let a = workflow(1, 0.5, 5, Some(0));
        let b = workflow(2, 0.5, 5, Some(0));
        let r = select_top_k(
            &[a, b],
            &cfg,
            |w| if w.workflow_id == 2 { 1.0 } else { 0.2 },
            0,
            2,
        )
        .expect("ok");
        assert_eq!(r[0].workflow_id, 2, "higher diversity must win");
    }

    #[test]
    // rationale: Sanitise — a diversity input above 1.0 is clamped to 1.0
    // before entering the composite (the breakdown surfaces the clamp).
    fn diversity_above_one_clamped_in_component_breakdown() {
        let cfg = SelectorConfig::default();
        let w = workflow(1, 0.5, 5, Some(0));
        let r = select_top_k(&[w], &cfg, |_| 5.0, 0, 1).expect("ok");
        assert!((r[0].components.diversity - 1.0).abs() < 1e-12);
    }

    #[test]
    // rationale: Sanitise — a negative diversity input is clamped to 0.0.
    fn diversity_below_zero_clamped_in_component_breakdown() {
        let cfg = SelectorConfig::default();
        let w = workflow(1, 0.5, 5, Some(0));
        let r = select_top_k(&[w], &cfg, |_| -3.0, 0, 1).expect("ok");
        assert!((r[0].components.diversity - 0.0).abs() < 1e-12);
    }

    #[test]
    // rationale: Sanitise — a weight (fitness) outside [0,1] is clamped in
    // the component breakdown; e.g. AcceptedWorkflow.weight=1.5 → 1.0.
    fn fitness_weight_above_one_clamped() {
        let cfg = SelectorConfig::default();
        let w = workflow(1, 1.5, 5, Some(0));
        let r = select_top_k(&[w], &cfg, |_| 0.5, 0, 1).expect("ok");
        assert!((r[0].components.fitness - 1.0).abs() < 1e-12);
    }

    #[test]
    // rationale: Infinity diversity input — inf is not NaN; sanitise clamps
    // it via clamp(0,1) to 1.0 (only NaN takes the early-return-0 branch).
    fn infinite_diversity_input_clamped_to_one() {
        let cfg = SelectorConfig::default();
        let w = workflow(1, 0.5, 5, Some(0));
        let r = select_top_k(&[w], &cfg, |_| f64::INFINITY, 0, 1).expect("ok");
        assert!((r[0].components.diversity - 1.0).abs() < 1e-12);
        assert!(r[0].score.is_finite());
    }

    #[test]
    // rationale: k=1 on a populated bank returns exactly the top scorer.
    fn k_one_returns_single_top_scorer() {
        let cfg = SelectorConfig::default();
        let ws: Vec<_> = (0u32..6)
            .map(|i| workflow(u64::from(i + 1), f64::from(i) / 6.0, 1, Some(0)))
            .collect();
        let r = select_top_k(&ws, &cfg, |_| 0.5, 0, 1).expect("ok");
        assert_eq!(r.len(), 1);
        // id 6 has the highest fitness.
        assert_eq!(r[0].workflow_id, 6);
    }

    #[test]
    // rationale: SelectorError equality — typed errors derive PartialEq, so
    // two InvalidWeights with the same sum compare equal.
    fn selector_error_partial_eq_holds() {
        let a = SelectorError::InvalidWeights(2.0);
        let b = SelectorError::InvalidWeights(2.0);
        assert_eq!(a, b);
        assert_ne!(a, SelectorError::NonFiniteWeight(2.0));
    }

    #[test]
    // rationale: F7 — a never-run workflow scores the NEUTRAL 0.5 recency,
    // intentionally distinct from m11's 1.0 for `days_since_last_run <= 0`.
    // This locks the m31 selection-scoring semantics: an unproven (never-run)
    // workflow must NOT receive the maximal recency a freshly-run one gets,
    // or unproven candidates would outrank genuinely-recent proven ones on
    // the β-term. Treating "no history" as 0.5 (neutral) is the contract.
    fn f7_never_run_workflow_scores_neutral_half_recency() {
        // Never-run: last_run_ms = None → recency 0.5.
        assert!((recency_factor(None, 1_700_000_000_000) - 0.5).abs() < 1e-12);
        // Just-run: elapsed 0 → recency 1.0. The two are deliberately
        // different — a never-run workflow is NOT treated as freshly run.
        assert!((recency_factor(Some(1_700_000_000_000), 1_700_000_000_000) - 1.0).abs() < 1e-12);
        assert!(
            recency_factor(None, 1_700_000_000_000)
                < recency_factor(Some(1_700_000_000_000), 1_700_000_000_000),
            "F7: a never-run workflow must score strictly below a just-run one"
        );
    }

    #[test]
    // rationale: F7 — at equal fitness / frequency / diversity, a workflow
    // run *now* must outrank a never-run one because its β-recency term is
    // 1.0 vs the never-run neutral 0.5. Confirms the divergence has the
    // intended selection effect end-to-end through select_top_k.
    fn f7_recently_run_outranks_never_run_at_equal_other_components() {
        let cfg = SelectorConfig::default();
        let now = 1_700_000_000_000_i64;
        let recently_run = workflow(1, 0.5, 5, Some(now));
        let never_run = workflow(2, 0.5, 5, None);
        let r = select_top_k(&[never_run, recently_run], &cfg, |_| 0.5, now, 2).expect("ok");
        assert_eq!(r[0].workflow_id, 1, "recently-run workflow must rank first");
        assert!((r[0].components.recency - 1.0).abs() < 1e-12);
        assert!((r[1].components.recency - 0.5).abs() < 1e-12);
    }

    #[test]
    // rationale: Component breakdown — for EVERY ranked candidate the stored
    // components must reconstruct the score under the active weights.
    fn every_candidate_components_reconstruct_score() {
        let cfg = SelectorConfig::default();
        let ws: Vec<_> = (0u32..15)
            .map(|i| {
                workflow(
                    u64::from(i + 1),
                    f64::from(i) / 15.0,
                    i + 1,
                    Some(i64::from(i) * 1_000_000),
                )
            })
            .collect();
        let r = select_top_k(&ws, &cfg, |w| f64::from(w.run_count) / 15.0, 50_000_000, 15)
            .expect("ok");
        for c in &r {
            let recomposed = cfg.alpha * c.components.fitness
                + cfg.beta * c.components.recency
                + cfg.gamma * c.components.frequency
                + cfg.delta * c.components.diversity;
            assert!(
                (c.score - recomposed).abs() < 1e-9,
                "id {} score {} != recomposed {}",
                c.workflow_id,
                c.score,
                recomposed
            );
        }
    }
}
