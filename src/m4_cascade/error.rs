//! Error types for m4 cascade_correlator.

use thiserror::Error;

/// Failure modes for the cascade correlator.
#[derive(Debug, Error)]
pub enum CascadeError {
    /// atuin schema-version probe found unexpected shape.
    #[error("atuin schema drift: expected ns timestamps, got {0}")]
    AtuinSchemaDrift(String),
    /// SQLite/rusqlite error reading atuin.
    #[error("atuin io: {0}")]
    AtuinIo(#[from] rusqlite::Error),
    /// Empty input passed to a function that requires ≥1 row.
    #[error("empty input for cluster id derivation")]
    EmptyInput,
}

#[cfg(test)]
mod tests {
    use super::CascadeError;

    #[test]
    fn schema_drift_display_carries_reason() {
        assert!(CascadeError::AtuinSchemaDrift("microseconds".into())
            .to_string()
            .contains("microseconds"));
    }

    #[test]
    fn empty_input_display_is_stable() {
        assert!(CascadeError::EmptyInput
            .to_string()
            .contains("empty input"));
    }

    #[test]
    fn implements_std_error_and_send_sync_static() {
        fn assert_error<T: std::error::Error>() {}
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_error::<CascadeError>();
        assert_send_sync_static::<CascadeError>();
    }
}
