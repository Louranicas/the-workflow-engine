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
/// - `sunset_threshold` — the **soft floor**: weight below which a workflow
///   enters [`SunsetPhase::PrunePending`]. Defaults from
///   [`crate::m30_bank::DEFAULT_PRUNE_PENDING_THRESHOLD`] (`0.10`).
/// - `prune_threshold` — the **hard floor**: weight below which a workflow
///   enters [`SunsetPhase::SunsetExpired`]. Defaults from
///   [`crate::m30_bank::DEFAULT_PRUNE_THRESHOLD`] (`0.05`).
///
/// # F4 — single source of truth (W2 hardening)
///
/// m11 and m30 drive the *same* `Active → PrunePending → SunsetExpired`
/// state machine: m30's [`crate::m30_bank::AcceptedWorkflow::phase_for`]
/// classifies a row by `weight < prune_threshold → SunsetExpired`,
/// `weight < prune_pending_threshold → PrunePending`. m11's consolidation
/// cycle drives the soft/hard transitions on the *same* thresholds.
/// Previously m11 defaulted to `sunset_threshold = 0.05` /
/// `prune_threshold = 0.01` while m30 used `0.10` / `0.05` — so m11
/// telemetry under-reported PrunePending/SunsetExpired transitions relative
/// to m30's actual eviction sweep. m30 is now the single source of truth:
/// m11's soft floor (`sunset_threshold`, → PrunePending) imports m30's
/// [`crate::m30_bank::DEFAULT_PRUNE_PENDING_THRESHOLD`]; m11's hard floor
/// (`prune_threshold`, → SunsetExpired) imports m30's
/// [`crate::m30_bank::DEFAULT_PRUNE_THRESHOLD`]. The field *names* differ
/// (m11 historically calls the soft floor `sunset_threshold`; m30 calls it
/// `prune_pending_threshold`) but the semantics map 1:1.
#[derive(Debug, Clone, PartialEq)]
pub struct DecayConfig {
    /// Per-cycle floor decay rate. `base_rate = 1.0 - plain_decay_rate`.
    pub plain_decay_rate: f64,
    /// Half-life for the recency exponential.
    pub recency_half_life_days: f64,
    /// Soft floor: weight below which a workflow enters
    /// [`SunsetPhase::PrunePending`]. Aligned with m30's
    /// [`crate::m30_bank::DEFAULT_PRUNE_PENDING_THRESHOLD`].
    pub sunset_threshold: f64,
    /// Hard floor: weight below which a workflow enters
    /// [`SunsetPhase::SunsetExpired`]. Aligned with m30's
    /// [`crate::m30_bank::DEFAULT_PRUNE_THRESHOLD`].
    pub prune_threshold: f64,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            plain_decay_rate: 0.02,
            recency_half_life_days: 30.0,
            // F4: m30 is the single source of truth for the lifecycle
            // thresholds — see the type-level doc above.
            sunset_threshold: crate::m30_bank::DEFAULT_PRUNE_PENDING_THRESHOLD,
            prune_threshold: crate::m30_bank::DEFAULT_PRUNE_THRESHOLD,
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

/// Run a single consolidation cycle.
///
/// # Steps
///
/// 0. **Pre-fetch** — read every workflow's substrate pathway weight FIRST
///    (transactional w.r.t. read failures: if any read fails, no earlier
///    workflow has been mutated). Future-dated `last_run_ms` is filtered
///    here to prevent the F-POVM-07 silent-zero pattern from inflating
///    recency credit on clock-skew.
/// 1. **Decay** — apply [`compute_decay_factor`] to every active workflow
///    whose pre-fetch succeeded; skipped workflows count toward
///    [`SunsetStats::workflows_clock_skew_skipped`].
/// 2. **Reinforce-read** — external; m42 Hebbian feedback updates the
///    pathway weights that m11 reads on the next cycle. m11 does not write
///    Hebbian updates.
/// 3. **Soft transitions** — transition workflows whose post-decay weight
///    falls below `sunset_threshold` (but not below `prune_threshold`) to
///    [`SunsetPhase::PrunePending`]; emit recovery edge → Active when
///    weight rises back above `sunset_threshold`.
/// 4. **Prune** — mark workflows whose weight `<` `prune_threshold` AND
///    whose `sunset_at` is set (defense-in-depth: a workflow with no
///    explicit sunset is never auto-pruned by decay alone). Track the set
///    of pruned workflows to prevent double-counting in Step 5.
/// 5. **Auto-sunset** — transition workflows whose `sunset_at < now` to
///    [`SunsetPhase::SunsetExpired`], EXCEPT those already pruned this
///    cycle (which would otherwise appear in both `workflows_pruned` AND
///    `workflows_auto_sunset`).
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

    // Pre-fetch all pathway weights BEFORE any mutation. This makes Step 1
    // transactional w.r.t. pathway-read failures: if the substrate read
    // fails on workflow N, no earlier workflow has been decayed yet.
    // (Fix: zen HIGH finding — partial-state on PathwayReadFailed mid-cycle.)
    let mut prefetched: Vec<Option<(f64, u64)>> = Vec::with_capacity(workflows.len());
    for wf in &workflows {
        // Clock-skew gate: a future-dated last_run_ms (negative elapsed)
        // would silently inflate recency to 1.0 via saturating_sub. Skip
        // the workflow this cycle and surface the count in stats.
        // (Fix: silent-failure-hunter CONFIRMED finding — F-POVM-07 pattern
        // at the previous `now_ms.saturating_sub(...).max(0)` line.)
        if wf.last_run_ms > now_ms {
            tracing::warn!(
                target: "m11.consolidation.clock_skew",
                workflow_id = %wf.workflow_id,
                last_run_ms = wf.last_run_ms,
                now_ms,
                delta_ms = wf.last_run_ms - now_ms,
                "future-dated last_run_ms (clock skew); skipping decay this cycle \
                 to avoid silent recency-inflation (F-POVM-07 pattern)"
            );
            prefetched.push(None);
            continue;
        }
        let weight = pathways.read_pathway_weight(&wf.pathway_id)?;
        let freq_count = freq.frequency(&wf.workflow_id);
        prefetched.push(Some((weight, freq_count)));
    }

