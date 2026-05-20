//! Integration tests for m6 context_cost (Wave-C2).
//!
//! Exercises the m6 public surface from outside the crate:
//!
//! - Adversarial — `classify` refuses non-finite EMA (NaN / Inf).
//! - Division-by-zero guard — non-positive EMA refuses to classify.
//! - F10 invariant — Converged-only burst followed by one Explored
//!   sample collapses to the Explored value (Converged never contributed).
//! - F10 invariant — Repeated never contributes to the EMA regardless of
//!   sample magnitude.
//! - Concurrency — `record_and_update_baseline` is atomic across N
//!   threads (no lost updates; final n matches threads × per_thread).
//! - Contract regression — `CostBand` enum has exactly three variants.

#![allow(clippy::doc_markdown)]

use std::sync::Arc;
use std::thread;

use workflow_core::m6_cost::{
    ContextCostRecord, ContextCostRecordConfig, CostBand, ExplorationBaseline, SessionCostRecord,
    WorkflowOutcome,
};

fn now_ms_for_test() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}

fn cost(session: &str, total: i64, outcome: Option<WorkflowOutcome>) -> SessionCostRecord {
    SessionCostRecord {
        session_id: session.to_owned(),
        token_cost_input_proxy: total / 2,
        token_cost_output_proxy: total - total / 2,
        total_cost_proxy: total,
        outcome,
        exploration_baseline: None,
        cost_band: None,
        recorded_at_ms: now_ms_for_test(),
    }
}

// rationale: Adversarial — `classify` MUST refuse to operate on a
// non-finite EMA (NaN / Inf would otherwise propagate into the band
// boundary multiplications and produce ambiguous classifications).
#[test]
fn m6_classify_rejects_non_finite_ema() {
    let mut b = ExplorationBaseline::new(20);
    for _ in 0..6_u32 {
        b.update(100, WorkflowOutcome::Explored);
    }
    // Inject NaN — only possible via direct field mutation; this pins
    // the guard against future bugs that could let NaN enter via the
    // sample path.
    b.ema = Some(f64::NAN);
    assert!(
        b.classify(100, 5, 0.8, 1.2).is_none(),
        "NaN EMA must refuse classify",
    );
    b.ema = Some(f64::INFINITY);
    assert!(
        b.classify(100, 5, 0.8, 1.2).is_none(),
        "Inf EMA must refuse classify",
    );
    b.ema = Some(f64::NEG_INFINITY);
    assert!(
        b.classify(100, 5, 0.8, 1.2).is_none(),
        "-Inf EMA must refuse classify",
    );
}

// rationale: Division-by-zero guard — pre-CR-2 a zero EMA would classify
// cost=0 as `AboveBaseline` since `0 >= 0 * 1.2` is `true`. Post-CR-2
// the classify path refuses non-positive EMA outright.
#[test]
fn m6_classify_rejects_zero_or_negative_ema() {
    let mut b = ExplorationBaseline::new(20);
    for _ in 0..6_u32 {
        b.update(0, WorkflowOutcome::Explored);
    }
    // EMA = 0.0 from a converged-to-zero exploration cohort.
    assert!(
        b.classify(0, 5, 0.8, 1.2).is_none(),
        "zero EMA must refuse classify",
    );
    b.ema = Some(-1.0);
    assert!(
        b.classify(0, 5, 0.8, 1.2).is_none(),
        "negative EMA must refuse classify",
    );
}

// rationale: F10 invariant — a burst of `Converged` followed by ONE
// `Explored` sample collapses to the Explored value (Converged never
// contributed to the EMA). Exploitation must NOT pull the exploration
// baseline.
#[test]
fn m6_burst_converged_then_one_explored_yields_explored_baseline() {
    let mut b = ExplorationBaseline::new(20);
    for _ in 0..500_u32 {
        b.update(999_999, WorkflowOutcome::Converged);
    }
    b.update(42, WorkflowOutcome::Explored);
    assert_eq!(b.ema, Some(42.0), "F10: Converged must not contribute");
    assert_eq!(b.n, 1);
}

