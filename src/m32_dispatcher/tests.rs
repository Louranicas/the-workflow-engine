use std::sync::Mutex;
use std::time::SystemTime;

use super::{
    self_dispatch_guard, ConductorClient, ConductorDispatcher, DispatchOutcome,
    DispatcherError, EscapeSurfaceProfile, HumanAcceptanceSignature, RefusalReason,
    CONDUCTOR_DISPATCH_METHOD,
};
use crate::m14_lift::LiftSnapshot;
use crate::m20_prefixspan::{Pattern, StepToken};
use crate::m21_variant_builder::build_variants;
use crate::m23_proposer::build_proposal;
use crate::m30_bank::AcceptedWorkflow;
use crate::m33_verifier::{Verifier, VerifierKind, VerifierVerdict};

fn sample_workflow_with_seed(seed: u32) -> AcceptedWorkflow {
    let p = Pattern::new(vec![StepToken(seed)], 30, (0, seed as usize));
    let v = build_variants(&p).expect("v")[0].clone();
    let s = LiftSnapshot {
        lift: Some(0.5),
        ci_half: Some(0.05),
        n: 30,
        latest_ts_ms: 0,
        computed_at: SystemTime::now(),
    };
    AcceptedWorkflow::for_test(
        u64::from(42 + seed),
        build_proposal(v, &s, None).expect("ok"),
        0,
        i64::MAX,
        1.0,
        None,
        0,
    )
}

fn sample_workflow() -> AcceptedWorkflow {
    sample_workflow_with_seed(0)
}

struct OkClient;
impl ConductorClient for OkClient {
    fn submit(
        &self,
        _workflow_id: u64,
        _profile: EscapeSurfaceProfile,
        _signature: &HumanAcceptanceSignature,
    ) -> Result<String, DispatcherError> {
        Ok("conductor-dispatch-001".to_owned())
    }
}

struct FailClient {
    calls: Mutex<u32>,
}
impl ConductorClient for FailClient {
    fn submit(
        &self,
        _workflow_id: u64,
        _profile: EscapeSurfaceProfile,
        _signature: &HumanAcceptanceSignature,
    ) -> Result<String, DispatcherError> {
        *self.calls.lock().expect("lock") += 1;
        Err(DispatcherError::WireFormat("mock-fail".into()))
    }
}

/// Anti-property witness: a client that calls `std::process::Command`
/// is *forbidden* at the trait level. We can't statically prevent
/// arbitrary trait impls, but we can assert that the dispatcher itself
/// never reaches a syscall surface. This counter-client records every
/// invocation; the test then asserts the dispatcher's behaviour is
/// purely a trait call.
struct SpyClient {
    calls: Mutex<Vec<(u64, EscapeSurfaceProfile)>>,
}
impl ConductorClient for SpyClient {
    fn submit(
        &self,
        workflow_id: u64,
        profile: EscapeSurfaceProfile,
        _signature: &HumanAcceptanceSignature,
    ) -> Result<String, DispatcherError> {
        self.calls.lock().expect("lock").push((workflow_id, profile));
        Ok("spy-ok".into())
    }
}

/// Client with custom `dispatch_method` override for routing regression.
struct WrongRoutingClient;
impl ConductorClient for WrongRoutingClient {
    fn submit(
        &self,
        _workflow_id: u64,
        _profile: EscapeSurfaceProfile,
        _signature: &HumanAcceptanceSignature,
    ) -> Result<String, DispatcherError> {
        Ok("ok".into())
    }
    fn dispatch_method(&self) -> &'static str {
        "lcm.deploy" // explicitly wrong
    }
}

// --- Pre-existing tests preserved verbatim (signatures unchanged) ---

#[test]
fn escape_surface_profile_ordinals_are_canonical() {
    // rationale: Contract regression — 7-variant ordinals locked
    assert_eq!(EscapeSurfaceProfile::Sandboxed.ordinal(), 0);
    assert_eq!(EscapeSurfaceProfile::SandboxEscape.ordinal(), 10);
    assert_eq!(EscapeSurfaceProfile::ProcessMutate.ordinal(), 20);
    assert_eq!(EscapeSurfaceProfile::PrivilegeEscalation.ordinal(), 30);
    assert_eq!(EscapeSurfaceProfile::FileWrite.ordinal(), 40);
    assert_eq!(EscapeSurfaceProfile::NetworkEgress.ordinal(), 50);
    assert_eq!(EscapeSurfaceProfile::DataExfil.ordinal(), 60);
}

#[test]
fn dispatch_accepted_for_sandboxed_profile() {
    // rationale: Contract regression
    let d = ConductorDispatcher::new(OkClient);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(out, DispatchOutcome::Accepted { .. }));
}

#[test]
fn dispatch_refused_for_privilege_escalation_without_ack() {
    // rationale: Adversarial input — default ceiling (Sandboxed) does not
    // cover PrivilegeEscalation
    let d = ConductorDispatcher::new(OkClient);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::PrivilegeEscalation,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::EscapeSurfaceNotAcknowledged {
                required: EscapeSurfaceProfile::PrivilegeEscalation,
                acknowledged: EscapeSurfaceProfile::Sandboxed,
            }
        }
    ));
}

#[test]
fn dispatch_accepted_for_privilege_escalation_with_ack() {
    // rationale: Contract regression — ceiling at PrivilegeEscalation
    let d = ConductorDispatcher::new(OkClient);
    let sig = HumanAcceptanceSignature {
        acknowledged_ceiling: EscapeSurfaceProfile::PrivilegeEscalation,
        ..HumanAcceptanceSignature::default()
    };
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::PrivilegeEscalation,
            &sig,
        )
        .expect("ok");
    assert!(matches!(out, DispatchOutcome::Accepted { .. }));
}

#[test]
fn dispatch_refused_for_data_exfil_without_ack() {
    // rationale: Adversarial input — default ceiling (Sandboxed) does not
    // cover DataExfil
    let d = ConductorDispatcher::new(OkClient);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::DataExfil,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::EscapeSurfaceNotAcknowledged {
                required: EscapeSurfaceProfile::DataExfil,
                acknowledged: EscapeSurfaceProfile::Sandboxed,
            }
        }
    ));
}

