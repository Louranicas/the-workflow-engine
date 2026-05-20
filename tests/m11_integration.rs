//! Integration tests for m11 fitness_weighted_decay.
//!
//! Per m11 spec § 6 F-Integration row (15 tests target). Exercises:
//!
//! - End-to-end consolidation cycles with mock bank + reader trait impls.
//! - Multi-workflow concurrent decay.
//! - Cycle re-entrancy (monotonically decreasing on unchanged signals).
//! - Supervisor-restart simulation (m11 task is stateless; state lives in
//!   the bank).
//! - Cohort edge cases (empty / single-workflow).

#![allow(clippy::doc_markdown)]

use std::collections::HashMap;

use workflow_core::m11_fitness_weighted_decay::{
    chrono_now_ms, compute_decay_factor, fitness_factor, frequency_factor, recency_factor,
    run_consolidation_cycle, AcceptedWorkflowDecay, DecayConfig, DecayError, DecayFactor,
    FrequencyReader, LifecycleBank, PathwayWeightReader, SunsetPhase,
};

// ---- Mocks ---------------------------------------------------------------

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

fn build_n_workflow_bank(n: usize, now: i64) -> (MockBank, MockPathways, MockFreq) {
    let active: Vec<AcceptedWorkflowDecay> = (0..n)
        .map(|i| make_active(&format!("wf_{i}"), &format!("pw_{i}"), now))
        .collect();
    let weights: HashMap<String, f64> = (0..n)
        .map(|i| (format!("wf_{i}"), 1.0))
        .collect();
    let pathways: HashMap<String, f64> = (0..n)
        .map(|i| (format!("pw_{i}"), 0.5))
        .collect();
    let counts: HashMap<String, u64> = (0..n)
        .map(|i| (format!("wf_{i}"), (i as u64) + 1))
        .collect();
    let cohort_max = n as u64;
    let bank = MockBank {
        active,
        weights,
        sunsets: HashMap::new(),
        transitions: Vec::new(),
        prunes: Vec::new(),
    };
    (bank, MockPathways(pathways), MockFreq { counts, cohort_max })
}

// ---- Cycle smoke (3) -----------------------------------------------------

#[test]
fn end_to_end_cycle_decays_three_workflows() {
    let now = 1_700_000_000_000_i64;
    let (mut bank, pathways, freq) = build_n_workflow_bank(3, now);
    let cfg = DecayConfig::default();
    let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
        .expect("cycle");
    assert_eq!(stats.workflows_decayed, 3);
    assert_eq!(stats.cycles_run, 1);
}

#[test]
fn end_to_end_cycle_records_stats_with_min_le_mean_le_max() {
    let now = 1_700_000_000_000_i64;
    let (mut bank, pathways, freq) = build_n_workflow_bank(5, now);
    let cfg = DecayConfig::default();
    let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
        .expect("cycle");
    assert!(stats.min_decay_factor.is_finite());
    assert!(stats.max_decay_factor.is_finite());
    assert!(stats.min_decay_factor <= stats.mean_decay_factor + 1e-12);
    assert!(stats.mean_decay_factor <= stats.max_decay_factor + 1e-12);
}

#[test]
fn end_to_end_cycle_no_workflows_yields_zero_stats() {
    let now = 1_700_000_000_000_i64;
    let mut bank = MockBank {
        active: vec![],
        weights: HashMap::new(),
        sunsets: HashMap::new(),
        transitions: Vec::new(),
        prunes: Vec::new(),
    };
    let pathways = MockPathways(HashMap::new());
    let freq = MockFreq {
        counts: HashMap::new(),
        cohort_max: 0,
    };
    let cfg = DecayConfig::default();
    let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
        .expect("empty cycle");
    assert_eq!(stats.workflows_decayed, 0);
}

// ---- Cycle re-entrancy (3) -----------------------------------------------

