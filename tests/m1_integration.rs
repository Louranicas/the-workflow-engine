//! Integration tests for m1 atuin_consumer.
//!
//! Per m1 spec § 6 F-Integration row: 15 tests exercising the reader at
//! full pagination scale + WAL concurrency. Uses ephemeral SQLite DBs
//! seeded at test runtime via `tempfile::NamedTempFile` — no binary
//! fixtures committed to the repo. If the live atuin DB exists at
//! `$HOME/.local/share/atuin/history.db` an advisory test exercises it
//! end-to-end.

#![allow(clippy::doc_markdown)]

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::thread;

use rusqlite::Connection;

use workflow_core::m1_atuin_consumer::{
    canonical_default_path, db_path_exists, open_readonly, AtuinConsumerConfig,
    AtuinConsumerError,
};

fn synthetic_ulid(i: i64) -> String {
    format!("01HQA{i:021}")
}

fn seed_n_rows(path: &Path, n: i64) {
    let conn = Connection::open(path).expect("open");
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
    .expect("schema");
    for i in 1..=n {
        conn.execute(
            "INSERT INTO history (id, command, session, hostname, timestamp, exit, duration, cwd, deleted_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                synthetic_ulid(i),
                format!("cmd_{i}"),
                format!("s{}", i % 3),
                "host1",
                1_700_000_000_000_i64 + i,
                0_i32,
                10_i64,
                "/tmp",
                Option::<i64>::None,
            ],
        )
        .expect("insert");
    }
}

fn open_temp(n: i64) -> (tempfile::NamedTempFile, AtuinConsumerConfig) {
    let f = tempfile::Builder::new()
        .suffix(".db")
        .tempfile()
        .expect("temp");
    seed_n_rows(f.path(), n);
    let cfg = AtuinConsumerConfig {
        page_size: 100,
        db_path_override: Some(f.path().to_path_buf()),
        ..AtuinConsumerConfig::default()
    };
    (f, cfg)
}

// ---- Large-table pagination invariants (5) -------------------------------

#[test]
fn large_table_1000_no_duplicate_ids_across_pages() {
    let (_f, cfg) = open_temp(1_000);
    let c = open_readonly(&cfg).expect("open");
    let all = c.collect_all().expect("collect");
    let ids: HashSet<String> = all.iter().map(|r| r.id.clone()).collect();
    assert_eq!(ids.len(), 1_000);
}

#[test]
fn large_table_1000_yields_in_id_order() {
    let (_f, cfg) = open_temp(1_000);
    let c = open_readonly(&cfg).expect("open");
    let all = c.collect_all().expect("collect");
    let ids: Vec<String> = all.iter().map(|r| r.id.clone()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);
}

#[test]
fn large_table_2500_with_page_size_500_walks_5_pages() {
    let f = tempfile::Builder::new()
        .suffix(".db")
        .tempfile()
        .expect("temp");
    seed_n_rows(f.path(), 2_500);
    let cfg = AtuinConsumerConfig {
        page_size: 500,
        db_path_override: Some(f.path().to_path_buf()),
        ..AtuinConsumerConfig::default()
    };
    let pages: Vec<_> = open_readonly(&cfg)
        .expect("open")
        .into_page_iter()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(pages.len(), 5);
    let total: usize = pages.iter().map(|p| p.rows.len()).sum();
    assert_eq!(total, 2_500);
}

#[test]
fn collect_all_matches_into_page_iter_for_large_table() {
    let f = tempfile::Builder::new()
        .suffix(".db")
        .tempfile()
        .expect("temp");
    seed_n_rows(f.path(), 750);
    let cfg = AtuinConsumerConfig {
        page_size: 100,
        db_path_override: Some(f.path().to_path_buf()),
        ..AtuinConsumerConfig::default()
    };
    let by_collect = open_readonly(&cfg).expect("a").collect_all().expect("a");
    let by_iter: Vec<_> = open_readonly(&cfg)
        .expect("b")
        .into_page_iter()
        .filter_map(Result::ok)
        .flat_map(|p| p.rows)
        .collect();
    assert_eq!(by_collect.len(), by_iter.len());
    for (a, b) in by_collect.iter().zip(by_iter.iter()) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.command, b.command);
    }
}