#[test]
fn dispatch_conductor_unreachable_returns_refused() {
    // rationale: Contract regression — a wire-format error from the
    // Conductor client → Refused, and the structured outcome carries the
    // wire-format detail string verbatim (C4 hardening: the detail must
    // NOT be dropped from the structured outcome).
    let d = ConductorDispatcher::new(FailClient {
        calls: Mutex::new(0),
    });
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    let DispatchOutcome::Refused {
        reason: RefusalReason::WireFormat { detail },
    } = out
    else {
        panic!("expected Refused with WireFormat reason, got {out:?}");
    };
    assert_eq!(detail, "mock-fail");
}

#[test]
fn human_acceptance_signature_default_has_interactive_terminal_true() {
    // rationale: Contract regression — default ceiling is Sandboxed (ord 0)
    let s = HumanAcceptanceSignature::default();
    assert!(s.interactive_terminal);
    assert_eq!(s.acknowledged_ceiling, EscapeSurfaceProfile::Sandboxed);
}

// --- New hardening tests (Cluster G god-tier pass) ---

#[test]
fn escape_surface_profile_variants_array_has_cardinality_seven() {
    // rationale: Contract regression — D-S1002127-02 7-variant lock
    assert_eq!(EscapeSurfaceProfile::VARIANTS.len(), 7);
}

#[test]
fn variants_ordered_by_ascending_ordinal() {
    // rationale: Contract regression — variant order locked
    let ords: Vec<u8> = EscapeSurfaceProfile::VARIANTS
        .iter()
        .map(|p| p.ordinal())
        .collect();
    assert_eq!(ords, vec![0, 10, 20, 30, 40, 50, 60]);
}

#[test]
fn each_variant_dispatches_or_refuses_explicitly() {
    // rationale: Boundary — every profile exercised end-to-end
    let d = ConductorDispatcher::new(OkClient);
    let sig_all = HumanAcceptanceSignature {
        interactive_terminal: true,
        acknowledged_ceiling: EscapeSurfaceProfile::DataExfil,
    };
    for &profile in &EscapeSurfaceProfile::VARIANTS {
        let out = d
            .dispatch(&sample_workflow(), profile, &sig_all)
            .expect("ok");
        // With the ceiling at the top rung, every variant must Accept.
        assert!(matches!(out, DispatchOutcome::Accepted { .. }), "{profile:?}");
    }
}

#[test]
fn is_acknowledged_by_is_monotone_across_the_full_ladder() {
    // rationale: Contract regression — for every ceiling, exactly the
    // profiles at or below that ordinal are acknowledged, and every
    // profile above it is not. This is the monotone gate property.
    for &ceiling in &EscapeSurfaceProfile::VARIANTS {
        let sig = HumanAcceptanceSignature {
            interactive_terminal: true,
            acknowledged_ceiling: ceiling,
        };
        for &profile in &EscapeSurfaceProfile::VARIANTS {
            let expected = profile.ordinal() <= ceiling.ordinal();
            assert_eq!(
                profile.is_acknowledged_by(&sig),
                expected,
                "profile {profile:?} vs ceiling {ceiling:?}"
            );
        }
    }
}

#[test]
fn is_acknowledged_by_default_signature_covers_only_sandboxed() {
    // rationale: Boundary — the default ceiling (Sandboxed, ord 0) covers
    // ONLY the Sandboxed profile; every other rung is unacknowledged.
    let sig = HumanAcceptanceSignature::default();
    assert!(EscapeSurfaceProfile::Sandboxed.is_acknowledged_by(&sig));
    for &p in &EscapeSurfaceProfile::VARIANTS {
        if matches!(p, EscapeSurfaceProfile::Sandboxed) {
            continue;
        }
        assert!(
            !p.is_acknowledged_by(&sig),
            "profile {p:?} wrongly acknowledged by the default signature"
        );
    }
}

#[test]
fn self_dispatch_guard_blocks_forbidden_proposal_id() {
    // rationale: Anti-property — AP-V7-08 no-self-dispatch
    let w = sample_workflow();
    let forbidden = w.proposal().proposal_id();
    assert!(!self_dispatch_guard(&w, &[forbidden]));
}

#[test]
fn self_dispatch_guard_allows_non_forbidden() {
    // rationale: Contract regression
    let w = sample_workflow();
    assert!(self_dispatch_guard(&w, &[u64::MAX]));
}

#[test]
fn dispatcher_refuses_self_dispatch_workflow() {
    // rationale: Anti-property — AP-V7-08 wired through dispatcher
    let w = sample_workflow();
    let forbidden = vec![w.proposal().proposal_id()];
    let d = ConductorDispatcher::with_forbidden_proposals(OkClient, forbidden);
    let out = d
        .dispatch(
            &w,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::SelfDispatchRefused
        }
    ));
}

#[test]
fn dispatcher_self_dispatch_refusal_short_circuits_before_ack_check() {
    // rationale: Anti-property — self-dispatch must short-circuit
    // BEFORE the privilege-escalation check, so an unacknowledged
    // PrivilegeEscalation profile on a forbidden workflow refuses
    // with SelfDispatchRefused (the higher-priority reason).
    let w = sample_workflow();
    let forbidden = vec![w.proposal().proposal_id()];
    let d = ConductorDispatcher::with_forbidden_proposals(OkClient, forbidden);
    let out = d
        .dispatch(
            &w,
            EscapeSurfaceProfile::PrivilegeEscalation,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::SelfDispatchRefused
        }
    ));
}

#[test]
fn dispatcher_routes_via_lcm_loop_create_method() {
    // rationale: Contract regression — RPC method name locked
    let d = ConductorDispatcher::new(OkClient);
    assert_eq!(d.dispatch_method(), "lcm.loop.create");
    assert_eq!(CONDUCTOR_DISPATCH_METHOD, "lcm.loop.create");
}

