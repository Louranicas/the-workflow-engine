//! Integration tests for the `wf-dispatch` orchestration layer.
//!
//! These exercise the lib↔binary seam directly: the binary `main()` is a
//! thin wrapper around `workflow_core::orchestration::dispatch::run`, so
//! calling `run` here against a temp-file proposals JSONL fixture in
//! `--dry-run` mode covers the m30→m31→m33 pipeline the binary drives —
//! without launching a subprocess and without contacting a live Conductor.
//!
//! The proposals JSONL fixture is built the same way `wf-crystallise`
//! produces it: real `WorkflowProposal`s composed via m20 `Pattern` → m21
//! `build_variants` → m23 `build_proposal`, then serde-serialised one per
//! line. This makes the bridge contract genuine, not mocked.

#![allow(clippy::doc_markdown)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tempfile::TempDir;

use workflow_core::m14_lift::LiftSnapshot;
use workflow_core::m20_prefixspan::{Pattern, StepToken};
use workflow_core::m21_variant_builder::build_variants;
use workflow_core::m23_proposer::{build_proposal, WorkflowProposal};
use workflow_core::m32_dispatcher::EscapeSurfaceProfile;
use workflow_core::orchestration::dispatch::{parse_args, run, ArgError, Config};

// ─── fixtures ───────────────────────────────────────────────────────────

/// A `LiftSnapshot` comfortably above the F2 evidence floor (20).
fn snapshot_above_f2() -> LiftSnapshot {
    LiftSnapshot {
        lift: Some(0.6),
        ci_half: Some(0.05),
        n: 40,
        latest_ts_ms: 0,
        computed_at: SystemTime::now(),
    }
}

/// Build one real `WorkflowProposal` keyed off `seed` — exactly the shape
/// `wf-crystallise` emits (m20 Pattern → m21 variant → m23 proposal).
fn proposal_with_seed(seed: u32) -> WorkflowProposal {
    let pattern = Pattern::new(
        vec![StepToken(seed), StepToken(seed.wrapping_add(1))],
        25,
        (0, seed as usize),
    );
    let variant = build_variants(&pattern).expect("m21 build_variants")[0].clone();
    build_proposal(variant, &snapshot_above_f2(), Some(seed as usize)).expect("m23 build_proposal")
}

/// Write `proposals` to `path` as one JSON object per line — the exact
/// JSONL bridge format `wf-crystallise` produces and `wf-dispatch` reads.
fn write_proposals_jsonl(path: &Path, proposals: &[WorkflowProposal]) {
    let mut file = fs::File::create(path).expect("create proposals fixture");
    for proposal in proposals {
        let line = serde_json::to_string(proposal).expect("serialise proposal");
        writeln!(file, "{line}").expect("write proposal line");
    }
}

/// A temp directory holding a proposals JSONL fixture.
struct Fixture {
    /// Keeps the temp directory alive for the test's lifetime.
    _dir: TempDir,
    /// Path to the seeded proposals JSONL.
    proposals_in: PathBuf,
}

impl Fixture {
    /// Build a fixture with `count` distinct proposals.
    fn with_proposals(count: u32) -> Self {
        let dir = TempDir::new().expect("temp dir");
        let proposals_in = dir.path().join("proposals.jsonl");
        let proposals: Vec<WorkflowProposal> =
            (0..count).map(|i| proposal_with_seed(i + 1)).collect();
        write_proposals_jsonl(&proposals_in, &proposals);
        Self {
            _dir: dir,
            proposals_in,
        }
    }

    /// A dry-run `Config` pointing at this fixture's proposals file.
    fn dry_run_config(&self) -> Config {
        Config {
            proposals_in: self.proposals_in.clone(),
            top_k: 5,
            conductor_url: "http://127.0.0.1:8141".to_owned(),
            dry_run: true,
            ack_ceiling: EscapeSurfaceProfile::Sandboxed,
            show_help: false,
            show_version: false,
        }
    }
}

// ─── pipeline tests ─────────────────────────────────────────────────────

#[test]
fn dispatch_dry_run_loads_banks_selects_and_verifies() {
    // rationale: Cross-module — the m30→m31→m33 chain runs to completion
    // in --dry-run mode against a real proposals JSONL fixture. Six
    // proposals are loaded, accepted into the bank, the top-5 selected,
    // and all five pass the conservative 4-verifier gate.
    let fx = Fixture::with_proposals(6);
    let report = run(&fx.dry_run_config()).expect("dry-run pipeline runs");

    assert!(report.completed, "pipeline must run end-to-end");
    assert!(report.dry_run, "report must record dry-run mode");
    assert_eq!(report.proposals_loaded, 6, "all 6 JSONL lines parsed");
    assert_eq!(report.bank_accepted, 6, "all 6 proposals banked");
    assert_eq!(report.candidates_selected, 5, "top-k = 5");
    assert_eq!(
        report.verifier_approved, 5,
        "the conservative gate approves every selected candidate"
    );
}

