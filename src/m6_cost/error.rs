//! Error types for m6 context_cost.

use thiserror::Error;

/// Failure modes for the context-cost recorder.
#[derive(Debug, Error)]
pub enum ContextCostError {
    /// Both the live stcortex `:3000` endpoint AND the offline JSON
    /// snapshot are unreachable. m6 SKIPS baseline writes (per
    /// CLAUDE.md memory row 8 — never silently falls back to POVM).
    #[error("stcortex substrate unreachable: live :3000 + snapshot both failed")]
    SubstrateUnreachable,
    /// Snapshot file parse failed.
    #[error("stcortex snapshot parse: {0}")]
    SnapshotParse(#[from] serde_json::Error),
    /// I/O error reading snapshot.
    #[error("stcortex io: {0}")]
    StcortexIo(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::ContextCostError;

    #[test]
    fn substrate_unreachable_display_names_both_paths() {
        let msg = ContextCostError::SubstrateUnreachable.to_string();
        assert!(msg.contains("live :3000"));
        assert!(msg.contains("snapshot"));
    }

    #[test]
    fn implements_std_error_and_send_sync_static() {
        fn assert_error<T: std::error::Error>() {}
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_error::<ContextCostError>();
        assert_send_sync_static::<ContextCostError>();
    }
}
