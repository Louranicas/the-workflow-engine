//! Integration tests for CC-3 — Evidence-Driven Iteration (m14 → m23).
//! Wave-C3 cross-cluster suite.
//!
//! CC-3 binds m14's lift-snapshot evidence floor to m23's proposal F2
//! gate via a compile-time `pub const PROPOSAL_F2_THRESHOLD: usize =
//! MIN_SAMPLE_SIZE` alias. These tests prove the binding is intact at
//! both ends: a drift in `MIN_SAMPLE_SIZE` (m14) propagates to
//! `PROPOSAL_F2_THRESHOLD` (m23), and the m14→m23 functional pipeline
//! refuses-or-accepts based on snapshot.n with the m14 floor as the
//! single source of truth.

#![allow(clippy::doc_markdown)]

use std::time::SystemTime;

use workflow_core::m14_lift::{LiftAggregator, LiftAggregatorConfig, LiftSnapshot, MIN_SAMPLE_SIZE};
use workflow_core::m20_prefixspan::{Pattern, StepToken};
use workflow_core::m21_variant_builder::build_variants;
use workflow_core::m23_proposer::{
    build_proposal, compose_proposals, ProposerError, PROPOSAL_F2_THRESHOLD,
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

fn snap(n: usize, lift: Option<f64>, ci: Option<f64>) -> LiftSnapshot {
    LiftSnapshot {
        lift,
        ci_half: ci,
        n,
        latest_ts_ms: 0,
        computed_at: SystemTime::now(),
    }
}

fn sample_pattern() -> Pattern {
    Pattern::new(vec![StepToken(1), StepToken(2)], 30, (0, 1))
}

// rationale: Cross-cluster const invariant — at compile time and at
// runtime, PROPOSAL_F2_THRESHOLD MUST equal MIN_SAMPLE_SIZE. m23's
// `pub const PROPOSAL_F2_THRESHOLD = MIN_SAMPLE_SIZE` is the binding
// site; the alias is enforced at the type system, but a regression
// (e.g. someone hard-coding 20) would only fire here, not in unit
// tests scoped to a single crate boundary.
#[test]
fn cc3_proposal_f2_threshold_locked_to_min_sample_size_at_compile_time() {
    // rationale: Cross-cluster const invariant
    const _CC3_ALIAS_HOLDS: () = assert!(PROPOSAL_F2_THRESHOLD == MIN_SAMPLE_SIZE);
    assert_eq!(PROPOSAL_F2_THRESHOLD, MIN_SAMPLE_SIZE);
}

// rationale: m14 → m23 functional — a real m14 snapshot below
// MIN_SAMPLE_SIZE MUST block m23 proposal emission with the typed
// EvidenceBelowThreshold variant. End-to-end across cluster E → F.
#[test]
fn cc3_lift_snapshot_below_min_blocks_proposal_emission() {
    // rationale: m14 → m23 functional (below-floor block)
    let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
    // Build a window strictly smaller than the F2 floor.
    let rows: Vec<WorkflowRunRow> = (0..MIN_SAMPLE_SIZE - 1)
        .map(|i| run(i64::try_from(i).expect("fits"), "ok", Some(100)))
        .collect();
    let snapshot = agg.compute_snapshot(&rows);
    assert!(
        snapshot.lift.is_none(),
        "m14 must refuse below-floor with lift=None"
    );
    // m23's batched compose_proposals MUST skip (yields empty batch).
    // `|_| None` = the m22-clustering-skipped diversity closure.
    let batch = compose_proposals(&[sample_pattern()], &snapshot, |_| None);
    assert!(batch.is_empty(), "below-floor snapshot must skip every variant");
    // m23's strict path returns the typed EvidenceBelowThreshold.
    let variant = build_variants(&sample_pattern()).expect("v")[0].clone();
    let err = build_proposal(variant, &snapshot, None).expect_err("must refuse");
    match err {
        ProposerError::EvidenceBelowThreshold { n, threshold } => {
            assert_eq!(n, MIN_SAMPLE_SIZE - 1);
            assert_eq!(threshold, PROPOSAL_F2_THRESHOLD);
        }
        // `ProposerError` is `#[non_exhaustive]` — wildcard required for
        // the cross-crate match.
        other => panic!("expected EvidenceBelowThreshold, got {other:?}"),
    }
}

// rationale: F2 boundary — at exactly MIN_SAMPLE_SIZE the m14 snapshot
// becomes evidentially complete and m23 emission unblocks. Locks the
// bottom edge of the CC-3 accept region.
#[test]
fn cc3_lift_snapshot_at_min_unblocks_proposal_emission() {
    // rationale: F2 boundary
    let agg = LiftAggregator::new(LiftAggregatorConfig::default()).expect("agg");
    let rows: Vec<WorkflowRunRow> = (0..MIN_SAMPLE_SIZE)
        .map(|i| run(i64::try_from(i).expect("fits"), "ok", Some(100)))
        .collect();
    let snapshot = agg.compute_snapshot(&rows);
    assert!(snapshot.lift.is_some(), "lift must be Some at n=MIN_SAMPLE_SIZE");
    assert!(snapshot.ci_half.is_some());
    let variant = build_variants(&sample_pattern()).expect("v")[0].clone();
    let proposal = build_proposal(variant, &snapshot, None)
        .expect("at-floor snapshot MUST unblock m23");
    assert_eq!(proposal.evidence_n(), MIN_SAMPLE_SIZE);
}

// rationale: m14 → m23 happy path — compose_proposals over a small set
// of patterns with sufficient evidence returns a non-empty batch where
// every surviving proposal carries `evidence_n >= PROPOSAL_F2_THRESHOLD`
// AND its evidence reflects the source snapshot's lift sign and
// magnitude. Locks the full CC-3 pipeline shape.
#[test]
fn cc3_compose_proposals_handles_mixed_evidence_quality() {
    // rationale: m14 → m23 happy path (full pipeline shape)
    // Manually-constructed snapshot is fine — m14 surface is locked by
    // its own integration suite; here we exercise the m23 consumer.
    let snapshot = snap(50, Some(0.42), Some(0.05));
    let patterns = vec![
        Pattern::new(vec![StepToken(1), StepToken(2), StepToken(3)], 25, (0, 1)),
        Pattern::new(vec![StepToken(4), StepToken(5)], 22, (0, 0)),
    ];
    let batch = compose_proposals(&patterns, &snapshot, |_| None);
    assert!(!batch.is_empty(), "sufficient-evidence batch must be non-empty");
    for p in &batch {
        assert!(p.evidence_n() >= PROPOSAL_F2_THRESHOLD);
        assert!((p.evidence_lift() - 0.42).abs() < 1e-12);
        assert!((p.evidence_ci_half() - 0.05).abs() < 1e-12);
    }
}

// rationale: Drift protection — the CC-3 binding works ONLY if
// PROPOSAL_F2_THRESHOLD is literally `MIN_SAMPLE_SIZE` (not a copy of
// 20). We synthesise a snapshot at MIN_SAMPLE_SIZE-1 and confirm m23
// uses the m14 floor (not a hard-coded 19) for its `threshold` field;
// the symmetric synthesis at MIN_SAMPLE_SIZE unblocks. A drift would
// cause one side to disagree numerically.
#[test]
fn cc3_evidence_threshold_change_propagates_to_proposer() {
    // rationale: Drift protection
    let variant = build_variants(&sample_pattern()).expect("v")[0].clone();

    // Just below: m23 must report the m14 floor as the threshold.
    let s_below = snap(MIN_SAMPLE_SIZE - 1, Some(0.5), Some(0.05));
    let err = build_proposal(variant.clone(), &s_below, None).expect_err("must refuse");
    match err {
        ProposerError::EvidenceBelowThreshold { n, threshold } => {
            assert_eq!(n, MIN_SAMPLE_SIZE - 1);
            // The critical drift-protection assertion: m23 must report
            // a threshold equal to m14's MIN_SAMPLE_SIZE (not a copy).
            assert_eq!(threshold, MIN_SAMPLE_SIZE);
        }
        // `ProposerError` is `#[non_exhaustive]` — wildcard required for
        // the cross-crate match.
        other => panic!("expected EvidenceBelowThreshold, got {other:?}"),
    }

    // At-floor: unblocks.
    let s_at = snap(MIN_SAMPLE_SIZE, Some(0.5), Some(0.05));
    let p = build_proposal(variant, &s_at, None).expect("at-floor MUST unblock");
    assert_eq!(p.evidence_n(), MIN_SAMPLE_SIZE);
}
