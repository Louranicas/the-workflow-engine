//! `m14_habitat_outcome_lift` — Wilson 95% CI aggregator + composite lift.
//!
//! F2 (sample-size inflation) mitigation: Wilson CI returns `None` for
//! `n < MIN_SAMPLE_SIZE`. m14 surfaces that `None` outward; consumers
//! MUST treat `None` as "insufficient evidence; hold current state" —
//! never as "zero lift".
//!
//! AP30 (POVM/stcortex namespace collision) is delegated to m9 — m14
//! imports the `workflow_trace` prefix constant via
//! [`crate::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX`] and
//! never hard-codes the literal.
//!
//! Spec: `ai_specs/modules/cluster-E/m14_habitat_outcome_lift.md` (§3, §4,
//! §5, §8). Hardened S1002388 (god-tier maintainer pass): F10
//! (exploration-cost-preservation) gate on `cost_lift`; explicit
//! `LiftError::InsufficientSamples` variant alongside the `Option` surface;
//! `latest_ts_ms` honoured per spec §11; rolling window evicts oldest
//! (`take(last N)`); arithmetic over `cost_tokens` uses checked addition.

use std::time::SystemTime;

use crate::m7_workflow_runs::WorkflowRunRow;
use crate::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX;

/// Hard minimum sample size for any Wilson CI emission. Below this the
/// aggregator emits `None` (the F2 gate).
pub const MIN_SAMPLE_SIZE: usize = 20;

/// Default rolling window size (run-count).
pub const DEFAULT_WINDOW_SIZE: usize = 120;

/// Cascade-success weight (composite formula).
pub const DEFAULT_CASCADE_WEIGHT: f64 = 0.6;

/// Cost-lift weight (composite formula).
pub const DEFAULT_COST_WEIGHT: f64 = 0.4;

/// 95% CI z-score.
pub const WILSON_Z: f64 = 1.96;

/// Per-workflow modulation clamp applied by m31 — surfaced as a const so
/// downstream selectors agree with the cluster spec § 7.
pub const M31_MODULATION_CLAMP: f64 = 0.3;

/// Workflow identifier newtype (AP30 prefix-guarded at the m13/m42 boundary).
///
/// The newtype is opaque on purpose: the inner `String` is **private** so
/// no caller can bypass the m9 namespace gate. Construct via either
/// [`WorkflowId::from_validated`] (when you already hold a
/// [`crate::m9_watcher_namespace_guard::ValidatedNamespace`]) or the
/// fallible [`WorkflowId::new`] (which runs the m9 validator). The inner
/// content is contractually guaranteed to carry the `workflow_trace_*`
/// prefix.
///
/// `#[serde(transparent)]` keeps the wire form identical to a bare string
/// — the encapsulation is a compile-time guarantee with zero wire cost.
/// Note: `Deserialize` reconstructs the newtype directly from the wire
/// string and does NOT re-run the m9 validator; deserialised ids are
/// trusted to the same degree as the upstream that serialised them.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct WorkflowId(String);

impl WorkflowId {
    /// Borrow the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Construct from an already-validated namespace (round-trip
    /// constructor for the m9 gate; never re-runs the validator).
    #[must_use]
    pub fn from_validated(
        v: &crate::m9_watcher_namespace_guard::ValidatedNamespace,
    ) -> Self {
        Self(v.as_str().to_owned())
    }

    /// Fallible constructor: runs the m9 namespace validator on `raw` and
    /// returns a `WorkflowId` only if the string carries the
    /// `workflow_trace_*` prefix (and is otherwise well-formed).
    ///
    /// This is the canonical entry point for callers that hold a raw,
    /// unvalidated string — it is impossible to construct a `WorkflowId`
    /// that bypasses the m9 namespace gate.
    ///
    /// # Errors
    ///
    /// Returns [`crate::m9_watcher_namespace_guard::NamespaceViolation`] if
    /// `raw` fails m9 validation (wrong prefix, empty, whitespace, control
    /// characters, etc.).
    pub fn new(
        raw: &str,
    ) -> Result<Self, crate::m9_watcher_namespace_guard::NamespaceViolation>
    {
        let validated =
            crate::m9_watcher_namespace_guard::assert_workflow_trace_namespace(raw)?;
        Ok(Self::from_validated(&validated))
    }

    /// Returns `true` iff the inner string carries the
    /// [`WORKFLOW_TRACE_NS_PREFIX`] (advisory check; m9 is the canonical
    /// gate).
    #[must_use]
    pub fn has_workflow_trace_prefix(&self) -> bool {
        self.0.starts_with(WORKFLOW_TRACE_NS_PREFIX)
    }
}

