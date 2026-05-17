//! `m14_habitat_outcome_lift` — Wilson 95% CI aggregator + composite lift.
//!
//! F2 (sample-size inflation) mitigation: Wilson CI returns `None` for
//! `n < MIN_SAMPLE_SIZE`. m14 surfaces that `None` outward; consumers
//! MUST treat `None` as "insufficient evidence; hold current state" —
//! never as "zero lift".

use std::time::SystemTime;

use crate::m7_workflow_runs::WorkflowRunRow;

/// Hard minimum sample size for any Wilson CI emission. Below this the
/// aggregator emits `None`.
pub const MIN_SAMPLE_SIZE: usize = 20;

/// Default rolling window size (run-count).
pub const DEFAULT_WINDOW_SIZE: usize = 120;

/// Cascade-success weight (composite formula).
pub const DEFAULT_CASCADE_WEIGHT: f64 = 0.6;

/// Cost-lift weight (composite formula).
pub const DEFAULT_COST_WEIGHT: f64 = 0.4;

/// 95% CI z-score.
pub const WILSON_Z: f64 = 1.96;

/// Workflow identifier newtype (AP30 prefix-guarded at the m13/m42 boundary).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct WorkflowId(pub String);

impl WorkflowId {
    /// Borrow the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Aggregate lift snapshot.
#[derive(Debug, Clone)]
pub struct LiftSnapshot {
    /// Composite habitat-outcome-lift; `None` below `MIN_SAMPLE_SIZE`.
    pub lift: Option<f64>,
    /// Wilson CI half-width; `Some` iff `lift` is `Some`.
    pub ci_half: Option<f64>,
    /// Number of rows in the window.
    pub n: usize,
    /// Most recent `ts_ms` in the window.
    pub latest_ts_ms: i64,
    /// Wall-clock time the snapshot was computed.
    pub computed_at: SystemTime,
}

/// Per-workflow contribution to aggregate lift.
#[derive(Debug, Clone)]
pub struct WorkflowLiftContribution {
    /// Workflow identifier.
    pub workflow_id: WorkflowId,
    /// Delta in approximately `[-1.0, +1.0]`; m31 clamps further.
    pub delta: f64,
    /// Number of runs contributing to this workflow's delta.
    pub run_count: usize,
    /// `true` iff `run_count >= MIN_SAMPLE_SIZE`.
    pub individually_significant: bool,
}

/// Errors for the lift aggregator.
#[derive(Debug, thiserror::Error)]
pub enum LiftError {
    /// Cascade/cost weights did not sum to 1.0.
    #[error("invalid weights: cascade={cascade} cost={cost} must sum to 1.0")]
    InvalidWeights {
        /// Cascade weight.
        cascade: f64,
        /// Cost weight.
        cost: f64,
    },
}

/// Aggregator config.
#[derive(Debug, Clone)]
pub struct LiftAggregatorConfig {
    /// Window size in run-count.
    pub window: usize,
    /// Cascade-success weight.
    pub cascade_weight: f64,
    /// Cost-lift weight.
    pub cost_weight: f64,
}

impl Default for LiftAggregatorConfig {
    fn default() -> Self {
        Self {
            window: DEFAULT_WINDOW_SIZE,
            cascade_weight: DEFAULT_CASCADE_WEIGHT,
            cost_weight: DEFAULT_COST_WEIGHT,
        }
    }
}

/// Wilson 95% CI half-width on a Bernoulli proportion `p` over `n` trials.
///
/// Returns `None` for `n < MIN_SAMPLE_SIZE` — the F2 hard gate.
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    reason = "n is bounded by window size << 2^53"
)]
pub fn wilson_ci_half(n_success: usize, n_total: usize) -> Option<f64> {
    if n_total < MIN_SAMPLE_SIZE {
        return None;
    }
    let p = n_success as f64 / n_total as f64;
    let n = n_total as f64;
    let z = WILSON_Z;
    let z2 = z * z;
    let half = z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / (1.0 + z2 / n);
    Some(half)
}

