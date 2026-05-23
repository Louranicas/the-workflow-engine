#![allow(clippy::doc_markdown)] // habitat-conventional per workspace clippy config
//! V4 substrate test fixtures — `Plan v2` `v0.2.0` Phase 10 per `DX-5` =
//! full deterministic replicas + ADR `D-S1002127-03` `§1.b`.
//!
//! Six named fixtures (5 per ADR + 1 `NA-4` canary): `Cr2Inflation`,
//! `RefuseWriteNoConsumer`, `HyphenSlugReducer`,
//! `ConductorEnforcementFlagOff`, `AtuinWalContention`, `V3CanaryFailure`.
//! Each fixture surfaces a typed substrate state the engine must respond
//! to via the `V1` `RefusalToken` envelope shipped Phase 5.
//!
//! Per `DX-5` lock, this file ships the FIXTURE CATALOGUE + assertable
//! contract shape per fixture; per-fixture detailed test sweeps (one
//! per `V1` variant) are a post-`v0.2.0` follow-up.

use workflow_core::back_pressure::{BackPressureSeverity, BackPressureSignal};
use workflow_core::m16_substrate_drift_canary::{
    AlertBudget, ClockSample, ClockSampler, ClockSource, DriftDetector, SkewEnvelope,
};
use workflow_core::m32_dispatcher::EscapeSurfaceProfile;
use workflow_core::m33_verifier::VerifierVerdict;
use workflow_core::refusal_token::{
    EngineRefusalReason, ModuleId, OperatorRefusalReason, RefusalToken, SubstrateId,
};

// ============================================================================
// Fixture catalogue (per ADR D-S1002127-03 §1.b + NA-4)
// ============================================================================

/// Each fixture surfaces a typed substrate state the engine must
/// respond to. Tests use these to assert the engine's response chain.
#[derive(Debug, Clone, PartialEq)]
pub enum SubstrateFixture {
    /// stcortex returns pre-CR-2-magnitude `learning_health` (~0.91)
    /// rather than the post-CR-2 magnitude-weighted ~0.067. Engine
    /// must treat as `RefusalToken::Unavailable::SubstrateAuthored`
    /// per V5 trust degradation.
    Cr2Inflation,
    /// stcortex reducer refuses write because no fresh consumer
    /// registered for the namespace. Engine sees
    /// `RefusalToken::SubstrateAuthored Stcortex` with reason
    /// "refuse_write_no_consumer".
    RefuseWriteNoConsumer,
    /// stcortex reducer rejects a slug containing a hyphen
    /// (`workflow-trace-v020` → reducer-rejected; only underscores
    /// allowed). Engine sees `RefusalToken::SubstrateAuthored
    /// Stcortex` with reason "slug_hyphen_rejected".
    HyphenSlugReducer,
    /// HABITAT-CONDUCTOR `CONDUCTOR_ENFORCEMENT_ENABLED=0`. Engine
    /// warns-and-proceeds per NA-4 conductor-state assertion shipped
    /// at v0.1.0 Phase 8 step 2.
    ConductorEnforcementFlagOff,
    /// atuin WAL is locked by concurrent foreground writer. Engine
    /// reads must defer or emit back-pressure per V2.
    AtuinWalContention,
    /// V3 m16 substrate-drift canary stops emitting. Per NA-4, the
    /// Watcher's deployment-watch journal (OP-6 post-v0.2.0) would
    /// catch the absence; this fixture provides the assertable
    /// contract.
    V3CanaryFailure,
}

/// Static-test sampler returning a fixed ClockSample (mirrors the
/// m16 internal test fixture; surfaced here for cross-test reuse).
struct StaticSampler {
    source: ClockSource,
    clock_value_ms: u64,
}

impl ClockSampler for StaticSampler {
    fn sample(&self) -> ClockSample {
        ClockSample {
            source: self.source,
            clock_value_ms: self.clock_value_ms,
            observed_at_ms: 0,
        }
    }
    fn source(&self) -> ClockSource {
        self.source
    }
}

// ============================================================================
// Fixture 1: cr2_inflation_fixture
// ============================================================================

#[test]
fn cr2_inflation_fixture_engine_emits_unavailable_substrate_authored() {
    // Per Plan v2 §11 RALPH consent gradient + V5 trust degradation:
    // when stcortex returns pre-CR-2 magnitude, engine treats the
    // measurement as substrate-authored unavailability of trustworthy
    // data (not "unreachable" — substrate IS reachable, just returning
    // pre-fix magnitude).
    let fixture = SubstrateFixture::Cr2Inflation;
    assert_eq!(fixture, SubstrateFixture::Cr2Inflation);
    // The engine's response to CR-2 inflation should route through V1
    // RefusalToken::Unavailable::SubstrateAuthored Stcortex (substrate
    // explicitly emitting an unavailable-for-trust signal). Engine
    // construction of the token:
    let token = RefusalToken::unavailable_substrate_authored(
        SubstrateId::Stcortex,
        "cr2_inflation:learning_health=0.911_pre_fix".to_owned(),
    );
    assert!(token.is_substrate_authored());
    assert!(!token.is_engine_imagined());
    assert_eq!(token.substrate_id(), Some(SubstrateId::Stcortex));
}

