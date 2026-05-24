//! W1 transport client tests — Plan v2 S1004590 v0.2.2+ Zen
//! PARTIAL APPROVE / AMEND.

use std::sync::Arc;

use super::{HeartbeatAck, HeartbeatTransport, HeartbeatWireEnvelope, SkewSummary};
use crate::m16_substrate_drift_canary::Heartbeat;
use crate::refusal_token::{RefusalToken, SubstrateId, UnavailableReason};
use crate::substrate_trust::{
    SubstrateParticipationStatus, SubstrateTrust, TrustEntry, TrustValue,
};

fn fixture_envelope() -> HeartbeatWireEnvelope {
    HeartbeatWireEnvelope {
        heartbeat: Heartbeat {
            emitted_at_ms: 1_735_689_600_000,
            cycle: 42,
        },
        heartbeat_source: "workflow-trace::m16_substrate_drift_canary".to_owned(),
        boot_id: "boot-test-uuid-fixture".to_owned(),
        instance_id: "wfe-test-instance".to_owned(),
        skew_summary: SkewSummary {
            max_observed_skew_ms: 12,
            samples_observed: 10,
            had_refusals: false,
        },
        alert_budget_remaining: 7,
    }
}

fn shipped_trust() -> Arc<SubstrateTrust> {
    let mut t = SubstrateTrust::new();
    let _prev = t.set(
        SubstrateId::SynthexV2,
        TrustEntry {
            status: SubstrateParticipationStatus::Live,
            value: TrustValue::Score(0.9),
        },
    );
    Arc::new(t)
}

// ============================================================================
// V5 gate (NA-5 audit-distinguishability primary check)
// ============================================================================

#[test]
fn v5_gate_not_shipped_short_circuits_to_engine_imagined_without_http_call() {
    // Default SubstrateTrust = every substrate NotShipped → engine-imagined.
    let transport = HeartbeatTransport::new(
        "http://127.0.0.1:1/v3/heartbeat".to_owned(), // port 1 = unreachable
        Arc::new(SubstrateTrust::new()),
    );
    let result = transport.send(&fixture_envelope());
    match result {
        Err(RefusalToken::Unavailable(UnavailableReason::EngineImagined {
            substrate_id,
            reason,
        })) => {
            assert_eq!(substrate_id, SubstrateId::SynthexV2);
            assert!(
                reason.contains("engine_imagined"),
                "expected engine_imagined: prefix per NA-5; got {reason}"
            );
        }
        other => panic!("expected EngineImagined short-circuit; got {other:?}"),
    }
}

// ============================================================================
// Transport-layer failure (unreachable port) — V5 gate Live, no HTTP success
// ============================================================================

#[test]
fn transport_error_routes_to_substrate_unreachable() {
    // Port 1 reliably refuses; transport error fires.
    let transport = HeartbeatTransport::new(
        "http://127.0.0.1:1/v3/heartbeat".to_owned(),
        shipped_trust(),
    );
    let result = transport.send(&fixture_envelope());
    match result {
        Err(RefusalToken::Unavailable(UnavailableReason::SubstrateUnreachable {
            substrate_id,
            transport_reason,
        })) => {
            assert_eq!(substrate_id, SubstrateId::SynthexV2);
            assert!(
                transport_reason.contains("substrate_unreachable"),
                "expected substrate_unreachable: prefix; got {transport_reason}"
            );
        }
        other => panic!("expected SubstrateUnreachable for transport error; got {other:?}"),
    }
}

// ============================================================================
// Envelope round-trip — serde shape
// ============================================================================

#[test]
fn envelope_serde_round_trips_preserves_all_fields() {
    let envelope = fixture_envelope();
    let s = serde_json::to_string(&envelope).expect("ser");
    let r: HeartbeatWireEnvelope = serde_json::from_str(&s).expect("de");
    assert_eq!(envelope, r);
    // Wire-shape audit: every field present in JSON
    assert!(s.contains("emitted_at_ms"));
    assert!(s.contains("cycle"));
    assert!(s.contains("heartbeat_source"));
    assert!(s.contains("boot_id"));
    assert!(s.contains("instance_id"));
    assert!(s.contains("skew_summary"));
    assert!(s.contains("alert_budget_remaining"));
}

#[test]
fn heartbeat_ack_serde_round_trips() {
    let ack = HeartbeatAck {
        cycle_acked: 42,
        synthex_v2_observed_at_ms: 1_735_689_600_015,
    };
    let s = serde_json::to_string(&ack).expect("ser");
    let r: HeartbeatAck = serde_json::from_str(&s).expect("de");
    assert_eq!(ack, r);
}

#[test]
fn skew_summary_serde_round_trips() {
    let s1 = SkewSummary {
        max_observed_skew_ms: 12,
        samples_observed: 10,
        had_refusals: false,
    };
    let s = serde_json::to_string(&s1).expect("ser");
    let r: SkewSummary = serde_json::from_str(&s).expect("de");
    assert_eq!(s1, r);
}

// ============================================================================
// V5 gate severity — Shipping status also gates? NO. Only NotShipped.
// (Shipping substrate may emit `SubstrateAuthored` refusals legitimately.)
// ============================================================================

#[test]
fn v5_gate_shipping_status_does_not_short_circuit() {
    let mut t = SubstrateTrust::new();
    let _prev = t.set(
        SubstrateId::SynthexV2,
        TrustEntry {
            status: SubstrateParticipationStatus::Shipping,
            value: TrustValue::Unavailable,
        },
    );
    let transport = HeartbeatTransport::new(
        "http://127.0.0.1:1/v3/heartbeat".to_owned(),
        Arc::new(t),
    );
    let result = transport.send(&fixture_envelope());
    // Shipping → does NOT short-circuit; transport error fires through to
    // SubstrateUnreachable. (NOT EngineImagined.)
    match result {
        Err(RefusalToken::Unavailable(UnavailableReason::SubstrateUnreachable { .. })) => {}
        Err(RefusalToken::Unavailable(UnavailableReason::EngineImagined { .. })) => {
            panic!("Shipping status MUST NOT short-circuit to EngineImagined per NA-5");
        }
        other => panic!("expected SubstrateUnreachable for Shipping + transport err; got {other:?}"),
    }
}

// ============================================================================
// Envelope wire-shape audit — minimal heartbeat preserved verbatim
// (per CONV-3 + ZA-2: m16's actual Heartbeat is the wire's inner shape)
// ============================================================================

#[test]
fn envelope_preserves_minimal_heartbeat_verbatim() {
    let envelope = fixture_envelope();
    let s = serde_json::to_string(&envelope).expect("ser");
    let v: serde_json::Value = serde_json::from_str(&s).expect("de");
    // The minimal {emitted_at_ms, cycle} m16 Heartbeat IS the inner shape
    // — no fat envelope per Plan v2 § S2 amendment.
    let heartbeat = v.get("heartbeat").expect("heartbeat field present");
    assert_eq!(
        heartbeat.get("emitted_at_ms").and_then(serde_json::Value::as_u64),
        Some(1_735_689_600_000)
    );
    assert_eq!(heartbeat.get("cycle").and_then(serde_json::Value::as_u64), Some(42));
    // No `skew_envelope` or `alert_budget` fields INSIDE the heartbeat —
    // those are enrichment ONLY at the envelope level (ZA-2 ban on
    // fat-envelope fiction).
    assert!(heartbeat.get("skew_envelope").is_none());
    assert!(heartbeat.get("alert_budget").is_none());
}
