//! CC-1 Cascade-Cost Coupling — Cluster B internal cross-module
//! integration via m7 join (Wave-C2).
//!
//! m4 (cascade correlator) + m5 (battern step record) + m6 (context
//! cost) outputs all flow into the central m7 `workflow_runs` SQLite
//! table. This suite exercises the synergy at the join surface:
//!
//! - m4 `CascadeClusterId` round-trips through m7 via the
//!   `ClusterBObservation::Cascade { cluster_id, .. }` discriminant
//!   stored in `consumer_inputs`.
//! - m5 `BatternId` round-trips through m7 via the
//!   `ClusterBObservation::BatternStep { battern_id, .. }` discriminant.
//! - m6 `SessionCostRecord.total_cost_proxy` records on the m7 row via
//!   `update_cost_tokens`; the F9 `cost_tokens=None` signal survives
//!   the trip without collapsing to `Some(0)`.
//! - F11 anti-property transit — m4/m5 opaque ids never spill a
//!   human-readable substring into the serialised m7 row's
//!   `consumer_inputs` JSON.
//! - Concurrent m4 + m5 + m6 writers against the same m7 row serialise
//!   atomically under SQLite WAL (no lost merges, no torn JSON).
//!
//! Real SQLite is used (`open_memory` for in-memory, `open_database` for
//! the concurrency case); m4/m5/m6 producers are exercised at their
//! public API surface.

#![allow(clippy::doc_markdown)]

use std::sync::Arc;
use std::thread;

use tempfile::tempdir;

use workflow_core::m4_cascade::{
    AtuinStep, CascadeCorrelator, CascadeCorrelatorConfig, DispatchRecord,
};
use workflow_core::m5_battern::{BatternStepRecord, BatternStepRecordConfig};
use workflow_core::m6_cost::{
    ContextCostRecord, ContextCostRecordConfig, SessionCostRecord, WorkflowOutcome,
};
use workflow_core::m7_workflow_runs::{
    close_run, find_by_id, insert_run, merge_observation, open_database, open_memory,
    update_cost_tokens, ClusterBObservation,
};

fn step(id: &str, ts_ns: i64, cmd: &str, session: &str) -> AtuinStep {
    AtuinStep {
        id: id.to_owned(),
        ts_ns,
        command: cmd.to_owned(),
        cwd: "/tmp".into(),
        session: session.to_owned(),
        exit: 0,
    }
}

fn dispatch_rec(ts_ns: i64, pane: &str, session: &str) -> DispatchRecord {
    DispatchRecord {
        ts_ns,
        pane_label: pane.to_owned(),
        binary: "cc-dispatch".to_owned(),
        session: session.to_owned(),
    }
}

fn now_ms_for_test() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}

// rationale: Cross-module — m4 emits `CascadeClusterId`; serialising it
// through `ClusterBObservation::Cascade` into m7's `consumer_inputs`
// JSON preserves the string identity end-to-end.
#[test]
fn cc1_m4_cluster_id_round_trips_through_m7_workflow_run() {
    let conn = open_memory().expect("memory open");
    let run_id = insert_run(&conn, "2026-05-20T00:00:00Z").expect("insert");

    // m4: produce a real cluster from a real correlation pass.
    let correlator = CascadeCorrelator::new(CascadeCorrelatorConfig {
        min_pane_count: 2,
        max_gap_ms: 30_000,
        ..CascadeCorrelatorConfig::default()
    });
    let steps = vec![
        step("a", 1_000_000_000, "rg foo", "session-A"),
        step("b", 1_500_000_000, "cc-dispatch ALPHA", "session-B"),
    ];
    let dispatches = vec![
        dispatch_rec(1_000_000_000, "ALPHA-LEFT", "session-A"),
        dispatch_rec(1_500_000_000, "BETA-LEFT", "session-B"),
    ];
    let clusters = correlator.correlate(&steps, &dispatches);
    assert_eq!(clusters.len(), 1);
    let m4_cluster_id_str = clusters[0].cluster_id.as_str().to_owned();

    // Merge into m7 row.
    merge_observation(
        &conn,
        run_id,
        &ClusterBObservation::Cascade {
            cluster_id: m4_cluster_id_str.clone(),
            session_range: (
                clusters[0].window_start_ns,
                clusters[0].window_end_ns,
            ),
        },
    )
    .expect("merge cascade");

    // Read back, parse the consumer_inputs JSON, assert string identity.
    let row = find_by_id(&conn, run_id).expect("find");
    let parsed: serde_json::Value = serde_json::from_str(&row.consumer_inputs).expect("parse");
    let cascade = parsed
        .as_object()
        .expect("object")
        .get("cascade")
        .expect("cascade key present");
    let stored = cascade
        .get("cluster_id")
        .and_then(serde_json::Value::as_str)
        .expect("cluster_id is a string");
    assert_eq!(
        stored, m4_cluster_id_str,
        "m4 CascadeClusterId must round-trip through m7 unchanged",
    );
}

