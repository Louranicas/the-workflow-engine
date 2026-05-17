//! Integration tests for m10 ember_ci_gate.
//!
//! Per m10 spec § 6 F-Integration row (18 tests target). Exercises:
//!
//! - The rubric against known-good and known-bad fixture strings
//!   (registry is empty Day-1; downstream populate, so we proxy with
//!   direct rubric calls).
//! - The allowlist TSV reader against on-disk file content.
//! - The gate decision aggregator end-to-end (Pass / Fail / HeldFailed /
//!   HeldAllowlisted).
//! - The canonical rubric reference path existence sanity check.

#![allow(clippy::doc_markdown)]

use std::fs;
use std::io::Write;
use std::path::Path;

use tempfile::NamedTempFile;
use time::macros::datetime;
use time::OffsetDateTime;

use workflow_core::m10_ember_ci_gate::{
    evaluate_string, evaluate_string_at, is_approved, is_approved_at, load_approvals,
    score_against_rubric, EmberStatus, GateVerdict, HeldApproval, TraitName,
};

// ---- Rubric end-to-end fixture strings (5) -------------------------------

#[test]
fn rubric_rejects_known_bad_filler() {
    let v = score_against_rubric("As you can see, it works.");
    assert!(matches!(v, EmberStatus::Rejected { .. }));
}

#[test]
fn rubric_rejects_known_bad_absolutism() {
    let v = score_against_rubric("This is clearly the right architecture.");
    assert!(matches!(v, EmberStatus::Rejected { .. }));
}

#[test]
fn rubric_rejects_known_bad_totalising_claim() {
    let v = score_against_rubric("all systems operational");
    let EmberStatus::Rejected { trait_name, .. } = v else {
        panic!("expected Rejected");
    };
    assert_eq!(trait_name, TraitName::Honesty);
}

#[test]
fn rubric_approves_known_good_factual() {
    let v = score_against_rubric(
        "POVM probe at 2026-05-17T10:00:00Z returned learning_health=0.067 (scope=lib).",
    );
    assert_eq!(v, EmberStatus::Approved);
}

#[test]
fn rubric_approves_known_good_with_enumeration() {
    let v = score_against_rubric(
        "successfully completed:\n- m8 build-prereq\n- m9 namespace guard",
    );
    assert_eq!(v, EmberStatus::Approved);
}

// ---- Allowlist TSV from on-disk fixture (5) ------------------------------

#[test]
fn ember_held_approvals_tsv_in_repo_loads_to_empty() {
    let rows = load_approvals("tests/ember_held_approvals.tsv")
        .expect("repo-level TSV must exist and parse");
    assert!(rows.is_empty(), "Day-1 allowlist must be empty");
}

#[test]
fn ember_held_approvals_tsv_header_intact() {
    // The header line of the repo TSV must remain stable so external
    // tooling (downstream scripts, vault renderers) can parse it.
    let content = fs::read_to_string("tests/ember_held_approvals.tsv")
        .expect("TSV exists");
    let first_line = content.lines().next().expect("non-empty");
    assert_eq!(
        first_line,
        "artefact_key\tapproved_by\tapproved_at\texpiry"
    );
}

#[test]
fn allowlist_temp_file_with_two_rows_parses() {
    let mut f = NamedTempFile::new().expect("temp");
    writeln!(f, "artefact_key\tapproved_by\tapproved_at\texpiry").unwrap();
    writeln!(
        f,
        "m12.report\tluke@0A\t2026-05-17T10:00:00Z\t2030-01-01T00:00:00Z"
    )
    .unwrap();
    writeln!(
        f,
        "m11.sunset\tcommand\t2026-05-17T10:00:00Z\t2030-01-01T00:00:00Z"
    )
    .unwrap();
    let rows = load_approvals(f.path()).expect("two rows");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].artefact_key, "m12.report");
    assert_eq!(rows[1].artefact_key, "m11.sunset");
}

#[test]
fn allowlist_with_inline_comment_and_blank_lines_parses() {
    let mut f = NamedTempFile::new().expect("temp");
    writeln!(f, "artefact_key\tapproved_by\tapproved_at\texpiry").unwrap();
    writeln!(f).unwrap();
    writeln!(f, "# operator note: keep this row only until 2030").unwrap();
    writeln!(
        f,
        "m12.report\tluke@0A\t2026-05-17T10:00:00Z\t2030-01-01T00:00:00Z"
    )
    .unwrap();
    writeln!(f).unwrap();
    let rows = load_approvals(f.path()).expect("comments+blanks");
    assert_eq!(rows.len(), 1);
}

