//! 4-step consolidation cycle: decay → reinforce-read → prune → auto-sunset.
//!
//! Per m11 spec § 5.4 + § 5.6: this is the engine-wide sunset law driver.
//! The real m7 / m14 / m42 reader implementations don't exist Day-1, so m11
//! consumes them via trait abstractions ([`LifecycleBank`],
//! [`PathwayWeightReader`], [`FrequencyReader`]). Real implementations land
//! when those modules ship (Day-2+).

use std::time::{SystemTime, UNIX_EPOCH};

use super::error::DecayError;
use super::formula::{compute_decay_factor, DecayFactor};
use super::inputs::{fitness_factor, frequency_factor, recency_factor};
use super::sunset::{AcceptedWorkflowDecay, SunsetPhase, SunsetStats};

/// Configuration defaults per m11 spec § 2.
///
/// - `plain_decay_rate = 0.02` → 228-cycle floor per spec § 5.3 calibration.
/// - `recency_half_life_days = 30.0` aligns with the Phase 6 D120 sunset.
/// - `sunset_threshold = 0.05` — weight below which a workflow enters
///   `SunsetExpired` (paired with the m30 sunset trigger).
/// - `prune_threshold = 0.01` — lifted from `povm-v2_lifecycle.rs`.
#[derive(Debug, Clone, PartialEq)]
pub struct DecayConfig {
    /// Per-cycle floor decay rate. `base_rate = 1.0 - plain_decay_rate`.
    pub plain_decay_rate: f64,
    /// Half-life for the recency exponential.
    pub recency_half_life_days: f64,
    /// Soft threshold for the SunsetExpired transition (paired with m30).
    pub sunset_threshold: f64,
    /// Hard prune threshold (weight below which the workflow is marked).
    pub prune_threshold: f64,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            plain_decay_rate: 0.02,
            recency_half_life_days: 30.0,
            sunset_threshold: 0.05,
            prune_threshold: 0.01,
        }
    }
}

/// Reader for substrate fitness signals (post-2026-05-17 m42 stcortex-only
/// ADR). Day-1: implemented by mock in tests; production impl ships with
/// m42.
pub trait PathwayWeightReader {
    /// Return the current stcortex `pathway.weight` for `pathway_id`,
    /// already in `[0, 1]` by substrate invariant (m11 defensively clamps).
    ///
    /// # Errors
    ///
    /// [`DecayError::PathwayReadFailed`] on substrate read failure.
    fn read_pathway_weight(&self, pathway_id: &str) -> Result<f64, DecayError>;
}

/// Reader for cohort frequency signals (from m14
/// `evidence_aggregator::run_count`).
pub trait FrequencyReader {
    /// Run-count for `workflow_id` over the observation window.
    fn frequency(&self, workflow_id: &str) -> u64;
    /// Maximum `run_count` across the entire bank cohort this cycle.
    fn cohort_max(&self) -> u64;
}

/// Lifecycle bank surface (m30 bank). Day-1: mock implementations live in
/// `tests/m11_integration.rs`. Production impl ships with m30.
pub trait LifecycleBank {
    /// All workflows currently in `SunsetPhase::Active` or
    /// `SunsetPhase::PrunePending`. The cycle iterates this slice.
    fn iter_active(&self) -> Vec<AcceptedWorkflowDecay>;

    /// Apply the decay factor multiplicatively to the workflow's current
    /// weight: `weight_next = weight × factor`.
    fn apply_decay(&mut self, workflow_id: &str, factor: DecayFactor);

    /// Current post-decay weight of the workflow, or `None` if unknown.
    fn weight_of(&self, workflow_id: &str) -> Option<f64>;

    /// Mark a workflow for prune at the next sweep. The bank may delete
    /// asynchronously; m11 only emits the hint.
    fn mark_for_prune(&mut self, workflow_id: &str);

    /// Hard sunset wall-clock timestamp (ms since UNIX epoch) for the
    /// workflow if Luke set one explicitly; `None` if the workflow has no
    /// explicit sunset boundary (decay alone governs).
    fn sunset_at_of(&self, workflow_id: &str) -> Option<i64>;