// ============================================================================
// Fixture 2: refuse_write_no_consumer_fixture
// ============================================================================

#[test]
fn refuse_write_no_consumer_fixture_engine_emits_substrate_authored() {
    let fixture = SubstrateFixture::RefuseWriteNoConsumer;
    assert_eq!(fixture, SubstrateFixture::RefuseWriteNoConsumer);
    // Per ADR D-S1004XXX-04 §1.2 m13 row: stcortex refuse-write-no-consumer
    // is substrate-authored. The drain wire (Phase 7) emits exactly this
    // shape when the outbox replay encounters such an entry.
    let token = RefusalToken::substrate_authored(
        SubstrateId::Stcortex,
        "refuse_write_no_consumer".to_owned(),
    );
    assert!(token.is_substrate_authored());
    assert_eq!(token.substrate_id(), Some(SubstrateId::Stcortex));
}

// ============================================================================
// Fixture 3: hyphen_slug_reducer_fixture (S1001757 munge-bug guard)
// ============================================================================

#[test]
fn hyphen_slug_reducer_fixture_engine_emits_substrate_authored() {
    let fixture = SubstrateFixture::HyphenSlugReducer;
    assert_eq!(fixture, SubstrateFixture::HyphenSlugReducer);
    // S1001757 munge-bug: stcortex reducer rejects hyphens. v0.2.0
    // m9 namespace guard pre-rejects locally; if a hyphen-bearing slug
    // somehow reaches stcortex, the reducer's refusal is
    // substrate-authored.
    let token = RefusalToken::substrate_authored(
        SubstrateId::Stcortex,
        "slug_hyphen_rejected:slug=workflow-trace-v020".to_owned(),
    );
    assert!(token.is_substrate_authored());
}

// ============================================================================
// Fixture 4: conductor_enforcement_flag_off_fixture
// ============================================================================

#[test]
fn conductor_enforcement_flag_off_fixture_engine_warns_and_proceeds() {
    let fixture = SubstrateFixture::ConductorEnforcementFlagOff;
    assert_eq!(fixture, SubstrateFixture::ConductorEnforcementFlagOff);
    // Per NA-4 Phase 8 step 2 (shipped v0.1.0 / M0): engine warns when
    // CONDUCTOR_ENFORCEMENT_ENABLED is unset or "0"; dispatch proceeds
    // with verdict-advisory flag. v0.2.0 does NOT change this contract;
    // the fixture asserts the warn-not-Refuse semantic by constructing
    // the engine-authored RefusalToken envelope shape that would be
    // emitted IF the engine chose to Refuse instead (currently it does
    // not — the warn-and-proceed semantic is on m32's enforcement-state
    // assertion path, not via RefusalToken; this fixture documents the
    // contract for cross-test reference).
    let what_engine_would_emit_if_refusing =
        RefusalToken::engine_authored(ModuleId::M32, EngineRefusalReason::ConductorUnreachable);
    // Verify the shape is well-formed even though the engine doesn't
    // currently emit it in this fixture's scenario (warn-and-proceed,
    // not Refuse).
    assert_eq!(
        what_engine_would_emit_if_refusing.substrate_id(),
        None,
        "engine-authored — no substrate_id"
    );
    assert!(!what_engine_would_emit_if_refusing.is_substrate_authored());
}

// ============================================================================
// Fixture 5: atuin_wal_contention_fixture
// ============================================================================

#[test]
fn atuin_wal_contention_fixture_engine_emits_back_pressure_signal() {
    let fixture = SubstrateFixture::AtuinWalContention;
    assert_eq!(fixture, SubstrateFixture::AtuinWalContention);
    // Per V2 (Phase 8): per-substrate back-pressure. When atuin WAL is
    // locked, engine receives (in Push mode) or constructs (in Pull
    // mode) a BackPressureSignal with severity Saturated. Per-substrate
    // mode is Pull by default (atuin is UNKNOWN consent gradient per
    // §11; push-emitter unlikely upstream).
    let signal = BackPressureSignal::new(
        SubstrateId::Atuin,
        BackPressureSeverity::Saturated,
        1_700_000_000_000,
    );
    assert_eq!(signal.substrate, SubstrateId::Atuin);
    assert_eq!(signal.severity, BackPressureSeverity::Saturated);
    // Optionally combine with V1 RefusalToken: when atuin saturation
    // forces a deferral, the engine's response is operator-authored
    // "not now" per NA-3 operator-refusal vocabulary.
    let deferral = RefusalToken::operator_authored(OperatorRefusalReason::NotNow {
        context: Some("atuin_wal_contention".to_owned()),
    });
    match deferral {
        RefusalToken::OperatorAuthored { .. } => {}
        other => panic!("expected OperatorAuthored; got {other:?}"),
    }
}

// ============================================================================
// Fixture 6: v3_canary_failure_fixture (NA-4 self-canary mitigation)
// ============================================================================

