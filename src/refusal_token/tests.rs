//! `RefusalToken` tests — V1 authorship-typed refusal envelope (Plan v2
//! v0.2.0 §3 Phase 5 step 5 per ADR D-S1004XXX-04).

use super::{
    EngineRefusalReason, ModuleId, OperatorRefusalReason, RefusalPayload, RefusalToken,
    SubstrateId,
};
use crate::m32_dispatcher::EscapeSurfaceProfile;

// ============================================================================
// Constructor + accessor tests.
// ============================================================================

#[test]
fn substrate_authored_constructor_sets_fields_and_no_payload() {
    let t = RefusalToken::substrate_authored(SubstrateId::Stcortex, "refuse_write_no_consumer".to_owned());
    match &t {
        RefusalToken::SubstrateAuthored {
            substrate_id,
            substrate_reason,
            payload,
        } => {
            assert_eq!(*substrate_id, SubstrateId::Stcortex);
            assert_eq!(substrate_reason, "refuse_write_no_consumer");
            assert!(payload.is_none());
        }
        other => panic!("expected SubstrateAuthored, got {other:?}"),
    }
    assert_eq!(t.substrate_id(), Some(SubstrateId::Stcortex));
    assert!(t.is_substrate_authored());
    assert!(!t.is_engine_imagined());
}

#[test]
fn engine_authored_wraps_v0_1_0_refusal_reason() {
    // EngineRefusalReason is an alias of the v0.1.0 m32 RefusalReason —
    // construct a v0.1.0 variant and wrap it.
    let reason: EngineRefusalReason =
        crate::m32_dispatcher::RefusalReason::WorkflowNotBanked;
    let t = RefusalToken::engine_authored(ModuleId::M32, reason.clone());
    match &t {
        RefusalToken::EngineAuthored {
            module_id,
            engine_reason,
            payload,
        } => {
            assert_eq!(*module_id, ModuleId::M32);
            assert_eq!(*engine_reason, reason);
            assert!(payload.is_none());
        }
        other => panic!("expected EngineAuthored, got {other:?}"),
    }
    assert_eq!(t.substrate_id(), None);
    assert!(!t.is_substrate_authored());
    assert!(!t.is_engine_imagined());
}

#[test]
fn operator_authored_carries_three_distinct_reason_shapes() {
    // Malformed
    let t_malformed = RefusalToken::operator_authored(OperatorRefusalReason::Malformed {
        context: "unknown surface".to_owned(),
    });
    assert!(matches!(
        t_malformed,
        RefusalToken::OperatorAuthored {
            operator_reason: OperatorRefusalReason::Malformed { .. },
            ..
        }
    ));
    // NotNow with optional context
    let t_now = RefusalToken::operator_authored(OperatorRefusalReason::NotNow { context: None });
    assert!(matches!(
        t_now,
        RefusalToken::OperatorAuthored {
            operator_reason: OperatorRefusalReason::NotNow { context: None },
            ..
        }
    ));
    // RequestReframing
    let t_reframe = RefusalToken::operator_authored(OperatorRefusalReason::RequestReframing {
        suggested_reframing: "try smaller batch".to_owned(),
    });
    assert!(matches!(
        t_reframe,
        RefusalToken::OperatorAuthored {
            operator_reason: OperatorRefusalReason::RequestReframing { .. },
            ..
        }
    ));
    // None of these are substrate-authored.
    assert!(!t_malformed.is_substrate_authored());
    assert!(!t_now.is_substrate_authored());
    assert!(!t_reframe.is_substrate_authored());
}

// ============================================================================
// NA-5 sub-tagging — the critical audit-distinguishability tests.
// ============================================================================