/// Aggregate lift snapshot.
#[derive(Debug, Clone)]
pub struct LiftSnapshot {
    /// Composite habitat-outcome-lift; `None` below `MIN_SAMPLE_SIZE` or
    /// when arithmetic preconditions fail.
    pub lift: Option<f64>,
    /// Wilson CI half-width; `Some` iff `lift` is `Some`.
    pub ci_half: Option<f64>,
    /// Number of rows in the window.
    pub n: usize,
    /// Most recent `ts_ms` proxy — the maximum `WorkflowRunRow::id` in the
    /// window (id is monotonic per m7's `INTEGER PRIMARY KEY
    /// AUTOINCREMENT`). Zero iff `n == 0`.
    pub latest_ts_ms: i64,
    /// Wall-clock time the snapshot was computed (AP-Hab-13 freshness).
    pub computed_at: SystemTime,
}

impl LiftSnapshot {
    /// Empty / refusal snapshot — `lift` and `ci_half` both `None`. Used
    /// when the window is below `MIN_SAMPLE_SIZE`.
    fn empty(n: usize, latest_ts_ms: i64) -> Self {
        Self {
            lift: None,
            ci_half: None,
            n,
            latest_ts_ms,
            computed_at: SystemTime::now(),
        }
    }
}

/// Per-workflow contribution to aggregate lift.
#[derive(Debug, Clone)]
pub struct WorkflowLiftContribution {
    /// Workflow identifier.
    pub workflow_id: WorkflowId,
    /// Delta in approximately `[-1.0, +1.0]`; m31 clamps further to
    /// `[-M31_MODULATION_CLAMP, +M31_MODULATION_CLAMP]`.
    pub delta: f64,
    /// Number of runs contributing to this workflow's delta.
    pub run_count: usize,
    /// `true` iff `run_count >= MIN_SAMPLE_SIZE`.
    pub individually_significant: bool,
}

/// Errors for the lift aggregator.
///
/// Variants are additive — new variants do NOT break the `Option`-typed
/// `compute_snapshot` surface. Direct callers of the fallible API see the
/// typed reason.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LiftError {
    /// Cascade/cost weights did not sum to 1.0.
    #[error("invalid weights: cascade={cascade} cost={cost} must sum to 1.0")]
    InvalidWeights {
        /// Cascade weight.
        cascade: f64,
        /// Cost weight.
        cost: f64,
    },
    /// Window contained fewer than [`MIN_SAMPLE_SIZE`] rows; the F2 gate
    /// refuses emission rather than fabricating a CI bar.
    #[error("insufficient samples: n={n} < MIN_SAMPLE_SIZE={min}")]
    InsufficientSamples {
        /// Observed sample size.
        n: usize,
        /// The F2 floor.
        min: usize,
    },
    /// Cost arithmetic was not safe (overflow, NaN, infinite, or
    /// non-positive baseline). Spec § 5: baseline must be > 0 and finite.
    #[error("cost arithmetic invalid: {reason}")]
    InvalidCostArithmetic {
        /// Human-readable failure reason.
        reason: &'static str,
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
            window: super::DEFAULT_WINDOW_SIZE,
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
    debug_assert!(
        n_success <= n_total,
        "wilson_ci_half: n_success={n_success} > n_total={n_total} (Bernoulli contract violated)"
    );
    let p = n_success as f64 / n_total as f64;
    let n = n_total as f64;
    let z = WILSON_Z;
    let z2 = z * z;
    let half = z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / (1.0 + z2 / n);
    Some(half)
}

/// Composite cost-lift: `(baseline - actual) / baseline`, clamped to
/// `[-1, 1]`.
///
/// Returns [`LiftError::InvalidCostArithmetic`] on:
///
/// - non-finite `baseline` or `actual` (NaN, ±∞),
/// - non-positive `baseline` (`baseline <= 0.0` — division-by-zero guard
///   per spec § 4),
/// - negative `actual` (cost must be non-negative per m6 contract).
///
/// This replaces the prior silent-`0.0` semantics — F10 mitigation
/// (exploration-cost-preservation collapse).
///
/// # Errors
///
/// See variants above.
pub fn cost_lift(baseline: f64, actual: f64) -> Result<f64, LiftError> {
    if !baseline.is_finite() {
        return Err(LiftError::InvalidCostArithmetic {
            reason: "baseline must be finite",
        });
    }
    if baseline <= 0.0 {
        return Err(LiftError::InvalidCostArithmetic {
            reason: "baseline must be strictly positive (division-by-zero guard)",
        });
    }
    if !actual.is_finite() {
        return Err(LiftError::InvalidCostArithmetic {
            reason: "actual must be finite",
        });
    }
    if actual < 0.0 {
        return Err(LiftError::InvalidCostArithmetic {
            reason: "actual cost must be non-negative",
        });
    }
    Ok(((baseline - actual) / baseline).clamp(-1.0, 1.0))
}

