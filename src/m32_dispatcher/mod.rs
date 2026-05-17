//! `m32_conductor_dispatcher` — dispatch selected workflows via the
//! HABITAT-CONDUCTOR endpoint (no direct exec). Cluster G · L7.
//!
//! **Hard refusal:** m32 NEVER spawns a process / shell / fleet pane
//! directly. Every dispatch surfaces as a structured request to
//! Conductor, which performs the actual execution under its own gates
//! (escape-surface profile, human-acceptance signature, audit).
//!
//! # Routing
//!
//! Production routing target is the Conductor RPC method
//! [`CONDUCTOR_DISPATCH_METHOD`] (`"lcm.loop.create"`), NOT `lcm.deploy`.
//! The trait [`ConductorClient::submit`] is the only egress; tests mock it.
//!
//! # AP-V7-08 self-dispatch refusal
//!
//! m32 MUST NOT execute the dispatch itself; it composes the
//! [`ConductorClient`] call. Additionally, m32 refuses to dispatch any
//! workflow whose `proposal_id` matches the `forbidden_step_targets` list
//! supplied by the caller. See [`self_dispatch_guard`].
//!
//! # `EscapeSurfaceProfile` cardinality
//!
//! Compile-time-asserted at 7 variants per D-S1002127-02 ADR. Adding or
//! removing a variant fails `cargo check` via [`EscapeSurfaceProfile::VARIANTS`].

use thiserror::Error;

use crate::m30_bank::AcceptedWorkflow;

/// Production Conductor RPC method routed by [`ConductorClient`]
/// implementations. Locked at `"lcm.loop.create"` — `lcm.deploy` is a
/// regression target documented in the v1.3 spec.
pub const CONDUCTOR_DISPATCH_METHOD: &str = "lcm.loop.create";

/// Closed-set escape-surface profile (7 variants per D-S1002127-02 ADR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscapeSurfaceProfile {
    /// Sandboxed (ord 0).
    Sandboxed,
    /// `SandboxEscape` (ord 10).
    SandboxEscape,
    /// `ProcessMutate` (ord 20).
    ProcessMutate,
    /// `PrivilegeEscalation` (ord 30; D-S1002127-02 amendment).
    PrivilegeEscalation,
    /// `FileWrite` (ord 40).
    FileWrite,
    /// `NetworkEgress` (ord 50).
    NetworkEgress,
    /// `DataExfil` (ord 60).
    DataExfil,
}

impl EscapeSurfaceProfile {
    /// Canonical enumeration of all variants in ordinal-ascending order.
    ///
    /// Used by the compile-time cardinality assertion below and by tests
    /// that exhaustively exercise each profile. Adding a variant requires
    /// updating this array, which is then enforced to be the same length
    /// (7) by the const-context assertion.
    pub const VARIANTS: [Self; 7] = [
        Self::Sandboxed,
        Self::SandboxEscape,
        Self::ProcessMutate,
        Self::PrivilegeEscalation,
        Self::FileWrite,
        Self::NetworkEgress,
        Self::DataExfil,
    ];

    /// Stable ordinal.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::Sandboxed => 0,
            Self::SandboxEscape => 10,
            Self::ProcessMutate => 20,
            Self::PrivilegeEscalation => 30,
            Self::FileWrite => 40,
            Self::NetworkEgress => 50,
            Self::DataExfil => 60,
        }
    }

    /// `true` if dispatch requires explicit
    /// [`HumanAcceptanceSignature::privilege_escalation_acknowledged`].
    #[must_use]
    pub const fn requires_privilege_ack(self) -> bool {
        matches!(self, Self::PrivilegeEscalation)
    }

    /// `true` if dispatch requires explicit
    /// [`HumanAcceptanceSignature::data_exfil_acknowledged`].
    #[must_use]
    pub const fn requires_data_exfil_ack(self) -> bool {
        matches!(self, Self::DataExfil)
    }
}

