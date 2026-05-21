//! Error types for the m1 atuin shell-history consumer.
//!
//! Per m1 spec § 2: typed taxonomy, no `Box<dyn Error>`. Every variant
//! carries structured fields so callers can match programmatically rather
//! than parsing display strings.

use std::path::PathBuf;

use thiserror::Error;

/// Failure modes for m1 atuin consumer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AtuinConsumerError {
    /// Could not open the atuin SQLite DB at `path`.
    #[error("database open failed at {path}: {reason}")]
    DatabaseOpenFailed {
        /// Filesystem path attempted.
        path: PathBuf,
        /// Underlying reason.
        reason: String,
    },

    /// SQLite busy-timeout exceeded — concurrent writer holding WAL lock
    /// longer than configured.
    #[error("WAL busy-timeout exceeded after {timeout_ms}ms")]
    BusyTimeout {
        /// Configured `busy_timeout` in milliseconds.
        timeout_ms: u64,
    },

    /// A SELECT or PRAGMA failed at runtime.
    #[error("query failed: {0}")]
    QueryFailed(String),

    /// `atuin history list` subprocess fallback failed.
    #[error("subprocess fallback failed: {0}")]
    SubprocessFailed(String),

    /// A row could not be parsed — column type mismatch or unexpected NULL.
    #[error("row parse error at id={row_id:?}: {reason}")]
    RowParseFailed {
        /// The atuin ULID of the offending row, or empty if id itself
        /// failed to parse.
        row_id: String,
        /// Free-text reason.
        reason: String,
    },
}

impl From<rusqlite::Error> for AtuinConsumerError {
    fn from(e: rusqlite::Error) -> Self {
        Self::QueryFailed(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::AtuinConsumerError;

    #[test]
    fn open_failed_display_names_path_and_reason() {
        let err = AtuinConsumerError::DatabaseOpenFailed {
            path: PathBuf::from("/x.db"),
            reason: "no such file".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("/x.db"));
        assert!(msg.contains("no such file"));
    }

    #[test]
    fn busy_timeout_display_names_timeout() {
        assert!(AtuinConsumerError::BusyTimeout { timeout_ms: 5000 }
            .to_string()
            .contains("5000"));
    }

    #[test]
    fn query_failed_display_carries_reason() {
        assert!(AtuinConsumerError::QueryFailed("syntax error".into())
            .to_string()
            .contains("syntax error"));
    }

    #[test]
    fn subprocess_failed_display_carries_reason() {
        assert!(
            AtuinConsumerError::SubprocessFailed("exit 1".into())
                .to_string()
                .contains("exit 1")
        );
    }

    #[test]
    fn row_parse_failed_display_names_id_and_reason() {
        let msg = AtuinConsumerError::RowParseFailed {
            row_id: "01HQA-ulid-42".into(),
            reason: "expected String, got INTEGER".into(),
        }
        .to_string();
        assert!(msg.contains("01HQA-ulid-42"));
        assert!(msg.contains("expected String, got INTEGER"));
    }

    #[test]
    fn implements_std_error_and_send_sync_static() {
        fn assert_error<T: std::error::Error>() {}
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_error::<AtuinConsumerError>();
        assert_send_sync_static::<AtuinConsumerError>();
    }
}