#[test]
fn dispatch_dry_run_never_dispatches() {
    // rationale: Anti-property — --dry-run (the default-safe mode) verifies
    // and selects but MUST NOT contact the Conductor; dispatched stays 0
    // and every candidate's disposition is `dry-run`.
    let fx = Fixture::with_proposals(4);
    let report = run(&fx.dry_run_config()).expect("dry-run");

    assert_eq!(report.dispatched, 0, "dry-run must not dispatch");
    assert!(!report.candidates.is_empty());
    for candidate in &report.candidates {
        assert_eq!(
            candidate.disposition, "dry-run",
            "every verified candidate's disposition is dry-run"
        );
        assert!(candidate.verifier_approved);
    }
}

#[test]
fn dispatch_respects_top_k() {
    // rationale: Boundary — with 10 proposals and --top-k 3, exactly 3
    // candidates are selected.
    let fx = Fixture::with_proposals(10);
    let mut config = fx.dry_run_config();
    config.top_k = 3;
    let report = run(&config).expect("dry-run top-k 3");
    assert_eq!(report.bank_accepted, 10);
    assert_eq!(report.candidates_selected, 3, "top-k caps selection");
}

#[test]
fn dispatch_empty_proposals_file_completes_with_zero_counts() {
    // rationale: Boundary — an empty (but present) proposals file is a
    // valid no-op run, not a fault: zero loaded, zero selected, completed.
    let dir = TempDir::new().expect("temp dir");
    let empty = dir.path().join("empty.jsonl");
    fs::File::create(&empty).expect("create empty");
    let mut config = Fixture::with_proposals(1).dry_run_config();
    config.proposals_in = empty;
    let report = run(&config).expect("empty file is a valid run");
    assert!(report.completed);
    assert_eq!(report.proposals_loaded, 0);
    assert_eq!(report.candidates_selected, 0);
}

#[test]
fn dispatch_blank_lines_are_skipped() {
    // rationale: Boundary — blank lines in the JSONL are tolerated and
    // skipped; only the two real proposal lines are loaded.
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("gappy.jsonl");
    let p1 = serde_json::to_string(&proposal_with_seed(1)).expect("ser 1");
    let p2 = serde_json::to_string(&proposal_with_seed(2)).expect("ser 2");
    fs::write(&path, format!("{p1}\n\n   \n{p2}\n")).expect("write gappy");
    let mut config = Fixture::with_proposals(1).dry_run_config();
    config.proposals_in = path;
    let report = run(&config).expect("gappy file parses");
    assert_eq!(report.proposals_loaded, 2, "blank lines must be skipped");
}

#[test]
fn dispatch_missing_proposals_file_is_a_typed_fault() {
    // rationale: Anti-property — a missing proposals file is a real
    // typed OrchestrationError, not a silent zero-proposal run.
    let dir = TempDir::new().expect("temp dir");
    let mut config = Fixture::with_proposals(1).dry_run_config();
    config.proposals_in = dir.path().join("nonexistent.jsonl");
    let err = run(&config).expect_err("missing file must fault");
    assert!(
        err.to_string().contains("proposals input"),
        "fault must name the proposals input: {err}"
    );
}

#[test]
fn dispatch_malformed_jsonl_line_is_a_typed_fault() {
    // rationale: Anti-property — a non-JSON line is a typed parse fault
    // carrying the 1-based line number.
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("bad.jsonl");
    let good = serde_json::to_string(&proposal_with_seed(1)).expect("ser");
    fs::write(&path, format!("{good}\nnot json at all\n")).expect("write bad");
    let mut config = Fixture::with_proposals(1).dry_run_config();
    config.proposals_in = path;
    let err = run(&config).expect_err("malformed line must fault");
    let msg = err.to_string();
    assert!(msg.contains("line 2"), "fault must name line 2: {msg}");
}

#[test]
fn dispatch_report_serialises_to_json() {
    // rationale: Contract — the Report derives Serialize; the JSON form
    // carries the documented fields.
    let fx = Fixture::with_proposals(3);
    let report = run(&fx.dry_run_config()).expect("dry-run");
    let json = serde_json::to_string(&report).expect("serialise report");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse json");
    assert!(value.get("proposals_loaded").is_some());
    assert!(value.get("candidates_selected").is_some());
    assert!(value.get("dry_run").is_some());
    assert_eq!(value["dry_run"], serde_json::Value::Bool(true));
}

#[test]
fn dispatch_jsonl_bridge_round_trips_from_crystallise_format() {
    // rationale: Cross-module — the JSONL bridge contract. A proposal
    // serialised exactly as wf-crystallise writes it must re-parse and
    // drive the dispatch pipeline. This is the load-bearing seam between
    // the two binaries.
    let fx = Fixture::with_proposals(2);
    // Re-read the fixture file the way wf-dispatch's loader does and
    // confirm each line is a valid WorkflowProposal before the run.
    let contents = fs::read_to_string(&fx.proposals_in).expect("read fixture");
    for line in contents.lines().filter(|l| !l.trim().is_empty()) {
        let _: WorkflowProposal =
            serde_json::from_str(line).expect("each line is a WorkflowProposal");
    }
    let report = run(&fx.dry_run_config()).expect("bridge run");
    assert_eq!(report.proposals_loaded, 2);
}