    /// Record a phase transition for the workflow.
    fn transition(&mut self, workflow_id: &str, phase: SunsetPhase);
}

/// Wall-clock ms since UNIX epoch, or `None` if the system clock is set
/// before the epoch (genuine fault condition per spec § 4 F-POVM-07).
#[must_use]
pub fn chrono_now_ms() -> Option<i64> {
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(dur.as_millis()).ok()
}

/// Run a single 4-step consolidation cycle.
///
/// # Steps
///
/// 1. **Decay** — apply [`compute_decay_factor`] to every active workflow.
/// 2. **Reinforce-read** — external; m42 Hebbian feedback updates the
///    pathway weights that m11 reads on the next cycle. m11 does not write
///    Hebbian updates.
/// 3. **Prune** — mark workflows whose weight `<` `prune_threshold` AND
///    whose `sunset_at` is set (defense-in-depth: a workflow with no
///    explicit sunset is never auto-pruned by decay alone).
/// 4. **Auto-sunset** — transition workflows whose `sunset_at < now` to
///    `SunsetPhase::SunsetExpired`.
///
/// # Errors
///
/// - [`DecayError::ClockUnavailable`] if `now_ms_fn` returns `None`. The
///   cycle is skipped (rather than treating timestamps as `0` — that would
///   be the F-POVM-07 silent-zero-timestamp pattern).
/// - [`DecayError::PathwayReadFailed`] propagated from the
///   [`PathwayWeightReader`].
/// - [`DecayError::OutOfRange`] propagated from [`compute_decay_factor`]
///   (impossible in practice but typed for exhaustiveness).
#[allow(
    clippy::cast_precision_loss,
    reason = "i64 ms-since-epoch fits in f64 mantissa for the relevant Earth-time range; \
              precision irrelevant for day-scale recency calculations"
)]
pub fn run_consolidation_cycle<B, R, F, T>(
    bank: &mut B,
    pathways: &R,
    freq: &F,
    config: &DecayConfig,
    now_ms_fn: T,
) -> Result<SunsetStats, DecayError>
where
    B: LifecycleBank,
    R: PathwayWeightReader,
    F: FrequencyReader,
    T: Fn() -> Option<i64>,
{
    let now_ms = now_ms_fn().ok_or_else(|| {
        tracing::warn!(
            target: "m11.consolidation.clock_skip",
            "clock unavailable; skipping consolidation cycle"
        );
        DecayError::ClockUnavailable
    })?;

    let workflows = bank.iter_active();
    let cohort_max = freq.cohort_max();
    let mut stats = SunsetStats::default();
    let mut factors: Vec<f64> = Vec::with_capacity(workflows.len());

    // Step 1 — Decay.
    for wf in &workflows {
        let f_norm = frequency_factor(freq.frequency(&wf.workflow_id), cohort_max);
        let fit = fitness_factor(pathways.read_pathway_weight(&wf.pathway_id)?);
        let elapsed_ms = now_ms.saturating_sub(wf.last_run_ms).max(0);
        let days = elapsed_ms as f64 / (1000.0 * 86400.0);
        let r = recency_factor(days, config.recency_half_life_days);
        let factor = compute_decay_factor(f_norm, fit, r, config.plain_decay_rate)?;
        bank.apply_decay(&wf.workflow_id, factor);
        factors.push(factor.as_f64());
        stats.workflows_decayed += 1;
    }

    // Step 2 — Reinforce-read: external (m42 → stcortex). No m11 action.

    // Step 3 — Prune.
    for wf in &workflows {
        let weight = bank.weight_of(&wf.workflow_id).unwrap_or(0.0);
        if weight < config.prune_threshold && bank.sunset_at_of(&wf.workflow_id).is_some() {
            bank.mark_for_prune(&wf.workflow_id);
            stats.workflows_pruned += 1;
        }
    }

    // Step 4 — Auto-sunset.
    for wf in &workflows {
        if let Some(sunset_at) = bank.sunset_at_of(&wf.workflow_id) {
            if sunset_at < now_ms {
                bank.transition(&wf.workflow_id, SunsetPhase::SunsetExpired);
                stats.workflows_auto_sunset += 1;
            }
        }
    }

    // Aggregate factor stats.
    if !factors.is_empty() {
        let sum: f64 = factors.iter().sum();
        let n = factors.len() as f64;
        stats.mean_decay_factor = sum / n;
        stats.min_decay_factor = factors
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        stats.max_decay_factor = factors
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
    }

    stats.cycles_run = 1;

    tracing::info!(
        target: "m11.consolidation.cycle",
        cycles_run = stats.cycles_run,
        workflows_decayed = stats.workflows_decayed,
        workflows_pruned = stats.workflows_pruned,
        workflows_auto_sunset = stats.workflows_auto_sunset,
        mean_decay_factor = stats.mean_decay_factor,
        "m11 consolidation cycle complete"
    );

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::error::DecayError;
    use super::super::formula::DecayFactor;
    use super::super::sunset::{AcceptedWorkflowDecay, SunsetPhase};
    use super::{
        chrono_now_ms, run_consolidation_cycle, DecayConfig, FrequencyReader,
        LifecycleBank, PathwayWeightReader,
    };

    // ---- Mock bank + readers --------------------------------------------

    struct MockBank {
        active: Vec<AcceptedWorkflowDecay>,
        weights: HashMap<String, f64>,
        sunsets: HashMap<String, i64>,
        transitions: Vec<(String, SunsetPhase)>,
        prunes: Vec<String>,
    }

    impl LifecycleBank for MockBank {
        fn iter_active(&self) -> Vec<AcceptedWorkflowDecay> {
            self.active.clone()
        }
        fn apply_decay(&mut self, workflow_id: &str, factor: DecayFactor) {
            let entry = self.weights.entry(workflow_id.to_owned()).or_insert(1.0);
            *entry *= factor.as_f64();
        }
        fn weight_of(&self, workflow_id: &str) -> Option<f64> {
            self.weights.get(workflow_id).copied()
        }
        fn mark_for_prune(&mut self, workflow_id: &str) {
            self.prunes.push(workflow_id.to_owned());
        }
        fn sunset_at_of(&self, workflow_id: &str) -> Option<i64> {
            self.sunsets.get(workflow_id).copied()
        }
        fn transition(&mut self, workflow_id: &str, phase: SunsetPhase) {
            self.transitions.push((workflow_id.to_owned(), phase));
        }
    }

    struct MockPathways(HashMap<String, f64>);

    impl PathwayWeightReader for MockPathways {
        fn read_pathway_weight(&self, pathway_id: &str) -> Result<f64, DecayError> {
            self.0
                .get(pathway_id)
                .copied()
                .ok_or_else(|| DecayError::PathwayReadFailed {
                    pathway_id: pathway_id.to_owned(),
                    reason: "not in mock".into(),
                })
        }
    }

    struct MockFreq {
        counts: HashMap<String, u64>,
        cohort_max: u64,
    }

    impl FrequencyReader for MockFreq {
        fn frequency(&self, workflow_id: &str) -> u64 {
            self.counts.get(workflow_id).copied().unwrap_or(0)
        }
        fn cohort_max(&self) -> u64 {
            self.cohort_max
        }
    }

    fn make_active(workflow_id: &str, pathway_id: &str, last_run_ms: i64) -> AcceptedWorkflowDecay {
        AcceptedWorkflowDecay {
            workflow_id: workflow_id.to_owned(),
            pathway_id: pathway_id.to_owned(),
            last_run_ms,
        }
    }

    // ---- Default config (1) ---------------------------------------------

    #[test]
    fn default_config_matches_spec_constants() {
        let c = DecayConfig::default();
        assert!((c.plain_decay_rate - 0.02).abs() < 1e-12);
        assert!((c.recency_half_life_days - 30.0).abs() < 1e-12);
        assert!((c.sunset_threshold - 0.05).abs() < 1e-12);
        assert!((c.prune_threshold - 0.01).abs() < 1e-12);
    }

    // ---- chrono_now_ms (1) ----------------------------------------------

    #[test]
    fn chrono_now_ms_returns_some_positive_value() {
        let v = chrono_now_ms().expect("wall clock");
        // Year 2020 onwards = > 1.5e12 ms.
        assert!(v > 1_500_000_000_000);
    }

    // ---- Cycle: clock unavailable -> error (1) --------------------------

    #[test]
    fn cycle_returns_clock_unavailable_when_now_fn_yields_none() {
        let mut bank = MockBank {
            active: vec![],
            weights: HashMap::new(),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::new());
        let freq = MockFreq {
            counts: HashMap::new(),
            cohort_max: 0,
        };
        let cfg = DecayConfig::default();
        let err = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || None)
            .unwrap_err();
        assert!(matches!(err, DecayError::ClockUnavailable));
    }

    // ---- Cycle: empty bank (1) ------------------------------------------

    #[test]
    fn cycle_on_empty_bank_yields_zero_stats() {
        let mut bank = MockBank {
            active: vec![],
            weights: HashMap::new(),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::new());
        let freq = MockFreq {
            counts: HashMap::new(),
            cohort_max: 0,
        };
        let cfg = DecayConfig::default();
        let now = 1_700_000_000_000_i64;
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("empty cycle");
        assert_eq!(stats.cycles_run, 1);
        assert_eq!(stats.workflows_decayed, 0);
        assert_eq!(stats.workflows_pruned, 0);
        assert_eq!(stats.workflows_auto_sunset, 0);
    }

    // ---- Cycle: single thriving workflow (1) ----------------------------

    #[test]
    fn cycle_on_thriving_workflow_yields_factor_one() {
        let now = 1_700_000_000_000_i64;
        let mut bank = MockBank {
            active: vec![make_active("wf_thrive", "pw_thrive", now)],
            weights: HashMap::from([(String::from("wf_thrive"), 1.0)]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([(String::from("pw_thrive"), 1.0)]));
        let freq = MockFreq {
            counts: HashMap::from([(String::from("wf_thrive"), 10)]),
            cohort_max: 10,
        };
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle");
        assert_eq!(stats.workflows_decayed, 1);
        // last_run == now → days = 0 → recency = 1.0; freq=1, fit=1 →
        // factor = 1.0; weight remains 1.0.
        let w = bank.weight_of("wf_thrive").expect("weight");
        assert!((w - 1.0).abs() < 1e-12);
    }

    // ---- Cycle: workflow with stale signals decays at base_rate (1) -----

    #[test]
    fn cycle_on_zero_signals_workflow_decays_at_base_rate() {
        let now = 1_700_000_000_000_i64;
        let mut bank = MockBank {
            active: vec![make_active("wf_stale", "pw_stale", now)],
            weights: HashMap::from([(String::from("wf_stale"), 1.0)]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        // Pathway weight 0 → fitness=0 → compound=0 → factor=base_rate=0.98.
        let pathways = MockPathways(HashMap::from([(String::from("pw_stale"), 0.0)]));
        let freq = MockFreq {
            counts: HashMap::from([(String::from("wf_stale"), 0)]),
            cohort_max: 10,
        };
        let cfg = DecayConfig::default();
        run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle");
        let w = bank.weight_of("wf_stale").expect("weight");
        assert!(
            (w - 0.98).abs() < 1e-12,
            "weight {w} should be 1.0 * 0.98 = 0.98"
        );
    }

    // ---- Cycle: prune marker only when weight < threshold AND sunset set
    #[test]
    fn cycle_marks_for_prune_only_when_under_threshold_and_sunset_set() {
        let now = 1_700_000_000_000_i64;
        let mut bank = MockBank {
            active: vec![
                make_active("wf_low_with_sunset", "pw1", now),
                make_active("wf_low_no_sunset", "pw2", now),
            ],
            weights: HashMap::from([
                (String::from("wf_low_with_sunset"), 0.005),
                (String::from("wf_low_no_sunset"), 0.005),
            ]),
            sunsets: HashMap::from([
                // Only the first workflow has an explicit sunset.
                (String::from("wf_low_with_sunset"), now + 86_400_000),
            ]),
            transitions: vec![],
            prunes: vec![],
        };
        // All pathways healthy enough; the prune step gates only on
        // post-decay weight + sunset set.
        let pathways = MockPathways(HashMap::from([
            (String::from("pw1"), 0.0),
            (String::from("pw2"), 0.0),
        ]));
        let freq = MockFreq {
            counts: HashMap::new(),
            cohort_max: 1,
        };
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle");
        assert_eq!(stats.workflows_pruned, 1);
        assert_eq!(bank.prunes, vec![String::from("wf_low_with_sunset")]);
    }

    // ---- Cycle: auto-sunset for expired timestamps ----------------------
    #[test]
    fn cycle_transitions_expired_workflows_to_sunset_expired() {
        let now = 1_700_000_000_000_i64;
        let past = now - 1;
        let mut bank = MockBank {
            active: vec![make_active("wf_expired", "pw", now)],
            weights: HashMap::from([(String::from("wf_expired"), 1.0)]),
            sunsets: HashMap::from([(String::from("wf_expired"), past)]),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([(String::from("pw"), 0.5)]));
        let freq = MockFreq {
            counts: HashMap::from([(String::from("wf_expired"), 1)]),
            cohort_max: 1,
        };
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle");
        assert_eq!(stats.workflows_auto_sunset, 1);
        assert_eq!(
            bank.transitions,
            vec![(String::from("wf_expired"), SunsetPhase::SunsetExpired)]
        );
    }

    // ---- Multi-workflow cycle aggregates stats (1) ----------------------

    #[test]
    fn cycle_aggregates_min_mean_max_decay_factors() {
        let now = 1_700_000_000_000_i64;
        let mut bank = MockBank {
            active: vec![
                make_active("wf_a", "pw_a", now),
                make_active("wf_b", "pw_b", now),
                make_active("wf_c", "pw_c", now),
            ],
            weights: HashMap::from([
                (String::from("wf_a"), 1.0),
                (String::from("wf_b"), 1.0),
                (String::from("wf_c"), 1.0),
            ]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([
            (String::from("pw_a"), 1.0),
            (String::from("pw_b"), 0.5),
            (String::from("pw_c"), 0.0),
        ]));
        let freq = MockFreq {
            counts: HashMap::from([
                (String::from("wf_a"), 10),
                (String::from("wf_b"), 5),
                (String::from("wf_c"), 0),
            ]),
            cohort_max: 10,
        };
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle");
        assert_eq!(stats.workflows_decayed, 3);
        assert!(stats.min_decay_factor.is_finite());
        assert!(stats.max_decay_factor.is_finite());
        assert!(stats.min_decay_factor <= stats.mean_decay_factor);
        assert!(stats.mean_decay_factor <= stats.max_decay_factor);
        // c had all-zero signals → factor 0.98 (the minimum);
        // a had all-one signals → factor 1.0 (the maximum).
        assert!((stats.min_decay_factor - 0.98).abs() < 1e-12);
        assert!((stats.max_decay_factor - 1.0).abs() < 1e-12);
    }

    // ---- Pathway read failure surfaces typed error (1) ------------------

    #[test]
    fn cycle_propagates_pathway_read_failure() {
        let now = 1_700_000_000_000_i64;
        let mut bank = MockBank {
            active: vec![make_active("wf", "pw_missing", now)],
            weights: HashMap::from([(String::from("wf"), 1.0)]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::new()); // no pw_missing entry
        let freq = MockFreq {
            counts: HashMap::new(),
            cohort_max: 0,
        };
        let cfg = DecayConfig::default();
        let err = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .unwrap_err();
        assert!(matches!(err, DecayError::PathwayReadFailed { .. }));
    }
}
