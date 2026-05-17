//! `m33_verifier` — 4-agent verification gate before m32 dispatch.
//! Cluster G · L7.
//!
//! Four named verifiers (security / consistency / cost / ember) are
//! collected; a workflow proceeds only when **all four** agree. Any
//! single REFUSE/AMEND blocks dispatch.

use thiserror::Error;

use crate::m30_bank::AcceptedWorkflow;

/// Verifier identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifierKind {
    /// Security: AP30 / EscapeSurfaceProfile checks.
    Security,
    /// Consistency: workflow vs prior bank entries.
    Consistency,
    /// Cost: budget projection.
    Cost,
    /// Ember: 7-trait Ember rubric on user-facing artefacts.
    Ember,
}

impl VerifierKind {
    /// Stable identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Security => "security",
            Self::Consistency => "consistency",
            Self::Cost => "cost",
            Self::Ember => "ember",
        }
    }
}

/// One verifier's verdict.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifierVerdict {
    /// Approve — proceed.
    Approve,
    /// Refuse — do not proceed.
    Refuse {
        /// Free-text reason.
        reason: String,
    },
    /// Amend — proceed only after caller addresses the request.
    Amend {
        /// Free-text amendment request.
        request: String,
    },
}

/// Aggregated verification result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregateVerdict {
    /// All four verifiers approved.
    AllApprove,
    /// At least one refused or requested amendment.
    Blocked {
        /// Per-verifier outcomes for the operator.
        per_verifier: Vec<(VerifierKind, VerifierVerdict)>,
    },
}

/// Verifier errors.
#[derive(Debug, Error)]
pub enum VerifierError {
    /// A required verifier was missing from the input.
    #[error("missing verifier {:?}", .0)]
    MissingVerifier(VerifierKind),
}

/// Trait implemented by each individual verifier.
pub trait Verifier: Send + Sync {
    /// Identifier.
    fn kind(&self) -> VerifierKind;
    /// Render a verdict for the given workflow.
    fn verify(&self, workflow: &AcceptedWorkflow) -> VerifierVerdict;
}

/// Run all four verifiers and aggregate.
///
/// `verifiers` MUST include exactly one of each `VerifierKind`.
///
/// # Errors
///
/// [`VerifierError::MissingVerifier`] when a required kind is absent.
pub fn aggregate(
    verifiers: &[&dyn Verifier],
    workflow: &AcceptedWorkflow,
) -> Result<AggregateVerdict, VerifierError> {
    let required = [
        VerifierKind::Security,
        VerifierKind::Consistency,
        VerifierKind::Cost,
        VerifierKind::Ember,
    ];
    for kind in required {
        if !verifiers.iter().any(|v| v.kind() == kind) {
            return Err(VerifierError::MissingVerifier(kind));
        }
    }
    let outcomes: Vec<(VerifierKind, VerifierVerdict)> = verifiers
        .iter()
        .map(|v| (v.kind(), v.verify(workflow)))
        .collect();
    let any_block = outcomes
        .iter()
        .any(|(_, v)| !matches!(v, VerifierVerdict::Approve));
    if any_block {
        Ok(AggregateVerdict::Blocked {
            per_verifier: outcomes,
        })
    } else {
        Ok(AggregateVerdict::AllApprove)
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::{
        aggregate, AggregateVerdict, Verifier, VerifierError, VerifierKind, VerifierVerdict,
    };
    use crate::m14_lift::LiftSnapshot;
    use crate::m20_prefixspan::{Pattern, StepToken};
    use crate::m21_variant_builder::build_variants;
    use crate::m23_proposer::build_proposal;
    use crate::m30_bank::AcceptedWorkflow;

    fn sample() -> AcceptedWorkflow {
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
            workflow_id: 1,
            proposal: build_proposal(v, &s, None).expect("ok"),
            accepted_at_ms: 0,
            sunset_at_ms: i64::MAX,
            weight: 1.0,
            last_run_ms: None,
            run_count: 0,
        }
    }

    struct ApproveAll {
        kind: VerifierKind,
    }
    impl Verifier for ApproveAll {
        fn kind(&self) -> VerifierKind {
            self.kind
        }
        fn verify(&self, _: &AcceptedWorkflow) -> VerifierVerdict {
            VerifierVerdict::Approve
        }
    }

    struct RefuseOne {
        kind: VerifierKind,
    }
    impl Verifier for RefuseOne {
        fn kind(&self) -> VerifierKind {
            self.kind
        }
        fn verify(&self, _: &AcceptedWorkflow) -> VerifierVerdict {
            VerifierVerdict::Refuse {
                reason: "test".into(),
            }
        }
    }

    fn approve_quad() -> [Box<dyn Verifier>; 4] {
        [
            Box::new(ApproveAll {
                kind: VerifierKind::Security,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Consistency,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Cost,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Ember,
            }),
        ]
    }

    #[test]
    fn all_approve_yields_all_approve_verdict() {
        let verifiers = approve_quad();
        let refs: Vec<&dyn Verifier> = verifiers.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample()).expect("ok");
        assert_eq!(r, AggregateVerdict::AllApprove);
    }

    #[test]
    fn one_refusal_blocks() {
        let v: [Box<dyn Verifier>; 4] = [
            Box::new(RefuseOne {
                kind: VerifierKind::Security,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Consistency,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Cost,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Ember,
            }),
        ];
        let refs: Vec<&dyn Verifier> = v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample()).expect("ok");
        assert!(matches!(r, AggregateVerdict::Blocked { .. }));
    }

    #[test]
    fn missing_verifier_yields_error() {
        let v: Vec<Box<dyn Verifier>> = vec![
            Box::new(ApproveAll {
                kind: VerifierKind::Security,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Consistency,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Cost,
            }),
            // Missing Ember.
        ];
        let refs: Vec<&dyn Verifier> = v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample());
        assert!(matches!(r, Err(VerifierError::MissingVerifier(VerifierKind::Ember))));
    }

    #[test]
    fn verifier_kind_as_str_stability() {
        assert_eq!(VerifierKind::Security.as_str(), "security");
        assert_eq!(VerifierKind::Ember.as_str(), "ember");
    }
}
