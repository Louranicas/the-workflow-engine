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
    let mut out = Vec::new();
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
    // Skip each element.
    for i in 0..pattern.steps.len() {
        if out.len() >= MAX_VARIANTS_PER_PATTERN {
            break;
        }
        if pattern.steps.len() <= 1 {
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
}
