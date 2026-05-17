//! Integration tests for m3 injection_db_consumer.
//!
//! Exercises the consumer against ephemeral SQLite DBs seeded at runtime
//! plus an advisory probe against the live habitat injection.db when
//! present.

#![allow(clippy::doc_markdown)]

use rusqlite::Connection;

use workflow_core::m3_injection_db_consumer::{
    db_path_exists, open_readonly, ChainType, ConsentLevel, InjectionDbConfig,
    InjectionDbError,
};

fn live_schema() -> &'static str {
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
    );
    CREATE INDEX idx_causal_unresolved ON causal_chain(reinforcement_count DESC)
        WHERE resolved_session IS NULL;"
}

fn seed_realistic_db() -> tempfile::NamedTempFile {
    let f = tempfile::Builder::new().suffix(".db").tempfile().expect("temp");
    let conn = Connection::open(f.path()).expect("open");
    conn.execute_batch(live_schema()).expect("schema");
    // 50 mixed chains: 30 unresolved (incl 5 Forget), 20 resolved.
    for i in 1..=30_i64 {
        let consent = if i % 6 == 0 { "Forget" } else { "Emit" };
        let chain_type = match i % 4 {
            0 => "bug",
            1 => "trap",
            2 => "plan",
            _ => "pattern",
        };
        let reinf = 50 - i;
        conn.execute(
            "INSERT INTO causal_chain (id, origin_session, resolved_session, chain_type, label, description, reinforcement_count, last_reinforced_session, consent) \
             VALUES (?1, 100, NULL, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![i, chain_type, format!("CHAIN-{i:03}"), format!("desc {i}"), reinf, Some(100_i64), consent],
        ).expect("insert unresolved");
    }
    for i in 31..=50_i64 {
        let resolved = 110 + (i - 31);
        conn.execute(
            "INSERT INTO causal_chain (id, origin_session, resolved_session, chain_type, label, description, reinforcement_count, last_reinforced_session, consent) \
             VALUES (?1, 100, ?2, 'plan', ?3, 'resolved desc', 1, 100, 'Emit')",
            rusqlite::params![i, resolved, format!("CHAIN-{i:03}")],
        ).expect("insert resolved");
    }
    f
}

// ---- Realistic-fixture e2e (4) ------------------------------------------

#[test]
fn realistic_fixture_unresolved_count_matches_expectation() {
    let f = seed_realistic_db();
    let cfg = InjectionDbConfig {
        db_path: f.path().to_path_buf(),
        ..InjectionDbConfig::default()
    };
    let c = open_readonly(&cfg).expect("open");
    let rows = c.read_unresolved().expect("read");
    // 30 unresolved total, 5 are Forget (i = 6, 12, 18, 24, 30) → 25 Emit.
    assert_eq!(rows.len(), 25);
}

#[test]
fn realistic_fixture_recently_resolved_within_window() {
    let f = seed_realistic_db();
    let cfg = InjectionDbConfig {
        db_path: f.path().to_path_buf(),
        ..InjectionDbConfig::default()
    };
    let c = open_readonly(&cfg).expect("open");
    let rows = c.read_recently_resolved().expect("read");
    // max_resolved = 129 (= 110 + 19); cutoff = 129 - 10 = 119;
    // rows with resolved_session > 119 → 130 doesn't exist; 120..=129 = 10.
    assert!(!rows.is_empty());
    for r in &rows {
        assert!(r.resolved_session.unwrap() > 119);
    }
}

#[test]
fn realistic_fixture_count_unresolved_includes_forget() {
    let f = seed_realistic_db();
    let cfg = InjectionDbConfig {
        db_path: f.path().to_path_buf(),
        ..InjectionDbConfig::default()
    };
    let c = open_readonly(&cfg).expect("open");
    let n = c.count_unresolved().expect("count");
    assert_eq!(n, 30);
}

#[test]
fn realistic_fixture_read_unresolved_ordered_by_reinforcement_desc() {
    let f = seed_realistic_db();
    let cfg = InjectionDbConfig {
        db_path: f.path().to_path_buf(),
        ..InjectionDbConfig::default()
    };
    let c = open_readonly(&cfg).expect("open");
    let rows = c.read_unresolved().expect("read");
    let counts: Vec<u32> = rows.iter().map(|r| r.reinforcement_count).collect();
    let mut sorted = counts.clone();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(counts, sorted);
}

// ---- ChainType / ConsentLevel surface (2) -------------------------------

#[test]
fn returned_rows_carry_typed_chain_type_enum() {
    let f = seed_realistic_db();
    let cfg = InjectionDbConfig {
        db_path: f.path().to_path_buf(),
        ..InjectionDbConfig::default()
    };
    let c = open_readonly(&cfg).expect("open");
    let rows = c.read_unresolved().expect("read");
    let seen: std::collections::HashSet<ChainType> = rows.iter().map(|r| r.chain_type).collect();
    assert!(seen.iter().any(|t| matches!(t, ChainType::Bug | ChainType::Trap | ChainType::Plan | ChainType::Pattern)));
}

#[test]
fn returned_rows_carry_typed_consent_enum() {
    let f = seed_realistic_db();
    let cfg = InjectionDbConfig {
        db_path: f.path().to_path_buf(),
        ..InjectionDbConfig::default()
    };
    let c = open_readonly(&cfg).expect("open");
    let rows = c.read_unresolved().expect("read");
    for r in &rows {
        // Forget rows are filtered at SQL level; remaining are Emit/Store.
        assert!(matches!(r.consent, ConsentLevel::Emit | ConsentLevel::Store));
    }
}

// ---- Error paths (2) -----------------------------------------------------

#[test]
fn missing_path_returns_database_open_failed() {
    let cfg = InjectionDbConfig {
        db_path: "/tmp/definitely-missing-9f3e7a1b-injection.db".into(),
        ..InjectionDbConfig::default()
    };
    assert!(matches!(
        open_readonly(&cfg),
        Err(InjectionDbError::DatabaseOpenFailed { .. })
    ));
}

#[test]
fn db_path_exists_advisory_check() {
    let cfg = InjectionDbConfig::default();
    let live_present = db_path_exists(&cfg);
    eprintln!("M3-LIVE injection.db advisory: present={live_present}");
}

// ---- Live habitat injection.db advisory (1) -----------------------------

#[test]
fn live_injection_db_advisory_read_unresolved_smoke() {
    let cfg = InjectionDbConfig::default();
    if !db_path_exists(&cfg) {
        eprintln!(
            "M3-LIVE advisory: {} not present; skipping live smoke",
            cfg.db_path.display()
        );
        return;
    }
    let c = open_readonly(&cfg).expect("live open");
    let rows = c.read_unresolved().expect("live read");
    eprintln!("M3-LIVE read_unresolved: {} rows", rows.len());
    for r in rows.iter().take(3) {
        eprintln!(
            "  id={} type={} consent={} reinf={} label={}",
            r.id,
            r.chain_type.as_str(),
            r.consent.as_str(),
            r.reinforcement_count,
            r.label,
        );
    }
    let n = c.count_unresolved().expect("count");
    eprintln!("M3-LIVE count_unresolved: {n}");
}
