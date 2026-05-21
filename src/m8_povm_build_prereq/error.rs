//! Error types for the m8 compile-time gate + runtime mirror.
//!
//! Per m8 spec § 4 — error-band assignment per `ERROR_TAXONOMY.md § E3xxx`
//! Trust-layer violations:
//!
//! - `BuildPrereqError::Cr2MarkerAbsent` = `E3001`
//! - `BuildPrereqError::OutOfBand`       = `E3002`
//! - `BuildPrereqError::ProbeFailed`     = `E3003`
//! - `RuntimeBandError::StartupRefused`  = `E3010`
//!
//! Every variant Display text names: (a) commit SHA `e2a8ed3` of the
//! povm-v2 CR-2 fix, (b) the `POVM_CR2_DEPLOYED` environment variable, and
//! (c) the canonical reference doc (Hebbian v3 reconciliation). Operators
//! can recover from these errors without log-hunting.

use thiserror::Error;

use super::cfg::BandClassification;

/// Build-time + probe-time error class for the m8 gate.
///
/// Used by both `build.rs` (via path-include if env-only path is rejected
/// in a future amendment) and the runtime mirror [`super::health::probe_band`].
#[derive(Debug, Error)]
pub enum BuildPrereqError {
    /// `POVM_CR2_DEPLOYED` is unset or not equal to `"1"`. Set the marker
    /// after confirming the magnitude-weighted `learning_health` formula
    /// is live (povm-v2 commit `e2a8ed3`).
    #[error(
        "POVM CR-2 not verified. Set POVM_CR2_DEPLOYED=1 after confirming \
         povm-v2 commit e2a8ed3 is live and the magnitude-weighted learning_health \
         formula is active. See: ~/projects/claude_code/Hebbian Deployment Plan v3 \
         — Post-CR-2 Threshold Reconciliation.md"
    )]
    Cr2MarkerAbsent,

    /// The POVM probe returned a value outside the Phase-1
    /// `[POVM_LH_BAND_LOW, POVM_LH_BAND_HIGH]` band.
    #[error(
        "POVM probe at {url} returned learning_health={value} \
         (band [{low}, {high}]); classification={classification:?}"
    )]
    OutOfBand {
        /// URL probed (typically `${POVM_HEALTH_URL:-http://127.0.0.1:8125/learning_health}`).
        url: String,
        /// The probe value returned (post-CR-2 magnitude-weighted).
        value: f64,
        /// Band floor at probe time (snapshot — not necessarily the const,
        /// in case a future amendment makes the band runtime-configurable).
        low: f64,
        /// Band ceiling at probe time.
        high: f64,
        /// Concrete classification (`BelowFloor` / `AboveCeiling` / `Nan`).
        classification: BandClassification,
    },

    /// The POVM probe could not reach the configured endpoint.
    #[error("POVM probe at {url} unreachable: {source}")]
    ProbeFailed {
        /// URL probed.
        url: String,
        /// Underlying transport error.
        #[source]
        source: reqwest::Error,
    },
}

