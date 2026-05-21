//! Integration tests for m31 `selector` (Wave-D1).
//!
//! Exercises the m31 surface at its public-API call boundary, consuming
//! a REAL `CuratedBank` `AcceptedWorkflow` slice (NO `MockBank`):
//!
//! - Boundary — `k == 0` returns empty; `k > n` returns all.
//! - Determinism — the `(score DESC, workflow_id ASC)` tie-break is
//!   stable across a 1000-run parity check.
//! - Adversarial — a NaN diversity score is quarantined (sanitised to
//!   0.0), not propagated into the composite score or the ranking.
//! - Adversarial — a non-finite weight is refused with a typed
//!   `SelectorError::NonFiniteWeight`.
//! - Cross-module (m30 → m31) — the selector consumes `bank.active(...)`
//!   directly; the m30→m31 admission seam is exercised end-to-end.
//! - Cross-module contract — `PrunePending` rows are excluded from the
//!   `active` slice the caller hands m31 (the bank filters; m31 never
//!   sees them).
//! - Contract regression — composite scores are emitted in descending
//!   order.
//! - F11 anti-property — a diversity closure that zeroes a repeated
//!   lineage demotes it, preventing a monoculture top-k.

#![allow(clippy::doc_markdown)]

use std::time::SystemTime;

use workflow_core::m14_lift::LiftSnapshot;
use workflow_core::m20_prefixspan::{Pattern, StepToken};
use workflow_core::m21_variant_builder::build_variants;
use workflow_core::m23_proposer::{build_proposal, WorkflowProposal};
use workflow_core::m30_bank::{AcceptedWorkflow, CuratedBank, DEFAULT_PRUNE_PENDING_THRESHOLD};
use workflow_core::m31_selector::{select_top_k, SelectorConfig, SelectorError};

// ---- fixtures ------------------------------------------------------------

/// A lift snapshot above the m14 evidence floor — m23 accepts it.
fn snap() -> LiftSnapshot {
    LiftSnapshot {
        lift: Some(0.5),
        ci_half: Some(0.05),
        n: 30,
        latest_ts_ms: 0,
        computed_at: SystemTime::now(),
    }
}

/// Build a real `WorkflowProposal` through the genuine m20→m21→m23 chain
/// — `seed` distinguishes the source pattern (and therefore provenance).
fn proposal_with_seed(seed: u32) -> WorkflowProposal {
    let p = Pattern::new(
        vec![StepToken(seed), StepToken(seed + 1)],
        30,
        (0, seed as usize),
    );
    let variant = build_variants(&p).expect("variants")[0].clone();
    build_proposal(variant, &snap(), None).expect("proposal")
}

/// Seed a real `CuratedBank` with `n` accepted workflows, then return the
/// production `AcceptedWorkflow` slice via `bank.active(...)` — this is
/// exactly the input m31 consumes in production (m30 → m31 seam).
fn bank_active_slice(n: u32) -> Vec<AcceptedWorkflow> {
    let bank = CuratedBank::new();
    for s in 0..n {
        bank.accept(proposal_with_seed(s + 1), 0).expect("accept");
    }
    bank.active(1, 0.0)
}

// ---- boundary ------------------------------------------------------------

// rationale: Boundary — `k == 0` MUST return an empty Vec without error,
// even over a non-empty bank slice.
#[test]
fn m31_top_k_zero_returns_empty() {
    // rationale: Boundary (k == 0)
    let workflows = bank_active_slice(8);
    assert!(!workflows.is_empty(), "fixture bank must be populated");
    let r = select_top_k(&workflows, &SelectorConfig::default(), |_| 0.5, 0, 0)
        .expect("k==0 must not error");
    assert!(r.is_empty(), "k==0 must return an empty selection");
}

// rationale: Boundary — `k` greater than the candidate count returns ALL
// candidates (truncate is a no-op), never panics or pads.
#[test]
fn m31_top_k_greater_than_n_returns_all() {
    // rationale: Boundary (k > n)
    let workflows = bank_active_slice(5);
    let r = select_top_k(&workflows, &SelectorConfig::default(), |_| 0.5, 0, 1_000)
        .expect("k>n must not error");
    assert_eq!(
        r.len(),
        workflows.len(),
        "k>n must return every candidate, no more, no fewer"
    );
}

// ---- determinism ---------------------------------------------------------

