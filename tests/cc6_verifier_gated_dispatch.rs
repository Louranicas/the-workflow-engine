//! Integration tests for CC-6 — Verifier-gated dispatch (m33 → m32).
//! Wave-B1 / S1002600 carry-forward H6 closure.
//!
//! Each test wires a 4-verifier set into a real `ConductorDispatcher` via
//! `with_verifiers(...)` and asserts the m33 aggregate verdict properly
//! gates m32 egress: AllApprove → wire fires once; any blocking verdict
//! → `VerifierGateBlocked` carrying ordinal-ordered `blocking_kinds`;
//! duplicate kinds → builder/aggregate refuses with `DuplicateVerifier`.

#![allow(clippy::doc_markdown)]

use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use workflow_core::m14_lift::LiftSnapshot;
use workflow_core::m20_prefixspan::{Pattern, StepToken};
use workflow_core::m21_variant_builder::build_variants;
use workflow_core::m23_proposer::build_proposal;
use workflow_core::m30_bank::{AcceptedWorkflow, CuratedBank};
use workflow_core::m32_dispatcher::{
    ConductorClient, ConductorDispatcher, DispatchOutcome, DispatcherError, EscapeSurfaceProfile,
    HumanAcceptanceSignature, RefusalReason,
};
use workflow_core::m33_verifier::{
    aggregate, AggregateVerdict, Verifier, VerifierError, VerifierKind, VerifierVerdict,
};

// ─── fixtures ───────────────────────────────────────────────────────────────

