//! V2 substrate back-pressure budget — per-substrate
//! `SubstrateBackPressureMode` enum (Plan v2 v0.2.0 §3 Phase 8 + §15
//! DX-2 = per-substrate per NA-8 reshape).
//!
//! v0.1.0 had no back-pressure surface (NA-GAP-04 spec-only at HEAD per
//! Phase 2 audit). v0.2.0 ships the type + per-substrate registry; the
//! engine receives `BackPressureSignal` envelopes (push mode) OR probes
//! substrate state on a cadence (pull mode). Per-substrate selection
//! lives in [`BackPressureRegistry`] keyed by [`SubstrateId`].
//!
//! ## Three modes per substrate (DX-2 + NA-8 reshape)
//!
//! - **`Push`**: substrate emits [`BackPressureSignal`] envelopes to the
//!   engine on its own cadence. Substrate participation required. Real
//!   Frame-B per the Plan v2 NA pass.
//! - **`Pull`**: engine probes substrate state on a cadence (the engine
//!   times the probe; the substrate doesn't actively participate). Honest
//!   Frame-A-half (per Plan v2 §9.1). Default mode per substrate at
//!   v0.2.0 ship — keeps the gate live without requiring substrate-side
//!   emitters that don't yet ship.
//! - **`Unavailable`**: substrate has neither push-emit nor pull-probe
//!   shipped. Engine emits `RefusalToken::Unavailable(EngineImagined)`
//!   per V5 fallback semantic.
//!
//! ## Deferred to post-v0.2.0 per §11
//!
//! This module ships the TYPES + registry. The cadence-modulation wire
//! into m1 (atuin throttle) / m13 (stcortex throttle) / m32 (Conductor
//! throttle) is per-substrate substrate-side participation per §11
//! consent gradient — atuin upstream may never ship a push-emitter; the
//! mode flip is per-substrate as emitters land. Pull-mode probe wires
//! are also deferred (the probe cadence is itself a v0.3.0 design
//! question — once per second? once per dispatch? configurable?). v0.2.0
//! ships the contract; v0.3.0+ ships the throttle integration per
//! substrate.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::refusal_token::SubstrateId;

/// Per-substrate back-pressure mode per DX-2 (NA-8 reshape).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubstrateBackPressureMode {
    /// Substrate emits [`BackPressureSignal`] envelopes to the engine.
    /// Real Frame-B per Plan v2 §9.1. Substrate participation required.
    Push,
    /// Engine probes substrate state on its own cadence. Honest
    /// Frame-A-half (engine-timed). Default per substrate at v0.2.0 ship.
    Pull,
    /// Substrate has neither push-emit nor pull-probe shipped. Engine
    /// emits `RefusalToken::Unavailable(EngineImagined)` per V5 fallback.
    Unavailable,
}

impl Default for SubstrateBackPressureMode {
    /// Default at v0.2.0 ship = [`SubstrateBackPressureMode::Pull`] per
    /// §15 DX-2 + Plan v2 v0.2.0 §3 Phase 8 step 2. Pull is the
    /// engine-timed honest-Frame-A-half mode that keeps the gate live
    /// without requiring substrate-side emitters.
    fn default() -> Self {
        Self::Pull
    }
}

/// Severity level for a back-pressure signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackPressureSeverity {
    /// Substrate reports nominal load — engine may proceed without throttle.
    Nominal,
    /// Substrate reports elevated load — engine SHOULD throttle next ops.
    Elevated,
    /// Substrate reports saturation — engine MUST throttle / defer.
    Saturated,
    /// Substrate reports refusal — engine MUST NOT issue further requests
    /// in the current window; pairs with `RefusalToken::SubstrateAuthored`
    /// emission via the throttle integration (post-v0.2.0 per §11).
    Refused,
}

/// Back-pressure envelope a substrate emits in Push mode (or the engine
/// constructs from its probe in Pull mode).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackPressureSignal {
    /// Which substrate is the source.
    pub substrate: SubstrateId,
    /// Severity level (Nominal / Elevated / Saturated / Refused).
    pub severity: BackPressureSeverity,
    /// Wall-clock ms at the substrate's clock when the signal was emitted
    /// (push) or observed (pull). Honest-label: in Pull mode this is the
    /// engine's clock per Plan v2 §9.1.
    pub observed_at_ms: u64,
}

impl BackPressureSignal {
    /// Convenience constructor.
    #[must_use]
    pub const fn new(
        substrate: SubstrateId,
        severity: BackPressureSeverity,
        observed_at_ms: u64,
    ) -> Self {
        Self {
            substrate,
            severity,
            observed_at_ms,
        }
    }
}

/// Per-substrate back-pressure mode registry. Default = `Pull` for every
/// known substrate; operators flip to `Push` as substrate-side emitters
/// ship per §11 consent gradient.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BackPressureRegistry {
    modes: HashMap<SubstrateId, SubstrateBackPressureMode>,
}

impl BackPressureRegistry {
    /// New empty registry. Per [`mode_for`] semantics, queries for any
    /// substrate that hasn't been explicitly set return
    /// [`SubstrateBackPressureMode::Pull`] (the v0.2.0 default).
    ///
    /// [`mode_for`]: BackPressureRegistry::mode_for
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a registry that ships the v0.2.0 default state: every known
    /// substrate keyed to [`SubstrateBackPressureMode::Pull`]. Use this
    /// when you want the registry to enumerate all substrates explicitly
    /// rather than relying on the implicit [`mode_for`] default.
    ///
    /// [`mode_for`]: BackPressureRegistry::mode_for
    #[must_use]
    pub fn all_pull_default() -> Self {
        let known = [
            SubstrateId::Atuin,
            SubstrateId::Stcortex,
            SubstrateId::HabitatConductor,
            SubstrateId::HabitatInjection,
            SubstrateId::Cc5LoopClocks,
            SubstrateId::Watcher,
            SubstrateId::Ralph,
            SubstrateId::CargoBuildGraph,
            SubstrateId::Lcm,
            SubstrateId::SynthexV2,
        ];
        let mut modes = HashMap::new();
        for s in known {
            modes.insert(s, SubstrateBackPressureMode::Pull);
        }
        Self { modes }
    }

    /// Look up the mode for a given substrate. Returns
    /// [`SubstrateBackPressureMode::Pull`] (the v0.2.0 default) when the
    /// substrate has not been explicitly set.
    #[must_use]
    pub fn mode_for(&self, substrate: SubstrateId) -> SubstrateBackPressureMode {
        self.modes
            .get(&substrate)
            .copied()
            .unwrap_or_default()
    }

    /// Set the mode for a substrate. Returns the previous mode (if any).
    pub fn set_mode(
        &mut self,
        substrate: SubstrateId,
        mode: SubstrateBackPressureMode,
    ) -> Option<SubstrateBackPressureMode> {
        self.modes.insert(substrate, mode)
    }

    /// Iterate over all explicitly-set substrate modes (HashMap-key order;
    /// not insertion-ordered).
    pub fn iter(&self) -> impl Iterator<Item = (SubstrateId, SubstrateBackPressureMode)> + '_ {
        self.modes.iter().map(|(s, m)| (*s, *m))
    }

    /// Number of explicitly-set substrate modes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.modes.len()
    }

    /// True when no substrate has been explicitly set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.modes.is_empty()
    }
}

#[cfg(test)]
mod tests;
