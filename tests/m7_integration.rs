//! Integration tests for m7 workflow_runs (Wave-C2).
//!
//! Exercises the m7 public surface from outside the crate:
//!
//! - F9 zero-weight — open run keeps `ended_at = None` and `outcome = None`.
//! - F9 zero-weight — explicit `cost_tokens = 0` is distinguished from
//!   the bare `None` signal.
//! - Contract regression — `close_run` is idempotent on an already-closed
//!   run (last write wins via UPDATE).
//! - Cross-module surface invariant — `Outcome` wire set matches the
//!   SQLite `CHECK` constraint exactly.
//! - Boundary — opening a fresh on-disk database creates the table when
//!   missing.
//! - Contract regression — `WorkflowRunRow` serde round-trip via
//!   `find_by_id` preserves all fields including `fitness_dimension`.
//! - Concurrency — concurrent `insert_run` across N threads serialises
//!   correctly under SQLite WAL.
//! - Boundary — `find_by_outcome` filters correctly across all 3 closed
//!   outcomes (`ok`, `fail`, `abort`).

#![allow(clippy::doc_markdown)]

use std::sync::Arc;
use std::thread;

use rusqlite::Connection;
use tempfile::tempdir;
use workflow_core::m7_workflow_runs::{
    close_run, find_by_id, find_by_outcome, find_open, insert_run, merge_observation,
    open_database, open_memory, update_cost_tokens, ClusterBObservation, Outcome, RunState,
    WorkflowRunRow,
};

fn mem() -> Connection {
    open_memory().expect("memory open")
}

// rationale: Anti-property F9 zero-weight — an open workflow run must
// preserve `ended_at = None` and `outcome = None` even after intermediate
// merges and cost updates; neither must be fabricated by the public API.
#[test]
fn m7_open_run_keeps_ended_at_and_outcome_none() {
    let conn = mem();
    let id = insert_run(&conn, "2026-05-20T00:00:00Z").expect("insert");
    merge_observation(
        &conn,
        id,
        &ClusterBObservation::BatternStep {
            battern_id: "battern_test_1".into(),
            step_index: 0,
            duration_ms: 25,
            outcome: "ok".into(),
        },
    )
    .expect("merge");
    update_cost_tokens(&conn, id, 750).expect("cost");
    let row = find_by_id(&conn, id).expect("find");
    assert_eq!(row.run_state, RunState::Open, "F9: open run stays Open");
    assert!(
        row.run_state.ended_at().is_none(),
        "F9: ended_at must stay None on open run"
    );
    assert!(
        row.run_state.outcome().is_none(),
        "F9: outcome must stay None on open run"
    );
    assert_eq!(row.cost_tokens, Some(750));
}

// rationale: Anti-property F9 zero-weight — `cost_tokens` distinguishes
// `None` (no signal) from `Some(0)` (explicit zero signal). The public
// API must never collapse one to the other.
#[test]
fn m7_cost_tokens_none_distinguishes_from_signal_zero() {
    let conn = mem();
    let id_none = insert_run(&conn, "2026-05-20T00:00:00Z").expect("ins");
    let id_zero = insert_run(&conn, "2026-05-20T00:00:01Z").expect("ins");
    update_cost_tokens(&conn, id_zero, 0).expect("explicit 0");
    let row_none = find_by_id(&conn, id_none).expect("find none");
    let row_zero = find_by_id(&conn, id_zero).expect("find zero");
    assert_eq!(row_none.cost_tokens, None);
    assert_eq!(row_zero.cost_tokens, Some(0));
    assert_ne!(row_none.cost_tokens, row_zero.cost_tokens);
}

