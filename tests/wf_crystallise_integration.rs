//! Integration tests for the `wf-crystallise` orchestration layer.
//!
//! These exercise the lib↔binary seam directly: the binary `main()` is a
//! thin wrapper around `workflow_core::orchestration::crystallise::run`,
//! so calling `run` here against temp-file SQLite fixtures in `--offline`
//! mode covers the same pipeline the binary drives — without launching a
//! subprocess.
//!
//! Fixtures are tiny SQLite databases shaped exactly like the live atuin
//! `history` table and the habitat `injection.db` `causal_chain` table
//! (schemas mirrored from `src/m1_atuin_consumer/row.rs` and
//! `tests/m3_integration.rs`).

#![allow(clippy::doc_markdown)]

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use tempfile::TempDir;

use workflow_core::m23_proposer::WorkflowProposal;
use workflow_core::orchestration::crystallise::{
    parse_args, run, ArgError, Config, ReportFormat,
};

// ─── fixtures ───────────────────────────────────────────────────────────

/// Lexicographically-sortable synthetic ULID for the atuin `id` column.
fn synthetic_ulid(i: i64) -> String {
    format!("01HQA{i:021}")
}

/// Build a tiny atuin-history-shaped SQLite DB at `path`.
///
/// Schema mirrors the live atuin `history` table (see
/// `src/m1_atuin_consumer/row.rs` § Live-schema discovery). The `command`
/// values are deliberately repeated across three sessions so the m20
/// PrefixSpan miner has a recurring sub-sequence to find.
fn seed_atuin_db(path: &Path) {
    let conn = Connection::open(path).expect("open atuin fixture");
    conn.execute_batch(
        "CREATE TABLE history (
            id TEXT PRIMARY KEY,
            timestamp INTEGER NOT NULL,
            duration INTEGER NOT NULL,
            exit INTEGER NOT NULL,
            command TEXT NOT NULL,
            cwd TEXT NOT NULL,
            session TEXT NOT NULL,
            hostname TEXT NOT NULL,
            deleted_at INTEGER
        );",
    )
    .expect("atuin schema");
    // 6 sessions × the same 4-command sequence → a strong recurring
    // pattern well above the default min_support of 3.
    let sequence = ["git status", "cargo check", "cargo test", "git commit"];
    let mut row = 1_i64;
    for session in 0..6 {
        for (step, cmd) in sequence.iter().enumerate() {
            conn.execute(
                "INSERT INTO history \
                 (id, command, session, hostname, timestamp, exit, duration, cwd, deleted_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    synthetic_ulid(row),
                    cmd,
                    format!("session-{session}"),
                    "host1",
                    1_700_000_000_000_i64
                        + row * 1_000
                        + i64::try_from(step).expect("step index fits i64"),
                    0_i32,
                    25_i64,
                    "/home/dev/project",
                    Option::<i64>::None,
                ],
            )
            .expect("atuin insert");
            row += 1;
        }
    }
}

/// Build a tiny injection.db-shaped SQLite DB at `path`.
///
/// Schema mirrors the live habitat `causal_chain` table (see
/// `tests/m3_integration.rs`). Seeds a mix of unresolved and resolved
/// chains plus one `Forget`-consent row that m3 must filter out.
fn seed_injection_db(path: &Path) {
    let conn = Connection::open(path).expect("open injection fixture");
    conn.execute_batch(
        "CREATE TABLE causal_chain (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            origin_session INTEGER NOT NULL,
            resolved_session INTEGER,
            chain_type TEXT NOT NULL CHECK(chain_type IN ('bug','trap','plan','pattern')),
            label TEXT NOT NULL,
            description TEXT NOT NULL,
            reinforcement_count INTEGER NOT NULL DEFAULT 1,
            last_reinforced_session INTEGER,
            consent TEXT NOT NULL DEFAULT 'Emit' CHECK(consent IN ('Emit','Store','Forget')),
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
        );",
    )
    .expect("injection schema");
    // 3 unresolved Emit chains + 1 unresolved Forget chain (must be
    // filtered) + 1 resolved chain.
    let rows = [
        (1_i64, None::<i64>, "bug", "CHAIN-001", 9, "Emit"),
        (2, None, "trap", "CHAIN-002", 7, "Emit"),
        (3, None, "plan", "CHAIN-003", 5, "Emit"),
        (4, None, "pattern", "CHAIN-004", 4, "Forget"),
        (5, Some(120), "plan", "CHAIN-005", 1, "Emit"),
    ];
    for (id, resolved, ctype, label, reinf, consent) in rows {
        conn.execute(
            "INSERT INTO causal_chain \
             (id, origin_session, resolved_session, chain_type, label, description, \
              reinforcement_count, last_reinforced_session, consent) \
             VALUES (?1, 100, ?2, ?3, ?4, ?5, ?6, 100, ?7)",
            rusqlite::params![id, resolved, ctype, label, "desc", reinf, consent],
        )
        .expect("injection insert");
    }
}

