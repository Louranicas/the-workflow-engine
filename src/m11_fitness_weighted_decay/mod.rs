//! `m11_fitness_weighted_decay` — Gap 2 NEW PRIMITIVE + lifecycle aspect.
//!
//! See [m11 spec](../../../ai_specs/modules/cluster-D/m11_fitness_weighted_decay.md).
//!
//! # Gap 2 ownership
//!
//! This module owns Gap 2 per CLAUDE.md § structural-gap authorship: the
//! `frequency × fitness × recency` compound decay formula. No upstream
//! ancestor composes all three signals; m11 is the exclusive author. The
//! formula lives in [`formula::compute_decay_factor`].
//!
//! # Compositional integrity
//!
//! Per the m11 spec § 5.1 interpretation: usage alone never grants
//! immortality. A workflow with `frequency = 1.0` but `fitness = 0.0`
//! decays at `base_rate` exactly as fast as an unused workflow. The
//! multiplicative product semantics enforces this structurally.
//!
//! # Lifecycle position
//!
//! m11 is the lifecycle checkpoint of the CC-2 trust regime:
//!
//! - m8 catches POVM misreading at **compile time**;
//! - m9 catches namespace drift at **write time**;
//! - m10 catches mis-calibrated output at **CI gate time**;
//! - m11 catches workflow ossification at **lifecycle time**.
//!
//! # Day-1 surface
//!
//! - [`compute_decay_factor`] + [`DecayFactor`] — the Gap 2 primitive.
//! - [`recency_factor`] / [`frequency_factor`] / [`fitness_factor`] —
//!   signal normalisations.
//! - [`SunsetPhase`] / [`SunsetStats`] / [`AcceptedWorkflowDecay`] — state
//!   machine + telemetry.
//! - [`run_consolidation_cycle`] + [`DecayConfig`] + reader/bank traits —
//!   the 4-step cycle, generic over upstream surface modules that ship
//!   Day-2+ (m7 / m14 / m42 / m30).
//! - [`DecayError`] — typed failure modes (`OutOfRange`, `ClockUnavailable`,
//!   `PathwayReadFailed`, `CycleAborted`).
//!
//! # Verb-class
//!
//! `record` — m11 RECORDS decayed weights; m31 SELECTS using them; m32
//! DISPATCHES. m11 never selects or dispatches itself (preserves the
//! Phase-A passive-verb invariant under the single-phase override).

pub mod consolidation;
pub mod error;
pub mod formula;
pub mod inputs;
pub mod sunset;

pub use consolidation::{
    chrono_now_ms, run_consolidation_cycle, DecayConfig, FrequencyReader, LifecycleBank,
    PathwayWeightReader,
};
pub use error::DecayError;
pub use formula::{compute_decay_factor, DecayFactor};
pub use inputs::{fitness_factor, frequency_factor, recency_factor};
pub use sunset::{AcceptedWorkflowDecay, SunsetPhase, SunsetStats};

#[cfg(test)]
mod tests {
    use super::{
        compute_decay_factor, fitness_factor, frequency_factor, recency_factor, DecayConfig,
        SunsetPhase, SunsetStats,
    };

    #[test]
    fn reexports_compute_decay_factor() {
        let d = compute_decay_factor(1.0, 1.0, 1.0, 0.02).unwrap();
        assert!((d.as_f64() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn reexports_signal_normalisers() {
        assert!((recency_factor(0.0, 30.0) - 1.0).abs() < 1e-12);
        assert!((frequency_factor(100, 100) - 1.0).abs() < 1e-12);
        assert!((fitness_factor(0.5) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn reexports_state_machine_and_config() {
        assert_eq!(SunsetPhase::Active.ordinal(), 0);
        assert_eq!(SunsetStats::default().cycles_run, 0);
        let c = DecayConfig::default();
        assert!((c.plain_decay_rate - 0.02).abs() < 1e-12);
    }
}
