//! Integration tests for m14 lift aggregator (Wave-C3).
//!
//! Exercises the public surface of `m14_lift` from the `workflow_core`
//! crate boundary — proves that the F2 floor (`MIN_SAMPLE_SIZE = 20`),
//! the typed `LiftError` channel from `cost_lift`, the Wave-A
//! `latest_ts_ms` direction fix, and the window-eviction (oldest-first)
//! contract all hold when consumed as an external crate would consume
//! them. The in-module `#[cfg(test)]` block tests the internals; this
//! file locks the public *integration* surface against regression.

#![allow(clippy::doc_markdown)]

use workflow_core::m14_lift::{
    cost_lift, wilson_ci_half, LiftAggregator, LiftAggregatorConfig, LiftError, MIN_SAMPLE_SIZE,
};
use workflow_core::m7_workflow_runs::{Outcome, RunState, WorkflowRunRow};

fn run(id: i64, outcome: &str, cost: Option<i64>) -> WorkflowRunRow {
    WorkflowRunRow {
        id,
        started_at: format!("2026-05-20T00:{:02}:00Z", id % 60),
        run_state: RunState::Closed {
            ended_at: "2026-05-20T01:00:00Z".into(),
            outcome: Outcome::parse(outcome).expect("test outcome must be a valid CHECK value"),
        },
        consumer_inputs: "{}".into(),
        cost_tokens: cost,
        fitness_dimension: 0.0,
    }
}

// rationale: F2 boundary — at exactly MIN_SAMPLE_SIZE - 1 the fallible
// surface MUST refuse with the typed InsufficientSamples error, not
// silently emit a None-bearing snapshot. Locks the floor against drift.
#[test]
fn m14_wilson_ci_at_min_sample_minus_one_returns_insufficient_samples() {
    // rationale: F2 boundary
    let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
    let rows: Vec<WorkflowRunRow> = (0..MIN_SAMPLE_SIZE - 1)
        .map(|i| run(i64::try_from(i).expect("fits"), "ok", Some(100)))
        .collect();
    let err = agg.try_compute_snapshot(&rows).expect_err("must refuse");
    match err {
        LiftError::InsufficientSamples { n, min } => {
            assert_eq!(n, MIN_SAMPLE_SIZE - 1);
            assert_eq!(min, MIN_SAMPLE_SIZE);
        }
        other => panic!("expected InsufficientSamples, got {other:?}"),
    }
    // wilson_ci_half itself returns None at the same boundary.
    assert!(wilson_ci_half(MIN_SAMPLE_SIZE / 2, MIN_SAMPLE_SIZE - 1).is_none());
}

// rationale: F2 boundary — at exactly MIN_SAMPLE_SIZE the fallible
// surface MUST emit a typed Ok(LiftSnapshot) with Some(lift) and
// Some(ci_half). Locks the bottom edge of the accept region.
#[test]
fn m14_wilson_ci_at_min_sample_returns_typed_ci() {
    // rationale: F2 boundary
    let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
    let rows: Vec<WorkflowRunRow> = (0..MIN_SAMPLE_SIZE)
        .map(|i| run(i64::try_from(i).expect("fits"), "ok", Some(100)))
        .collect();
    let snap = agg
        .try_compute_snapshot(&rows)
        .expect("at min must succeed");
    assert!(snap.lift.is_some(), "lift must be Some at n=MIN_SAMPLE_SIZE");
    assert!(snap.ci_half.is_some(), "ci_half must be Some at n=MIN_SAMPLE_SIZE");
    assert_eq!(snap.n, MIN_SAMPLE_SIZE);
    assert!(wilson_ci_half(MIN_SAMPLE_SIZE / 2, MIN_SAMPLE_SIZE).is_some());
}

// rationale: Adversarial — cost_lift MUST surface typed
// InvalidCostArithmetic on baseline == 0.0 (division-by-zero guard,
// spec § 4). No silent zero.
#[test]
fn m14_cost_lift_returns_typed_error_on_zero_baseline_cost() {
    // rationale: Adversarial (F10 division-by-zero guard)
    let err = cost_lift(0.0, 100.0).expect_err("zero baseline must refuse");
    assert!(
        matches!(err, LiftError::InvalidCostArithmetic { .. }),
        "expected InvalidCostArithmetic, got {err:?}"
    );
}