    // Step 1 — Decay (now safe: all reads succeeded above).
    for (wf, slot) in workflows.iter().zip(prefetched.iter()) {
        let Some((raw_weight, freq_count)) = *slot else {
            stats.workflows_clock_skew_skipped += 1;
            continue;
        };
        let f_norm = frequency_factor(freq_count, cohort_max);
        let fit = fitness_factor(raw_weight);
        // Guaranteed non-negative since the clock-skew gate above filtered
        // future-dated last_run_ms entries.
        let elapsed_ms = now_ms - wf.last_run_ms;
        let days = elapsed_ms as f64 / (1000.0 * 86400.0);
        let r = recency_factor(days, config.recency_half_life_days);
        let factor = compute_decay_factor(f_norm, fit, r, config.plain_decay_rate)?;
        bank.apply_decay(&wf.workflow_id, factor);
        factors.push(factor.as_f64());
        stats.workflows_decayed += 1;
    }

    // Step 2 — Reinforce-read: external (m42 → stcortex). No m11 action.

    // Steps 2.5 + 3 + 4 — state-machine transitions on post-decay weights.
    run_state_machine_transitions(bank, &workflows, config, now_ms, &mut stats);

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
        workflows_prune_pending = stats.workflows_prune_pending,
        workflows_clock_skew_skipped = stats.workflows_clock_skew_skipped,
        mean_decay_factor = stats.mean_decay_factor,
        "m11 consolidation cycle complete"
    );

    Ok(stats)
}

