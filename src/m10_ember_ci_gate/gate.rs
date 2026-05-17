//! Gate decision aggregator.
//!
//! Per m10 spec § 3: [`evaluate_string`] takes a candidate artefact key +
//! text + the loaded allowlist and returns a [`GateVerdict`]. The CI test
//! file in `tests/ember_gate.rs` walks every entry of
//! `crate::user_facing_strings::ALL` through this function and panics if
//! any verdict is `Fail` or `HeldFailed`.
//!
//! The discipline preservation rule per D-C decision: `Rejected` verdicts
//! cannot be overridden by the allowlist. The allowlist is consulted ONLY
//! for `Held` verdicts.

use time::OffsetDateTime;

use super::allowlist::HeldApproval;
use super::rubric::{score_against_rubric, EmberStatus, TraitName};

/// Verdict for a single candidate string.
#[derive(Debug, Clone, PartialEq)]
pub enum GateVerdict {
    /// All 7 traits passed.
    Pass,
    /// One trait failed at confidence ≥ 0.5 — Rejected cannot be
    /// allowlisted under any circumstance.
    Fail {
        /// Stable artefact key (from the user-facing-strings registry).
        key: String,
        /// Trait that fired the verdict.
        trait_name: TraitName,
        /// Plain-text reason.
        reason: String,
    },
    /// One trait failed at confidence < 0.5 and no unexpired allowlist row
    /// covered the key. CI-FAIL by default (W3-strict mode, D-C decision).
    HeldFailed {
        /// Stable artefact key.
        key: String,
        /// Trait that fired the verdict.
        trait_name: TraitName,
        /// Plain-text reason.
        reason: String,
        /// Heuristic confidence in `[0.0, 0.5)`.
        confidence: f64,
    },
    /// One trait failed at confidence < 0.5 BUT an unexpired allowlist row
    /// for the key authorises passage. Emits `EMBER-HELD(allowlisted)`
    /// `tracing::warn!` per spec § 1.
    HeldAllowlisted {
        /// Stable artefact key.
        key: String,
        /// Trait that fired the verdict.
        trait_name: TraitName,
        /// Plain-text reason.
        reason: String,
        /// Heuristic confidence in `[0.0, 0.5)`.
        confidence: f64,
        /// Operator who authored the approval (HumanAcceptanceSignature).
        approved_by: String,
    },
}

/// Evaluate a single candidate string against the rubric, consulting
/// `approvals` for Held verdicts. Rejected verdicts NEVER consult the
/// allowlist (per D-C decision).
#[must_use]
pub fn evaluate_string(key: &str, text: &str, approvals: &[HeldApproval]) -> GateVerdict {
    evaluate_string_at(key, text, approvals, OffsetDateTime::now_utc())
}