#[test]
fn re_entrant_cycle_monotonically_decreases_weight_for_stale_signals() {
    let now = 1_700_000_000_000_i64;
    let mut bank = MockBank {
        active: vec![make_active("wf_stale", "pw_stale", now)],
        weights: HashMap::from([(String::from("wf_stale"), 1.0)]),
        sunsets: HashMap::new(),
        transitions: Vec::new(),
        prunes: Vec::new(),
    };
    let pathways = MockPathways(HashMap::from([(String::from("pw_stale"), 0.0)]));
    let freq = MockFreq {
        counts: HashMap::from([(String::from("wf_stale"), 0)]),
        cohort_max: 10,
    };
    let cfg = DecayConfig::default();
    let mut prev = bank.weight_of("wf_stale").unwrap();
    for _ in 0..10_u32 {
        run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now)).expect("cycle");
        let w = bank.weight_of("wf_stale").unwrap();
        assert!(w < prev, "weight should monotonically decrease (prev={prev} now={w})");
        prev = w;
    }
}

#[test]
fn re_entrant_cycle_preserves_thriving_workflow_weight() {
    let now = 1_700_000_000_000_i64;
    let mut bank = MockBank {
        active: vec![make_active("wf_thrive", "pw_thrive", now)],
        weights: HashMap::from([(String::from("wf_thrive"), 1.0)]),
        sunsets: HashMap::new(),
        transitions: Vec::new(),
        prunes: Vec::new(),
    };
    let pathways = MockPathways(HashMap::from([(String::from("pw_thrive"), 1.0)]));
    let freq = MockFreq {
        counts: HashMap::from([(String::from("wf_thrive"), 10)]),
        cohort_max: 10,
    };
    let cfg = DecayConfig::default();
    for _ in 0..50_u32 {
        run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now)).expect("cycle");
    }
    let w = bank.weight_of("wf_thrive").unwrap();
    assert!(
        (w - 1.0).abs() < 1e-9,
        "thriving workflow weight should remain ~1.0 after 50 cycles, got {w}"
    );
}

#[test]
fn re_entrant_cycle_228_iterations_reaches_prune_threshold() {
    // Per m11 spec § 5.3 calibration: 0.98^228 < 0.01.
    let now = 1_700_000_000_000_i64;
    let mut bank = MockBank {
        active: vec![make_active("wf_floor", "pw_floor", now)],
        weights: HashMap::from([(String::from("wf_floor"), 1.0)]),
        sunsets: HashMap::new(),
        transitions: Vec::new(),
        prunes: Vec::new(),
    };
    let pathways = MockPathways(HashMap::from([(String::from("pw_floor"), 0.0)]));
    let freq = MockFreq {
        counts: HashMap::from([(String::from("wf_floor"), 0)]),
        cohort_max: 10,
    };
    let cfg = DecayConfig::default();
    for _ in 0..230_u32 {
        run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now)).expect("cycle");
    }
    let w = bank.weight_of("wf_floor").unwrap();
    assert!(
        w < cfg.prune_threshold,
        "after 230 cycles weight {w} should be below prune_threshold {prune}",
        prune = cfg.prune_threshold
    );
}

// ---- Supervisor-restart simulation (1) -----------------------------------

#[test]
fn supervisor_restart_simulation_state_lives_in_bank_not_m11() {
    // m11's consolidation cycle is stateless: state lives in the bank. We
    // simulate a "supervisor restart" by dropping and recreating the
    // cycle closure between iterations — the bank's weights persist.
    let now = 1_700_000_000_000_i64;
    let mut bank = MockBank {
        active: vec![make_active("wf_persist", "pw_persist", now)],
        weights: HashMap::from([(String::from("wf_persist"), 1.0)]),
        sunsets: HashMap::new(),
        transitions: Vec::new(),
        prunes: Vec::new(),
    };
    let pathways = MockPathways(HashMap::from([(String::from("pw_persist"), 0.5)]));
    let freq = MockFreq {
        counts: HashMap::from([(String::from("wf_persist"), 1)]),
        cohort_max: 1,
    };
    let cfg = DecayConfig::default();
    // Iteration 1.
    run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now)).expect("c1");
    let w_after_1 = bank.weight_of("wf_persist").unwrap();
    // Simulate restart — recreate the closure; bank survives.
    let now_fn = || Some(now);
    run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, now_fn).expect("c2");
    let w_after_2 = bank.weight_of("wf_persist").unwrap();
    assert!(w_after_2 <= w_after_1);
}