/// Run the post-decay state-machine transitions (Steps 2.5 + 3 + 4 of the
/// consolidation cycle). Extracted to keep [`run_consolidation_cycle`] under
/// the clippy::too_many_lines threshold; semantically it is the **second
/// half** of one logical pipeline and must always run after Step 1 decay.
///
/// # Step ordering
///
/// 1. Soft transitions: PrunePending arm + recovery edge.
/// 2. Hard prune: tracks `pruned_this_cycle` to prevent Step 4 double-count.
/// 3. Auto-sunset: skips workflows already pruned this cycle.
fn run_state_machine_transitions<B: LifecycleBank>(
    bank: &mut B,
    workflows: &[AcceptedWorkflowDecay],
    config: &DecayConfig,
    now_ms: i64,
    stats: &mut SunsetStats,
) {
    // Step 2.5 — Soft transitions (PrunePending arm only).
    //
    // Per spec § 5.5: a workflow whose post-decay weight has fallen below
    // `sunset_threshold` but is still above `prune_threshold` enters
    // [`SunsetPhase::PrunePending`] — de-ranked from dispatch but still
    // recoverable on subsequent fitness rise.
    //
    // The **recovery edge** (PrunePending → Active when weight rises back
    // above `sunset_threshold`) is owned by the bank consumer (m30), not
    // by m11. Reason: m11 has no `phase_of()` getter on the
    // [`LifecycleBank`] trait and cannot tell whether emitting `Active`
    // would be a true recovery or a no-op echo on an already-Active
    // workflow (the latter would inflate stats counters and noise the
    // transition log). m30 sees the weight delta on every dispatch
    // decision and transitions back to Active there.
    //
    // (Fix: silent-failure-hunter CONFIRMED finding — `sunset_threshold`
    // was declared/defaulted/tested but never read in any control flow;
    // the PrunePending arm was completely unwired prior to this commit.)
    for wf in workflows {
        let weight = bank.weight_of(&wf.workflow_id).unwrap_or(0.0);
        if weight < config.sunset_threshold && weight >= config.prune_threshold {
            bank.transition(&wf.workflow_id, SunsetPhase::PrunePending);
            stats.workflows_prune_pending += 1;
        }
    }

    // Step 3 — Hard prune. Track to prevent double-counting in Step 4.
    // (Fix: silent-failure-hunter CONFIRMED finding — pre-fix, a workflow
    // with weight < prune_threshold AND sunset_at < now was incremented
    // in BOTH workflows_pruned and workflows_auto_sunset.)
    let mut pruned_this_cycle: std::collections::HashSet<&str> =
        std::collections::HashSet::with_capacity(workflows.len());
    for wf in workflows {
        let weight = bank.weight_of(&wf.workflow_id).unwrap_or(0.0);
        if weight < config.prune_threshold && bank.sunset_at_of(&wf.workflow_id).is_some() {
            bank.mark_for_prune(&wf.workflow_id);
            stats.workflows_pruned += 1;
            pruned_this_cycle.insert(wf.workflow_id.as_str());
        }
    }

    // Step 4 — Auto-sunset, excluding already-pruned workflows.
    for wf in workflows {
        if pruned_this_cycle.contains(wf.workflow_id.as_str()) {
            continue;
        }
        if let Some(sunset_at) = bank.sunset_at_of(&wf.workflow_id) {
            if sunset_at < now_ms {
                bank.transition(&wf.workflow_id, SunsetPhase::SunsetExpired);
                stats.workflows_auto_sunset += 1;
            }
        }
    }
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
        // F4: thresholds default from m30 (single source of truth).
        assert!((c.sunset_threshold - 0.10).abs() < 1e-12);
        assert!((c.prune_threshold - 0.05).abs() < 1e-12);
    }

    // rationale: Contract regression — F4 single-source-of-truth. m11's
    // DecayConfig::default() lifecycle thresholds MUST equal the m30 bank
    // constants exactly: soft floor (sunset_threshold → PrunePending) =
    // DEFAULT_PRUNE_PENDING_THRESHOLD; hard floor (prune_threshold →
    // SunsetExpired) = DEFAULT_PRUNE_THRESHOLD. If m30 retunes a constant,
    // m11 follows automatically and this test stays green; if someone
    // forks m11's defaults back to literals this test fires the canary.
    #[test]
    fn default_config_thresholds_track_m30_single_source_of_truth() {
        let c = DecayConfig::default();
        assert!(
            (c.sunset_threshold - crate::m30_bank::DEFAULT_PRUNE_PENDING_THRESHOLD).abs() < 1e-12,
            "m11 soft floor must equal m30 DEFAULT_PRUNE_PENDING_THRESHOLD",
        );
        assert!(
            (c.prune_threshold - crate::m30_bank::DEFAULT_PRUNE_THRESHOLD).abs() < 1e-12,
            "m11 hard floor must equal m30 DEFAULT_PRUNE_THRESHOLD",
        );
        // The soft floor must sit strictly above the hard floor — a sane
        // PrunePending band requires sunset_threshold > prune_threshold.
        assert!(
            c.sunset_threshold > c.prune_threshold,
            "soft floor must exceed hard floor for a non-empty PrunePending band",
        );
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

    // ====================================================================
    // Hardening pass (S1002388) — +10 tests for the m11 critical fixes.
    // Each test carries a `rationale:` covering one of the high-leverage
    // categories from godtier-rust-maintainer § E.
    // ====================================================================

    fn one_wf_bank(weight: f64) -> (MockBank, MockPathways, MockFreq) {
        let bank = MockBank {
            active: vec![AcceptedWorkflowDecay {
                workflow_id: "wf".into(),
                pathway_id: "pw".into(),
                last_run_ms: 1_700_000_000_000,
            }],
            weights: HashMap::from([(String::from("wf"), weight)]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([(String::from("pw"), 0.5)]));
        let freq = MockFreq {
            counts: HashMap::from([(String::from("wf"), 5)]),
            cohort_max: 10,
        };
        (bank, pathways, freq)
    }

    // rationale: Anti-property — silent F-POVM-07 clock-skew MUST NOT
    // silently inflate recency credit. A future-dated last_run_ms must
    // skip-not-clamp; the skip is surfaced in `workflows_clock_skew_skipped`.
    #[test]
    fn clock_skew_future_dated_last_run_ms_skips_workflow_and_counts() {
        let now = 1_700_000_000_000;
        let mut bank = MockBank {
            active: vec![AcceptedWorkflowDecay {
                workflow_id: "wf_future".into(),
                pathway_id: "pw".into(),
                last_run_ms: now + 1_000_000, // 1000s into the future
            }],
            weights: HashMap::from([(String::from("wf_future"), 0.9)]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([(String::from("pw"), 0.5)]));
        let freq = MockFreq {
            counts: HashMap::from([(String::from("wf_future"), 5)]),
            cohort_max: 10,
        };
        let cfg = DecayConfig::default();
        let stats =
            run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
                .expect("cycle ok");
        assert_eq!(stats.workflows_clock_skew_skipped, 1);
        assert_eq!(stats.workflows_decayed, 0);
        // Weight unchanged — no decay applied to the skipped workflow.
        assert!((bank.weight_of("wf_future").unwrap_or(0.0) - 0.9).abs() < 1e-12);
    }

    // rationale: Boundary — exactly equal timestamps (last_run_ms == now)
    // must NOT trip the future-dated gate (elapsed = 0, recency = 1.0 is
    // the legitimate value, not the F-POVM-07 silent-zero pattern).
    #[test]
    fn clock_skew_gate_open_when_last_run_equals_now() {
        let now = 1_700_000_000_000;
        let mut bank = MockBank {
            active: vec![AcceptedWorkflowDecay {
                workflow_id: "wf_now".into(),
                pathway_id: "pw".into(),
                last_run_ms: now,
            }],
            weights: HashMap::from([(String::from("wf_now"), 1.0)]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([(String::from("pw"), 1.0)]));
        let freq = MockFreq {
            counts: HashMap::from([(String::from("wf_now"), 10)]),
            cohort_max: 10,
        };
        let cfg = DecayConfig::default();
        let stats =
            run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
                .expect("cycle ok");
        assert_eq!(stats.workflows_clock_skew_skipped, 0);
        assert_eq!(stats.workflows_decayed, 1);
    }

    // rationale: Contract regression — Step 2.5 PrunePending arm wiring.
    // weight < sunset_threshold && weight >= prune_threshold MUST emit
    // exactly one PrunePending transition for that workflow.
    #[test]
    fn step_2_5_emits_prune_pending_in_soft_band() {
        let now = 1_700_000_000_000;
        // F4-aligned thresholds: sunset=0.10, prune=0.05 → 0.07 is in the
        // soft band [prune_threshold, sunset_threshold).
        let (mut bank, pathways, freq) = one_wf_bank(0.07);
        let cfg = DecayConfig::default();
        let stats =
            run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
                .expect("cycle ok");
        assert_eq!(stats.workflows_prune_pending, 1, "must mark exactly one");
        assert!(
            bank.transitions
                .iter()
                .any(|(id, p)| id == "wf" && *p == SunsetPhase::PrunePending),
            "transitions log missing PrunePending: {:?}",
            bank.transitions
        );
    }

    // rationale: Anti-property — Step 2.5 MUST NOT emit PrunePending for
    // healthy workflows (weight ≥ sunset_threshold).
    #[test]
    fn step_2_5_does_not_emit_prune_pending_for_healthy() {
        let now = 1_700_000_000_000;
        let (mut bank, pathways, freq) = one_wf_bank(0.95); // well above sunset
        let cfg = DecayConfig::default();
        let stats =
            run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
                .expect("cycle ok");
        assert_eq!(stats.workflows_prune_pending, 0);
        assert!(
            bank.transitions.is_empty(),
            "healthy workflow generated transitions: {:?}",
            bank.transitions
        );
    }

    // rationale: Anti-property — Step 2.5 MUST NOT emit PrunePending for
    // workflows BELOW the prune_threshold (those go to hard prune in
    // Step 3, not the soft band).
    #[test]
    fn step_2_5_does_not_emit_prune_pending_below_prune_threshold() {
        let now = 1_700_000_000_000;
        let (mut bank, pathways, freq) = one_wf_bank(0.005); // below prune (0.05, F4)
        let cfg = DecayConfig::default();
        let stats =
            run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
                .expect("cycle ok");
        assert_eq!(stats.workflows_prune_pending, 0);
        // No sunset_at set → workflow does not auto-sunset either; stays Active
        // until either bank consumer transitions it or a sunset_at is set.
    }

    // rationale: Contract regression — double-count guard. A workflow with
    // weight < prune_threshold AND sunset_at < now MUST be counted in
    // EXACTLY ONE of {workflows_pruned, workflows_auto_sunset}.
    #[test]
    fn pruned_workflow_not_double_counted_in_auto_sunset() {
        let now = 1_700_000_000_000;
        let mut bank = MockBank {
            active: vec![AcceptedWorkflowDecay {
                workflow_id: "wf_both".into(),
                pathway_id: "pw".into(),
                last_run_ms: now - 1000,
            }],
            // 0.005 < prune_threshold 0.05 (F4) → Step 3 hits
            weights: HashMap::from([(String::from("wf_both"), 0.005)]),
            // sunset_at < now → Step 4 would ALSO hit but the new guard
            // (pruned_this_cycle) prevents the double-count.
            sunsets: HashMap::from([(String::from("wf_both"), now - 100)]),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([(String::from("pw"), 0.0)]));
        let freq = MockFreq {
            counts: HashMap::from([(String::from("wf_both"), 0)]),
            cohort_max: 1,
        };
        let cfg = DecayConfig::default();
        let stats =
            run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
                .expect("cycle ok");
        // Exactly one of the two should fire — the prune.
        assert_eq!(stats.workflows_pruned, 1);
        assert_eq!(stats.workflows_auto_sunset, 0, "double-count regression");
        assert_eq!(bank.prunes, vec!["wf_both"]);
        assert!(
            !bank
                .transitions
                .iter()
                .any(|(id, p)| id == "wf_both" && *p == SunsetPhase::SunsetExpired),
            "SunsetExpired must not fire on already-pruned workflow"
        );
    }

    // rationale: Contract regression — transactional pre-fetch. A
    // PathwayReadFailed on workflow #N MUST NOT leave workflows #1..N-1
    // in a half-decayed state. Pre-fix, decay was applied workflow-by-
    // workflow and Workflow #1's weight would be mutated before #N's read
    // failed.
    #[test]
    fn pathway_read_failure_leaves_no_partial_state() {
        let now = 1_700_000_000_000;
        let mut bank = MockBank {
            active: vec![
                AcceptedWorkflowDecay {
                    workflow_id: "wf_good".into(),
                    pathway_id: "pw_good".into(),
                    last_run_ms: now - 1000,
                },
                AcceptedWorkflowDecay {
                    workflow_id: "wf_bad".into(),
                    pathway_id: "pw_missing".into(),
                    last_run_ms: now - 1000,
                },
            ],
            weights: HashMap::from([
                (String::from("wf_good"), 1.0),
                (String::from("wf_bad"), 1.0),
            ]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        // pw_missing not in the map → second pre-fetch fails.
        let pathways = MockPathways(HashMap::from([(String::from("pw_good"), 0.5)]));
        let freq = MockFreq {
            counts: HashMap::new(),
            cohort_max: 1,
        };
        let cfg = DecayConfig::default();
        let err = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .unwrap_err();
        assert!(matches!(err, DecayError::PathwayReadFailed { .. }));
        // CRITICAL: wf_good must STILL be at 1.0 — no decay was applied
        // because the read for wf_bad failed BEFORE Step 1.
        assert!((bank.weight_of("wf_good").unwrap() - 1.0).abs() < 1e-12,
            "wf_good was decayed despite wf_bad's read failure — partial state regression");
    }

    // rationale: Boundary — Step 4 must STILL fire for workflows that are
    // expired AND not below prune_threshold (the new pruned_this_cycle
    // guard must not over-block).
    #[test]
    fn auto_sunset_still_fires_for_expired_non_pruned_workflow() {
        let now = 1_700_000_000_000;
        let mut bank = MockBank {
            active: vec![AcceptedWorkflowDecay {
                workflow_id: "wf_exp".into(),
                pathway_id: "pw".into(),
                last_run_ms: now - 1000,
            }],
            // Above prune (so Step 3 won't fire) but expired by Step 4.
            weights: HashMap::from([(String::from("wf_exp"), 0.5)]),
            sunsets: HashMap::from([(String::from("wf_exp"), now - 100)]),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([(String::from("pw"), 0.5)]));
        let freq = MockFreq {
            counts: HashMap::new(),
            cohort_max: 1,
        };
        let cfg = DecayConfig::default();
        let stats =
            run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
                .expect("cycle ok");
        assert_eq!(stats.workflows_pruned, 0);
        assert_eq!(stats.workflows_auto_sunset, 1);
        assert!(
            bank.transitions
                .iter()
                .any(|(id, p)| id == "wf_exp" && *p == SunsetPhase::SunsetExpired)
        );
    }

    // rationale: Determinism — repeat-invocation parity on a stable bank
    // configuration; m11 has no internal RNG, so two runs produce
    // identical stats.
    #[test]
    fn cycle_is_deterministic_under_repeat() {
        let now = 1_700_000_000_000;
        let cfg = DecayConfig::default();
        let (mut bank_a, pathways_a, freq_a) = one_wf_bank(0.5);
        let stats_a =
            run_consolidation_cycle(&mut bank_a, &pathways_a, &freq_a, &cfg, || Some(now))
                .expect("ok a");
        let (mut bank_b, pathways_b, freq_b) = one_wf_bank(0.5);
        let stats_b =
            run_consolidation_cycle(&mut bank_b, &pathways_b, &freq_b, &cfg, || Some(now))
                .expect("ok b");
        assert_eq!(stats_a.workflows_decayed, stats_b.workflows_decayed);
        assert_eq!(stats_a.workflows_pruned, stats_b.workflows_pruned);
        assert!((stats_a.mean_decay_factor - stats_b.mean_decay_factor).abs() < 1e-12);
    }

    // rationale: Adversarial input — empty bank. iter_active returns []
    // and the cycle must complete with zeroed stats AND cycles_run == 1.
    #[test]
    fn empty_bank_completes_with_zero_decay_stats() {
        let now = 1_700_000_000_000;
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
        let stats =
            run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
                .expect("ok empty");
        assert_eq!(stats.cycles_run, 1);
        assert_eq!(stats.workflows_decayed, 0);
        assert_eq!(stats.workflows_pruned, 0);
        assert_eq!(stats.workflows_auto_sunset, 0);
        assert_eq!(stats.workflows_prune_pending, 0);
        assert_eq!(stats.workflows_clock_skew_skipped, 0);
        // mean_decay_factor stays at default 0.0 when no workflows decayed.
        assert!((stats.mean_decay_factor - 0.0).abs() < 1e-12);
    }

    // ====================================================================
    // W4 mutation-kill pass (S1003529) — targeted tests for cargo-mutants
    // SURVIVING mutants in run_consolidation_cycle + run_state_machine_
    // transitions. Each test pins exact arithmetic / comparison behaviour
    // with hand-computed expectations so a mutated operator fails loudly.
    // ====================================================================

    // rationale: Arithmetic mutant kill — `consolidation.rs:238` computes
    // `days = elapsed_ms as f64 / (1000.0 * 86400.0)`. Three surviving
    // mutants: `/`→`*` (238:38), `*`→`+` and `*`→`/` (238:48). At a known
    // 30-day elapsed the correct `days` is exactly 30.0 → recency_factor =
    // 0.5 (one half-life) → with frequency=1, fitness=1 the compound signal
    // cs = 1×1×0.5 = 0.5 → factor = 0.98 + 0.02×0.5 = 0.99 → post-decay
    // weight = 1.0 × 0.99 = 0.99. Every mutant inflates `days` enormously
    // (`/`→`*` multiplies by ~8.6e10; `*`→`+` divides by 87 400 ≈ days
    // 29 657; `*`→`/` divides by ~0.0116 ≈ days 2.2e11) so recency
    // collapses to ~0 → cs ≈ 0 → factor = base_rate = 0.98. The exact
    // 0.99-vs-0.98 split kills all three.
    #[test]
    fn decay_at_thirty_day_elapsed_yields_half_life_recency_factor() {
        // 30 days expressed in ms: 30 * 86_400_000 = 2_592_000_000.
        let now = 1_700_000_000_000_i64;
        let last_run = now - 2_592_000_000_i64;
        let mut bank = MockBank {
            active: vec![make_active("wf_30d", "pw_30d", last_run)],
            weights: HashMap::from([(String::from("wf_30d"), 1.0)]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        // fitness 1.0 (pathway weight 1.0) and frequency 1.0 (count ==
        // cohort_max) isolate recency as the only sub-unity signal.
        let pathways = MockPathways(HashMap::from([(String::from("pw_30d"), 1.0)]));
        let freq = MockFreq {
            counts: HashMap::from([(String::from("wf_30d"), 10)]),
            cohort_max: 10,
        };
        let cfg = DecayConfig::default(); // recency_half_life_days = 30.0
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle ok");
        assert_eq!(stats.workflows_decayed, 1);
        // CORRECT arithmetic: days = 2_592_000_000 / 86_400_000 = 30.0
        // exactly = one half-life → recency = 0.5 → cs = 0.5 → factor =
        // 0.98 + 0.02*0.5 = 0.99. Any 238 mutant drives factor to 0.98.
        let w = bank.weight_of("wf_30d").expect("weight");
        assert!(
            (w - 0.99).abs() < 1e-12,
            "expected post-decay weight 0.99 (factor 0.99 at 30-day = \
             one half-life); got {w} — a :238 arithmetic mutant survived",
        );
        // Tighten: a 238 mutant produces base_rate 0.98 exactly; assert
        // the weight is strictly above that floor.
        assert!(
            w > 0.985,
            "weight {w} collapsed toward base_rate 0.98 — recency signal \
             was lost (`days` mis-computed by a :238 mutant)",
        );
    }

    // rationale: Arithmetic mutant kill — second independent witness for
    // `consolidation.rs:238` at a 60-day elapsed (two half-lives). Correct
    // `days` = 60.0 → recency = 0.25 → cs = 0.25 → factor = 0.98 +
    // 0.02*0.25 = 0.985. A distinct expected value from the 30-day test
    // above guards against a mutant that happens to coincidentally land on
    // 0.99 for one specific input.
    #[test]
    fn decay_at_sixty_day_elapsed_yields_two_half_life_recency_factor() {
        // 60 days in ms: 60 * 86_400_000 = 5_184_000_000.
        let now = 1_700_000_000_000_i64;
        let last_run = now - 5_184_000_000_i64;
        let mut bank = MockBank {
            active: vec![make_active("wf_60d", "pw_60d", last_run)],
            weights: HashMap::from([(String::from("wf_60d"), 1.0)]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([(String::from("pw_60d"), 1.0)]));
        let freq = MockFreq {
            counts: HashMap::from([(String::from("wf_60d"), 10)]),
            cohort_max: 10,
        };
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle ok");
        assert_eq!(stats.workflows_decayed, 1);
        // days = 5_184_000_000 / 86_400_000 = 60.0 = two half-lives →
        // recency = 0.25 → cs = 0.25 → factor = 0.985.
        let w = bank.weight_of("wf_60d").expect("weight");
        assert!(
            (w - 0.985).abs() < 1e-9,
            "expected post-decay weight 0.985 (factor 0.985 at 60-day = \
             two half-lives); got {w} — a :238 arithmetic mutant survived",
        );
    }

    // rationale: Counter mutant kill — `consolidation.rs:243` increments
    // `stats.workflows_decayed += 1`. The surviving `+=`→`-=` mutant would
    // make the counter wrap (usize underflow). This test pins the EXACT
    // count for a three-workflow bank where all three decay: correct =
    // 3; `-=` yields a wrapped huge value (usize::MAX-class). The strict
    // `== 3` equality fires the canary.
    #[test]
    fn workflows_decayed_counter_increments_exactly_once_per_workflow() {
        let now = 1_700_000_000_000_i64;
        let mut bank = MockBank {
            active: vec![
                make_active("wf_d1", "pw_d1", now),
                make_active("wf_d2", "pw_d2", now),
                make_active("wf_d3", "pw_d3", now),
            ],
            weights: HashMap::from([
                (String::from("wf_d1"), 1.0),
                (String::from("wf_d2"), 1.0),
                (String::from("wf_d3"), 1.0),
            ]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([
            (String::from("pw_d1"), 0.5),
            (String::from("pw_d2"), 0.5),
            (String::from("pw_d3"), 0.5),
        ]));
        let freq = MockFreq {
            counts: HashMap::from([
                (String::from("wf_d1"), 1),
                (String::from("wf_d2"), 1),
                (String::from("wf_d3"), 1),
            ]),
            cohort_max: 1,
        };
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle ok");
        // EXACTLY 3 — a `+=`→`-=` mutant wraps to a usize::MAX-class value.
        assert_eq!(
            stats.workflows_decayed, 3,
            "workflows_decayed must equal the number of decayed workflows; \
             a non-3 value means the :243 += counter was mutated",
        );
        // The aggregate factor stats must also reflect 3 non-empty samples.
        assert!(stats.mean_decay_factor.is_finite());
        assert!(stats.min_decay_factor.is_finite());
        assert!(stats.max_decay_factor.is_finite());
    }

    /// One-workflow bank whose decay is an exact no-op (`factor == 1.0`).
    /// Pathway weight 1.0 (fitness 1.0) + frequency at cohort_max (1.0) +
    /// `last_run_ms == now` (recency 1.0) → compound signal 1.0 → decay
    /// factor 1.0 → post-decay weight EQUALS the supplied pre-decay
    /// `weight`. This lets a boundary test place the post-decay weight at
    /// an exact threshold value without a fragile `weight / factor`
    /// pre-image.
    fn one_wf_bank_no_decay(weight: f64) -> (MockBank, MockPathways, MockFreq) {
        let bank = MockBank {
            active: vec![AcceptedWorkflowDecay {
                workflow_id: "wf".into(),
                pathway_id: "pw".into(),
                last_run_ms: 1_700_000_000_000,
            }],
            weights: HashMap::from([(String::from("wf"), weight)]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        // pathway weight 1.0 → fitness 1.0.
        let pathways = MockPathways(HashMap::from([(String::from("pw"), 1.0)]));
        // count == cohort_max → frequency 1.0.
        let freq = MockFreq {
            counts: HashMap::from([(String::from("wf"), 10)]),
            cohort_max: 10,
        };
        (bank, pathways, freq)
    }

    // rationale: Comparison mutant kill — `consolidation.rs:321` is the
    // Step 2.5 soft-band gate `weight < sunset_threshold && weight >=
    // prune_threshold`. Two surviving mutants: `<`→`<=` (321:19) and
    // `>=`→`<` (321:55). At weight EXACTLY equal to sunset_threshold
    // (0.10), the correct `<` is FALSE → no PrunePending; a `<=` mutant
    // makes it TRUE → spurious PrunePending. This boundary test pins the
    // exclusive upper bound. Decay is a no-op (factor 1.0 via
    // one_wf_bank_no_decay) so the post-decay weight Step 2.5 reads is
    // exactly 0.10.
    #[test]
    fn soft_band_upper_bound_is_exclusive_at_sunset_threshold() {
        let now = 1_700_000_000_000_i64;
        // post-decay weight EXACTLY at the soft floor (0.10, F4).
        let (mut bank, pathways, freq) = one_wf_bank_no_decay(0.10);
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle ok");
        // Sanity: decay is a no-op, so the weight Step 2.5 reads is 0.10.
        assert!(
            (bank.weight_of("wf").unwrap_or(0.0) - 0.10).abs() < 1e-12,
            "decay must be a no-op for this boundary fixture",
        );
        // `weight < sunset_threshold` is FALSE at weight == threshold →
        // workflow stays Active. A `<`→`<=` mutant fires PrunePending here.
        assert_eq!(
            stats.workflows_prune_pending, 0,
            "weight exactly at sunset_threshold (0.10) must NOT enter the \
             soft band — the `<` at :321 is strictly exclusive",
        );
        assert!(
            bank.transitions.is_empty(),
            "no transition expected at the exact soft-floor boundary: {:?}",
            bank.transitions,
        );
    }

    // rationale: Comparison mutant kill — `consolidation.rs:321:55` is the
    // `weight >= prune_threshold` lower bound of the soft band. At weight
    // EXACTLY equal to prune_threshold (0.05), the correct `>=` is TRUE →
    // the workflow IS in the soft band → PrunePending fires. A `>=`→`<`
    // mutant makes it FALSE → no PrunePending. This boundary test pins the
    // inclusive lower bound. Decay is a no-op (factor 1.0) so the
    // post-decay weight Step 2.5 reads is exactly 0.05.
    #[test]
    fn soft_band_lower_bound_is_inclusive_at_prune_threshold() {
        let now = 1_700_000_000_000_i64;
        // post-decay weight EXACTLY at the hard floor (0.05, F4). No
        // sunset_at set so Step 3 hard-prune cannot fire (it gates on
        // sunset_at_of(...).is_some()), isolating the Step 2.5 lower edge.
        let (mut bank, pathways, freq) = one_wf_bank_no_decay(0.05);
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle ok");
        assert!(
            (bank.weight_of("wf").unwrap_or(0.0) - 0.05).abs() < 1e-12,
            "decay must be a no-op for this boundary fixture",
        );
        // `weight >= prune_threshold` is TRUE at weight == threshold, and
        // `weight < sunset_threshold` is TRUE (0.05 < 0.10) → soft band.
        assert_eq!(
            stats.workflows_prune_pending, 1,
            "weight exactly at prune_threshold (0.05) IS inside the soft \
             band — the `>=` at :321 is inclusive of the lower bound",
        );
        assert!(
            bank.transitions
                .iter()
                .any(|(id, p)| id == "wf" && *p == SunsetPhase::PrunePending),
            "PrunePending transition missing at the prune_threshold edge: {:?}",
            bank.transitions,
        );
    }

    // rationale: Counter mutant kill — `consolidation.rs:323` increments
    // `stats.workflows_prune_pending += 1`. Surviving mutants `+=`→`-=`
    // and `+=`→`*=`. With a two-workflow bank both in the soft band the
    // correct count is 2; `-=` underflow-wraps, `*=` would yield 0 (0*1)
    // then 0 (0*1) → stays 0. The exact `== 2` kills both.
    #[test]
    fn prune_pending_counter_increments_exactly_once_per_soft_band_workflow() {
        let now = 1_700_000_000_000_i64;
        let mut bank = MockBank {
            active: vec![
                make_active("wf_sb1", "pw1", now),
                make_active("wf_sb2", "pw2", now),
            ],
            // Both in the soft band [0.05, 0.10): 0.07 and 0.08.
            weights: HashMap::from([
                (String::from("wf_sb1"), 0.07),
                (String::from("wf_sb2"), 0.08),
            ]),
            sunsets: HashMap::new(),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([
            (String::from("pw1"), 0.5),
            (String::from("pw2"), 0.5),
        ]));
        let freq = MockFreq {
            counts: HashMap::new(),
            cohort_max: 1,
        };
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle ok");
        // EXACTLY 2 — `+=`→`-=` underflows, `+=`→`*=` stays 0.
        assert_eq!(
            stats.workflows_prune_pending, 2,
            "two soft-band workflows must yield prune_pending == 2; a \
             non-2 value means the :323 += counter was mutated",
        );
    }

    // rationale: Comparison mutant kill — `consolidation.rs:348` is the
    // Step 4 auto-sunset gate `sunset_at < now_ms`. Surviving mutant
    // `<`→`<=`. At `sunset_at` EXACTLY equal to `now_ms`, the correct `<`
    // is FALSE → no SunsetExpired; a `<=` mutant makes it TRUE → spurious
    // auto-sunset. This boundary test pins the strict-less-than semantics
    // (a workflow whose sunset boundary is "right now" has not yet
    // expired — expiry is strictly past-tense).
    #[test]
    fn auto_sunset_gate_is_strict_at_exact_sunset_boundary() {
        let now = 1_700_000_000_000_i64;
        let mut bank = MockBank {
            active: vec![make_active("wf_edge", "pw", now)],
            // Above prune_threshold so Step 3 cannot fire — isolates Step 4.
            weights: HashMap::from([(String::from("wf_edge"), 0.5)]),
            // sunset_at EXACTLY == now_ms.
            sunsets: HashMap::from([(String::from("wf_edge"), now)]),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([(String::from("pw"), 0.5)]));
        let freq = MockFreq {
            counts: HashMap::new(),
            cohort_max: 1,
        };
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle ok");
        // `sunset_at < now_ms` is FALSE when sunset_at == now_ms → no
        // auto-sunset. A `<`→`<=` mutant fires SunsetExpired here.
        assert_eq!(
            stats.workflows_auto_sunset, 0,
            "sunset_at exactly equal to now_ms must NOT auto-sunset — the \
             `<` at :348 is strictly exclusive of the present instant",
        );
        assert!(
            !bank
                .transitions
                .iter()
                .any(|(id, p)| id == "wf_edge" && *p == SunsetPhase::SunsetExpired),
            "SunsetExpired must not fire at the exact sunset boundary: {:?}",
            bank.transitions,
        );
    }

    // rationale: Counter mutant kill — `consolidation.rs:350` increments
    // `stats.workflows_auto_sunset += 1`. Surviving mutants `+=`→`-=` and
    // `+=`→`*=`. With a two-workflow bank both expired the correct count
    // is 2; `-=` underflow-wraps, `*=` stays 0. The exact `== 2` kills
    // both. Both workflows are above prune_threshold and have no
    // sunset-prune interaction, so Step 4 is the only counter touched.
    #[test]
    fn auto_sunset_counter_increments_exactly_once_per_expired_workflow() {
        let now = 1_700_000_000_000_i64;
        let mut bank = MockBank {
            active: vec![
                make_active("wf_e1", "pw1", now),
                make_active("wf_e2", "pw2", now),
            ],
            // Above prune_threshold (0.5) → Step 3 hard-prune skipped.
            weights: HashMap::from([
                (String::from("wf_e1"), 0.5),
                (String::from("wf_e2"), 0.5),
            ]),
            // Both expired: sunset_at strictly < now.
            sunsets: HashMap::from([
                (String::from("wf_e1"), now - 1),
                (String::from("wf_e2"), now - 1000),
            ]),
            transitions: vec![],
            prunes: vec![],
        };
        let pathways = MockPathways(HashMap::from([
            (String::from("pw1"), 0.5),
            (String::from("pw2"), 0.5),
        ]));
        let freq = MockFreq {
            counts: HashMap::new(),
            cohort_max: 1,
        };
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle ok");
        assert_eq!(stats.workflows_pruned, 0, "no hard prune expected");
        // EXACTLY 2 — `+=`→`-=` underflows, `+=`→`*=` stays 0.
        assert_eq!(
            stats.workflows_auto_sunset, 2,
            "two expired workflows must yield auto_sunset == 2; a non-2 \
             value means the :350 += counter was mutated",
        );
    }

    // rationale: Resource accounting — pre-fetch buffer uses
    // `with_capacity(workflows.len())` to avoid grow-realloc on hot path.
    // We test the OBSERVABLE behaviour: a bank with N workflows runs N
    // pathway reads even when later reads fail. (Pre-fix: reads were
    // interleaved with mutations; a failure on read #3 ran only reads #1
    // through #3. The new pre-fetch runs reads #1 through #N or fails on
    // the FIRST failure, whichever comes first.)
    //
    // This test verifies the first-failure-is-fatal behaviour: with the
    // failure ordered at position 1, reads #2 and #3 never run.
    #[test]
    fn pre_fetch_short_circuits_on_first_pathway_read_failure() {
        let now = 1_700_000_000_000;
        let mut bank = MockBank {
            active: vec![
                AcceptedWorkflowDecay {
                    workflow_id: "wf_a".into(),
                    pathway_id: "pw_missing".into(), // FIRST will fail
                    last_run_ms: now - 1000,
                },
                AcceptedWorkflowDecay {
                    workflow_id: "wf_b".into(),
                    pathway_id: "pw_b".into(),
                    last_run_ms: now - 1000,
                },
                AcceptedWorkflowDecay {
                    workflow_id: "wf_c".into(),
                    pathway_id: "pw_c".into(),
                    last_run_ms: now - 1000,
                },
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
            (String::from("pw_b"), 0.5),
            (String::from("pw_c"), 0.5),
            // pw_missing absent
        ]));
        let freq = MockFreq {
            counts: HashMap::new(),
            cohort_max: 1,
        };
        let cfg = DecayConfig::default();
        let err = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .unwrap_err();
        let DecayError::PathwayReadFailed { pathway_id, .. } = err else {
            panic!("wrong error type");
        };
        // Failure must name the FIRST missing pathway (wf_a's pw_missing).
        assert_eq!(pathway_id, "pw_missing");
        // None of the workflows must have been decayed.
        for id in ["wf_a", "wf_b", "wf_c"] {
            assert!((bank.weight_of(id).unwrap() - 1.0).abs() < 1e-12,
                "{id} was mutated despite Step-0 short-circuit");
        }
    }

    // rationale: Comparison mutant kill — `consolidation.rs:335:19` is the
    // Step 3 hard-prune gate `weight < config.prune_threshold && sunset_at
    // .is_some()`. SURVIVING mutant `<`→`<=` (335:19). The discriminating
    // input is the equality boundary: post-decay weight EXACTLY equal to
    // `prune_threshold` (0.05, F4) AND an explicit `sunset_at` set so the
    // second `&&` conjunct is satisfied.
    //   real `<`  : `0.05 < 0.05` is FALSE → hard prune does NOT fire →
    //               workflows_pruned == 0, bank.prunes empty.
    //   `<=` mut  : `0.05 <= 0.05` is TRUE → `&& sunset_at.is_some()`
    //               (true) → mark_for_prune fires → workflows_pruned == 1,
    //               bank.prunes == ["wf_edge"].
    // The exact `workflows_pruned == 0` + empty `bank.prunes` assertion
    // fires the canary: a `<=` mutant produces 1 / ["wf_edge"].
    //
    // `sunset_at` is set to a FUTURE instant (`now + 1 day`) so Step 4
    // auto-sunset (`sunset_at < now`) cannot fire and confound the count.
    // Decay is a no-op (factor 1.0 via one_wf_bank_no_decay-style fixture)
    // so the post-decay weight Step 3 reads is exactly 0.05. Step 2.5
    // PrunePending DOES fire here (0.05 is inside the soft band
    // [0.05, 0.10)) — that is the `workflows_prune_pending` counter, a
    // different field, and is asserted == 1 to document the expected
    // co-occurring soft transition.
    #[test]
    fn hard_prune_gate_is_strict_at_exact_prune_threshold_boundary() {
        let now = 1_700_000_000_000_i64;
        let mut bank = MockBank {
            active: vec![make_active("wf_edge", "pw", now)],
            // post-decay weight EXACTLY at the hard floor (0.05, F4).
            weights: HashMap::from([(String::from("wf_edge"), 0.05)]),
            // Explicit sunset set so the `&& sunset_at.is_some()` conjunct
            // of the :335 gate is satisfied — FUTURE so Step 4 cannot fire.
            sunsets: HashMap::from([(String::from("wf_edge"), now + 86_400_000)]),
            transitions: vec![],
            prunes: vec![],
        };
        // pathway weight 1.0 → fitness 1.0; count == cohort_max → freq 1.0;
        // last_run_ms == now → recency 1.0 → compound 1.0 → decay factor
        // 1.0 → post-decay weight stays EXACTLY 0.05.
        let pathways = MockPathways(HashMap::from([(String::from("pw"), 1.0)]));
        let freq = MockFreq {
            counts: HashMap::from([(String::from("wf_edge"), 10)]),
            cohort_max: 10,
        };
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle ok");
        // Decay must be a no-op so Step 3 reads exactly 0.05.
        assert!(
            (bank.weight_of("wf_edge").unwrap_or(0.0) - 0.05).abs() < 1e-12,
            "decay must be a no-op for this boundary fixture",
        );
        // `weight < prune_threshold` is FALSE at weight == threshold →
        // hard prune does NOT fire. A `<`→`<=` mutant at :335 fires it.
        assert_eq!(
            stats.workflows_pruned, 0,
            "weight exactly at prune_threshold (0.05) must NOT hard-prune \
             — the `<` at :335 is strictly exclusive of the boundary",
        );
        assert!(
            bank.prunes.is_empty(),
            "no mark_for_prune expected at the exact hard-floor boundary; \
             a :335 `<`→`<=` mutant marks wf_edge: {:?}",
            bank.prunes,
        );
        // Documented co-occurring soft transition: 0.05 IS inside the
        // soft band [0.05, 0.10) so Step 2.5 PrunePending fires (separate
        // counter — pins that the test exercises the real control flow).
        assert_eq!(
            stats.workflows_prune_pending, 1,
            "0.05 is inside the soft band → exactly one PrunePending",
        );
    }
}
