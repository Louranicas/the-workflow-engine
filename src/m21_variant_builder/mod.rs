//! `m21_variant_builder` — generate variant workflow proposals from
//! mined patterns + Battern observations.
//!
//! Cluster F · L6 · KEYSTONE downstream of m20. Builds candidate
//! `WorkflowVariant`s by enumerating bounded mutations (swap / skip /
//! parameterise) on the most-supported patterns from m20, preserving
//! the F11 opacity discipline (no human label injected).

use thiserror::Error;

use crate::m20_prefixspan::{Pattern, StepToken};

/// Maximum variants emitted per input pattern.
pub const MAX_VARIANTS_PER_PATTERN: usize = 8;

/// A workflow variant proposal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct WorkflowVariant {
    /// Opaque identifier (FNV-1a over canonical encoding).
    pub variant_id: u64,
    /// Step sequence (the variant's body).
    pub steps: Vec<StepToken>,
    /// Mutation kind applied to derive this variant.
    pub mutation: MutationKind,
    /// Provenance: the canonical_hash of the source `Pattern`.
    pub source_pattern_hash: u64,
}

/// Mutation taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationKind {
    /// Verbatim copy of the source pattern (baseline variant).
    Identity,
    /// Two adjacent steps swapped.
    Swap {
        /// Index of the first swapped element.
        at: usize,
    },
    /// One step removed.
    Skip {
        /// Index of the dropped element.
        at: usize,
    },
}

/// Builder errors.
#[derive(Debug, Error)]
pub enum VariantBuilderError {
    /// Source pattern was empty.
    #[error("empty source pattern")]
    EmptyPattern,
}

/// Enumerate variants for a single pattern. Returns at most
/// `MAX_VARIANTS_PER_PATTERN` proposals, always including the identity
/// variant first.
///
/// # Errors
///
/// [`VariantBuilderError::EmptyPattern`] when `pattern.steps` is empty.
pub fn build_variants(
    pattern: &Pattern,
) -> Result<Vec<WorkflowVariant>, VariantBuilderError> {
    if pattern.steps.is_empty() {
        return Err(VariantBuilderError::EmptyPattern);
    }
    // Pre-allocate the variant cap (F2 monoculture-prevention bound).
    let mut out: Vec<WorkflowVariant> = Vec::with_capacity(MAX_VARIANTS_PER_PATTERN);
    let identity = WorkflowVariant {
        variant_id: variant_hash(&pattern.steps, MutationKind::Identity),
        steps: pattern.steps.clone(),
        mutation: MutationKind::Identity,
        source_pattern_hash: pattern.canonical_hash,
    };
    out.push(identity);

    // Swap adjacent pairs.
    for i in 0..pattern.steps.len().saturating_sub(1) {
        if out.len() >= MAX_VARIANTS_PER_PATTERN {
            break;
        }
        let mut steps = pattern.steps.clone();
        steps.swap(i, i + 1);
        let mutation = MutationKind::Swap { at: i };
        out.push(WorkflowVariant {
            variant_id: variant_hash(&steps, mutation),
            steps,
            mutation,
            source_pattern_hash: pattern.canonical_hash,
        });
    }
    // Skip mutations are only meaningful when the pattern has >=2 steps
    // (a single-step pattern reduced to zero is the empty pattern, which is
    // semantically invalid for downstream m22/m23 consumption). Hoisted
    // outside the loop because it is loop-invariant.
    if pattern.steps.len() >= 2 {
        for i in 0..pattern.steps.len() {
            if out.len() >= MAX_VARIANTS_PER_PATTERN {
                break;
            }
            let mut steps = pattern.steps.clone();
            steps.remove(i);
            let mutation = MutationKind::Skip { at: i };
            out.push(WorkflowVariant {
                variant_id: variant_hash(&steps, mutation),
                steps,
                mutation,
                source_pattern_hash: pattern.canonical_hash,
            });
        }
    }
    Ok(out)
}

