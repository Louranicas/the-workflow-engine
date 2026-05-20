//! Integration tests for m33 4-verifier aggregator (Wave-C3).
//!
//! Locks the 4-verifier cardinality contract, the
//! ordinal-sorted determinism of `Blocked` outcomes, the
//! `is_blocking` semantics of `Amend` (parity with `Refuse`), and the
//! adversarial `DuplicateVerifier` refusal path.

#![allow(clippy::doc_markdown)]

use std::time::SystemTime;

use workflow_core::m14_lift::LiftSnapshot;
use workflow_core::m20_prefixspan::{Pattern, StepToken};
use workflow_core::m21_variant_builder::build_variants;
use workflow_core::m23_proposer::build_proposal;
use workflow_core::m30_bank::{AcceptedWorkflow, CuratedBank};
use workflow_core::m33_verifier::{
    aggregate, AggregateVerdict, Verifier, VerifierError, VerifierKind, VerifierVerdict,
};

// ─── fixtures ────────────────────────────────────────────────────────

fn sample_workflow() -> AcceptedWorkflow {
    let p = Pattern::new(vec![StepToken(1), StepToken(2)], 30, (0, 1));
    let v = build_variants(&p).expect("v")[0].clone();
    let s = LiftSnapshot {
        lift: Some(0.5),
        ci_half: Some(0.05),
        n: 30,
        latest_ts_ms: 0,
        computed_at: SystemTime::now(),
    };
    let bank = CuratedBank::new();
    let id = bank
        .accept(build_proposal(v, &s, None).expect("p"), 0)
        .expect("accept");
    bank.get(id).expect("get")
}

struct Static {
    kind: VerifierKind,
    verdict: VerifierVerdict,
}
impl Verifier for Static {
    fn kind(&self) -> VerifierKind {
        self.kind
    }
    fn verify(&self, _: &AcceptedWorkflow) -> VerifierVerdict {
        self.verdict.clone()
    }
}

fn approve(k: VerifierKind) -> Box<dyn Verifier> {
    Box::new(Static {
        kind: k,
        verdict: VerifierVerdict::Approve,
    })
}
fn refuse(k: VerifierKind, reason: &str) -> Box<dyn Verifier> {
    Box::new(Static {
        kind: k,
        verdict: VerifierVerdict::Refuse {
            reason: reason.to_owned(),
        },
    })
}
fn amend(k: VerifierKind, req: &str) -> Box<dyn Verifier> {
    Box::new(Static {
        kind: k,
        verdict: VerifierVerdict::Amend {
            request: req.to_owned(),
        },
    })
}

fn to_refs(v: &[Box<dyn Verifier>]) -> Vec<&dyn Verifier> {
    v.iter().map(std::convert::AsRef::as_ref).collect()
}

// ─── tests ───────────────────────────────────────────────────────────

// rationale: Happy path — all 4 verifiers approve → AllApprove.
#[test]
fn m33_aggregate_with_all_approvers_returns_all_approve() {
    // rationale: Happy path
    let v = [
        approve(VerifierKind::Security),
        approve(VerifierKind::Consistency),
        approve(VerifierKind::Cost),
        approve(VerifierKind::Ember),
    ];
    let r = aggregate(&to_refs(&v), &sample_workflow()).expect("ok");
    assert_eq!(r, AggregateVerdict::AllApprove);
}

// rationale: Anti-property — a single Refuse MUST block; the resulting
// Blocked log MUST carry exactly the refused VerifierKind among its
// per_verifier outcomes (and 3 others recording Approve).
#[test]
fn m33_aggregate_with_one_refuse_returns_blocked_with_kind() {
    // rationale: Anti-property (single-refusal blocks)
    let v = [
        approve(VerifierKind::Security),
        refuse(VerifierKind::Consistency, "duplicate"),
        approve(VerifierKind::Cost),
        approve(VerifierKind::Ember),
    ];
    let r = aggregate(&to_refs(&v), &sample_workflow()).expect("ok");
    match r {
        AggregateVerdict::Blocked { per_verifier } => {
            assert_eq!(per_verifier.len(), 4);
            let refusing: Vec<&VerifierKind> = per_verifier
                .iter()
                .filter(|(_, v)| matches!(v, VerifierVerdict::Refuse { .. }))
                .map(|(k, _)| k)
                .collect();
            assert_eq!(refusing.len(), 1);
            assert_eq!(*refusing[0], VerifierKind::Consistency);
        }
        AggregateVerdict::AllApprove => panic!("expected Blocked, got AllApprove"),
    }
}