#[test]
fn unavailable_engine_imagined_is_distinguishable_from_substrate_authored() {
    // NA-5: V5 in-engine-receiver-only fallback emits EngineImagined.
    let engine_imagined = RefusalToken::unavailable_engine_imagined(
        SubstrateId::Stcortex,
        "stcortex consumer-trust schema not shipped".to_owned(),
    );
    assert!(engine_imagined.is_engine_imagined());
    assert!(
        !engine_imagined.is_substrate_authored(),
        "EngineImagined MUST NOT be classified as substrate-authored — this is the NA-5 audit-distinguishability contract"
    );

    // Substrate explicitly emitting "unavailable" — the SubstrateAuthored
    // sub-variant inside Unavailable.
    let substrate_unavail = RefusalToken::unavailable_substrate_authored(
        SubstrateId::Stcortex,
        "refuse_write_no_consumer".to_owned(),
    );
    assert!(!substrate_unavail.is_engine_imagined());
    assert!(
        substrate_unavail.is_substrate_authored(),
        "Unavailable::SubstrateAuthored IS substrate-authored — distinct from EngineImagined"
    );

    // Both report the substrate_id.
    assert_eq!(engine_imagined.substrate_id(), Some(SubstrateId::Stcortex));
    assert_eq!(substrate_unavail.substrate_id(), Some(SubstrateId::Stcortex));
}

#[test]
fn unavailable_substrate_unreachable_is_third_distinct_case() {
    let unreachable = RefusalToken::unavailable_substrate_unreachable(
        SubstrateId::HabitatConductor,
        "connection refused on :8141".to_owned(),
    );
    assert!(!unreachable.is_engine_imagined());
    assert!(
        !unreachable.is_substrate_authored(),
        "SubstrateUnreachable is the THIRD sub-tag — neither engine-imagined nor substrate-authored (it's substrate-exists-but-cannot-be-contacted)"
    );
    assert_eq!(
        unreachable.substrate_id(),
        Some(SubstrateId::HabitatConductor)
    );
}

#[test]
fn three_unavailable_sub_tags_are_pairwise_distinct_in_audit() {
    // The DX-V5.b 3-variant sub-tag prevents audit-indistinguishability
    // between the three Unavailable modes. This test exercises all three
    // simultaneously to confirm pairwise distinguishability.
    let imagined = RefusalToken::unavailable_engine_imagined(
        SubstrateId::Atuin,
        "atuin push-emitter post-v0.2.0".to_owned(),
    );
    let unreachable = RefusalToken::unavailable_substrate_unreachable(
        SubstrateId::Atuin,
        "WAL lock contention".to_owned(),
    );
    let authored = RefusalToken::unavailable_substrate_authored(
        SubstrateId::Atuin,
        "read_quota_exceeded".to_owned(),
    );
    // All three carry the same SubstrateId.
    assert_eq!(imagined.substrate_id(), Some(SubstrateId::Atuin));
    assert_eq!(unreachable.substrate_id(), Some(SubstrateId::Atuin));
    assert_eq!(authored.substrate_id(), Some(SubstrateId::Atuin));
    // But the classifications are distinct.
    assert!(imagined.is_engine_imagined());
    assert!(!imagined.is_substrate_authored());
    assert!(!unreachable.is_engine_imagined());
    assert!(!unreachable.is_substrate_authored());
    assert!(!authored.is_engine_imagined());
    assert!(authored.is_substrate_authored());
    // Pairwise inequality (the structural enforcement).
    assert_ne!(imagined, unreachable);
    assert_ne!(imagined, authored);
    assert_ne!(unreachable, authored);
}

// ============================================================================
// Payload tests — RefusalPayload typed envelope.
// ============================================================================

#[test]
fn refusal_payload_acceptable_escape_surface_couples_to_d_s1002127_02() {
    // RefusalPayload::AcceptableEscapeSurface carries the
    // EscapeSurfaceProfile the substrate would have accepted instead —
    // pairs naturally with D-S1002127-02 (7-variant ordinal).
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
        other => panic!("expected SubstrateAuthored with payload, got {other:?}"),
    }
}

// ============================================================================
// Serde round-trip tests — every variant.
// ============================================================================

