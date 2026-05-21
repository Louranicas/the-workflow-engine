//! Integration tests for m32 `ConductorDispatcher` — exercises each
//! `EscapeSurfaceProfile` against the canonical `HumanAcceptanceSignature`
//! monotone acknowledged-ceiling gate (C10), plus the C3 routing-method
//! check + AP-V7-08 self-dispatch guard. Wave-B1 / S1002600 carry-forward.

#![allow(clippy::doc_markdown)]

use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use workflow_core::m14_lift::LiftSnapshot;
use workflow_core::m20_prefixspan::{Pattern, StepToken};
use workflow_core::m21_variant_builder::build_variants;
use workflow_core::m23_proposer::build_proposal;
use workflow_core::m30_bank::{AcceptedWorkflow, CuratedBank};
use workflow_core::m32_dispatcher::{
    self_dispatch_guard, ConductorClient, ConductorDispatcher, DispatchOutcome, DispatcherError,
    EscapeSurfaceProfile, HumanAcceptanceSignature, RefusalReason, CONDUCTOR_DISPATCH_METHOD,
};
use workflow_core::m33_verifier::{Verifier, VerifierKind, VerifierVerdict};

// ─── fixtures ───────────────────────────────────────────────────────────────

fn workflow_with_seed(seed: u32) -> AcceptedWorkflow {
    let p = Pattern::new(vec![StepToken(seed), StepToken(seed + 1)], 30, (0, seed as usize));
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
    bank.get(id).expect("g")
}

struct OkClient {
    calls: Arc<Mutex<u32>>,
}
impl ConductorClient for OkClient {
    fn submit(
        &self,
        _: u64,
        _: EscapeSurfaceProfile,
        _: &HumanAcceptanceSignature,
    ) -> Result<String, DispatcherError> {
        *self.calls.lock().expect("lock") += 1;
        Ok("conductor-dispatch-001".into())
    }
}
fn ok_pair() -> (OkClient, Arc<Mutex<u32>>) {
    let calls = Arc::new(Mutex::new(0_u32));
    (OkClient { calls: Arc::clone(&calls) }, calls)
}

struct WrongRoutingClient;
impl ConductorClient for WrongRoutingClient {
    fn submit(
        &self,
        _: u64,
        _: EscapeSurfaceProfile,
        _: &HumanAcceptanceSignature,
    ) -> Result<String, DispatcherError> {
        Ok("ok".into())
    }
    fn dispatch_method(&self) -> &'static str {
        "lcm.deploy"
    }
}

struct RefuseVerifier {
    kind: VerifierKind,
}
impl Verifier for RefuseVerifier {
    fn kind(&self) -> VerifierKind {
        self.kind
    }
    fn verify(&self, _: &AcceptedWorkflow) -> VerifierVerdict {
        VerifierVerdict::Refuse {
            reason: "block".into(),
        }
    }
}

/// Build a `HumanAcceptanceSignature` whose acknowledged ceiling is exactly
/// `p`, so the monotone ack-gate passes cleanly for a dispatch at profile `p`.
fn signature_for_profile(p: EscapeSurfaceProfile) -> HumanAcceptanceSignature {
    HumanAcceptanceSignature {
        interactive_terminal: true,
        acknowledged_ceiling: p,
    }
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[test]
fn m32_each_escape_surface_profile_dispatches_or_refuses_per_ack_table() {
    // rationale: Boundary — every one of the 7 EscapeSurfaceProfile variants
    // is exercised end-to-end with the correct ack-bit signature.
    for &profile in &EscapeSurfaceProfile::VARIANTS {
        let (client, calls) = ok_pair();
        let d = ConductorDispatcher::new(client);
        let sig = signature_for_profile(profile);
        let out = d
            .dispatch(&workflow_with_seed(u32::from(profile.ordinal()) + 1), profile, &sig)
            .expect("dispatch ok");
        assert!(
            matches!(out, DispatchOutcome::Accepted { .. }),
            "profile {profile:?} did not Accept with matching ack: {out:?}"
        );
        assert_eq!(*calls.lock().expect("lock"), 1);
    }
}

#[test]
fn m32_privilege_escalation_without_ack_refused() {
    // rationale: Adversarial input — PrivilegeEscalation (ord 30) is not
    // covered by the default (Sandboxed, ord 0) ceiling; dispatch refuses
    // with EscapeSurfaceNotAcknowledged and the client is not invoked.
    let (client, calls) = ok_pair();
    let d = ConductorDispatcher::new(client);
    let out = d
        .dispatch(
            &workflow_with_seed(101),
            EscapeSurfaceProfile::PrivilegeEscalation,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::EscapeSurfaceNotAcknowledged {
                required: EscapeSurfaceProfile::PrivilegeEscalation,
                acknowledged: EscapeSurfaceProfile::Sandboxed,
            }
        }
    ));
    assert_eq!(*calls.lock().expect("lock"), 0);
}

