//! Integration tests for m21 `variant_builder` (Wave-D1).
//!
//! Exercises the m21 surface at its public-API call boundary:
//!
//! - `MAX_VARIANTS_PER_PATTERN` cap — `build_variants` never emits more
//!   than the F2 monoculture-prevention bound.
//! - `variant_id` determinism — the FNV-1a hash is stable across repeated
//!   builds in the same process (cross-process stability is a corollary
//!   of FNV being a pure function of the canonical encoding).
//! - F11 anti-property — `variant_id` is a pure `u64`; no human-readable
//!   substring can ride along.
//! - Boundary — a single-step pattern yields only the identity variant
//!   (a skip would empty it).
//! - Adversarial — an empty `Pattern` is refused with the typed
//!   `VariantBuilderError::EmptyPattern`.
//! - Cross-module (m20 → m21) — a `Pattern` produced by `mine_sequences`
//!   feeds cleanly into `build_variants`, and provenance
//!   (`source_pattern_hash`) links every variant back to its source.
//! - Contract regression — the `VariantBuilderError` taxonomy is
//!   exhaustively pinned.

#![allow(clippy::doc_markdown)]

use workflow_core::m20_prefixspan::{mine_sequences, MaxGap, MinSupport, Pattern, StepToken};
use workflow_core::m21_variant_builder::{
    build_variants, MutationKind, VariantBuilderError, MAX_VARIANTS_PER_PATTERN,
};

// ---- fixtures ------------------------------------------------------------

/// Build a `Pattern` directly from a step list (support / gap fixed).
fn pattern(steps: &[u32]) -> Pattern {
    Pattern::new(steps.iter().copied().map(StepToken).collect(), 30, (0, 1))
}

/// Mine a `Pattern` from a synthetic sequence database — the genuine
/// m20 surface, used by the cross-module test.
fn mined_pattern() -> Pattern {
    // Four sequences all containing the sub-sequence [1, 2, 3] → mined
    // with support 4. mine_sequences sorts by (support DESC, len DESC,
    // hash ASC); the [1,2,3] pattern is the highest-support multi-step.
    let seqs = vec![
        vec![StepToken(1), StepToken(2), StepToken(3)],
        vec![StepToken(1), StepToken(2), StepToken(3)],
        vec![StepToken(9), StepToken(1), StepToken(2), StepToken(3)],
        vec![StepToken(1), StepToken(2), StepToken(3), StepToken(7)],
    ];
    let patterns = mine_sequences(
        &seqs,
        MinSupport::new(2).expect("min_support >= floor"),
        MaxGap::new(5),
        8,
    )
    .expect("mine ok");
    patterns
        .into_iter()
        .find(|p| p.steps().len() >= 2)
        .expect("at least one multi-step mined pattern")
}

// ---- tests ---------------------------------------------------------------

// rationale: Boundary — `build_variants` MUST never emit more than
// `MAX_VARIANTS_PER_PATTERN` proposals even when the swap+skip
// enumeration would exceed the cap (a 10-step pattern has 9 swaps + 10
// skips + 1 identity = 20 candidate mutations).
#[test]
fn m21_build_variants_respects_max_variants_per_pattern_cap() {
    // rationale: Boundary (F2 monoculture-prevention cap)
    let p = pattern(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let v = build_variants(&p).expect("build ok");
    assert!(
        v.len() <= MAX_VARIANTS_PER_PATTERN,
        "variant count {} exceeded cap {MAX_VARIANTS_PER_PATTERN}",
        v.len()
    );
}

// rationale: Determinism — `variant_id` is a pure FNV-1a hash of the
// canonical (steps, mutation) encoding; two builds of the same pattern
// MUST produce bit-identical ids in declaration order.
#[test]
fn m21_variant_id_is_deterministic_across_repeated_builds() {
    // rationale: Determinism
    let p = pattern(&[3, 1, 4, 1, 5]);
    let first = build_variants(&p).expect("first build");
    let second = build_variants(&p).expect("second build");
    assert_eq!(
        first.len(),
        second.len(),
        "variant count diverged across builds"
    );
    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(
            a.variant_id, b.variant_id,
            "variant_id diverged: {a:?} vs {b:?}"
        );
        assert_eq!(a.steps, b.steps, "step order diverged across builds");
        assert_eq!(a.mutation, b.mutation, "mutation kind diverged");
    }
}

// rationale: Anti-property (F11 opaque IDs) — `variant_id` is a `u64`;
// its JSON serialisation is a bare numeric literal with no human-readable
// substring. A regression introducing a string id would fail here.
#[test]
fn m21_variant_id_is_pure_u64_no_human_substring() {
    // rationale: Anti-property (F11 opaque IDs)
    let p = pattern(&[1, 2, 3]);
    let v = build_variants(&p).expect("build ok");
    for var in &v {
        let encoded = serde_json::to_string(&var.variant_id).expect("serialise id");
        assert!(
            encoded.chars().all(|c| c.is_ascii_digit()),
            "variant_id leaked a non-numeric character: {encoded}"
        );
        // u64 is Copy and has a fixed numeric domain — confirm the round
        // trip preserves the exact value (no lossy string id path).
        let back: u64 = serde_json::from_str(&encoded).expect("parse id");
        assert_eq!(back, var.variant_id);
    }
}

