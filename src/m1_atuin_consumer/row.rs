//! `AtuinHistoryRow` — verbatim representation of one atuin `history` row.
//!
//! Per m1 spec § 1 invariants: m1 preserves byte-for-byte
//! command/session/cwd payloads. Any pre-folding here would silently
//! corrupt CC-1 cascade correlation downstream. The `SessionId` newtype
//! prevents primitive-obsession over the session column.
//!
//! # Live-schema discovery (S1002211 amendment to m1 spec § 2)
//!
//! The live atuin schema at `~/.local/share/atuin/history.db` differs
//! from the spec's paper assumption:
//!
//! - `id` is `TEXT` (ULID), not `INTEGER` — ULIDs are lexicographically
//!   sortable so `WHERE id > ? ORDER BY id ASC` still works as a cursor.
//! - `timestamp`, `duration`, `exit`, `command`, `cwd`, `session`,
//!   `hostname` are all `NOT NULL` — no `Option<T>` wrappers needed.
//! - `deleted_at INTEGER` is the only nullable column.
//!
//! This module matches the live schema, not the paper spec. A v1.4 spec
//! amendment is owed.

use rusqlite::Row;

use super::error::AtuinConsumerError;

/// Validated atuin session identifier. Newtype enforces "this is a session
/// id, not just a random string" at the type-system layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    /// Construct without validation. atuin's `session` column is a free-form
    /// string; m1's responsibility is to preserve it verbatim.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One normalised atuin history row, matching the live schema as of
/// 2026-05-17.
#[derive(Debug, Clone)]
pub struct AtuinHistoryRow {
    /// ULID primary key (lexicographically sortable by issue time).
    pub id: String,
    /// The shell command verbatim.
    pub command: String,
    /// Session identifier (newtype-wrapped).
    pub session: SessionId,
    /// Host where the command was issued.
    pub hostname: String,
    /// Wall-clock time of command issue (ms since UNIX epoch).
    pub timestamp_ms: i64,
    /// Process exit code (atuin stores as `NOT NULL`).
    pub exit: i32,
    /// Command duration in milliseconds (atuin stores as `NOT NULL`).
    pub duration_ms: i64,
    /// Working directory at issue time (atuin stores as `NOT NULL`).
    pub cwd: String,
    /// Wall-clock time at which the row was marked deleted, if ever. The
    /// only nullable column in atuin's schema.
    pub deleted_at: Option<i64>,
}

/// Result of a single page fetch.
#[derive(Debug, Clone)]
pub struct PageResult {
    /// Rows in this page (`<= page_size`).
    pub rows: Vec<AtuinHistoryRow>,
    /// Maximum `id` (ULID) in the page; the cursor advances to this value
    /// for the next page.
    pub last_id: String,
    /// True when the page contained `< page_size` rows OR the configured
    /// `row_cap` has been reached.
    pub exhausted: bool,
    /// Wall-clock time taken to fetch + parse the page.
    pub elapsed_ms: u64,
}

