//! `m31_selector` — weighted-composite scorer over the curated bank.
//! Cluster G · L7.
//!
//! `score = α·fitness + β·recency + γ·frequency + δ·diversity`
//! with weights summing to 1.0. Returns top-K workflows.

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
#[derive(Debug, Error)]
pub enum SelectorError {
    /// Weights did not sum to 1.0.
    #[error("invalid weights: alpha+beta+gamma+delta != 1.0")]
    InvalidWeights,
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

/// Score + rank workflows. Returns top-K sorted by score DESC.
///
/// `diversity_score` is supplied externally (typically from m22 K-means).
///
/// # Errors
///
/// [`SelectorError::InvalidWeights`] if weights don't sum to 1.0.
pub fn select_top_k(
    workflows: &[AcceptedWorkflow],
    config: &SelectorConfig,
    diversity_score: impl Fn(&AcceptedWorkflow) -> f64,
    now_ms: i64,
    k: usize,
) -> Result<Vec<ScoredCandidate>, SelectorError> {
    let sum = config.alpha + config.beta + config.gamma + config.delta;
    if (sum - 1.0).abs() > 1e-9 {
        return Err(SelectorError::InvalidWeights);
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
    let mut scored: Vec<ScoredCandidate> = workflows
        .iter()
        .map(|w| {
            let fitness = w.weight.clamp(0.0, 1.0);
            let recency = recency_factor(w.last_run_ms, now_ms);
            let frequency = f64::from(w.run_count) / max_run_count_f;
            let diversity = diversity_score(w).clamp(0.0, 1.0);
            let score = config.alpha.mul_add(
                fitness,
                config.beta.mul_add(
                    recency,
                    config.gamma.mul_add(frequency, config.delta * diversity),
                ),
            );
            ScoredCandidate {
                workflow_id: w.workflow_id,
                score,
                components: ScoreComponents {
                    fitness,
                    recency,
                    frequency,
                    diversity,
                },
            }
        })
        .collect();
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    Ok(scored)
}

fn recency_factor(last_run_ms: Option<i64>, now_ms: i64) -> f64 {
    let Some(last) = last_run_ms else {
        return 0.5; // Mid-recency for never-run workflows (neutral prior).
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
        recency_factor, select_top_k, SelectorConfig, SelectorError, DEFAULT_ALPHA,
        DEFAULT_BETA, DEFAULT_DELTA, DEFAULT_GAMMA,
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

    #[test]
    fn default_weights_sum_to_one() {
        let sum = DEFAULT_ALPHA + DEFAULT_BETA + DEFAULT_GAMMA + DEFAULT_DELTA;
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_invalid_weights() {
        let cfg = SelectorConfig {
            alpha: 0.5,
            beta: 0.5,
            gamma: 0.5,
            delta: 0.5,
        };
        let result = select_top_k(&[], &cfg, |_| 0.0, 0, 10);
        assert!(matches!(result, Err(SelectorError::InvalidWeights)));
    }

    #[test]
    fn empty_workflows_yields_empty_output() {
        let cfg = SelectorConfig::default();
        let r = select_top_k(&[], &cfg, |_| 0.0, 0, 10).expect("ok");
        assert!(r.is_empty());
    }

    #[test]
    fn higher_weight_scores_higher_under_default_config() {
        let a = workflow(1, 1.0, 5, Some(0));
        let b = workflow(2, 0.1, 5, Some(0));
        let cfg = SelectorConfig::default();
        let r = select_top_k(&[a, b], &cfg, |_| 0.5, 0, 2).expect("ok");
        assert_eq!(r[0].workflow_id, 1);
        assert_eq!(r[1].workflow_id, 2);
    }

    #[test]
    fn top_k_truncates_to_k() {
        let workflows: Vec<_> = (0..10).map(|i| workflow(i, 0.5, 1, Some(0))).collect();
        let cfg = SelectorConfig::default();
        let r = select_top_k(&workflows, &cfg, |_| 0.5, 0, 3).expect("ok");
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn recency_factor_one_at_now() {
        assert!((recency_factor(Some(100), 100) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn recency_factor_half_at_half_life() {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "half-life constant fits in i64 for the relevant range"
        )]
        let half_life_ms = (super::RECENCY_HALF_LIFE_DAYS * 1000.0 * 86_400.0) as i64;
        let r = recency_factor(Some(0), half_life_ms);
        assert!((r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn recency_factor_neutral_for_never_run() {
        assert!((recency_factor(None, 100) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn score_components_match_breakdown() {
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
}