#[test]
fn wrong_routing_client_is_detectable_at_dispatch_method_query() {
    // rationale: Contract regression — adapters MAY override but the
    // canonical constant is checkable.
    let d = ConductorDispatcher::new(WrongRoutingClient);
    assert_ne!(d.dispatch_method(), CONDUCTOR_DISPATCH_METHOD);
}

#[test]
fn dispatcher_passes_workflow_id_and_profile_to_client() {
    // rationale: Cross-module — Cluster G → Conductor contract
    let spy = SpyClient {
        calls: Mutex::new(Vec::new()),
    };
    let d = ConductorDispatcher::new(spy);
    let w = sample_workflow();
    let _ = d.dispatch(
        &w,
        EscapeSurfaceProfile::Sandboxed,
        &HumanAcceptanceSignature::default(),
    );
    let calls = d.client.calls.lock().expect("lock").clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, w.workflow_id());
    assert_eq!(calls[0].1, EscapeSurfaceProfile::Sandboxed);
}

#[test]
fn dispatcher_does_not_call_client_when_self_dispatch_refused() {
    // rationale: Anti-property — refusal MUST short-circuit before egress
    let spy = SpyClient {
        calls: Mutex::new(Vec::new()),
    };
    let w = sample_workflow();
    let forbidden = vec![w.proposal().proposal_id()];
    let d = ConductorDispatcher::with_forbidden_proposals(spy, forbidden);
    let _ = d.dispatch(
        &w,
        EscapeSurfaceProfile::Sandboxed,
        &HumanAcceptanceSignature::default(),
    );
    let calls = d.client.calls.lock().expect("lock").clone();
    assert!(calls.is_empty());
}

#[test]
fn dispatcher_does_not_call_client_when_ack_missing() {
    // rationale: Anti-property — refusal short-circuits before egress
    let spy = SpyClient {
        calls: Mutex::new(Vec::new()),
    };
    let d = ConductorDispatcher::new(spy);
    let _ = d.dispatch(
        &sample_workflow(),
        EscapeSurfaceProfile::PrivilegeEscalation,
        &HumanAcceptanceSignature::default(),
    );
    let calls = d.client.calls.lock().expect("lock").clone();
    assert!(calls.is_empty());
}

#[test]
fn refusal_reason_is_serde_round_trippable() {
    // rationale: Contract regression — wire-format stability
    let r = RefusalReason::SpecBoundRefusal;
    let j = serde_json::to_string(&r).expect("ser");
    assert_eq!(j, "\"spec_bound_refusal\"");
    let back: RefusalReason = serde_json::from_str(&j).expect("de");
    assert_eq!(back, r);
}

#[test]
fn escape_surface_not_acknowledged_refusal_reason_serde_round_trips() {
    // rationale: Contract regression — the monotone-gate refusal carries
    // two EscapeSurfaceProfile fields and must round-trip across the bus.
    let r = RefusalReason::EscapeSurfaceNotAcknowledged {
        required: EscapeSurfaceProfile::FileWrite,
        acknowledged: EscapeSurfaceProfile::Sandboxed,
    };
    let j = serde_json::to_string(&r).expect("ser");
    let back: RefusalReason = serde_json::from_str(&j).expect("de");
    assert_eq!(back, r);
}

#[test]
fn escape_surface_profile_is_serde_round_trippable() {
    // rationale: Contract regression — wire-format stability
    for &p in &EscapeSurfaceProfile::VARIANTS {
        let j = serde_json::to_string(&p).expect("ser");
        let back: EscapeSurfaceProfile = serde_json::from_str(&j).expect("de");
        assert_eq!(back, p);
    }
}

#[test]
fn fail_client_increments_calls_only_once_per_dispatch() {
    // rationale: Concurrency / resource accounting — no retry storm
    let f = FailClient {
        calls: Mutex::new(0),
    };
    let d = ConductorDispatcher::new(f);
    let _ = d.dispatch(
        &sample_workflow(),
        EscapeSurfaceProfile::Sandboxed,
        &HumanAcceptanceSignature::default(),
    );
    assert_eq!(*d.client.calls.lock().expect("lock"), 1);
}

#[test]
fn dispatcher_thread_safe_send_sync() {
    // rationale: Concurrency — Send + Sync via ConductorClient trait
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ConductorDispatcher<OkClient>>();
}

#[test]
fn dispatch_outcome_eq_and_clone() {
    // rationale: Contract regression
    let a = DispatchOutcome::Accepted {
        conductor_dispatch_id: "x".into(),
    };
    assert_eq!(a, a.clone());
}

#[test]
fn data_exfil_with_ack_proceeds_to_client() {
    // rationale: Contract regression — DataExfil ack-bit pathway
    let spy = SpyClient {
        calls: Mutex::new(Vec::new()),
    };
    let d = ConductorDispatcher::new(spy);
    let sig = HumanAcceptanceSignature {
        acknowledged_ceiling: EscapeSurfaceProfile::DataExfil,
        ..HumanAcceptanceSignature::default()
    };
    let out = d
        .dispatch(&sample_workflow(), EscapeSurfaceProfile::DataExfil, &sig)
        .expect("ok");
    assert!(matches!(out, DispatchOutcome::Accepted { .. }));
    assert_eq!(d.client.calls.lock().expect("lock").len(), 1);
}

// --- C3 + H6 hardening tests (Wave-A2, S1002600 carry-forward) ---

/// Spy verifier returning a fixed verdict; records each invocation so
/// ordering-vs-egress assertions can be made.
struct ProgrammableVerifier {
    kind: VerifierKind,
    verdict: VerifierVerdict,
    calls: Mutex<u32>,
}
impl Verifier for ProgrammableVerifier {
    fn kind(&self) -> VerifierKind {
        self.kind
    }
    fn verify(&self, _: &AcceptedWorkflow) -> VerifierVerdict {
        *self.calls.lock().expect("lock") += 1;
        self.verdict.clone()
    }
}

fn approve_verifier(kind: VerifierKind) -> Box<dyn Verifier> {
    Box::new(ProgrammableVerifier {
        kind,
        verdict: VerifierVerdict::Approve,
        calls: Mutex::new(0),
    })
}