/// Assemble an `(atuin.db, injection.db, runs.db, proposals.jsonl)`
/// fixture set inside a fresh temp directory.
struct Fixture {
    /// Keeps the temp directory alive for the test's lifetime.
    dir: TempDir,
    /// Path to the seeded atuin history DB.
    atuin_db: PathBuf,
    /// Path to the seeded injection.db.
    injection_db: PathBuf,
    /// Path to the (not-yet-created) workflow-runs DB.
    runs_db: PathBuf,
    /// Path to the (not-yet-created) proposals JSONL output.
    proposals_out: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = TempDir::new().expect("temp dir");
        let atuin_db = dir.path().join("atuin.db");
        let injection_db = dir.path().join("injection.db");
        let runs_db = dir.path().join("workflow_runs.db");
        let proposals_out = dir.path().join("proposals.jsonl");
        seed_atuin_db(&atuin_db);
        seed_injection_db(&injection_db);
        Self {
            dir,
            atuin_db,
            injection_db,
            runs_db,
            proposals_out,
        }
    }

    /// An offline `Config` pointing at this fixture's files.
    fn offline_config(&self) -> Config {
        Config {
            atuin_db: self.atuin_db.clone(),
            injection_db: self.injection_db.clone(),
            runs_db: self.runs_db.clone(),
            proposals_out: self.proposals_out.clone(),
            min_support: 3,
            max_gap: 5,
            offline: true,
            format: ReportFormat::Text,
            show_help: false,
            show_version: false,
        }
    }
}

// ─── pipeline tests ─────────────────────────────────────────────────────

#[test]
fn crystallise_offline_run_completes_and_counts_substrate_inputs() {
    // rationale: Cross-module — the full m1→m3→m4→m7→m14→m20→m23 pipeline
    // runs to completion in --offline mode against real SQLite fixtures.
    // The atuin fixture has 24 rows; the injection fixture has 3 emitting
    // unresolved chains (the 4th is Forget-filtered, the 5th is resolved).
    let fx = Fixture::new();
    let report = run(&fx.offline_config()).expect("offline pipeline runs");

    assert!(report.completed, "pipeline must run end-to-end");
    assert_eq!(report.atuin_rows, 24, "6 sessions × 4 commands");
    assert_eq!(
        report.injection_chains, 3,
        "Forget-consent + resolved chains must be excluded"
    );
    assert!(report.run_id > 0, "m7 must assign a positive run id");
    // 3 emitting chains were merged as InjectionChain observations; cascade
    // observations depend on m4's correlation but the total must be ≥ 3.
    assert!(
        report.observations_merged >= 3,
        "at least the 3 injection chains were merged"
    );
    // Phase 8 step 3 / gap NA-2 — m1 read latency is captured as the
    // engine-timed proxy for substrate-side load (per § 15 D37). The
    // tiny test fixture (24 atuin rows) reads in milliseconds; the
    // observation MUST be populated (`<` MAX) and the perturbation flag
    // MUST be false (24 rows << 500 ms threshold).
    assert!(
        report.m1_read_latency_ms < u64::MAX,
        "m1_read_latency_ms must be populated (got u64::MAX → record default)"
    );
    assert!(
        !report.m1_read_perturbation_observed,
        "the test fixture is too small to exceed the perturbation threshold"
    );
}