// rationale: Adversarial — NaN propagates silently through naive
// arithmetic; cost_lift MUST trap it with a typed error on either arg.
#[test]
fn m14_cost_lift_returns_typed_error_on_nan_input() {
    // rationale: Adversarial (NaN propagation trap)
    let err_b = cost_lift(f64::NAN, 100.0).expect_err("NaN baseline must refuse");
    assert!(matches!(err_b, LiftError::InvalidCostArithmetic { .. }));
    let err_a = cost_lift(100.0, f64::NAN).expect_err("NaN actual must refuse");
    assert!(matches!(err_a, LiftError::InvalidCostArithmetic { .. }));
}

// rationale: Contract regression (Wave-A1 window-direction fix) — when
// rows.len() > window, the aggregator MUST evict the OLDEST rows and
// retain the LAST `window` entries. A reversed-direction regression
// here would silently corrupt every downstream snapshot.
#[test]
fn m14_window_evicts_oldest_per_wave1_fix() {
    // rationale: Contract regression (window direction)
    let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
    let total = 125_usize;
    let window = LiftAggregatorConfig::default().window;
    let rows: Vec<WorkflowRunRow> = (0..total)
        .map(|i| run(i64::try_from(i).expect("fits"), "ok", Some(100)))
        .collect();
    let snap = agg.compute_snapshot(&rows);
    assert_eq!(snap.n, window, "must clamp to default window {window}");
    let expected_latest = i64::try_from(total - 1).expect("fits");
    assert_eq!(
        snap.latest_ts_ms, expected_latest,
        "window MUST keep the LAST `window` rows (Wave-A1 direction fix)"
    );
}

// rationale: Concurrency — LiftAggregator must be Send + Sync + 'static
// so callers can share it across tokio tasks. Compile-time bound check.
#[test]
fn m14_lift_aggregator_is_send_sync_static() {
    // rationale: Concurrency
    fn assert_send_sync_static<T: Send + Sync + 'static>() {}
    assert_send_sync_static::<LiftAggregator>();
}

// rationale: Contract regression (Wave-A direction fix) — `latest_ts_ms`
// MUST equal the maximum `WorkflowRunRow::id` within the active window,
// per spec § 11. A reversed-direction or min-id regression would feed
// stale snapshots into m23's F2 gate and cause it to misjudge freshness.
#[test]
fn m14_snapshot_latest_ts_ms_is_max_row_id_in_window() {
    // rationale: Contract regression (Wave-A latest_ts_ms direction)
    let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
    // Build rows whose ids are NOT in monotonic insertion order — last
    // row in the slice carries the smallest id; the aggregator must
    // still report MAX(id) within the window, not slice[-1].id.
    let mut rows: Vec<WorkflowRunRow> = (0_i64..30)
        .map(|i| run(i * 10, "ok", Some(100)))
        .collect();
    // Append a low-id row at the slice tail — ensures we are reading max(id),
    // not the last element.
    rows.push(run(3, "ok", Some(100)));
    let snap = agg.compute_snapshot(&rows);
    // The window includes the appended low-id row; max(id) in the
    // trailing window (which contains ids 0,10,20,...,290 + 3) is 290.
    assert_eq!(snap.latest_ts_ms, 290);
}

// rationale: F2 invariant — direct Wilson CI sweep across [0..MIN) all
// return None; lifting the floor would break this; lowering it would
// break the m14::tests::wilson_below_min_sample_size_returns_none case.
#[test]
fn m14_wilson_ci_returns_none_below_floor_for_full_sweep() {
    // rationale: F2 invariant (full sweep across refusal region)
    for n in 0..MIN_SAMPLE_SIZE {
        assert!(
            wilson_ci_half(n / 2, n).is_none(),
            "wilson_ci_half must refuse for n={n} < MIN_SAMPLE_SIZE={MIN_SAMPLE_SIZE}"
        );
    }
}

// rationale: Determinism — at sufficient n, repeated compute_snapshot
// calls yield identical (lift, ci_half, n, latest_ts_ms). Confirms the
// LiftSnapshot integration surface is pure modulo `computed_at`.
#[test]
fn m14_compute_snapshot_is_deterministic_modulo_clock() {
    // rationale: Determinism
    let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
    let rows: Vec<WorkflowRunRow> = (0_i64..30)
        .map(|i| {
            run(
                i,
                if i % 3 == 0 { "ok" } else { "fail" },
                Some(80 + i),
            )
        })
        .collect();
    let a = agg.compute_snapshot(&rows);
    let b = agg.compute_snapshot(&rows);
    assert_eq!(a.n, b.n);
    assert_eq!(a.latest_ts_ms, b.latest_ts_ms);
    assert!((a.lift.expect("lift") - b.lift.expect("lift")).abs() < 1e-12);
    assert!((a.ci_half.expect("ci") - b.ci_half.expect("ci")).abs() < 1e-12);
}