// ---- Cohort edge cases (2) -----------------------------------------------

#[test]
fn cohort_max_zero_yields_frequency_zero_for_all() {
    assert!(frequency_factor(5, 0).abs() < 1e-12);
    assert!(frequency_factor(0, 0).abs() < 1e-12);
}

#[test]
fn cohort_of_one_workflow_normalises_to_unit() {
    let now = 1_700_000_000_000_i64;
    let mut bank = MockBank {
        active: vec![make_active("wf_solo", "pw_solo", now)],
        weights: HashMap::from([(String::from("wf_solo"), 1.0)]),
        sunsets: HashMap::new(),
        transitions: Vec::new(),
        prunes: Vec::new(),
    };
    let pathways = MockPathways(HashMap::from([(String::from("pw_solo"), 1.0)]));
    let freq = MockFreq {
        counts: HashMap::from([(String::from("wf_solo"), 7)]),
        cohort_max: 7,
    };
    let cfg = DecayConfig::default();
    run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now)).expect("cycle");
    // freq=1.0 (single in cohort), fit=1.0, rec=1.0 (now==last_run) →
    // factor=1.0 → weight preserved.
    let w = bank.weight_of("wf_solo").unwrap();
    assert!((w - 1.0).abs() < 1e-12);
}

// ---- Cross-module surface stability (3) ----------------------------------

#[test]
fn surface_stability_compute_decay_factor_signature() {
    let _: fn(f64, f64, f64, f64) -> Result<DecayFactor, DecayError> = compute_decay_factor;
}

#[test]
fn surface_stability_input_normalisers() {
    let _: fn(f64, f64) -> f64 = recency_factor;
    let _: fn(u64, u64) -> f64 = frequency_factor;
    let _: fn(f64) -> f64 = fitness_factor;
}

#[test]
fn surface_stability_sunset_phase_variants_are_exactly_three() {
    let p = SunsetPhase::Active;
    match p {
        SunsetPhase::Active | SunsetPhase::PrunePending | SunsetPhase::SunsetExpired => {}
    }
}

// ---- Day-1 contract anchors (2) ------------------------------------------

#[test]
fn day_1_chrono_now_ms_yields_post_2020_timestamp() {
    let v = chrono_now_ms().expect("clock");
    assert!(v > 1_500_000_000_000);
}

#[test]
fn day_1_clock_unavailable_skips_cycle_returns_typed_error() {
    let mut bank = MockBank {
        active: vec![],
        weights: HashMap::new(),
        sunsets: HashMap::new(),
        transitions: Vec::new(),
        prunes: Vec::new(),
    };
    let pathways = MockPathways(HashMap::new());
    let freq = MockFreq {
        counts: HashMap::new(),
        cohort_max: 0,
    };
    let cfg = DecayConfig::default();
    let err = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || None).unwrap_err();
    assert!(matches!(err, DecayError::ClockUnavailable));
}

// =============================================================================
// H9-rem — m11 consolidation cycle against REAL m30 CuratedBank
//   (NOT MockBank). Wave-C1 hardening; closes the headline H9 carry-forward
//   item "m11 (decay cycle correctness)" by exercising the m30↔m11 bridge
//   end-to-end with the production bank.
// =============================================================================

mod m30_real_bank {
    use super::{run_consolidation_cycle, DecayConfig, DecayError, FrequencyReader,
                PathwayWeightReader, SunsetPhase};
    use std::collections::HashMap;
    use std::time::SystemTime;
    use workflow_core::m14_lift::LiftSnapshot;
    use workflow_core::m20_prefixspan::{Pattern, StepToken};
    use workflow_core::m21_variant_builder::build_variants;
    use workflow_core::m23_proposer::{build_proposal, WorkflowProposal};
    use workflow_core::m30_bank::{
        CuratedBank, DEFAULT_PRUNE_PENDING_THRESHOLD, DEFAULT_PRUNE_THRESHOLD,
    };

