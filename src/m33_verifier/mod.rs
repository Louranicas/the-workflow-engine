//! `m33_verifier` — 4-agent verification gate before m32 dispatch.
//! Cluster G · L7.
//!
//! Four named verifiers (security / consistency / cost / ember) are
//! collected; a workflow proceeds only when **all four** agree. Any
//! single REFUSE/AMEND blocks dispatch.
//!
//! # Aggregation determinism
//!
//! Outcomes are sorted by [`VerifierKind::ordinal`] before being returned to
//! the operator, so a Blocked verdict's `per_verifier` list is the same
//! regardless of caller slice ordering. This is the m33 spec § 5
//! "human-comparable refusal log" invariant.
//!
//! # AP-V7-08 pairing
//!
//! Per m33 spec § 6, m33 must refuse to verify a workflow whose steps
//! target m32 itself. This is operationalised by composing the m33
//! verifier set with [`crate::m32_dispatcher::self_dispatch_guard`] in the
//! caller; m33 itself does not own the m32 step-kind taxonomy.

use thiserror::Error;

use crate::m30_bank::AcceptedWorkflow;

/// Verifier identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifierKind {
    /// Security: AP30 / `EscapeSurfaceProfile` checks.
    Security,
    /// Consistency: workflow vs prior bank entries.
    Consistency,
    /// Cost: budget projection.
    Cost,
    /// Ember: 7-trait Ember rubric on user-facing artefacts.
    Ember,
}

impl VerifierKind {
    /// Canonical enumeration, in ordinal-ascending order. Used for the
    /// compile-time cardinality assertion below and for deterministic
    /// aggregation in [`aggregate`].
    pub const VARIANTS: [Self; 4] =
        [Self::Security, Self::Consistency, Self::Cost, Self::Ember];

    /// Stable ordinal projection for metrics, snapshot stability, and the
    /// aggregator's deterministic sort. `Security = 0`, `Consistency = 1`,
    /// `Cost = 2`, `Ember = 3`.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::Security => 0,
            Self::Consistency => 1,
            Self::Cost => 2,
            Self::Ember => 3,
        }
    }

    /// Stable identifier for log lines and metric labels.
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

// Compile-time cardinality enforcement: m33 contract is exactly 4
// verifiers. Adding or removing a kind fails `cargo check`.
const _: () = {
    assert!(
        VerifierKind::VARIANTS.len() == 4,
        "VerifierKind cardinality drift — m33 contract is 4 verifiers"
    );
};

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

impl VerifierVerdict {
    /// `true` if this verdict blocks dispatch (Refuse or Amend).
    #[must_use]
    pub const fn is_blocking(&self) -> bool {
        !matches!(self, Self::Approve)
    }
}

/// Aggregated verification result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregateVerdict {
    /// All four verifiers approved.
    AllApprove,
    /// At least one refused or requested amendment.
    Blocked {
        /// Per-verifier outcomes for the operator, sorted by
        /// [`VerifierKind::ordinal`].
        per_verifier: Vec<(VerifierKind, VerifierVerdict)>,
    },
}