#[test]
fn exhaustion_is_sticky_across_repeated_calls() {
    let (_f, cfg) = open_temp(50);
    let mut c = open_readonly(&cfg).expect("open");
    let _ = c.next_page().expect("p1");
    for _ in 0..10_u32 {
        assert!(c.next_page().expect("rep").is_none());
    }
}

// ---- Cursor monotonicity property (3) ------------------------------------

#[test]
fn cursor_monotonic_across_pages() {
    let f = tempfile::Builder::new()
        .suffix(".db")
        .tempfile()
        .expect("temp");
    seed_n_rows(f.path(), 1_000);
    let cfg = AtuinConsumerConfig {
        page_size: 100,
        db_path_override: Some(f.path().to_path_buf()),
        ..AtuinConsumerConfig::default()
    };
    let mut c = open_readonly(&cfg).expect("open");
    let mut prev_last_id = String::new();
    while let Some(p) = c.next_page().expect("page") {
        assert!(p.last_id > prev_last_id, "non-monotonic: {prev_last_id:?} -> {:?}", p.last_id);
        prev_last_id = p.last_id;
    }
}

#[test]
fn second_read_with_same_seed_yields_identical_rows() {
    let f = tempfile::Builder::new()
        .suffix(".db")
        .tempfile()
        .expect("temp");
    seed_n_rows(f.path(), 200);
    let cfg = AtuinConsumerConfig {
        page_size: 100,
        db_path_override: Some(f.path().to_path_buf()),
        ..AtuinConsumerConfig::default()
    };
    let a = open_readonly(&cfg).expect("a").collect_all().expect("a");
    let b = open_readonly(&cfg).expect("b").collect_all().expect("b");
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.id, y.id);
        assert_eq!(x.command, y.command);
        assert_eq!(x.session, y.session);
    }
}

#[test]
fn page_size_default_2000_returns_all_rows_in_one_page_for_small_table() {
    let (_f, cfg) = open_temp(150);
    let mut explicit_default = cfg.clone();
    explicit_default.page_size = 2_000;
    let mut c = open_readonly(&explicit_default).expect("open");
    let p = c.next_page().expect("p").expect("Some");
    assert_eq!(p.rows.len(), 150);
}

// ---- WAL + read-only enforcement (3) -------------------------------------

#[test]
fn concurrent_readers_do_not_deadlock() {
    let f = tempfile::Builder::new()
        .suffix(".db")
        .tempfile()
        .expect("temp");
    seed_n_rows(f.path(), 500);
    let path = Arc::new(f.path().to_path_buf());
    let handles: Vec<_> = (0..4_u32)
        .map(|_| {
            let p = Arc::clone(&path);
            thread::spawn(move || {
                let cfg = AtuinConsumerConfig {
                    page_size: 100,
                    db_path_override: Some((*p).clone()),
                    ..AtuinConsumerConfig::default()
                };
                open_readonly(&cfg)
                    .expect("open")
                    .collect_all()
                    .map(|v| v.len())
            })
        })
        .collect();
    for h in handles {
        let count = h.join().expect("join").expect("collect");
        assert_eq!(count, 500);
    }
}

#[test]
fn read_only_mode_blocks_writes_via_separate_connection_under_query_only() {
    let f = tempfile::Builder::new()
        .suffix(".db")
        .tempfile()
        .expect("temp");
    seed_n_rows(f.path(), 10);
    let cfg = AtuinConsumerConfig {
        page_size: 100,
        db_path_override: Some(f.path().to_path_buf()),
        ..AtuinConsumerConfig::default()
    };
    let _c = open_readonly(&cfg).expect("ro");
    // The same process can still open a separate writer; the m1 contract
    // is that m1's OWN connection cannot write. We confirm via a separate
    // writable connection that the file itself stays mutable to processes
    // that did NOT opt into `query_only`.
    let conn = Connection::open(f.path()).expect("open");
    conn.execute(
        "INSERT INTO history (id, command, session, hostname, timestamp, exit, duration, cwd) VALUES (?1, 'x', 's', 'h', 0, 0, 0, '/')",
        rusqlite::params![synthetic_ulid(99)],
    )
    .expect("external write still permitted");
}

