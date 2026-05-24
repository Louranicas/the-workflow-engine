//! V5 substrate-trust tests (Plan v2 v0.2.0 §3 Phase 11 per DX-V5 +
//! DX-V5.b + NA-5).

use super::{
    SubstrateParticipationStatus, SubstrateTrust, TrustEntry, TrustValue,
};
use crate::refusal_token::{RefusalToken, SubstrateId, UnavailableReason};

// ============================================================================
// SubstrateParticipationStatus enum + default
// ============================================================================

#[test]
fn participation_status_default_is_not_shipped() {
    assert_eq!(
        SubstrateParticipationStatus::default(),
        SubstrateParticipationStatus::NotShipped,
        "v0.2.0 ship default per Plan v2 §3 Phase 11"
    );
}

#[test]
fn participation_status_3_variants_serde_distinct() {
    let statuses = [
        SubstrateParticipationStatus::NotShipped,
        SubstrateParticipationStatus::Shipping,
        SubstrateParticipationStatus::Live,
    ];
    let strings: Vec<String> = statuses
        .iter()
        .map(|s| serde_json::to_string(s).expect("ser"))
        .collect();
    let unique: std::collections::HashSet<&String> = strings.iter().collect();
    assert_eq!(unique.len(), 3);
}

// ============================================================================
// TrustEntry constructors
// ============================================================================

#[test]
fn trust_entry_not_shipped_constructor() {
    let e = TrustEntry::not_shipped();
    assert_eq!(e.status, SubstrateParticipationStatus::NotShipped);
    assert_eq!(e.value, TrustValue::Unavailable);
}

#[test]
fn trust_entry_live_constructors_produce_live_status() {
    let s = TrustEntry::live_score(0.5);
    assert_eq!(s.status, SubstrateParticipationStatus::Live);
    assert_eq!(s.value, TrustValue::Score(0.5));

    let f = TrustEntry::live_flag(true);
    assert_eq!(f.status, SubstrateParticipationStatus::Live);
    assert_eq!(f.value, TrustValue::Flag(true));

    let b = TrustEntry::live_budget(100);
    assert_eq!(b.status, SubstrateParticipationStatus::Live);
    assert_eq!(b.value, TrustValue::BudgetRemaining(100));
}

#[test]
fn trust_entry_serde_round_trip() {
    let cases = [
        TrustEntry::not_shipped(),
        TrustEntry::live_score(0.75),
        TrustEntry::live_flag(false),
        TrustEntry::live_budget(-1),
    ];
    for entry in cases {
        let s = serde_json::to_string(&entry).expect("ser");
        let r: TrustEntry = serde_json::from_str(&s).expect("de");
        assert_eq!(entry, r);
    }
}

// ============================================================================
// SubstrateTrust accumulator
// ============================================================================

#[test]
fn substrate_trust_new_is_empty() {
    let t = SubstrateTrust::new();
    assert!(t.is_empty());
    assert_eq!(t.len(), 0);
}

#[test]
fn substrate_trust_get_unset_returns_not_shipped_default() {
    let t = SubstrateTrust::new();
    for substrate in [
        SubstrateId::Stcortex,
        SubstrateId::Atuin,
        SubstrateId::HabitatConductor,
        SubstrateId::Ralph,
        SubstrateId::Watcher,
    ] {
        let entry = t.get(substrate);
        assert_eq!(entry.status, SubstrateParticipationStatus::NotShipped);
        assert_eq!(entry.value, TrustValue::Unavailable);
    }
}

#[test]
fn substrate_trust_set_then_get_round_trips() {
    let mut t = SubstrateTrust::new();
    let _ = t.set(SubstrateId::Stcortex, TrustEntry::live_score(0.85));
    assert_eq!(
        t.get(SubstrateId::Stcortex),
        TrustEntry::live_score(0.85)
    );
    // Other substrates remain at default.
    assert_eq!(
        t.get(SubstrateId::Atuin),
        TrustEntry::not_shipped()
    );
}

// ============================================================================
// NA-5 audit-distinguishability — the load-bearing contract
// ============================================================================

#[test]
fn is_substrate_imagined_for_unset_substrate_returns_true() {
    let t = SubstrateTrust::new();
    // Per NA-5: any substrate that has not been set is engine-imagined.
    for substrate in [
        SubstrateId::Stcortex,
        SubstrateId::Atuin,
        SubstrateId::Ralph,
        SubstrateId::Watcher,
        SubstrateId::Cc5LoopClocks,
    ] {
        assert!(
            t.is_substrate_imagined_for(substrate),
            "unset substrate {substrate:?} MUST be engine-imagined per NA-5"
        );
    }
}

#[test]
fn is_substrate_imagined_for_live_substrate_returns_false() {
    let mut t = SubstrateTrust::new();
    let _ = t.set(SubstrateId::Stcortex, TrustEntry::live_score(0.5));
    assert!(
        !t.is_substrate_imagined_for(SubstrateId::Stcortex),
        "Live substrate MUST NOT be engine-imagined"
    );
}

