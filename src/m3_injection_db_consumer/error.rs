//! Error types for the m3 injection.db consumer.
//!
//! Per m3 spec § 4 — error-band assignment per `ERROR_TAXONOMY.md § E3xxx`:
//!
//! - [`InjectionDbError::DatabaseOpenFailed`]
//! - [`InjectionDbError::QueryFailed`]
//! - [`InjectionDbError::RowParseFailed`]
//! - [`InjectionDbError::UnknownChainType`] — closed-set parse rejection.
//! - [`InjectionDbError::UnknownConsent`]   — closed-set parse rejection.

use std::path::PathBuf;

use thiserror::Error;

/// Failure modes for the m3 injection.db consumer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum InjectionDbError {
    /// Could not open the injection.db SQLite file at `path`.
    #[error("database open failed at {path}: {reason}")]
    DatabaseOpenFailed {
        /// Filesystem path attempted.
        path: PathBuf,
        /// Underlying reason.
        reason: String,
    },

    /// A SELECT, prepare, or PRAGMA failed.
    #[error("query failed: {0}")]
    QueryFailed(String),

    /// A row could not be parsed — column type mismatch, unexpected NULL,
    /// or `u32` overflow.
    #[error("row parse error at id={row_id}: {reason}")]
    RowParseFailed {
        /// The injection.db row primary key of the offending row, or `-1`
        /// if the id itself failed to parse.
        row_id: i64,
        /// Free-text reason.
        reason: String,
    },

    /// The `chain_type` column held a value not in the closed set
    /// (`bug` / `trap` / `plan` / `pattern`).
    #[error("unknown chain_type value: {0}")]
    UnknownChainType(String),

    /// The `consent` column held a value not in the closed set
    /// (`Emit` / `Store` / `Forget`).
    #[error("unknown consent value: {0}")]
    UnknownConsent(String),
}

impl From<rusqlite::Error> for InjectionDbError {
    fn from(e: rusqlite::Error) -> Self {
        Self::QueryFailed(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::InjectionDbError;

    #[test]
    fn database_open_failed_display_names_path() {
        let err = InjectionDbError::DatabaseOpenFailed {
            path: PathBuf::from("/x.db"),
            reason: "no such file".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("/x.db"));
        assert!(msg.contains("no such file"));
    }

    #[test]
    fn query_failed_display_carries_reason() {
        assert!(InjectionDbError::QueryFailed("syntax".into())
            .to_string()
            .contains("syntax"));
    }

    #[test]
    fn row_parse_failed_display_names_id_and_reason() {
        let msg = InjectionDbError::RowParseFailed {
            row_id: 42,
            reason: "overflow".into(),
        }
        .to_string();
        assert!(msg.contains("id=42"));
        assert!(msg.contains("overflow"));
    }

    #[test]
    fn unknown_chain_type_preserves_bad_value_for_diagnostics() {
        assert!(InjectionDbError::UnknownChainType("BUG".into())
            .to_string()
            .contains("BUG"));
    }

    #[test]
    fn unknown_consent_preserves_bad_value_for_diagnostics() {
        assert!(InjectionDbError::UnknownConsent("yes".into())
            .to_string()
            .contains("yes"));
    }

    #[test]
    fn implements_std_error_and_send_sync_static() {
        fn assert_error<T: std::error::Error>() {}
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_error::<InjectionDbError>();
        assert_send_sync_static::<InjectionDbError>();
    }
}
