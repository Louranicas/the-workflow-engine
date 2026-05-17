//! `m8_povm_build_prereq` — compile-time CR-2 gate + runtime mirror probe.
//!
//! See [m8 spec](../../../ai_specs/modules/cluster-D/m8_povm_build_prereq.md)
//! for the canonical contract. The intellectual contribution is the
//! **placement** of the gate at compile-time rather than runtime: any code
//! that reads POVM `learning_health` data must be annotated
//! `#[cfg(povm_calibrated)]`; the corresponding
//! `#[cfg(not(povm_calibrated))]` tombstone emits a `compile_error!` so the
//! read path cannot reach a binary unless the CR-2 deployment marker is
//! present.
//!
//! # Activation paths
//!
//! 1. **Build-time:** `POVM_CR2_DEPLOYED=1` in the build env causes
//!    `build.rs` to emit `cargo:rustc-cfg=povm_calibrated`. This is the only
//!    way to activate `cfg(povm_calibrated)`; it is NOT a Cargo
//!    `[features]` flag (F7 / AP-V7-09 defense).
//! 2. **Runtime mirror:** [`health::probe_band`] performs the same
//!    band-classification at startup against
//!    `${POVM_HEALTH_URL:-http://127.0.0.1:8125/learning_health}`. If the
//!    live POVM is outside the `[0.05, 0.15]` magnitude-weighted band the
//!    process refuses to run.

pub mod cfg;
pub mod error;
pub mod health;

pub use cfg::{
    classify, BandClassification, POVM_LH_BAND_HIGH, POVM_LH_BAND_LOW, POVM_LH_EDGE_TOLERANCE,
};
pub use error::{BuildPrereqError, RuntimeBandError};
pub use health::{probe_band, HealthClient};

#[cfg(test)]
mod tests {
    use super::{
        classify, BandClassification, POVM_LH_BAND_HIGH, POVM_LH_BAND_LOW, POVM_LH_EDGE_TOLERANCE,
    };

    #[test]
    fn reexports_classify_function() {
        assert_eq!(classify(0.10), BandClassification::InBand);
    }

    #[test]
    fn reexports_band_constants() {
        // Constants must match the Hebbian v3 Phase 1 reconciliation thresholds
        // verbatim. If these drift, m8's reason-for-being is undermined.
        // `black_box` defeats const-propagation so clippy::assertions_on_constants
        // (pedantic) doesn't fire — the test is meaningful at the type+value
        // level even when the expression evaluates at compile time.
        let low = std::hint::black_box(POVM_LH_BAND_LOW);
        let high = std::hint::black_box(POVM_LH_BAND_HIGH);
        let tol = std::hint::black_box(POVM_LH_EDGE_TOLERANCE);
        assert!((low - 0.05).abs() < f64::EPSILON);
        assert!((high - 0.15).abs() < f64::EPSILON);
        assert!((tol - 0.01).abs() < f64::EPSILON);
    }

    #[test]
    fn band_constants_form_valid_interval() {
        let low = std::hint::black_box(POVM_LH_BAND_LOW);
        let high = std::hint::black_box(POVM_LH_BAND_HIGH);
        let tol = std::hint::black_box(POVM_LH_EDGE_TOLERANCE);
        assert!(low < high);
        assert!(tol < high - low);
    }
}