    // ---- fixtures --------------------------------------------------------

    fn snap() -> LiftSnapshot {
        LiftSnapshot {
            lift: Some(0.5),
            ci_half: Some(0.05),
            n: 30,
            latest_ts_ms: 0,
            computed_at: SystemTime::now(),
        }
    }

    fn proposal_with_seed(seed: u32) -> WorkflowProposal {
        let p = Pattern::new(
            vec![StepToken(seed), StepToken(seed.wrapping_add(1))],
            30,
            (0, seed as usize),
        );
        let v = build_variants(&p).expect("variant")[0].clone();
        build_proposal(v, &snap(), None).expect("proposal")
    }

    /// Accept N workflows into a fresh `CuratedBank`. Returns the bank and
    /// the ordered list of accepted `workflow_id` values (u64).
    fn build_real_bank(n: u32, now_ms: i64) -> (CuratedBank, Vec<u64>) {
        let bank = CuratedBank::new();
        let mut ids = Vec::with_capacity(n as usize);
        for i in 0..n {
            // Use distinct seeds so each proposal hashes to a distinct
            // workflow_id; the m30 bank rejects duplicates.
            let id = bank.accept(proposal_with_seed(1000 + i), now_ms).expect("accept");
            ids.push(id);
        }
        (bank, ids)
    }

    struct StubPathways {
        weights: HashMap<String, f64>,
    }
    impl PathwayWeightReader for StubPathways {
        fn read_pathway_weight(&self, pid: &str) -> Result<f64, DecayError> {
            self.weights
                .get(pid)
                .copied()
                .ok_or_else(|| DecayError::PathwayReadFailed {
                    pathway_id: pid.to_owned(),
                    reason: "stub: not seeded".into(),
                })
        }
    }
    struct StubFreq {
        counts: HashMap<String, u64>,
        cohort_max: u64,
    }
    impl FrequencyReader for StubFreq {
        fn frequency(&self, wid: &str) -> u64 {
            self.counts.get(wid).copied().unwrap_or(0)
        }
        fn cohort_max(&self) -> u64 {
            self.cohort_max
        }
    }

    /// `StubPathways` + `StubFreq` seeded so every workflow has fitness=1.0
    /// (preserves weight under decay → exercise other invariants without
    /// fitness-side perturbation unless a test overrides).
    fn make_thriving_readers(ids: &[u64]) -> (StubPathways, StubFreq) {
        let weights: HashMap<String, f64> = ids.iter().map(|id| (id.to_string(), 1.0)).collect();
        let counts: HashMap<String, u64> = ids.iter().map(|id| (id.to_string(), 1)).collect();
        (
            StubPathways { weights },
            StubFreq {
                counts,
                cohort_max: 1,
            },
        )
    }

    // ---- T2 — H9-rem integration tests ----------------------------------

    #[test]
    fn m11_consolidation_against_curated_bank_decays_all_active() {
        // rationale: cross-module integration — the headline H9 carry-forward.
        // Run a full consolidation cycle against a real m30 `CuratedBank`
        // (no MockBank) and verify every active row had decay applied.
        let now = 1_700_000_000_000_i64;
        let (mut bank, ids) = build_real_bank(5, now);
        let (pathways, freq) = make_thriving_readers(&ids);
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle ok against real bank");
        assert_eq!(
            stats.workflows_decayed, 5,
            "all 5 real-bank workflows must be decayed in one cycle"
        );
        assert_eq!(stats.cycles_run, 1);
        // Thriving signals → factor 1.0 → weights stay at 1.0.
        for id in &ids {
            let w = bank.get(*id).expect("row").weight;
            assert!(
                (w - 1.0).abs() < 1e-12,
                "thriving workflow {id} should stay at 1.0, got {w}"
            );
        }
    }

