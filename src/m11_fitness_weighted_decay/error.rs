//! Error types for m11 fitness-weighted decay.
//!
//! Per m11 spec § 4 — error-band assignment per
//! `ERROR_TAXONOMY.md § E5xxx` (Lifecycle errors):
//!
//! - [`DecayError::OutOfRange`]         = `E5001`
//! - [`DecayError::ClockUnavailable`]   = `E5002`
//! - [`DecayError::PathwayReadFailed`]  = `E5004`
//! - [`DecayError::CycleAborted`]       = `E5099`
//!
//! Read failures from the source modules (m7, m14, m42) are surfaced through
//! the generic [`DecayError::PathwayReadFailed`] variant; [`DecayError::CycleAborted`]
//! covers an aborted decay cycle.

use thiserror::Error;

/// Failure modes for m11 fitness-weighted decay.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DecayError {
    /// A computed or supplied [`super::formula::DecayFactor`] value was
    /// non-finite or outside `[0.0, 1.0]`. This is a programming error
    /// (newtype invariant violation); it should never reach production
    /// because [`super::formula::compute_decay_factor`] always clamps.
    #[error("DecayFactor out of range: {value} (expected finite in [0.0, 1.0])")]
    OutOfRange {
        /// The offending value.
        value: f64,
    },

    /// `SystemTime::now()` returned `Err` (system pre-UNIX-epoch or fault).
    /// m11 SKIPS the consolidation cycle rather than treating timestamps as
    /// zero — that would silently poison decay calculations with the
    /// F-POVM-07 pattern.
    #[error(
        "clock returned None (system pre-epoch or fault); skipping decay cycle \
         to avoid silent zero-timestamp poisoning (F-POVM-07 pattern)"
    )]
    ClockUnavailable,

    /// Reading a workflow's substrate pathway weight failed. Until m7 / m14
    /// / m42 ship, this is the generic variant for any reader failure.
    #[error("stcortex pathway read failed for pathway {pathway_id}: {reason}")]
    PathwayReadFailed {
        /// Pathway id whose weight read failed.
        pathway_id: String,
        /// Free-text reason from the upstream reader.
        reason: String,
    },

    /// The consolidation cycle aborted before completion — for any reason
    /// other than the four above. Carries operational context for the
    /// supervisor / Watcher.
    #[error("consolidation cycle aborted: {reason}")]
    CycleAborted {
        /// Free-text reason.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::DecayError;

    #[test]
    fn out_of_range_display_names_value() {
        let err = DecayError::OutOfRange { value: 1.5 };
        let msg = err.to_string();
        assert!(msg.contains("1.5"));
        assert!(msg.contains("[0.0, 1.0]"));
    }

    #[test]
    fn clock_unavailable_display_names_skip_and_f_povm_07() {
        let msg = DecayError::ClockUnavailable.to_string();
        assert!(msg.contains("skipping decay cycle"));
        assert!(msg.contains("F-POVM-07"));
    }

    #[test]
    fn pathway_read_failed_display_names_id_and_reason() {
        let err = DecayError::PathwayReadFailed {
            pathway_id: "pw_001".into(),
            reason: "connection refused".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("pw_001"));
        assert!(msg.contains("connection refused"));
    }

    #[test]
    fn cycle_aborted_display_names_reason() {
        let err = DecayError::CycleAborted {
            reason: "supervisor cancel".into(),
        };
        assert!(err.to_string().contains("supervisor cancel"));
    }

    #[test]
    fn implements_std_error_and_send_sync_static() {
        fn assert_error<T: std::error::Error>() {}
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_error::<DecayError>();
        assert_send_sync_static::<DecayError>();
    }
}