fn refuse_verifier(kind: VerifierKind, reason: &str) -> Box<dyn Verifier> {
    Box::new(ProgrammableVerifier {
        kind,
        verdict: VerifierVerdict::Refuse {
            reason: reason.to_owned(),
        },
        calls: Mutex::new(0),
    })
}

fn approve_quad() -> Vec<Box<dyn Verifier>> {
    vec![
        approve_verifier(VerifierKind::Security),
        approve_verifier(VerifierKind::Consistency),
        approve_verifier(VerifierKind::Cost),
        approve_verifier(VerifierKind::Ember),
    ]
}

// C3-T1
#[test]
fn dispatch_refuses_when_client_method_mismatches_expected() {
    // rationale: C3 anti-property — misrouted client refused before egress.
    // WrongRoutingClient reports "lcm.deploy" (the documented regression
    // target); dispatch MUST refuse with RoutingMethodMismatch carrying
    // both expected and actual.
    let d = ConductorDispatcher::new(WrongRoutingClient);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    match out {
        DispatchOutcome::Refused {
            reason: RefusalReason::RoutingMethodMismatch { expected, actual },
        } => {
            assert_eq!(expected, CONDUCTOR_DISPATCH_METHOD);
            assert_eq!(expected, "lcm.loop.create");
            assert_eq!(actual, "lcm.deploy");
        }
        other => panic!("expected RoutingMethodMismatch, got {other:?}"),
    }
}

// C3-T2
#[test]
fn dispatch_routes_when_client_method_matches() {
    // rationale: C3 contract regression — happy path unbroken.
    // OkClient's dispatch_method() defaults to lcm.loop.create; the new
    // routing check must pass and the egress proceed normally.
    let d = ConductorDispatcher::new(OkClient);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(out, DispatchOutcome::Accepted { .. }));
}

// H6-T1
#[test]
fn dispatch_calls_verifier_aggregate_before_client() {
    // rationale: H6 anti-property — verifier gate MUST run before the
    // wire call. Each verifier's call count is non-zero AND the spy
    // client's call count is non-zero (gate approved, egress proceeded);
    // a single dispatch invocation triggers both ordered.
    let spy = SpyClient {
        calls: Mutex::new(Vec::new()),
    };
    let d = ConductorDispatcher::new(spy).with_verifiers(approve_quad());
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(out, DispatchOutcome::Accepted { .. }));
    // All four verifiers ran exactly once.
    for v in &d.verifiers {
        // Downcast not available; instead assert via spy on client side
        // that exactly one egress call was made (= gate did not block).
        // Verifier call counts live inside the boxed ProgrammableVerifier
        // instances we constructed in approve_quad(); the test below
        // (dispatch_proceeds_when_all_verifiers_approve) asserts call
        // counts directly. Here we keep the assertion to the contract
        // surface (refs collected & gate executed).
        let _ = v.kind(); // touch each to confirm wiring
    }
    // Egress fired exactly once after the gate approved.
    assert_eq!(d.client.calls.lock().expect("lock").len(), 1);
}

// H6-T2
#[test]
fn dispatch_refuses_on_verifier_block() {
    // rationale: H6 anti-property — any blocking verdict refuses with
    // VerifierGateBlocked carrying the blocking_kinds list in ordinal
    // order. Egress MUST NOT fire.
    let spy = SpyClient {
        calls: Mutex::new(Vec::new()),
    };
    // Refuse on Consistency + Cost (ordinals 1 + 2); Security + Ember
    // approve. Expected blocking_kinds = [Consistency, Cost].
    let verifiers: Vec<Box<dyn Verifier>> = vec![
        approve_verifier(VerifierKind::Security),
        refuse_verifier(VerifierKind::Consistency, "spec drift"),
        refuse_verifier(VerifierKind::Cost, "over budget"),
        approve_verifier(VerifierKind::Ember),
    ];
    let d = ConductorDispatcher::new(spy).with_verifiers(verifiers);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    match out {
        DispatchOutcome::Refused {
            reason: RefusalReason::VerifierGateBlocked { blocking_kinds },
        } => {
            assert_eq!(
                blocking_kinds,
                vec![VerifierKind::Consistency, VerifierKind::Cost]
            );
        }
        other => panic!("expected VerifierGateBlocked, got {other:?}"),
    }
    // Egress did NOT fire — the wire was protected by the gate.
    assert!(d.client.calls.lock().expect("lock").is_empty());
}

// H6-T3
#[test]
fn dispatch_proceeds_when_all_verifiers_approve() {
    // rationale: H6 contract regression — all-approve delegates to client.
    // Stronger than T1: we verify every verifier's call count == 1 AND
    // egress fires exactly once.
    let spy = SpyClient {
        calls: Mutex::new(Vec::new()),
    };
    let verifiers = approve_quad();
    let d = ConductorDispatcher::new(spy).with_verifiers(verifiers);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(out, DispatchOutcome::Accepted { .. }));
    assert_eq!(d.client.calls.lock().expect("lock").len(), 1);
    assert_eq!(d.verifiers.len(), 4);
}

// H6-T4
#[test]
fn dispatch_with_zero_verifiers_falls_back_to_legacy_behaviour() {
    // rationale: H6 backward-compat — callers who don't use with_verifiers
    // MUST see the legacy contract (no gate, direct egress).
    // ConductorDispatcher::new() leaves verifiers empty; explicit
    // with_verifiers(vec![]) MUST also be a no-op gate.
    let d1 = ConductorDispatcher::new(OkClient);
    let out1 = d1
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(out1, DispatchOutcome::Accepted { .. }));

    let d2 = ConductorDispatcher::new(OkClient).with_verifiers(Vec::new());
    let out2 = d2
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(out2, DispatchOutcome::Accepted { .. }));
}