// Compile-time cardinality enforcement (D-S1002127-02): the variant count
// must remain 7. Any drift breaks `cargo check`.
const _: () = {
    assert!(
        EscapeSurfaceProfile::VARIANTS.len() == 7,
        "EscapeSurfaceProfile cardinality drift — D-S1002127-02 ADR locks at 7"
    );
};

/// Human acceptance signature accompanying a dispatch (D-S1002127-02).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HumanAcceptanceSignature {
    /// Operator confirmed interactive terminal.
    pub interactive_terminal: bool,
    /// Required for `PrivilegeEscalation` profile.
    pub privilege_escalation_acknowledged: bool,
    /// Required for `DataExfil` profile.
    pub data_exfil_acknowledged: bool,
}

impl Default for HumanAcceptanceSignature {
    fn default() -> Self {
        Self {
            interactive_terminal: true,
            privilege_escalation_acknowledged: false,
            data_exfil_acknowledged: false,
        }
    }
}

/// Outcome of a dispatch attempt.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DispatchOutcome {
    /// Conductor accepted the request; carries a Conductor-assigned id.
    Accepted {
        /// Conductor's dispatch id.
        conductor_dispatch_id: String,
    },
    /// Refused by the 5-check sequence.
    Refused {
        /// Reason variant.
        reason: RefusalReason,
    },
}

/// Refusal reasons (closed set).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalReason {
    /// `PrivilegeEscalation` without acknowledgement.
    PrivilegeNotAcknowledged,
    /// `DataExfil` without acknowledgement.
    DataExfilNotAcknowledged,
    /// Workflow not in the curated bank.
    WorkflowNotBanked,
    /// Conductor endpoint unreachable.
    ConductorUnreachable,
    /// Self-dispatch (AP-V7-08) refused.
    SelfDispatchRefused,
    /// Generic spec-bound refusal.
    SpecBoundRefusal,
}

/// Dispatcher errors.
#[derive(Debug, Error)]
pub enum DispatcherError {
    /// Conductor wire-format failure.
    #[error("wire format: {0}")]
    WireFormat(String),
}

/// Trait for the Conductor client (mocked in tests; production hits
/// `:8141`).
pub trait ConductorClient: Send + Sync {
    /// Submit a dispatch request; return the Conductor-assigned id.
    ///
    /// # Errors
    ///
    /// [`DispatcherError::WireFormat`] for unparseable response.
    fn submit(
        &self,
        workflow_id: u64,
        profile: EscapeSurfaceProfile,
        signature: &HumanAcceptanceSignature,
    ) -> Result<String, DispatcherError>;

    /// Production RPC method this client invokes. Default is
    /// [`CONDUCTOR_DISPATCH_METHOD`]; tests and adapters may override
    /// (but the production wire MUST route `lcm.loop.create`).
    fn dispatch_method(&self) -> &'static str {
        CONDUCTOR_DISPATCH_METHOD
    }
}

/// AP-V7-08 self-dispatch guard.
///
/// Returns `true` if the workflow is allowed to dispatch (no forbidden
/// proposal ids match); `false` if the workflow targets m32 itself.
///
/// Callers compose this with [`ConductorDispatcher::dispatch`]; the
/// dispatcher also invokes the guard internally when a `forbidden_proposals`
/// list is configured.
#[must_use]
pub fn self_dispatch_guard(workflow: &AcceptedWorkflow, forbidden_proposals: &[u64]) -> bool {
    let pid = workflow.proposal.proposal_id;
    !forbidden_proposals.contains(&pid)
}

/// The dispatcher.
pub struct ConductorDispatcher<C: ConductorClient> {
    client: C,
    /// Caller-supplied list of `proposal_id` values that target m32 itself.
    forbidden_proposals: Vec<u64>,
}

impl<C: ConductorClient> std::fmt::Debug for ConductorDispatcher<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConductorDispatcher")
            .field("forbidden_count", &self.forbidden_proposals.len())
            .field("dispatch_method", &self.client.dispatch_method())
            .finish_non_exhaustive()
    }
}