// ─── parse_args tests ───────────────────────────────────────────────────

fn args(raw: &[&str]) -> Vec<String> {
    raw.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn dispatch_parse_args_defaults_and_full_set() {
    // rationale: Boundary — empty args ⇒ dry-run default; the full flag
    // set parses into the matching Config fields.
    let defaults = parse_args(&[]).expect("empty");
    assert!(defaults.dry_run, "dry-run is the default-safe mode");
    assert_eq!(defaults.top_k, 5);
    assert_eq!(defaults.ack_ceiling, EscapeSurfaceProfile::Sandboxed);

    let full = parse_args(&args(&[
        "--proposals-in",
        "/tmp/p.jsonl",
        "--top-k",
        "2",
        "--ack-ceiling",
        "network-egress",
        "--execute",
    ]))
    .expect("full");
    assert_eq!(full.proposals_in, PathBuf::from("/tmp/p.jsonl"));
    assert_eq!(full.top_k, 2);
    assert_eq!(full.ack_ceiling, EscapeSurfaceProfile::NetworkEgress);
    assert!(!full.dry_run, "--execute flips off the dry-run default");
}

#[test]
fn dispatch_parse_args_help_version_and_unknown_flag() {
    // rationale: Boundary + anti-property — --help/--version set their
    // flags; an unknown flag is a typed ArgError.
    assert!(parse_args(&args(&["--help"])).expect("help").show_help);
    assert!(parse_args(&args(&["--version"]))
        .expect("version")
        .show_version);
    let err = parse_args(&args(&["--bogus"])).expect_err("unknown");
    assert_eq!(err, ArgError::UnknownFlag("--bogus".to_owned()));
}

#[test]
fn dispatch_parse_args_missing_value_is_typed_error() {
    // rationale: Anti-property — a value-bearing flag with no value.
    let err = parse_args(&args(&["--conductor-url"])).expect_err("missing");
    assert_eq!(err, ArgError::MissingValue("--conductor-url"));
}

// rationale: Phase 8 step 2 / gap NA-4 — Conductor enforcement-state
// assertion. When `CONDUCTOR_ENFORCEMENT_ENABLED` is unset OR not "1",
// the report MUST flag `conductor_enforcement_advisory: true` so the
// operator knows m33 Approve verdicts are advisory until the Conductor
// itself enforces them. Run with the env var explicitly unset.
//
// NOTE: env vars are process-global; this test uses
// `std::env::remove_var` which can race with parallel tests. Workflow-
// trace's other tests do not touch CONDUCTOR_ENFORCEMENT_ENABLED, so no
// pollution is expected. Hold guard via single-threaded asserts only.
// rationale: Phase 8 step 2 / gap NA-4 — Conductor enforcement-state
// assertion. The flag CONDUCTOR_ENFORCEMENT_ENABLED governs whether
// m33 verdicts are advisory or enforced at the dispatcher; only the
// exact value "1" silences the report's `conductor_enforcement_advisory`
// flag. Combined into ONE test (rather than two) to keep the env-var
// mutations strictly sequential — `cargo test` parallelises tests by
// default, so two parallel tests touching the same process-global env
// var would race. The three sub-cases exercise the documented contract:
// unset, "0", and "1".
//
// SAFETY: `std::env::set_var` / `remove_var` are process-global; this is
// the only test in workflow-trace that touches CONDUCTOR_ENFORCEMENT_ENABLED.
#[test]
fn dispatch_conductor_enforcement_advisory_three_branches() {
    let fx = Fixture::with_proposals(1);

    // Branch 1: env unset → advisory=true (NA-4 fold).
    unsafe {
        std::env::remove_var("CONDUCTOR_ENFORCEMENT_ENABLED");
    }
    let r = run(&fx.dry_run_config()).expect("dry-run completes (env unset)");
    assert!(
        r.conductor_enforcement_advisory,
        "env unset → conductor_enforcement_advisory must be true"
    );

    // Branch 2: env = "0" → advisory=true.
    unsafe {
        std::env::set_var("CONDUCTOR_ENFORCEMENT_ENABLED", "0");
    }
    let r = run(&fx.dry_run_config()).expect("dry-run completes (env 0)");
    assert!(
        r.conductor_enforcement_advisory,
        "env=\"0\" → conductor_enforcement_advisory must be true"
    );

    // Branch 3: env = "1" → advisory=false (silent pass-through).
    unsafe {
        std::env::set_var("CONDUCTOR_ENFORCEMENT_ENABLED", "1");
    }
    let r = run(&fx.dry_run_config()).expect("dry-run completes (env 1)");
    assert!(
        !r.conductor_enforcement_advisory,
        "env=\"1\" → conductor_enforcement_advisory must be false"
    );

    // Restore default for any later parallel test that might (incorrectly)
    // assume the env is unset.
    unsafe {
        std::env::remove_var("CONDUCTOR_ENFORCEMENT_ENABLED");
    }
}
