//! `m23_workflow_proposer` — compose `WorkflowVariant`s with feature
//! context into structured `WorkflowProposal`s for human review.
//!
//! Cluster F · L6 · KEYSTONE downstream of m21 + m22. m23 NEVER
//! auto-promotes; per AP-V7-07 + v1.3 § 2 every proposal exits the
//! engine as a markdown artefact for operator review.

use thiserror::Error;

use crate::m14_lift::{LiftSnapshot, MIN_SAMPLE_SIZE};
use crate::m20_prefixspan::Pattern;
use crate::m21_variant_builder::WorkflowVariant;

/// **F2 hard gate:** proposals refuse to build when the m14 lift
/// snapshot reports `n < MIN_SAMPLE_SIZE` or `lift.is_none()`.
pub const PROPOSAL_F2_THRESHOLD: usize = MIN_SAMPLE_SIZE;

/// A structured proposal for operator review.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowProposal {
    /// Opaque identifier.
    pub proposal_id: u64,
    /// Source variant.
    pub variant: WorkflowVariant,
    /// Aggregate evidence at proposal time.
    pub evidence_n: usize,
    /// m14 composite lift.
    pub evidence_lift: f64,
    /// Wilson CI half-width.
    pub evidence_ci_half: f64,
    /// Cluster index assigned by m22 (`None` if feature clustering
    /// skipped).
    pub diversity_cluster: Option<usize>,
}

/// Proposal-builder errors.
#[derive(Debug, Error)]
pub enum ProposerError {
    /// m14 lift snapshot indicates insufficient evidence.
    #[error("F2 gate: evidence n={n} < {threshold}")]
    EvidenceBelowThreshold {
        /// Observed n.
        n: usize,
        /// Required threshold.
        threshold: usize,
    },
    /// m14 lift snapshot returned `None` for either `lift` or `ci_half`.
    #[error("F2 gate: lift / ci_half is None")]
    LiftUnavailable,
}

/// Build a proposal from a variant + evidence snapshot.
///
/// **F2 hard refusal:** returns
/// [`ProposerError::EvidenceBelowThreshold`] if `snapshot.n <
/// PROPOSAL_F2_THRESHOLD`, and [`ProposerError::LiftUnavailable`] if
/// either `snapshot.lift` or `snapshot.ci_half` is `None`.
///
/// # Errors
///
/// See above.
pub fn build_proposal(
    variant: WorkflowVariant,
    snapshot: &LiftSnapshot,
    diversity_cluster: Option<usize>,
) -> Result<WorkflowProposal, ProposerError> {
    if snapshot.n < PROPOSAL_F2_THRESHOLD {
        return Err(ProposerError::EvidenceBelowThreshold {
            n: snapshot.n,
            threshold: PROPOSAL_F2_THRESHOLD,
        });
    }
    let lift = snapshot.lift.ok_or(ProposerError::LiftUnavailable)?;
    let ci_half = snapshot
        .ci_half
        .ok_or(ProposerError::LiftUnavailable)?;
    let proposal_id = crate::m4_cascade::cluster_id::fnv1a_64(
        format!("proposal:{}:{}", variant.variant_id, snapshot.n).as_bytes(),
    );
    Ok(WorkflowProposal {
        proposal_id,
        variant,
        evidence_n: snapshot.n,
        evidence_lift: lift,
        evidence_ci_half: ci_half,
        diversity_cluster,
    })
}

/// Compose variants from a top-N pattern slice into a proposal batch.
///
/// Skips patterns/variants whose evidence fails the F2 gate. The
/// returned vec preserves source ordering.
#[must_use]
pub fn compose_proposals(
    patterns: &[Pattern],
    snapshot: &LiftSnapshot,
) -> Vec<WorkflowProposal> {
    let mut out = Vec::new();
    for p in patterns {
        let Ok(variants) = crate::m21_variant_builder::build_variants(p) else {
            continue;
        };
        for v in variants {
            if let Ok(proposal) = build_proposal(v, snapshot, None) {
                out.push(proposal);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::{
        build_proposal, compose_proposals, ProposerError, PROPOSAL_F2_THRESHOLD,
    };
    use crate::m14_lift::LiftSnapshot;
    use crate::m20_prefixspan::{Pattern, StepToken};
    use crate::m21_variant_builder::{build_variants, MutationKind, WorkflowVariant};

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
        Pattern::new(vec![StepToken(1), StepToken(2)], 10, (0, 1))
    }

    fn sample_variant() -> WorkflowVariant {
        build_variants(&sample_pattern()).expect("v")[0].clone()
    }

    #[test]
    fn proposal_refuses_below_n_threshold() {
        let s = snap(10, Some(0.5), Some(0.1));
        assert!(matches!(
            build_proposal(sample_variant(), &s, None),
            Err(ProposerError::EvidenceBelowThreshold { n: 10, .. })
        ));
    }

    #[test]
    fn proposal_refuses_when_lift_none() {
        let s = snap(30, None, Some(0.1));
        assert!(matches!(
            build_proposal(sample_variant(), &s, None),
            Err(ProposerError::LiftUnavailable)
        ));
    }

    #[test]
    fn proposal_refuses_when_ci_none() {
        let s = snap(30, Some(0.5), None);
        assert!(matches!(
            build_proposal(sample_variant(), &s, None),
            Err(ProposerError::LiftUnavailable)
        ));
    }

    #[test]
    fn proposal_built_when_evidence_sufficient() {
        let s = snap(30, Some(0.6), Some(0.05));
        let p = build_proposal(sample_variant(), &s, Some(0)).expect("ok");
        assert_eq!(p.evidence_n, 30);
        assert!((p.evidence_lift - 0.6).abs() < 1e-12);
        assert_eq!(p.diversity_cluster, Some(0));
    }

    #[test]
    fn proposal_id_deterministic() {
        let s = snap(30, Some(0.6), Some(0.05));
        let p1 = build_proposal(sample_variant(), &s, None).expect("p1");
        let p2 = build_proposal(sample_variant(), &s, None).expect("p2");
        assert_eq!(p1.proposal_id, p2.proposal_id);
    }

    #[test]
    fn compose_proposals_skips_insufficient_evidence() {
        let s = snap(10, Some(0.5), Some(0.05));
        let patterns = vec![sample_pattern()];
        let p = compose_proposals(&patterns, &s);
        assert!(p.is_empty());
    }

    #[test]
    fn compose_proposals_yields_variants_under_sufficient_evidence() {
        let s = snap(30, Some(0.5), Some(0.05));
        let patterns = vec![sample_pattern()];
        let proposals = compose_proposals(&patterns, &s);
        assert!(!proposals.is_empty());
        // First proposal should be derived from the identity variant.
        assert!(matches!(
            proposals[0].variant.mutation,
            MutationKind::Identity
        ));
    }

    #[test]
    fn f2_threshold_equals_min_sample_size() {
        assert_eq!(PROPOSAL_F2_THRESHOLD, crate::m14_lift::MIN_SAMPLE_SIZE);
    }
}
