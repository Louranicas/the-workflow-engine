//! `m16_substrate_drift_canary` — substrate-drift canary V3 KEYSTONE
//! (Plan v2 v0.2.0 §3 Phase 9 per DX-V3 = own module + Genesis Prompt
//! v1.4 amendment lifting module count 26 → 27 + Cluster E placement).
//!
//! Cluster E · L5 (Evidence + Pressure + **Substrate-Observation**).
//!
//! ## Purpose
//!
//! Detect substrate clock-incoherence — when the engine's internal clock
//! assumption (the CC-5 loop expects monotone agreement across m11
//! recency, m13 stcortex-decay, injection-TTL, atuin-checkpoint, and
//! stcortex-pathway-decay) breaks down. Each substrate runs its own
//! clock on its own schedule; when those clocks disagree past the
//! agree-to-skew envelope, the same pattern can be "recent" to m11 and
//! "swept" by injection.db at the same wall-clock instant — silent
//! corruption of the CC-5 substrate-learning loop.
//!
//! m16 samples all 5 clocks, computes pair-wise skew, and emits a
//! [`RefusalToken::SubstrateAuthored Cc5LoopClocks`] event when any
//! pair exceeds the envelope.
//!
//! ## NA-4 self-canary mitigation (Watcher liveness assertion)
//!
//! V3's self-canary problem (NA-4): if m16 itself stops emitting, the
//! engine has no internal observer to flag the absence (m40 Nexus
//! shares fate with m16 in ~all failure modes). The honest mitigation
//! is the Watcher's deployment-watch journal asserting m16 heartbeat
//! liveness per cycle; missing heartbeat for N cycles is a
//! substrate-emitted alert via Watcher's separate clock.
//!
//! This module ships the engine-side heartbeat emission via
//! [`Heartbeat`]; the Watcher-side assertion is post-v0.2.0 OP-6 per
//! Plan v2 §16. If Watcher integration cannot ship, the self-canary
//! problem is honestly NOT mitigated — Plan v2 §6 risk-register row
//! says so.
//!
//! ## C-9 alert-fatigue mitigation (rate-limited dedup)
//!
//! [`AlertBudget`] enforces ≤N alerts per soak hour with dedup by
//! `(clock-pair, envelope-band)`; operator visible only after N
//! consecutive crossings. Prevents a false-positive storm from
//! desensitising the operator.

use std::collections::HashMap;

use crate::refusal_token::{RefusalToken, SubstrateId};

/// Identifier for each of the 5 substrate clocks the CC-5 loop crosses.
///
/// Per `ai_specs/cross-cutting/substrate-drift.md` + Plan v2 v0.2.0 §3
/// Phase 8 step 4 (CC-5 clock enumeration originally landed at v0.1.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClockSource {
    /// m11 fitness-weighted-decay recency clock (engine wall-clock-based).
    M11Recency,
    /// m13 stcortex-writer decay clock (stcortex schedule).
    M13StcortexDecay,
    /// injection.db hourly TTL sweep clock.
    InjectionTtl,
    /// atuin WAL checkpoint clock.
    AtuinCheckpoint,
    /// stcortex Hebbian-pathway decay clock.
    StcortexPathwayDecay,
}

impl ClockSource {
    /// All 5 known CC-5 clock sources in canonical order.
    pub const ALL: [Self; 5] = [
        Self::M11Recency,
        Self::M13StcortexDecay,
        Self::InjectionTtl,
        Self::AtuinCheckpoint,
        Self::StcortexPathwayDecay,
    ];
}

/// A single observed sample of a clock source, with the observation
/// timestamp in ms (the engine's wall-clock at sample time — honestly
/// labelled per Plan v2 §9.1 Frame-A pacing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ClockSample {
    /// Which clock was sampled.
    pub source: ClockSource,
    /// Clock value reported by the substrate (in ms, substrate-clock
    /// units). For stcortex-decay this is the next-decay-tick ms; for
    /// m11 recency this is the last-touched ms; etc.
    pub clock_value_ms: u64,
    /// Engine wall-clock at sample time.
    pub observed_at_ms: u64,
}

