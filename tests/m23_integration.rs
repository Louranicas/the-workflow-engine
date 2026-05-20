//! Integration tests for m23 workflow proposer (Wave-C3).
//!
//! Locks the F2 hard-refusal anti-properties, AP-V7-07 "no
//! auto-promote" public-surface invariant, determinism of the
//! proposal_id derivation, the compile-time alias between
//! `PROPOSAL_F2_THRESHOLD` and `m14_lift::MIN_SAMPLE_SIZE`, and the
//! documented silent-skip behaviour of `compose_proposals`.

#![allow(clippy::doc_markdown)]

use std::time::SystemTime;

use workflow_core::m14_lift::{LiftSnapshot, MIN_SAMPLE_SIZE};
use workflow_core::m20_prefixspan::{Pattern, StepToken};
use workflow_core::m21_variant_builder::{build_variants, WorkflowVariant};
use workflow_core::m23_proposer::{
    build_proposal, compose_proposals, ProposerError, WorkflowProposal, PROPOSAL_F2_THRESHOLD,
};

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
    Pattern::new(vec![StepToken(11), StepToken(22)], 30, (0, 1))
}

fn sample_variant() -> WorkflowVariant {
    build_variants(&sample_pattern()).expect("v")[0].clone()
}

// rationale: F2 anti-property — at PROPOSAL_F2_THRESHOLD - 1 the
// proposer MUST refuse with the typed EvidenceBelowThreshold variant.
// No silent acceptance.
#[test]
fn m23_build_proposal_refuses_below_proposal_f2_threshold() {
    // rationale: F2 anti-property
    let s = snap(PROPOSAL_F2_THRESHOLD - 1, Some(0.5), Some(0.05));
    let err = build_proposal(sample_variant(), &s, None)
        .expect_err("below F2 threshold MUST refuse");
    match err {
        ProposerError::EvidenceBelowThreshold { n, threshold } => {
            assert_eq!(n, PROPOSAL_F2_THRESHOLD - 1);
            assert_eq!(threshold, PROPOSAL_F2_THRESHOLD);
        }
        ProposerError::LiftUnavailable => {
            panic!("expected EvidenceBelowThreshold, got LiftUnavailable")
        }
    }
}

// rationale: F2 boundary — exactly at PROPOSAL_F2_THRESHOLD the
// proposer MUST accept and return a typed WorkflowProposal carrying the
// snapshot evidence.
#[test]
fn m23_build_proposal_accepts_exactly_at_proposal_f2_threshold() {
    // rationale: F2 boundary
    let s = snap(PROPOSAL_F2_THRESHOLD, Some(0.5), Some(0.05));
    let p: WorkflowProposal =
        build_proposal(sample_variant(), &s, None).expect("at threshold must accept");
    assert_eq!(p.evidence_n, PROPOSAL_F2_THRESHOLD);
    assert!((p.evidence_lift - 0.5).abs() < 1e-12);
    assert!((p.evidence_ci_half - 0.05).abs() < 1e-12);
}

// rationale: Determinism — identical (variant, snapshot.n) inputs MUST
// produce identical proposal_id values; m23 derives the id from a
// FNV-1a hash and downstream m30 admission uses the id as the bank key.
#[test]
fn m23_proposal_id_is_deterministic_for_same_inputs() {
    // rationale: Determinism
    let s = snap(30, Some(0.6), Some(0.05));
    let p1 = build_proposal(sample_variant(), &s, None).expect("p1");
    let p2 = build_proposal(sample_variant(), &s, None).expect("p2");
    let p3 = build_proposal(sample_variant(), &s, None).expect("p3");
    assert_eq!(p1.proposal_id, p2.proposal_id);
    assert_eq!(p2.proposal_id, p3.proposal_id);
}