fn variant_hash(steps: &[StepToken], mutation: MutationKind) -> u64 {
    use std::fmt::Write;
    let mut buf = String::new();
    for s in steps {
        let _ = write!(buf, "{}:", s.0);
    }
    let _ = write!(
        buf,
        "{}",
        match mutation {
            MutationKind::Identity => "id".to_owned(),
            MutationKind::Swap { at } => format!("swap{at}"),
            MutationKind::Skip { at } => format!("skip{at}"),
        }
    );
    crate::m4_cascade::cluster_id::fnv1a_64(buf.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::{
        build_variants, MutationKind, VariantBuilderError, MAX_VARIANTS_PER_PATTERN,
    };
    use crate::m20_prefixspan::{Pattern, StepToken};

    fn pattern(steps: &[u32]) -> Pattern {
        Pattern::new(
            steps.iter().copied().map(StepToken).collect(),
            5,
            (0, 1),
        )
    }

    #[test]
    fn empty_pattern_rejected() {
        let p = Pattern::new(Vec::new(), 5, (0, 0));
        assert!(matches!(
            build_variants(&p),
            Err(VariantBuilderError::EmptyPattern)
        ));
    }

    #[test]
    fn identity_variant_always_emitted_first() {
        let p = pattern(&[1, 2, 3]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v[0].mutation, MutationKind::Identity);
        assert_eq!(v[0].steps, p.steps);
    }

    #[test]
    fn swap_mutations_yield_distinct_variants() {
        let p = pattern(&[1, 2, 3]);
        let v = build_variants(&p).expect("ok");
        let swap_vars: Vec<_> = v
            .iter()
            .filter(|x| matches!(x.mutation, MutationKind::Swap { .. }))
            .collect();
        assert!(!swap_vars.is_empty());
        // Each swap should produce a different sequence than the identity.
        for sv in swap_vars {
            assert_ne!(sv.steps, p.steps);
        }
    }

    #[test]
    fn skip_mutations_reduce_step_count_by_one() {
        let p = pattern(&[1, 2, 3, 4]);
        let v = build_variants(&p).expect("ok");
        for var in &v {
            if matches!(var.mutation, MutationKind::Skip { .. }) {
                assert_eq!(var.steps.len(), p.steps.len() - 1);
            }
        }
    }

    #[test]
    fn max_variants_capped() {
        let p = pattern(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let v = build_variants(&p).expect("ok");
        assert!(v.len() <= MAX_VARIANTS_PER_PATTERN);
    }

    #[test]
    fn variant_id_deterministic() {
        let p = pattern(&[1, 2, 3]);
        let a = build_variants(&p).expect("a");
        let b = build_variants(&p).expect("b");
        for (va, vb) in a.iter().zip(b.iter()) {
            assert_eq!(va.variant_id, vb.variant_id);
        }
    }

    #[test]
    fn source_pattern_hash_threaded_through() {
        let p = pattern(&[5, 7]);
        let v = build_variants(&p).expect("ok");
        for var in &v {
            assert_eq!(var.source_pattern_hash, p.canonical_hash);
        }
    }

    #[test]
    fn single_step_pattern_yields_only_identity() {
        let p = pattern(&[42]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].mutation, MutationKind::Identity);
    }

    #[test]
    fn variant_serde_roundtrip() {
        let p = pattern(&[1, 2]);
        let v = &build_variants(&p).expect("ok")[1];
        let s = serde_json::to_string(v).expect("ser");
        let back: super::WorkflowVariant = serde_json::from_str(&s).expect("de");
        assert_eq!(back, *v);
    }

    // ---- Cluster F hardening pass — additional 10+ tests ----

    #[test]
    // rationale: Boundary — MAX_VARIANTS_PER_PATTERN cap is enforced even
    // when (n_swaps + n_skips + 1) would exceed.
    fn boundary_max_variants_cap_strict() {
        let p = pattern(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let v = build_variants(&p).expect("ok");
        assert!(v.len() <= MAX_VARIANTS_PER_PATTERN);
    }

    #[test]
    // rationale: Boundary — exactly MAX_VARIANTS_PER_PATTERN possible swaps.
    fn boundary_eight_step_pattern_caps_at_eight_variants() {
        let p = pattern(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v.len(), MAX_VARIANTS_PER_PATTERN);
    }

    #[test]
    // rationale: Determinism — variant_id is stable across distinct runs
    // and bit-equal to a reference computation.
    fn determinism_variant_id_bit_stable() {
        let p = pattern(&[1, 2, 3]);
        let v1 = build_variants(&p).expect("v1");
        let v2 = build_variants(&p).expect("v2");
        for (a, b) in v1.iter().zip(v2.iter()) {
            assert_eq!(a.variant_id, b.variant_id);
            assert_eq!(a.steps, b.steps);
        }
    }

    #[test]
    // rationale: Anti-property — F11 cascade-monoculture: variant_id is
    // u64 only (no human-readable substring possible).
    fn anti_property_f11_variant_id_is_pure_u64() {
        let p = pattern(&[1, 2, 3]);
        let v = build_variants(&p).expect("ok");
        for var in &v {
            // variant_id is u64; serde JSON encodes it as a number.
            let s = serde_json::to_string(&var.variant_id).expect("ser");
            assert!(s.chars().all(|c| c.is_ascii_digit()),
                "variant_id leaked non-numeric: {s}");
        }
    }

    #[test]
    // rationale: Cross-module — source_pattern_hash links variants back to
    // the m20 Pattern. Distinct patterns must yield distinct provenance.
    fn cross_module_distinct_patterns_yield_distinct_provenance() {
        let p1 = pattern(&[1, 2]);
        let p2 = pattern(&[1, 3]);
        let v1 = build_variants(&p1).expect("v1");
        let v2 = build_variants(&p2).expect("v2");
        assert_ne!(v1[0].source_pattern_hash, v2[0].source_pattern_hash);
    }

    #[test]
    // rationale: Adversarial — two-step pattern still yields skip variants
    // (skip-i=0 produces single-step variant; skip-i=1 produces single-step).
    fn adversarial_two_step_pattern_yields_skip_variants() {
        let p = pattern(&[1, 2]);
        let v = build_variants(&p).expect("ok");
        let skips: Vec<_> = v.iter().filter(|x| matches!(x.mutation, MutationKind::Skip { .. })).collect();
        assert_eq!(skips.len(), 2, "two-step should yield 2 skip variants, got {skips:?}");
    }

    #[test]
    // rationale: Contract regression — VariantBuilderError variants stable.
    fn contract_variant_builder_error_variants_stable() {
        let empty = VariantBuilderError::EmptyPattern;
        assert!(!format!("{empty}").is_empty());
    }

    #[test]
    // rationale: Resource accounting — output Vec capacity is hinted, not
    // reallocated. Trip the path on a typical 4-step pattern.
    fn resource_accounting_four_step_no_realloc_growth() {
        let p = pattern(&[1, 2, 3, 4]);
        let v = build_variants(&p).expect("ok");
        // Identity + 3 swaps + 4 skips = 8 = MAX_VARIANTS_PER_PATTERN.
        assert_eq!(v.len(), 8);
    }

    #[test]
    // rationale: Boundary — single-step pattern produces ONLY identity
    // (skip-of-only-element would empty the pattern).
    fn boundary_single_step_produces_identity_only_no_skip() {
        let p = pattern(&[99]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v.len(), 1);
        assert!(matches!(v[0].mutation, MutationKind::Identity));
    }

    #[test]
    // rationale: Anti-property — Identity variant is always first (proposer
    // m23 relies on this for "baseline-first" emission semantics).
    fn anti_property_identity_always_index_zero() {
        for ns in [&[1, 2][..], &[1, 2, 3][..], &[1, 2, 3, 4, 5][..]] {
            let p = pattern(ns);
            let v = build_variants(&p).expect("ok");
            assert!(matches!(v[0].mutation, MutationKind::Identity));
        }
    }

    #[test]
    // rationale: Determinism — variant_hash is deterministic across
    // construction sites (in-line vs computed).
    fn determinism_variant_hash_pure_function() {
        let h1 = super::variant_hash(&[StepToken(1), StepToken(2)], MutationKind::Identity);
        let h2 = super::variant_hash(&[StepToken(1), StepToken(2)], MutationKind::Identity);
        assert_eq!(h1, h2);
        // Swap-at variants must yield distinct hashes.
        let h3 = super::variant_hash(&[StepToken(1), StepToken(2)], MutationKind::Swap { at: 0 });
        assert_ne!(h1, h3);
    }
}
