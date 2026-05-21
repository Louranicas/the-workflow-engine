//! Integration tests for m30 — `CuratedBank` cross-module with the real
//! m11 `LifecycleBank` consolidation cycle (NO `MockBank`).
//!
//! Wave-B1 / S1002600 carry-forward H8 closure (consolidation cycle runs
//! against the production bank for the first time, scout C1).

#![allow(clippy::doc_markdown)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use workflow_core::m11_fitness_weighted_decay::{
    run_consolidation_cycle, DecayConfig, DecayError, DecayFactor, FrequencyReader, LifecycleBank,
    PathwayWeightReader, SunsetPhase,
};
use workflow_core::m14_lift::LiftSnapshot;
use workflow_core::m20_prefixspan::{Pattern, StepToken};
use workflow_core::m21_variant_builder::build_variants;
use workflow_core::m23_proposer::build_proposal;
use workflow_core::m30_bank::{
    BankError, CuratedBank, DEFAULT_PRUNE_PENDING_THRESHOLD, DEFAULT_PRUNE_THRESHOLD,
    DEFAULT_SUNSET_DAYS, MS_PER_DAY,
};

// ─── fixtures + readers ─────────────────────────────────────────────────────

fn snap() -> LiftSnapshot {
    LiftSnapshot {
        lift: Some(0.5),
        ci_half: Some(0.05),
        n: 30,
        latest_ts_ms: 0,
        computed_at: SystemTime::now(),
    }
}