/// Verifier errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum VerifierError {
    /// A required verifier was missing from the input.
    #[error("missing verifier {:?}", .0)]
    MissingVerifier(VerifierKind),
    /// A verifier kind appeared more than once in the input.
    #[error("duplicate verifier {:?}", .0)]
    DuplicateVerifier(VerifierKind),
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
/// `verifiers` MUST include exactly one of each [`VerifierKind`].
///
/// Output ordering: `per_verifier` is sorted by [`VerifierKind::ordinal`] so
/// that the refusal log is deterministic regardless of caller slice order.
///
/// # Errors
///
/// - [`VerifierError::MissingVerifier`] when a required kind is absent.
/// - [`VerifierError::DuplicateVerifier`] when a kind appears more than once.
pub fn aggregate(
    verifiers: &[&dyn Verifier],
    workflow: &AcceptedWorkflow,
) -> Result<AggregateVerdict, VerifierError> {
    // Check for duplicates FIRST so a duplicate-of-the-only-supplied-kind
    // doesn't accidentally read as "all required present".
    for &required in &VerifierKind::VARIANTS {
        let count = verifiers.iter().filter(|v| v.kind() == required).count();
        if count > 1 {
            return Err(VerifierError::DuplicateVerifier(required));
        }
    }
    for &required in &VerifierKind::VARIANTS {
        if !verifiers.iter().any(|v| v.kind() == required) {
            return Err(VerifierError::MissingVerifier(required));
        }
    }
    let mut outcomes: Vec<(VerifierKind, VerifierVerdict)> = verifiers
        .iter()
        .map(|v| (v.kind(), v.verify(workflow)))
        .collect();
    // Deterministic sort by kind ordinal — anti-caller-slice-order.
    outcomes.sort_by_key(|(k, _)| k.ordinal());
    let any_block = outcomes.iter().any(|(_, v)| v.is_blocking());
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

    struct AmendOne {
        kind: VerifierKind,
    }
    impl Verifier for AmendOne {
        fn kind(&self) -> VerifierKind {
            self.kind
        }
        fn verify(&self, _: &AcceptedWorkflow) -> VerifierVerdict {
            VerifierVerdict::Amend {
                request: "please specify".into(),
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

    // --- Pre-existing tests preserved verbatim ---

    #[test]
    fn all_approve_yields_all_approve_verdict() {
        // rationale: Contract regression
        let verifiers = approve_quad();
        let refs: Vec<&dyn Verifier> = verifiers.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample()).expect("ok");
        assert_eq!(r, AggregateVerdict::AllApprove);
    }

    #[test]
    fn one_refusal_blocks() {
        // rationale: Contract regression
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
        // rationale: Contract regression
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
        assert!(matches!(
            r,
            Err(VerifierError::MissingVerifier(VerifierKind::Ember))
        ));
    }

    #[test]
    fn verifier_kind_as_str_stability() {
        // rationale: Contract regression — log-label stability
        assert_eq!(VerifierKind::Security.as_str(), "security");
        assert_eq!(VerifierKind::Ember.as_str(), "ember");
    }

    // --- New hardening tests (Cluster G god-tier pass) ---

    #[test]
    fn variants_array_cardinality_four() {
        // rationale: Contract regression — m33 4-verifier contract
        assert_eq!(VerifierKind::VARIANTS.len(), 4);
    }

    #[test]
    fn variants_ordered_by_ascending_ordinal() {
        // rationale: Contract regression
        let ords: Vec<u8> = VerifierKind::VARIANTS
            .iter()
            .map(|k| k.ordinal())
            .collect();
        assert_eq!(ords, vec![0, 1, 2, 3]);
    }

    #[test]
    fn one_amend_blocks_as_well_as_refuse() {
        // rationale: Anti-property — Amend is also a block
        let v: [Box<dyn Verifier>; 4] = [
            Box::new(ApproveAll {
                kind: VerifierKind::Security,
            }),
            Box::new(AmendOne {
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
        match r {
            AggregateVerdict::Blocked { per_verifier } => {
                assert!(per_verifier
                    .iter()
                    .any(|(_, v)| matches!(v, VerifierVerdict::Amend { .. })));
            }
            AggregateVerdict::AllApprove => panic!("expected Blocked, got AllApprove"),
        }
    }

    #[test]
    fn all_refuse_blocks_and_carries_each_refusal() {
        // rationale: Adversarial input — all-verifiers-refuse
        let v: [Box<dyn Verifier>; 4] = [
            Box::new(RefuseOne {
                kind: VerifierKind::Security,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Consistency,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Cost,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Ember,
            }),
        ];
        let refs: Vec<&dyn Verifier> = v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample()).expect("ok");
        match r {
            AggregateVerdict::Blocked { per_verifier } => {
                assert_eq!(per_verifier.len(), 4);
                for (_, v) in &per_verifier {
                    assert!(matches!(v, VerifierVerdict::Refuse { .. }));
                }
            }
            AggregateVerdict::AllApprove => panic!("expected Blocked, got AllApprove"),
        }
    }

    #[test]
    fn duplicate_verifier_kind_yields_typed_error() {
        // rationale: Adversarial input — duplicate kinds rejected
        let v: Vec<Box<dyn Verifier>> = vec![
            Box::new(ApproveAll {
                kind: VerifierKind::Security,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Security,
            }), // duplicate
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
        let r = aggregate(&refs, &sample());
        assert!(matches!(
            r,
            Err(VerifierError::DuplicateVerifier(VerifierKind::Security))
        ));
    }

    #[test]
    fn missing_each_kind_individually_yields_correct_error() {
        // rationale: Contract regression — each missing-kind path
        for &missing in &VerifierKind::VARIANTS {
            let v: Vec<Box<dyn Verifier>> = VerifierKind::VARIANTS
                .iter()
                .filter(|k| **k != missing)
                .map(|k| Box::new(ApproveAll { kind: *k }) as Box<dyn Verifier>)
                .collect();
            let refs: Vec<&dyn Verifier> =
                v.iter().map(std::convert::AsRef::as_ref).collect();
            let r = aggregate(&refs, &sample());
            assert!(
                matches!(r, Err(VerifierError::MissingVerifier(k)) if k == missing),
                "missing kind {missing:?} not detected"
            );
        }
    }

    #[test]
    fn blocked_outcomes_deterministically_ordered_by_kind_ordinal() {
        // rationale: Determinism — anti-caller-slice-order
        let v: Vec<Box<dyn Verifier>> = vec![
            Box::new(RefuseOne {
                kind: VerifierKind::Ember,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Cost,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Consistency,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Security,
            }),
        ];
        let refs: Vec<&dyn Verifier> = v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample()).expect("ok");
        match r {
            AggregateVerdict::Blocked { per_verifier } => {
                let kinds: Vec<VerifierKind> =
                    per_verifier.iter().map(|(k, _)| *k).collect();
                assert_eq!(
                    kinds,
                    vec![
                        VerifierKind::Security,
                        VerifierKind::Consistency,
                        VerifierKind::Cost,
                        VerifierKind::Ember,
                    ]
                );
            }
            AggregateVerdict::AllApprove => panic!("expected Blocked, got AllApprove"),
        }
    }

    #[test]
    fn aggregation_parity_across_input_permutations() {
        // rationale: Determinism — permutations produce identical Blocked log
        let baseline_v: Vec<Box<dyn Verifier>> = vec![
            Box::new(RefuseOne {
                kind: VerifierKind::Security,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Consistency,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Cost,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Ember,
            }),
        ];
        let refs: Vec<&dyn Verifier> =
            baseline_v.iter().map(std::convert::AsRef::as_ref).collect();
        let baseline = aggregate(&refs, &sample()).expect("ok");

        let permuted_v: Vec<Box<dyn Verifier>> = vec![
            Box::new(ApproveAll {
                kind: VerifierKind::Ember,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Cost,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Consistency,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Security,
            }),
        ];
        let refs: Vec<&dyn Verifier> =
            permuted_v.iter().map(std::convert::AsRef::as_ref).collect();
        let permuted = aggregate(&refs, &sample()).expect("ok");
        assert_eq!(baseline, permuted);
    }

    #[test]
    fn verifier_verdict_is_blocking_for_refuse_and_amend_only() {
        // rationale: Anti-property — `is_blocking` predicate locked
        assert!(!VerifierVerdict::Approve.is_blocking());
        assert!(VerifierVerdict::Refuse {
            reason: "x".into()
        }
        .is_blocking());
        assert!(VerifierVerdict::Amend {
            request: "y".into()
        }
        .is_blocking());
    }

    #[test]
    fn verifier_kind_serde_round_trip() {
        // rationale: Contract regression — wire-format stability
        for &k in &VerifierKind::VARIANTS {
            let j = serde_json::to_string(&k).expect("ser");
            let back: VerifierKind = serde_json::from_str(&j).expect("de");
            assert_eq!(back, k);
        }
    }

    #[test]
    fn verifier_verdict_serde_round_trip() {
        // rationale: Contract regression — wire-format stability
        let vs = [
            VerifierVerdict::Approve,
            VerifierVerdict::Refuse {
                reason: "r".into(),
            },
            VerifierVerdict::Amend {
                request: "a".into(),
            },
        ];
        for v in &vs {
            let j = serde_json::to_string(v).expect("ser");
            let back: VerifierVerdict = serde_json::from_str(&j).expect("de");
            assert_eq!(back, *v);
        }
    }

    #[test]
    fn empty_verifiers_yields_missing_security_first() {
        // rationale: Boundary — empty slice
        let refs: Vec<&dyn Verifier> = Vec::new();
        let r = aggregate(&refs, &sample());
        assert!(matches!(
            r,
            Err(VerifierError::MissingVerifier(VerifierKind::Security))
        ));
    }

    #[test]
    fn approve_then_refuse_aggregates_to_blocked_with_full_log() {
        // rationale: Cross-module — Cluster G → m32 dispatch gate
        let v: [Box<dyn Verifier>; 4] = [
            Box::new(ApproveAll {
                kind: VerifierKind::Security,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Consistency,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Cost,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Ember,
            }),
        ];
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample()).expect("ok");
        match r {
            AggregateVerdict::Blocked { per_verifier } => {
                // All 4 outcomes recorded even when only one blocks.
                assert_eq!(per_verifier.len(), 4);
                let approves =
                    per_verifier.iter().filter(|(_, v)| !v.is_blocking()).count();
                assert_eq!(approves, 3);
            }
            AggregateVerdict::AllApprove => panic!("expected Blocked, got AllApprove"),
        }
    }

    #[test]
    fn verifier_error_is_eq_and_displayable() {
        // rationale: Contract regression
        let e = VerifierError::MissingVerifier(VerifierKind::Ember);
        assert_eq!(e, VerifierError::MissingVerifier(VerifierKind::Ember));
        let s = format!("{e}");
        assert!(s.contains("Ember"));
    }

    // --- God-tier hardening pass: 4-agent gate invariant, errors, ordering ---

    /// Verifier that records into a shared log the `workflow_id` it saw —
    /// used to prove `aggregate` forwards the workflow to every verifier.
    /// The log is an `Arc<Mutex<..>>` cloned out before boxing so the test
    /// can inspect it without downcasting the trait object.
    struct WitnessVerifier {
        kind: VerifierKind,
        seen: std::sync::Arc<std::sync::Mutex<Vec<(VerifierKind, u64)>>>,
    }
    impl Verifier for WitnessVerifier {
        fn kind(&self) -> VerifierKind {
            self.kind
        }
        fn verify(&self, workflow: &AcceptedWorkflow) -> VerifierVerdict {
            self.seen
                .lock()
                .expect("lock")
                .push((self.kind, workflow.workflow_id));
            VerifierVerdict::Approve
        }
    }

    fn quad_with(refuser: VerifierKind) -> [Box<dyn Verifier>; 4] {
        let mk = |k: VerifierKind| -> Box<dyn Verifier> {
            if k == refuser {
                Box::new(RefuseOne { kind: k })
            } else {
                Box::new(ApproveAll { kind: k })
            }
        };
        [
            mk(VerifierKind::Security),
            mk(VerifierKind::Consistency),
            mk(VerifierKind::Cost),
            mk(VerifierKind::Ember),
        ]
    }

    #[test]
    fn any_single_kind_refusing_blocks_the_gate() {
        // rationale: Anti-property — the 4-agent gate's core invariant: a
        // single REFUSE from ANY of the four kinds blocks dispatch. Exercise
        // each kind individually.
        for &k in &VerifierKind::VARIANTS {
            let v = quad_with(k);
            let refs: Vec<&dyn Verifier> =
                v.iter().map(std::convert::AsRef::as_ref).collect();
            let r = aggregate(&refs, &sample()).expect("ok");
            assert!(
                matches!(r, AggregateVerdict::Blocked { .. }),
                "refusal from {k:?} did not block the gate"
            );
        }
    }

    #[test]
    fn gate_passes_only_when_all_four_approve() {
        // rationale: Contract regression — the 4-agent gate's positive case
        // requires unanimity. Removing any approval (replacing with refuse)
        // must flip the verdict.
        let v = approve_quad();
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        assert_eq!(
            aggregate(&refs, &sample()).expect("ok"),
            AggregateVerdict::AllApprove
        );
    }

    #[test]
    fn aggregate_forwards_workflow_to_every_verifier() {
        // rationale: Cross-module — aggregate must call verify() on all four
        // verifiers exactly once with the SAME workflow. A shared log records
        // (kind, workflow_id) per call; the test asserts all 4 kinds appear
        // exactly once carrying the correct id.
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let v: Vec<Box<dyn Verifier>> = VerifierKind::VARIANTS
            .iter()
            .map(|&k| {
                Box::new(WitnessVerifier {
                    kind: k,
                    seen: std::sync::Arc::clone(&log),
                }) as Box<dyn Verifier>
            })
            .collect();
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        let wf = sample();
        let expected_id = wf.workflow_id;
        let r = aggregate(&refs, &wf).expect("ok");
        assert_eq!(r, AggregateVerdict::AllApprove);
        let mut seen = log.lock().expect("lock").clone();
        seen.sort_by_key(|(k, _)| k.ordinal());
        assert_eq!(
            seen,
            vec![
                (VerifierKind::Security, expected_id),
                (VerifierKind::Consistency, expected_id),
                (VerifierKind::Cost, expected_id),
                (VerifierKind::Ember, expected_id),
            ]
        );
    }

    #[test]
    fn duplicate_detected_before_missing_when_both_conditions_hold() {
        // rationale: Determinism — a set that has BOTH a duplicate AND a
        // missing kind must surface the DuplicateVerifier error first
        // (duplicate scan runs before the missing scan).
        let v: Vec<Box<dyn Verifier>> = vec![
            Box::new(ApproveAll {
                kind: VerifierKind::Security,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Security,
            }), // duplicate Security
            Box::new(ApproveAll {
                kind: VerifierKind::Consistency,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Cost,
            }),
            // Ember missing entirely.
        ];
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample());
        assert!(matches!(
            r,
            Err(VerifierError::DuplicateVerifier(VerifierKind::Security))
        ));
    }

    #[test]
    fn each_kind_duplicated_individually_yields_correct_error() {
        // rationale: Adversarial input — duplicating any of the four kinds
        // must surface that specific kind in DuplicateVerifier.
        for &dup in &VerifierKind::VARIANTS {
            let mut v: Vec<Box<dyn Verifier>> = VerifierKind::VARIANTS
                .iter()
                .map(|&k| Box::new(ApproveAll { kind: k }) as Box<dyn Verifier>)
                .collect();
            v.push(Box::new(ApproveAll { kind: dup }));
            let refs: Vec<&dyn Verifier> =
                v.iter().map(std::convert::AsRef::as_ref).collect();
            let r = aggregate(&refs, &sample());
            assert!(
                matches!(r, Err(VerifierError::DuplicateVerifier(k)) if k == dup),
                "duplicate of {dup:?} not detected"
            );
        }
    }

    #[test]
    fn triple_duplicate_of_one_kind_still_detected() {
        // rationale: Boundary — three copies of the same kind (count > 2)
        // must still raise DuplicateVerifier, not slip through.
        let v: Vec<Box<dyn Verifier>> = vec![
            Box::new(ApproveAll {
                kind: VerifierKind::Cost,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Cost,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Cost,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Security,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Consistency,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Ember,
            }),
        ];
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample());
        assert!(matches!(
            r,
            Err(VerifierError::DuplicateVerifier(VerifierKind::Cost))
        ));
    }

    #[test]
    fn single_verifier_only_yields_missing_not_duplicate() {
        // rationale: Boundary — exactly one verifier supplied. The duplicate
        // scan must NOT false-positive (count == 1 is fine); the missing
        // scan fires for the first absent kind.
        let v: Vec<Box<dyn Verifier>> = vec![Box::new(ApproveAll {
            kind: VerifierKind::Cost,
        })];
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample());
        assert!(matches!(
            r,
            Err(VerifierError::MissingVerifier(VerifierKind::Security))
        ));
    }

    #[test]
    fn missing_scan_reports_first_missing_kind_in_ordinal_order() {
        // rationale: Determinism — when several kinds are missing, the error
        // names the lowest-ordinal missing kind (Security < Consistency).
        let v: Vec<Box<dyn Verifier>> = vec![Box::new(ApproveAll {
            kind: VerifierKind::Ember,
        })];
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample());
        assert!(matches!(
            r,
            Err(VerifierError::MissingVerifier(VerifierKind::Security))
        ));
    }

    #[test]
    fn blocked_log_records_all_four_even_with_mixed_verdicts() {
        // rationale: Contract regression — the human-comparable refusal log
        // always carries all four outcomes (approve + refuse + amend mixed).
        let v: [Box<dyn Verifier>; 4] = [
            Box::new(ApproveAll {
                kind: VerifierKind::Security,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Consistency,
            }),
            Box::new(AmendOne {
                kind: VerifierKind::Cost,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Ember,
            }),
        ];
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample()).expect("ok");
        let AggregateVerdict::Blocked { per_verifier } = r else {
            panic!("expected Blocked");
        };
        assert_eq!(per_verifier.len(), 4);
        let blocking = per_verifier.iter().filter(|(_, v)| v.is_blocking()).count();
        assert_eq!(blocking, 2, "exactly Consistency + Cost block");
    }

    #[test]
    fn blocked_log_preserves_each_verifier_verdict_payload() {
        // rationale: Contract regression — the refuse reason / amend request
        // strings must survive into the per_verifier log verbatim.
        let v: [Box<dyn Verifier>; 4] = [
            Box::new(ApproveAll {
                kind: VerifierKind::Security,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Consistency,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Cost,
            }),
            Box::new(AmendOne {
                kind: VerifierKind::Ember,
            }),
        ];
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample()).expect("ok");
        let AggregateVerdict::Blocked { per_verifier } = r else {
            panic!("expected Blocked");
        };
        let cost = per_verifier
            .iter()
            .find(|(k, _)| *k == VerifierKind::Cost)
            .expect("cost present");
        assert!(matches!(&cost.1, VerifierVerdict::Refuse { reason } if reason == "test"));
        let ember = per_verifier
            .iter()
            .find(|(k, _)| *k == VerifierKind::Ember)
            .expect("ember present");
        assert!(matches!(
            &ember.1,
            VerifierVerdict::Amend { request } if request == "please specify"
        ));
    }

    #[test]
    fn all_approve_log_is_not_blocked_variant() {
        // rationale: Anti-property — an all-approve set must NEVER produce a
        // Blocked verdict (even an empty-per_verifier one).
        let v = approve_quad();
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample()).expect("ok");
        assert!(!matches!(r, AggregateVerdict::Blocked { .. }));
    }

    #[test]
    fn verdict_ordering_independent_of_which_kind_refuses() {
        // rationale: Determinism — regardless of which kind refuses, the
        // per_verifier log is ordinal-ordered Security..Ember.
        for &refuser in &VerifierKind::VARIANTS {
            let v = quad_with(refuser);
            let refs: Vec<&dyn Verifier> =
                v.iter().map(std::convert::AsRef::as_ref).collect();
            let r = aggregate(&refs, &sample()).expect("ok");
            let AggregateVerdict::Blocked { per_verifier } = r else {
                panic!("expected Blocked for refuser {refuser:?}");
            };
            let kinds: Vec<VerifierKind> =
                per_verifier.iter().map(|(k, _)| *k).collect();
            assert_eq!(
                kinds,
                vec![
                    VerifierKind::Security,
                    VerifierKind::Consistency,
                    VerifierKind::Cost,
                    VerifierKind::Ember,
                ],
                "ordering drift when {refuser:?} refuses"
            );
        }
    }

    #[test]
    fn verifier_kind_ordinal_matches_variants_index() {
        // rationale: Contract regression — ordinal must equal the position
        // of the kind in VARIANTS (0-based), the documented projection.
        for (i, &k) in VerifierKind::VARIANTS.iter().enumerate() {
            assert_eq!(
                u8::try_from(i).expect("idx fits"),
                k.ordinal(),
                "ordinal/VARIANTS index mismatch for {k:?}"
            );
        }
    }

    #[test]
    fn verifier_kind_as_str_all_distinct() {
        // rationale: Anti-property — each kind's log label must be unique;
        // a collision would make metric labels ambiguous.
        let set: std::collections::HashSet<&str> = VerifierKind::VARIANTS
            .iter()
            .map(|k| k.as_str())
            .collect();
        assert_eq!(set.len(), 4);
    }

    #[test]
    fn verifier_kind_as_str_full_mapping_pinned() {
        // rationale: Contract regression — the exact label strings are
        // operator-facing and must not drift.
        assert_eq!(VerifierKind::Security.as_str(), "security");
        assert_eq!(VerifierKind::Consistency.as_str(), "consistency");
        assert_eq!(VerifierKind::Cost.as_str(), "cost");
        assert_eq!(VerifierKind::Ember.as_str(), "ember");
    }

    #[test]
    fn verifier_kind_ord_matches_ordinal_ordering() {
        // rationale: Determinism — the derived Ord on VerifierKind must agree
        // with the explicit ordinal() projection (used in the sort).
        let mut by_ord = VerifierKind::VARIANTS;
        by_ord.sort_unstable();
        assert_eq!(by_ord, VerifierKind::VARIANTS);
        for w in VerifierKind::VARIANTS.windows(2) {
            assert!(w[0] < w[1], "Ord disagrees with ordinal at {w:?}");
        }
    }

    #[test]
    fn verifier_kind_serde_uses_snake_case() {
        // rationale: Contract regression — the wire form is snake_case; a
        // future rename of the rust variant must not silently change the
        // serialized token.
        assert_eq!(
            serde_json::to_string(&VerifierKind::Consistency).expect("ser"),
            "\"consistency\""
        );
    }

    #[test]
    fn verifier_verdict_approve_serializes_as_bare_string() {
        // rationale: Contract regression — Approve is a unit variant; with
        // snake_case rename it must serialize as the bare "approve" string.
        let j = serde_json::to_string(&VerifierVerdict::Approve).expect("ser");
        assert_eq!(j, "\"approve\"");
    }

    #[test]
    fn verifier_verdict_refuse_carries_reason_in_json() {
        // rationale: Contract regression — Refuse must serialize the reason
        // field; an operator reading the wire log needs the cause.
        let v = VerifierVerdict::Refuse {
            reason: "escape-surface AP30".to_owned(),
        };
        let j = serde_json::to_string(&v).expect("ser");
        assert!(j.contains("refuse"));
        assert!(j.contains("escape-surface AP30"));
        let back: VerifierVerdict = serde_json::from_str(&j).expect("de");
        assert_eq!(back, v);
    }

    #[test]
    fn verifier_verdict_amend_carries_request_in_json() {
        // rationale: Contract regression — Amend must serialize the request.
        let v = VerifierVerdict::Amend {
            request: "narrow the namespace".to_owned(),
        };
        let j = serde_json::to_string(&v).expect("ser");
        assert!(j.contains("amend"));
        assert!(j.contains("narrow the namespace"));
        let back: VerifierVerdict = serde_json::from_str(&j).expect("de");
        assert_eq!(back, v);
    }

    #[test]
    fn is_blocking_is_pure_approve_complement() {
        // rationale: Anti-property — is_blocking() must be exactly !Approve.
        // Empty-string reason / request must still count as blocking.
        assert!(!VerifierVerdict::Approve.is_blocking());
        assert!(VerifierVerdict::Refuse {
            reason: String::new()
        }
        .is_blocking());
        assert!(VerifierVerdict::Amend {
            request: String::new()
        }
        .is_blocking());
    }

    #[test]
    fn duplicate_verifier_error_displays_kind() {
        // rationale: Contract regression — the DuplicateVerifier Display
        // string must name the offending kind for operator triage.
        let e = VerifierError::DuplicateVerifier(VerifierKind::Cost);
        let s = format!("{e}");
        assert!(s.contains("duplicate"));
        assert!(s.contains("Cost"));
    }

    #[test]
    fn missing_and_duplicate_errors_are_not_equal() {
        // rationale: Anti-property — the two error variants must be
        // distinguishable even for the same kind.
        assert_ne!(
            VerifierError::MissingVerifier(VerifierKind::Ember),
            VerifierError::DuplicateVerifier(VerifierKind::Ember)
        );
    }

    #[test]
    fn verifier_error_distinct_kinds_not_equal() {
        // rationale: Anti-property — same variant, different kind ≠ equal.
        assert_ne!(
            VerifierError::MissingVerifier(VerifierKind::Security),
            VerifierError::MissingVerifier(VerifierKind::Cost)
        );
    }

    #[test]
    fn aggregate_verdict_all_approve_eq_self() {
        // rationale: Contract regression — AllApprove must be Eq-stable.
        assert_eq!(AggregateVerdict::AllApprove, AggregateVerdict::AllApprove);
    }

    #[test]
    fn aggregate_verdict_blocked_eq_requires_same_log() {
        // rationale: Anti-property — two Blocked verdicts with different
        // per_verifier logs must NOT compare equal.
        let a = AggregateVerdict::Blocked {
            per_verifier: vec![(
                VerifierKind::Security,
                VerifierVerdict::Refuse {
                    reason: "a".to_owned(),
                },
            )],
        };
        let b = AggregateVerdict::Blocked {
            per_verifier: vec![(
                VerifierKind::Security,
                VerifierVerdict::Refuse {
                    reason: "b".to_owned(),
                },
            )],
        };
        assert_ne!(a, b);
    }

    #[test]
    fn aggregate_verdict_all_approve_ne_blocked() {
        // rationale: Anti-property — the two aggregate variants must never
        // compare equal regardless of payload.
        let blocked = AggregateVerdict::Blocked {
            per_verifier: Vec::new(),
        };
        assert_ne!(AggregateVerdict::AllApprove, blocked);
    }

    #[test]
    fn verifier_trait_object_is_send_and_sync() {
        // rationale: Concurrency — the Verifier trait is declared Send+Sync
        // so verifier sets can cross thread boundaries (fleet dispatch).
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn Verifier>();
    }

    #[test]
    fn aggregate_callable_from_spawned_thread() {
        // rationale: Concurrency — aggregate over Send+Sync verifiers must be
        // usable inside a spawned thread without compile-time friction.
        let handle = std::thread::spawn(|| {
            let v = approve_quad();
            let refs: Vec<&dyn Verifier> =
                v.iter().map(std::convert::AsRef::as_ref).collect();
            aggregate(&refs, &sample()).expect("ok")
        });
        assert_eq!(
            handle.join().expect("join"),
            AggregateVerdict::AllApprove
        );
    }

    #[test]
    fn two_refusals_block_and_log_both_as_blocking() {
        // rationale: Adversarial input — two of four refuse; the log must
        // mark exactly two entries as blocking and two as non-blocking.
        let v: [Box<dyn Verifier>; 4] = [
            Box::new(RefuseOne {
                kind: VerifierKind::Security,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Consistency,
            }),
            Box::new(RefuseOne {
                kind: VerifierKind::Cost,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Ember,
            }),
        ];
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample()).expect("ok");
        let AggregateVerdict::Blocked { per_verifier } = r else {
            panic!("expected Blocked");
        };
        let blocking: Vec<VerifierKind> = per_verifier
            .iter()
            .filter(|(_, v)| v.is_blocking())
            .map(|(k, _)| *k)
            .collect();
        assert_eq!(blocking, vec![VerifierKind::Security, VerifierKind::Cost]);
    }

    #[test]
    fn aggregate_result_stable_across_repeated_calls() {
        // rationale: Determinism — calling aggregate twice on the same
        // verifier set + workflow yields identical verdicts (pure function).
        let v = quad_with(VerifierKind::Ember);
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        let wf = sample();
        let first = aggregate(&refs, &wf).expect("ok");
        let second = aggregate(&refs, &wf).expect("ok");
        assert_eq!(first, second);
    }

    #[test]
    fn five_verifiers_with_no_duplicate_still_errors_is_impossible() {
        // rationale: Boundary — there are exactly 4 kinds, so any 5-element
        // set MUST contain a duplicate; aggregate must surface it (never an
        // AllApprove). Pigeonhole invariant.
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
            Box::new(ApproveAll {
                kind: VerifierKind::Ember,
            }),
            Box::new(ApproveAll {
                kind: VerifierKind::Ember,
            }),
        ];
        let refs: Vec<&dyn Verifier> =
            v.iter().map(std::convert::AsRef::as_ref).collect();
        let r = aggregate(&refs, &sample());
        assert!(
            matches!(r, Err(VerifierError::DuplicateVerifier(_))),
            "5-element set must always raise DuplicateVerifier"
        );
    }
}