// rationale: Cross-module — m5 emits `BatternId`; serialising it
// through `ClusterBObservation::BatternStep` into m7's `consumer_inputs`
// JSON preserves the string identity end-to-end.
#[test]
fn cc1_m5_battern_id_round_trips_through_m7_workflow_run() {
    let conn = open_memory().expect("memory open");
    let run_id = insert_run(&conn, "2026-05-20T00:00:00Z").expect("insert");

    let recorder = BatternStepRecord::new(BatternStepRecordConfig::default()).expect("regex");
    let steps = vec![
        step("a", 1_000_000_000, "cc-dispatch ALPHA", "s1"),
        step("b", 2_000_000_000, "cc-health", "s1"),
    ];
    let obs = recorder.observe(&steps);
    assert!(!obs.is_empty(), "m5 must emit at least one observation");
    let m5_battern_id_str = obs[0].battern_id.as_str().to_owned();

    merge_observation(
        &conn,
        run_id,
        &ClusterBObservation::BatternStep {
            battern_id: m5_battern_id_str.clone(),
            step_index: 0,
            duration_ms: 100,
            outcome: "ok".into(),
        },
    )
    .expect("merge battern_step");

    let row = find_by_id(&conn, run_id).expect("find");
    let parsed: serde_json::Value = serde_json::from_str(&row.consumer_inputs).expect("parse");
    let battern = parsed
        .as_object()
        .expect("object")
        .get("battern_step")
        .expect("battern_step key present");
    let stored = battern
        .get("battern_id")
        .and_then(serde_json::Value::as_str)
        .expect("battern_id is a string");
    assert_eq!(
        stored, m5_battern_id_str,
        "m5 BatternId must round-trip through m7 unchanged",
    );
}

// rationale: Cross-module — m6 produces a `SessionCostRecord` per
// workflow run; the `total_cost_proxy` field is the join surface against
// m7's `cost_tokens` column. Records on the same run id must agree
// after both writers complete.
#[test]
fn cc1_m6_session_cost_record_joins_to_m7_workflow_run_via_workflow_id() {
    let conn = open_memory().expect("memory open");
    let run_id = insert_run(&conn, "2026-05-20T00:00:00Z").expect("insert");

    let cost_recorder = ContextCostRecord::new(ContextCostRecordConfig::default());
    let raw = SessionCostRecord {
        session_id: format!("run-{run_id}"),
        token_cost_input_proxy: 60,
        token_cost_output_proxy: 40,
        total_cost_proxy: 100,
        outcome: Some(WorkflowOutcome::Explored),
        exploration_baseline: None,
        cost_band: None,
        recorded_at_ms: now_ms_for_test(),
    };
    let recorded = cost_recorder.record_and_update_baseline(raw);
    // m6 wrote into the baseline; the record carries the same total_cost.
    assert_eq!(recorded.total_cost_proxy, 100);

    // Carry the cost_tokens onto the m7 row.
    update_cost_tokens(&conn, run_id, recorded.total_cost_proxy).expect("cost update");
    let row = find_by_id(&conn, run_id).expect("find");
    assert_eq!(
        row.cost_tokens,
        Some(recorded.total_cost_proxy),
        "m6 SessionCostRecord.total_cost_proxy must agree with m7 cost_tokens",
    );
}