// rationale: Determinism — when scores tie, the secondary sort is
// `workflow_id ASC`. Over 1000 runs the ranked id sequence MUST be
// bit-identical (no HashMap iteration order, no input-slice leakage).
#[test]
fn m31_tie_break_deterministic_by_score_then_workflow_id() {
    // rationale: Determinism (1000-run tie-break parity)
    let mut workflows = bank_active_slice(30);
    // All accepted workflows share weight 1.0, run_count 0, last_run None
    // → identical composite scores → the id tie-break is the only signal.
    // Reverse to ensure the result does not depend on input order.
    workflows.reverse();
    let cfg = SelectorConfig::default();
    let baseline: Vec<u64> = select_top_k(&workflows, &cfg, |_| 0.5, 0, 10)
        .expect("baseline")
        .iter()
        .map(|c| c.workflow_id)
        .collect();
    // The tie-break is ascending id — baseline must be sorted ascending.
    let mut sorted = baseline.clone();
    sorted.sort_unstable();
    assert_eq!(
        baseline, sorted,
        "tie-break must order tied scores by workflow_id ASC"
    );
    for _ in 0..1_000 {
        let ids: Vec<u64> = select_top_k(&workflows, &cfg, |_| 0.5, 0, 10)
            .expect("run")
            .iter()
            .map(|c| c.workflow_id)
            .collect();
        assert_eq!(ids, baseline, "tie-break drifted across runs");
    }
}

// ---- adversarial: NaN diversity ------------------------------------------

// rationale: Adversarial input (Wave-A regression) — a NaN diversity
// score MUST be quarantined: the composite score stays finite, the
// `diversity` component is sanitised to 0.0, and the ranking remains
// deterministic regardless of input-slice order.
#[test]
fn m31_nan_diversity_score_quarantined_not_propagated() {
    // rationale: Adversarial input (NaN diversity — Wave-A regression)
    let workflows = bank_active_slice(4);
    let cfg = SelectorConfig::default();
    let r = select_top_k(&workflows, &cfg, |_| f64::NAN, 0, 4).expect("NaN must not error");
    for c in &r {
        assert!(
            c.score.is_finite(),
            "NaN diversity leaked into the composite score: {}",
            c.score
        );
        assert!(
            (c.components.diversity - 0.0).abs() < 1e-12,
            "NaN diversity must be sanitised to 0.0, got {}",
            c.components.diversity
        );
    }
    // Ranking with NaN diversity must still be order-independent.
    let mut reversed = workflows.clone();
    reversed.reverse();
    let ids_fwd: Vec<u64> = r.iter().map(|c| c.workflow_id).collect();
    let ids_rev: Vec<u64> = select_top_k(&reversed, &cfg, |_| f64::NAN, 0, 4)
        .expect("reversed run")
        .iter()
        .map(|c| c.workflow_id)
        .collect();
    assert_eq!(
        ids_fwd, ids_rev,
        "NaN diversity must not make the ranking input-order dependent"
    );
}

// ---- adversarial: non-finite weight --------------------------------------

// rationale: Adversarial input — a non-finite weight (NaN or inf) in the
// `SelectorConfig` MUST be refused with the typed
// `SelectorError::NonFiniteWeight`, never silently sanitised.
#[test]
fn m31_nonfinite_weight_rejected_typed_error() {
    // rationale: Adversarial input (non-finite weight → typed error)
    let workflows = bank_active_slice(3);
    let inf_cfg = SelectorConfig {
        alpha: f64::INFINITY,
        beta: 0.0,
        gamma: 0.0,
        delta: 0.0,
    };
    let inf_err =
        select_top_k(&workflows, &inf_cfg, |_| 0.5, 0, 3).expect_err("inf weight must be refused");
    assert!(
        matches!(inf_err, SelectorError::NonFiniteWeight(_)),
        "infinite weight must yield NonFiniteWeight, got {inf_err:?}"
    );
    let nan_cfg = SelectorConfig {
        alpha: f64::NAN,
        beta: 0.25,
        gamma: 0.25,
        delta: 0.25,
    };
    let nan_err =
        select_top_k(&workflows, &nan_cfg, |_| 0.5, 0, 3).expect_err("NaN weight must be refused");
    assert!(
        matches!(nan_err, SelectorError::NonFiniteWeight(_)),
        "NaN weight must yield NonFiniteWeight, got {nan_err:?}"
    );
}

// ---- cross-module: m30 → m31 ---------------------------------------------

// rationale: Cross-module surface (m30 → m31) — the selector consumes the
// `AcceptedWorkflow` slice produced by the real `CuratedBank::active`,
// proving the m30→m31 admission seam end-to-end. Every selected id MUST
// trace back to a workflow the bank actually accepted.
#[test]
fn m31_selects_from_m30_curated_bank_active_set() {
    // rationale: Cross-module surface (m30 CuratedBank → m31 selector)
    let bank = CuratedBank::new();
    let mut accepted_ids = Vec::new();
    for s in 0..6_u32 {
        accepted_ids.push(bank.accept(proposal_with_seed(s + 1), 0).expect("accept"));
    }
    let active = bank.active(1, 0.0);
    assert_eq!(
        active.len(),
        accepted_ids.len(),
        "every accepted workflow must appear in the active slice"
    );
    let r = select_top_k(&active, &SelectorConfig::default(), |_| 0.5, 0, 3)
        .expect("select from real bank slice");
    assert_eq!(r.len(), 3, "top-3 over a 6-row bank");
    for c in &r {
        assert!(
            accepted_ids.contains(&c.workflow_id),
            "selected id {} was never accepted by the bank",
            c.workflow_id
        );
    }
}