#[test]
fn serde_round_trip_substrate_authored() {
    let t = RefusalToken::substrate_authored(
        SubstrateId::Stcortex,
        "refuse_write_no_consumer".to_owned(),
    );
    let s = serde_json::to_string(&t).expect("ser");
    let r: RefusalToken = serde_json::from_str(&s).expect("de");
    assert_eq!(t, r);
}

#[test]
fn serde_round_trip_engine_authored() {
    let t = RefusalToken::engine_authored(
        ModuleId::M32,
        crate::m32_dispatcher::RefusalReason::WorkflowNotBanked,
    );
    let s = serde_json::to_string(&t).expect("ser");
    let r: RefusalToken = serde_json::from_str(&s).expect("de");
    assert_eq!(t, r);
}

#[test]
fn serde_round_trip_operator_authored_all_three_reasons() {
    for reason in [
        OperatorRefusalReason::Malformed {
            context: "bad".to_owned(),
        },
        OperatorRefusalReason::NotNow { context: None },
        OperatorRefusalReason::NotNow {
            context: Some("busy".to_owned()),
        },
        OperatorRefusalReason::RequestReframing {
            suggested_reframing: "smaller".to_owned(),
        },
    ] {
        let t = RefusalToken::operator_authored(reason);
        let s = serde_json::to_string(&t).expect("ser");
        let r: RefusalToken = serde_json::from_str(&s).expect("de");
        assert_eq!(t, r);
    }
}

#[test]
fn serde_round_trip_unavailable_all_three_sub_tags() {
    let cases = [
        RefusalToken::unavailable_engine_imagined(SubstrateId::Stcortex, "x".to_owned()),
        RefusalToken::unavailable_substrate_unreachable(
            SubstrateId::HabitatConductor,
            "y".to_owned(),
        ),
        RefusalToken::unavailable_substrate_authored(SubstrateId::Atuin, "z".to_owned()),
    ];
    for t in cases {
        let s = serde_json::to_string(&t).expect("ser");
        let r: RefusalToken = serde_json::from_str(&s).expect("de");
        assert_eq!(t, r);
    }
}

#[test]
fn serde_round_trip_with_payload() {
    let t = RefusalToken::SubstrateAuthored {
        substrate_id: SubstrateId::Stcortex,
        substrate_reason: "x".to_owned(),
        payload: Some(RefusalPayload::AcceptableEscapeSurface(
            EscapeSurfaceProfile::FileWrite,
        )),
    };
    let s = serde_json::to_string(&t).expect("ser");
    let r: RefusalToken = serde_json::from_str(&s).expect("de");
    assert_eq!(t, r);
}

// ============================================================================
// SubstrateId + ModuleId enumeration — verifies all 10 substrate IDs +
// 7 module IDs are constructible and serde-distinct.
// ============================================================================

#[test]
fn all_substrate_ids_serde_distinct() {
    let ids = [
        SubstrateId::Atuin,
        SubstrateId::Stcortex,
        SubstrateId::HabitatConductor,
        SubstrateId::HabitatInjection,
        SubstrateId::Cc5LoopClocks,
        SubstrateId::Watcher,
        SubstrateId::Ralph,
        SubstrateId::CargoBuildGraph,
        SubstrateId::Lcm,
        SubstrateId::SynthexV2,
    ];
    let strings: Vec<String> = ids
        .iter()
        .map(|id| serde_json::to_string(id).expect("ser"))
        .collect();
    let unique: std::collections::HashSet<&String> = strings.iter().collect();
    assert_eq!(
        unique.len(),
        strings.len(),
        "every SubstrateId variant must serialise to a distinct string"
    );
}

#[test]
fn all_module_ids_serde_distinct() {
    let ids = [
        ModuleId::M9,
        ModuleId::M13,
        ModuleId::M32,
        ModuleId::M33,
        ModuleId::M40,
        ModuleId::M41,
        ModuleId::M42,
    ];
    let strings: Vec<String> = ids
        .iter()
        .map(|id| serde_json::to_string(id).expect("ser"))
        .collect();
    let unique: std::collections::HashSet<&String> = strings.iter().collect();
    assert_eq!(unique.len(), strings.len());
}