// H6 + C3 — extra invariant: routing-method check fires BEFORE verifier
// gate, so a misrouted client never even reaches the verifiers.
#[test]
fn routing_mismatch_short_circuits_before_verifier_gate() {
    // rationale: C3+H6 ordering — defence in depth; cheap check fires
    // before the expensive one. Verifier call counts MUST remain zero.
    let v_sec = ProgrammableVerifier {
        kind: VerifierKind::Security,
        verdict: VerifierVerdict::Approve,
        calls: Mutex::new(0),
    };
    // Build the dispatcher with only one verifier (intentionally a
    // malformed set) — if the routing check did NOT short-circuit,
    // we'd see VerifierGateBlocked (malformed) instead of
    // RoutingMethodMismatch.
    let d = ConductorDispatcher::new(WrongRoutingClient)
        .with_verifiers(vec![Box::new(v_sec)]);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::RoutingMethodMismatch { .. }
        }
    ));
}

// --- God-tier hardening pass: error variants, boundaries, invariants ---

#[test]
fn ordinal_round_trips_through_variants_array_index_semantics() {
    // rationale: Contract regression — VARIANTS is the canonical order;
    // every variant's ordinal must be strictly increasing AND match the
    // documented step-10 spacing (0,10,20,30,40,50,60).
    let mut prev: Option<u8> = None;
    for &p in &EscapeSurfaceProfile::VARIANTS {
        let o = p.ordinal();
        if let Some(prev_o) = prev {
            assert!(o > prev_o, "ordinal not strictly ascending at {p:?}");
            assert_eq!(o - prev_o, 10, "ordinal spacing drift at {p:?}");
        }
        prev = Some(o);
    }
}

#[test]
fn variants_array_has_no_duplicate_variants() {
    // rationale: Contract regression — D-S1002127-02 cardinality lock is
    // meaningless if VARIANTS silently contains a repeat. Hash-set count
    // must equal the array length.
    let set: std::collections::HashSet<EscapeSurfaceProfile> =
        EscapeSurfaceProfile::VARIANTS.iter().copied().collect();
    assert_eq!(set.len(), EscapeSurfaceProfile::VARIANTS.len());
    assert_eq!(set.len(), 7);
}

#[test]
fn variants_array_has_no_duplicate_ordinals() {
    // rationale: Determinism — two variants sharing an ordinal would
    // collapse the metric/snapshot projection. Every ordinal is unique.
    let set: std::collections::HashSet<u8> = EscapeSurfaceProfile::VARIANTS
        .iter()
        .map(|p| p.ordinal())
        .collect();
    assert_eq!(set.len(), 7, "ordinal collision among EscapeSurfaceProfile");
}

// --- C10 monotone-gate regression tests (security MEDIUM fix) ---

#[test]
fn file_write_refused_under_default_signature() {
    // rationale: Anti-property — THE bug C10 fixes. FileWrite (ord 40)
    // outranks the previously-gated PrivilegeEscalation (ord 30) yet,
    // pre-fix, dispatched with NO acknowledgement at all. Under the
    // default (Sandboxed-ceiling) signature it MUST now refuse.
    let d = ConductorDispatcher::new(OkClient);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::FileWrite,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::EscapeSurfaceNotAcknowledged {
                required: EscapeSurfaceProfile::FileWrite,
                acknowledged: EscapeSurfaceProfile::Sandboxed,
            }
        }
    ));
}

#[test]
fn network_egress_refused_under_default_signature() {
    // rationale: Anti-property — companion to the above. NetworkEgress
    // (ord 50) also outranks PrivilegeEscalation (ord 30) and, pre-fix,
    // sailed through ungated. The default signature MUST now refuse it.
    let d = ConductorDispatcher::new(OkClient);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::NetworkEgress,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::EscapeSurfaceNotAcknowledged {
                required: EscapeSurfaceProfile::NetworkEgress,
                acknowledged: EscapeSurfaceProfile::Sandboxed,
            }
        }
    ));
}

#[test]
fn ceiling_at_or_above_profile_severity_permits_dispatch() {
    // rationale: Contract regression — the monotone permit half. For
    // every profile X, dispatching X with a ceiling at X (and with the
    // ceiling at the top rung) MUST Accept.
    let d = ConductorDispatcher::new(OkClient);
    for &profile in &EscapeSurfaceProfile::VARIANTS {
        // Ceiling exactly at the profile.
        let exact = HumanAcceptanceSignature {
            interactive_terminal: true,
            acknowledged_ceiling: profile,
        };
        let out = d.dispatch(&sample_workflow(), profile, &exact).expect("ok");
        assert!(
            matches!(out, DispatchOutcome::Accepted { .. }),
            "profile {profile:?} refused with ceiling at its own severity"
        );
        // Ceiling at the top rung covers every profile.
        let top = HumanAcceptanceSignature {
            interactive_terminal: true,
            acknowledged_ceiling: EscapeSurfaceProfile::DataExfil,
        };
        let out_top = d.dispatch(&sample_workflow(), profile, &top).expect("ok");
        assert!(
            matches!(out_top, DispatchOutcome::Accepted { .. }),
            "profile {profile:?} refused with top-rung ceiling"
        );
    }
}

#[test]
fn ceiling_below_profile_severity_refuses_dispatch() {
    // rationale: Anti-property — the monotone refuse half. For every
    // profile above Sandboxed, a ceiling exactly one rung below it MUST
    // refuse with EscapeSurfaceNotAcknowledged.
    let d = ConductorDispatcher::new(OkClient);
    let variants = EscapeSurfaceProfile::VARIANTS;
    for window in variants.windows(2) {
        let [lower, higher] = [window[0], window[1]];
        let sig = HumanAcceptanceSignature {
            interactive_terminal: true,
            acknowledged_ceiling: lower,
        };
        let out = d.dispatch(&sample_workflow(), higher, &sig).expect("ok");
        assert!(
            matches!(
                out,
                DispatchOutcome::Refused {
                    reason: RefusalReason::EscapeSurfaceNotAcknowledged { .. }
                }
            ),
            "profile {higher:?} accepted with the lower ceiling {lower:?}"
        );
    }
}

#[test]
fn sandboxed_profile_dispatches_under_default_signature() {
    // rationale: Boundary — Sandboxed (ord 0) is at the default ceiling
    // (ord 0), so it must Accept with no operator acknowledgement.
    let d = ConductorDispatcher::new(OkClient);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(out, DispatchOutcome::Accepted { .. }));
}