#[test]
fn crystallise_offline_skips_live_stages() {
    // rationale: Anti-property — --offline must skip every live-service
    // stage (m2 stcortex, m40 nexus emit) and record them as skipped,
    // never attempt them.
    let fx = Fixture::new();
    let report = run(&fx.offline_config()).expect("offline run");

    assert!(
        report.stages_skipped.contains(&"m2-stcortex".to_owned()),
        "m2 stcortex must be skipped offline"
    );
    assert!(
        report.stages_skipped.contains(&"m40-nexus-emit".to_owned()),
        "m40 nexus emit must be skipped offline"
    );
}

#[test]
fn crystallise_writes_proposals_jsonl_that_reparses() {
    // rationale: Cross-module — the JSONL bridge is the contract between
    // wf-crystallise and wf-dispatch. Every written line must re-parse as
    // a WorkflowProposal (the exact type wf-dispatch reads back).
    let fx = Fixture::new();
    let report = run(&fx.offline_config()).expect("offline run");

    assert!(
        fx.proposals_out.exists(),
        "the proposals JSONL file must be created"
    );
    let contents = fs::read_to_string(&fx.proposals_out).expect("read proposals");
    let line_count = contents.lines().filter(|l| !l.trim().is_empty()).count();
    assert_eq!(
        line_count, report.proposals_written,
        "report count must match the JSONL line count"
    );
    // Every non-empty line must round-trip through serde as a real
    // WorkflowProposal — no assertion theatre, the actual deserialise.
    for (idx, line) in contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let parsed: WorkflowProposal = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("proposals line {} must re-parse: {e}", idx + 1));
        // A proposal carries a non-zero opaque id and an evidence_n at or
        // above the F2 floor (20) — m23 refuses anything below.
        assert!(parsed.evidence_n() >= 20, "F2 floor must hold on disk");
    }
}

// rationale: v0.1.1 R3 — m22 K-means CLI batch-path distribution test.
// The Phase 5 test below proves that ≥ 1 proposal carries Some(cluster);
// R3 strengthens the contract to: across a multi-proposal batch on the
// seeded fixture, the kmeans output distributes proposals across
// MULTIPLE distinct cluster indices (≥ 2 when the recommended `k`
// permits). This locks the "real kmeans is running, not collapsing to
// one cluster" invariant per § 15 D17–D20 (5-dim feature vector +
// adaptive k + diversity_cluster emitted).
//
// Honest fallback: if the fixture happens to yield a variant count
// where `recommended_k_for_variant_count(n) == 1`, this test accepts
// 1 distinct cluster (the degenerate-but-correct k=1 path). The
// fixture in practice produces enough variants for k ≥ 2.
#[test]
fn crystallise_proposals_distribute_across_multiple_m22_clusters() {
    use std::collections::HashSet;
    use workflow_core::m7_workflow_runs::{insert_run, open_database};

    let fx = Fixture::new();
    {
        let conn = open_database(&fx.runs_db).expect("open runs db");
        for i in 0..25 {
            insert_run(&conn, &format!("seed-{i}")).expect("seed run");
        }
    }
    let report = run(&fx.offline_config()).expect("offline run");
    assert!(
        report.proposals_written >= 4,
        "R3 needs ≥4 proposals to assert distribution; got {}",
        report.proposals_written
    );
    let contents = fs::read_to_string(&fx.proposals_out).expect("read proposals");
    let proposals: Vec<WorkflowProposal> = contents
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("proposal reparse"))
        .collect();
    let clusters: HashSet<usize> = proposals
        .iter()
        .filter_map(workflow_core::m23_proposer::WorkflowProposal::diversity_cluster)
        .collect();
    let recommended_k =
        workflow_core::m22_kmeans::recommended_k_for_variant_count(proposals.len());
    // If recommended_k allows ≥2 buckets AND we have ≥4 proposals, we
    // expect ≥2 distinct clusters. Otherwise the degenerate k=1 case
    // is the honest correct answer.
    if recommended_k >= 2 {
        assert!(
            clusters.len() >= 2,
            "recommended_k={recommended_k} on {} proposals — kmeans must distribute \
             across ≥2 cluster indices, got {} distinct: {:?}",
            proposals.len(),
            clusters.len(),
            clusters,
        );
    } else {
        assert_eq!(
            clusters.len(),
            1,
            "recommended_k=1 — exactly one cluster index expected, got {clusters:?}"
        );
    }
}