fn proposal_with_seed(seed: u32) -> workflow_core::m23_proposer::WorkflowProposal {
    let p = Pattern::new(vec![StepToken(seed), StepToken(seed + 1)], 30, (0, seed as usize));
    let v = build_variants(&p).expect("v")[0].clone();
    build_proposal(v, &snap(), None).expect("ok")
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
                reason: "stub".into(),
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

fn seed_readers(bank: &CuratedBank) -> (StubPathways, StubFreq) {
    // pathway_id = workflow_id (Day-1 convention per m30 bridge docs).
    let mut weights = HashMap::new();
    let actives = bank.active(0, 0.0);
    for w in &actives {
        weights.insert(w.workflow_id.to_string(), 0.5);
    }
    let pw = StubPathways { weights };
    let freq = StubFreq {
        counts: HashMap::new(),
        cohort_max: 1,
    };
    (pw, freq)
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[test]
fn m30_accept_then_consolidation_cycle_decays_weight() {
    // rationale: Cross-module — running m11 cycle against CuratedBank
    // applies multiplicative decay to every active row.
    let mut bank = CuratedBank::new();
    let id_a = bank.accept(proposal_with_seed(11), 0).expect("a");
    let id_b = bank.accept(proposal_with_seed(12), 0).expect("b");
    let id_c = bank.accept(proposal_with_seed(13), 0).expect("c");

    let (pw, freq) = seed_readers(&bank);
    let cfg = DecayConfig::default();
    let now = 1_700_000_000_000_i64;

    let stats =
        run_consolidation_cycle(&mut bank, &pw, &freq, &cfg, || Some(now)).expect("cycle");
    assert_eq!(stats.cycles_run, 1);
    assert!(stats.workflows_decayed >= 3);

    for id in [id_a, id_b, id_c] {
        let w = bank.get(id).expect("g").weight;
        assert!(w < 1.0, "weight must decay below 1.0; got {w}");
        assert!(w > 0.0, "weight must remain positive; got {w}");
    }
}

#[test]
fn m30_consolidation_cycle_skips_clock_skew_workflows() {
    // rationale: Adversarial input — future-dated last_run_ms triggers the
    // F-POVM-07 silent-zero-timestamp guard and increments the dedicated
    // counter rather than corrupting the weight via negative-elapsed recency.
    let mut bank = CuratedBank::new();
    let id = bank.accept(proposal_with_seed(21), 0).expect("accept");
    // Force last_run_ms into the future via record_run(now=BIG).
    let future = i64::from(i32::MAX) * 1000;
    bank.record_run(id, future);
    assert_eq!(bank.get(id).expect("g").last_run_ms, Some(future));

    let (pw, freq) = seed_readers(&bank);
    let cfg = DecayConfig::default();
    let now = 1_700_000_000_000_i64; // earlier than future

    let stats = run_consolidation_cycle(&mut bank, &pw, &freq, &cfg, || Some(now)).expect("cycle");
    assert_eq!(stats.workflows_clock_skew_skipped, 1);
    // Weight UNCHANGED because the row was skipped pre-decay.
    assert!(
        (bank.get(id).expect("g").weight - 1.0).abs() < f64::EPSILON,
        "clock-skewed row must not have decayed"
    );
}

#[test]
fn m30_consolidation_cycle_emits_prune_pending_when_weight_drops_into_soft_band() {
    // rationale: Cross-module — m11 cycle drives Step 2.5 PrunePending
    // transition; m30's phase_for() classification matches.
    let mut bank = CuratedBank::new();
    let id = bank.accept(proposal_with_seed(31), 0).expect("accept");
    // Pre-decay weight into the soft band manually so the cycle's Step 2.5
    // condition (`weight < sunset_threshold && weight >= prune_threshold`)
    // fires regardless of formula sensitivity. Post-W2-F4 the m11
    // thresholds single-source from m30: prune_pending (soft) = 0.10,
    // prune (hard) = 0.05, so the PrunePending band is [0.05, 0.10).
    bank.apply_decay(id, 0.07); // weight 1.0 -> 0.07, inside [0.05, 0.10)

    let (pw, freq) = seed_readers(&bank);
    let cfg = DecayConfig::default();
    let now = 1_700_000_000_000_i64;

    let stats = run_consolidation_cycle(&mut bank, &pw, &freq, &cfg, || Some(now)).expect("cycle");
    assert_eq!(
        stats.workflows_prune_pending, 1,
        "expected 1 PrunePending emit; stats={stats:?}"
    );
}

#[test]
fn m30_prune_expired_called_after_cycle_evicts_below_hard_floor() {
    // rationale: Cross-module — manual prune sweep evicts rows whose
    // bank-side phase_for() classifies as SunsetExpired
    // (weight < DEFAULT_PRUNE_THRESHOLD).
    let bank = CuratedBank::new();
    let id_keep = bank.accept(proposal_with_seed(41), 0).expect("keep");
    let id_evict = bank.accept(proposal_with_seed(42), 0).expect("evict");
    bank.apply_decay(id_evict, 0.04); // < 0.05 hard floor
    let pre_len = bank.len();
    let evicted = bank.prune_expired(1);
    assert_eq!(evicted, 1);
    assert_eq!(bank.len(), pre_len - 1);
    assert!(bank.get(id_keep).is_ok());
    assert!(matches!(bank.get(id_evict), Err(BankError::NotFound(_))));
}

#[test]
fn m30_recovery_edge_via_phase_for_after_weight_rise() {
    // rationale: Anti-property — PrunePending → Active recovery edge is
    // AUTOMATIC via phase_for(); no explicit transition emit needed.
    let bank = CuratedBank::new();
    let id = bank.accept(proposal_with_seed(51), 0).expect("accept");
    // Drop weight into PrunePending band.
    bank.apply_decay(id, 0.08);
    let w_low = bank.get(id).expect("low");
    assert_eq!(
        w_low.phase_for(1, DEFAULT_PRUNE_PENDING_THRESHOLD, DEFAULT_PRUNE_THRESHOLD),
        SunsetPhase::PrunePending
    );
    // Multiplicative boost — fitness recovery substrate event.
    bank.apply_decay(id, 50.0); // 0.08 * 50 = 4.0 → clamped to 1.0
    let w_high = bank.get(id).expect("high");
    assert!((w_high.weight - 1.0).abs() < f64::EPSILON);
    assert_eq!(
        w_high.phase_for(1, DEFAULT_PRUNE_PENDING_THRESHOLD, DEFAULT_PRUNE_THRESHOLD),
        SunsetPhase::Active,
        "phase_for must auto-reclassify to Active on weight rise"
    );
}

#[test]
fn m30_accept_with_far_future_now_saturates_sunset() {
    // rationale: Arithmetic / overflow — sunset_at_ms must saturate not
    // wrap when accepted_at_ms is far-future.
    let bank = CuratedBank::new();
    let id = bank.accept(proposal_with_seed(61), i64::MAX - 1).expect("ok");
    let w = bank.get(id).expect("get");
    assert_eq!(w.sunset_at_ms, i64::MAX, "saturating_add did not saturate");
}

#[test]
fn m30_active_excludes_prune_pending_workflows_for_m31_input() {
    // rationale: Cross-module — m31's input slice is `bank.active(now,
    // soft_threshold)`; rows in PrunePending must be filtered out so the
    // selector cannot rank them.
    let bank = CuratedBank::new();
    let id_active = bank.accept(proposal_with_seed(71), 0).expect("a");
    let id_pending = bank.accept(proposal_with_seed(72), 0).expect("b");
    bank.apply_decay(id_pending, 0.08); // PrunePending band

    let actives = bank.active(1, DEFAULT_PRUNE_PENDING_THRESHOLD);
    let ids: Vec<u64> = actives.iter().map(|w| w.workflow_id).collect();
    assert!(ids.contains(&id_active));
    assert!(!ids.contains(&id_pending));
}

#[test]
fn m30_concurrent_apply_decay_under_arc_is_serialized() {
    // rationale: Concurrency — 4 threads concurrently call apply_decay on
    // distinct workflow_ids via Arc<CuratedBank>; interior Mutex guarantees
    // every decay lands and final weights are deterministic.
    let bank = Arc::new(CuratedBank::new());
    let mut ids = Vec::new();
    for s in 81..85_u32 {
        ids.push(bank.accept(proposal_with_seed(s), 0).expect("accept"));
    }
    let mut handles = Vec::new();
    for id in &ids {
        let b = Arc::clone(&bank);
        let id = *id;
        handles.push(std::thread::spawn(move || {
            // Apply 0.5 then 0.5 → final factor 0.25.
            b.apply_decay(id, 0.5);
            b.apply_decay(id, 0.5);
        }));
    }
    for h in handles {
        h.join().expect("join");
    }
    for id in ids {
        let w = bank.get(id).expect("g").weight;
        assert!(
            (w - 0.25).abs() < 1e-12,
            "expected 0.25 after two 0.5 decays, got {w}"
        );
    }
}

#[test]
fn m30_lifecyclebank_apply_decay_via_trait_round_trip() {
    // rationale: Cross-module contract — &mut-self trait method delegates
    // to interior-mutable &self impl via the inner Mutex; decay applies.
    let mut bank = CuratedBank::new();
    let id = bank.accept(proposal_with_seed(91), 0).expect("ok");
    let key = id.to_string();
    let factor = DecayFactor::new(0.5).expect("factor");
    LifecycleBank::apply_decay(&mut bank, &key, factor);
    let w = LifecycleBank::weight_of(&bank, &key).expect("weight");
    assert!((w - 0.5).abs() < 1e-12);
}

#[test]
fn m30_sunset_window_invariant_through_trait_surface() {
    // rationale: Contract regression — sunset_at_of via trait matches the
    // direct accessor AND the canonical formula
    // (accepted_at_ms + DEFAULT_SUNSET_DAYS × MS_PER_DAY).
    let bank = CuratedBank::new();
    let now = 1_700_000_000_000_i64;
    let id = bank.accept(proposal_with_seed(95), now).expect("ok");
    let key = id.to_string();
    let via_trait = LifecycleBank::sunset_at_of(&bank, &key).expect("trait");
    let via_direct = bank.get(id).expect("direct").sunset_at_ms;
    assert_eq!(via_trait, via_direct);
    assert_eq!(via_trait, now.saturating_add(DEFAULT_SUNSET_DAYS * MS_PER_DAY));
}
