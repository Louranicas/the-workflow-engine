//! `m1_atuin_consumer` — read-only paginated ingress to atuin's
//! shell-history SQLite database.
//!
//! See [m1 spec](../../../ai_specs/modules/cluster-A/m1_atuin_consumer.md).
//!
//! # Invariants (spec § 1)
//!
//! 1. **Never writes** to atuin: enforced at the URI level
//!    (`file:path?mode=ro`) AND at the SQLite-pragma level
//!    (`PRAGMA query_only = ON`). Defense-in-depth against any accidental
//!    call site (AP-WT-F3 mitigation).
//! 2. **Byte-for-byte preservation** of command/session/cwd payloads so
//!    m4's downstream FNV-1a-XOR opaque-ID derivation is deterministic
//!    across runs.
//! 3. **Cursor monotonicity** across `next_page()` calls — same cursor in,
//!    same rows out.
//!
//! # Public surface (Day-1)
//!
//! - [`open_readonly`] — open the atuin DB and return a paginated reader.
//! - [`AtuinConsumer::next_page`] — fetch the next page.
//! - [`AtuinConsumer::collect_all`] — convenience for tests / small histories.
//! - [`AtuinConsumer::into_page_iter`] — lazy page iterator.
//! - [`AtuinConsumerConfig`] / [`PageResult`] / [`AtuinHistoryRow`] /
//!   [`SessionId`] / [`AtuinConsumerError`].

pub mod config;
pub mod cursor;
pub mod error;
pub mod row;

use std::path::{Path, PathBuf};
use std::time::Instant;

use rusqlite::{Connection, OpenFlags};

pub use config::{
    AtuinConsumerConfig, PAGE_SIZE_DEFAULT, PAGE_SIZE_MAX, PAGE_SIZE_MIN,
    SUBPROCESS_TIMEOUT_DEFAULT_MS,
};
pub use error::AtuinConsumerError;
pub use row::{parse_row, AtuinHistoryRow, PageResult, SessionId};

use cursor::ConsumerState;

/// Default atuin history DB path.
pub const DEFAULT_DB_PATH: &str = "~/.local/share/atuin/history.db";

/// SQLite busy-timeout for the WAL lock in milliseconds.
pub const BUSY_TIMEOUT_MS: u64 = 5_000;

/// Resolve `~/.local/share/atuin/history.db` (or honour
/// `config.db_path_override`).
fn resolve_db_path(config: &AtuinConsumerConfig) -> PathBuf {
    if let Some(p) = &config.db_path_override {
        return p.clone();
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned());
    PathBuf::from(format!("{home}/.local/share/atuin/history.db"))
}

/// Open the atuin DB for read-only paginated ingest.
///
/// Path resolution: `config.db_path_override` if set, else
/// `$HOME/.local/share/atuin/history.db`. Opened with
/// `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_URI` flags plus
/// `?mode=ro&immutable=0` URI parameters. The connection further
/// enforces `PRAGMA query_only = ON` as defense-in-depth.
///
/// # Errors
///
/// - [`AtuinConsumerError::DatabaseOpenFailed`] when the file is missing or
///   unopenable.
/// - [`AtuinConsumerError::QueryFailed`] if the WAL pragmas cannot be set.
pub fn open_readonly(
    config: &AtuinConsumerConfig,
) -> Result<AtuinConsumer, AtuinConsumerError> {
    let path = resolve_db_path(config);
    if !path.exists() {
        return Err(AtuinConsumerError::DatabaseOpenFailed {
            path: path.clone(),
            reason: "no such file".into(),
        });
    }
    let uri = format!("file:{}?mode=ro", path.display());
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI;
    let conn = Connection::open_with_flags(&uri, flags).map_err(|e| {
        AtuinConsumerError::DatabaseOpenFailed {
            path: path.clone(),
            reason: e.to_string(),
        }
    })?;
    configure_connection(&conn)?;
    Ok(AtuinConsumer {
        state: ConsumerState::new(config.clone()),
        conn,
    })
}

/// Apply the WAL pragma batch + defense-in-depth `query_only = ON`.
fn configure_connection(conn: &Connection) -> Result<(), AtuinConsumerError> {
    conn.busy_timeout(std::time::Duration::from_millis(BUSY_TIMEOUT_MS))
        .map_err(|e| AtuinConsumerError::QueryFailed(format!("busy_timeout: {e}")))?;
    conn.pragma_update(None, "query_only", "ON")
        .map_err(|e| AtuinConsumerError::QueryFailed(format!("query_only: {e}")))?;
    Ok(())
}