/// Trait for sampling a single substrate clock. Test fixtures
/// implement this; the production wires (post-v0.2.0 per §11) read
/// each substrate's actual clock primitive.
pub trait ClockSampler {
    /// Sample the clock. Implementations should be side-effect-free
    /// (idempotent for the current sample window).
    fn sample(&self) -> ClockSample;
    /// Which clock source this sampler tracks (for routing in the
    /// detector).
    fn source(&self) -> ClockSource;
}

/// Agree-to-skew envelope: pair-wise clocks are considered coherent
/// when their skew (|a - b|) is at or below `max_skew_ms`.
///
/// Default = 5_000ms (5 seconds) — generous for substrate-stable
/// systems; operator-tunable for tighter substrate coupling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SkewEnvelope {
    /// Maximum allowed pair-wise clock skew in ms.
    pub max_skew_ms: u64,
}

impl Default for SkewEnvelope {
    fn default() -> Self {
        Self {
            max_skew_ms: 5_000,
        }
    }
}

/// Heartbeat record emitted per m16 detection cycle. The Watcher's
/// deployment-watch journal asserts this is present per cycle (NA-4
/// self-canary mitigation; OP-6 wire is post-v0.2.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Heartbeat {
    /// Engine wall-clock at heartbeat emission.
    pub emitted_at_ms: u64,
    /// Cycle counter (monotone-increasing across detector lifetime).
    pub cycle: u64,
}

/// Alert budget — rate-limited dedup per (clock-pair, envelope-band) to
/// mitigate C-9 alert-fatigue. Tracks the last-alerted timestamp per
/// distinct pair-key; suppresses subsequent alerts within the
/// configured cooldown.
#[derive(Debug, Clone)]
pub struct AlertBudget {
    /// Minimum interval between alerts for the same (a, b) clock pair, ms.
    min_interval_ms: u64,
    /// Last-alerted ms per (a, b) sorted-pair key.
    last_alert: HashMap<(ClockSource, ClockSource), u64>,
}

impl AlertBudget {
    /// Construct with operator-supplied cooldown. Default 60_000ms (1 min).
    #[must_use]
    pub fn new(min_interval_ms: u64) -> Self {
        Self {
            min_interval_ms,
            last_alert: HashMap::new(),
        }
    }

    /// Should an alert for this (a, b) pair fire now? Updates the
    /// last-alerted bookkeeping on `true` returns. Pair is normalised
    /// to (min, max) so symmetric crossings dedup correctly.
    ///
    /// **M5 silent-failure-hunter fix (CRITICAL — m16 self-canary
    /// meta-failure):** the previous `saturating_sub` would return 0
    /// on a clock rewind (`now_ms < last`), which is `< min_interval_ms`,
    /// which would SUPPRESS the alert. That is precisely the failure
    /// mode m16 EXISTS to detect — clock-skew rewind silently muting
    /// the canary that watches for clock-skew rewind. Explicit
    /// rewind-detection now fires the alert AND emits a separate
    /// rewind-warn for operator visibility.
    pub fn should_fire(&mut self, a: ClockSource, b: ClockSource, now_ms: u64) -> bool {
        let (lo, hi) = if (a as u8) <= (b as u8) { (a, b) } else { (b, a) };
        let key = (lo, hi);
        match self.last_alert.get(&key).copied() {
            Some(last) if now_ms < last => {
                // Clock rewind detected — the exact failure m16 is built
                // for. Fire anyway (do not let cooldown suppress drift
                // detection during the rewind window) + emit the
                // meta-warn so operators see the clock-skew itself.
                tracing::warn!(
                    target: "m16.alert_budget.clock_rewind",
                    pair_lo = ?lo, pair_hi = ?hi,
                    last_alert_ms = last, now_ms,
                    "engine clock appears to have rewound; firing alert anyway \
                     to preserve m16 detection during the very failure mode it exists for"
                );
                self.last_alert.insert(key, now_ms);
                true
            }
            Some(last) if now_ms.saturating_sub(last) < self.min_interval_ms => false,
            _ => {
                self.last_alert.insert(key, now_ms);
                true
            }
        }
    }
}