    #[test]
    fn m11_consolidation_skips_clock_skewed_workflow_and_counts_it() {
        // rationale: anti-property (F-POVM-07 silent-zero) — a workflow
        // whose `last_run_ms` is future-dated relative to `now_ms` MUST
        // be skipped and surfaced in `workflows_clock_skew_skipped`, never
        // silently rewarded with recency=1.0 via the saturating-sub bug.
        //
        // m30's bridge surfaces `last_run_ms = w.last_run_ms.unwrap_or(w.accepted_at_ms)`.
        // To inject clock skew, record a run with a future timestamp before
        // the cycle.
        let now = 1_700_000_000_000_i64;
        let (mut bank, ids) = build_real_bank(2, now);
        // Mark id_0 with a future last_run; id_1 stays at accepted_at.
        bank.record_run(ids[0], now + 1_000_000); // 1000s into the future
        let (pathways, freq) = make_thriving_readers(&ids);
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle ok");
        assert_eq!(stats.workflows_clock_skew_skipped, 1, "must count clock-skew skip");
        assert_eq!(stats.workflows_decayed, 1, "non-skewed workflow still decayed");
    }

    #[test]
    fn m11_consolidation_emits_prune_pending_at_soft_floor() {
        // rationale: contract regression — Step 2.5 PrunePending arm fires
        // when post-decay weight is in `[prune_threshold, sunset_threshold)`.
        // We pre-drive the workflow into that band, then run the cycle.
        let now = 1_700_000_000_000_i64;
        let (mut bank, ids) = build_real_bank(1, now);
        // Soft floor is sunset_threshold=0.05 (m11 cfg, distinct from m30's
        // soft DEFAULT_PRUNE_PENDING_THRESHOLD=0.10). Hard floor
        // prune_threshold=0.01. Drive weight to 0.02 (in m11 soft band).
        bank.apply_decay(ids[0], 0.02);
        assert!((bank.get(ids[0]).expect("row").weight - 0.02).abs() < 1e-12);
        let (pathways, freq) = make_thriving_readers(&ids);
        let cfg = DecayConfig::default();
        let stats = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .expect("cycle ok");
        assert_eq!(
            stats.workflows_prune_pending, 1,
            "weight 0.02 is in [prune=0.01, sunset=0.05) — must mark PrunePending"
        );
    }