// rationale: Contract regression — `close_run` is idempotent on an
// already-closed run. Calling it again updates `ended_at` + `outcome`
// (last write wins via UPDATE). No error, no spurious row.
#[test]
fn m7_close_run_is_idempotent_on_already_closed_run() {
    let conn = mem();
    let id = insert_run(&conn, "2026-05-20T00:00:00Z").expect("ins");
    close_run(&conn, id, "2026-05-20T01:00:00Z", "ok").expect("first close");
    close_run(&conn, id, "2026-05-20T02:00:00Z", "fail").expect("second close");
    let row = find_by_id(&conn, id).expect("find");
    assert_eq!(row.run_state.outcome(), Some(Outcome::Fail));
    assert_eq!(row.run_state.ended_at(), Some("2026-05-20T02:00:00Z"));
    // Only ONE row exists (idempotent UPDATE, not duplicate INSERT).
    let n: i64 = conn
        .query_row("SELECT COUNT(*) FROM workflow_runs", [], |r| r.get(0))
        .expect("count");
    assert_eq!(n, 1);
}

// rationale: Cross-module surface invariant — the `Outcome` wire set
// must match the SQLite `CHECK` constraint exactly. Drift detection
// for future variant additions: cardinality is locked at 4.
#[test]
fn m7_outcome_wire_set_matches_sqlite_check_constraint() {
    let conn = mem();
    let id = insert_run(&conn, "2026-05-20T00:00:00Z").expect("ins");
    // All 4 variants round-trip via close_run + the SQLite CHECK.
    for wire in ["ok", "fail", "abort", "unknown"] {
        close_run(&conn, id, "2026-05-20T01:00:00Z", wire).expect("CHECK passes");
        let row = find_by_id(&conn, id).expect("find");
        assert_eq!(row.run_state.outcome(), Some(Outcome::parse(wire).expect("wire")));
    }
    // An invalid wire string is rejected before the DB hit.
    assert!(close_run(&conn, id, "2026-05-20T01:00:00Z", "WEIRD").is_err());
    // Direct cardinality lock — set has 4 distinct entries.
    let set: std::collections::HashSet<&str> = [
        Outcome::Ok.as_str(),
        Outcome::Fail.as_str(),
        Outcome::Abort.as_str(),
        Outcome::Unknown.as_str(),
    ]
    .into_iter()
    .collect();
    assert_eq!(set.len(), 4);
}

// rationale: Boundary — `open_database` on a fresh path must create
// the table from the embedded DDL (no migration scripts required).
#[test]
fn m7_open_database_creates_table_if_missing() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("fresh.db");
    assert!(!path.exists(), "precondition: file must not exist");
    let conn = open_database(&path).expect("open");
    // Table must exist + be empty.
    let n: i64 = conn
        .query_row("SELECT COUNT(*) FROM workflow_runs", [], |r| r.get(0))
        .expect("count");
    assert_eq!(n, 0);
    // Re-opening the same path is idempotent.
    drop(conn);
    let conn2 = open_database(&path).expect("reopen");
    let n2: i64 = conn2
        .query_row("SELECT COUNT(*) FROM workflow_runs", [], |r| r.get(0))
        .expect("count");
    assert_eq!(n2, 0);
}

// rationale: Contract regression — `WorkflowRunRow` serde round-trip
// via `find_by_id` preserves all fields (including the F9 zero-weight
// `fitness_dimension`).
#[test]
fn m7_serde_round_trip_via_find_by_id() {
    let conn = mem();
    let id = insert_run(&conn, "2026-05-20T00:00:00Z").expect("ins");
    update_cost_tokens(&conn, id, 1234).expect("cost");
    close_run(&conn, id, "2026-05-20T01:00:00Z", "fail").expect("close");
    let row_a = find_by_id(&conn, id).expect("find");
    let json = serde_json::to_string(&row_a).expect("serialise");
    let row_b: WorkflowRunRow = serde_json::from_str(&json).expect("deserialise");
    assert_eq!(row_a.id, row_b.id);
    assert_eq!(row_a.started_at, row_b.started_at);
    assert_eq!(row_a.run_state, row_b.run_state);
    assert_eq!(row_a.consumer_inputs, row_b.consumer_inputs);
    assert_eq!(row_a.cost_tokens, row_b.cost_tokens);
    assert!((row_a.fitness_dimension - row_b.fitness_dimension).abs() < f64::EPSILON);
    // F9: fitness_dimension stays at its DDL zero default through the trip.
    assert!(row_b.fitness_dimension.abs() < f64::EPSILON);
}

