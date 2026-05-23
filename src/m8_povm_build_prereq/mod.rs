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
//! **Architecture decision — RESOLVED (node 0.A, S1003733): KEEP-DORMANT.**
//! Post-m42-pivot m8's POVM gate is dormant by construction. The decision
//! is to retain the machinery as a dormant tripwire: the `build.rs`
//! `rustc-cfg` path is ready, so if a POVM coupling is ever reintroduced
//! the compile-time gate and band classification activate without new
//! design work. Retiring m8 was rejected — it is structurally the floor of
//! Cluster D's trust regime, the machinery is small and fully tested, and
//! deleting it would discard the ready-made tripwire. Wiring it now would
//! gate nothing (there are no POVM read sites). No further action required.

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

    // rationale: v0.1.1 R4 — M0 invariant lock for the KEEP-DORMANT
    // architectural decision (S1003733, Plan v2 §15 R4 + ADR
    // D-S1002127-01 m42 stcortex-only pivot).
    //
    // m8 is the floor of Cluster D's trust regime, but workflow-trace
    // routes substrate feedback to stcortex only — there are ZERO POVM
    // read sites in `src/`. The `build.rs` `rustc-cfg=povm_calibrated`
    // flag is emitted ONLY when `POVM_CR2_DEPLOYED=1` is set in the
    // build env. At M0 the cfg is OFF, the gate is dormant, and there
    // is nothing for it to protect.
    //
    // This test fails LOUDLY if someone sets `POVM_CR2_DEPLOYED=1`
    // without ratifying the re-introduction of a POVM coupling. If a
    // POVM read site is ever re-introduced in workflow-trace, the
    // ratification must update this assertion AND the m8 module
    // docstring's KEEP-DORMANT block.
    #[test]
    fn m0_dormant_invariant_povm_calibrated_cfg_is_off() {
        let calibrated = cfg!(povm_calibrated);
        assert!(
            !calibrated,
            "M0 invariant breach: `povm_calibrated` cfg is ON. Per S1003733 \
             KEEP-DORMANT (Plan v2 §15 R4) + ADR D-S1002127-01 (m42 stcortex-\
             only pivot), m8's POVM gate is dormant — workflow-trace has zero \
             POVM read sites. Either unset POVM_CR2_DEPLOYED in the build env, \
             OR ratify POVM re-introduction by amending the m8 module docstring \
             KEEP-DORMANT block and updating this assertion."
        );
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
