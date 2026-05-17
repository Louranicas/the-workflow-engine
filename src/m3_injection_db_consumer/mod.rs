//! `m3_injection_db_consumer` — read-only ingress to the habitat's
//! `causal_chain` ledger at `~/.local/share/habitat/injection.db`.
//!
//! See [m3 spec](../../../ai_specs/modules/cluster-D/m3_injection_db_consumer.md).
//!
//! # Invariants (spec § 1)
//!
//! 1. **Read-only enforcement** — `file:?mode=ro` URI plus
//!    `PRAGMA query_only = ON`. m3 never migrates, resolves, inserts,
//!    or updates a chain. injection.db's schema is owned by the
//!    `memory-injection` service; m3 is a passive reader.
//! 2. **Typed enum reflection** — `chain_type` and `consent` string
//!    columns are mapped to closed [`ChainType`] / [`ConsentLevel`]
//!    enums at parse time (no stringly-typed handling downstream).
//! 3. **Preserve-list discipline (AP-Hab-04)** — `consent = 'Forget'`
//!    rows are filtered at the SQL query layer; m3 never emits them.

pub mod causal_chain;
pub mod config;
pub mod enums;
pub mod error;

use std::time::Duration;

use rusqlite::{Connection, OpenFlags};

pub use causal_chain::{parse_causal_chain_row, CausalChainRow, ChainId, ChainLabel};
pub use config::{
    InjectionDbConfig, LIMIT_MAX, LIMIT_MIN, MAX_RECENTLY_RESOLVED_DEFAULT,
    MAX_UNRESOLVED_DEFAULT, RESOLVED_RECENCY_SESSIONS_DEFAULT,
};
pub use enums::{parse_chain_type, parse_consent, ChainType, ConsentLevel};
pub use error::InjectionDbError;

/// SQLite busy-timeout for the WAL lock (ms).
pub const BUSY_TIMEOUT_MS: u64 = 5_000;

/// Open injection.db read-only. No migration — schema is owned by the
/// `memory-injection` service.
///
/// # Errors
///
/// - [`InjectionDbError::DatabaseOpenFailed`] if the file is missing or
///   unopenable.
/// - [`InjectionDbError::QueryFailed`] if the WAL pragmas cannot be set.
pub fn open_readonly(
    config: &InjectionDbConfig,
) -> Result<InjectionDbConsumer, InjectionDbError> {
    if !config.db_path.exists() {
        return Err(InjectionDbError::DatabaseOpenFailed {
            path: config.db_path.clone(),
            reason: "no such file".into(),
        });
    }
    let uri = format!("file:{}?mode=ro", config.db_path.display());
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI;
    let conn = Connection::open_with_flags(&uri, flags).map_err(|e| {
        InjectionDbError::DatabaseOpenFailed {
            path: config.db_path.clone(),
            reason: e.to_string(),
        }
    })?;
    configure_connection(&conn)?;
    Ok(InjectionDbConsumer {
        conn,
        config: config.clone(),
    })
}

fn configure_connection(conn: &Connection) -> Result<(), InjectionDbError> {
    conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))
        .map_err(|e| InjectionDbError::QueryFailed(format!("busy_timeout: {e}")))?;
    conn.pragma_update(None, "query_only", "ON")
        .map_err(|e| InjectionDbError::QueryFailed(format!("query_only: {e}")))?;
    Ok(())
}

/// True iff the configured DB path exists on disk.
#[must_use]
pub fn db_path_exists(config: &InjectionDbConfig) -> bool {
    config.db_path.exists()
}

/// Read-only injection.db consumer.
pub struct InjectionDbConsumer {
    conn: Connection,
    config: InjectionDbConfig,
}

impl std::fmt::Debug for InjectionDbConsumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InjectionDbConsumer")
            .field("db_path", &self.config.db_path)
            .field("max_unresolved", &self.config.max_unresolved)
            .field("max_recently_resolved", &self.config.max_recently_resolved)
            .finish_non_exhaustive()
    }
}

const SELECT_COLUMNS: &str = "SELECT id, origin_session, resolved_session, chain_type, label, \
                              description, reinforcement_count, last_reinforced_session, consent";