// rationale: Concurrency — multiple producers inserting open runs
// against the same on-disk DB serialise correctly under SQLite WAL
// (no duplicate primary keys, no row loss).
#[test]
fn m7_concurrent_insert_run_serializes_correctly() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("concurrent.db");
    // Pre-create the schema so each thread's open_database call is a no-op
    // (idempotent CREATE IF NOT EXISTS).
    {
        let _ = open_database(&path).expect("init");
    }
    let path = Arc::new(path);
    let threads = 4_u32;
    let per_thread = 25_u32;
    let mut handles = Vec::with_capacity(threads as usize);
    for t in 0..threads {
        let p = Arc::clone(&path);
        handles.push(thread::spawn(move || {
            let conn = open_database(&p).expect("thread open");
            for i in 0..per_thread {
                let ts = format!("2026-05-20T00:00:{t:02}.{i:03}Z");
                insert_run(&conn, &ts).expect("insert");
            }
        }));
    }
    for h in handles {
        h.join().expect("thread join");
    }
    let conn = open_database(&path).expect("final open");
    let n: i64 = conn
        .query_row("SELECT COUNT(*) FROM workflow_runs", [], |r| r.get(0))
        .expect("count");
    let expected = i64::from(threads * per_thread);
    assert_eq!(n, expected, "expected {expected} rows after concurrent inserts, got {n}");
    // All ids must be distinct (monotonic AUTOINCREMENT).
    let mut stmt = conn
        .prepare("SELECT id FROM workflow_runs")
        .expect("prepare");
    let ids: std::collections::HashSet<i64> = stmt
        .query_map([], |r| r.get::<_, i64>(0))
        .expect("query")
        .map(|r| r.expect("row"))
        .collect();
    let expected_count = usize::try_from(expected).expect("count fits in usize on 64-bit hosts");
    assert_eq!(
        ids.len(),
        expected_count,
        "id collision under concurrent insert",
    );
}

// rationale: Boundary — `find_by_outcome` filters correctly across all
// 3 distinct closed outcomes; `find_open` finds the open one. The four
// states (open, ok, fail, abort) partition cleanly.
#[test]
fn m7_find_by_outcome_filters_correctly_across_3_states() {
    let conn = mem();
    let id_open = insert_run(&conn, "2026-05-20T00:00:00Z").expect("ins");
    let id_ok = insert_run(&conn, "2026-05-20T00:00:01Z").expect("ins");
    let id_fail = insert_run(&conn, "2026-05-20T00:00:02Z").expect("ins");
    let id_abort = insert_run(&conn, "2026-05-20T00:00:03Z").expect("ins");
    close_run(&conn, id_ok, "2026-05-20T01:00:00Z", "ok").expect("close ok");
    close_run(&conn, id_fail, "2026-05-20T01:00:01Z", "fail").expect("close fail");
    close_run(&conn, id_abort, "2026-05-20T01:00:02Z", "abort").expect("close abort");

    let open_rows = find_open(&conn, 10).expect("find_open");
    assert_eq!(open_rows.len(), 1);
    assert_eq!(open_rows[0].id, id_open);

    let ok_rows = find_by_outcome(&conn, "ok", 10).expect("ok");
    assert_eq!(ok_rows.len(), 1);
    assert_eq!(ok_rows[0].id, id_ok);

    let fail_rows = find_by_outcome(&conn, "fail", 10).expect("fail");
    assert_eq!(fail_rows.len(), 1);
    assert_eq!(fail_rows[0].id, id_fail);

    let abort_rows = find_by_outcome(&conn, "abort", 10).expect("abort");
    assert_eq!(abort_rows.len(), 1);
    assert_eq!(abort_rows[0].id, id_abort);

    // `unknown` returns nothing — no row was closed with that outcome.
    let unknown_rows = find_by_outcome(&conn, "unknown", 10).expect("unknown");
    assert!(unknown_rows.is_empty());
}
