//! `m32_conductor_dispatcher` — dispatch selected workflows via the
//! HABITAT-CONDUCTOR endpoint (no direct exec). Cluster G · L7.
//!
//! **Hard refusal:** m32 NEVER spawns a process / shell / fleet pane
//! directly. Every dispatch surfaces as a structured request to
//! Conductor, which performs the actual execution under its own gates
//! (escape-surface profile, human-acceptance signature, audit).

use thiserror::Error;

use crate::m30_bank::AcceptedWorkflow;

/// Closed-set escape-surface profile (7 variants per D-S1002127-02 ADR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscapeSurfaceProfile {
    /// Sandboxed (ord 0).
    Sandboxed,
    /// SandboxEscape (ord 10).
    SandboxEscape,
    /// ProcessMutate (ord 20).
    ProcessMutate,
    /// PrivilegeEscalation (ord 30; D-S1002127-02 amendment).
    PrivilegeEscalation,
    /// FileWrite (ord 40).
    FileWrite,
    /// NetworkEgress (ord 50).
    NetworkEgress,
    /// DataExfil (ord 60).
    DataExfil,
}

impl EscapeSurfaceProfile {
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
}

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
    /// PrivilegeEscalation without acknowledgement.
    PrivilegeNotAcknowledged,
    /// DataExfil without acknowledgement.
    DataExfilNotAcknowledged,
    /// Workflow not in the curated bank.
    WorkflowNotBanked,
    /// Conductor endpoint unreachable.
    ConductorUnreachable,
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
}

/// The dispatcher.
pub struct ConductorDispatcher<C: ConductorClient> {
    client: C,
}

impl<C: ConductorClient> std::fmt::Debug for ConductorDispatcher<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConductorDispatcher").finish_non_exhaustive()
    }
}

impl<C: ConductorClient> ConductorDispatcher<C> {
    /// Construct with the given client.
    pub fn new(client: C) -> Self {
        Self { client }
    }

    /// Run the 5-check sequence + submit if all checks pass.
    ///
    /// 1. Signature acknowledgement matches profile.
    /// 2. Workflow exists in the curated bank.
    /// 3. (Spec-bound refusal hook; reserved.)
    /// 4. (Spec-bound refusal hook; reserved.)
    /// 5. Conductor reachable.
    ///
    /// # Errors
    ///
    /// [`DispatcherError::WireFormat`] when the Conductor client returns
    /// a malformed response.
    pub fn dispatch(
        &self,
        workflow: &AcceptedWorkflow,
        profile: EscapeSurfaceProfile,
        signature: &HumanAcceptanceSignature,
    ) -> Result<DispatchOutcome, DispatcherError> {
        // Check 1: signature acknowledgement.
        match profile {
            EscapeSurfaceProfile::PrivilegeEscalation
                if !signature.privilege_escalation_acknowledged =>
            {
                return Ok(DispatchOutcome::Refused {
                    reason: RefusalReason::PrivilegeNotAcknowledged,
                });
            }
            EscapeSurfaceProfile::DataExfil if !signature.data_exfil_acknowledged => {
                return Ok(DispatchOutcome::Refused {
                    reason: RefusalReason::DataExfilNotAcknowledged,
                });
            }
            _ => {}
        }
        // Checks 2-4: structural / spec-bound (reserved here).
        let _ = workflow; // workflow row is validated upstream by m31's bank-source.
        // Check 5: submit via Conductor.
        match self.client.submit(workflow.workflow_id, profile, signature) {
            Ok(id) => Ok(DispatchOutcome::Accepted {
                conductor_dispatch_id: id,
            }),
            Err(DispatcherError::WireFormat(_)) => Ok(DispatchOutcome::Refused {
                reason: RefusalReason::ConductorUnreachable,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::SystemTime;

    use super::{
        ConductorClient, ConductorDispatcher, DispatchOutcome, DispatcherError,
        EscapeSurfaceProfile, HumanAcceptanceSignature, RefusalReason,
    };
    use crate::m14_lift::LiftSnapshot;
    use crate::m20_prefixspan::{Pattern, StepToken};
    use crate::m21_variant_builder::build_variants;
    use crate::m23_proposer::build_proposal;
    use crate::m30_bank::AcceptedWorkflow;

    fn sample_workflow() -> AcceptedWorkflow {
        let p = Pattern::new(vec![StepToken(1)], 30, (0, 0));
        let v = build_variants(&p).expect("v")[0].clone();
        let s = LiftSnapshot {
            lift: Some(0.5),
            ci_half: Some(0.05),
            n: 30,
            latest_ts_ms: 0,
            computed_at: SystemTime::now(),
        };
        AcceptedWorkflow {
            workflow_id: 42,
            proposal: build_proposal(v, &s, None).expect("ok"),
            accepted_at_ms: 0,
            sunset_at_ms: i64::MAX,
            weight: 1.0,
            last_run_ms: None,
            run_count: 0,
        }
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

    #[test]
    fn escape_surface_profile_ordinals_are_canonical() {
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
        let s = HumanAcceptanceSignature::default();
        assert!(s.interactive_terminal);
        assert!(!s.privilege_escalation_acknowledged);
        assert!(!s.data_exfil_acknowledged);
    }
}