/// Runtime-only startup-refusal error. Distinct from [`BuildPrereqError`] so
/// downstream code can handle the build-vs-startup distinction by exhaustive
/// `match` rather than string matching.
///
/// Per m8 spec § 10 this error is *intended* to drive an `exit(78)`
/// (`EX_CONFIG`) startup-refusal. That wiring is NOT present: the m8 gate
/// is dormant post-m42-pivot (workflow-trace has no POVM read sites — see
/// the [`m8_povm_build_prereq`](crate::m8_povm_build_prereq) module doc).
/// The type and its `EX_CONFIG` convention are retained as ready
/// machinery; no production path constructs or acts on it today.
#[derive(Debug, Error)]
pub enum RuntimeBandError {
    /// POVM `learning_health` is outside the magnitude-weighted band at
    /// startup. Refusing to run against an uncalibrated substrate.
    #[error(
        "startup refused: POVM learning_health={value} outside band [{low}, {high}] \
         — refusing to run against uncalibrated substrate (POVM CR-2 commit e2a8ed3; \
         POVM_CR2_DEPLOYED=1)"
    )]
    StartupRefused {
        /// Probe value at startup.
        value: f64,
        /// Band floor.
        low: f64,
        /// Band ceiling.
        high: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::{BuildPrereqError, RuntimeBandError};
    use crate::m8_povm_build_prereq::cfg::BandClassification;

    // ---- F-Unit (error-variant Display) ---------------------------------

    #[test]
    fn cr2_marker_absent_display_contains_commit_sha() {
        // F-Contract test for m8 spec § 6: the message text MUST contain the
        // literal commit SHA `e2a8ed3` so operators can grep for it.
        let msg = BuildPrereqError::Cr2MarkerAbsent.to_string();
        assert!(
            msg.contains("e2a8ed3"),
            "Cr2MarkerAbsent message missing commit SHA: {msg}"
        );
    }

    #[test]
    fn cr2_marker_absent_display_contains_env_var() {
        let msg = BuildPrereqError::Cr2MarkerAbsent.to_string();
        assert!(
            msg.contains("POVM_CR2_DEPLOYED=1"),
            "Cr2MarkerAbsent message missing env var: {msg}"
        );
    }

    #[test]
    fn cr2_marker_absent_display_contains_reference_doc() {
        let msg = BuildPrereqError::Cr2MarkerAbsent.to_string();
        assert!(
            msg.contains("Hebbian Deployment Plan v3"),
            "Cr2MarkerAbsent message missing reference doc: {msg}"
        );
    }

    #[test]
    fn out_of_band_display_contains_url_and_value() {
        let err = BuildPrereqError::OutOfBand {
            url: "http://localhost:8125/learning_health".into(),
            value: 0.9114,
            low: 0.05,
            high: 0.15,
            classification: BandClassification::AboveCeiling,
        };
        let msg = err.to_string();
        assert!(msg.contains("http://localhost:8125/learning_health"));
        assert!(msg.contains("0.9114"));
        assert!(msg.contains("AboveCeiling"));
    }

    #[test]
    fn out_of_band_carries_classification() {
        let err = BuildPrereqError::OutOfBand {
            url: "x".into(),
            value: 0.02,
            low: 0.05,
            high: 0.15,
            classification: BandClassification::BelowFloor,
        };
        match err {
            BuildPrereqError::OutOfBand { classification, .. } => {
                assert_eq!(classification, BandClassification::BelowFloor);
            }
            other => panic!("expected OutOfBand, got {other:?}"),
        }
    }

    #[test]
    fn startup_refused_display_contains_value_and_band() {
        let err = RuntimeBandError::StartupRefused {
            value: 0.92,
            low: 0.05,
            high: 0.15,
        };
        let msg = err.to_string();
        assert!(msg.contains("0.92"));
        assert!(msg.contains("0.05"));
        assert!(msg.contains("0.15"));
        assert!(msg.contains("refusing to run"));
    }

    #[test]
    fn startup_refused_display_names_commit_and_env() {
        // Operator recovery: the runtime-refusal error must name both the
        // commit SHA + env var so a tail of logs is self-sufficient.
        let err = RuntimeBandError::StartupRefused {
            value: 0.92,
            low: 0.05,
            high: 0.15,
        };
        let msg = err.to_string();
        assert!(msg.contains("e2a8ed3"));
        assert!(msg.contains("POVM_CR2_DEPLOYED=1"));
    }

    #[test]
    fn build_prereq_error_is_send_sync_static() {
        // Required to thread errors across async boundaries in m42 emitter.
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_send_sync_static::<BuildPrereqError>();
        assert_send_sync_static::<RuntimeBandError>();
    }

    #[test]
    fn errors_implement_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<BuildPrereqError>();
        assert_error::<RuntimeBandError>();
    }
}
