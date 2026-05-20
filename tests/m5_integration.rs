//! Integration tests for m5 battern_step_record (Wave-C2).
//!
//! Exercises the m5 public surface from outside the crate:
//!
//! - Determinism of `derive_battern_id` on identical first-dispatch ts.
//! - F11 anti-property: `BatternId` does not leak dispatch command
//!   substrings.
//! - Battern boundary rule — Design → Dispatch → Gate stays one battern.
//! - Battern boundary rule — second dispatch opens a new battern.
//! - Boundary — empty command sequence yields empty observations.
//! - Contract regression — `MIN_COMPLETE_STEPS` matches the default
//!   `BatternStepRecordConfig::min_steps`, gating `summarise` semantics.
//!
//! Per Wave-A1 scout finding F-A-02, `BatternId(pub String)` exposes its
//! inner string. We test the AS-IS public surface and do NOT mutate the
//! type.

#![allow(clippy::doc_markdown)]

use workflow_core::m4_cascade::AtuinStep;
use workflow_core::m5_battern::{
    derive_battern_id, BatternStepLabel, BatternStepRecord, BatternStepRecordConfig,
    MIN_COMPLETE_STEPS,
};

fn step(ts_ns: i64, cmd: &str, session: &str) -> AtuinStep {
    AtuinStep {
        id: format!("ulid-{ts_ns}"),
        ts_ns,
        command: cmd.to_owned(),
        cwd: "/tmp".into(),
        session: session.to_owned(),
        exit: 0,
    }
}

fn rec() -> BatternStepRecord {
    BatternStepRecord::new(BatternStepRecordConfig::default()).expect("regex compile")
}

// rationale: Determinism — `derive_battern_id` is a pure function of the
// first-dispatch timestamp; same input must always yield same id.
#[test]
fn m5_battern_id_is_deterministic_for_same_observations() {
    let r = rec();
    let steps = vec![
        step(1_700_000_000_000_000_000, "cc-dispatch ALPHA", "s1"),
        step(1_700_000_001_000_000_000, "cc-health", "s1"),
    ];
    let obs_a = r.observe(&steps);
    let obs_b = r.observe(&steps);
    assert!(!obs_a.is_empty());
    assert_eq!(obs_a.len(), obs_b.len());
    for (a, b) in obs_a.iter().zip(obs_b.iter()) {
        assert_eq!(a.battern_id, b.battern_id, "battern_id drift across calls");
        assert_eq!(a.step_index, b.step_index);
        assert_eq!(a.step_label, b.step_label);
    }
    // Direct derive_battern_id determinism.
    let direct_a = derive_battern_id(1_700_000_000_000_000_000);
    let direct_b = derive_battern_id(1_700_000_000_000_000_000);
    assert_eq!(direct_a, direct_b);
}

// rationale: Anti-property F11 — battern_id is an opaque hex hash; it
// MUST NOT leak any dispatch-command substring even when the command
// carries semantically meaningful labels (`ALPHA-LEFT`, `cc-dispatch`).
#[test]
fn m5_battern_id_does_not_leak_dispatch_command_substring() {
    let r = rec();
    let steps = vec![
        step(1_000_000_000, "cc-dispatch ALPHA-LEFT-meaningful-pane", "s1"),
        step(2_000_000_000, "cc-health", "s1"),
    ];
    let obs = r.observe(&steps);
    assert!(!obs.is_empty());
    let id_str = obs[0].battern_id.as_str();
    assert!(
        id_str.starts_with("battern_"),
        "battern_id prefix drift: {id_str:?}",
    );
    let suffix = &id_str["battern_".len()..];
    assert_eq!(suffix.len(), 16, "expected 16-hex suffix, got {suffix:?}");
    assert!(suffix.chars().all(|c| c.is_ascii_hexdigit()));
    for forbidden in ["ALPHA", "LEFT", "meaningful", "pane", "dispatch", "health", "cc-"] {
        assert!(
            !id_str.contains(forbidden),
            "F11 leak: {forbidden:?} in battern_id {id_str:?}",
        );
    }
}