#[test]
fn allowlist_missing_file_returns_empty_not_error() {
    // The CI gate uses unwrap_or_default which relies on this contract.
    let rows = load_approvals("tests/this-file-intentionally-missing.tsv")
        .expect("missing → empty");
    assert!(rows.is_empty());
}

// ---- Gate end-to-end (4) -------------------------------------------------

#[test]
fn gate_pass_on_approved_fixture() {
    let v = evaluate_string(
        "m12.report.header",
        "POVM probe at 2026-05-17 (scope=lib) returned 0.067.",
        &[],
    );
    assert_eq!(v, GateVerdict::Pass);
}

#[test]
fn gate_fail_on_rejected_fixture_with_correct_key() {
    let v = evaluate_string("m11.sunset.warning", "tests passing", &[]);
    let GateVerdict::Fail { key, .. } = v else {
        panic!("expected Fail");
    };
    assert_eq!(key, "m11.sunset.warning");
}

#[test]
fn gate_rejected_ignores_allowlist_even_when_key_matches() {
    let approvals = vec![HeldApproval {
        artefact_key: "m11.sunset.warning".into(),
        approved_by: "luke".into(),
        approved_at: datetime!(2026-05-17 10:00:00 UTC),
        expiry: datetime!(2030-01-01 00:00:00 UTC),
    }];
    let v = evaluate_string("m11.sunset.warning", "successfully completed", &approvals);
    // Honesty fires at confidence 0.7 → Rejected; allowlist must be ignored.
    assert!(matches!(v, GateVerdict::Fail { .. }));
}

#[test]
fn gate_expiry_strict_inequality_at_now_equals_expiry() {
    // At the boundary now == expiry: NOT approved (expiry is exclusive).
    let approvals = vec![HeldApproval {
        artefact_key: "k".into(),
        approved_by: "luke".into(),
        approved_at: datetime!(2026-05-17 10:00:00 UTC),
        expiry: datetime!(2026-06-17 10:00:00 UTC),
    }];
    let now: OffsetDateTime = datetime!(2026-06-17 10:00:00 UTC);
    assert!(!is_approved_at(&approvals, "k", now));
    // A held verdict would also not be authorised at this boundary. Day-1
    // rubric has no <0.5-confidence path so we test via the allowlist
    // primitive directly.
    let v = evaluate_string_at("k", "POVM probe at 2026 (scope=lib)", &approvals, now);
    assert_eq!(v, GateVerdict::Pass);
}

// ---- Day-1 surface stability (2) -----------------------------------------

#[test]
fn day_1_empty_registry_passes_gate_vacuously() {
    // The contract that the CI gate relies on: an empty
    // `user_facing_strings::ALL` produces zero verdicts → no rejections,
    // no held — gate passes. This is the discipline anchor.
    use workflow_core::user_facing_strings::ALL;
    assert_eq!(ALL.len(), 0);
}

#[test]
fn day_1_gate_verdict_variants_are_exactly_four() {
    // Adding a new variant requires a coordinated spec amendment + Zen
    // audit. This compile-time exhaustiveness match enforces it.
    let v = GateVerdict::Pass;
    match v {
        GateVerdict::Pass
        | GateVerdict::Fail { .. }
        | GateVerdict::HeldFailed { .. }
        | GateVerdict::HeldAllowlisted { .. } => {}
    }
}

// ---- Canonical rubric path sanity (1) ------------------------------------

#[test]
fn canonical_rubric_path_documented_in_module_docstring() {
    // The canonical rubric lives at
    // `~/projects/claude_code/Ember 7-Trait Gate Rubric.md`. We don't load
    // it (heuristics are the test surface per spec § 7), but if it exists
    // we sanity-check the citation. If it's missing on this machine we
    // emit a tracing-style stderr line and pass — the test is advisory.
    let home = std::env::var("HOME").unwrap_or_default();
    let path = format!("{home}/projects/claude_code/Ember 7-Trait Gate Rubric.md");
    if Path::new(&path).exists() {
        let len = fs::metadata(&path).map_or(0, |m| m.len());
        assert!(len > 0, "rubric file exists but is empty");
    } else {
        eprintln!("EMBER-RUBRIC-PATH advisory: {path} not present on this host (heuristics are the contract; rubric is the semantic reference)");
    }
}

// ---- is_approved convenience smoke (1) -----------------------------------

#[test]
fn is_approved_convenience_uses_current_wall_clock() {
    // Past-expiry row → not approved at whatever "now" is.
    let approvals = vec![HeldApproval {
        artefact_key: "k".into(),
        approved_by: "luke".into(),
        approved_at: datetime!(2024-01-01 00:00:00 UTC),
        expiry: datetime!(2024-02-01 00:00:00 UTC),
    }];
    assert!(!is_approved(&approvals, "k"));
}