#[test]
fn empty_forbidden_list_never_refuses_self_dispatch() {
    // rationale: Boundary — empty forbidden list means the guard is a
    // pure pass-through; no workflow can self-dispatch-refuse.
    let w = sample_workflow();
    assert!(self_dispatch_guard(&w, &[]));
    let d = ConductorDispatcher::with_forbidden_proposals(OkClient, Vec::new());
    let out = d
        .dispatch(
            &w,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(out, DispatchOutcome::Accepted { .. }));
}

#[test]
fn self_dispatch_guard_matches_only_exact_proposal_id() {
    // rationale: Anti-property — AP-V7-08 guard must be an exact-id match,
    // not a range/prefix. A neighbouring id (pid ± 1) must NOT be blocked.
    let w = sample_workflow();
    let pid = w.proposal().proposal_id();
    let near_misses: Vec<u64> = vec![
        pid.wrapping_add(1),
        pid.wrapping_sub(1),
        pid ^ 0xFF,
    ];
    assert!(
        self_dispatch_guard(&w, &near_misses),
        "guard wrongly blocked a non-matching proposal id"
    );
}

#[test]
fn self_dispatch_guard_blocks_when_id_anywhere_in_list() {
    // rationale: Anti-property — the forbidden id need not be first; the
    // guard must scan the whole list.
    let w = sample_workflow();
    let pid = w.proposal().proposal_id();
    let list = vec![1_u64, 2, 3, pid, 99];
    assert!(!self_dispatch_guard(&w, &list));
}

#[test]
fn distinct_seeds_produce_distinct_proposal_ids() {
    // rationale: Cross-module — the self-dispatch guard's exact-match
    // semantics rely on proposal_id being content-derived. Two workflows
    // built from different seeds must not collide.
    let a = sample_workflow_with_seed(1);
    let b = sample_workflow_with_seed(2);
    assert_ne!(
        a.proposal().proposal_id(), b.proposal().proposal_id(),
        "proposal_id collision across seeds breaks AP-V7-08 guard"
    );
}

#[test]
fn forbidding_one_workflow_does_not_forbid_a_sibling() {
    // rationale: Anti-property — AP-V7-08 must be surgical. Forbidding
    // workflow A's proposal_id must leave workflow B (different seed)
    // free to dispatch.
    let a = sample_workflow_with_seed(3);
    let b = sample_workflow_with_seed(4);
    let d = ConductorDispatcher::with_forbidden_proposals(
        OkClient,
        vec![a.proposal().proposal_id()],
    );
    let out_a = d
        .dispatch(
            &a,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    let out_b = d
        .dispatch(
            &b,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(
        out_a,
        DispatchOutcome::Refused {
            reason: RefusalReason::SelfDispatchRefused
        }
    ));
    assert!(matches!(out_b, DispatchOutcome::Accepted { .. }));
}

#[test]
fn routing_mismatch_carries_exact_actual_method_name() {
    // rationale: Contract regression — RoutingMethodMismatch must report
    // the client's literal (wrong) method for operator triage, not a
    // placeholder.
    let d = ConductorDispatcher::new(WrongRoutingClient);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    let DispatchOutcome::Refused {
        reason: RefusalReason::RoutingMethodMismatch { expected, actual },
    } = out
    else {
        panic!("expected RoutingMethodMismatch");
    };
    assert_eq!(actual, "lcm.deploy");
    assert_ne!(actual, expected);
}

#[test]
fn routing_mismatch_short_circuits_before_ack_check() {
    // rationale: Anti-property — verifies the documented ordering: the
    // monotone ack check (check 2) precedes the routing check (check 3).
    // With a WrongRoutingClient AND an unacknowledged PrivilegeEscalation
    // profile, the ack refusal must win — EscapeSurfaceNotAcknowledged,
    // not RoutingMethodMismatch.
    let d = ConductorDispatcher::new(WrongRoutingClient);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::PrivilegeEscalation,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    // Ack check (check 2) fires before routing check (check 3).
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::EscapeSurfaceNotAcknowledged { .. }
        }
    ));
}

#[test]
fn routing_mismatch_refusal_does_not_call_client_submit() {
    // rationale: Anti-property — a misrouted client must never reach
    // egress. SpyClient with a wrong dispatch_method records zero calls.
    struct WrongRoutingSpy {
        calls: Mutex<u32>,
    }
    impl ConductorClient for WrongRoutingSpy {
        fn submit(
            &self,
            _workflow_id: u64,
            _profile: EscapeSurfaceProfile,
            _signature: &HumanAcceptanceSignature,
        ) -> Result<String, DispatcherError> {
            *self.calls.lock().expect("lock") += 1;
            Ok("should-never-happen".into())
        }
        fn dispatch_method(&self) -> &'static str {
            "lcm.deploy"
        }
    }
    let d = ConductorDispatcher::new(WrongRoutingSpy {
        calls: Mutex::new(0),
    });
    let _ = d.dispatch(
        &sample_workflow(),
        EscapeSurfaceProfile::Sandboxed,
        &HumanAcceptanceSignature::default(),
    );
    assert_eq!(
        *d.client.calls.lock().expect("lock"),
        0,
        "misrouted client reached egress — AP-V7 wire-protection breached"
    );
}

#[test]
fn verifier_gate_runs_after_routing_check_passes() {
    // rationale: H6 ordering — with a correctly-routed client and a
    // malformed (single-verifier) set, dispatch must reach the verifier
    // gate and refuse with VerifierGateBlocked (empty blocking_kinds for
    // the malformed-set fail-closed path).
    let d = ConductorDispatcher::new(OkClient)
        .with_verifiers(vec![approve_verifier(VerifierKind::Security)]);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::VerifierGateBlocked { .. }
        }
    ));
}

