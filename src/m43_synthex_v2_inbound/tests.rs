//! m43 tests — library-only coverage per v0.2.2+ scaffold scope.

use super::{
    HandlerError, HandlerOutcome, InboundEvent, InboundEventKind, InboundHandler, InboundSource,
    InboundServerConfig, DEFAULT_INBOUND_PORT,
};
use crate::refusal_token::{RefusalToken, SubstrateId, UnavailableReason};

fn fixture_event(kind: InboundEventKind) -> InboundEvent {
    InboundEvent {
        source: InboundSource::M46Observer,
        kind,
        emitted_at_ms: 1_735_689_600_000,
        boot_id: "sx2-boot-fixture".to_owned(),
        data: serde_json::json!({"placeholder": true}),
    }
}

// ============================================================================
// Config + defaults
// ============================================================================

#[test]
fn server_config_default_port_is_8094_per_wiring_02b_spec() {
    let cfg = InboundServerConfig::default();
    assert_eq!(cfg.port, 8094);
    assert_eq!(cfg.port, DEFAULT_INBOUND_PORT);
    assert_eq!(cfg.max_body_bytes, 64 * 1024);
    assert!(!cfg.log_payloads);
}

#[test]
fn server_config_serde_round_trips() {
    let cfg = InboundServerConfig {
        port: 9999,
        max_body_bytes: 1024,
        log_payloads: true,
    };
    let s = serde_json::to_string(&cfg).expect("ser");
    let r: InboundServerConfig = serde_json::from_str(&s).expect("de");
    assert_eq!(cfg, r);
}

// ============================================================================
// Event taxonomy — 5 kinds + 4 sources serde-distinct
// ============================================================================

#[test]
fn all_5_inbound_event_kinds_serde_distinct() {
    let kinds = [
        InboundEventKind::WfeDriftObserved,
        InboundEventKind::WfeSilenceObserved,
        InboundEventKind::WfeDriftHypothesis,
        InboundEventKind::WfeProposalBlocked,
        InboundEventKind::WfeUnreachablePersisting,
    ];
    let strings: Vec<String> = kinds
        .iter()
        .map(|k| serde_json::to_string(k).expect("ser"))
        .collect();
    let unique: std::collections::HashSet<&String> = strings.iter().collect();
    assert_eq!(unique.len(), 5, "5 kinds must serde-distinctly");
}

#[test]
fn all_4_inbound_sources_serde_distinct() {
    let sources = [
        InboundSource::M46Observer,
        InboundSource::M47Critic,
        InboundSource::M51EmberProtector,
        InboundSource::M10HttpPoller,
    ];
    let strings: Vec<String> = sources
        .iter()
        .map(|s| serde_json::to_string(s).expect("ser"))
        .collect();
    let unique: std::collections::HashSet<&String> = strings.iter().collect();
    assert_eq!(unique.len(), 4, "4 sources must serde-distinctly");
}

#[test]
fn inbound_event_serde_preserves_all_fields() {
    let event = fixture_event(InboundEventKind::WfeDriftObserved);
    let s = serde_json::to_string(&event).expect("ser");
    let r: InboundEvent = serde_json::from_str(&s).expect("de");
    assert_eq!(event, r);
    // Wire-shape audit: NA-10' replay-safety fields present
    assert!(s.contains("source"));
    assert!(s.contains("kind"));
    assert!(s.contains("emitted_at_ms"));
    assert!(s.contains("boot_id"));
}

// ============================================================================
// Handler dispatch — per-kind outcomes
// ============================================================================

#[test]
fn handler_routes_wfe_drift_observed_to_m11_decay_hint() {
    let handler = InboundHandler::new();
    let outcome = handler
        .dispatch(&fixture_event(InboundEventKind::WfeDriftObserved))
        .expect("dispatch ok");
    assert_eq!(outcome, HandlerOutcome::M11DecayHintApplied);
}