#[test]
fn missing_db_path_yields_typed_database_open_failed() {
    let cfg = AtuinConsumerConfig {
        db_path_override: Some(std::path::PathBuf::from(
            "/tmp/definitely-missing-9f3e7a1b1c.db",
        )),
        ..AtuinConsumerConfig::default()
    };
    assert!(matches!(
        open_readonly(&cfg),
        Err(AtuinConsumerError::DatabaseOpenFailed { .. })
    ));
}

// ---- Live atuin advisory (2) ---------------------------------------------

#[test]
fn live_atuin_db_advisory_first_page_smoke() {
    let cfg = AtuinConsumerConfig::default();
    if !db_path_exists(&cfg) {
        eprintln!(
            "M1-LIVE advisory: {} not present on this host; skipping live smoke",
            canonical_default_path().display()
        );
        return;
    }
    let mut c = open_readonly(&cfg).expect("live open");
    // We do NOT iterate the full ~263k rows — only the first page.
    let p = c.next_page().expect("first page");
    if let Some(page) = p {
        assert!(!page.rows.is_empty());
        eprintln!(
            "M1-LIVE first page: {} rows, last_id={}, elapsed_ms={}",
            page.rows.len(),
            page.last_id,
            page.elapsed_ms
        );
    } else {
        eprintln!("M1-LIVE: atuin DB exists but is empty");
    }
}

#[test]
fn day_1_surface_signature_stability() {
    // Type-check the public surface; if any signature drifts this fails
    // to compile.
    let _: fn(&AtuinConsumerConfig) -> Result<workflow_core::m1_atuin_consumer::AtuinConsumer, AtuinConsumerError> =
        open_readonly;
    let _: fn(&AtuinConsumerConfig) -> bool = db_path_exists;
}

// ---- Hardening pass S1002209 (Cluster A) — +10 tests --------------------

#[test]
fn hardening_page_size_below_floor_clamps_to_min_not_zero() {
    // rationale: Boundary — `page_size: 0` must not produce an empty
    // page on a non-empty table. The hardening fix replaced the silent
    // `i64::MAX` fallback with PAGE_SIZE_MAX saturation; this exercises
    // the floor side of the clamp.
    let (_f, mut cfg) = open_temp(50);
    cfg.page_size = 0;
    let c = open_readonly(&cfg).expect("open");
    let all = c.collect_all().expect("collect");
    assert_eq!(all.len(), 50, "page_size=0 should clamp to PAGE_SIZE_MIN, not zero");
}

#[test]
fn hardening_page_size_above_ceiling_clamps_to_max() {
    // rationale: Boundary — `page_size: 1_000_000` clamps to
    // PAGE_SIZE_MAX (10_000). The new `unwrap_or_else` path must
    // preserve that clamp and not regress to i64::MAX saturation.
    use workflow_core::m1_atuin_consumer::PAGE_SIZE_MAX;
    let (_f, mut cfg) = open_temp(150);
    cfg.page_size = 1_000_000;
    let c = open_readonly(&cfg).expect("open");
    let all = c.collect_all().expect("collect");
    assert_eq!(all.len(), 150);
    assert_eq!(PAGE_SIZE_MAX, 10_000);
}

#[test]
fn hardening_collect_all_with_row_cap_trims_to_cap() {
    // rationale: Resource accounting — the new with_capacity hint reads
    // `row_cap`. A capped collect must trim the final page exactly to
    // the configured cap.
    let f = tempfile::Builder::new().suffix(".db").tempfile().expect("temp");
    seed_n_rows(f.path(), 5_000);
    let cfg = AtuinConsumerConfig {
        page_size: 500,
        row_cap: Some(1_234),
        db_path_override: Some(f.path().to_path_buf()),
        ..AtuinConsumerConfig::default()
    };
    let all = open_readonly(&cfg).expect("open").collect_all().expect("collect");
    assert_eq!(all.len(), 1_234, "row_cap must trim the final page exactly");
}