impl Default for AlertBudget {
    /// 60_000ms (1 minute) cooldown between alerts for the same pair.
    fn default() -> Self {
        Self::new(60_000)
    }
}

/// Compute the absolute skew between two clock values in ms.
#[must_use]
pub const fn pair_skew_ms(a: u64, b: u64) -> u64 {
    a.abs_diff(b)
}

/// Substrate-drift detector — owns N clock samplers + envelope + alert
/// budget. Per detection cycle: sample all clocks, compute pair-wise
/// skews, emit RefusalToken::SubstrateAuthored Cc5LoopClocks events
/// for any pair exceeding the envelope (rate-limited by AlertBudget).
pub struct DriftDetector {
    samplers: Vec<Box<dyn ClockSampler>>,
    envelope: SkewEnvelope,
    budget: AlertBudget,
    cycle: u64,
}

impl DriftDetector {
    /// Construct a detector with the given samplers + envelope + budget.
    #[must_use]
    pub fn new(
        samplers: Vec<Box<dyn ClockSampler>>,
        envelope: SkewEnvelope,
        budget: AlertBudget,
    ) -> Self {
        Self {
            samplers,
            envelope,
            budget,
            cycle: 0,
        }
    }

    /// Run one detection cycle. Returns the list of drift events
    /// emitted (as V1 RefusalToken envelopes) + a [`Heartbeat`] for
    /// the Watcher liveness assertion.
    pub fn detect(&mut self, now_ms: u64) -> DetectionResult {
        // M4 silent-failure-hunter fix: previous saturating_add would
        // silently freeze the cycle counter at u64::MAX, breaking
        // Watcher liveness assertion (NA-4 mitigation) — heartbeat
        // would stop advancing while emitted_at_ms still ticked. Log
        // an error-level event on saturation so operators see the
        // canary's own canary fail.
        if self.cycle == u64::MAX {
            tracing::error!(
                target: "m16.detector.cycle_saturated",
                "m16 detector cycle counter saturated at u64::MAX; \
                 Watcher liveness assertion (NA-4) heartbeat will FREEZE — \
                 restart the detector to recover"
            );
        }
        self.cycle = self.cycle.saturating_add(1);
        let samples: Vec<ClockSample> = self.samplers.iter().map(|s| s.sample()).collect();
        let mut events: Vec<RefusalToken> = Vec::new();
        // Pair-wise skew check; each unordered pair (i, j) where i < j.
        for i in 0..samples.len() {
            for j in (i + 1)..samples.len() {
                let a = samples[i];
                let b = samples[j];
                let skew = pair_skew_ms(a.clock_value_ms, b.clock_value_ms);
                if skew > self.envelope.max_skew_ms
                    && self.budget.should_fire(a.source, b.source, now_ms)
                {
                    let reason = format!(
                        "cc5_clock_skew:pair=({:?},{:?}) skew_ms={} envelope_ms={}",
                        a.source, b.source, skew, self.envelope.max_skew_ms
                    );
                    events.push(RefusalToken::substrate_authored(
                        SubstrateId::Cc5LoopClocks,
                        reason,
                    ));
                }
            }
        }
        DetectionResult {
            heartbeat: Heartbeat {
                emitted_at_ms: now_ms,
                cycle: self.cycle,
            },
            samples,
            events,
        }
    }

    /// Current cycle counter (monotone since construction).
    #[must_use]
    pub const fn cycle(&self) -> u64 {
        self.cycle
    }
}

/// Result of one [`DriftDetector::detect`] cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct DetectionResult {
    /// Per-cycle heartbeat for Watcher liveness assertion (NA-4).
    pub heartbeat: Heartbeat,
    /// All clock samples taken this cycle.
    pub samples: Vec<ClockSample>,
    /// Drift events emitted (one RefusalToken per (clock-pair) skew
    /// violation that passed the alert budget). Empty when no drift OR
    /// when the budget suppressed all detected drift.
    pub events: Vec<RefusalToken>,
}

#[cfg(test)]
mod tests;