/// Compute the baseline cost (mean of `cost_tokens` over the window),
/// guarded against `i64` summation overflow and against zero-cardinality
/// cohorts.
///
/// Returns `Ok(None)` when no row in the window carries a `cost_tokens`
/// value. Returns the typed error on overflow or negative-cost.
#[allow(
    clippy::cast_precision_loss,
    reason = "window size and cost magnitudes are bounded well below 2^53"
)]
fn baseline_cost_from_window(
    window: &[&WorkflowRunRow],
) -> Result<Option<f64>, LiftError> {
    let mut sum: i64 = 0;
    let mut count: usize = 0;
    for r in window {
        if let Some(c) = r.cost_tokens {
            if c < 0 {
                return Err(LiftError::InvalidCostArithmetic {
                    reason: "negative cost_tokens in window (m6 contract violation)",
                });
            }
            sum = sum.checked_add(c).ok_or(LiftError::InvalidCostArithmetic {
                reason: "i64 overflow summing cost_tokens",
            })?;
            count += 1;
        }
    }
    if count == 0 {
        return Ok(None);
    }
    Ok(Some(sum as f64 / count as f64))
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
    /// Window discipline (spec § 6): when `rows.len() > window`, the
    /// **oldest** rows are evicted — we operate on the last `window`
    /// entries (a `VecDeque::pop_front` semantic).
    ///
    /// `latest_ts_ms` is the maximum `WorkflowRunRow::id` in the window
    /// (m7 ids are monotonic per the table's `AUTOINCREMENT`).
    ///
    /// On any arithmetic failure, the snapshot degrades to `lift=None`,
    /// `ci_half=None` and emits a `tracing::warn!`. Use
    /// [`Self::try_compute_snapshot`] for the typed-error variant.
    #[allow(
        clippy::cast_precision_loss,
        reason = "n is bounded by window size << 2^53"
    )]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "cost_tokens is non-negative i64 bounded by m6 contract"
    )]
    pub fn compute_snapshot(&self, rows: &[WorkflowRunRow]) -> LiftSnapshot {
        match self.try_compute_snapshot(rows) {
            Ok(snap) => snap,
            Err(LiftError::InsufficientSamples { n, .. }) => {
                let latest = max_id_in_window(rows, self.config.window);
                LiftSnapshot::empty(n, latest)
            }
            Err(e) => {
                tracing::warn!(
                    target: "m14.compute_snapshot",
                    error = %e,
                    "lift snapshot degraded to None on arithmetic failure"
                );
                let n = self.config.window.min(rows.len());
                let latest = max_id_in_window(rows, self.config.window);
                LiftSnapshot::empty(n, latest)
            }
        }
    }

    /// Fallible variant of [`Self::compute_snapshot`] — returns the typed
    /// reason on refusal instead of degrading to `None`.
    ///
    /// # Errors
    ///
    /// - [`LiftError::InsufficientSamples`] when window has < `MIN_SAMPLE_SIZE` rows.
    /// - [`LiftError::InvalidCostArithmetic`] on overflow or contract drift.
    #[allow(
        clippy::cast_precision_loss,
        reason = "n is bounded by window size << 2^53"
    )]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "cost_tokens is non-negative i64 bounded by m6 contract"
    )]
    pub fn try_compute_snapshot(
        &self,
        rows: &[WorkflowRunRow],
    ) -> Result<LiftSnapshot, LiftError> {
        let window: Vec<&WorkflowRunRow> = last_n(rows, self.config.window).collect();
        let n = window.len();
        let latest_ts_ms = window.iter().map(|r| r.id).max().unwrap_or(0);
        if n < MIN_SAMPLE_SIZE {
            return Err(LiftError::InsufficientSamples {
                n,
                min: MIN_SAMPLE_SIZE,
            });
        }
        let successes = window
            .iter()
            .filter(|r| r.outcome.as_deref() == Some("ok"))
            .count();
        let cascade_rate = successes as f64 / n as f64;
        let baseline = baseline_cost_from_window(&window)?;
        let c_lift = if let Some(b) = baseline {
            let latest_cost = window
                .last()
                .and_then(|r| r.cost_tokens)
                .map_or(0.0, |c| c as f64);
            if b <= 0.0 {
                0.0
            } else {
                cost_lift(b, latest_cost)?
            }
        } else {
            0.0
        };
        let composite = self
            .config
            .cascade_weight
            .mul_add(cascade_rate, self.config.cost_weight * c_lift);
        let ci_half = wilson_ci_half(successes, n);
        debug_assert!(
            ci_half.is_some(),
            "ci_half MUST be Some whenever n >= MIN_SAMPLE_SIZE (n={n})"
        );
        Ok(LiftSnapshot {
            lift: Some(composite),
            ci_half,
            n,
            latest_ts_ms,
            computed_at: SystemTime::now(),
        })
    }
}