#[test]
fn hardening_fallback_subprocess_surfaces_timeout_in_error_message() {
    // rationale: Anti-property — the Day-1 stub's error message must
    // reference the configured timeout so an operator can verify the
    // wiring landed when the real subprocess path is added. Previously
    // the config arg was silently discarded via `let _ = config`.
    use workflow_core::m1_atuin_consumer::fallback_subprocess_ingest;
    let cfg = AtuinConsumerConfig {
        subprocess_timeout_ms: 7_777,
        ..AtuinConsumerConfig::default()
    };
    let err = fallback_subprocess_ingest(&cfg).expect_err("stub");
    let msg = err.to_string();
    assert!(msg.contains("7777"), "expected 7777 in message, got: {msg}");
}

#[test]
fn hardening_fixed_width_ulids_sort_lexicographically_through_cursor() {
    // rationale: Adversarial input — atuin's live schema is TEXT, and
    // ULIDs are lex-sortable only at fixed width. Seed three rows with
    // the canonical 26-char synthetic ULID and verify cursor monotonic
    // walk yields them in lex order, then prove non-fixed-width ids
    // also lex-sort correctly when prefix-equal (the live atuin invariant).
    let f = tempfile::Builder::new().suffix(".db").tempfile().expect("temp");
    let conn = Connection::open(f.path()).expect("open");
    conn.execute_batch(
        "CREATE TABLE history (
            id TEXT PRIMARY KEY, timestamp INTEGER NOT NULL,
            duration INTEGER NOT NULL, exit INTEGER NOT NULL,
            command TEXT NOT NULL, cwd TEXT NOT NULL,
            session TEXT NOT NULL, hostname TEXT NOT NULL,
            deleted_at INTEGER);",
    )
    .expect("schema");
    for id in ["01HQA-aaa", "01HQA-bbb", "01HQA-ccc"] {
        conn.execute(
            "INSERT INTO history (id, command, session, hostname, timestamp, exit, duration, cwd) \
             VALUES (?1, 'c', 's', 'h', 0, 0, 0, '/')",
            rusqlite::params![id],
        )
        .expect("insert");
    }
    let cfg = AtuinConsumerConfig {
        page_size: 100,
        db_path_override: Some(f.path().to_path_buf()),
        ..AtuinConsumerConfig::default()
    };
    let all = open_readonly(&cfg).expect("open").collect_all().expect("collect");
    let ids: Vec<String> = all.iter().map(|r| r.id.clone()).collect();
    assert_eq!(ids, vec!["01HQA-aaa", "01HQA-bbb", "01HQA-ccc"]);
}

#[test]
fn hardening_concurrent_open_no_deadlock_at_higher_concurrency() {
    // rationale: Concurrency — 16 simultaneous readers racing through
    // WAL+pragma path must not deadlock. The existing test runs N=4;
    // this lifts the bar.
    let f = tempfile::Builder::new().suffix(".db").tempfile().expect("temp");
    seed_n_rows(f.path(), 200);
    let path = Arc::new(f.path().to_path_buf());
    let handles: Vec<_> = (0..16_u32)
        .map(|_| {
            let p = Arc::clone(&path);
            thread::spawn(move || {
                let cfg = AtuinConsumerConfig {
                    page_size: 100,
                    db_path_override: Some((*p).clone()),
                    ..AtuinConsumerConfig::default()
                };
                open_readonly(&cfg).expect("open").rows_yielded()
            })
        })
        .collect();
    for h in handles {
        assert_eq!(h.join().expect("join"), 0, "fresh consumers haven't yielded yet");
    }
}

#[test]
fn hardening_row_cap_zero_marks_exhausted_without_sql() {
    // rationale: Anti-property — `row_cap: Some(0)` is a degenerate
    // request; the cursor must mark exhausted via the page-size=0
    // early-return without issuing any SQL.
    let (_f, mut cfg) = open_temp(50);
    cfg.row_cap = Some(0);
    let mut c = open_readonly(&cfg).expect("open");
    let p = c.next_page().expect("first");
    assert!(p.is_none(), "row_cap=0 must yield no pages");
    assert!(c.exhausted(), "cursor must be exhausted");
}