/// Parse one rusqlite row into an [`AtuinHistoryRow`]. Column order MUST
/// match the SELECT in `next_page`:
///
/// ```text
/// SELECT id, command, session, hostname, timestamp, exit, duration, cwd, deleted_at
/// FROM history WHERE id > ?1 ORDER BY id ASC LIMIT ?2
/// ```
///
/// # Errors
///
/// [`AtuinConsumerError::RowParseFailed`] on any column type-mismatch. The
/// ULID id (column 0) is preserved when parseable so operators can grep
/// atuin directly for the offending row.
pub fn parse_row(row: &Row<'_>) -> Result<AtuinHistoryRow, AtuinConsumerError> {
    let id: String = row
        .get(0)
        .map_err(|e| AtuinConsumerError::RowParseFailed {
            row_id: String::new(),
            reason: format!("id column: {e}"),
        })?;
    let command: String = row
        .get(1)
        .map_err(|e| AtuinConsumerError::RowParseFailed {
            row_id: id.clone(),
            reason: format!("command column: {e}"),
        })?;
    let session: String = row
        .get(2)
        .map_err(|e| AtuinConsumerError::RowParseFailed {
            row_id: id.clone(),
            reason: format!("session column: {e}"),
        })?;
    let hostname: String = row
        .get(3)
        .map_err(|e| AtuinConsumerError::RowParseFailed {
            row_id: id.clone(),
            reason: format!("hostname column: {e}"),
        })?;
    let timestamp_ms: i64 = row
        .get(4)
        .map_err(|e| AtuinConsumerError::RowParseFailed {
            row_id: id.clone(),
            reason: format!("timestamp column: {e}"),
        })?;
    let exit: i32 = row
        .get(5)
        .map_err(|e| AtuinConsumerError::RowParseFailed {
            row_id: id.clone(),
            reason: format!("exit column: {e}"),
        })?;
    let duration_ms: i64 = row
        .get(6)
        .map_err(|e| AtuinConsumerError::RowParseFailed {
            row_id: id.clone(),
            reason: format!("duration column: {e}"),
        })?;
    let cwd: String = row
        .get(7)
        .map_err(|e| AtuinConsumerError::RowParseFailed {
            row_id: id.clone(),
            reason: format!("cwd column: {e}"),
        })?;
    let deleted_at: Option<i64> =
        row.get(8)
            .map_err(|e| AtuinConsumerError::RowParseFailed {
                row_id: id.clone(),
                reason: format!("deleted_at column: {e}"),
            })?;
    Ok(AtuinHistoryRow {
        id,
        command,
        session: SessionId::new(session),
        hostname,
        timestamp_ms,
        exit,
        duration_ms,
        cwd,
        deleted_at,
    })
}

#[cfg(test)]
mod tests {
    use super::{AtuinHistoryRow, PageResult, SessionId};

    #[test]
    fn session_id_roundtrip() {
        let s = SessionId::new("abc123");
        assert_eq!(s.as_str(), "abc123");
    }

    #[test]
    fn session_id_display_emits_inner() {
        let s = SessionId::new("xyz");
        assert_eq!(format!("{s}"), "xyz");
    }

    #[test]
    fn session_id_implements_eq_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(SessionId::new("a"));
        set.insert(SessionId::new("a"));
        set.insert(SessionId::new("b"));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn atuin_history_row_clone_preserves_all_fields() {
        let r = AtuinHistoryRow {
            id: "01HQA-ulid-1".into(),
            command: "ls".into(),
            session: SessionId::new("s1"),
            hostname: "h".into(),
            timestamp_ms: 1_700_000_000_000,
            exit: 0,
            duration_ms: 10,
            cwd: "/tmp".into(),
            deleted_at: None,
        };
        let c = r.clone();
        assert_eq!(c.id, r.id);
        assert_eq!(c.command, r.command);
        assert_eq!(c.session, r.session);
        assert_eq!(c.cwd, r.cwd);
        assert_eq!(c.deleted_at, r.deleted_at);
    }

    #[test]
    fn atuin_history_row_supports_deleted_at_some() {
        let r = AtuinHistoryRow {
            id: "01HQA-ulid-2".into(),
            command: "x".into(),
            session: SessionId::new("s"),
            hostname: "h".into(),
            timestamp_ms: 1,
            exit: 0,
            duration_ms: 0,
            cwd: "/".into(),
            deleted_at: Some(1_700_000_000_001),
        };
        assert!(r.deleted_at.is_some());
    }

    #[test]
    fn page_result_carries_ulid_cursor_and_exhausted_flag() {
        let p = PageResult {
            rows: vec![],
            last_id: "01HQA-ulid-99".into(),
            exhausted: true,
            elapsed_ms: 1,
        };
        assert_eq!(p.last_id, "01HQA-ulid-99");
        assert!(p.exhausted);
    }
}