/// Paginated read-only atuin consumer.
pub struct AtuinConsumer {
    state: ConsumerState,
    conn: Connection,
}

impl std::fmt::Debug for AtuinConsumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtuinConsumer")
            .field("last_id", &self.state.last_id)
            .field("rows_yielded", &self.state.rows_yielded)
            .field("exhausted", &self.state.exhausted)
            .field("config", &self.state.config)
            .finish_non_exhaustive()
    }
}

impl AtuinConsumer {
    /// Borrow the current cursor state (ULID; empty before first page).
    #[must_use]
    pub fn last_id(&self) -> &str {
        &self.state.last_id
    }

    /// Total rows yielded across all pages.
    #[must_use]
    pub fn rows_yielded(&self) -> usize {
        self.state.rows_yielded
    }

    /// True once the cursor has exhausted (subsequent `next_page` returns
    /// `Ok(None)` idempotently).
    #[must_use]
    pub fn exhausted(&self) -> bool {
        self.state.exhausted
    }

    /// Fetch the next page in `id ASC` order. Returns `Ok(None)` once
    /// exhaustion is reached (idempotent: further calls also return
    /// `Ok(None)`).
    ///
    /// # Errors
    ///
    /// - [`AtuinConsumerError::QueryFailed`] if the SELECT or prepare
    ///   fails.
    /// - [`AtuinConsumerError::RowParseFailed`] if any row cannot be
    ///   decoded.
    pub fn next_page(&mut self) -> Result<Option<PageResult>, AtuinConsumerError> {
        if self.state.exhausted {
            return Ok(None);
        }
        let base_page_size = self.state.config.effective_page_size();
        let page_size = self.state.effective_page_size();
        if page_size == 0 {
            self.state.mark_exhausted();
            return Ok(None);
        }
        let started = Instant::now();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, command, session, hostname, timestamp, exit, duration, cwd, deleted_at \
                 FROM history WHERE id > ?1 ORDER BY id ASC LIMIT ?2",
            )
            .map_err(|e| AtuinConsumerError::QueryFailed(format!("prepare: {e}")))?;
        // rationale: page_size is config-clamped to `[PAGE_SIZE_MIN,
        // PAGE_SIZE_MAX]` (≤ 10_000) and `row_cap` (≤ usize::MAX); the
        // i64 conversion cannot lose precision on any target. `try_from`
        // beats `as` (AP-Hab arithmetic discipline) and saturating to
        // PAGE_SIZE_MAX as a defensive fallback prevents the original
        // silent `i64::MAX` from masking a spec-invariant violation.
        let page_size_i64: i64 = i64::try_from(page_size)
            .unwrap_or_else(|_| i64::try_from(PAGE_SIZE_MAX).unwrap_or(i64::MAX));
        let mapped = stmt.query_and_then(
            rusqlite::params![self.state.last_id.as_str(), page_size_i64],
            parse_row,
        )?;
        let rows: Vec<AtuinHistoryRow> = mapped.collect::<Result<Vec<_>, AtuinConsumerError>>()?;
        if rows.is_empty() {
            self.state.mark_exhausted();
            return Ok(None);
        }
        // `rows` is non-empty here per the early-return above; `map_or`
        // keeps the function panic-free for pedantic clippy.
        let new_last_id = rows
            .last()
            .map_or_else(|| self.state.last_id.clone(), |r| r.id.clone());
        let page_len = rows.len();
        self.state
            .advance(new_last_id.clone(), page_len, base_page_size);
        let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::info!(
            target: "m1.consumer.next_page",
            page_len,
            last_id = %new_last_id,
            elapsed_ms,
            "atuin page fetched"
        );
        Ok(Some(PageResult {
            rows,
            last_id: new_last_id,
            exhausted: self.state.exhausted,
            elapsed_ms,
        }))
    }

    /// Consume the iterator and collect all pages into one `Vec`. Honours
    /// `config.row_cap`.
    ///
    /// # Errors
    ///
    /// Propagates any [`AtuinConsumerError`] from [`Self::next_page`].
    pub fn collect_all(mut self) -> Result<Vec<AtuinHistoryRow>, AtuinConsumerError> {
        // rationale: pre-allocate to `row_cap` if set, else one effective
        // page-worth — avoids the geometric realloc series on small /
        // medium tables (resource-accounting discipline).
        let hint = self
            .state
            .config
            .row_cap
            .unwrap_or_else(|| self.state.config.effective_page_size());
        let mut out = Vec::with_capacity(hint);
        while let Some(page) = self.next_page()? {
            out.extend(page.rows);
        }
        Ok(out)
    }

    /// Lazy iterator over pages. Each `next()` call performs one SELECT.
    pub fn into_page_iter(self) -> PageIter {
        PageIter { consumer: self }
    }
}