fn sample_workflow() -> AcceptedWorkflow {
    let p = Pattern::new(vec![StepToken(7), StepToken(8)], 30, (0, 7));
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

type SpyLog = Arc<Mutex<u32>>;

struct SpyClient {
    calls: SpyLog,
}
impl ConductorClient for SpyClient {
    fn submit(
        &self,
        _: u64,
        _: EscapeSurfaceProfile,
        _: &HumanAcceptanceSignature,
    ) -> Result<String, DispatcherError> {
        *self.calls.lock().expect("lock") += 1;
        Ok("ok".into())
    }
}

fn spy() -> (SpyClient, SpyLog) {
    let calls = Arc::new(Mutex::new(0_u32));
    (SpyClient { calls: Arc::clone(&calls) }, calls)
}

struct StaticVerifier {
    kind: VerifierKind,
    verdict: VerifierVerdict,
}
impl Verifier for StaticVerifier {
    fn kind(&self) -> VerifierKind {
        self.kind
    }
    fn verify(&self, _: &AcceptedWorkflow) -> VerifierVerdict {
        self.verdict.clone()
    }
}

fn approve(k: VerifierKind) -> Box<dyn Verifier> {
    Box::new(StaticVerifier {
        kind: k,
        verdict: VerifierVerdict::Approve,
    })
}
fn refuse(k: VerifierKind, reason: &str) -> Box<dyn Verifier> {
    Box::new(StaticVerifier {
        kind: k,
        verdict: VerifierVerdict::Refuse {
            reason: reason.to_owned(),
        },
    })
}
fn amend(k: VerifierKind, request: &str) -> Box<dyn Verifier> {
    Box::new(StaticVerifier {
        kind: k,
        verdict: VerifierVerdict::Amend {
            request: request.to_owned(),
        },
    })
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[test]
fn cc6_all_approve_proceeds_to_client() {
    // rationale: Cross-module — AllApprove verdict delegates to the wire
    // client; exactly one egress submit occurs.
    let (client, calls) = spy();
    let verifiers: Vec<Box<dyn Verifier>> = vec![
        approve(VerifierKind::Security),
        approve(VerifierKind::Consistency),
        approve(VerifierKind::Cost),
        approve(VerifierKind::Ember),
    ];
    let d = ConductorDispatcher::new(client).with_verifiers(verifiers);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    assert!(matches!(out, DispatchOutcome::Accepted { .. }));
    assert_eq!(*calls.lock().expect("lock"), 1);
}

#[test]
fn cc6_any_refuse_blocks_dispatch_carries_kind() {
    // rationale: Anti-property — Security refuses; dispatch returns
    // VerifierGateBlocked with blocking_kinds = [Security]; client NOT called.
    let (client, calls) = spy();
    let verifiers: Vec<Box<dyn Verifier>> = vec![
        refuse(VerifierKind::Security, "ssrf"),
        approve(VerifierKind::Consistency),
        approve(VerifierKind::Cost),
        approve(VerifierKind::Ember),
    ];
    let d = ConductorDispatcher::new(client).with_verifiers(verifiers);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    match out {
        DispatchOutcome::Refused {
            reason: RefusalReason::VerifierGateBlocked { blocking_kinds },
        } => {
            assert_eq!(blocking_kinds, vec![VerifierKind::Security]);
        }
        other => panic!("expected VerifierGateBlocked, got {other:?}"),
    }
    assert_eq!(*calls.lock().expect("lock"), 0);
}

#[test]
fn cc6_any_amend_blocks_dispatch_too() {
    // rationale: Anti-property — `is_blocking()` returns true for both
    // Refuse AND Amend; Amend by Cost must block dispatch identically.
    let (client, calls) = spy();
    let verifiers: Vec<Box<dyn Verifier>> = vec![
        approve(VerifierKind::Security),
        approve(VerifierKind::Consistency),
        amend(VerifierKind::Cost, "narrow budget"),
        approve(VerifierKind::Ember),
    ];
    let d = ConductorDispatcher::new(client).with_verifiers(verifiers);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    match out {
        DispatchOutcome::Refused {
            reason: RefusalReason::VerifierGateBlocked { blocking_kinds },
        } => {
            assert_eq!(blocking_kinds, vec![VerifierKind::Cost]);
        }
        other => panic!("expected VerifierGateBlocked with Cost, got {other:?}"),
    }
    assert_eq!(*calls.lock().expect("lock"), 0);
}

#[test]
fn cc6_multiple_blockers_listed_in_ordinal_order() {
    // rationale: Determinism — Security (ord 0) refuses AND Cost (ord 2)
    // amends; the dispatcher's RefusalReason::VerifierGateBlocked carries
    // blocking_kinds in VerifierKind::ordinal order (Security, Cost),
    // regardless of constructor slice ordering.
    let (client, _calls) = spy();
    let verifiers: Vec<Box<dyn Verifier>> = vec![
        // Insert Cost FIRST to exercise ordering invariant.
        amend(VerifierKind::Cost, "over budget"),
        approve(VerifierKind::Consistency),
        refuse(VerifierKind::Security, "deny egress"),
        approve(VerifierKind::Ember),
    ];
    let d = ConductorDispatcher::new(client).with_verifiers(verifiers);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    match out {
        DispatchOutcome::Refused {
            reason: RefusalReason::VerifierGateBlocked { blocking_kinds },
        } => {
            assert_eq!(
                blocking_kinds,
                vec![VerifierKind::Security, VerifierKind::Cost],
                "blocking_kinds not in ordinal order"
            );
        }
        other => panic!("expected VerifierGateBlocked, got {other:?}"),
    }
}

#[test]
fn cc6_duplicate_verifier_kinds_rejected_at_aggregate() {
    // rationale: Contract regression — m33::aggregate enforces "exactly
    // one of each VerifierKind"; duplicate Security must surface as
    // VerifierError::DuplicateVerifier. The dispatcher's H6 gate folds
    // any aggregator error into VerifierGateBlocked (failsafe), so we
    // independently verify the aggregator's typed-error path here.
    let v: Vec<Box<dyn Verifier>> = vec![
        approve(VerifierKind::Security),
        approve(VerifierKind::Security), // duplicate
        approve(VerifierKind::Consistency),
        approve(VerifierKind::Cost),
        approve(VerifierKind::Ember),
    ];
    let refs: Vec<&dyn Verifier> = v.iter().map(std::convert::AsRef::as_ref).collect();
    let r = aggregate(&refs, &sample_workflow());
    assert!(matches!(
        r,
        Err(VerifierError::DuplicateVerifier(VerifierKind::Security))
    ));

    // And the dispatcher must NOT silently pass when given the same
    // malformed set — fail-closed semantics yield VerifierGateBlocked.
    let (client, calls) = spy();
    let dup: Vec<Box<dyn Verifier>> = vec![
        approve(VerifierKind::Security),
        approve(VerifierKind::Security),
        approve(VerifierKind::Consistency),
        approve(VerifierKind::Cost),
        approve(VerifierKind::Ember),
    ];
    let d = ConductorDispatcher::new(client).with_verifiers(dup);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::VerifierGateBlocked { .. }
        }
    ));
    assert_eq!(*calls.lock().expect("lock"), 0);
}

#[test]
fn cc6_aggregate_directly_reports_all_approve_when_quad_approves() {
    // rationale: Contract regression — m33::aggregate raw surface confirms
    // AllApprove on the canonical 4-verifier approve set.
    let v: Vec<Box<dyn Verifier>> = vec![
        approve(VerifierKind::Security),
        approve(VerifierKind::Consistency),
        approve(VerifierKind::Cost),
        approve(VerifierKind::Ember),
    ];
    let refs: Vec<&dyn Verifier> = v.iter().map(std::convert::AsRef::as_ref).collect();
    let r = aggregate(&refs, &sample_workflow()).expect("ok");
    assert_eq!(r, AggregateVerdict::AllApprove);
}