/// Iterator over the last `n` elements of `slice`, in original order.
fn last_n<T>(slice: &[T], n: usize) -> std::slice::Iter<'_, T> {
    let len = slice.len();
    let start = len.saturating_sub(n);
    slice[start..].iter()
}

/// Maximum `id` over the trailing `window` rows; 0 when window is empty.
fn max_id_in_window(rows: &[WorkflowRunRow], window: usize) -> i64 {
    last_n(rows, window).map(|r| r.id).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        baseline_cost_from_window, cost_lift, last_n, max_id_in_window, wilson_ci_half,
        LiftAggregator, LiftAggregatorConfig, LiftError, LiftSnapshot, WorkflowId,
        M31_MODULATION_CLAMP, MIN_SAMPLE_SIZE,
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
        assert!(cost_lift(100.0, 50.0).expect("ok") > 0.0);
    }

    #[test]
    fn cost_lift_negative_when_actual_above_baseline() {
        assert!(cost_lift(100.0, 200.0).expect("ok") < 0.0);
    }

    #[test]
    fn cost_lift_clamped_to_negative_one() {
        assert!(cost_lift(100.0, f64::MAX).expect("ok").abs() <= 1.0);
    }

    #[test]
    fn cost_lift_typed_error_when_baseline_zero() {
        // rationale: Anti-property F10 — division-by-zero guard surfaces typed.
        let err = cost_lift(0.0, 100.0).unwrap_err();
        assert!(matches!(err, LiftError::InvalidCostArithmetic { .. }));
    }

    #[test]
    fn cost_lift_typed_error_when_baseline_negative() {
        // rationale: Anti-property — baseline must be strictly positive.
        let err = cost_lift(-50.0, 100.0).unwrap_err();
        assert!(matches!(err, LiftError::InvalidCostArithmetic { .. }));
    }

    #[test]
    fn cost_lift_typed_error_when_baseline_nan() {
        // rationale: Adversarial input — NaN propagates silently otherwise.
        let err = cost_lift(f64::NAN, 100.0).unwrap_err();
        assert!(matches!(err, LiftError::InvalidCostArithmetic { .. }));
    }

    #[test]
    fn cost_lift_typed_error_when_baseline_infinite() {
        // rationale: Adversarial input — +Inf baseline collapses lift to 1.0 silently.
        let err = cost_lift(f64::INFINITY, 100.0).unwrap_err();
        assert!(matches!(err, LiftError::InvalidCostArithmetic { .. }));
    }

    #[test]
    fn cost_lift_typed_error_when_actual_nan() {
        // rationale: Adversarial input — NaN actual would propagate to composite.
        let err = cost_lift(100.0, f64::NAN).unwrap_err();
        assert!(matches!(err, LiftError::InvalidCostArithmetic { .. }));
    }

    #[test]
    fn cost_lift_typed_error_when_actual_negative() {
        // rationale: Anti-property — m6 emits non-negative cost_tokens.
        let err = cost_lift(100.0, -50.0).unwrap_err();
        assert!(matches!(err, LiftError::InvalidCostArithmetic { .. }));
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
        let id = WorkflowId::new("workflow_trace_abc").expect("valid ns");
        assert_eq!(id.as_str(), "workflow_trace_abc");
    }

    #[test]
    fn workflow_id_serde_roundtrip() {
        let id = WorkflowId::new("workflow_trace_xyz").expect("valid ns");
        let s = serde_json::to_string(&id).expect("ser");
        let back: WorkflowId = serde_json::from_str(&s).expect("de");
        assert_eq!(back, id);
        // #[serde(transparent)] — the wire form is a bare JSON string.
        assert_eq!(s, "\"workflow_trace_xyz\"");
    }

    // rationale: m9-gate enforcement — WorkflowId::new rejects any string
    // outside the `workflow_trace` namespace, so the newtype cannot be
    // constructed in a way that bypasses the namespace guard.
    #[test]
    fn workflow_id_new_rejects_non_workflow_trace_namespace() {
        let err = WorkflowId::new("orac_learn")
            .expect_err("non-workflow_trace namespace must be rejected");
        assert!(matches!(
            err,
            crate::m9_watcher_namespace_guard::NamespaceViolation::WrongPrefix { .. }
        ));
        // Sanity: the happy path still constructs.
        assert!(WorkflowId::new("workflow_trace_ok").is_ok());
    }

    // ====================================================================
    // Hardening pass (S1002388) — m14 god-tier maintainer pass.
    // ====================================================================

    // rationale: Boundary — wilson_ci_half at n=MIN_SAMPLE_SIZE-1 refuses;
    // at MIN_SAMPLE_SIZE returns Some. F2 floor exact.
    #[test]
    fn wilson_boundary_n_minus_1_refuses_n_returns_some() {
        assert!(wilson_ci_half(5, MIN_SAMPLE_SIZE - 1).is_none());
        assert!(wilson_ci_half(5, MIN_SAMPLE_SIZE).is_some());
    }

    // rationale: Anti-property F2 — try_compute_snapshot at n=19 returns
    // typed InsufficientSamples (not Option::None silently).
    #[test]
    fn try_compute_snapshot_below_min_returns_typed_insufficient() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let rows: Vec<WorkflowRunRow> = (0..19).map(|i| run(i, "ok", Some(100))).collect();
        let err = agg.try_compute_snapshot(&rows).unwrap_err();
        let LiftError::InsufficientSamples { n, min } = err else {
            panic!("expected InsufficientSamples, got {err:?}");
        };
        assert_eq!(n, 19);
        assert_eq!(min, MIN_SAMPLE_SIZE);
    }

    // rationale: Anti-property F2 — try_compute_snapshot at n=20 succeeds.
    #[test]
    fn try_compute_snapshot_at_floor_succeeds() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let rows: Vec<WorkflowRunRow> =
            (0..20).map(|i| run(i, "ok", Some(100))).collect();
        let snap = agg.try_compute_snapshot(&rows).expect("ok");
        assert!(snap.lift.is_some());
        assert_eq!(snap.n, 20);
    }

    // rationale: Boundary — latest_ts_ms is the max row id in the window,
    // not zero. Spec §11 contract.
    #[test]
    fn snapshot_latest_ts_ms_is_max_row_id_in_window() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let rows: Vec<WorkflowRunRow> =
            (0..30).map(|i| run(i, "ok", Some(100))).collect();
        let snap = agg.compute_snapshot(&rows);
        assert_eq!(snap.latest_ts_ms, 29);
    }

    // rationale: Boundary — latest_ts_ms is also set on the refusal path.
    #[test]
    fn snapshot_latest_ts_ms_set_on_refusal_path() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let rows: Vec<WorkflowRunRow> = (0..5).map(|i| run(i, "ok", Some(100))).collect();
        let snap = agg.compute_snapshot(&rows);
        assert!(snap.lift.is_none());
        assert_eq!(snap.latest_ts_ms, 4);
    }

    // rationale: Contract regression — window discipline. With rows=125
    // and window=120, the window must keep the LAST 120 (ids 5..124).
    #[test]
    fn window_evicts_oldest_first_per_spec_section_6() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let rows: Vec<WorkflowRunRow> =
            (0..125).map(|i| run(i, "ok", Some(100))).collect();
        let snap = agg.compute_snapshot(&rows);
        assert_eq!(snap.n, 120);
        assert_eq!(snap.latest_ts_ms, 124);
    }

    // rationale: Anti-property — i64 overflow on cost summation MUST NOT
    // panic; returns InvalidCostArithmetic instead.
    #[test]
    fn baseline_cost_returns_overflow_error_on_i64_saturation() {
        let rows = [run(0, "ok", Some(i64::MAX)), run(1, "ok", Some(1))];
        let refs: Vec<&WorkflowRunRow> = rows.iter().collect();
        let err = baseline_cost_from_window(&refs).unwrap_err();
        assert!(matches!(err, LiftError::InvalidCostArithmetic { .. }));
    }

    // rationale: Anti-property — negative cost_tokens (m6 contract
    // violation) surfaces typed; never silently inverts lift sign.
    #[test]
    fn baseline_cost_returns_error_on_negative_cost() {
        let rows = [run(0, "ok", Some(-1))];
        let refs: Vec<&WorkflowRunRow> = rows.iter().collect();
        let err = baseline_cost_from_window(&refs).unwrap_err();
        assert!(matches!(err, LiftError::InvalidCostArithmetic { .. }));
    }

    // rationale: Boundary — zero-cardinality cohort returns Ok(None).
    #[test]
    fn baseline_cost_none_when_no_cost_tokens_in_window() {
        let rows = [run(0, "ok", None), run(1, "ok", None)];
        let row_refs: Vec<&WorkflowRunRow> = rows.iter().collect();
        let outcome = baseline_cost_from_window(&row_refs).expect("ok");
        assert!(outcome.is_none());
    }

    // rationale: Resource accounting — last_n on empty / zero-window.
    #[test]
    fn last_n_handles_empty_and_zero_window() {
        let v: Vec<i32> = vec![];
        assert_eq!(last_n(&v, 5).count(), 0);
        let v = vec![1, 2, 3];
        assert_eq!(last_n(&v, 0).count(), 0);
        assert_eq!(last_n(&v, 100).count(), 3);
    }

    // rationale: Cross-module surface invariant — WorkflowId's
    // has_workflow_trace_prefix agrees with m9 const.
    #[test]
    fn workflow_id_prefix_check_agrees_with_m9_const() {
        let good = WorkflowId::new("workflow_trace_xyz").expect("valid ns");
        // `bad` carries a non-workflow_trace prefix; it can only be built
        // by deserialising raw wire bytes (m9 gate cannot be bypassed via
        // `new`/`from_validated`). This exercises the advisory check on
        // such a deserialised value.
        let bad: WorkflowId =
            serde_json::from_str("\"orac_xyz\"").expect("de raw");
        assert!(good.has_workflow_trace_prefix());
        assert!(!bad.has_workflow_trace_prefix());
    }

    // rationale: Cross-module — from_validated round-trips through m9.
    #[test]
    fn workflow_id_from_validated_round_trip() {
        let v = crate::m9_watcher_namespace_guard::assert_workflow_trace_namespace(
            "workflow_trace_abc",
        )
        .expect("v");
        let id = WorkflowId::from_validated(&v);
        assert!(id.has_workflow_trace_prefix());
        assert_eq!(id.as_str(), "workflow_trace_abc");
    }

    // rationale: Determinism — same input yields same snapshot.
    #[test]
    fn compute_snapshot_is_deterministic_modulo_computed_at() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let rows: Vec<WorkflowRunRow> = (0..30)
            .map(|i| run(i, if i % 2 == 0 { "ok" } else { "fail" }, Some(100)))
            .collect();
        let first = agg.compute_snapshot(&rows);
        for _ in 0..100_u32 {
            let again = agg.compute_snapshot(&rows);
            assert_eq!(first.n, again.n);
            assert_eq!(first.latest_ts_ms, again.latest_ts_ms);
            let lift_first = first.lift.expect("lift");
            let lift_again = again.lift.expect("lift");
            assert!((lift_first - lift_again).abs() < 1e-12);
            let ci_first = first.ci_half.expect("ci");
            let ci_again = again.ci_half.expect("ci");
            assert!((ci_first - ci_again).abs() < 1e-12);
        }
    }

    // rationale: Adversarial input — compute_snapshot degrades to None on
    // i64 overflow without panicking.
    #[test]
    fn compute_snapshot_degrades_to_none_on_overflow() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let mut rows: Vec<WorkflowRunRow> =
            (0..20).map(|i| run(i, "ok", Some(1))).collect();
        rows.push(run(20, "ok", Some(i64::MAX)));
        let snap = agg.compute_snapshot(&rows);
        assert!(snap.lift.is_none(), "lift must degrade on overflow");
        assert!(snap.ci_half.is_none());
    }

    // rationale: Contract — M31_MODULATION_CLAMP within unit range.
    #[test]
    fn m31_modulation_clamp_within_unit_range() {
        let clamp = std::hint::black_box(M31_MODULATION_CLAMP);
        assert!(clamp > 0.0);
        assert!(clamp <= 1.0);
    }

    // rationale: Concurrency — LiftAggregator is Send + Sync.
    #[test]
    fn lift_aggregator_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LiftAggregator>();
    }

    // rationale: Contract regression — refusal snapshot n field reflects
    // input size (clamped), never silently zero.
    #[test]
    fn refusal_snapshot_n_matches_input_clamped_to_window() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let rows: Vec<WorkflowRunRow> = (0..7).map(|i| run(i, "ok", Some(50))).collect();
        let snap = agg.compute_snapshot(&rows);
        let LiftSnapshot { n, .. } = snap;
        assert_eq!(n, 7);
    }

    // rationale: Boundary — max_id_in_window on empty slice returns 0.
    #[test]
    fn max_id_in_window_zero_for_empty_input() {
        let rows: Vec<WorkflowRunRow> = vec![];
        assert_eq!(max_id_in_window(&rows, 10), 0);
    }

    // ====================================================================
    // Hardening pass 2 — +12 tests. Composite formula, weight blends,
    // cost-lift contribution, Wilson monotonicity, custom windows.
    // ====================================================================

    // rationale: Correctness — the composite formula is
    // `cascade_weight*cascade_rate + cost_weight*cost_lift`. With a 50/50
    // success split, all-equal cost (c_lift=0), default weights:
    // composite = 0.6*0.5 + 0.4*0 = 0.30.
    #[test]
    fn composite_formula_half_success_equal_cost_is_point_three() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let rows: Vec<WorkflowRunRow> = (0..30)
            .map(|i| run(i, if i % 2 == 0 { "ok" } else { "fail" }, Some(100)))
            .collect();
        let lift = agg.compute_snapshot(&rows).lift.expect("lift");
        assert!((lift - 0.30).abs() < 1e-9, "composite was {lift}");
    }

    // rationale: Correctness — custom weights blend the composite. With
    // cascade=1.0/cost=0.0 and all-ok rows, composite = 1.0*1.0 = 1.0.
    #[test]
    fn composite_with_all_cascade_weight_equals_cascade_rate() {
        let agg = LiftAggregator::new(LiftAggregatorConfig {
            window: super::DEFAULT_WINDOW_SIZE,
            cascade_weight: 1.0,
            cost_weight: 0.0,
        })
        .expect("agg");
        let rows: Vec<WorkflowRunRow> = (0..25).map(|i| run(i, "ok", Some(100))).collect();
        let lift = agg.compute_snapshot(&rows).lift.expect("lift");
        assert!((lift - 1.0).abs() < 1e-9, "all-cascade lift was {lift}");
    }

    // rationale: Correctness — cost_lift contributes to the composite.
    // With cascade=0.0/cost=1.0, the composite equals the cost-lift of
    // the latest run vs window-mean baseline. Latest run is cheaper than
    // the mean → positive composite.
    #[test]
    fn composite_with_all_cost_weight_reflects_cost_lift() {
        let agg = LiftAggregator::new(LiftAggregatorConfig {
            window: super::DEFAULT_WINDOW_SIZE,
            cascade_weight: 0.0,
            cost_weight: 1.0,
        })
        .expect("agg");
        // 24 expensive runs + 1 cheap latest run.
        let mut rows: Vec<WorkflowRunRow> =
            (0..24).map(|i| run(i, "ok", Some(1_000))).collect();
        rows.push(run(24, "ok", Some(10)));
        let lift = agg.compute_snapshot(&rows).lift.expect("lift");
        assert!(lift > 0.0, "cheaper latest run → positive cost lift: {lift}");
    }

    // rationale: Correctness — when the latest run is more expensive than
    // the window mean, the all-cost composite is negative.
    #[test]
    fn composite_negative_when_latest_run_above_baseline() {
        let agg = LiftAggregator::new(LiftAggregatorConfig {
            window: super::DEFAULT_WINDOW_SIZE,
            cascade_weight: 0.0,
            cost_weight: 1.0,
        })
        .expect("agg");
        let mut rows: Vec<WorkflowRunRow> =
            (0..24).map(|i| run(i, "ok", Some(100))).collect();
        rows.push(run(24, "ok", Some(10_000)));
        let lift = agg.compute_snapshot(&rows).lift.expect("lift");
        assert!(lift < 0.0, "expensive latest run → negative lift: {lift}");
    }

    // rationale: Boundary — baseline_cost is the arithmetic mean over
    // ONLY the costed rows; None-cost rows are skipped from the divisor.
    #[test]
    fn baseline_cost_mean_skips_none_cost_rows_in_divisor() {
        let rows = [
            run(0, "ok", Some(100)),
            run(1, "ok", None),
            run(2, "ok", Some(200)),
        ];
        let refs: Vec<&WorkflowRunRow> = rows.iter().collect();
        let mean = baseline_cost_from_window(&refs).expect("ok").expect("some");
        // Mean of {100,200} = 150, NOT (100+0+200)/3.
        assert!((mean - 150.0).abs() < 1e-9, "mean was {mean}");
    }

    // rationale: Boundary — Wilson CI half-width is widest at p=0.5 and
    // narrower at the extremes for the same n (maximum-variance point).
    #[test]
    fn wilson_ci_is_widest_at_proportion_half() {
        let n = 100;
        let at_half = wilson_ci_half(50, n).expect("half");
        let at_low = wilson_ci_half(5, n).expect("low");
        let at_high = wilson_ci_half(95, n).expect("high");
        assert!(at_half > at_low, "p=0.5 wider than p=0.05: {at_half} {at_low}");
        assert!(at_half > at_high, "p=0.5 wider than p=0.95: {at_half} {at_high}");
    }

    // rationale: Correctness — Wilson CI half-width shrinks as n grows
    // for a fixed proportion (more evidence → tighter interval).
    #[test]
    fn wilson_ci_narrows_as_sample_size_grows() {
        let small = wilson_ci_half(20, 40).expect("small");
        let large = wilson_ci_half(200, 400).expect("large");
        assert!(large < small, "larger n → tighter CI: {large} < {small}");
    }

    // rationale: Boundary — a smaller custom window evicts more rows; n
    // is capped at the window size even with many input rows.
    #[test]
    fn custom_window_caps_n_at_window_size() {
        let agg = LiftAggregator::new(LiftAggregatorConfig {
            window: 30,
            cascade_weight: 0.6,
            cost_weight: 0.4,
        })
        .expect("agg");
        let rows: Vec<WorkflowRunRow> = (0..200).map(|i| run(i, "ok", Some(100))).collect();
        let snap = agg.compute_snapshot(&rows);
        assert_eq!(snap.n, 30, "n capped at custom window size");
        assert_eq!(snap.latest_ts_ms, 199, "latest id is from the tail");
    }

    // rationale: Boundary — when the window size is smaller than
    // MIN_SAMPLE_SIZE, the aggregator can never emit a lift (the F2 gate
    // dominates the window cap).
    #[test]
    fn window_below_min_sample_always_refuses() {
        let agg = LiftAggregator::new(LiftAggregatorConfig {
            window: 10,
            cascade_weight: 0.6,
            cost_weight: 0.4,
        })
        .expect("agg");
        let rows: Vec<WorkflowRunRow> = (0..1_000).map(|i| run(i, "ok", Some(100))).collect();
        let snap = agg.compute_snapshot(&rows);
        assert!(snap.lift.is_none(), "window<MIN_SAMPLE_SIZE always refuses");
        assert_eq!(snap.n, 10);
    }

    // rationale: Correctness — config() borrows back the exact config the
    // aggregator was constructed with.
    #[test]
    fn aggregator_config_accessor_returns_construction_config() {
        let cfg = LiftAggregatorConfig {
            window: 77,
            cascade_weight: 0.25,
            cost_weight: 0.75,
        };
        let agg = LiftAggregator::new(cfg).expect("agg");
        assert_eq!(agg.config().window, 77);
        assert!((agg.config().cascade_weight - 0.25).abs() < 1e-12);
        assert!((agg.config().cost_weight - 0.75).abs() < 1e-12);
    }

    // rationale: Boundary — weights summing to 1.0 within the 1e-9
    // tolerance are accepted; just outside it are rejected.
    #[test]
    fn aggregator_weight_sum_tolerance_is_one_e_minus_nine() {
        // Inside tolerance: accepted.
        assert!(LiftAggregator::new(LiftAggregatorConfig {
            window: 120,
            cascade_weight: 0.6,
            cost_weight: 0.400_000_000_5,
        })
        .is_ok());
        // Outside tolerance: rejected.
        assert!(LiftAggregator::new(LiftAggregatorConfig {
            window: 120,
            cascade_weight: 0.6,
            cost_weight: 0.401,
        })
        .is_err());
    }

    // rationale: Anti-property — try_compute_snapshot surfaces the typed
    // overflow error rather than degrading silently (the fallible API
    // does NOT collapse to None like compute_snapshot does).
    #[test]
    fn try_compute_snapshot_surfaces_overflow_typed_not_degraded() {
        let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
        let mut rows: Vec<WorkflowRunRow> = (0..20).map(|i| run(i, "ok", Some(1))).collect();
        rows.push(run(20, "ok", Some(i64::MAX)));
        let err = agg.try_compute_snapshot(&rows).unwrap_err();
        assert!(
            matches!(err, LiftError::InvalidCostArithmetic { .. }),
            "fallible API must surface typed overflow, got {err:?}"
        );
    }
}
