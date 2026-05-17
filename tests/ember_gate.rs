//! Day-1 Ember 7-trait CI gate test.
//!
//! Per m10 spec § 5: walks every entry of
//! `workflow_core::user_facing_strings::ALL` through the rubric. Rejected
//! verdicts trigger a panic naming `key + trait + reason`. Held verdicts
//! also panic UNLESS covered by an unexpired allowlist row in
//! `tests/ember_held_approvals.tsv` per the D-C hybrid CI-FAIL+allowlist
//! decision (S1002127). Allowlisted Held verdicts emit
//! `EMBER-HELD(allowlisted)` to stderr.
//!
//! For Day-1 the registry is empty (downstream modules m11/m12/m23/m32
//! populate it as they ship), so this test passes vacuously. It is the
//! discipline anchor that grows teeth the moment the registry does.
//!
//! Rubric reference: `~/projects/claude_code/Ember 7-Trait Gate Rubric.md`
//! Authority: The Watcher \u{2624} (S1001882) — Zen audit 2026-05-16T224523Z.
//! HELD semantics (W3 flag, B4 closed via D-C decision 2026-05-17 S1002127):
//! Held verdicts are CI failures unless explicitly allowlisted with
//! HumanAcceptanceSignature.

#![allow(clippy::doc_markdown)]

use workflow_core::m10_ember_ci_gate::{evaluate_string, load_approvals, GateVerdict};
use workflow_core::user_facing_strings::ALL;

#[test]
fn ember_gate_all_user_facing_strings() {
    let approvals = load_approvals("tests/ember_held_approvals.tsv").unwrap_or_default();
    let mut rejections: Vec<(String, &'static str, String)> = Vec::new();
    let mut held: Vec<(String, &'static str, String, f64)> = Vec::new();

    for (key, text) in ALL {
        match evaluate_string(key, text, &approvals) {
            GateVerdict::Pass => {}
            GateVerdict::Fail {
                key,
                trait_name,
                reason,
            } => {
                rejections.push((key, trait_name.as_str(), reason));
            }
            GateVerdict::HeldFailed {
                key,
                trait_name,
                reason,
                confidence,
            } => {
                held.push((key, trait_name.as_str(), reason, confidence));
            }
            GateVerdict::HeldAllowlisted {
                key, approved_by, ..
            } => {
                eprintln!(
                    "EMBER-HELD(allowlisted)  key={key}  approved_by={approved_by}"
                );
            }
        }
    }

    for (k, t, r) in &rejections {
        eprintln!("EMBER-REJECT  key={k}  trait={t}  reason={r}");
    }
    for (k, t, r, c) in &held {
        eprintln!("EMBER-HELD(W3-fail)  key={k}  trait={t}  reason={r}  confidence={c}");
    }

    assert!(
        rejections.is_empty() && held.is_empty(),
        "Ember gate: {} rejected, {} held-as-fail (W3 flag, D-C hybrid CI-FAIL+allowlist). \
         Fix the string or add an unexpired allowlist row with HumanAcceptanceSignature.",
        rejections.len(),
        held.len()
    );
}