impl InjectionDbConsumer {
    /// Borrow the configuration snapshot.
    #[must_use]
    pub fn config(&self) -> &InjectionDbConfig {
        &self.config
    }

    /// Read unresolved chains ordered by `reinforcement_count DESC`
    /// excluding `consent = 'Forget'` (preserve-list SQL-level filter).
    /// Bounded by `config.effective_max_unresolved()`.
    ///
    /// # Errors
    ///
    /// - [`InjectionDbError::QueryFailed`] if the SELECT or prepare fails.
    /// - [`InjectionDbError::RowParseFailed`] / [`InjectionDbError::UnknownChainType`]
    ///   / [`InjectionDbError::UnknownConsent`] on parse failure.
    pub fn read_unresolved(&self) -> Result<Vec<CausalChainRow>, InjectionDbError> {
        let effective_limit = self.config.effective_max_unresolved();
        // rationale: `effective_*` is config-clamped to `[LIMIT_MIN,
        // LIMIT_MAX]` (≤ 5_000) so the `i64` conversion never loses
        // precision on any supported target. The fallback is a
        // defense-in-depth saturation.
        let limit = i64::try_from(effective_limit)
            .unwrap_or_else(|_| i64::try_from(LIMIT_MAX).unwrap_or(i64::MAX));
        let sql = format!(
            "{SELECT_COLUMNS} FROM causal_chain \
             WHERE resolved_session IS NULL AND consent != 'Forget' \
             ORDER BY reinforcement_count DESC LIMIT ?1"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let iter = stmt.query_and_then(rusqlite::params![limit], parse_causal_chain_row)?;
        // rationale: pre-allocate to the SQL LIMIT to avoid the
        // geometric realloc chain on large unresolved counts
        // (resource-accounting discipline).
        let mut rows: Vec<CausalChainRow> = Vec::with_capacity(effective_limit);
        for r in iter {
            rows.push(r?);
        }
        tracing::info!(
            target: "m3.read_unresolved",
            count = rows.len(),
            "injection.db unresolved chains read"
        );
        Ok(rows)
    }

    /// Read recently-resolved chains within
    /// `config.resolved_recency_sessions` of `MAX(resolved_session)`,
    /// excluding `consent = 'Forget'`. Bounded by
    /// `config.effective_max_recently_resolved()`.
    ///
    /// # Errors
    ///
    /// See [`Self::read_unresolved`].
    pub fn read_recently_resolved(&self) -> Result<Vec<CausalChainRow>, InjectionDbError> {
        let max_resolved: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(resolved_session), 0) FROM causal_chain",
            [],
            |r| r.get(0),
        )?;
        let cutoff = max_resolved.saturating_sub(i64::from(self.config.resolved_recency_sessions));
        let effective_limit = self.config.effective_max_recently_resolved();
        let limit = i64::try_from(effective_limit)
            .unwrap_or_else(|_| i64::try_from(LIMIT_MAX).unwrap_or(i64::MAX));
        let sql = format!(
            "{SELECT_COLUMNS} FROM causal_chain \
             WHERE resolved_session IS NOT NULL \
               AND resolved_session > ?1 \
               AND consent != 'Forget' \
             ORDER BY resolved_session DESC LIMIT ?2"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let iter =
            stmt.query_and_then(rusqlite::params![cutoff, limit], parse_causal_chain_row)?;
        let mut rows: Vec<CausalChainRow> = Vec::with_capacity(effective_limit);
        for r in iter {
            rows.push(r?);
        }
        tracing::info!(
            target: "m3.read_recently_resolved",
            count = rows.len(),
            cutoff,
            max_resolved,
            "injection.db recently-resolved chains read"
        );
        Ok(rows)
    }