// rationale: F9 zero-weight — m6 records carrying `cost_tokens = None`
// (no signal) must survive the m6→m7 boundary as `None`, NOT collapse
// to `Some(0)`. The `Some(0)` signal must remain reachable as an
// explicit explicit-zero-cost record (distinguishable from no-signal).
#[test]
fn cc1_f9_zero_weight_preserved_across_cluster_b_to_c_boundary() {
    let conn = open_memory().expect("memory open");

    // Row A: m6 record with outcome but cost-not-yet-known → no
    // `update_cost_tokens` call; m7 row `cost_tokens` stays None.
    let id_a = insert_run(&conn, "2026-05-20T00:00:00Z").expect("ins");
    merge_observation(
        &conn,
        id_a,
        &ClusterBObservation::ContextCost {
            session_id: id_a,
            cost_tokens: 0, // observation payload may signal zero …
        },
    )
    .expect("merge ctx-cost zero");
    // … but the m7 column itself stays None because we didn't update_cost.
    let row_a = find_by_id(&conn, id_a).expect("find a");
    assert_eq!(
        row_a.cost_tokens, None,
        "F9: no update_cost_tokens call must leave cost_tokens column None",
    );

    // Row B: explicit zero — `Some(0)` is a real signal, distinct from None.
    let id_b = insert_run(&conn, "2026-05-20T00:00:01Z").expect("ins");
    update_cost_tokens(&conn, id_b, 0).expect("cost zero");
    let row_b = find_by_id(&conn, id_b).expect("find b");
    assert_eq!(row_b.cost_tokens, Some(0));
    assert_ne!(
        row_a.cost_tokens, row_b.cost_tokens,
        "F9: None vs Some(0) MUST remain distinct across the m6→m7 boundary",
    );
}

// rationale: F11 anti-property transit — m4 cluster ids + m5 battern
// ids must remain opaque after being stored in m7's serialised JSON.
// No human-meaningful substring of the producer inputs must leak.
#[test]
fn cc1_cluster_id_battern_id_battern_step_token_no_human_substring_in_m7_serialised_row() {
    let conn = open_memory().expect("memory open");
    let run_id = insert_run(&conn, "2026-05-20T00:00:00Z").expect("insert");

    // m4 producer.
    let correlator = CascadeCorrelator::new(CascadeCorrelatorConfig {
        min_pane_count: 2,
        max_gap_ms: 30_000,
        ..CascadeCorrelatorConfig::default()
    });
    let m4_steps = vec![
        step("a", 1_000_000_000, "rg foo", "s1"),
        step("b", 1_500_000_000, "cc-dispatch ALPHA", "s2"),
    ];
    let m4_dispatches = vec![
        dispatch_rec(1_000_000_000, "ALPHA-LEFT-meaningful-pane", "s1"),
        dispatch_rec(1_500_000_000, "BETA-RIGHT-meaningful-pane", "s2"),
    ];
    let clusters = correlator.correlate(&m4_steps, &m4_dispatches);
    assert_eq!(clusters.len(), 1);
    merge_observation(
        &conn,
        run_id,
        &ClusterBObservation::Cascade {
            cluster_id: clusters[0].cluster_id.as_str().to_owned(),
            session_range: (
                clusters[0].window_start_ns,
                clusters[0].window_end_ns,
            ),
        },
    )
    .expect("merge cascade");

    // m5 producer.
    let recorder = BatternStepRecord::new(BatternStepRecordConfig::default()).expect("regex");
    let m5_steps = vec![
        step("c", 1_000_000_000, "cc-dispatch ALPHA-LEFT-meaningful-pane", "s1"),
        step("d", 2_000_000_000, "cc-health", "s1"),
    ];
    let obs = recorder.observe(&m5_steps);
    assert!(!obs.is_empty());
    merge_observation(
        &conn,
        run_id,
        &ClusterBObservation::BatternStep {
            battern_id: obs[0].battern_id.as_str().to_owned(),
            step_index: 0,
            duration_ms: 100,
            outcome: "ok".into(),
        },
    )
    .expect("merge battern_step");

    // Pull the row back; the entire serialised JSON must not contain
    // ANY of the human-meaningful labels we fed into m4/m5.
    let row = find_by_id(&conn, run_id).expect("find");
    let serialised = serde_json::to_string(&row).expect("ser");
    for forbidden in [
        "ALPHA",
        "BETA",
        "LEFT",
        "RIGHT",
        "meaningful",
        "dispatch",
        "health",
    ] {
        assert!(
            !serialised.contains(forbidden),
            "F11 transitive leak: {forbidden:?} found in m7-serialised row {serialised}",
        );
    }
}