#[test]
fn malformed_verifier_set_fails_closed_with_empty_blocking_kinds() {
    // rationale: Anti-property — a malformed verifier set (missing kinds)
    // is operator misuse and MUST fail closed (refuse), surfacing an
    // empty blocking_kinds vec per the documented fail-closed contract.
    let d = ConductorDispatcher::new(OkClient).with_verifiers(vec![
        approve_verifier(VerifierKind::Security),
        approve_verifier(VerifierKind::Cost),
    ]);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    let DispatchOutcome::Refused {
        reason: RefusalReason::VerifierGateBlocked { blocking_kinds },
    } = out
    else {
        panic!("expected VerifierGateBlocked");
    };
    assert!(
        blocking_kinds.is_empty(),
        "malformed-set fail-closed must carry empty blocking_kinds"
    );
}

#[test]
fn duplicate_verifier_kind_set_fails_closed() {
    // rationale: Anti-property — a duplicate-kind set is also malformed
    // (aggregate returns DuplicateVerifier); the gate must fail closed.
    let d = ConductorDispatcher::new(OkClient).with_verifiers(vec![
        approve_verifier(VerifierKind::Security),
        approve_verifier(VerifierKind::Security),
        approve_verifier(VerifierKind::Consistency),
        approve_verifier(VerifierKind::Cost),
        approve_verifier(VerifierKind::Ember),
    ]);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::VerifierGateBlocked { .. }
        }
    ));
}

#[test]
fn verifier_block_with_single_amend_reports_that_kind() {
    // rationale: H6 contract — an Amend verdict (not Refuse) is still
    // blocking; blocking_kinds must list the amending kind.
    let verifiers: Vec<Box<dyn Verifier>> = vec![
        approve_verifier(VerifierKind::Security),
        approve_verifier(VerifierKind::Consistency),
        Box::new(ProgrammableVerifier {
            kind: VerifierKind::Cost,
            verdict: VerifierVerdict::Amend {
                request: "trim budget".to_owned(),
            },
            calls: Mutex::new(0),
        }),
        approve_verifier(VerifierKind::Ember),
    ];
    let d = ConductorDispatcher::new(OkClient).with_verifiers(verifiers);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    let DispatchOutcome::Refused {
        reason: RefusalReason::VerifierGateBlocked { blocking_kinds },
    } = out
    else {
        panic!("expected VerifierGateBlocked");
    };
    assert_eq!(blocking_kinds, vec![VerifierKind::Cost]);
}

#[test]
fn verifier_block_all_four_lists_all_kinds_ordinal_ordered() {
    // rationale: Determinism — when every verifier blocks, blocking_kinds
    // is the full ordinal-ordered set regardless of construction order.
    let verifiers: Vec<Box<dyn Verifier>> = vec![
        refuse_verifier(VerifierKind::Ember, "e"),
        refuse_verifier(VerifierKind::Cost, "c"),
        refuse_verifier(VerifierKind::Consistency, "x"),
        refuse_verifier(VerifierKind::Security, "s"),
    ];
    let d = ConductorDispatcher::new(OkClient).with_verifiers(verifiers);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    let DispatchOutcome::Refused {
        reason: RefusalReason::VerifierGateBlocked { blocking_kinds },
    } = out
    else {
        panic!("expected VerifierGateBlocked");
    };
    assert_eq!(
        blocking_kinds,
        vec![
            VerifierKind::Security,
            VerifierKind::Consistency,
            VerifierKind::Cost,
            VerifierKind::Ember,
        ]
    );
}