impl<C: ConductorClient> ConductorDispatcher<C> {
    /// Construct with the given client and no self-dispatch list.
    pub fn new(client: C) -> Self {
        Self {
            client,
            forbidden_proposals: Vec::new(),
        }
    }

    /// Construct with explicit AP-V7-08 forbidden-proposal ids. The
    /// dispatcher refuses any workflow whose `proposal_id` matches one of
    /// these values.
    pub fn with_forbidden_proposals(client: C, forbidden: Vec<u64>) -> Self {
        Self {
            client,
            forbidden_proposals: forbidden,
        }
    }

    /// Run the 5-check sequence + submit if all checks pass.
    ///
    /// 1. AP-V7-08 self-dispatch refusal.
    /// 2. Signature acknowledgement matches profile.
    /// 3. (Spec-bound refusal hook; reserved for m30 bank-membership check.)
    /// 4. (Spec-bound refusal hook; reserved for m11 sunset gate.)
    /// 5. Conductor reachable.
    ///
    /// # Errors
    ///
    /// This method only returns [`DispatcherError`] for explicit caller
    /// faults; all refusals (including transport errors) are folded into
    /// [`DispatchOutcome::Refused`].
    pub fn dispatch(
        &self,
        workflow: &AcceptedWorkflow,
        profile: EscapeSurfaceProfile,
        signature: &HumanAcceptanceSignature,
    ) -> Result<DispatchOutcome, DispatcherError> {
        // Check 1 — AP-V7-08 self-dispatch refusal.
        if !self_dispatch_guard(workflow, &self.forbidden_proposals) {
            tracing::warn!(
                workflow_id = workflow.workflow_id,
                proposal_id = workflow.proposal.proposal_id,
                "m32: AP-V7-08 self-dispatch refused"
            );
            return Ok(DispatchOutcome::Refused {
                reason: RefusalReason::SelfDispatchRefused,
            });
        }
        // Check 2 — signature acknowledgement per profile.
        if profile.requires_privilege_ack() && !signature.privilege_escalation_acknowledged {
            return Ok(DispatchOutcome::Refused {
                reason: RefusalReason::PrivilegeNotAcknowledged,
            });
        }
        if profile.requires_data_exfil_ack() && !signature.data_exfil_acknowledged {
            return Ok(DispatchOutcome::Refused {
                reason: RefusalReason::DataExfilNotAcknowledged,
            });
        }
        // Checks 3-4 reserved for upstream (bank membership / sunset gate).
        // The workflow id is read here so that the upstream contract is
        // explicit at this site; it is the only egress argument the
        // Conductor sees aside from profile + signature.
        let workflow_id = workflow.workflow_id;
        // Check 5 — submit via Conductor.
        match self.client.submit(workflow_id, profile, signature) {
            Ok(id) => Ok(DispatchOutcome::Accepted {
                conductor_dispatch_id: id,
            }),
            Err(DispatcherError::WireFormat(detail)) => {
                // Surface the detail via tracing — folding into Refused
                // discards the string from the public outcome surface, so
                // we MUST log it here or the operator cannot triage.
                tracing::warn!(
                    workflow_id,
                    method = %self.client.dispatch_method(),
                    detail = %detail,
                    "m32: conductor unreachable"
                );
                Ok(DispatchOutcome::Refused {
                    reason: RefusalReason::ConductorUnreachable,
                })
            }
        }
    }

    /// Production routing target inspected by adapters and tests.
    pub fn dispatch_method(&self) -> &'static str {
        self.client.dispatch_method()
    }
}