#[test]
fn crystallise_proposals_carry_non_none_m22_diversity_cluster() {
    // rationale: R2 (Plan v2 §15 D17–D20) — verifies the m22 K-means wiring
    // on the `wf-crystallise` CLI path. Prior to Phase 5 (commit 97bb331..)
    // `compose_proposals(&patterns, &snapshot, |_v| None)` hard-coded every
    // proposal's `diversity_cluster` to `None`; the m22 signal never
    // reached a proposal. The Phase 5 wiring pre-builds every variant,
    // extracts 5-dim features (D17), runs K-means with adaptive `k`
    // (D19), and threads the result via a real lookup closure — so at
    // least one proposal on a fixture whose evidence_n clears the F2
    // floor (PROPOSAL_F2_THRESHOLD = 20) must carry `Some(cluster)`
    // (D20: emitted via the JSONL bridge to any downstream consumer —
    // `m31_selector`/`wf-dispatch` included).
    //
    // Pattern mirrors `crystallise_lift_window_grows_with_prior_runs_in_
    // persistent_db`: pre-seed 25 prior open runs into the persistent
    // runs DB so the m14 lift snapshot's `n` exceeds F2 and m23 admits
    // proposals (rather than refusing every one as below-threshold).
    use workflow_core::m7_workflow_runs::{insert_run, open_database};

    let fx = Fixture::new();
    {
        let conn = open_database(&fx.runs_db).expect("open runs db");
        for i in 0..25 {
            insert_run(&conn, &format!("seed-{i}")).expect("seed run");
        }
    }
    let report = run(&fx.offline_config()).expect("offline run");
    assert!(
        report.proposals_written > 0,
        "with 25 seeded runs evidence_n must clear F2 and produce proposals; got {}",
        report.proposals_written
    );
    let contents = fs::read_to_string(&fx.proposals_out).expect("read proposals");
    let proposals: Vec<WorkflowProposal> = contents
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("proposal reparse"))
        .collect();
    assert!(!proposals.is_empty(), "fixture must yield ≥ 1 proposal");
    let with_cluster: Vec<_> = proposals
        .iter()
        .filter_map(workflow_core::m23_proposer::WorkflowProposal::diversity_cluster)
        .collect();
    assert!(
        !with_cluster.is_empty(),
        "Phase 5 wiring must produce ≥ 1 proposal with `Some(diversity_cluster)`; \
         got {} proposals all with None — the m22 K-means signal regressed",
        proposals.len()
    );
    // Cluster indices land in [0, k) where k = recommended_k_for_variant_count
    // applied to the variant count. We don't pin the exact `k` (it varies
    // with fixture mining results) but every cluster index must be
    // < RECOMMENDED_K_MAX = 8.
    for &c in &with_cluster {
        assert!(
            c < workflow_core::RECOMMENDED_K_MAX,
            "cluster index {c} must be < RECOMMENDED_K_MAX (8)"
        );
    }
}

#[test]
fn crystallise_creates_runs_db_when_absent() {
    // rationale: Boundary — --runs-db is created if absent; a fresh temp
    // path with no pre-existing file must end the run with a real SQLite
    // database on disk.
    let fx = Fixture::new();
    assert!(!fx.runs_db.exists(), "precondition: runs DB absent");
    let _ = run(&fx.offline_config()).expect("offline run");
    assert!(
        fx.runs_db.exists(),
        "wf-crystallise must create the workflow-runs DB"
    );
}

