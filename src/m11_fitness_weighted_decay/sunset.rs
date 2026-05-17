//! Sunset state machine + per-cycle statistics.
//!
//! Per m11 spec § 3 + § 5.5: each `AcceptedWorkflow` in the m30 bank
//! transitions through [`SunsetPhase::Active`] →
//! [`SunsetPhase::PrunePending`] (on weight < soft threshold) →
//! [`SunsetPhase::SunsetExpired`] (on hard sunset boundary OR weight <
//! prune threshold). `PrunePending → Active` is allowed on fitness
//! recovery; `SunsetExpired → Active` requires explicit Luke override.

/// Workflow lifecycle phase per m11 spec § 5.5 state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SunsetPhase {
    /// Weight ≥ soft threshold (default `0.1`); dispatch-eligible.
    Active,
    /// Weight `<` soft threshold but `>` prune threshold; de-ranked but
    /// still recoverable on fitness rise.
    PrunePending,
    /// `sunset_at < now` OR weight `<` prune threshold; excluded from
    /// dispatch. Returns to Active only via explicit Luke override.
    SunsetExpired,
}

impl SunsetPhase {
    /// Stable ordinal projection for metrics and snapshot-stability tests.
    /// `Active = 0`, `PrunePending = 1`, `SunsetExpired = 2`.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::Active => 0,
            Self::PrunePending => 1,
            Self::SunsetExpired => 2,
        }
    }

    /// Stable identifier for log lines and metric labels.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::PrunePending => "PrunePending",
            Self::SunsetExpired => "SunsetExpired",
        }
    }
}

/// Per-cycle telemetry per m11 spec § 3.
#[derive(Debug, Clone, PartialEq)]
pub struct SunsetStats {
    /// Number of consolidation cycles that produced this stats record. For
    /// a single [`super::consolidation::run_consolidation_cycle`] call this
    /// is exactly `1`; aggregators may sum across cycles.
    pub cycles_run: u64,
    /// Number of active workflows that had decay applied this cycle.
    pub workflows_decayed: usize,
    /// Number of workflows transitioned (or marked for) prune this cycle.
    pub workflows_pruned: usize,
    /// Number of workflows transitioned to SunsetExpired this cycle.
    pub workflows_auto_sunset: usize,
    /// Number of workflows transitioned to PrunePending (soft floor) this
    /// cycle. Recoverable on fitness rise — the recovery edge
    /// (PrunePending → Active) is owned by the bank consumer (m30), not by
    /// m11, so no symmetric counter lives in this struct.
    pub workflows_prune_pending: usize,
    /// Number of workflows whose `last_run_ms` was future-dated relative to
    /// `now_ms` (clock-skew) and therefore SKIPPED this cycle. Surfacing
    /// this prevents the F-POVM-07 silent-zero-timestamp pattern from
    /// inflating recency credit on phantom future times.
    pub workflows_clock_skew_skipped: usize,
    /// Mean of the per-workflow [`super::formula::DecayFactor`] applied
    /// this cycle; `0.0` if `workflows_decayed == 0`.
    pub mean_decay_factor: f64,
    /// Minimum factor applied; `f64::INFINITY` if no workflows decayed.
    pub min_decay_factor: f64,
    /// Maximum factor applied; `f64::NEG_INFINITY` if no workflows decayed.
    pub max_decay_factor: f64,
}

impl Default for SunsetStats {
    fn default() -> Self {
        Self {
            cycles_run: 0,
            workflows_decayed: 0,
            workflows_pruned: 0,
            workflows_auto_sunset: 0,
            workflows_prune_pending: 0,
            workflows_clock_skew_skipped: 0,
            mean_decay_factor: 0.0,
            min_decay_factor: f64::INFINITY,
            max_decay_factor: f64::NEG_INFINITY,
        }
    }
}

/// Snapshot of an active workflow as the bank exposes it to the
/// consolidation cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct AcceptedWorkflowDecay {
    /// Stable workflow identifier (m30 bank key).
    pub workflow_id: String,
    /// Pathway id used to look up substrate fitness via the m42 stcortex
    /// route (post-2026-05-17 ADR).
    pub pathway_id: String,
    /// Wall-clock timestamp (ms since UNIX epoch) of the last dispatch of
    /// this workflow.
    pub last_run_ms: i64,
}

#[cfg(test)]
mod tests {
    use super::{AcceptedWorkflowDecay, SunsetPhase, SunsetStats};

    #[test]
    fn phase_ordinals_match_state_machine() {
        assert_eq!(SunsetPhase::Active.ordinal(), 0);
        assert_eq!(SunsetPhase::PrunePending.ordinal(), 1);
        assert_eq!(SunsetPhase::SunsetExpired.ordinal(), 2);
    }

    #[test]
    fn phase_as_str_is_stable() {
        assert_eq!(SunsetPhase::Active.as_str(), "Active");
        assert_eq!(SunsetPhase::PrunePending.as_str(), "PrunePending");
        assert_eq!(SunsetPhase::SunsetExpired.as_str(), "SunsetExpired");
    }

    #[test]
    fn phase_implements_copy_eq_hash() {
        use std::collections::HashSet;
        let mut s: HashSet<SunsetPhase> = HashSet::new();
        s.insert(SunsetPhase::Active);
        s.insert(SunsetPhase::Active);
        s.insert(SunsetPhase::PrunePending);
        s.insert(SunsetPhase::SunsetExpired);
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn stats_default_initialises_min_max_sentinels() {
        let s = SunsetStats::default();
        assert_eq!(s.cycles_run, 0);
        assert_eq!(s.workflows_decayed, 0);
        assert!(s.min_decay_factor.is_infinite() && s.min_decay_factor.is_sign_positive());
        assert!(s.max_decay_factor.is_infinite() && s.max_decay_factor.is_sign_negative());
    }

    #[test]
    fn stats_clone_preserves_values() {
        let s = SunsetStats {
            cycles_run: 5,
            workflows_decayed: 42,
            ..SunsetStats::default()
        };
        let c = s.clone();
        assert_eq!(c, s);
    }

    #[test]
    fn accepted_workflow_decay_clone_eq_debug() {
        let w = AcceptedWorkflowDecay {
            workflow_id: "wf_1".into(),
            pathway_id: "pw_a".into(),
            last_run_ms: 1_700_000_000_000,
        };
        let w2 = w.clone();
        assert_eq!(w, w2);
        let s = format!("{w:?}");
        assert!(s.contains("wf_1"));
        assert!(s.contains("pw_a"));
    }
}