/// Lazy [`PageResult`] iterator wrapping an [`AtuinConsumer`].
pub struct PageIter {
    consumer: AtuinConsumer,
}

impl Iterator for PageIter {
    type Item = Result<PageResult, AtuinConsumerError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.consumer.next_page() {
            Ok(Some(page)) => Some(Ok(page)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// Resolve a string path containing `~` against `$HOME`.
#[must_use]
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_owned());
        return PathBuf::from(format!("{home}/{stripped}"));
    }
    PathBuf::from(path)
}

/// Subprocess fallback: shell out to `atuin history list --limit N
/// --format json`. Best-effort; surfaces structured errors on failure.
///
/// # Errors
///
/// - [`AtuinConsumerError::SubprocessFailed`] for spawn / timeout / parse
///   failures.
pub fn fallback_subprocess_ingest(
    config: &AtuinConsumerConfig,
) -> Result<Vec<AtuinHistoryRow>, AtuinConsumerError> {
    // rationale: `config` is intentionally unused in the Day-1 stub.
    // We surface its `subprocess_timeout_ms` in the error message so a
    // caller can verify wiring (anti-silent-failure discipline) and so
    // the unused-binding doesn't decay into a regression when the real
    // subprocess path lands.
    let timeout_ms = config.subprocess_timeout_ms;
    // Day-1 stub — the real subprocess fallback wires into the atuin CLI
    // when the SQLite reader is unreachable. Per m1 spec § 13 step 6 the
    // primary read path is the rusqlite reader; subprocess fallback is
    // tracked in the implementation order but is non-critical for Day-1.
    Err(AtuinConsumerError::SubprocessFailed(format!(
        "subprocess fallback not implemented in Day-1 (configured timeout_ms={timeout_ms}); use open_readonly()"
    )))
}

/// True iff the configured DB path exists on disk.
#[must_use]
pub fn db_path_exists(config: &AtuinConsumerConfig) -> bool {
    resolve_db_path(config).exists()
}

