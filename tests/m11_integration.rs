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