#[cfg(test)]
mod tests {
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
        AcceptedWorkflow {
            workflow_id: u64::from(42 + seed),
            proposal: build_proposal(v, &s, None).expect("ok"),
            accepted_at_ms: 0,
            sunset_at_ms: i64::MAX,
            weight: 1.0,
            last_run_ms: None,
            run_count: 0,
        }
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
        // rationale: Adversarial input — missing ack bit
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
                reason: RefusalReason::PrivilegeNotAcknowledged
            }
        ));
    }

    #[test]
    fn dispatch_accepted_for_privilege_escalation_with_ack() {
        // rationale: Contract regression — ack-bit pathway
        let d = ConductorDispatcher::new(OkClient);
        let sig = HumanAcceptanceSignature {
            privilege_escalation_acknowledged: true,
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
        // rationale: Adversarial input — missing ack bit
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
                reason: RefusalReason::DataExfilNotAcknowledged
            }
        ));
    }

    #[test]
    fn dispatch_conductor_unreachable_returns_refused() {
        // rationale: Contract regression — wire-format error → Refused
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
        assert!(matches!(
            out,
            DispatchOutcome::Refused {
                reason: RefusalReason::ConductorUnreachable
            }
        ));
    }

    #[test]
    fn human_acceptance_signature_default_has_interactive_terminal_true() {
        // rationale: Contract regression
        let s = HumanAcceptanceSignature::default();
        assert!(s.interactive_terminal);
        assert!(!s.privilege_escalation_acknowledged);
        assert!(!s.data_exfil_acknowledged);
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
            privilege_escalation_acknowledged: true,
            data_exfil_acknowledged: true,
        };
        for &profile in &EscapeSurfaceProfile::VARIANTS {
            let out = d
                .dispatch(&sample_workflow(), profile, &sig_all)
                .expect("ok");
            // With both ack bits set, every variant must Accept.
            assert!(matches!(out, DispatchOutcome::Accepted { .. }), "{profile:?}");
        }
    }

    #[test]
    fn requires_privilege_ack_only_for_privilege_escalation() {
        // rationale: Contract regression — ack predicate
        for &p in &EscapeSurfaceProfile::VARIANTS {
            let expected = matches!(p, EscapeSurfaceProfile::PrivilegeEscalation);
            assert_eq!(p.requires_privilege_ack(), expected);
        }
    }

    #[test]
    fn requires_data_exfil_ack_only_for_data_exfil() {
        // rationale: Contract regression — ack predicate
        for &p in &EscapeSurfaceProfile::VARIANTS {
            let expected = matches!(p, EscapeSurfaceProfile::DataExfil);
            assert_eq!(p.requires_data_exfil_ack(), expected);
        }
    }

    #[test]
    fn self_dispatch_guard_blocks_forbidden_proposal_id() {
        // rationale: Anti-property — AP-V7-08 no-self-dispatch
        let w = sample_workflow();
        let forbidden = w.proposal.proposal_id;
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
        let forbidden = vec![w.proposal.proposal_id];
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
        let forbidden = vec![w.proposal.proposal_id];
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
        assert_eq!(calls[0].0, w.workflow_id);
        assert_eq!(calls[0].1, EscapeSurfaceProfile::Sandboxed);
    }

    #[test]
    fn dispatcher_does_not_call_client_when_self_dispatch_refused() {
        // rationale: Anti-property — refusal MUST short-circuit before egress
        let spy = SpyClient {
            calls: Mutex::new(Vec::new()),
        };
        let w = sample_workflow();
        let forbidden = vec![w.proposal.proposal_id];
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
        let r = RefusalReason::PrivilegeNotAcknowledged;
        let j = serde_json::to_string(&r).expect("ser");
        assert_eq!(j, "\"privilege_not_acknowledged\"");
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
            data_exfil_acknowledged: true,
            ..HumanAcceptanceSignature::default()
        };
        let out = d
            .dispatch(&sample_workflow(), EscapeSurfaceProfile::DataExfil, &sig)
            .expect("ok");
        assert!(matches!(out, DispatchOutcome::Accepted { .. }));
        assert_eq!(d.client.calls.lock().expect("lock").len(), 1);
    }
}
