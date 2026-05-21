//! Error types for m7 workflow_runs.

use thiserror::Error;

/// Failure modes for the workflow_runs central hub.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WorkflowError {
    /// Generic SQLite failure (prepare / execute / column extraction).
    #[error("sqlite error: {0}")]
    Sqlite(String),
    /// Migration step `step` failed.
    #[error("schema migration failed at step {step}: {source}")]
    Migration {
        /// Migration index (1-based).
        step: u32,
        /// Underlying rusqlite error.
        #[source]
        source: rusqlite::Error,
    },
    /// Row id was not found (used by callers that need NotFound semantics).
    #[error("row {id} not found in workflow_runs")]
    RowNotFound {
        /// The id queried.
        id: i64,
    },
    /// Outcome string did not match the CHECK constraint set.
    #[error("invalid outcome '{0}' — must be one of ok|fail|abort|unknown")]
    InvalidOutcome(String),
    /// JSON patch failure when merging consumer_inputs.
    #[error("consumer_inputs JSON patch failed: {0}")]
    JsonPatch(String),
    /// Connection-open failure.
    #[error("connection acquisition failed: {0}")]
    Connection(String),
}

impl From<rusqlite::Error> for WorkflowError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e.to_string())
    }
}

impl From<serde_json::Error> for WorkflowError {
    fn from(e: serde_json::Error) -> Self {
        Self::JsonPatch(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::WorkflowError;

    #[test]
    fn invalid_outcome_carries_string() {
        let err = WorkflowError::InvalidOutcome("BAD".into());
        assert!(err.to_string().contains("BAD"));
    }

    #[test]
    fn row_not_found_carries_id() {
        let err = WorkflowError::RowNotFound { id: 42 };
        assert!(err.to_string().contains("42"));
    }

    #[test]
    fn implements_std_error_and_send_sync_static() {
        fn assert_error<T: std::error::Error>() {}
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_error::<WorkflowError>();
        assert_send_sync_static::<WorkflowError>();
    }
}
