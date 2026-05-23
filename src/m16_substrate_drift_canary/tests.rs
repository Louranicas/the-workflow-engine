//! m16 substrate-drift canary tests (Plan v2 v0.2.0 §3 Phase 9; spec
//! per `ai_specs/cross-cutting/substrate-drift.md`).

use super::{
    pair_skew_ms, AlertBudget, ClockSample, ClockSampler, ClockSource, DetectionResult,
    DriftDetector, Heartbeat, SkewEnvelope,
};
use crate::refusal_token::{RefusalToken, SubstrateId};

// ============================================================================
// Static test sampler — emits a fixed ClockSample for deterministic tests.
// ============================================================================

struct StaticSampler {
    source: ClockSource,
    clock_value_ms: u64,
    observed_at_ms: u64,
}

impl ClockSampler for StaticSampler {
    fn sample(&self) -> ClockSample {
        ClockSample {
            source: self.source,
            clock_value_ms: self.clock_value_ms,
            observed_at_ms: self.observed_at_ms,
        }
    }
    fn source(&self) -> ClockSource {
        self.source
    }
}

fn sampler(source: ClockSource, clock_value_ms: u64) -> Box<dyn ClockSampler> {
    Box::new(StaticSampler {
        source,
        clock_value_ms,
        observed_at_ms: 0,
    })
}

// ============================================================================
// ClockSource enumeration
// ============================================================================

#[test]
fn clock_source_all_enumerates_5_clocks_per_cc5() {
    assert_eq!(ClockSource::ALL.len(), 5);
    let set: std::collections::HashSet<ClockSource> = ClockSource::ALL.iter().copied().collect();
    assert_eq!(set.len(), 5, "all 5 ClockSource variants must be distinct");
    for expected in [
        ClockSource::M11Recency,
        ClockSource::M13StcortexDecay,
        ClockSource::InjectionTtl,
        ClockSource::AtuinCheckpoint,
        ClockSource::StcortexPathwayDecay,
    ] {
        assert!(set.contains(&expected), "{expected:?} must be in ALL");
    }
}

#[test]
fn clock_source_all_serde_distinct() {
    let strings: Vec<String> = ClockSource::ALL
        .iter()
        .map(|c| serde_json::to_string(c).expect("ser"))
        .collect();
    let unique: std::collections::HashSet<&String> = strings.iter().collect();
    assert_eq!(unique.len(), 5);
}

// ============================================================================
// pair_skew_ms — absolute-difference math
// ============================================================================

#[test]
fn pair_skew_ms_symmetric_and_zero_when_equal() {
    assert_eq!(pair_skew_ms(0, 0), 0);
    assert_eq!(pair_skew_ms(1_000_000, 1_000_000), 0);
    assert_eq!(pair_skew_ms(100, 200), pair_skew_ms(200, 100));
    assert_eq!(pair_skew_ms(100, 200), 100);
}

// ============================================================================
// SkewEnvelope default
// ============================================================================

#[test]
fn skew_envelope_default_is_5000ms() {
    assert_eq!(SkewEnvelope::default().max_skew_ms, 5_000);
}

// ============================================================================
// AlertBudget — C-9 rate-limit dedup
// ============================================================================

#[test]
fn alert_budget_fires_first_alert_then_suppresses_within_cooldown() {
    let mut b = AlertBudget::new(60_000);
    // First fire at t=0
    assert!(b.should_fire(ClockSource::M11Recency, ClockSource::AtuinCheckpoint, 0));
    // Same pair within cooldown — suppressed.
    assert!(!b.should_fire(ClockSource::M11Recency, ClockSource::AtuinCheckpoint, 1));
    assert!(!b.should_fire(ClockSource::M11Recency, ClockSource::AtuinCheckpoint, 30_000));
    assert!(!b.should_fire(
        ClockSource::M11Recency,
        ClockSource::AtuinCheckpoint,
        59_999
    ));
    // After cooldown — fires again.
    assert!(b.should_fire(ClockSource::M11Recency, ClockSource::AtuinCheckpoint, 60_000));
}

#[test]
fn alert_budget_pair_normalised_so_symmetric_calls_dedup() {
    let mut b = AlertBudget::new(60_000);
    assert!(b.should_fire(ClockSource::M11Recency, ClockSource::AtuinCheckpoint, 0));
    // Reverse-order: same pair, must be deduplicated.
    assert!(!b.should_fire(ClockSource::AtuinCheckpoint, ClockSource::M11Recency, 1));
}

#[test]
fn alert_budget_distinct_pairs_each_fire_independently() {
    let mut b = AlertBudget::new(60_000);
    assert!(b.should_fire(ClockSource::M11Recency, ClockSource::AtuinCheckpoint, 0));
    // Different pair — independent.
    assert!(b.should_fire(
        ClockSource::M13StcortexDecay,
        ClockSource::StcortexPathwayDecay,
        0
    ));
    assert!(b.should_fire(ClockSource::InjectionTtl, ClockSource::M11Recency, 0));
}

// ============================================================================
// DriftDetector — end-to-end detect cycle
// ============================================================================