/// Composite cost-lift: `(baseline - actual) / baseline`, clamped to `[-1, 1]`.
#[must_use]
pub fn cost_lift(baseline: f64, actual: f64) -> f64 {
    if !baseline.is_finite() || baseline <= 0.0 || !actual.is_finite() {
        return 0.0;
    }
    ((baseline - actual) / baseline).clamp(-1.0, 1.0)
}

/// The lift aggregator.
pub struct LiftAggregator {
    config: LiftAggregatorConfig,
}

impl std::fmt::Debug for LiftAggregator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiftAggregator")
            .field("config", &self.config)
            .finish()
    }
}

impl LiftAggregator {
    /// Construct with the given config. Weights MUST sum to 1.0 ± ε.
    ///
    /// # Errors
    ///
    /// [`LiftError::InvalidWeights`] if `cascade_weight + cost_weight` is
    /// outside `[1.0 - 1e-9, 1.0 + 1e-9]`.
    pub fn new(config: LiftAggregatorConfig) -> Result<Self, LiftError> {
        let sum = config.cascade_weight + config.cost_weight;
        if (sum - 1.0).abs() > 1e-9 {
            return Err(LiftError::InvalidWeights {
                cascade: config.cascade_weight,
                cost: config.cost_weight,
            });
        }
        Ok(Self { config })
    }

    /// Borrow the config.
    #[must_use]
    pub fn config(&self) -> &LiftAggregatorConfig {
        &self.config
    }