// rationale: Anti-property (AP-V7-07 no-auto-promote) — m23's public
// surface, surveyed via its serde shape, MUST carry no
// bank/promotion/commit field; any such field would be a structural
// admission backdoor circumventing m30/m31/m32/m33 review. We also
// scan the proposer source file directly for forbidden `pub fn`
// signatures (compile-time-adjacent regression slot).
#[test]
fn m23_ap_v7_07_no_auto_promote_public_surface_check() {
    // rationale: Anti-property (AP-V7-07)
    let s = snap(30, Some(0.5), Some(0.05));
    let p = build_proposal(sample_variant(), &s, Some(0)).expect("ok");
    let json = serde_json::to_string(&p).expect("ser");
    for forbidden in [
        "promoted",
        "committed",
        "accepted_at",
        "bank",
        "promote_to",
        "auto_accept",
        "auto_promote",
    ] {
        assert!(
            !json.contains(forbidden),
            "AP-V7-07 violation: proposal serde contains '{forbidden}': {json}"
        );
    }
    // Static-scan the m23 source for forbidden public-fn signatures.
    // Skip `//`/`///`/`//!` comment lines so doc-comments mentioning the
    // forbidden phrases as PROSE (e.g. "// `pub fn promote_proposal`")
    // don't false-match the regression check; only structural `pub fn`
    // declarations at the start of a code line trip the assertion.
    let src = include_str!("../src/m23_proposer/mod.rs");
    for (lineno, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        for forbidden_sig in [
            "pub fn promote",
            "pub fn accept",
            "pub fn commit",
            "pub fn auto_promote",
        ] {
            assert!(
                !trimmed.starts_with(forbidden_sig),
                "AP-V7-07 violation: m23 line {lineno} declares forbidden public fn '{forbidden_sig}'"
            );
        }
    }
}

// rationale: Documented behaviour — compose_proposals is the batched
// form whose contract is "skip-and-trace on F2 failure". A mixed input
// where the snapshot is below threshold MUST yield an empty batch
// without raising. (The strict typed-refusal path is build_proposal.)
#[test]
fn m23_compose_proposals_silently_skips_individual_failures_with_tracing() {
    // rationale: Documented behaviour (batched F2 skip-and-trace)
    // Below-threshold snapshot: compose_proposals must return [].
    let s_below = snap(PROPOSAL_F2_THRESHOLD - 1, Some(0.5), Some(0.05));
    let batch = compose_proposals(&[sample_pattern()], &s_below);
    assert!(batch.is_empty(), "below-F2 batch must be empty");

    // Sufficient-evidence snapshot but lift=None: also must skip every
    // variant (LiftUnavailable path).
    let s_lift_none = snap(30, None, Some(0.05));
    let batch_none = compose_proposals(&[sample_pattern()], &s_lift_none);
    assert!(batch_none.is_empty(), "lift=None batch must be empty");

    // Mixed: with sufficient evidence, batch is non-empty and every
    // surviving proposal carries n >= PROPOSAL_F2_THRESHOLD.
    let s_ok = snap(30, Some(0.5), Some(0.05));
    let batch_ok = compose_proposals(&[sample_pattern()], &s_ok);
    assert!(!batch_ok.is_empty());
    for p in &batch_ok {
        assert!(p.evidence_n >= PROPOSAL_F2_THRESHOLD);
    }
}

// rationale: Cross-module invariant — PROPOSAL_F2_THRESHOLD is declared
// in m23 as `pub const PROPOSAL_F2_THRESHOLD: usize = MIN_SAMPLE_SIZE`,
// a compile-time alias of the m14 floor. This test asserts numeric
// equality AT RUNTIME and the compile-time alias is enforced by const
// declaration. A drift would break the entire CC-3 evidence-iteration
// contract (m14 → m23) silently.
#[test]
fn m23_proposal_f2_threshold_const_equals_min_sample_size() {
    // rationale: Cross-module invariant (compile-time alias)
    assert_eq!(PROPOSAL_F2_THRESHOLD, MIN_SAMPLE_SIZE);
}

// rationale: Boundary — lift=None and ci_half=None each independently
// fire LiftUnavailable; both branches must surface the same typed
// variant so downstream callers can match on a single error class.
#[test]
fn m23_build_proposal_refuses_when_lift_or_ci_half_is_none() {
    // rationale: Boundary (LiftUnavailable on either None)
    let lift_none = snap(30, None, Some(0.05));
    let ci_none = snap(30, Some(0.5), None);
    assert!(matches!(
        build_proposal(sample_variant(), &lift_none, None),
        Err(ProposerError::LiftUnavailable)
    ));
    assert!(matches!(
        build_proposal(sample_variant(), &ci_none, None),
        Err(ProposerError::LiftUnavailable)
    ));
}

// rationale: Contract regression — proposal serde round-trips fully,
// downstream m30 bank reads the wire form. Confirms the public type's
// serde derivation has not regressed.
#[test]
fn m23_proposal_serde_round_trip_is_lossless() {
    // rationale: Contract regression (serde wire-form)
    let s = snap(30, Some(-0.3), Some(0.05));
    let p = build_proposal(sample_variant(), &s, Some(7)).expect("ok");
    let ser = serde_json::to_string(&p).expect("ser");
    let back: WorkflowProposal = serde_json::from_str(&ser).expect("de");
    assert_eq!(back, p);
}