#[test]
fn is_substrate_imagined_for_shipping_substrate_returns_false() {
    let mut t = SubstrateTrust::new();
    let _ = t.set(
        SubstrateId::HabitatConductor,
        TrustEntry {
            status: SubstrateParticipationStatus::Shipping,
            value: TrustValue::Unavailable,
        },
    );
    assert!(
        !t.is_substrate_imagined_for(SubstrateId::HabitatConductor),
        "Shipping substrate MUST NOT be engine-imagined (it's substrate-unreachable transient)"
    );
}

#[test]
fn refusal_for_unavailable_routes_correct_na5_sub_tag_per_status() {
    let mut t = SubstrateTrust::new();
    // 1. NotShipped (default) → EngineImagined.
    let r1 = t.refusal_for_unavailable(SubstrateId::Atuin, "no schema yet");
    match &r1 {
        RefusalToken::Unavailable(UnavailableReason::EngineImagined {
            substrate_id,
            reason,
        }) => {
            assert_eq!(*substrate_id, SubstrateId::Atuin);
            // Zen #2 post-v0.2.0 hardening: reason is prefixed with the
            // status tag so log-grep audits can distinguish branches.
            assert_eq!(reason, "engine_imagined:no schema yet");
        }
        other => panic!("expected EngineImagined; got {other:?}"),
    }
    assert!(r1.is_engine_imagined());
    assert!(!r1.is_substrate_authored());

    // 2. Shipping → SubstrateUnreachable.
    let _ = t.set(
        SubstrateId::HabitatConductor,
        TrustEntry {
            status: SubstrateParticipationStatus::Shipping,
            value: TrustValue::Unavailable,
        },
    );
    let r2 = t.refusal_for_unavailable(SubstrateId::HabitatConductor, "transient");
    match &r2 {
        RefusalToken::Unavailable(UnavailableReason::SubstrateUnreachable {
            substrate_id,
            transport_reason,
        }) => {
            assert_eq!(*substrate_id, SubstrateId::HabitatConductor);
            assert_eq!(transport_reason, "substrate_unreachable:transient");
        }
        other => panic!("expected SubstrateUnreachable; got {other:?}"),
    }
    assert!(!r2.is_engine_imagined());
    assert!(!r2.is_substrate_authored());

    // 3. Live → Unavailable::SubstrateAuthored.
    let _ = t.set(SubstrateId::Stcortex, TrustEntry::live_score(0.5));
    let r3 = t.refusal_for_unavailable(SubstrateId::Stcortex, "rate_limited");
    match &r3 {
        RefusalToken::Unavailable(UnavailableReason::SubstrateAuthored {
            substrate_id,
            substrate_reason,
        }) => {
            assert_eq!(*substrate_id, SubstrateId::Stcortex);
            assert_eq!(substrate_reason, "substrate_authored:rate_limited");
        }
        other => panic!("expected SubstrateAuthored unavailable; got {other:?}"),
    }
    assert!(!r3.is_engine_imagined());
    assert!(r3.is_substrate_authored());
}

#[test]
fn na5_audit_distinguishability_three_sub_tags_pairwise_distinct_via_trust() {
    let mut t = SubstrateTrust::new();
    // Three substrates, three different statuses → three distinct sub-tags.
    let _ = t.set(SubstrateId::Atuin, TrustEntry::not_shipped());
    let _ = t.set(
        SubstrateId::HabitatConductor,
        TrustEntry {
            status: SubstrateParticipationStatus::Shipping,
            value: TrustValue::Unavailable,
        },
    );
    let _ = t.set(SubstrateId::Stcortex, TrustEntry::live_score(0.9));

    let r_atuin = t.refusal_for_unavailable(SubstrateId::Atuin, "x");
    let r_conductor =
        t.refusal_for_unavailable(SubstrateId::HabitatConductor, "x");
    let r_stcortex = t.refusal_for_unavailable(SubstrateId::Stcortex, "x");

    // Pairwise inequality — structural enforcement of NA-5
    // audit-distinguishability.
    assert_ne!(r_atuin, r_conductor);
    assert_ne!(r_atuin, r_stcortex);
    assert_ne!(r_conductor, r_stcortex);

    // Each substrate retains its own SubstrateId in the sub-tag.
    assert_eq!(r_atuin.substrate_id(), Some(SubstrateId::Atuin));
    assert_eq!(
        r_conductor.substrate_id(),
        Some(SubstrateId::HabitatConductor)
    );
    assert_eq!(r_stcortex.substrate_id(), Some(SubstrateId::Stcortex));
}

#[test]
fn substrate_participation_status_accessor_independent_per_substrate() {
    // Per NA-8 reshape pattern: setting one substrate's status MUST
    // NOT affect another's.
    let mut t = SubstrateTrust::new();
    let _ = t.set(SubstrateId::Stcortex, TrustEntry::live_score(0.9));
    assert_eq!(
        t.substrate_participation_status(SubstrateId::Stcortex),
        SubstrateParticipationStatus::Live
    );
    assert_eq!(
        t.substrate_participation_status(SubstrateId::Atuin),
        SubstrateParticipationStatus::NotShipped,
        "per-substrate independence: stcortex set does not change atuin"
    );
    // Per-substrate accessor matches per-substrate get.
    assert_eq!(
        t.get(SubstrateId::Atuin),
        TrustEntry::not_shipped()
    );
}