    /// Compute a snapshot over the given rolling window of rows.
    ///
    /// Treats `outcome == "ok"` as cascade-success; uses `cost_tokens` as
    /// the cost proxy. `latest_ts_ms` is derived from the row with the
    /// largest `started_at` lexicographic value (proxy for "newest").
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        reason = "n is bounded by window size << 2^53"
    )]
    pub fn compute_snapshot(&self, rows: &[WorkflowRunRow]) -> LiftSnapshot {
        let take = self.config.window.min(rows.len());
        let window: Vec<&WorkflowRunRow> = rows.iter().take(take).collect();
        let n = window.len();
        if n < MIN_SAMPLE_SIZE {
            return LiftSnapshot {
                lift: None,
                ci_half: None,
                n,
                latest_ts_ms: 0,
                computed_at: SystemTime::now(),
            };
        }
        let successes = window
            .iter()
            .filter(|r| r.outcome.as_deref() == Some("ok"))
            .count();
        let cascade_rate = successes as f64 / n as f64;
        let costs: Vec<i64> = window.iter().filter_map(|r| r.cost_tokens).collect();
        let baseline = if costs.is_empty() {
            0.0
        } else {
            let sum: i64 = costs.iter().sum();
            sum as f64 / costs.len() as f64
        };
        // Cost-lift uses the latest row's cost vs baseline.
        let latest_cost = window.last().and_then(|r| r.cost_tokens).unwrap_or(0);
        let c_lift = cost_lift(baseline, latest_cost as f64);
        let composite = self.config.cascade_weight.mul_add(
            cascade_rate,
            self.config.cost_weight * c_lift,
        );
        let ci_half = wilson_ci_half(successes, n);
        LiftSnapshot {
            lift: Some(composite),
            ci_half,
            n,
            latest_ts_ms: 0,
            computed_at: SystemTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cost_lift, wilson_ci_half, LiftAggregator, LiftAggregatorConfig, LiftError,
        MIN_SAMPLE_SIZE, WorkflowId,
    };
    use crate::m7_workflow_runs::WorkflowRunRow;

    fn run(id: i64, outcome: &str, cost: Option<i64>) -> WorkflowRunRow {
        WorkflowRunRow {
            id,
            started_at: format!("2026-05-17T00:{:02}:00Z", id % 60),
            ended_at: Some("2026-05-17T01:00:00Z".into()),
            outcome: Some(outcome.to_owned()),
            consumer_inputs: "{}".into(),
            cost_tokens: cost,
            fitness_dimension: 0.0,
        }
    }

    #[test]
    fn min_sample_size_is_20_per_v1_3() {
        assert_eq!(MIN_SAMPLE_SIZE, 20);
    }

    #[test]
    fn wilson_below_min_sample_size_returns_none_f2_invariant() {
        for n in 0..MIN_SAMPLE_SIZE {
            assert!(wilson_ci_half(n / 2, n).is_none(), "n={n}");
        }
    }

    #[test]
    fn wilson_at_min_sample_size_returns_some() {
        assert!(wilson_ci_half(10, MIN_SAMPLE_SIZE).is_some());
    }

    #[test]
    fn wilson_ci_half_for_perfect_proportion_at_p_zero() {
        let h = wilson_ci_half(0, 100).expect("ci");
        assert!(h > 0.0 && h < 0.1);
    }

    #[test]
    fn wilson_ci_half_for_perfect_proportion_at_p_one() {
        let h = wilson_ci_half(100, 100).expect("ci");
        assert!(h > 0.0 && h < 0.1);
    }

    #[test]
    fn cost_lift_positive_when_actual_below_baseline() {
        assert!(cost_lift(100.0, 50.0) > 0.0);
    }

    #[test]
    fn cost_lift_negative_when_actual_above_baseline() {
        assert!(cost_lift(100.0, 200.0) < 0.0);
    }

    #[test]
    fn cost_lift_clamped_to_negative_one() {
        assert!(cost_lift(100.0, f64::MAX).abs() <= 1.0);
    }

    #[test]
    fn cost_lift_zero_when_baseline_invalid() {
        assert!(cost_lift(0.0, 100.0).abs() < f64::EPSILON);
        assert!(cost_lift(-50.0, 100.0).abs() < f64::EPSILON);
        assert!(cost_lift(f64::NAN, 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn aggregator_invalid_weights_rejected() {
        let err = LiftAggregator::new(LiftAggregatorConfig {
            cascade_weight: 0.5,
            cost_weight: 0.3,
            ..LiftAggregatorConfig::default()
        })
        .unwrap_err();
        assert!(matches!(err, LiftError::InvalidWeights { .. }));
    }

    #[test]
    fn aggregator_default_weights_accepted() {
        assert!(LiftAggregator::new(LiftAggregatorConfig::default()).is_ok());
    }

    #[test]
    fn compute_snapshot_below_min_returns_none_lift() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let rows: Vec<WorkflowRunRow> = (0..10).map(|i| run(i, "ok", Some(100))).collect();
        let snap = agg.compute_snapshot(&rows);
        assert!(snap.lift.is_none());
        assert!(snap.ci_half.is_none());
        assert_eq!(snap.n, 10);
    }

    #[test]
    fn compute_snapshot_at_min_returns_some_lift() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let rows: Vec<WorkflowRunRow> = (0..25).map(|i| run(i, "ok", Some(100))).collect();
        let snap = agg.compute_snapshot(&rows);
        assert!(snap.lift.is_some());
        assert!(snap.ci_half.is_some());
    }

    #[test]
    fn compute_snapshot_perfect_success_yields_high_lift() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let rows: Vec<WorkflowRunRow> = (0..30).map(|i| run(i, "ok", Some(100))).collect();
        let snap = agg.compute_snapshot(&rows);
        let lift = snap.lift.expect("lift");
        // cascade_rate=1.0, cost_lift=0 (cost==baseline). composite=0.6*1.0+0.4*0=0.6
        assert!((lift - 0.6).abs() < 1e-9);
    }

    #[test]
    fn compute_snapshot_zero_success_yields_low_lift() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let rows: Vec<WorkflowRunRow> = (0..30).map(|i| run(i, "fail", Some(100))).collect();
        let snap = agg.compute_snapshot(&rows);
        let lift = snap.lift.expect("lift");
        assert!(lift.abs() < 1e-9);
    }

    #[test]
    fn workflow_id_as_str_roundtrip() {
        let id = WorkflowId("workflow_trace_abc".into());
        assert_eq!(id.as_str(), "workflow_trace_abc");
    }

    #[test]
    fn workflow_id_serde_roundtrip() {
        let id = WorkflowId("workflow_trace_xyz".into());
        let s = serde_json::to_string(&id).expect("ser");
        let back: WorkflowId = serde_json::from_str(&s).expect("de");
        assert_eq!(back, id);
    }
}
