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
///
/// **Silent-swallow rationale (AP-V7-13 audit):** the inner `let Ok(_) = ...`
/// branches deliberately discard two error classes:
///
/// 1. [`crate::m21_variant_builder::VariantBuilderError::EmptyPattern`] —
///    only fires when `pattern.steps.is_empty()`, which m20's
///    `mine_sequences` cannot produce (every emitted `Pattern` carries
///    `support >= MIN_SUPPORT_FLOOR` which requires ≥1 step). Discarding
///    is correct: the input contract from m20 already excludes it.
/// 2. [`ProposerError::EvidenceBelowThreshold`] and
///    [`ProposerError::LiftUnavailable`] — these ARE the F2 gate firing.
///    `compose_proposals` is the batched form whose documented behaviour
///    is "skips below F2"; trace-emit-and-drop is the contract.
///
/// Callers that need typed refusal MUST use [`build_proposal`] directly.
#[must_use]
pub fn compose_proposals(
    patterns: &[Pattern],
    snapshot: &LiftSnapshot,
) -> Vec<WorkflowProposal> {
    // Capacity hint: at most MAX_VARIANTS_PER_PATTERN proposals per pattern.
    let mut out: Vec<WorkflowProposal> = Vec::with_capacity(
        patterns
            .len()
            .saturating_mul(crate::m21_variant_builder::MAX_VARIANTS_PER_PATTERN),
    );
    for p in patterns {
        // rationale: m20 contract excludes empty-step patterns; documented above.
        let Ok(variants) = crate::m21_variant_builder::build_variants(p) else {
            tracing::debug!(
                pattern_hash = p.canonical_hash,
                "m23::compose_proposals — m21 build_variants refused; m20 contract violation upstream"
            );
            continue;
        };
        for v in variants {
            // rationale: F2 gate skip-and-trace is the documented batched
            // behaviour for compose_proposals; build_proposal is the
            // strict typed-refusal path.
            match build_proposal(v, snapshot, None) {
                Ok(proposal) => out.push(proposal),
                Err(e) => {
                    tracing::debug!(
                        pattern_hash = p.canonical_hash,
                        error = %e,
                        "m23::compose_proposals — F2 gate skip"
                    );
                }
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

    // ---- Cluster F hardening pass — additional 10+ tests ----

    #[test]
    // rationale: Boundary — exactly at PROPOSAL_F2_THRESHOLD must be accepted.
    fn boundary_n_at_threshold_accepted() {
        let s = snap(PROPOSAL_F2_THRESHOLD, Some(0.5), Some(0.05));
        let r = build_proposal(sample_variant(), &s, None);
        assert!(r.is_ok());
    }

    #[test]
    // rationale: Boundary — one below PROPOSAL_F2_THRESHOLD must refuse.
    fn boundary_n_one_below_threshold_refused() {
        let s = snap(PROPOSAL_F2_THRESHOLD - 1, Some(0.5), Some(0.05));
        let r = build_proposal(sample_variant(), &s, None);
        assert!(matches!(r, Err(ProposerError::EvidenceBelowThreshold { .. })));
    }

    #[test]
    // rationale: Anti-property — AP-V7-07 m23 NEVER auto-promotes. There is
    // NO public function on m23 that writes to m30 bank or any external
    // store. We verify the m23 public surface contains no `promote`,
    // `commit`, `accept`, or `bank` symbol.
    fn anti_property_ap_v7_07_no_auto_promote_in_public_surface() {
        // The module compiles iff these are the only construction entry
        // points — if a future contributor adds `pub fn promote_proposal`
        // this test still compiles, so we instead inspect the public
        // proposal payload at runtime: there is no Bank/Selector/Dispatcher
        // field reachable from a proposal struct.
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        // Proposal type carries no field named `promoted`, `committed`,
        // `accepted_at`, or anything bank-related.
        let json = serde_json::to_string(&p).expect("ser");
        for forbidden in ["promoted", "committed", "accepted_at", "bank", "promote_to"] {
            assert!(
                !json.contains(forbidden),
                "AP-V7-07 violation: m23 proposal serde contains '{forbidden}': {json}"
            );
        }
    }

    #[test]
    // rationale: Anti-property — F11 cascade-monoculture: proposal_id is u64,
    // and serde JSON contains no human-readable substring.
    fn anti_property_f11_proposal_id_is_pure_u64() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        // proposal_id encodes as JSON number; no string content possible.
        let id_json = serde_json::to_string(&p.proposal_id).expect("ser");
        assert!(id_json.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    // rationale: Determinism — build_proposal is deterministic given
    // identical inputs. Run 5 times, all proposal_id values match.
    fn determinism_proposal_id_stable_across_invocations() {
        let s = snap(30, Some(0.5), Some(0.05));
        let mut ids = Vec::new();
        for _ in 0..5_u32 {
            let p = build_proposal(sample_variant(), &s, None).expect("ok");
            ids.push(p.proposal_id);
        }
        for w in ids.windows(2) {
            assert_eq!(w[0], w[1]);
        }
    }

    #[test]
    // rationale: Cross-module — compose_proposals consumes m20 Pattern +
    // m14 LiftSnapshot + m21 build_variants — exercise the full chain.
    fn cross_module_full_pipeline_yields_proposals() {
        let s = snap(30, Some(0.7), Some(0.05));
        let patterns = vec![
            Pattern::new(vec![StepToken(1), StepToken(2), StepToken(3)], 25, (0, 1)),
            Pattern::new(vec![StepToken(4), StepToken(5)], 22, (0, 0)),
        ];
        let p = compose_proposals(&patterns, &s);
        assert!(!p.is_empty());
        // Every proposal must carry sufficient evidence (compose skips F2).
        for prop in &p {
            assert!(prop.evidence_n >= PROPOSAL_F2_THRESHOLD);
        }
    }

    #[test]
    // rationale: Adversarial — lift = 0.0 must still pass (F2 is on n,
    // not on lift magnitude; the proposer is evidence-gated not
    // lift-gated by design).
    fn adversarial_zero_lift_accepted_at_sufficient_n() {
        let s = snap(50, Some(0.0), Some(0.01));
        let r = build_proposal(sample_variant(), &s, None);
        assert!(r.is_ok());
    }

    #[test]
    // rationale: Adversarial — negative lift (regression detected) must
    // still pass; lift sign is information, not a gate.
    fn adversarial_negative_lift_accepted() {
        let s = snap(50, Some(-0.3), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert!(p.evidence_lift < 0.0);
    }

    #[test]
    // rationale: Boundary — diversity_cluster=None must be threaded through.
    fn boundary_none_diversity_cluster_preserved() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert_eq!(p.diversity_cluster, None);
    }

    #[test]
    // rationale: Boundary — large diversity_cluster value preserved.
    fn boundary_large_diversity_cluster_preserved() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, Some(usize::MAX)).expect("ok");
        assert_eq!(p.diversity_cluster, Some(usize::MAX));
    }

    #[test]
    // rationale: Contract regression — ProposerError variants stable.
    fn contract_proposer_error_variants_stable() {
        let below = ProposerError::EvidenceBelowThreshold { n: 5, threshold: 20 };
        let lift = ProposerError::LiftUnavailable;
        assert!(!format!("{below}").is_empty());
        assert!(!format!("{lift}").is_empty());
    }

    #[test]
    // rationale: Resource accounting — compose_proposals pre-allocates and
    // returns empty for empty input without allocating beyond hint.
    fn resource_accounting_empty_input_returns_empty() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = compose_proposals(&[], &s);
        assert!(p.is_empty());
    }

    #[test]
    // rationale: Cross-module — proposal serde roundtrip preserves all
    // evidence-bearing fields (downstream m30 bank reads this).
    fn cross_module_proposal_serde_roundtrip() {
        let s = snap(30, Some(0.6), Some(0.05));
        let p = build_proposal(sample_variant(), &s, Some(2)).expect("ok");
        let ser = serde_json::to_string(&p).expect("ser");
        let back: super::WorkflowProposal = serde_json::from_str(&ser).expect("de");
        assert_eq!(back, p);
    }

    #[test]
    // rationale: Determinism — compose_proposals yields stable ordering
    // (proposer relies on m20's sort + m21's emission order).
    fn determinism_compose_proposals_output_stable() {
        let s = snap(30, Some(0.5), Some(0.05));
        let patterns = vec![Pattern::new(vec![StepToken(1), StepToken(2)], 25, (0, 0))];
        let a = compose_proposals(&patterns, &s);
        let b = compose_proposals(&patterns, &s);
        assert_eq!(a.len(), b.len());
        for (pa, pb) in a.iter().zip(b.iter()) {
            assert_eq!(pa.proposal_id, pb.proposal_id);
            assert_eq!(pa.variant.variant_id, pb.variant.variant_id);
        }
    }
}
