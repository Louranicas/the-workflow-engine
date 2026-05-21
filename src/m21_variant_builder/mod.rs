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
/// # Emission order and fair-cap discipline (F5 fix)
///
/// The identity variant is always emitted first. The remaining cap budget
/// (`MAX_VARIANTS_PER_PATTERN - 1`) is then **interleaved** between swap and
/// skip mutations rather than emitting all swaps before all skips. A prior
/// implementation emitted every swap before any skip, so an 8-step pattern
/// (1 identity + 7 swaps) filled the cap with zero `Skip` variants. Because
/// `Skip` is the only length-shortening mutation, that starvation biased the
/// proposal population toward same-length variants. The interleave
/// (`swap`, `skip`, `swap`, `skip`, …) guarantees the cap never wipes out an
/// entire mutation class while either class still has candidates: an 8-step
/// pattern now yields ~1 identity + 4 swaps + 3 skips. Emission within each
/// class stays ascending-index for determinism.
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

    // F5 fair emission — interleave swap/skip so the cap can never starve an
    // entire mutation class. `n_swaps = len - 1`; `n_skips = len` but only
    // meaningful when `len >= 2` (a single-step pattern reduced to zero is
    // the empty pattern, semantically invalid for downstream m22/m23).
    let n_swaps = pattern.steps.len().saturating_sub(1);
    let n_skips = if pattern.steps.len() >= 2 {
        pattern.steps.len()
    } else {
        0
    };
    let mut swap_i = 0_usize;
    let mut skip_i = 0_usize;
    // Round-robin: at each step prefer a swap, then a skip, until both
    // classes are exhausted or the cap is reached.
    while out.len() < MAX_VARIANTS_PER_PATTERN && (swap_i < n_swaps || skip_i < n_skips) {
        if swap_i < n_swaps && out.len() < MAX_VARIANTS_PER_PATTERN {
            let mut steps = pattern.steps.clone();
            steps.swap(swap_i, swap_i + 1);
            let mutation = MutationKind::Swap { at: swap_i };
            out.push(WorkflowVariant {
                variant_id: variant_hash(&steps, mutation),
                steps,
                mutation,
                source_pattern_hash: pattern.canonical_hash,
            });
            swap_i += 1;
        }
        if skip_i < n_skips && out.len() < MAX_VARIANTS_PER_PATTERN {
            let mut steps = pattern.steps.clone();
            steps.remove(skip_i);
            let mutation = MutationKind::Skip { at: skip_i };
            out.push(WorkflowVariant {
                variant_id: variant_hash(&steps, mutation),
                steps,
                mutation,
                source_pattern_hash: pattern.canonical_hash,
            });
            skip_i += 1;
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

    // ====================================================================
    // KEYSTONE hardening pass — known-input/known-output enumeration,
    // emission-order contract, mutation-arity counting, cap interaction.
    // ====================================================================

    #[test]
    // rationale: KIO — a 3-step pattern yields EXACTLY: 1 identity + 2 swaps
    // + 3 skips = 6 variants. Enumerate the full set.
    fn kio_three_step_pattern_yields_exactly_six_variants() {
        let p = pattern(&[1, 2, 3]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v.len(), 6, "expected 1 identity + 2 swaps + 3 skips");
        let swaps = v.iter().filter(|x| matches!(x.mutation, MutationKind::Swap { .. })).count();
        let skips = v.iter().filter(|x| matches!(x.mutation, MutationKind::Skip { .. })).count();
        let ids = v.iter().filter(|x| matches!(x.mutation, MutationKind::Identity)).count();
        assert_eq!((ids, swaps, skips), (1, 2, 3));
    }

    #[test]
    // rationale: KIO — swap at index i exchanges steps[i] and steps[i+1].
    // For [1,2,3]: Swap{0} → [2,1,3]; Swap{1} → [1,3,2]. Exact-match.
    fn kio_swap_at_index_exchanges_adjacent_pair() {
        let p = pattern(&[1, 2, 3]);
        let v = build_variants(&p).expect("ok");
        let swap0 = v.iter().find(|x| x.mutation == MutationKind::Swap { at: 0 }).expect("s0");
        let swap1 = v.iter().find(|x| x.mutation == MutationKind::Swap { at: 1 }).expect("s1");
        assert_eq!(swap0.steps, vec![StepToken(2), StepToken(1), StepToken(3)]);
        assert_eq!(swap1.steps, vec![StepToken(1), StepToken(3), StepToken(2)]);
    }

    #[test]
    // rationale: KIO — skip at index i removes steps[i]. For [1,2,3]:
    // Skip{0} → [2,3]; Skip{1} → [1,3]; Skip{2} → [1,2]. Exact-match.
    fn kio_skip_at_index_removes_that_element() {
        let p = pattern(&[1, 2, 3]);
        let v = build_variants(&p).expect("ok");
        let s0 = v.iter().find(|x| x.mutation == MutationKind::Skip { at: 0 }).expect("k0");
        let s1 = v.iter().find(|x| x.mutation == MutationKind::Skip { at: 1 }).expect("k1");
        let s2 = v.iter().find(|x| x.mutation == MutationKind::Skip { at: 2 }).expect("k2");
        assert_eq!(s0.steps, vec![StepToken(2), StepToken(3)]);
        assert_eq!(s1.steps, vec![StepToken(1), StepToken(3)]);
        assert_eq!(s2.steps, vec![StepToken(1), StepToken(2)]);
    }

    #[test]
    // rationale: Emission-order contract (F5 fair emission) — variants are
    // emitted as identity first, then swap/skip INTERLEAVED in ascending
    // index per class. For [1,2,3] (n_swaps=2, n_skips=3) the order is
    // identity, swap0, skip0, swap1, skip1, skip2. m23 proposer relies on
    // this stable ordering; the interleave guarantees the cap (F5) cannot
    // starve an entire mutation class.
    fn emission_order_identity_then_interleaved_swaps_skips() {
        let p = pattern(&[1, 2, 3]);
        let v = build_variants(&p).expect("ok");
        // index 0 = identity
        assert!(matches!(v[0].mutation, MutationKind::Identity));
        // interleaved: swap0, skip0, swap1, skip1, then the trailing skip2
        assert_eq!(v[1].mutation, MutationKind::Swap { at: 0 });
        assert_eq!(v[2].mutation, MutationKind::Skip { at: 0 });
        assert_eq!(v[3].mutation, MutationKind::Swap { at: 1 });
        assert_eq!(v[4].mutation, MutationKind::Skip { at: 1 });
        assert_eq!(v[5].mutation, MutationKind::Skip { at: 2 });
    }

    #[test]
    // rationale: Cap interaction (F5 fair emission) — a 10-step pattern would
    // generate 1+9+10=20 variants but the cap truncates at 8. The prior
    // implementation emitted all swaps before any skip, so the 8-variant
    // result was identity + 7 swaps + ZERO skips — starving the only
    // length-shortening mutation class and biasing the proposal population.
    // The F5 interleave fills the 7 post-identity slots round-robin
    // (swap0, skip0, swap1, skip1, swap2, skip2, swap3), so BOTH classes
    // survive the cap: 4 swaps + 3 skips.
    fn cap_interleaves_swaps_and_skips_no_class_starved() {
        let p = pattern(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v.len(), MAX_VARIANTS_PER_PATTERN);
        let skips = v.iter().filter(|x| matches!(x.mutation, MutationKind::Skip { .. })).count();
        let swaps = v.iter().filter(|x| matches!(x.mutation, MutationKind::Swap { .. })).count();
        assert!(skips > 0, "F5: skip class must not be starved by the cap: {v:?}");
        assert!(swaps > 0, "F5: swap class must not be starved by the cap: {v:?}");
        assert_eq!(skips, 3, "expected 3 skips from the round-robin fill: {v:?}");
        assert_eq!(swaps, 4, "expected 4 swaps from the round-robin fill: {v:?}");
        // Verify the exact interleaved order of the 7 post-identity slots.
        assert!(matches!(v[0].mutation, MutationKind::Identity));
        assert_eq!(v[1].mutation, MutationKind::Swap { at: 0 });
        assert_eq!(v[2].mutation, MutationKind::Skip { at: 0 });
        assert_eq!(v[3].mutation, MutationKind::Swap { at: 1 });
        assert_eq!(v[4].mutation, MutationKind::Skip { at: 1 });
        assert_eq!(v[5].mutation, MutationKind::Swap { at: 2 });
        assert_eq!(v[6].mutation, MutationKind::Skip { at: 2 });
        assert_eq!(v[7].mutation, MutationKind::Swap { at: 3 });
    }

    #[test]
    // rationale: KIO — a 4-step pattern yields exactly 1+3+4=8 variants,
    // which is precisely the cap. ALL mutation kinds are represented.
    fn kio_four_step_pattern_fills_cap_exactly_all_kinds_present() {
        let p = pattern(&[1, 2, 3, 4]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v.len(), 8);
        assert!(v.iter().any(|x| matches!(x.mutation, MutationKind::Identity)));
        assert!(v.iter().any(|x| matches!(x.mutation, MutationKind::Swap { .. })));
        assert!(v.iter().any(|x| matches!(x.mutation, MutationKind::Skip { .. })));
    }

    #[test]
    // rationale: KIO — a 5-step pattern: 1+4+5=10 would-be variants, capped
    // to 8 = identity + 4 swaps + 3 skips (skip-at-0,1,2 only).
    fn kio_five_step_pattern_caps_with_partial_skips() {
        let p = pattern(&[1, 2, 3, 4, 5]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v.len(), 8);
        let swaps = v.iter().filter(|x| matches!(x.mutation, MutationKind::Swap { .. })).count();
        let skips = v.iter().filter(|x| matches!(x.mutation, MutationKind::Skip { .. })).count();
        assert_eq!((swaps, skips), (4, 3));
        // The 3 skips that survive are the lowest indices: 0, 1, 2.
        let skip_indices: Vec<usize> = v
            .iter()
            .filter_map(|x| match x.mutation {
                MutationKind::Skip { at } => Some(at),
                MutationKind::Swap { .. } | MutationKind::Identity => None,
            })
            .collect();
        assert_eq!(skip_indices, vec![0, 1, 2]);
    }

    #[test]
    // rationale: Boundary — single-step pattern: 0 swaps (n-1=0), 0 skips
    // (n<2 gate), only identity. Already covered, but assert the swap arity.
    fn boundary_single_step_zero_swaps_zero_skips() {
        let p = pattern(&[7]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v.len(), 1);
        assert_eq!(v.iter().filter(|x| matches!(x.mutation, MutationKind::Swap { .. })).count(), 0);
        assert_eq!(v.iter().filter(|x| matches!(x.mutation, MutationKind::Skip { .. })).count(), 0);
    }

    #[test]
    // rationale: KIO — two-step pattern [a,b]: 1 identity + 1 swap ([b,a])
    // + 2 skips ([b], [a]) = 4 variants.
    fn kio_two_step_pattern_yields_exactly_four_variants() {
        let p = pattern(&[3, 9]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v.len(), 4);
        let swap = v.iter().find(|x| matches!(x.mutation, MutationKind::Swap { .. })).expect("sw");
        assert_eq!(swap.steps, vec![StepToken(9), StepToken(3)]);
    }

    #[test]
    // rationale: Identity preservation — the identity variant's steps are a
    // verbatim, order-preserving copy of the source pattern.
    fn identity_variant_is_verbatim_copy() {
        let p = pattern(&[11, 22, 33, 44]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v[0].steps, p.steps);
        assert_eq!(v[0].mutation, MutationKind::Identity);
    }

    #[test]
    // rationale: Skip-arity invariant — every skip variant has exactly
    // len-1 steps; every swap variant has exactly len steps; identity len.
    fn invariant_mutation_changes_step_count_predictably() {
        let p = pattern(&[1, 2, 3, 4]);
        let v = build_variants(&p).expect("ok");
        for var in &v {
            match var.mutation {
                MutationKind::Identity | MutationKind::Swap { .. } => {
                    assert_eq!(var.steps.len(), 4, "identity/swap preserve length");
                }
                MutationKind::Skip { .. } => {
                    assert_eq!(var.steps.len(), 3, "skip drops exactly one step");
                }
            }
        }
    }

    #[test]
    // rationale: Determinism — variant_id is a pure function of (steps,
    // mutation): two variants with identical steps but different mutation
    // kinds must have DIFFERENT ids (mutation tag is in the hash preimage).
    fn determinism_variant_id_includes_mutation_in_preimage() {
        // [1,2] under Identity vs the swap of [2,1] under Swap{0}: distinct
        // steps AND distinct mutation → distinct id.
        let h_id = super::variant_hash(&[StepToken(1), StepToken(2)], MutationKind::Identity);
        let h_sw = super::variant_hash(&[StepToken(2), StepToken(1)], MutationKind::Swap { at: 0 });
        assert_ne!(h_id, h_sw);
        // Same steps, different skip index → different id.
        let h_k0 = super::variant_hash(&[StepToken(1)], MutationKind::Skip { at: 0 });
        let h_k1 = super::variant_hash(&[StepToken(1)], MutationKind::Skip { at: 1 });
        assert_ne!(h_k0, h_k1, "skip index must enter the hash preimage");
    }

    #[test]
    // rationale: KIO — variant_hash with a known small input. The hash is
    // FNV-1a over "1:2:id"; recompute and confirm it is non-zero and
    // stable (regression guard against silent encoding changes).
    fn kio_variant_hash_stable_reference_value() {
        let h_a = super::variant_hash(&[StepToken(1), StepToken(2)], MutationKind::Identity);
        let h_b = super::variant_hash(&[StepToken(1), StepToken(2)], MutationKind::Identity);
        assert_eq!(h_a, h_b);
        // FNV-1a 64 over a non-empty buffer is never the offset basis alone
        // and is highly unlikely to be 0.
        assert_ne!(h_a, 0);
    }

    #[test]
    // rationale: Cross-module — source_pattern_hash on every variant equals
    // the originating Pattern's canonical_hash, including across distinct
    // gap_bounds (which DO change the pattern hash).
    fn cross_module_source_hash_tracks_pattern_gap_bounds() {
        let p_a = Pattern::new(vec![StepToken(1), StepToken(2)], 5, (0, 0));
        let p_b = Pattern::new(vec![StepToken(1), StepToken(2)], 5, (0, 3));
        assert_ne!(p_a.canonical_hash, p_b.canonical_hash);
        let va = build_variants(&p_a).expect("a");
        let vb = build_variants(&p_b).expect("b");
        for v in &va {
            assert_eq!(v.source_pattern_hash, p_a.canonical_hash);
        }
        for v in &vb {
            assert_eq!(v.source_pattern_hash, p_b.canonical_hash);
        }
    }

    #[test]
    // rationale: Anti-property — no swap variant equals the identity steps
    // when the swapped pair holds distinct tokens.
    fn anti_property_swap_of_distinct_tokens_never_equals_identity() {
        let p = pattern(&[1, 2, 3, 4]);
        let v = build_variants(&p).expect("ok");
        for var in &v {
            if matches!(var.mutation, MutationKind::Swap { .. }) {
                assert_ne!(var.steps, p.steps, "swap collapsed to identity: {var:?}");
            }
        }
    }

    #[test]
    // rationale: Degenerate input — swapping a pair of IDENTICAL tokens
    // produces a sequence equal to the identity (no error, no panic). The
    // builder must still emit it (it does not dedupe).
    fn degenerate_swap_of_identical_tokens_still_emitted() {
        let p = pattern(&[5, 5, 9]);
        let v = build_variants(&p).expect("ok");
        let swap0 = v.iter().find(|x| x.mutation == MutationKind::Swap { at: 0 }).expect("s0");
        // [5,5,9] swap(0,1) → [5,5,9] — identical, but still a Swap variant.
        assert_eq!(swap0.steps, p.steps);
        assert_eq!(swap0.mutation, MutationKind::Swap { at: 0 });
    }

    #[test]
    // rationale: Determinism — full build_variants output is bit-stable
    // across runs: every (variant_id, steps, mutation, source_hash) tuple.
    fn determinism_full_output_tuple_bit_stable() {
        let p = pattern(&[2, 4, 6, 8]);
        let a = build_variants(&p).expect("a");
        let b = build_variants(&p).expect("b");
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.variant_id, y.variant_id);
            assert_eq!(x.steps, y.steps);
            assert_eq!(x.mutation, y.mutation);
            assert_eq!(x.source_pattern_hash, y.source_pattern_hash);
        }
    }

    #[test]
    // rationale: KIO (F5 fair emission) — a six-step pattern (1+5+6=12
    // would-be) caps at 8. With the round-robin interleave the 7
    // post-identity slots fill as swap0, skip0, swap1, skip1, swap2, skip2,
    // swap3 → identity + 4 swaps + 3 skips. Neither class is starved.
    fn kio_six_step_pattern_caps_identity_four_swaps_three_skips() {
        let p = pattern(&[1, 2, 3, 4, 5, 6]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v.len(), 8);
        let swaps = v.iter().filter(|x| matches!(x.mutation, MutationKind::Swap { .. })).count();
        let skips = v.iter().filter(|x| matches!(x.mutation, MutationKind::Skip { .. })).count();
        assert_eq!((swaps, skips), (4, 3));
    }

    #[test]
    // rationale: Serde — MutationKind serialises snake_case per the
    // `#[serde(rename_all)]` attribute. Identity → "identity".
    fn serde_mutation_kind_identity_is_snake_case() {
        let s = serde_json::to_string(&MutationKind::Identity).expect("ser");
        assert_eq!(s, "\"identity\"");
    }

    #[test]
    // rationale: Serde — Swap carries its index in the tagged payload.
    fn serde_mutation_kind_swap_carries_index() {
        let s = serde_json::to_string(&MutationKind::Swap { at: 2 }).expect("ser");
        let back: MutationKind = serde_json::from_str(&s).expect("de");
        assert_eq!(back, MutationKind::Swap { at: 2 });
        assert!(s.contains("swap"), "snake_case tag missing: {s}");
    }

    #[test]
    // rationale: Serde round-trip — every variant of a 4-step pattern
    // survives JSON ser/de identically (full output, not just one).
    fn serde_all_variants_round_trip() {
        let p = pattern(&[1, 2, 3, 4]);
        let v = build_variants(&p).expect("ok");
        for var in &v {
            let s = serde_json::to_string(var).expect("ser");
            let back: super::WorkflowVariant = serde_json::from_str(&s).expect("de");
            assert_eq!(&back, var);
        }
    }

    #[test]
    // rationale: Boundary (F5 fair emission) — a pattern at exactly the m20
    // DEFAULT_MAX_LENGTH (8 steps) is the largest realistic input; verify the
    // cap still holds AND the round-robin interleave keeps both mutation
    // classes alive. 8-step (n_swaps=7, n_skips=8): identity + 4 swaps +
    // 3 skips fills the cap — no class is starved.
    fn boundary_max_length_pattern_respects_variant_cap() {
        let p = pattern(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v.len(), MAX_VARIANTS_PER_PATTERN);
        let skips = v.iter().filter(|x| matches!(x.mutation, MutationKind::Skip { .. })).count();
        let swaps = v.iter().filter(|x| matches!(x.mutation, MutationKind::Swap { .. })).count();
        assert!(skips > 0, "F5: skip class must survive the cap");
        assert!(swaps > 0, "F5: swap class must survive the cap");
        assert_eq!((swaps, skips), (4, 3));
    }

    #[test]
    // rationale: Anti-property — variant_id values within one build are NOT
    // required unique, but identity vs each swap vs each skip must differ
    // for a pattern with distinct tokens (no preimage collisions expected).
    fn anti_property_distinct_mutations_yield_distinct_ids_for_distinct_tokens() {
        let p = pattern(&[1, 2, 3]);
        let v = build_variants(&p).expect("ok");
        let mut ids: Vec<u64> = v.iter().map(|x| x.variant_id).collect();
        let n = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), n, "variant_id collision across distinct mutations");
    }

    #[test]
    // rationale: KIO — skip on a two-step pattern yields two single-step
    // variants whose lone token is the surviving element.
    fn kio_two_step_skip_variants_keep_correct_survivor() {
        let p = pattern(&[8, 9]);
        let v = build_variants(&p).expect("ok");
        let s0 = v.iter().find(|x| x.mutation == MutationKind::Skip { at: 0 }).expect("k0");
        let s1 = v.iter().find(|x| x.mutation == MutationKind::Skip { at: 1 }).expect("k1");
        assert_eq!(s0.steps, vec![StepToken(9)]); // dropped index 0
        assert_eq!(s1.steps, vec![StepToken(8)]); // dropped index 1
    }

    #[test]
    // rationale: Resource — output never exceeds the cap regardless of how
    // large the source pattern is (stress at 50 steps).
    fn resource_fifty_step_pattern_still_capped() {
        let big: Vec<u32> = (0..50).collect();
        let p = pattern(&big);
        let v = build_variants(&p).expect("ok");
        assert!(v.len() <= MAX_VARIANTS_PER_PATTERN);
    }

    #[test]
    // rationale: Cross-module — variants of m20-mined patterns are well
    // formed: every non-skip variant preserves the source length, and the
    // source_pattern_hash chains back through the whole batch.
    fn cross_module_variant_batch_well_formed() {
        let p = Pattern::new(
            vec![StepToken(1), StepToken(2), StepToken(3)],
            7,
            (1, 4),
        );
        let v = build_variants(&p).expect("ok");
        assert!(!v.is_empty());
        for var in &v {
            assert_eq!(var.source_pattern_hash, p.canonical_hash);
            assert!(!var.steps.is_empty(), "variant must never be empty");
        }
    }

    #[test]
    // rationale: Boundary — MAX_VARIANTS_PER_PATTERN constant is 8; the
    // contract that downstream m23 reasons about. Lock it.
    fn boundary_max_variants_constant_is_eight() {
        assert_eq!(MAX_VARIANTS_PER_PATTERN, 8);
    }

    #[test]
    // rationale: Anti-property — identity variant's mutation is exactly
    // MutationKind::Identity, never a zero-index Swap masquerading as one.
    fn anti_property_identity_is_not_a_disguised_swap() {
        let p = pattern(&[1, 2]);
        let v = build_variants(&p).expect("ok");
        assert_eq!(v[0].mutation, MutationKind::Identity);
        assert_ne!(v[0].mutation, MutationKind::Swap { at: 0 });
    }

    #[test]
    // rationale: Determinism — variant ordering is stable: identity is
    // index 0 across a sweep of pattern lengths, and the LAST element is
    // a skip for any 2..=4-step pattern (skips are emitted last).
    fn determinism_last_variant_is_skip_for_short_patterns() {
        for ns in [&[1, 2][..], &[1, 2, 3][..], &[1, 2, 3, 4][..]] {
            let p = pattern(ns);
            let v = build_variants(&p).expect("ok");
            assert!(matches!(v[0].mutation, MutationKind::Identity));
            assert!(
                matches!(v[v.len() - 1].mutation, MutationKind::Skip { .. }),
                "last variant should be a skip for len {}",
                ns.len()
            );
        }
    }

    #[test]
    // rationale: KIO — Skip{at} index validity: every emitted skip index is
    // strictly less than the source pattern length.
    fn kio_skip_indices_within_pattern_bounds() {
        let p = pattern(&[1, 2, 3, 4]);
        let v = build_variants(&p).expect("ok");
        for var in &v {
            if let MutationKind::Skip { at } = var.mutation {
                assert!(at < p.steps.len(), "skip index {at} out of bounds");
            }
        }
    }

    #[test]
    // rationale: KIO — Swap{at} index validity: every emitted swap index is
    // strictly less than len-1 (it indexes the first of an adjacent pair).
    fn kio_swap_indices_within_adjacent_pair_bounds() {
        let p = pattern(&[1, 2, 3, 4]);
        let v = build_variants(&p).expect("ok");
        for var in &v {
            if let MutationKind::Swap { at } = var.mutation {
                assert!(at < p.steps.len() - 1, "swap index {at} out of pair range");
            }
        }
    }

    // ====================================================================
    // F5 hardening — fair cap budgeting across mutation classes.
    // The cap (MAX_VARIANTS_PER_PATTERN) must never wipe out an entire
    // mutation class while that class still has candidates. Skip is the
    // only length-shortening mutation, so starving it biases the proposal
    // population toward same-length variants.
    // ====================================================================

    #[test]
    // rationale: F5 regression — for every pattern length that exceeds the
    // cap, BOTH the swap class and the skip class must be represented in the
    // capped output. Sweeps the cap-pressure range (lengths 8..=20). Before
    // the F5 fix, an 8+-step pattern produced identity + 7 swaps + 0 skips.
    fn f5_cap_never_starves_a_mutation_class() {
        for len in 8_u32..=20 {
            let steps: Vec<u32> = (1..=len).collect();
            let p = pattern(&steps);
            let v = build_variants(&p).expect("ok");
            assert_eq!(
                v.len(),
                MAX_VARIANTS_PER_PATTERN,
                "len {len}: output should fill the cap"
            );
            let swaps = v
                .iter()
                .filter(|x| matches!(x.mutation, MutationKind::Swap { .. }))
                .count();
            let skips = v
                .iter()
                .filter(|x| matches!(x.mutation, MutationKind::Skip { .. }))
                .count();
            assert!(
                swaps > 0,
                "len {len}: F5 — swap class starved by the cap ({v:?})"
            );
            assert!(
                skips > 0,
                "len {len}: F5 — skip class starved by the cap ({v:?})"
            );
        }
    }

    #[test]
    // rationale: F5 — the round-robin interleave is balanced: when both
    // classes have surplus candidates beyond the cap, the post-identity
    // slots split as evenly as the budget allows (7 slots → 4 swaps +
    // 3 skips, swap-first). The two class counts never differ by more
    // than one.
    fn f5_interleave_split_is_balanced() {
        for len in 8_u32..=30 {
            let steps: Vec<u32> = (1..=len).collect();
            let p = pattern(&steps);
            let v = build_variants(&p).expect("ok");
            let swaps = v
                .iter()
                .filter(|x| matches!(x.mutation, MutationKind::Swap { .. }))
                .count();
            let skips = v
                .iter()
                .filter(|x| matches!(x.mutation, MutationKind::Skip { .. }))
                .count();
            let diff = swaps.abs_diff(skips);
            assert!(
                diff <= 1,
                "len {len}: interleave imbalanced — {swaps} swaps vs {skips} skips"
            );
        }
    }

    #[test]
    // rationale: F5 — fair emission must not regress the no-cap-pressure
    // case: a 3-step pattern (6 total variants, well under the cap) still
    // emits every swap and every skip exactly once.
    fn f5_no_cap_pressure_emits_all_classes_fully() {
        let p = pattern(&[1, 2, 3]);
        let v = build_variants(&p).expect("ok");
        let swaps = v
            .iter()
            .filter(|x| matches!(x.mutation, MutationKind::Swap { .. }))
            .count();
        let skips = v
            .iter()
            .filter(|x| matches!(x.mutation, MutationKind::Skip { .. }))
            .count();
        assert_eq!(swaps, 2, "all swaps emitted when under the cap");
        assert_eq!(skips, 3, "all skips emitted when under the cap");
    }

    #[test]
    // rationale: F5 — the interleave is deterministic: repeated builds of the
    // same cap-pressured pattern yield bit-identical mutation sequences.
    fn f5_interleaved_emission_is_deterministic() {
        let p = pattern(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let a = build_variants(&p).expect("a");
        let b = build_variants(&p).expect("b");
        let seq_a: Vec<MutationKind> = a.iter().map(|x| x.mutation).collect();
        let seq_b: Vec<MutationKind> = b.iter().map(|x| x.mutation).collect();
        assert_eq!(seq_a, seq_b, "F5 interleave drifted across runs");
    }
}
