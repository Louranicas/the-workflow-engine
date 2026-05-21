//! Error types for the m10 Ember CI gate.
//!
//! Per m10 spec § 4 — error-band assignment per
//! `ERROR_TAXONOMY.md § E3xxx` (Trust-layer violations):
//!
//! - [`EmberGateError::GateFailed`]      = `E3201`
//! - [`EmberGateError::AllowlistRead`]   = `E3202`
//! - [`EmberGateError::AllowlistParse`]  = `E3203`
//! - [`EmberGateError::RubricMissing`]   = `E3204`

use std::io;

use thiserror::Error;

/// Failure modes for the m10 Ember CI gate.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EmberGateError {
    /// One or more registered user-facing strings failed the rubric scoring.
    /// Carries the rejection / held-as-fail counts so operators can scan the
    /// stderr trail for the matching `EMBER-REJECT` / `EMBER-HELD(W3-fail)`
    /// detail lines.
    #[error(
        "ember gate failed: {rejected} string(s) rejected, {held} string(s) \
         held-as-fail (W3 flag active; D-C hybrid CI-FAIL+allowlist)"
    )]
    GateFailed {
        /// Count of `EmberStatus::Rejected` verdicts.
        rejected: usize,
        /// Count of `EmberStatus::Held` verdicts NOT covered by an unexpired
        /// allowlist row.
        held: usize,
    },

    /// The allowlist TSV file at `path` could not be read.
    #[error("allowlist TSV at {path} unreadable: {source}")]
    AllowlistRead {
        /// Filesystem path attempted.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },

    /// The allowlist TSV file at `path` was readable but malformed at the
    /// named line. `reason` carries the parser's diagnostic message.
    #[error("allowlist TSV at {path} malformed at line {line}: {reason}")]
    AllowlistParse {
        /// Filesystem path of the offending file.
        path: String,
        /// 1-indexed line number where the parser gave up.
        line: usize,
        /// Free-text reason from the parser.
        reason: String,
    },

    /// The canonical Ember rubric reference document is missing at the
    /// expected `path`. The heuristic implementation is the test surface
    /// (per spec § 7); this error fires only if an explicit
    /// `RubricMissing`-aware check elects to load the canonical file for
    /// snapshot comparison.
    #[error("rubric reference missing at {path}: cannot score without canonical rubric")]
    RubricMissing {
        /// Filesystem path attempted.
        path: String,
    },
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::EmberGateError;

    #[test]
    fn gate_failed_display_carries_counts_and_hybrid_marker() {
        let err = EmberGateError::GateFailed {
            rejected: 2,
            held: 3,
        };
        let msg = err.to_string();
        assert!(msg.contains("2 string(s) rejected"));
        assert!(msg.contains("3 string(s) held-as-fail"));
        assert!(msg.contains("hybrid CI-FAIL+allowlist"));
        assert!(msg.contains("D-C"));
    }

    #[test]
    fn allowlist_read_display_carries_path_and_source() {
        let err = EmberGateError::AllowlistRead {
            path: "tests/missing.tsv".into(),
            source: io::Error::new(io::ErrorKind::NotFound, "no such file"),
        };
        let msg = err.to_string();
        assert!(msg.contains("tests/missing.tsv"));
        assert!(msg.contains("no such file"));
    }

    #[test]
    fn allowlist_parse_display_carries_path_line_reason() {
        let err = EmberGateError::AllowlistParse {
            path: "tests/x.tsv".into(),
            line: 7,
            reason: "expected 4 tab-separated fields".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("tests/x.tsv"));
        assert!(msg.contains("line 7"));
        assert!(msg.contains("expected 4 tab-separated fields"));
    }

    #[test]
    fn rubric_missing_display_carries_path() {
        let err = EmberGateError::RubricMissing {
            path: "/home/x/Ember Rubric.md".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("/home/x/Ember Rubric.md"));
    }

    #[test]
    fn implements_std_error_and_send_sync_static() {
        fn assert_error<T: std::error::Error>() {}
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_error::<EmberGateError>();
        assert_send_sync_static::<EmberGateError>();
    }
}
