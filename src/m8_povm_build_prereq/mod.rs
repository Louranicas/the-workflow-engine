//! `m8_povm_build_prereq` — POVM CR-2 calibration-gate machinery (dormant).
//!
//! See [m8 spec](../../../ai_specs/modules/cluster-D/m8_povm_build_prereq.md)
//! for the canonical contract. m8 provides the gate **machinery**: band
//! classification ([`cfg::classify`]), the live mirror probe
//! ([`health::probe_band`]), the `[0.05, 0.15]` magnitude-weighted band
//! constants, and the [`BuildPrereqError`] / [`RuntimeBandError`] types.
//!
//! # Status — machinery present, gate intentionally NOT wired
//!
//! The m8 spec describes a compile-time gate: any code reading POVM
//! `learning_health` annotated `#[cfg(povm_calibrated)]`, with a
//! `#[cfg(not(povm_calibrated))]` `compile_error!` tombstone. Hardening
//! Fleet W2 (2026-05-21) corrected this module doc, which previously
//! asserted the gate was implemented and that the process "refuses to
//! run" out-of-band — **neither was true**:
//!
//! - **No compile-time tombstones exist**, and there is nothing for them
//!   to protect: workflow-trace has **no POVM read sites**. It routes
//!   substrate feedback to stcortex only, per the m42 stcortex-only pivot
//!   (`ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`).
//! - **No runtime startup-refusal is wired.** [`health::probe_band`] is a
//!   tested helper but is not called from `wf-crystallise` / `wf-dispatch`
//!   `main()` (both are Cluster-D-Day-1 stubs).
//!
//! `build.rs` still emits `cargo:rustc-cfg=povm_calibrated` when
//! `POVM_CR2_DEPLOYED=1`, so the compile-time path is *ready* should a
//! POVM coupling ever be reintroduced; it is a `rustc-cfg` flag, never a
//! Cargo `[features]` flag (F7 / AP-V7-09 defense).
//!
//! **Open architecture decision (node 0.A):** post-m42-pivot m8's POVM
//! gate is dormant by construction. Whether to keep the machinery dormant,
//! wire it if a POVM coupling returns, or retire m8 is a decision for the
//! charter owner — it is NOT a hardening fix and is deliberately left open.

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