// rationale: F10 invariant — `Repeated` is the symmetric exploitation
// outcome; it must never contribute regardless of cost magnitude.
#[test]
fn m6_repeated_outcomes_never_contribute_to_ema() {
    let mut b = ExplorationBaseline::new(20);
    b.update(i64::MAX, WorkflowOutcome::Repeated);
    b.update(i64::MIN, WorkflowOutcome::Repeated);
    b.update(0, WorkflowOutcome::Repeated);
    assert!(b.ema.is_none(), "F10: Repeated must not move EMA");
    assert_eq!(b.n, 0);
    // Round-trip — one Explored sample bootstraps a clean EMA.
    b.update(50, WorkflowOutcome::Explored);
    assert_eq!(b.ema, Some(50.0));
    assert_eq!(b.n, 1);
}

// rationale: Concurrency — `record_and_update_baseline` is atomic
// across threads (post-hardening). N threads contributing N samples
// each must yield exactly N*N exploration counts (no lost updates).
#[test]
fn m6_record_and_update_baseline_is_atomic_under_concurrent_writers() {
    let r = Arc::new(ContextCostRecord::new(ContextCostRecordConfig::default()));
    let threads = 8_u32;
    let per_thread = 50_u32;
    let mut handles = Vec::with_capacity(threads as usize);
    for t in 0..threads {
        let r2 = Arc::clone(&r);
        handles.push(thread::spawn(move || {
            for i in 0..per_thread {
                let _ = r2.record_and_update_baseline(cost(
                    &format!("t{t}s{i}"),
                    100,
                    Some(WorkflowOutcome::Explored),
                ));
            }
        }));
    }
    for h in handles {
        h.join().expect("thread join");
    }
    let snap = r.baseline_snapshot();
    let expected = (threads * per_thread) as usize;
    assert_eq!(
        snap.n, expected,
        "expected {expected} updates after concurrent writers, got {}",
        snap.n,
    );
    let ema = snap.ema.expect("ema present after writes");
    // All samples cost=100; EMA should converge to 100 within rounding.
    assert!(
        (ema - 100.0).abs() < 1e-6,
        "EMA drift after concurrent writers: got {ema}",
    );
}

// rationale: Contract regression — `CostBand` has exactly three
// variants. Exhaustive match below compile-fails if a new variant is
// added; the iter assertion locks the runtime cardinality.
#[test]
fn m6_cost_band_has_exactly_three_variants() {
    let all = [
        CostBand::BelowBaseline,
        CostBand::NearBaseline,
        CostBand::AboveBaseline,
    ];
    assert_eq!(all.len(), 3);
    for variant in all {
        match variant {
            CostBand::BelowBaseline => assert_eq!(variant.as_str(), "BelowBaseline"),
            CostBand::NearBaseline => assert_eq!(variant.as_str(), "NearBaseline"),
            CostBand::AboveBaseline => assert_eq!(variant.as_str(), "AboveBaseline"),
        }
    }
}

// rationale: Cross-module surface invariant — `record_and_update_baseline`
// returns a record whose `cost_band` matches the EMA in the SAME atomic
// update (no time-of-check-vs-time-of-use gap with other threads).
#[test]
fn m6_record_and_update_band_matches_ema_atomically() {
    let r = ContextCostRecord::new(ContextCostRecordConfig::default());
    for i in 0..6_u32 {
        let _ = r.record_and_update_baseline(cost(
            &format!("s{i}"),
            100,
            Some(WorkflowOutcome::Explored),
        ));
    }
    let out = r.record_and_update_baseline(cost(
        "probe",
        100,
        Some(WorkflowOutcome::Explored),
    ));
    let ema = out.exploration_baseline.expect("ema must be Some");
    let band = out.cost_band.expect("band must be Some");
    assert!(ema.is_finite() && ema > 0.0);
    assert_eq!(band, CostBand::NearBaseline);
}