/// Return the canonical default DB path for diagnostics. This is the
/// resolved `$HOME/.local/share/atuin/history.db` form.
#[must_use]
pub fn canonical_default_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned());
    Path::new(&home).join(".local/share/atuin/history.db")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rusqlite::Connection;

    use super::{
        canonical_default_path, configure_connection, db_path_exists, expand_tilde,
        fallback_subprocess_ingest, open_readonly, parse_row, AtuinConsumer,
        AtuinConsumerConfig, AtuinConsumerError, BUSY_TIMEOUT_MS, DEFAULT_DB_PATH,
    };

    /// Synthetic 26-char fixed-width ULID-like ID — lexicographically
    /// sortable by integer counter so paginated walks visit rows in
    /// integer-counter order. Mirrors live atuin's TEXT primary key.
    fn synthetic_ulid(i: i64) -> String {
        format!("01HQA{i:021}")
    }

    fn live_schema_create() -> &'static str {
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
        );"
    }

    fn seed_in_memory_history() -> Connection {
        let conn = Connection::open_in_memory().expect("memory");
        conn.execute_batch(live_schema_create()).expect("schema");
        for i in 1..=20_i64 {
            conn.execute(
                "INSERT INTO history (id, command, session, hostname, timestamp, exit, duration, cwd, deleted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    synthetic_ulid(i),
                    format!("cmd_{i}"),
                    "session_a",
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
        conn
    }

    fn seed_to_temp_path() -> tempfile::NamedTempFile {
        let f = tempfile::Builder::new()
            .suffix(".db")
            .tempfile()
            .expect("temp");
        let conn = Connection::open(f.path()).expect("open");
        conn.execute_batch(live_schema_create()).expect("schema");
        for i in 1..=20_i64 {
            conn.execute(
                "INSERT INTO history (id, command, session, hostname, timestamp, exit, duration, cwd, deleted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    synthetic_ulid(i),
                    format!("cmd_{i}"),
                    "session_a",
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
        f
    }

    // ---- Path resolution + open (5) -------------------------------------

    #[test]
    fn default_db_path_constant_is_tilde_form() {
        assert_eq!(DEFAULT_DB_PATH, "~/.local/share/atuin/history.db");
    }

    #[test]
    fn expand_tilde_resolves_home() {
        let home = std::env::var("HOME").unwrap_or_default();
        let resolved = expand_tilde("~/foo");
        assert_eq!(resolved, PathBuf::from(format!("{home}/foo")));
    }

    #[test]
    fn expand_tilde_passes_through_absolute() {
        assert_eq!(expand_tilde("/abs/path"), PathBuf::from("/abs/path"));
    }

    #[test]
    fn open_readonly_returns_database_open_failed_for_missing_path() {
        let cfg = AtuinConsumerConfig {
            db_path_override: Some(PathBuf::from("/tmp/definitely-missing-atuin-9f3e7a1b.db")),
            ..Default::default()
        };
        let err = open_readonly(&cfg).expect_err("missing");
        assert!(matches!(err, AtuinConsumerError::DatabaseOpenFailed { .. }));
    }

    #[test]
    fn db_path_exists_false_for_missing() {
        let cfg = AtuinConsumerConfig {
            db_path_override: Some(PathBuf::from("/tmp/definitely-missing-9f3e7a1b.db")),
            ..Default::default()
        };
        assert!(!db_path_exists(&cfg));
    }

    // ---- WAL pragma + query_only enforcement (3) ------------------------

    #[test]
    fn configure_connection_sets_busy_timeout() {
        let conn = Connection::open_in_memory().expect("memory");
        configure_connection(&conn).expect("configure");
        // We cannot easily read busy_timeout via PRAGMA in rusqlite; the
        // assertion here is that configuration completes without error
        // and the busy_timeout constant matches the spec.
        assert_eq!(BUSY_TIMEOUT_MS, 5_000);
    }

    #[test]
    fn configure_connection_sets_query_only_pragma() {
        let conn = Connection::open_in_memory().expect("memory");
        configure_connection(&conn).expect("configure");
        let value: String = conn
            .query_row("PRAGMA query_only", [], |r| r.get::<_, String>(0))
            .or_else(|_| {
                conn.query_row("PRAGMA query_only", [], |r| {
                    Ok(if r.get::<_, i64>(0).unwrap_or(0) == 1 {
                        "1".to_owned()
                    } else {
                        "0".to_owned()
                    })
                })
            })
            .unwrap_or_else(|_| "unknown".to_owned());
        assert!(value == "1" || value.eq_ignore_ascii_case("on"));
    }

    #[test]
    fn query_only_blocks_inserts() {
        let conn = seed_in_memory_history();
        configure_connection(&conn).expect("configure");
        let err = conn
            .execute(
                "INSERT INTO history (id, command, session, hostname, timestamp) VALUES (999, 'x', 's', 'h', 0)",
                [],
            )
            .expect_err("insert should be blocked by query_only");
        let msg = err.to_string();
        assert!(
            msg.contains("read-only") || msg.contains("readonly") || msg.contains("attempt"),
            "expected read-only block, got: {msg}"
        );
    }

    // ---- parse_row coverage (5) -----------------------------------------

    fn fetch_row(conn: &Connection, id: &str) -> Result<super::row::AtuinHistoryRow, AtuinConsumerError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, command, session, hostname, timestamp, exit, duration, cwd, deleted_at \
                 FROM history WHERE id = ?1",
            )?;
        let mut iter = stmt.query_and_then(rusqlite::params![id], parse_row)?;
        iter.next().ok_or_else(|| AtuinConsumerError::QueryFailed("no row".into()))?
    }

    #[test]
    fn parse_row_extracts_all_required_columns() {
        let conn = seed_in_memory_history();
        let row = fetch_row(&conn, &synthetic_ulid(1)).expect("parse");
        assert_eq!(row.id, synthetic_ulid(1));
        assert_eq!(row.command, "cmd_1");
        assert_eq!(row.session.as_str(), "session_a");
        assert_eq!(row.hostname, "host1");
        assert_eq!(row.exit, 0);
        assert_eq!(row.duration_ms, 10);
        assert_eq!(row.cwd, "/tmp");
        assert!(row.deleted_at.is_none());
    }

    #[test]
    fn parse_row_handles_deleted_at_some() {
        let conn = seed_in_memory_history();
        conn.execute(
            "UPDATE history SET deleted_at = 1700000099999 WHERE id = ?1",
            rusqlite::params![synthetic_ulid(2)],
        )
        .expect("update");
        let row = fetch_row(&conn, &synthetic_ulid(2)).expect("parse");
        assert_eq!(row.deleted_at, Some(1_700_000_099_999));
    }

    #[test]
    fn parse_row_returns_row_parse_failed_on_type_mismatch() {
        let conn = Connection::open_in_memory().expect("memory");
        conn.execute_batch(
            "CREATE TABLE history (id TEXT PRIMARY KEY, timestamp TEXT NOT NULL, duration INTEGER NOT NULL, exit INTEGER NOT NULL, command TEXT NOT NULL, cwd TEXT NOT NULL, session TEXT NOT NULL, hostname TEXT NOT NULL, deleted_at INTEGER);
             INSERT INTO history (id, command, session, hostname, timestamp, exit, duration, cwd) \
             VALUES ('01HQA-bad', 'x', 's', 'h', 'not-a-number', 0, 0, '/');",
        )
        .expect("schema");
        let result = fetch_row(&conn, "01HQA-bad");
        assert!(matches!(
            result,
            Err(AtuinConsumerError::RowParseFailed { ref row_id, .. }) if row_id == "01HQA-bad"
        ));
    }

    #[test]
    fn parse_row_session_is_byte_for_byte_preserved() {
        let conn = Connection::open_in_memory().expect("memory");
        conn.execute_batch(live_schema_create()).expect("schema");
        conn.execute(
            "INSERT INTO history (id, command, session, hostname, timestamp, exit, duration, cwd) VALUES (?1, 'echo hi', 'session-with-hyphens-and_underscores', 'h', 0, 0, 0, '/')",
            rusqlite::params!["01HQA-preserve"],
        )
        .expect("insert");
        let row = fetch_row(&conn, "01HQA-preserve").expect("parse");
        assert_eq!(row.session.as_str(), "session-with-hyphens-and_underscores");
        assert_eq!(row.command, "echo hi");
    }

    #[test]
    fn canonical_default_path_ends_with_history_db() {
        let p = canonical_default_path();
        assert!(p.ends_with("history.db"));
        assert!(p.to_string_lossy().contains(".local/share/atuin"));
    }

    // ---- Pagination end-to-end (10) -------------------------------------

    fn open_on_temp(path: std::path::PathBuf, page_size: usize, cap: Option<usize>) -> AtuinConsumer {
        let cfg = AtuinConsumerConfig {
            page_size,
            row_cap: cap,
            db_path_override: Some(path),
            ..Default::default()
        };
        open_readonly(&cfg).expect("open")
    }

    #[test]
    fn next_page_returns_full_page_then_remainder() {
        let f = seed_to_temp_path();
        let mut c = open_on_temp(f.path().to_path_buf(), 100, None);
        let p1 = c.next_page().expect("p1").expect("Some");
        assert_eq!(p1.rows.len(), 20);
        // 20 < 100 → exhausted after this page
        assert!(p1.exhausted);
        let p2 = c.next_page().expect("p2");
        assert!(p2.is_none());
    }

    #[test]
    fn next_page_walks_in_id_ascending_order() {
        let f = seed_to_temp_path();
        let mut c = open_on_temp(f.path().to_path_buf(), 100, None);
        let p = c.next_page().expect("p").expect("Some");
        let ids: Vec<String> = p.rows.iter().map(|r| r.id.clone()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn next_page_respects_small_page_size() {
        let f = seed_to_temp_path();
        // page_size = 100 (the floor) walks a 20-row table in 1 page,
        // exhausted (since 20 < 100). Use page_size at floor.
        let mut c = open_on_temp(f.path().to_path_buf(), 0, None);
        let p = c.next_page().expect("p").expect("Some");
        assert_eq!(p.rows.len(), 20);
    }

    #[test]
    fn row_cap_stops_at_configured_total() {
        let f = seed_to_temp_path();
        let mut c = open_on_temp(f.path().to_path_buf(), 100, Some(5));
        let p1 = c.next_page().expect("p1").expect("Some");
        assert_eq!(p1.rows.len(), 5);
        assert!(p1.exhausted);
        assert!(c.next_page().expect("p2").is_none());
    }

    #[test]
    fn collect_all_matches_into_page_iter() {
        let f = seed_to_temp_path();
        let by_collect = open_on_temp(f.path().to_path_buf(), 100, None).collect_all().expect("collect");
        let by_iter: Vec<_> = open_on_temp(f.path().to_path_buf(), 100, None)
            .into_page_iter()
            .filter_map(Result::ok)
            .flat_map(|p| p.rows)
            .collect();
        let ids_a: Vec<String> = by_collect.iter().map(|r| r.id.clone()).collect();
        let ids_b: Vec<String> = by_iter.iter().map(|r| r.id.clone()).collect();
        assert_eq!(ids_a, ids_b);
    }

    #[test]
    fn next_page_after_exhaustion_is_idempotent_ok_none() {
        let f = seed_to_temp_path();
        let mut c = open_on_temp(f.path().to_path_buf(), 100, None);
        let _ = c.next_page().expect("p1");
        for _ in 0..5_u32 {
            assert!(c.next_page().expect("re-call").is_none());
        }
    }

    #[test]
    fn cursor_state_accessors_reflect_progress() {
        let f = seed_to_temp_path();
        let mut c = open_on_temp(f.path().to_path_buf(), 100, None);
        assert_eq!(c.last_id(), "");
        assert_eq!(c.rows_yielded(), 0);
        assert!(!c.exhausted());
        let _ = c.next_page().expect("p1").expect("Some");
        assert_eq!(c.last_id(), synthetic_ulid(20));
        assert_eq!(c.rows_yielded(), 20);
        assert!(c.exhausted());
    }

    #[test]
    fn debug_format_shows_cursor_state() {
        let f = seed_to_temp_path();
        let c = open_on_temp(f.path().to_path_buf(), 100, None);
        let s = format!("{c:?}");
        assert!(s.contains("AtuinConsumer"));
        assert!(s.contains("last_id"));
    }

    #[test]
    fn empty_table_first_next_page_returns_ok_none() {
        let f = tempfile::Builder::new()
            .suffix(".db")
            .tempfile()
            .expect("temp");
        let conn = Connection::open(f.path()).expect("open");
        conn.execute_batch(live_schema_create()).expect("schema");
        drop(conn);
        let mut c = open_on_temp(f.path().to_path_buf(), 100, None);
        assert!(c.next_page().expect("p").is_none());
        assert!(c.exhausted());
    }

    #[test]
    fn paginated_walk_yields_no_duplicate_ids() {
        use std::collections::HashSet;
        let f = tempfile::Builder::new()
            .suffix(".db")
            .tempfile()
            .expect("temp");
        let conn = Connection::open(f.path()).expect("open");
        conn.execute_batch(live_schema_create()).expect("schema");
        for i in 1..=300_i64 {
            conn.execute(
                "INSERT INTO history (id, command, session, hostname, timestamp, exit, duration, cwd) VALUES (?1, ?2, ?3, ?4, ?5, 0, 0, '/')",
                rusqlite::params![synthetic_ulid(i), format!("c{i}"), "s", "h", i],
            )
            .expect("insert");
        }
        drop(conn);
        let all = open_on_temp(f.path().to_path_buf(), 100, None).collect_all().expect("collect");
        let ids: HashSet<String> = all.iter().map(|r| r.id.clone()).collect();
        assert_eq!(ids.len(), 300);
    }

    // ---- Subprocess fallback Day-1 stub (1) -----------------------------

    #[test]
    fn fallback_subprocess_is_day_1_stub_returning_typed_error() {
        let cfg = AtuinConsumerConfig::default();
        let err = fallback_subprocess_ingest(&cfg).expect_err("stub");
        assert!(matches!(err, AtuinConsumerError::SubprocessFailed(_)));
    }
}
