//! `m10_ember_ci_gate` — output-time 7-trait Ember rubric CI gate.
//!
//! See [m10 spec](../../../ai_specs/modules/cluster-D/m10_ember_ci_gate.md).
//!
//! # Canonical rubric reference
//!
//! `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` (canonical
//! semantic definition; NOT embedded here per spec § 5). The heuristic
//! implementation in [`rubric`] is the test surface; the canonical rubric
//! is the operational-discipline authority.
//!
//! # Day-1 posture: hybrid CI-FAIL + allowlist (D-C decision, S1002127)
//!
//! - `Rejected` verdicts (confidence ≥ 0.5) → CI-FAIL, **NEVER**
//!   allowlistable (operator approval cannot override Rejected — this is
//!   the discipline preservation rule).
//! - `Held` verdicts (confidence < 0.5) → CI-FAIL by default; an unexpired
//!   allowlist row in `tests/ember_held_approvals.tsv` (with
//!   `HumanAcceptanceSignature` in the `approved_by` field) permits passage
//!   with an operator-visible `tracing::warn!` emission.
//! - `Approved` → silent pass.
//!
//! B4 (Watcher Ember §5.1 amendment) is CLOSED per D-C decision; the
//! Watcher's matching service-adoption clause will land in the next
//! Watcher cycle, after which the gate posture is unchanged (only the
//! semantic anchor in the rubric document updates).
//!
//! # Day-1 registry status
//!
//! `crate::user_facing_strings::ALL` is empty at Day-1. Modules m11, m12,
//! m23, m32 populate the registry as they ship. The CI gate test in
//! `tests/ember_gate.rs` therefore passes vacuously and acts as the
//! discipline anchor until the registry grows.
//!
//! # Public surface
//!
//! - [`score_against_rubric`] / [`EmberStatus`] / [`TraitName`] — pure
//!   scoring primitive.
//! - [`evaluate_string`] / [`GateVerdict`] — gate decision aggregator.
//! - [`load_approvals`] / [`HeldApproval`] / [`is_approved`] /
//!   [`is_approved_at`] — TSV allowlist surface.
//! - [`EmberGateError`] — typed failure modes.

pub mod allowlist;
pub mod error;
pub mod gate;
pub mod rubric;

pub use allowlist::{is_approved, is_approved_at, load_approvals, HeldApproval};
pub use error::EmberGateError;
pub use gate::{evaluate_string, evaluate_string_at, GateVerdict};
pub use rubric::{score_against_rubric, EmberStatus, TraitName};

#[cfg(test)]
mod tests {
    use super::{
        evaluate_string, load_approvals, score_against_rubric, EmberStatus, GateVerdict,
        TraitName,
    };

    #[test]
    fn reexports_score_function() {
        assert_eq!(
            score_against_rubric("POVM probe at 2026-05-17 scope=lib"),
            EmberStatus::Approved
        );
    }

    #[test]
    fn reexports_evaluate_function() {
        assert_eq!(
            evaluate_string("k", "POVM probe at 2026-05-17 scope=lib", &[]),
            GateVerdict::Pass
        );
    }

    #[test]
    fn reexports_trait_name() {
        assert_eq!(TraitName::Equanimity.as_str(), "Equanimity");
    }

    #[test]
    fn reexports_load_approvals_returns_empty_on_missing_file() {
        let rows = load_approvals("/tmp/nonexistent-allowlist-9f3e7a1b.tsv")
            .expect("missing → empty");
        assert!(rows.is_empty());
    }
}