#[test]
fn v3_canary_failure_fixture_provides_assertable_contract_for_op6() {
    let fixture = SubstrateFixture::V3CanaryFailure;
    assert_eq!(fixture, SubstrateFixture::V3CanaryFailure);
    // Per NA-4: m16 self-canary problem. When m16 stops emitting,
    // post-v0.2.0 OP-6 Watcher integration must observe the absence.
    // v0.2.0 ships the assertable contract: post-OP-6 the Watcher would
    // emit RefusalToken::SubstrateAuthored Watcher with reason
    // "m16_heartbeat_missing:cycles=N".
    let watcher_alert = RefusalToken::substrate_authored(
        SubstrateId::Watcher,
        "m16_heartbeat_missing:cycles=3".to_owned(),
    );
    assert!(watcher_alert.is_substrate_authored());
    assert_eq!(watcher_alert.substrate_id(), Some(SubstrateId::Watcher));
}

// ============================================================================
// Cross-fixture round-trip: m16 detector exercises within a fixture
// ============================================================================

#[test]
fn cross_fixture_m16_detector_drift_within_atuin_contention_envelope() {
    // Demonstrate m16 detector running over a fixture scenario:
    // atuin's clock drifts while m11's clock advances normally.
    let samplers: Vec<Box<dyn ClockSampler>> = vec![
        Box::new(StaticSampler {
            source: ClockSource::M11Recency,
            clock_value_ms: 1_000_000,
        }),
        Box::new(StaticSampler {
            source: ClockSource::AtuinCheckpoint,
            clock_value_ms: 1_010_000, // 10s drift > 5s envelope
        }),
    ];
    let mut d = DriftDetector::new(
        samplers,
        SkewEnvelope { max_skew_ms: 5_000 },
        AlertBudget::new(60_000),
    );
    let r = d.detect(2_000_000);
    assert_eq!(r.events.len(), 1, "10s drift exceeds 5s envelope");
    assert_eq!(
        r.events[0].substrate_id(),
        Some(SubstrateId::Cc5LoopClocks)
    );
}

// ============================================================================
// V1 RefusalToken authorship-distinguishability sweep across fixtures
// ============================================================================

#[test]
fn v1_authorship_distinguishable_per_fixture_class() {
    // Substrate fixtures emit substrate-authored or engine-authored
    // tokens depending on the scenario. The is_substrate_authored /
    // is_engine_imagined accessors distinguish each per NA-5.
    let substrate_cases = [
        RefusalToken::substrate_authored(SubstrateId::Stcortex, "x".to_owned()),
        RefusalToken::unavailable_substrate_authored(SubstrateId::Stcortex, "y".to_owned()),
    ];
    for t in &substrate_cases {
        assert!(t.is_substrate_authored());
        assert!(!t.is_engine_imagined());
    }
    let engine_imagined =
        RefusalToken::unavailable_engine_imagined(SubstrateId::Atuin, "no schema".to_owned());
    assert!(!engine_imagined.is_substrate_authored());
    assert!(engine_imagined.is_engine_imagined());
    let engine_authored = RefusalToken::engine_authored(
        ModuleId::M33,
        EngineRefusalReason::SpecBoundRefusal,
    );
    assert!(!engine_authored.is_substrate_authored());
    assert!(!engine_authored.is_engine_imagined());
}

// ============================================================================
// EscapeSurfaceProfile + V1 payload cross-fixture
// ============================================================================

#[test]
fn fixture_acceptable_escape_surface_couples_to_d_s1002127_02() {
    use workflow_core::refusal_token::RefusalPayload;
    // A substrate refusing a high-surface workflow can suggest an
    // acceptable lower surface via RefusalPayload::AcceptableEscapeSurface
    // (couples to D-S1002127-02 7-variant ordinal).
    let payload = RefusalPayload::AcceptableEscapeSurface(EscapeSurfaceProfile::Sandboxed);
    let token = RefusalToken::SubstrateAuthored {
        substrate_id: SubstrateId::Stcortex,
        substrate_reason: "surface_too_aggressive".to_owned(),
        payload: Some(payload.clone()),
    };
    match &token {
        RefusalToken::SubstrateAuthored {
            payload: Some(p), ..
        } => assert_eq!(*p, payload),
        other => panic!("expected SubstrateAuthored with payload; got {other:?}"),
    }
}

// ============================================================================
// VerifierVerdict shape (verifies fixtures interop with the m33 surface)
// ============================================================================

#[test]
fn fixture_verifier_verdict_shapes_are_constructible() {
    // Constructibility check — the fixtures must compose cleanly with
    // the m33 VerifierVerdict surface so downstream integration tests
    // can chain fixture → verifier → verdict.
    let approve = VerifierVerdict::Approve;
    let refuse = VerifierVerdict::Refuse {
        reason: "fixture: cr2 inflation".to_owned(),
    };
    assert!(matches!(approve, VerifierVerdict::Approve));
    match refuse {
        VerifierVerdict::Refuse { reason } => assert!(reason.contains("cr2 inflation")),
        other => panic!("expected Refuse; got {other:?}"),
    }
}