#[test]
fn crystallise_run_is_deterministic_across_invocations() {
    // rationale: Determinism — two runs over the same fixture yield the
    // same pattern + proposal counts (the m20/m21/m23 chain is pure).
    let fx_a = Fixture::new();
    let fx_b = Fixture::new();
    let report_a = run(&fx_a.offline_config()).expect("run a");
    let report_b = run(&fx_b.offline_config()).expect("run b");
    assert_eq!(report_a.atuin_rows, report_b.atuin_rows);
    assert_eq!(report_a.patterns_mined, report_b.patterns_mined);
    assert_eq!(report_a.proposals_written, report_b.proposals_written);
}

#[test]
fn crystallise_missing_atuin_db_is_a_typed_fault() {
    // rationale: Anti-property — a missing atuin DB is a real pipeline
    // fault (typed OrchestrationError), not a silent empty run.
    let fx = Fixture::new();
    let mut config = fx.offline_config();
    config.atuin_db = fx.dir.path().join("does-not-exist.db");
    let err = run(&config).expect_err("missing atuin DB must fault");
    let msg = err.to_string();
    assert!(
        msg.contains("atuin"),
        "fault must name the atuin stage: {msg}"
    );
}

#[test]
fn crystallise_lift_window_grows_with_prior_runs_in_persistent_db() {
    // rationale: Cross-module — the lift window is computed over the open
    // runs in a persistent --runs-db. Pre-seeding the runs DB with prior
    // open runs proves evidence accumulates across invocations: the lift
    // window must include both the seeded runs and the current run.
    use workflow_core::m7_workflow_runs::{insert_run, open_database};

    let fx = Fixture::new();
    // Pre-seed 25 prior open runs into the runs DB (above the F2 floor).
    {
        let conn = open_database(&fx.runs_db).expect("open runs db");
        for i in 0..25 {
            insert_run(&conn, &format!("seed-{i}")).expect("seed run");
        }
    }
    let report = run(&fx.offline_config()).expect("offline run");
    // The window saw the 25 seeded open runs plus the current run.
    assert!(
        report.lift_window_runs >= 25,
        "lift window must include the seeded prior runs, got {}",
        report.lift_window_runs
    );
    assert!(report.completed);
}

#[test]
fn crystallise_report_serialises_to_json() {
    // rationale: Contract — the binary's --format json path serialises the
    // Report; the derived Serialize impl must produce a JSON object with
    // the documented fields.
    let fx = Fixture::new();
    let report = run(&fx.offline_config()).expect("offline run");
    let json = serde_json::to_string(&report).expect("report serialises");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json parses");
    assert!(value.get("atuin_rows").is_some());
    assert!(value.get("proposals_written").is_some());
    assert!(value.get("completed").is_some());
    assert_eq!(value["completed"], serde_json::Value::Bool(true));
}

// ─── parse_args tests ───────────────────────────────────────────────────

fn args(raw: &[&str]) -> Vec<String> {
    raw.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn crystallise_parse_args_defaults_and_full_set() {
    // rationale: Boundary — empty args ⇒ defaults; the full flag set
    // parses into the matching Config fields.
    let defaults = parse_args(&[]).expect("empty");
    assert!(!defaults.offline);
    assert_eq!(defaults.min_support, 3);

    let full = parse_args(&args(&[
        "--atuin-db",
        "/tmp/a.db",
        "--min-support",
        "8",
        "--offline",
        "--format",
        "json",
    ]))
    .expect("full");
    assert_eq!(full.atuin_db, PathBuf::from("/tmp/a.db"));
    assert_eq!(full.min_support, 8);
    assert!(full.offline);
    assert_eq!(full.format, ReportFormat::Json);
}

#[test]
fn crystallise_parse_args_help_version_and_unknown_flag() {
    // rationale: Boundary + anti-property — --help/--version set their
    // flags; an unknown flag is a typed ArgError.
    assert!(parse_args(&args(&["--help"])).expect("help").show_help);
    assert!(parse_args(&args(&["--version"]))
        .expect("version")
        .show_version);
    let err = parse_args(&args(&["--unknown-flag"])).expect_err("unknown");
    assert_eq!(err, ArgError::UnknownFlag("--unknown-flag".to_owned()));
}