// rationale: Battern boundary rule — Design → Dispatch → Gate is ONE
// battern. The boundary rule states `cc-dispatch` opens a new battern
// only if the CURRENT battern already saw a dispatch.
#[test]
fn m5_design_then_dispatch_then_gate_stays_one_battern() {
    let r = rec();
    let steps = vec![
        step(1_000_000_000, "rg foo", "s1"),
        step(2_000_000_000, "cc-dispatch ALPHA", "s1"),
        step(3_000_000_000, "cc-health", "s1"),
    ];
    let obs = r.observe(&steps);
    assert_eq!(obs.len(), 3);
    let ids: std::collections::HashSet<_> = obs.iter().map(|o| o.battern_id.clone()).collect();
    assert_eq!(
        ids.len(),
        1,
        "Design→Dispatch→Gate must collapse to ONE battern, got {} ids",
        ids.len(),
    );
}

// rationale: Battern boundary rule — a SECOND dispatch (e.g.
// Dispatch→Gate→Dispatch) opens a NEW battern. This is the symmetric
// half of the rule above.
#[test]
fn m5_second_dispatch_opens_new_battern() {
    let r = rec();
    let steps = vec![
        step(1_000_000_000, "cc-dispatch A", "s1"),
        step(2_000_000_000, "cc-health", "s1"),
        step(3_000_000_000, "cc-dispatch B", "s1"),
        step(4_000_000_000, "cc-health", "s1"),
    ];
    let obs = r.observe(&steps);
    let ids: std::collections::HashSet<_> = obs.iter().map(|o| o.battern_id.clone()).collect();
    assert_eq!(
        ids.len(),
        2,
        "second dispatch must open a new battern; got {} ids",
        ids.len(),
    );
}

// rationale: Boundary — empty command sequence yields empty
// observations (no fabricated battern, no allocator surprise).
#[test]
fn m5_observe_handles_empty_command_sequence() {
    let r = rec();
    assert!(r.observe(&[]).is_empty());
}

// rationale: Contract regression — exported `MIN_COMPLETE_STEPS` must
// equal the default `BatternStepRecordConfig::min_steps`; otherwise
// `summarise` and `observe` disagree on completeness gating.
#[test]
fn m5_min_complete_steps_const_matches_summarise_threshold() {
    let default_cfg = BatternStepRecordConfig::default();
    assert_eq!(
        MIN_COMPLETE_STEPS, default_cfg.min_steps,
        "MIN_COMPLETE_STEPS / min_steps drift",
    );
    // Cross-check with `summarise`: a one-step observation set is NOT
    // complete; a two-step observation set IS (when not partial).
    let r = rec();
    let steps_below = vec![step(1, "cc-dispatch A", "s1")];
    let obs_below = r.observe(&steps_below);
    // min_steps = 2 default; the single dispatch is dropped.
    assert!(obs_below.is_empty());
    let steps_at = vec![
        step(1, "cc-dispatch A", "s1"),
        step(2_000_000_000, "cc-health", "s1"),
    ];
    let obs_at = r.observe(&steps_at);
    let rec_at = BatternStepRecord::summarise(&obs_at);
    assert!(rec_at.total_steps >= MIN_COMPLETE_STEPS);
}

// rationale: Cross-module surface — the public `BatternStepLabel`
// variants used as the heuristic table are present and exhaustively
// reachable via `label_command`. Drift detection for the soft schema.
#[test]
fn m5_label_command_exposes_all_five_labels_via_public_surface() {
    let r = rec();
    assert_eq!(
        r.label_command("cc-dispatch ALPHA"),
        Some(BatternStepLabel::Dispatch),
    );
    assert_eq!(r.label_command("cc-health"), Some(BatternStepLabel::Gate));
    assert_eq!(r.label_command("cc-harvest"), Some(BatternStepLabel::Collect));
    assert_eq!(
        r.label_command("cc-cascade --to BETA"),
        Some(BatternStepLabel::Compose),
    );
    assert_eq!(r.label_command("rg foo"), Some(BatternStepLabel::Design));
    assert!(r.label_command("totally-unmatched-command").is_none());
}