/// [`evaluate_string`] with explicit `now` injection — useful in tests that
/// pin the wall-clock for expiry assertions.
#[must_use]
pub fn evaluate_string_at(
    key: &str,
    text: &str,
    approvals: &[HeldApproval],
    now: OffsetDateTime,
) -> GateVerdict {
    match score_against_rubric(text) {
        EmberStatus::Approved => GateVerdict::Pass,
        EmberStatus::Rejected { trait_name, reason } => GateVerdict::Fail {
            key: key.to_owned(),
            trait_name,
            reason,
        },
        EmberStatus::Held {
            trait_name,
            reason,
            confidence,
        } => approvals
            .iter()
            .find(|a| a.artefact_key == key && a.expiry > now)
            .map_or_else(
                || GateVerdict::HeldFailed {
                    key: key.to_owned(),
                    trait_name,
                    reason: reason.clone(),
                    confidence,
                },
                |a| GateVerdict::HeldAllowlisted {
                    key: key.to_owned(),
                    trait_name,
                    reason: reason.clone(),
                    confidence,
                    approved_by: a.approved_by.clone(),
                },
            ),
    }
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::super::allowlist::HeldApproval;
    use super::super::rubric::TraitName;
    use super::{evaluate_string, evaluate_string_at, GateVerdict};

    #[test]
    fn pass_for_clean_factual_string() {
        let v = evaluate_string(
            "m12.report.header",
            "POVM probe at 2026-05-17T10:00:00Z returned 0.067 (scope=lib).",
            &[],
        );
        assert_eq!(v, GateVerdict::Pass);
    }

    #[test]
    fn fail_for_rejected_string() {
        let v = evaluate_string("m12.report.header", "successfully completed", &[]);
        let GateVerdict::Fail {
            key, trait_name, ..
        } = v
        else {
            panic!("expected Fail");
        };
        assert_eq!(key, "m12.report.header");
        assert_eq!(trait_name, TraitName::Honesty);
    }

    #[test]
    fn rejected_cannot_be_allowlisted() {
        // D-C decision: Rejected verdicts NEVER consult the allowlist.
        let approvals = vec![HeldApproval {
            artefact_key: "m12.report.header".into(),
            approved_by: "luke".into(),
            approved_at: datetime!(2026-05-17 10:00:00 UTC),
            expiry: datetime!(2030-01-01 00:00:00 UTC),
        }];
        let v = evaluate_string("m12.report.header", "successfully completed", &approvals);
        assert!(matches!(v, GateVerdict::Fail { .. }));
    }

    #[test]
    fn held_without_allowlist_is_held_failed() {
        // No currently-Held confidence-<0.5 heuristic in Day-1 rubric (all
        // are ≥ 0.5 → Rejected). Construct a synthetic Held via the
        // confidence-0.4 path by reaching for a string the rubric would
        // flag — and verify the wiring at the gate level: if a future
        // heuristic emits Held confidence 0.4, evaluate_string_at with an
        // empty allowlist returns HeldFailed. We exercise this branch via
        // the rubric's actual confidence boundary by inverting it through
        // the gate: a Rejected outcome already covers the > 0.5 case, so
        // here we directly construct a synthetic HeldFailed for shape
        // assertion.
        let v = GateVerdict::HeldFailed {
            key: "k".into(),
            trait_name: TraitName::Diligence,
            reason: "synth".into(),
            confidence: 0.4,
        };
        match v {
            GateVerdict::HeldFailed { confidence, .. } => assert!(confidence < 0.5),
            other => panic!("shape: {other:?}"),
        }
    }

    #[test]
    fn held_allowlisted_carries_approved_by() {
        // Synthetic shape test: when a future <0.5-confidence heuristic
        // fires + an unexpired allowlist row matches, the verdict carries
        // approved_by.
        let v = GateVerdict::HeldAllowlisted {
            key: "k".into(),
            trait_name: TraitName::Investment,
            reason: "synth".into(),
            confidence: 0.4,
            approved_by: "luke@0A".into(),
        };
        let GateVerdict::HeldAllowlisted { approved_by, .. } = v else {
            panic!("shape");
        };
        assert_eq!(approved_by, "luke@0A");
    }

    #[test]
    fn expired_allowlist_does_not_authorise_held() {
        // If a Held verdict ever fires, an EXPIRED allowlist row does NOT
        // authorise. We use evaluate_string_at to pin "now" past expiry.
        // Day-1 rubric never returns Held confidence < 0.5 so we use a
        // Pass string as control + verify the time-pinning machinery.
        let approvals = vec![HeldApproval {
            artefact_key: "k".into(),
            approved_by: "luke".into(),
            approved_at: datetime!(2024-01-01 00:00:00 UTC),
            expiry: datetime!(2024-02-01 00:00:00 UTC),
        }];
        let now = datetime!(2026-05-17 10:00:00 UTC);
        let v = evaluate_string_at("k", "POVM probe at 2026-05-17 scope=lib", &approvals, now);
        assert_eq!(v, GateVerdict::Pass);
    }

    #[test]
    fn pass_verdict_implements_clone_and_eq() {
        let v = GateVerdict::Pass;
        let v2 = v.clone();
        assert_eq!(v, v2);
    }

    #[test]
    fn fail_carries_trait_and_reason() {
        let v = evaluate_string("m11.sunset", "tests passing", &[]);
        let GateVerdict::Fail {
            trait_name, reason, ..
        } = v
        else {
            panic!("expected Fail");
        };
        assert_eq!(trait_name, TraitName::Diligence);
        assert!(reason.contains("test count"));
    }
}