#[test]
fn handler_routes_wfe_silence_observed_to_m16_budget_hint() {
    let handler = InboundHandler::new();
    let outcome = handler
        .dispatch(&fixture_event(InboundEventKind::WfeSilenceObserved))
        .expect("dispatch ok");
    assert_eq!(outcome, HandlerOutcome::M16BudgetHintApplied);
}

#[test]
fn handler_routes_wfe_drift_hypothesis_to_noop_advisory() {
    let handler = InboundHandler::new();
    let outcome = handler
        .dispatch(&fixture_event(InboundEventKind::WfeDriftHypothesis))
        .expect("dispatch ok");
    assert_eq!(outcome, HandlerOutcome::NoOpAdvisory);
}

#[test]
fn handler_routes_wfe_proposal_blocked_to_logged() {
    let handler = InboundHandler::new();
    let outcome = handler
        .dispatch(&fixture_event(InboundEventKind::WfeProposalBlocked))
        .expect("dispatch ok");
    assert_eq!(outcome, HandlerOutcome::Logged);
}

#[test]
fn handler_routes_wfe_unreachable_persisting_to_v5_bilateral() {
    let handler = InboundHandler::new();
    let outcome = handler
        .dispatch(&fixture_event(InboundEventKind::WfeUnreachablePersisting))
        .expect("dispatch ok");
    assert_eq!(outcome, HandlerOutcome::V5BilateralMirrorUpdated);
}

// ============================================================================
// Drain mode + NA-5 reciprocity refusal
// ============================================================================

#[test]
fn handler_in_drain_mode_returns_draining_error() {
    let mut handler = InboundHandler::new();
    assert!(!handler.is_draining());
    handler.drain();
    assert!(handler.is_draining());
    let result = handler.dispatch(&fixture_event(InboundEventKind::WfeDriftObserved));
    assert_eq!(result, Err(HandlerError::Draining));
}

#[test]
fn refusal_for_draining_emits_substrate_authored_wfe_draining() {
    let refusal = InboundHandler::refusal_for(&HandlerError::Draining);
    match refusal {
        RefusalToken::Unavailable(UnavailableReason::SubstrateAuthored {
            substrate_id,
            substrate_reason,
        }) => {
            assert_eq!(substrate_id, SubstrateId::SynthexV2);
            assert_eq!(substrate_reason, "wfe_draining");
        }
        other => panic!("expected SubstrateAuthored draining refusal; got {other:?}"),
    }
}

#[test]
fn refusal_for_payload_mismatch_emits_substrate_authored_with_detail() {
    let err = HandlerError::PayloadShapeMismatch {
        kind: InboundEventKind::WfeDriftObserved,
        detail: "missing severity field".to_owned(),
    };
    let refusal = InboundHandler::refusal_for(&err);
    match refusal {
        RefusalToken::Unavailable(UnavailableReason::SubstrateAuthored {
            substrate_reason,
            ..
        }) => {
            assert!(substrate_reason.starts_with("wfe_malformed_inbound:"));
            assert!(substrate_reason.contains("missing severity field"));
        }
        other => panic!("expected SubstrateAuthored payload-mismatch refusal; got {other:?}"),
    }
}

#[test]
fn refusal_for_policy_rejected_emits_substrate_authored_hypothesis_rejected() {
    let err = HandlerError::PolicyRejected {
        kind: InboundEventKind::WfeDriftHypothesis,
        reason: "contradicts m11 invariant".to_owned(),
    };
    let refusal = InboundHandler::refusal_for(&err);
    match refusal {
        RefusalToken::Unavailable(UnavailableReason::SubstrateAuthored {
            substrate_reason,
            ..
        }) => {
            assert!(substrate_reason.starts_with("wfe_hypothesis_rejected:"));
            assert!(substrate_reason.contains("contradicts m11 invariant"));
        }
        other => panic!("expected SubstrateAuthored policy-rejected refusal; got {other:?}"),
    }
}

// ============================================================================
// Default + Clone derives
// ============================================================================

#[test]
fn handler_default_constructs_non_draining() {
    let handler = InboundHandler::default();
    assert!(!handler.is_draining());
}