#[test]
fn detector_no_drift_when_all_clocks_within_envelope() {
    let samplers: Vec<Box<dyn ClockSampler>> = vec![
        sampler(ClockSource::M11Recency, 1_000_000),
        sampler(ClockSource::M13StcortexDecay, 1_000_500),
        sampler(ClockSource::InjectionTtl, 999_800),
        sampler(ClockSource::AtuinCheckpoint, 1_000_100),
        sampler(ClockSource::StcortexPathwayDecay, 999_500),
    ];
    let mut d = DriftDetector::new(
        samplers,
        SkewEnvelope { max_skew_ms: 5_000 },
        AlertBudget::new(60_000),
    );
    let DetectionResult {
        heartbeat,
        samples,
        events,
    } = d.detect(2_000_000);
    assert_eq!(heartbeat.cycle, 1);
    assert_eq!(heartbeat.emitted_at_ms, 2_000_000);
    assert_eq!(samples.len(), 5);
    assert!(
        events.is_empty(),
        "all clocks within 5_000ms envelope → no drift events"
    );
}

#[test]
fn detector_emits_substrate_authored_cc5_token_on_skew_violation() {
    let samplers: Vec<Box<dyn ClockSampler>> = vec![
        sampler(ClockSource::M11Recency, 1_000_000),
        // M13 60s ahead → 60_000ms skew, exceeds 5_000ms envelope.
        sampler(ClockSource::M13StcortexDecay, 1_060_000),
    ];
    let mut d = DriftDetector::new(
        samplers,
        SkewEnvelope { max_skew_ms: 5_000 },
        AlertBudget::new(60_000),
    );
    let result = d.detect(2_000_000);
    assert_eq!(result.events.len(), 1, "1 pair-wise skew detected");
    match &result.events[0] {
        RefusalToken::SubstrateAuthored {
            substrate_id,
            substrate_reason,
            ..
        } => {
            assert_eq!(*substrate_id, SubstrateId::Cc5LoopClocks);
            assert!(substrate_reason.contains("cc5_clock_skew"));
            assert!(substrate_reason.contains("M11Recency"));
            assert!(substrate_reason.contains("M13StcortexDecay"));
            assert!(substrate_reason.contains("skew_ms=60000"));
            assert!(substrate_reason.contains("envelope_ms=5000"));
        }
        other => panic!("expected SubstrateAuthored Cc5LoopClocks; got {other:?}"),
    }
    // The token is substrate-authored speech per NA-5 / Plan v2 §9.1.
    assert!(result.events[0].is_substrate_authored());
    assert_eq!(
        result.events[0].substrate_id(),
        Some(SubstrateId::Cc5LoopClocks)
    );
}

#[test]
fn detector_alert_budget_suppresses_repeat_drift_within_cooldown() {
    let samplers: Vec<Box<dyn ClockSampler>> = vec![
        sampler(ClockSource::M11Recency, 1_000_000),
        sampler(ClockSource::M13StcortexDecay, 1_060_000),
    ];
    let mut d = DriftDetector::new(
        samplers,
        SkewEnvelope { max_skew_ms: 5_000 },
        AlertBudget::new(60_000),
    );
    let r1 = d.detect(2_000_000);
    assert_eq!(r1.events.len(), 1, "first detect emits the drift event");
    // Same drift, 30s later (within 60s cooldown) — suppressed.
    let r2 = d.detect(2_030_000);
    assert!(
        r2.events.is_empty(),
        "second detect within cooldown suppressed (C-9 alert-fatigue mitigation)"
    );
    // 60s+ later — fires again.
    let r3 = d.detect(2_060_001);
    assert_eq!(r3.events.len(), 1, "post-cooldown drift fires again");
}

#[test]
fn detector_cycle_monotonically_increments_per_detect() {
    let samplers: Vec<Box<dyn ClockSampler>> = vec![
        sampler(ClockSource::M11Recency, 0),
        sampler(ClockSource::M13StcortexDecay, 0),
    ];
    let mut d = DriftDetector::new(
        samplers,
        SkewEnvelope::default(),
        AlertBudget::default(),
    );
    assert_eq!(d.cycle(), 0);
    let _ = d.detect(0);
    assert_eq!(d.cycle(), 1);
    let _ = d.detect(1);
    assert_eq!(d.cycle(), 2);
    let _ = d.detect(2);
    assert_eq!(d.cycle(), 3);
}

#[test]
fn detector_heartbeat_records_observation_timestamp_per_cycle() {
    let samplers: Vec<Box<dyn ClockSampler>> = vec![
        sampler(ClockSource::M11Recency, 0),
        sampler(ClockSource::AtuinCheckpoint, 0),
    ];
    let mut d = DriftDetector::new(
        samplers,
        SkewEnvelope::default(),
        AlertBudget::default(),
    );
    let r = d.detect(1_700_000_000_000);
    let hb: Heartbeat = r.heartbeat;
    assert_eq!(hb.cycle, 1);
    assert_eq!(hb.emitted_at_ms, 1_700_000_000_000);
}

#[test]
fn detector_multi_pair_drift_emits_one_token_per_pair() {
    // Three clocks all 60s apart in pairwise terms (M11=0, M13=60_000,
    // AtuinCheckpoint=120_000) → 3 pairs all exceed envelope.
    let samplers: Vec<Box<dyn ClockSampler>> = vec![
        sampler(ClockSource::M11Recency, 0),
        sampler(ClockSource::M13StcortexDecay, 60_000),
        sampler(ClockSource::AtuinCheckpoint, 120_000),
    ];
    let mut d = DriftDetector::new(
        samplers,
        SkewEnvelope { max_skew_ms: 5_000 },
        AlertBudget::new(60_000),
    );
    let r = d.detect(2_000_000);
    assert_eq!(
        r.events.len(),
        3,
        "3 pair-wise skews (M11-M13, M11-Atuin, M13-Atuin) → 3 events"
    );
    for event in &r.events {
        assert_eq!(event.substrate_id(), Some(SubstrateId::Cc5LoopClocks));
    }
}
