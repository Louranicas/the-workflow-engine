//! Error types for m5 battern_step_record.

use thiserror::Error;

/// Failure modes for the Battern step recorder.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BatternError {
    /// SQLite/rusqlite error reading atuin.
    #[error("atuin io: {0}")]
    AtuinIo(#[from] rusqlite::Error),
    /// Regex heuristic table compile failed at `BatternStepRecord::new`.
    #[error("regex compile: {0}")]
    RegexCompile(#[from] regex::Error),
}

#[cfg(test)]
mod tests {
    use super::BatternError;

    #[test]
    fn implements_std_error_and_send_sync_static() {
        fn assert_error<T: std::error::Error>() {}
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_error::<BatternError>();
        assert_send_sync_static::<BatternError>();
    }
}