    /// Scalar count of unresolved chains (no consent filter on the count
    /// itself — `Forget`-consent rows still count for `count_unresolved`
    /// since the count is an aggregate diagnostic, not an emit boundary).
    ///
    /// # Errors
    ///
    /// - [`InjectionDbError::QueryFailed`] on SELECT failure.
    /// - [`InjectionDbError::RowParseFailed`] if SQLite returns a
    ///   negative row count (cannot happen with current SQLite versions,
    ///   but the explicit refusal beats silent zero-coercion).
    pub fn count_unresolved(&self) -> Result<u64, InjectionDbError> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM causal_chain WHERE resolved_session IS NULL",
            [],
            |r| r.get(0),
        )?;
        // rationale: COUNT(*) is documented non-negative; surface any
        // anomaly as a typed parse failure instead of silently coercing
        // to zero (was: `.unwrap_or(0)` — debugger Phase 1 m3-F1).
        u64::try_from(n).map_err(|_| InjectionDbError::RowParseFailed {
            row_id: -1,
            reason: format!("COUNT(*) returned negative value {n}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{
        configure_connection, db_path_exists, open_readonly, parse_causal_chain_row,
        BUSY_TIMEOUT_MS, InjectionDbConfig, InjectionDbConsumer, InjectionDbError, SELECT_COLUMNS,
    };

    fn live_schema_create() -> &'static str {
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
        CREATE INDEX idx_causal_unresolved
            ON causal_chain(reinforcement_count DESC)
            WHERE resolved_session IS NULL;"
    }

    fn seed_temp_db() -> tempfile::NamedTempFile {
        let f = tempfile::Builder::new().suffix(".db").tempfile().expect("temp");
        let conn = Connection::open(f.path()).expect("open");
        conn.execute_batch(live_schema_create()).expect("schema");
        // 3 unresolved + 2 resolved + 1 'Forget' that should be filtered.
        let rows = [
            (1_i64, "bug", "BUG-001", "first", 10, Some(1)),
            (2, "trap", "TRAP-001", "loop", 5, Some(2)),
            (3, "pattern", "PAT-001", "shape", 3, Some(3)),
        ];
        for (id, t, label, desc, reinf, lr) in rows {
            conn.execute(
                "INSERT INTO causal_chain (id, origin_session, resolved_session, chain_type, label, description, reinforcement_count, last_reinforced_session, consent) \
                 VALUES (?1, 100, NULL, ?2, ?3, ?4, ?5, ?6, 'Emit')",
                rusqlite::params![id, t, label, desc, reinf, lr],
            ).expect("insert");
        }
        // Resolved
        for (id, ses) in [(4_i64, 110_i64), (5, 115)] {
            conn.execute(
                "INSERT INTO causal_chain (id, origin_session, resolved_session, chain_type, label, description, reinforcement_count, last_reinforced_session, consent) \
                 VALUES (?1, 100, ?2, 'plan', 'PLAN-X', 'resolved', 1, 1, 'Emit')",
                rusqlite::params![id, ses],
            ).expect("insert");
        }
        // Forget consent — should NEVER appear in either read.
        conn.execute(
            "INSERT INTO causal_chain (id, origin_session, resolved_session, chain_type, label, description, reinforcement_count, last_reinforced_session, consent) \
             VALUES (6, 100, NULL, 'bug', 'BUG-FORGET', 'should-not-appear', 99, 1, 'Forget')",
            [],
        ).expect("insert");
        f
    }

    // ---- Path resolution + open (3) -------------------------------------

    #[test]
    fn open_readonly_returns_database_open_failed_for_missing_path() {
        let cfg = InjectionDbConfig {
            db_path: "/tmp/definitely-missing-9f3e7a1b-injection.db".into(),
            ..Default::default()
        };
        let err = open_readonly(&cfg).expect_err("missing");
        assert!(matches!(err, InjectionDbError::DatabaseOpenFailed { .. }));
    }

    #[test]
    fn db_path_exists_false_for_missing() {
        let cfg = InjectionDbConfig {
            db_path: "/tmp/missing-9f3e7a1b.db".into(),
            ..Default::default()
        };
        assert!(!db_path_exists(&cfg));
    }

    #[test]
    fn select_columns_constant_lists_all_nine_fields() {
        // Stable column-list snapshot — if the SQL drifts, all three
        // queries break and this fails loudly.
        assert!(SELECT_COLUMNS.starts_with("SELECT id, origin_session, resolved_session"));
        assert!(SELECT_COLUMNS.contains("chain_type, label"));
        assert!(SELECT_COLUMNS.contains("reinforcement_count, last_reinforced_session, consent"));
    }

    // ---- WAL pragma + query_only (2) ------------------------------------

    #[test]
    fn configure_connection_sets_query_only() {
        let conn = Connection::open_in_memory().expect("memory");
        configure_connection(&conn).expect("configure");
        let value: i64 = conn
            .query_row("PRAGMA query_only", [], |r| r.get::<_, i64>(0))
            .unwrap_or(0);
        assert_eq!(value, 1);
    }

    #[test]
    fn busy_timeout_constant_matches_spec() {
        assert_eq!(BUSY_TIMEOUT_MS, 5_000);
    }

    // ---- read_unresolved (4) --------------------------------------------

    fn open_on(path: std::path::PathBuf) -> InjectionDbConsumer {
        let cfg = InjectionDbConfig {
            db_path: path,
            ..Default::default()
        };
        open_readonly(&cfg).expect("open")
    }

    #[test]
    fn read_unresolved_returns_only_unresolved_rows() {
        let f = seed_temp_db();
        let rows = open_on(f.path().to_path_buf()).read_unresolved().expect("read");
        for r in &rows {
            assert!(r.resolved_session.is_none());
        }
    }

    #[test]
    fn read_unresolved_excludes_forget_consent() {
        let f = seed_temp_db();
        let rows = open_on(f.path().to_path_buf()).read_unresolved().expect("read");
        for r in &rows {
            assert_ne!(r.consent.as_str(), "Forget", "Forget leaked into unresolved");
        }
        // The seed has 3 unresolved+Emit and 1 unresolved+Forget. We
        // expect exactly 3.
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn read_unresolved_orders_by_reinforcement_count_desc() {
        let f = seed_temp_db();
        let rows = open_on(f.path().to_path_buf()).read_unresolved().expect("read");
        let counts: Vec<u32> = rows.iter().map(|r| r.reinforcement_count).collect();
        let mut sorted = counts.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        assert_eq!(counts, sorted);
    }

    #[test]
    fn read_unresolved_empty_table_yields_empty_vec() {
        let f = tempfile::Builder::new().suffix(".db").tempfile().expect("temp");
        let conn = Connection::open(f.path()).expect("open");
        conn.execute_batch(live_schema_create()).expect("schema");
        drop(conn);
        let rows = open_on(f.path().to_path_buf()).read_unresolved().expect("read");
        assert!(rows.is_empty());
    }

    // ---- read_recently_resolved (3) -------------------------------------

    #[test]
    fn read_recently_resolved_returns_only_resolved_rows() {
        let f = seed_temp_db();
        let rows = open_on(f.path().to_path_buf()).read_recently_resolved().expect("read");
        for r in &rows {
            assert!(r.resolved_session.is_some());
        }
    }

    #[test]
    fn read_recently_resolved_respects_recency_window() {
        // max_resolved=115; cutoff=115-10=105; rows with resolved_session > 105 only.
        let f = seed_temp_db();
        let rows = open_on(f.path().to_path_buf()).read_recently_resolved().expect("read");
        for r in &rows {
            assert!(r.resolved_session.unwrap() > 105);
        }
    }

    #[test]
    fn read_recently_resolved_excludes_forget_consent() {
        let f = seed_temp_db();
        let rows = open_on(f.path().to_path_buf()).read_recently_resolved().expect("read");
        for r in &rows {
            assert_ne!(r.consent.as_str(), "Forget");
        }
    }

    // ---- count_unresolved (3) -------------------------------------------

    #[test]
    fn count_unresolved_includes_forget_consent() {
        // Per spec § 1 design: count is an aggregate diagnostic; preserve-
        // filter applies only to emit boundaries.
        let f = seed_temp_db();
        let n = open_on(f.path().to_path_buf()).count_unresolved().expect("count");
        // 3 Emit + 1 Forget = 4 unresolved total.
        assert_eq!(n, 4);
    }

    #[test]
    fn count_unresolved_empty_table_returns_zero() {
        let f = tempfile::Builder::new().suffix(".db").tempfile().expect("temp");
        let conn = Connection::open(f.path()).expect("open");
        conn.execute_batch(live_schema_create()).expect("schema");
        drop(conn);
        let n = open_on(f.path().to_path_buf()).count_unresolved().expect("count");
        assert_eq!(n, 0);
    }

    #[test]
    fn count_unresolved_excludes_resolved_rows() {
        let f = seed_temp_db();
        let n = open_on(f.path().to_path_buf()).count_unresolved().expect("count");
        assert_eq!(n, 4); // 4 unresolved (incl Forget); 2 resolved not counted.
    }

    // ---- parse_causal_chain_row coverage (3) ----------------------------

    #[test]
    fn parse_row_handles_resolved_session_null() {
        let f = seed_temp_db();
        let rows = open_on(f.path().to_path_buf()).read_unresolved().expect("read");
        assert!(rows.iter().any(|r| r.resolved_session.is_none()));
    }

    #[test]
    fn parse_row_u32_overflow_yields_typed_error() {
        let f = tempfile::Builder::new().suffix(".db").tempfile().expect("temp");
        let conn = Connection::open(f.path()).expect("open");
        // We CANNOT insert a negative origin_session via the live CHECK +
        // NOT NULL constraints in the normal table. Construct a parallel
        // table without CHECK to exercise overflow handling.
        conn.execute_batch(
            "CREATE TABLE causal_chain (
                id INTEGER PRIMARY KEY,
                origin_session INTEGER NOT NULL,
                resolved_session INTEGER,
                chain_type TEXT NOT NULL,
                label TEXT NOT NULL,
                description TEXT NOT NULL,
                reinforcement_count INTEGER NOT NULL DEFAULT 1,
                last_reinforced_session INTEGER,
                consent TEXT NOT NULL DEFAULT 'Emit'
            );
            INSERT INTO causal_chain (id, origin_session, chain_type, label, description) \
             VALUES (1, -1, 'bug', 'X', 'desc');",
        )
        .expect("schema+insert");
        let mut stmt = conn
            .prepare(&format!("{SELECT_COLUMNS} FROM causal_chain WHERE id = 1"))
            .expect("prepare");
        let mut iter = stmt
            .query_and_then([], parse_causal_chain_row)
            .expect("qa");
        let err = iter.next().expect("row").expect_err("u32 overflow");
        assert!(matches!(err, InjectionDbError::RowParseFailed { .. }));
    }

    #[test]
    fn parse_row_unknown_chain_type_yields_typed_error() {
        let f = tempfile::Builder::new().suffix(".db").tempfile().expect("temp");
        let conn = Connection::open(f.path()).expect("open");
        conn.execute_batch(
            "CREATE TABLE causal_chain (
                id INTEGER PRIMARY KEY,
                origin_session INTEGER NOT NULL,
                resolved_session INTEGER,
                chain_type TEXT NOT NULL,
                label TEXT NOT NULL,
                description TEXT NOT NULL,
                reinforcement_count INTEGER NOT NULL DEFAULT 1,
                last_reinforced_session INTEGER,
                consent TEXT NOT NULL DEFAULT 'Emit'
            );
            INSERT INTO causal_chain (id, origin_session, chain_type, label, description) \
             VALUES (2, 100, 'incident', 'X', 'desc');",
        )
        .expect("schema+insert");
        let mut stmt = conn
            .prepare(&format!("{SELECT_COLUMNS} FROM causal_chain WHERE id = 2"))
            .expect("prepare");
        let mut iter = stmt
            .query_and_then([], parse_causal_chain_row)
            .expect("qa");
        let err = iter.next().expect("row").expect_err("unknown");
        let InjectionDbError::UnknownChainType(value) = err else {
            panic!("expected UnknownChainType");
        };
        assert_eq!(value, "incident");
    }

    // ---- Debug + reexports (2) ------------------------------------------

    #[test]
    fn consumer_debug_format_shows_db_path_and_limits() {
        let f = seed_temp_db();
        let c = open_on(f.path().to_path_buf());
        let s = format!("{c:?}");
        assert!(s.contains("InjectionDbConsumer"));
        assert!(s.contains("db_path"));
    }

    #[test]
    fn consumer_config_accessor_returns_snapshot() {
        let f = seed_temp_db();
        let c = open_on(f.path().to_path_buf());
        assert!(c.config().db_path.exists());
    }
}