// rationale: Cross-module contract (m30 → m31) — workflows whose weight
// has decayed into the `PrunePending` band are excluded from the
// `bank.active(...)` slice when the caller passes the soft threshold;
// m31 therefore can never rank a prune-pending lineage.
#[test]
fn m31_excludes_prune_pending_workflows_when_caller_filters() {
    // rationale: Cross-module contract (PrunePending exclusion)
    let bank = CuratedBank::new();
    let id_active = bank.accept(proposal_with_seed(1), 0).expect("active");
    let id_pending = bank.accept(proposal_with_seed(2), 0).expect("pending");
    // Decay one workflow's weight into the soft PrunePending band
    // (0.08 < DEFAULT_PRUNE_PENDING_THRESHOLD 0.10, above the hard floor).
    bank.apply_decay(id_pending, 0.08);
    // active() with the soft threshold filters PrunePending rows out.
    let active = bank.active(1, DEFAULT_PRUNE_PENDING_THRESHOLD);
    let active_ids: Vec<u64> = active.iter().map(|w| w.workflow_id).collect();
    assert!(
        active_ids.contains(&id_active),
        "the healthy workflow must remain in the active slice"
    );
    assert!(
        !active_ids.contains(&id_pending),
        "the prune-pending workflow must be filtered from the active slice"
    );
    // m31 over the filtered slice can only ever select the healthy id.
    let r = select_top_k(&active, &SelectorConfig::default(), |_| 0.5, 0, 10)
        .expect("select over filtered slice");
    for c in &r {
        assert_ne!(
            c.workflow_id, id_pending,
            "m31 must never rank a prune-pending workflow"
        );
    }
}

// ---- contract regression -------------------------------------------------

// rationale: Contract regression — `select_top_k` emits candidates in
// descending composite-score order; every adjacent pair MUST satisfy
// `score[i] >= score[i+1]`.
#[test]
fn m31_composite_score_ordering_is_descending() {
    // rationale: Contract regression (descending score order)
    // Build a real bank, then vary weight via decay so scores differ.
    let bank = CuratedBank::new();
    let mut ids = Vec::new();
    for s in 0..10_u32 {
        ids.push(bank.accept(proposal_with_seed(s + 1), 0).expect("accept"));
    }
    // Apply graded decay so each workflow ends with a distinct weight.
    for (rank, id) in ids.iter().enumerate() {
        #[allow(
            clippy::cast_precision_loss,
            reason = "rank is bounded by 10; precision irrelevant"
        )]
        let factor = 0.1 + (rank as f64) * 0.08;
        bank.apply_decay(*id, factor);
    }
    let active = bank.active(1, 0.0);
    let r = select_top_k(&active, &SelectorConfig::default(), |_| 0.5, 0, active.len())
        .expect("scored selection");
    for win in r.windows(2) {
        assert!(
            win[0].score >= win[1].score,
            "score ordering not descending: {} then {}",
            win[0].score,
            win[1].score
        );
    }
}

// ---- F11 anti-property: monoculture suppression --------------------------

// rationale: Anti-property (F11) — a diversity closure that zeroes a
// repeated lineage demotes it below an otherwise-identical sibling,
// preventing a monoculture top-k. m31 enforces the arithmetic half of
// the F11 gate; the cooldown table lives in the caller's closure.
#[test]
fn m31_diversity_enforcement_prevents_monoculture_top_k() {
    // rationale: Anti-property (F11 monoculture suppression)
    let bank = CuratedBank::new();
    let id_repeat = bank.accept(proposal_with_seed(1), 0).expect("repeat");
    let id_fresh = bank.accept(proposal_with_seed(2), 0).expect("fresh");
    let active = bank.active(1, 0.0);
    // Cooldown closure: the "repeat" lineage gets diversity 0.0; the
    // fresh lineage gets full diversity. With identical fitness/recency/
    // frequency, the δ-component must flip the ordering.
    let r = select_top_k(
        &active,
        &SelectorConfig::default(),
        |w| {
            if w.workflow_id == id_repeat {
                0.0
            } else {
                1.0
            }
        },
        0,
        2,
    )
    .expect("diversity-weighted selection");
    assert_eq!(r.len(), 2);
    assert_eq!(
        r[0].workflow_id, id_fresh,
        "the diverse lineage must rank first when the repeat is cooled down"
    );
    assert_eq!(
        r[1].workflow_id, id_repeat,
        "the cooled-down repeat lineage must be demoted"
    );
    // The demoted row's diversity component is exactly the zeroed value.
    assert!(
        (r[1].components.diversity - 0.0).abs() < 1e-12,
        "cooled-down lineage must carry diversity 0.0"
    );
}