// rationale: is_blocking semantics — Amend MUST block dispatch with
// the same strength as Refuse. Parity test: a single Amend produces
// Blocked, not AllApprove.
#[test]
fn m33_aggregate_with_one_amend_returns_blocked_too() {
    // rationale: is_blocking semantics (Amend = block)
    let v = [
        approve(VerifierKind::Security),
        approve(VerifierKind::Consistency),
        amend(VerifierKind::Cost, "clarify budget"),
        approve(VerifierKind::Ember),
    ];
    let r = aggregate(&to_refs(&v), &sample_workflow()).expect("ok");
    match r {
        AggregateVerdict::Blocked { per_verifier } => {
            assert!(per_verifier
                .iter()
                .any(|(k, v)| *k == VerifierKind::Cost
                    && matches!(v, VerifierVerdict::Amend { .. })));
        }
        AggregateVerdict::AllApprove => {
            panic!("Amend MUST block (is_blocking parity with Refuse)")
        }
    }
    // Sanity: predicate matches both Refuse and Amend.
    assert!(VerifierVerdict::Refuse {
        reason: "x".into()
    }
    .is_blocking());
    assert!(VerifierVerdict::Amend {
        request: "y".into()
    }
    .is_blocking());
    assert!(!VerifierVerdict::Approve.is_blocking());
}

// rationale: Determinism — the per_verifier log MUST be sorted by
// VerifierKind::ordinal regardless of caller slice order. Lock the
// human-comparable refusal log invariant (m33 spec § 5).
#[test]
fn m33_aggregate_multiple_blockers_in_ordinal_order() {
    // rationale: Determinism (ordinal-sorted log)
    let v = [
        refuse(VerifierKind::Ember, "ember"),
        refuse(VerifierKind::Cost, "cost"),
        refuse(VerifierKind::Consistency, "consistency"),
        refuse(VerifierKind::Security, "security"),
    ];
    let r = aggregate(&to_refs(&v), &sample_workflow()).expect("ok");
    match r {
        AggregateVerdict::Blocked { per_verifier } => {
            let kinds: Vec<VerifierKind> = per_verifier.iter().map(|(k, _)| *k).collect();
            assert_eq!(
                kinds,
                vec![
                    VerifierKind::Security,
                    VerifierKind::Consistency,
                    VerifierKind::Cost,
                    VerifierKind::Ember,
                ],
                "per_verifier MUST be sorted by ordinal"
            );
            // Also: ordinal sequence is 0,1,2,3 exactly.
            let ords: Vec<u8> = kinds.iter().map(|k| k.ordinal()).collect();
            assert_eq!(ords, vec![0, 1, 2, 3]);
        }
        AggregateVerdict::AllApprove => panic!("expected Blocked"),
    }
}

// rationale: Adversarial — DuplicateVerifier MUST be detected FIRST
// (before MissingVerifier), so a duplicate-of-the-only-supplied-kind
// cannot read as "all required present".
#[test]
fn m33_aggregate_duplicate_verifier_kinds_rejected() {
    // rationale: Adversarial (DuplicateVerifier wins over MissingVerifier)
    let v = vec![
        approve(VerifierKind::Security),
        approve(VerifierKind::Security), // duplicate
        approve(VerifierKind::Consistency),
        approve(VerifierKind::Cost),
        approve(VerifierKind::Ember),
    ];
    let r = aggregate(&to_refs(&v), &sample_workflow());
    assert!(
        matches!(
            r,
            Err(VerifierError::DuplicateVerifier(VerifierKind::Security))
        ),
        "expected DuplicateVerifier(Security), got {r:?}"
    );
}

// rationale: Contract regression (compile-time) — VerifierKind has
// EXACTLY 4 variants. A new variant would silently make
// `aggregate` MissingVerifier-prone; removal would break dispatch
// gating. The const _ assert in src/m33_verifier/mod.rs also guards
// this, but a runtime test against VARIANTS.len() locks the test
// surface too.
#[test]
fn m33_verifier_kind_variants_cardinality_is_4() {
    // rationale: Contract regression (4-verifier cardinality)
    assert_eq!(VerifierKind::VARIANTS.len(), 4);
    // Ordinals are 0..4 contiguous.
    let mut ords: Vec<u8> = VerifierKind::VARIANTS.iter().map(|k| k.ordinal()).collect();
    ords.sort_unstable();
    assert_eq!(ords, vec![0, 1, 2, 3]);
}