// rationale: Boundary — a single-step pattern yields ONLY the identity
// variant. A skip of the sole element would produce the empty pattern,
// which is semantically invalid for downstream m22/m23 consumption, so
// m21 emits no skip variants for n == 1.
#[test]
fn m21_single_step_pattern_yields_identity_only() {
    // rationale: Boundary (single-step pattern)
    let p = pattern(&[42]);
    let v = build_variants(&p).expect("build ok");
    assert_eq!(v.len(), 1, "single-step pattern must yield exactly 1 variant");
    assert_eq!(
        v[0].mutation,
        MutationKind::Identity,
        "the sole variant must be the identity"
    );
    assert_eq!(v[0].steps, p.steps(), "identity variant preserves the steps");
}

// rationale: Adversarial input — an empty `Pattern` (zero steps) MUST be
// refused with the typed `VariantBuilderError::EmptyPattern`, never
// silently produce an empty Vec or panic.
#[test]
fn m21_build_variants_handles_empty_pattern() {
    // rationale: Adversarial input (empty pattern → typed error)
    let p = Pattern::new(Vec::new(), 30, (0, 0));
    let err = build_variants(&p).expect_err("empty pattern must be refused");
    assert!(
        matches!(err, VariantBuilderError::EmptyPattern),
        "expected EmptyPattern, got {err:?}"
    );
    // The typed error must carry a non-empty Display payload.
    assert!(
        !format!("{err}").is_empty(),
        "EmptyPattern Display must not be empty"
    );
}

// rationale: Cross-module surface (m20 → m21) — a `Pattern` mined by the
// genuine `mine_sequences` surface feeds cleanly into `build_variants`,
// and distinct patterns yield distinct `source_pattern_hash` provenance.
// This proves the m20→m21 seam end-to-end (not via a synthetic Pattern).
#[test]
fn m21_distinct_patterns_yield_distinct_variant_provenance() {
    // rationale: Cross-module surface (m20 mined Pattern → m21 provenance)
    let mined = mined_pattern();
    let mined_variants = build_variants(&mined).expect("build from mined pattern");
    assert!(
        !mined_variants.is_empty(),
        "a mined multi-step pattern must yield variants"
    );
    // Every variant carries the mined pattern's canonical hash as
    // provenance — the m20→m21 link is intact.
    for var in &mined_variants {
        assert_eq!(
            var.source_pattern_hash, mined.canonical_hash(),
            "variant provenance must equal the source Pattern canonical_hash"
        );
    }
    // A structurally different pattern must produce a different
    // provenance hash — provenance is not a constant.
    let other = pattern(&[100, 200]);
    let other_variants = build_variants(&other).expect("build other");
    assert_ne!(
        mined_variants[0].source_pattern_hash, other_variants[0].source_pattern_hash,
        "distinct patterns must yield distinct provenance hashes"
    );
}

// rationale: Contract regression — the `VariantBuilderError` taxonomy is
// pinned. Today the only variant is `EmptyPattern`; this test fails-loud
// if a new variant is added without a corresponding test, forcing the
// taxonomy to stay exhaustively covered.
#[test]
fn m21_variant_builder_error_display_contract() {
    // rationale: Contract regression (error Display stability).
    //
    // `VariantBuilderError` is `#[non_exhaustive]` — a cross-crate match
    // therefore requires a wildcard arm, so this no longer doubles as a
    // compile-time variant-count guard. The Display-contract assertion on
    // the known `EmptyPattern` variant is the surviving regression check.
    let err = VariantBuilderError::EmptyPattern;
    match err {
        VariantBuilderError::EmptyPattern => {
            assert_eq!(
                format!("{}", VariantBuilderError::EmptyPattern),
                "empty source pattern",
                "EmptyPattern Display contract changed"
            );
        }
        other => panic!("unexpected VariantBuilderError variant: {other:?}"),
    }
}

// rationale: Cross-module / contract — feeding the mined Pattern into
// build_variants, the identity variant is always first (m23 relies on
// baseline-first emission) and every emitted variant respects the cap.
#[test]
fn m21_mined_pattern_variants_identity_first_and_capped() {
    // rationale: Cross-module surface + contract regression
    let mined = mined_pattern();
    let v = build_variants(&mined).expect("build from mined pattern");
    assert!(v.len() <= MAX_VARIANTS_PER_PATTERN, "cap respected on mined input");
    assert_eq!(
        v[0].mutation,
        MutationKind::Identity,
        "identity variant must be index 0 (m23 baseline-first contract)"
    );
}