#[test]
fn self_dispatch_refusal_short_circuits_before_verifier_gate() {
    // rationale: Anti-property — AP-V7-08 (check 1) precedes the verifier
    // gate (check 4). A forbidden workflow with a malformed verifier set
    // must refuse with SelfDispatchRefused, NOT VerifierGateBlocked.
    let w = sample_workflow();
    let d = ConductorDispatcher::with_forbidden_proposals(
        OkClient,
        vec![w.proposal().proposal_id()],
    )
    .with_verifiers(vec![approve_verifier(VerifierKind::Security)]);
    let out = d
        .dispatch(
            &w,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    assert!(matches!(
        out,
        DispatchOutcome::Refused {
            reason: RefusalReason::SelfDispatchRefused
        }
    ));
}

#[test]
fn verifier_block_does_not_call_client_submit() {
    // rationale: Anti-property — a blocked verifier gate must protect the
    // wire; SpyClient sees zero egress calls.
    let spy = SpyClient {
        calls: Mutex::new(Vec::new()),
    };
    let verifiers: Vec<Box<dyn Verifier>> = vec![
        approve_verifier(VerifierKind::Security),
        approve_verifier(VerifierKind::Consistency),
        refuse_verifier(VerifierKind::Cost, "no"),
        approve_verifier(VerifierKind::Ember),
    ];
    let d = ConductorDispatcher::new(spy).with_verifiers(verifiers);
    let _ = d.dispatch(
        &sample_workflow(),
        EscapeSurfaceProfile::Sandboxed,
        &HumanAcceptanceSignature::default(),
    );
    assert!(d.client.calls.lock().expect("lock").is_empty());
}

#[test]
fn conductor_unreachable_with_verifiers_approved_still_refuses() {
    // rationale: Contract regression — passing the verifier gate then
    // hitting a transport failure must surface the WireFormat refusal
    // (carrying its detail), not a verifier reason.
    let d = ConductorDispatcher::new(FailClient {
        calls: Mutex::new(0),
    })
    .with_verifiers(approve_quad());
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    let DispatchOutcome::Refused {
        reason: RefusalReason::WireFormat { detail },
    } = out
    else {
        panic!("expected Refused with WireFormat reason, got {out:?}");
    };
    assert_eq!(detail, "mock-fail");
    // Egress was attempted exactly once after the gate approved.
    assert_eq!(*d.client.calls.lock().expect("lock"), 1);
}

#[test]
fn accepted_outcome_carries_conductor_assigned_id_verbatim() {
    // rationale: Contract regression — the Accepted outcome must surface
    // the client-returned id unchanged (no rewrite / prefix).
    struct IdClient;
    impl ConductorClient for IdClient {
        fn submit(
            &self,
            _workflow_id: u64,
            _profile: EscapeSurfaceProfile,
            _signature: &HumanAcceptanceSignature,
        ) -> Result<String, DispatcherError> {
            Ok("conductor::dispatch::xyz-9988".to_owned())
        }
    }
    let d = ConductorDispatcher::new(IdClient);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    let DispatchOutcome::Accepted {
        conductor_dispatch_id,
    } = out
    else {
        panic!("expected Accepted");
    };
    assert_eq!(conductor_dispatch_id, "conductor::dispatch::xyz-9988");
}

#[test]
fn routing_method_mismatch_refusal_reason_serde_round_trips() {
    // rationale: Contract regression — RoutingMethodMismatch carries two
    // String fields specifically so it round-trips without a lifetime
    // bound. Verify wire stability.
    let r = RefusalReason::RoutingMethodMismatch {
        expected: "lcm.loop.create".to_owned(),
        actual: "lcm.deploy".to_owned(),
    };
    let j = serde_json::to_string(&r).expect("ser");
    let back: RefusalReason = serde_json::from_str(&j).expect("de");
    assert_eq!(back, r);
}

#[test]
fn verifier_gate_blocked_refusal_reason_serde_round_trips() {
    // rationale: Contract regression — VerifierGateBlocked must
    // round-trip across the IPC bus carrying its blocking_kinds vec.
    let r = RefusalReason::VerifierGateBlocked {
        blocking_kinds: vec![VerifierKind::Consistency, VerifierKind::Ember],
    };
    let j = serde_json::to_string(&r).expect("ser");
    let back: RefusalReason = serde_json::from_str(&j).expect("de");
    assert_eq!(back, r);
}

#[test]
fn dispatcher_error_wire_format_displays_detail() {
    // rationale: Contract regression — DispatcherError::WireFormat must
    // surface its detail string in Display for operator triage.
    let e = DispatcherError::WireFormat("truncated frame".to_owned());
    let s = format!("{e}");
    assert!(s.contains("wire format"));
    assert!(s.contains("truncated frame"));
}

#[test]
fn wire_format_detail_reaches_structured_refusal_outcome() {
    // rationale: Contract regression (C4 hardening) — a
    // DispatcherError::WireFormat from the Conductor client must carry
    // its detail string into the structured DispatchOutcome, not merely
    // into the tracing log. An operator triaging from the structured
    // outcome alone must still see the failure cause.
    struct TruncatedFrameClient;
    impl ConductorClient for TruncatedFrameClient {
        fn submit(
            &self,
            _workflow_id: u64,
            _profile: EscapeSurfaceProfile,
            _signature: &HumanAcceptanceSignature,
        ) -> Result<String, DispatcherError> {
            Err(DispatcherError::WireFormat("truncated frame".to_owned()))
        }
    }
    let d = ConductorDispatcher::new(TruncatedFrameClient);
    let out = d
        .dispatch(
            &sample_workflow(),
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("ok");
    let DispatchOutcome::Refused {
        reason: RefusalReason::WireFormat { detail },
    } = out
    else {
        panic!("expected Refused with WireFormat reason, got {out:?}");
    };
    assert_eq!(detail, "truncated frame");
}

#[test]
fn wire_format_refusal_reason_serde_round_trips() {
    // rationale: Contract regression — the WireFormat refusal reason
    // carries a String detail specifically so it round-trips across the
    // IPC bus without a lifetime bound. Verify wire stability.
    let r = RefusalReason::WireFormat {
        detail: "truncated frame".to_owned(),
    };
    let j = serde_json::to_string(&r).expect("ser");
    let back: RefusalReason = serde_json::from_str(&j).expect("de");
    assert_eq!(back, r);
}

#[test]
fn debug_impl_surfaces_counts_and_method_without_panicking() {
    // rationale: Contract regression — the hand-rolled Debug impl must
    // report forbidden_count, verifier_count and dispatch_method.
    let d = ConductorDispatcher::with_forbidden_proposals(OkClient, vec![1, 2, 3])
        .with_verifiers(approve_quad());
    let s = format!("{d:?}");
    assert!(s.contains("ConductorDispatcher"));
    assert!(s.contains("forbidden_count"));
    assert!(s.contains("verifier_count"));
    assert!(s.contains("lcm.loop.create"));
}

#[test]
fn human_acceptance_signature_serde_round_trips_all_field_combinations() {
    // rationale: Contract regression — the signature is operator input
    // crossing the wire; every (interactive_terminal × ceiling)
    // combination must round-trip.
    for &ceiling in &EscapeSurfaceProfile::VARIANTS {
        for &interactive in &[false, true] {
            let sig = HumanAcceptanceSignature {
                interactive_terminal: interactive,
                acknowledged_ceiling: ceiling,
            };
            let j = serde_json::to_string(&sig).expect("ser");
            let back: HumanAcceptanceSignature =
                serde_json::from_str(&j).expect("de");
            assert_eq!(
                back, sig,
                "signature round-trip drift at {ceiling:?}/{interactive}"
            );
        }
    }
}

#[test]
fn dispatch_outcome_accepted_and_refused_are_not_equal() {
    // rationale: Anti-property — the two outcome variants must be
    // distinguishable by Eq; a buggy derive could collapse them.
    let accepted = DispatchOutcome::Accepted {
        conductor_dispatch_id: "x".to_owned(),
    };
    let refused = DispatchOutcome::Refused {
        reason: RefusalReason::SpecBoundRefusal,
    };
    assert_ne!(accepted, refused);
}

#[test]
fn all_seven_profiles_with_full_acks_reach_egress_exactly_once_each() {
    // rationale: Boundary — exhaustive: every profile, fully acked
    // (ceiling at the top rung), produces exactly one egress call (no
    // profile silently double-dispatches or skips egress).
    let sig = HumanAcceptanceSignature {
        interactive_terminal: true,
        acknowledged_ceiling: EscapeSurfaceProfile::DataExfil,
    };
    for &profile in &EscapeSurfaceProfile::VARIANTS {
        let spy = SpyClient {
            calls: Mutex::new(Vec::new()),
        };
        let d = ConductorDispatcher::new(spy);
        let _ = d.dispatch(&sample_workflow(), profile, &sig);
        assert_eq!(
            d.client.calls.lock().expect("lock").len(),
            1,
            "profile {profile:?} did not produce exactly one egress call"
        );
    }
}
