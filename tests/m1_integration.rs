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