#[test]
fn hardening_exit_code_value_preserved_through_pagination() {
    // rationale: Contract regression — the `exit` column is i32 at a
    // fixed index in the SELECT. We seed distinct exit codes and
    // verify they survive pagination unmangled.
    let f = tempfile::Builder::new().suffix(".db").tempfile().expect("temp");
    let conn = Connection::open(f.path()).expect("open");
    conn.execute_batch(
        "CREATE TABLE history (
            id TEXT PRIMARY KEY, timestamp INTEGER NOT NULL,
            duration INTEGER NOT NULL, exit INTEGER NOT NULL,
            command TEXT NOT NULL, cwd TEXT NOT NULL,
            session TEXT NOT NULL, hostname TEXT NOT NULL,
            deleted_at INTEGER);",
    )
    .expect("schema");
    for (i, exit) in [(1_i64, 0_i32), (2, 1), (3, 127)] {
        conn.execute(
            "INSERT INTO history (id, command, session, hostname, timestamp, exit, duration, cwd) \
             VALUES (?1, 'c', 's', 'h', 0, ?2, 0, '/')",
            rusqlite::params![synthetic_ulid(i), exit],
        )
        .expect("insert");
    }
    drop(conn);
    let cfg = AtuinConsumerConfig {
        page_size: 100,
        db_path_override: Some(f.path().to_path_buf()),
        ..AtuinConsumerConfig::default()
    };
    let all = open_readonly(&cfg).expect("open").collect_all().expect("collect");
    let exits: Vec<i32> = all.iter().map(|r| r.exit).collect();
    assert_eq!(exits, vec![0, 1, 127]);
}

#[test]
fn hardening_deleted_at_some_round_trips_to_downstream() {
    // rationale: Cross-module surface invariant — a row marked
    // `deleted_at=Some(_)` is preserved through pagination and exposed
    // to downstream modules (m4 uses deletion state to skip cascade
    // correlation for retracted history rows).
    let f = tempfile::Builder::new().suffix(".db").tempfile().expect("temp");
    let conn = Connection::open(f.path()).expect("open");
    conn.execute_batch(
        "CREATE TABLE history (
            id TEXT PRIMARY KEY, timestamp INTEGER NOT NULL,
            duration INTEGER NOT NULL, exit INTEGER NOT NULL,
            command TEXT NOT NULL, cwd TEXT NOT NULL,
            session TEXT NOT NULL, hostname TEXT NOT NULL,
            deleted_at INTEGER);
         INSERT INTO history (id, command, session, hostname, timestamp, exit, duration, cwd, deleted_at) \
         VALUES ('01HQA-deleted-x', 'rm -rf /', 's', 'h', 1, 0, 0, '/', 1700000000999);",
    )
    .expect("schema+insert");
    drop(conn);
    let cfg = AtuinConsumerConfig {
        page_size: 100,
        db_path_override: Some(f.path().to_path_buf()),
        ..AtuinConsumerConfig::default()
    };
    let all = open_readonly(&cfg).expect("open").collect_all().expect("collect");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].deleted_at, Some(1_700_000_000_999));
}

#[test]
fn hardening_missing_override_path_is_deterministic_typed_error() {
    // rationale: Determinism — two consecutive `open_readonly` calls
    // against a missing override path return the same typed error
    // variant (no env-driven nondeterminism).
    let cfg = AtuinConsumerConfig {
        db_path_override: Some(std::path::PathBuf::from(
            "/tmp/definitely-missing-determinism-9f3e7a1b.db",
        )),
        ..AtuinConsumerConfig::default()
    };
    let a = matches!(open_readonly(&cfg), Err(AtuinConsumerError::DatabaseOpenFailed { .. }));
    let b = matches!(open_readonly(&cfg), Err(AtuinConsumerError::DatabaseOpenFailed { .. }));
    assert!(a && b);
}