#[test]
fn m32_data_exfil_without_ack_refused() {
    // rationale: Adversarial input — DataExfil (ord 60) is not covered by the
    // default (Sandboxed, ord 0) ceiling; dispatch refuses with
    // EscapeSurfaceNotAcknowledged.
    let (client, calls) = ok_pair();
    let d = ConductorDispatcher::new(client);
    let out = d
        .dispatch(
            &workflow_with_seed(102),
            EscapeSurfaceProfile::DataExfil,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::EscapeSurfaceNotAcknowledged {
                required: EscapeSurfaceProfile::DataExfil,
                acknowledged: EscapeSurfaceProfile::Sandboxed,
            }
        }
    ));
    assert_eq!(*calls.lock().expect("lock"), 0);
}

#[test]
fn m32_file_write_without_ack_refused() {
    // rationale: C10 regression — the monotone-gate bug. FileWrite (ord 40)
    // outranks the previously-gated PrivilegeEscalation (ord 30) yet, pre-fix,
    // dispatched with no acknowledgement. Under the default (Sandboxed)
    // ceiling it MUST now refuse and the client MUST NOT be invoked.
    let (client, calls) = ok_pair();
    let d = ConductorDispatcher::new(client);
    let out = d
        .dispatch(
            &workflow_with_seed(105),
            EscapeSurfaceProfile::FileWrite,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::EscapeSurfaceNotAcknowledged {
                required: EscapeSurfaceProfile::FileWrite,
                acknowledged: EscapeSurfaceProfile::Sandboxed,
            }
        }
    ));
    assert_eq!(*calls.lock().expect("lock"), 0);
}

#[test]
fn m32_network_egress_without_ack_refused() {
    // rationale: C10 regression — companion to the FileWrite case.
    // NetworkEgress (ord 50) also outranks PrivilegeEscalation (ord 30) and,
    // pre-fix, sailed through ungated. The default ceiling MUST now refuse it.
    let (client, calls) = ok_pair();
    let d = ConductorDispatcher::new(client);
    let out = d
        .dispatch(
            &workflow_with_seed(106),
            EscapeSurfaceProfile::NetworkEgress,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::EscapeSurfaceNotAcknowledged {
                required: EscapeSurfaceProfile::NetworkEgress,
                acknowledged: EscapeSurfaceProfile::Sandboxed,
            }
        }
    ));
    assert_eq!(*calls.lock().expect("lock"), 0);
}

#[test]
fn m32_ceiling_at_profile_severity_permits_dispatch() {
    // rationale: C10 contract regression — the monotone permit half. For
    // every profile, a signature whose ceiling is exactly that profile MUST
    // Accept (the helper `signature_for_profile` builds exactly this).
    for &profile in &EscapeSurfaceProfile::VARIANTS {
        let (client, calls) = ok_pair();
        let d = ConductorDispatcher::new(client);
        let out = d
            .dispatch(
                &workflow_with_seed(u32::from(profile.ordinal()) + 200),
                profile,
                &signature_for_profile(profile),
            )
            .expect("dispatch ok");
        assert!(
            matches!(out, DispatchOutcome::Accepted { .. }),
            "profile {profile:?} refused with ceiling at its own severity: {out:?}"
        );
        assert_eq!(*calls.lock().expect("lock"), 1);
    }
}

#[test]
fn m32_routing_method_check_fires_before_verifier_gate() {
    // rationale: Anti-property — ordering invariant: a misrouted client AND
    // a blocking verifier present → returns RoutingMethodMismatch (NOT
    // VerifierGateBlocked). Defense-in-depth: cheap check first.
    let verifier: Box<dyn Verifier> = Box::new(RefuseVerifier {
        kind: VerifierKind::Security,
    });
    let d = ConductorDispatcher::new(WrongRoutingClient).with_verifiers(vec![verifier]);
    let out = d
        .dispatch(
            &workflow_with_seed(103),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    match out {
        DispatchOutcome::Refused {
            reason: RefusalReason::RoutingMethodMismatch { expected, actual },
        } => {
            assert_eq!(expected, "lcm.loop.create");
            assert_eq!(actual, "lcm.deploy");
        }
        other => panic!("expected RoutingMethodMismatch, got {other:?}"),
    }
}

#[test]
fn m32_dispatch_method_constant_equals_lcm_loop_create_literal() {
    // rationale: Contract regression — defensive const stability. The
    // production wire method is "lcm.loop.create"; "lcm.deploy" is the
    // documented regression target and MUST never accidentally become the
    // constant. Locks the literal at the integration boundary.
    assert_eq!(CONDUCTOR_DISPATCH_METHOD, "lcm.loop.create");
}

#[test]
fn m32_self_dispatch_guard_refuses_forbidden_proposal_id() {
    // rationale: Anti-property — AP-V7-08 self-dispatch wiring. Construct
    // the dispatcher with `with_forbidden_proposals` containing the test
    // workflow's proposal_id; dispatch refuses with SelfDispatchRefused
    // BEFORE any ack / routing / verifier check.
    let w = workflow_with_seed(104);
    let forbidden = vec![w.proposal().proposal_id()];

    // Sanity: the bare guard agrees.
    assert!(!self_dispatch_guard(&w, &forbidden));

    let (client, calls) = ok_pair();
    let d = ConductorDispatcher::with_forbidden_proposals(client, forbidden);
    let out = d
        .dispatch(
            &w,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::SelfDispatchRefused
        }
    ));
    assert_eq!(*calls.lock().expect("lock"), 0);
}