// rationale: Concurrency — m4, m5, m6 are three independent producers
// that all write into the central m7 SQLite `workflow_runs` table.
//
// `merge_observation` is read-modify-write on `consumer_inputs`, so
// concurrent merges against the SAME row from independent connections
// can clobber each other (a known limitation flagged for the
// orchestrator — see report). The producer architecture in `wf-crystallise`
// serialises merge-writes through a single owner thread. This test
// exercises that pattern: m4 + m5 merges go through one writer thread
// (matching the binary's design), while m6 `update_cost_tokens` (which
// is a single-column UPDATE, not RMW) runs in parallel against a
// separate connection. Under SQLite WAL the column-update + the
// JSON-blob merges serialise atomically — both surfaces land, no torn
// JSON.
#[test]
fn cc1_concurrent_m4_m5_m6_writes_to_m7_serialise_atomically() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("cc1_concurrent.db");
    let conn_init = open_database(&path).expect("init");
    let run_id = insert_run(&conn_init, "2026-05-20T00:00:00Z").expect("insert");
    drop(conn_init);

    let path = Arc::new(path);

    // Thread A — m4+m5 merge writer (serialised, matches wf-crystallise
    // owner-thread pattern).
    let p_merge = Arc::clone(&path);
    let h_merge = thread::spawn(move || {
        let conn = open_database(&p_merge).expect("merge open");
        let correlator = CascadeCorrelator::new(CascadeCorrelatorConfig {
            min_pane_count: 2,
            max_gap_ms: 30_000,
            ..CascadeCorrelatorConfig::default()
        });
        let m4_steps = vec![
            step("a", 1_000_000_000, "rg foo", "s1"),
            step("b", 1_500_000_000, "cc-dispatch X", "s2"),
        ];
        let m4_dispatches = vec![
            dispatch_rec(1_000_000_000, "X-LEFT", "s1"),
            dispatch_rec(1_500_000_000, "Y-LEFT", "s2"),
        ];
        let clusters = correlator.correlate(&m4_steps, &m4_dispatches);
        assert_eq!(clusters.len(), 1);
        let recorder = BatternStepRecord::new(BatternStepRecordConfig::default()).expect("regex");
        let m5_steps = vec![
            step("c", 1_000_000_000, "cc-dispatch X", "s1"),
            step("d", 2_000_000_000, "cc-health", "s1"),
        ];
        let obs = recorder.observe(&m5_steps);
        assert!(!obs.is_empty());
        // Interleave merges through the same owner connection.
        for i in 0..10_u32 {
            merge_observation(
                &conn,
                run_id,
                &ClusterBObservation::Cascade {
                    cluster_id: clusters[0].cluster_id.as_str().to_owned(),
                    session_range: (
                        clusters[0].window_start_ns,
                        clusters[0].window_end_ns,
                    ),
                },
            )
            .expect("merge cascade");
            merge_observation(
                &conn,
                run_id,
                &ClusterBObservation::BatternStep {
                    battern_id: obs[0].battern_id.as_str().to_owned(),
                    step_index: u8::try_from(i % 6).unwrap_or(0),
                    duration_ms: 50,
                    outcome: "ok".into(),
                },
            )
            .expect("merge battern_step");
        }
    });

    // Thread B — m6 cost producer (column update, not RMW; safe in
    // parallel via WAL).
    let p_cost = Arc::clone(&path);
    let h_cost = thread::spawn(move || {
        let conn = open_database(&p_cost).expect("cost open");
        for i in 0..10_i64 {
            update_cost_tokens(&conn, run_id, 100 + i).expect("update_cost");
        }
    });

    h_merge.join().expect("merge join");
    h_cost.join().expect("cost join");

    // After both threads finish, the row's consumer_inputs must parse
    // as a JSON object containing BOTH discriminant keys; the cost
    // column carries one of the m6 writes; nothing is torn.
    let conn = open_database(&path).expect("final open");
    let row = find_by_id(&conn, run_id).expect("find");
    let parsed: serde_json::Value = serde_json::from_str(&row.consumer_inputs)
        .expect("torn JSON: consumer_inputs failed to parse after concurrent writers");
    let obj = parsed.as_object().expect("must be JSON object");
    assert!(
        obj.contains_key("cascade"),
        "m4 cascade discriminant lost in serialised merge stream",
    );
    assert!(
        obj.contains_key("battern_step"),
        "m5 battern_step discriminant lost in serialised merge stream",
    );
    let cost = row.cost_tokens.expect("m6 cost lost under concurrent writers");
    assert!(
        (100..=109).contains(&cost),
        "unexpected cost_tokens: {cost} — m6 writer corrupt",
    );
    // Round-trip closure: m7 row remains operationally usable post-CC-1 churn.
    close_run(&conn, run_id, "2026-05-20T01:00:00Z", "ok").expect("close after CC-1 churn");
}
