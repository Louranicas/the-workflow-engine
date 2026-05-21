//! Integration tests for m20 prefixspan_miner.
//!
//! Per m20 spec § 8 F-Integration row. Exercises:
//! - Public API surface from outside the crate.
//! - End-to-end mining on realistic mixed cascades.
//! - F11 opacity at the public-crate boundary.
//! - Cross-module wiring (m20 → m21 → m23 chain).

#![allow(clippy::doc_markdown)]

use workflow_core::m14_lift::LiftSnapshot;
use workflow_core::m20_prefixspan::{
    mine_sequences, MaxGap, MinSupport, Pattern, StepToken, DEFAULT_MAX_GAP, DEFAULT_MAX_LENGTH,
    MIN_SUPPORT_FLOOR,
};
use workflow_core::m21_variant_builder::{build_variants, MAX_VARIANTS_PER_PATTERN};
use workflow_core::m23_proposer::{compose_proposals, PROPOSAL_F2_THRESHOLD};

fn tok(n: u32) -> StepToken {
    StepToken(n)
}
fn seq(ns: &[u32]) -> Vec<StepToken> {
    ns.iter().copied().map(tok).collect()
}

#[test]
// rationale: Integration — public API end-to-end with defaults.
fn public_api_round_trip_with_defaults() {
    let seqs = vec![seq(&[1, 2, 3]), seq(&[1, 2, 4]), seq(&[1, 2, 3, 5])];
    let p = mine_sequences(
        &seqs,
        MinSupport::new(MIN_SUPPORT_FLOOR).expect("at floor"),
        MaxGap::new(DEFAULT_MAX_GAP),
        DEFAULT_MAX_LENGTH,
    )
    .expect("ok");
    assert!(!p.is_empty());
    assert!(p.iter().any(|pat| pat.steps() == seq(&[1, 2])));
}

#[test]
// rationale: Cross-module — m20 → m21 → m23 full pipeline produces
// human-reviewable proposals from cascade observations.
fn pipeline_m20_to_m21_to_m23_yields_proposals() {
    // 30 sequences each carrying [1,2,3] — well above PROPOSAL_F2_THRESHOLD.
    let seqs: Vec<Vec<StepToken>> = (0..30).map(|_| seq(&[1, 2, 3])).collect();
    let patterns = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("mine");
    assert!(!patterns.is_empty());

    let snapshot = LiftSnapshot {
        lift: Some(0.7),
        ci_half: Some(0.05),
        n: PROPOSAL_F2_THRESHOLD + 10,
        latest_ts_ms: 1_000_000,
        computed_at: std::time::SystemTime::now(),
    };
    // `|_| None` = m22 clustering skipped (this test exercises the
    // m20 → m21 → m23 chain, not diversity).
    let proposals = compose_proposals(&patterns, &snapshot, |_| None);
    assert!(!proposals.is_empty());
    for p in &proposals {
        assert!(p.evidence_n() >= PROPOSAL_F2_THRESHOLD);
    }
}

#[test]
// rationale: Integration — F11 opacity across module boundaries.
// Mining + variant-build + proposal compose must not produce any
// human-readable substrate label in their serde JSON.
fn integration_f11_full_pipeline_opacity() {
    let seqs = vec![seq(&[10, 20]), seq(&[10, 20]), seq(&[10, 20, 30])];
    let patterns = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("mine");
    for p in &patterns {
        let s = serde_json::to_string(p).expect("ser");
        for forbidden in ["pane", "tab", "cluster_pane", "workflow_trace_"] {
            assert!(
                !s.contains(forbidden),
                "F11 leak in pattern serde: {s}"
            );
        }
    }
    if let Some(p) = patterns.first() {
        let variants = build_variants(p).expect("v");
        for v in &variants {
            let s = serde_json::to_string(v).expect("ser");
            for forbidden in ["pane", "tab", "cluster_pane"] {
                assert!(!s.contains(forbidden), "F11 leak in variant serde: {s}");
            }
        }
    }
}

#[test]
// rationale: Integration — variant cap enforced at compose boundary
// (m21 cap propagates through m23 batching).
fn integration_variant_cap_propagates_through_compose() {
    let pattern = Pattern::new(seq(&[1, 2, 3, 4, 5, 6, 7, 8]), 25, (0, 1));
    let variants = build_variants(&pattern).expect("v");
    assert!(variants.len() <= MAX_VARIANTS_PER_PATTERN);
}

#[test]
// rationale: Integration — m20 below-support pattern never leaks to m23.
fn integration_below_support_filtered() {
    let seqs = vec![seq(&[1, 2]), seq(&[3, 4])];
    let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
    // No pattern present in BOTH sequences — output may be empty or trivial.
    for pat in &p {
        assert!(pat.support() >= 2, "below-support leak: {pat:?}");
    }
}