    #[test]
    fn m11_consolidation_double_count_guard_pruned_not_also_auto_sunset() {
        // rationale: anti-property (double-count regression) — a workflow
        // simultaneously below `prune_threshold` AND past its `sunset_at`
        // MUST be counted in EXACTLY ONE of {workflows_pruned,
        // workflows_auto_sunset}. m30's `sunset_at` is set at accept-time
        // to now + 120d; we drive an artificial near-expiry by force-
        // decaying weight to below prune_threshold AND moving the clock
        // past the sunset window.
        let now = 1_700_000_000_000_i64;
        let (mut bank, ids) = build_real_bank(1, now);
        bank.apply_decay(ids[0], 0.005); // below prune_threshold 0.01
        // Advance virtual clock past the sunset window (120d ahead).
        let later = now + 121 * 86_400_000_i64;
        let (pathways, freq) = make_thriving_readers(&ids);
        let cfg = DecayConfig::default();
        let stats =
            run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(later)).expect("ok");
        // Exactly one of the two counters must fire — the prune (Step 3
        // runs before Step 4, and Step 4's guard skips already-pruned).
        assert_eq!(stats.workflows_pruned, 1);
        assert_eq!(
            stats.workflows_auto_sunset, 0,
            "double-count regression: workflow appears in BOTH pruned + auto_sunset"
        );
    }

    #[test]
    fn m11_consolidation_pre_fetch_short_circuits_on_pathway_read_failure() {
        // rationale: anti-property (transactional invariant) — a substrate
        // read failure on workflow #N must NOT leave workflows #1..N-1 in
        // a half-decayed state. We seed N=2; the SECOND workflow's pathway
        // is missing from the reader; cycle must return PathwayReadFailed
        // and the first workflow's weight must remain untouched (1.0).
        let now = 1_700_000_000_000_i64;
        let (mut bank, ids) = build_real_bank(2, now);
        // Only seed id_0's pathway weight; id_1 missing.
        let mut weights = HashMap::new();
        weights.insert(ids[0].to_string(), 0.5);
        let pathways = StubPathways { weights };
        let counts: HashMap<String, u64> =
            ids.iter().map(|id| (id.to_string(), 1)).collect();
        let freq = StubFreq {
            counts,
            cohort_max: 1,
        };
        let cfg = DecayConfig::default();
        let err = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now))
            .unwrap_err();
        assert!(matches!(err, DecayError::PathwayReadFailed { .. }));
        // CRITICAL: both workflows must STILL be at 1.0 — no decay
        // applied because pre-fetch short-circuited on the second read.
        for id in &ids {
            let w = bank.get(*id).expect("row").weight;
            assert!(
                (w - 1.0).abs() < 1e-12,
                "workflow {id} was decayed despite pre-fetch failure — partial-state regression: got {w}"
            );
        }
    }

    #[test]
    fn m11_consolidation_clockunavailable_skips_entire_cycle() {
        // rationale: contract regression — `now_ms_fn` returning None must
        // surface as `DecayError::ClockUnavailable` AND leave the bank
        // entirely unmutated (no partial-state writes mid-cycle).
        let now = 1_700_000_000_000_i64;
        let (mut bank, ids) = build_real_bank(3, now);
        let weights_before: Vec<f64> = ids.iter().map(|id| bank.get(*id).expect("r").weight).collect();
        let (pathways, freq) = make_thriving_readers(&ids);
        let cfg = DecayConfig::default();
        let err = run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || None).unwrap_err();
        assert!(matches!(err, DecayError::ClockUnavailable));
        let weights_after: Vec<f64> = ids.iter().map(|id| bank.get(*id).expect("r").weight).collect();
        assert_eq!(
            weights_before, weights_after,
            "ClockUnavailable must leave bank unmutated"
        );
    }

    #[test]
    fn m11_consolidation_recovery_edge_via_weight_rise_phase_for() {
        // rationale: anti-property (auto-recovery) — m30 derives phase via
        // `phase_for`, not via stored state. PrunePending → Active is
        // automatic on weight rise; m11's bridge does NOT need to emit a
        // transition. Verify by driving weight into PrunePending band,
        // observing phase_for, then driving it back up.
        let now = 1_700_000_000_000_i64;
        let (bank, ids) = build_real_bank(1, now);
        // Drive weight into m30's PrunePending band ([0.05, 0.10)).
        bank.apply_decay(ids[0], 0.07);
        let row_low = bank.get(ids[0]).expect("row");
        assert_eq!(
            row_low.phase_for(now + 1, DEFAULT_PRUNE_PENDING_THRESHOLD, DEFAULT_PRUNE_THRESHOLD),
            SunsetPhase::PrunePending
        );
        // Substrate rises (m42 reinforce; here a multiplicative >1 factor
        // clamped to 1.0 by the bank's apply_decay implementation).
        bank.apply_decay(ids[0], 100.0); // 0.07 * 100 = 7.0 → clamp to 1.0
        let row_high = bank.get(ids[0]).expect("row");
        assert!((row_high.weight - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            row_high.phase_for(now + 1, DEFAULT_PRUNE_PENDING_THRESHOLD, DEFAULT_PRUNE_THRESHOLD),
            SunsetPhase::Active,
            "recovery edge: phase_for must reclassify to Active automatically"
        );
    }

    #[test]
    fn m11_consolidation_compositional_integrity_high_freq_low_fitness() {
        // rationale: CC-2 trust invariant — frequency alone never grants
        // immortality. A workflow with frequency=1.0 and fitness=0.0 must
        // decay at exactly the base_rate (= 1 - plain_decay_rate = 0.98).
        // Multiplicative composition is the structural guarantor of this;
        // the test fires a canary if a future refactor breaks it.
        let now = 1_700_000_000_000_i64;
        let (mut bank, ids) = build_real_bank(1, now);
        // High frequency (max cohort), zero fitness (pathway weight 0.0),
        // recency = 1.0 (last_run == accepted_at == now).
        let mut weights = HashMap::new();
        weights.insert(ids[0].to_string(), 0.0);
        let pathways = StubPathways { weights };
        let mut counts = HashMap::new();
        counts.insert(ids[0].to_string(), 100);
        let freq = StubFreq {
            counts,
            cohort_max: 100,
        };
        let cfg = DecayConfig::default();
        run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now)).expect("ok");
        let w = bank.get(ids[0]).expect("row").weight;
        // 1.0 * base_rate (0.98) — multiplicative collapse to base.
        assert!(
            (w - 0.98).abs() < 1e-12,
            "high freq + zero fitness must decay at base_rate 0.98; got {w}"
        );
    }

    #[test]
    fn m11_decay_factor_clamp_safety() {
        // rationale: adversarial input — non-finite / out-of-range values
        // MUST be rejected at the typed `DecayFactor::new` boundary so
        // they never reach the bank's weight field. `compute_decay_factor`
        // upstream uses `debug_assert!` for [0,1] inputs (release builds
        // rely on the clamp at composition time + this typed boundary at
        // the constructor). The bank's `try_apply_decay` additionally
        // rejects non-finite multipliers; together they enforce the
        // anti-property "NaN never reaches the weight field".
        use workflow_core::m11_fitness_weighted_decay::DecayFactor;
        use workflow_core::m11_fitness_weighted_decay::DecayError;
        // Non-finite values rejected at the typed boundary.
        assert!(matches!(DecayFactor::new(f64::NAN), Err(DecayError::OutOfRange { .. })));
        assert!(matches!(DecayFactor::new(f64::INFINITY), Err(DecayError::OutOfRange { .. })));
        assert!(matches!(
            DecayFactor::new(f64::NEG_INFINITY),
            Err(DecayError::OutOfRange { .. })
        ));
        // Out-of-range values rejected.
        assert!(matches!(DecayFactor::new(-0.5), Err(DecayError::OutOfRange { .. })));
        assert!(matches!(DecayFactor::new(1.5), Err(DecayError::OutOfRange { .. })));
        // Boundary values [0.0, 1.0] accepted exactly.
        assert!((DecayFactor::new(0.0).expect("0").as_f64() - 0.0).abs() < f64::EPSILON);
        assert!((DecayFactor::new(1.0).expect("1").as_f64() - 1.0).abs() < f64::EPSILON);
        // Bank-side defense-in-depth: non-finite multiplier rejected by
        // CuratedBank::try_apply_decay (NaN must never reach `weight`).
        let now = 1_700_000_000_000_i64;
        let (bank, ids) = build_real_bank(1, now);
        assert!(bank.try_apply_decay(ids[0], f64::NAN).is_err());
        assert!(bank.try_apply_decay(ids[0], f64::INFINITY).is_err());
        let w = bank.get(ids[0]).expect("row").weight;
        assert!(w.is_finite() && (w - 1.0).abs() < f64::EPSILON, "weight stayed pristine");
    }

    #[test]
    fn m11_stats_serde_round_trip_after_full_cycle() {
        // rationale: cross-target invariant — locks T1 (serde derive) and
        // T2 (real-bank integration) together. Run a real cycle, capture
        // the SunsetStats, serialise to JSON, deserialise, assert
        // structural equality. If T1's json_safe_float adapter ever
        // regresses (e.g. INFINITY → null silent loss), the equality
        // assertion fires the canary on any future stats producer that
        // happens to ship Default sentinels.
        let now = 1_700_000_000_000_i64;
        let (mut bank, ids) = build_real_bank(3, now);
        let (pathways, freq) = make_thriving_readers(&ids);
        let cfg = DecayConfig::default();
        let stats =
            run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now)).expect("ok");
        // Round-trip via serde_json.
        let serialized = serde_json::to_string(&stats).expect("serialize");
        let round_trip: workflow_core::m11_fitness_weighted_decay::SunsetStats =
            serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(round_trip, stats);
        assert_eq!(round_trip.workflows_decayed, 3);
        assert_eq!(round_trip.cycles_run, 1);
    }
}
