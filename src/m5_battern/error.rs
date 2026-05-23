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

    // rationale: §15 D31 — keep + test (construct each). The `#[from]
    // rusqlite::Error` derive yields the `AtuinIo` variant; a real
    // rusqlite failure must round-trip through `?` into that variant
    // and the `to_string()` must surface the expected "atuin io:" prefix.
    #[test]
    fn rusqlite_error_converts_into_atuin_io_variant() {
        // Open a connection to a non-existent file in a non-existent dir;
        // rusqlite::Error::SqliteFailure on cannot-open is reliable.
        let result: Result<rusqlite::Connection, rusqlite::Error> =
            rusqlite::Connection::open("/nonexistent-dir-x4f7a/missing.db");
        let rusqlite_err = result.expect_err("open-from-nonexistent-dir must fail");
        let err: BatternError = rusqlite_err.into();
        assert!(
            matches!(err, BatternError::AtuinIo(_)),
            "rusqlite::Error must map to BatternError::AtuinIo"
        );
        assert!(
            err.to_string().contains("atuin io:"),
            "AtuinIo Display must contain its prefix; got {err}"
        );
    }

    // rationale: §15 D31 — keep + test (construct each). The `#[from]
    // regex::Error` derive yields the `RegexCompile` variant; a real
    // regex compile failure (built at runtime so clippy's `invalid_regex`
    // lint doesn't object) must round-trip through `?` into that variant.
    #[test]
    fn regex_error_converts_into_regex_compile_variant() {
        // Build the malformed pattern at runtime so the regex literal is
        // not statically analysable by clippy::invalid_regex.
        let bad_pattern: String = format!("{}unbalanced", '(');
        let result: Result<regex::Regex, regex::Error> = regex::Regex::new(&bad_pattern);
        let regex_err = result.expect_err("unbalanced regex must fail to compile");
        let err: BatternError = regex_err.into();
        assert!(
            matches!(err, BatternError::RegexCompile(_)),
            "regex::Error must map to BatternError::RegexCompile"
        );
        assert!(
            err.to_string().contains("regex compile:"),
            "RegexCompile Display must contain its prefix; got {err}"
        );
    }
}
