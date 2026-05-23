//! V2 substrate back-pressure tests (Plan v2 v0.2.0 §3 Phase 8 +
//! DX-2 + NA-8).

use super::{
    BackPressureRegistry, BackPressureSeverity, BackPressureSignal, SubstrateBackPressureMode,
};
use crate::refusal_token::SubstrateId;

// ============================================================================
// SubstrateBackPressureMode enum + default
// ============================================================================

#[test]
fn substrate_back_pressure_mode_default_is_pull_per_dx2() {
    assert_eq!(
        SubstrateBackPressureMode::default(),
        SubstrateBackPressureMode::Pull,
        "v0.2.0 default per DX-2 + Plan v2 §3 Phase 8 step 2 = Pull"
    );
}

#[test]
fn substrate_back_pressure_mode_all_three_variants_serde_distinct() {
    let modes = [
        SubstrateBackPressureMode::Push,
        SubstrateBackPressureMode::Pull,
        SubstrateBackPressureMode::Unavailable,
    ];
    let strings: Vec<String> = modes
        .iter()
        .map(|m| serde_json::to_string(m).expect("ser"))
        .collect();
    let unique: std::collections::HashSet<&String> = strings.iter().collect();
    assert_eq!(unique.len(), 3, "Push / Pull / Unavailable must serde-distinctly");
}

// ============================================================================
// BackPressureSeverity ordering
// ============================================================================

#[test]
fn back_pressure_severity_ordering_is_nominal_lt_elevated_lt_saturated_lt_refused() {
    assert!(BackPressureSeverity::Nominal < BackPressureSeverity::Elevated);
    assert!(BackPressureSeverity::Elevated < BackPressureSeverity::Saturated);
    assert!(BackPressureSeverity::Saturated < BackPressureSeverity::Refused);
}

#[test]
fn back_pressure_severity_serde_round_trips() {
    for sev in [
        BackPressureSeverity::Nominal,
        BackPressureSeverity::Elevated,
        BackPressureSeverity::Saturated,
        BackPressureSeverity::Refused,
    ] {
        let s = serde_json::to_string(&sev).expect("ser");
        let r: BackPressureSeverity = serde_json::from_str(&s).expect("de");
        assert_eq!(sev, r);
    }
}

// ============================================================================
// BackPressureSignal envelope
// ============================================================================

#[test]
fn back_pressure_signal_carries_substrate_severity_and_observed_at_ms() {
    let sig = BackPressureSignal::new(
        SubstrateId::Stcortex,
        BackPressureSeverity::Elevated,
        1_700_000_000_000,
    );
    assert_eq!(sig.substrate, SubstrateId::Stcortex);
    assert_eq!(sig.severity, BackPressureSeverity::Elevated);
    assert_eq!(sig.observed_at_ms, 1_700_000_000_000);
}

#[test]
fn back_pressure_signal_serde_round_trips() {
    let sig = BackPressureSignal::new(
        SubstrateId::Atuin,
        BackPressureSeverity::Saturated,
        42,
    );
    let s = serde_json::to_string(&sig).expect("ser");
    let r: BackPressureSignal = serde_json::from_str(&s).expect("de");
    assert_eq!(sig, r);
}

// ============================================================================
// BackPressureRegistry — per-substrate mode lookup
// ============================================================================

#[test]
fn registry_new_is_empty() {
    let r = BackPressureRegistry::new();
    assert!(r.is_empty());
    assert_eq!(r.len(), 0);
}

#[test]
fn registry_mode_for_unset_substrate_returns_default_pull() {
    let r = BackPressureRegistry::new();
    for substrate in [
        SubstrateId::Atuin,
        SubstrateId::Stcortex,
        SubstrateId::Watcher,
        SubstrateId::Ralph,
    ] {
        assert_eq!(
            r.mode_for(substrate),
            SubstrateBackPressureMode::Pull,
            "unset substrate {substrate:?} must default to Pull per DX-2"
        );
    }
}

#[test]
fn registry_set_mode_overrides_default_and_returns_prior() {
    let mut r = BackPressureRegistry::new();
    // First set: no prior.
    assert!(r
        .set_mode(SubstrateId::Stcortex, SubstrateBackPressureMode::Push)
        .is_none());
    assert_eq!(
        r.mode_for(SubstrateId::Stcortex),
        SubstrateBackPressureMode::Push
    );
    // Second set: returns prior.
    let prior = r
        .set_mode(SubstrateId::Stcortex, SubstrateBackPressureMode::Unavailable)
        .expect("prior must be Some");
    assert_eq!(prior, SubstrateBackPressureMode::Push);
    assert_eq!(
        r.mode_for(SubstrateId::Stcortex),
        SubstrateBackPressureMode::Unavailable
    );
}

#[test]
fn registry_all_pull_default_enumerates_known_substrates() {
    let r = BackPressureRegistry::all_pull_default();
    assert_eq!(r.len(), 10, "10 known substrates per SubstrateId variants");
    for (substrate, mode) in r.iter() {
        assert_eq!(
            mode,
            SubstrateBackPressureMode::Pull,
            "all_pull_default must set every substrate to Pull; {substrate:?} != Pull"
        );
    }
    // Spot-check a few known substrates are present.
    for substrate in [
        SubstrateId::Atuin,
        SubstrateId::Stcortex,
        SubstrateId::HabitatConductor,
        SubstrateId::Watcher,
        SubstrateId::Ralph,
    ] {
        assert_eq!(r.mode_for(substrate), SubstrateBackPressureMode::Pull);
    }
}

#[test]
fn registry_per_substrate_independence_set_one_does_not_affect_others() {
    // NA-8 reshape: heterogeneous substrate landscape. Setting stcortex
    // to Push must NOT change atuin's mode.
    let mut r = BackPressureRegistry::all_pull_default();
    r.set_mode(SubstrateId::Stcortex, SubstrateBackPressureMode::Push);
    assert_eq!(
        r.mode_for(SubstrateId::Stcortex),
        SubstrateBackPressureMode::Push
    );
    assert_eq!(
        r.mode_for(SubstrateId::Atuin),
        SubstrateBackPressureMode::Pull,
        "per-substrate independence: stcortex flip does not change atuin"
    );
    assert_eq!(
        r.mode_for(SubstrateId::HabitatConductor),
        SubstrateBackPressureMode::Pull
    );
}

#[test]
fn registry_iter_yields_all_set_substrates() {
    let mut r = BackPressureRegistry::new();
    r.set_mode(SubstrateId::Stcortex, SubstrateBackPressureMode::Push);
    r.set_mode(SubstrateId::Atuin, SubstrateBackPressureMode::Pull);
    r.set_mode(SubstrateId::Ralph, SubstrateBackPressureMode::Unavailable);
    let collected: std::collections::HashMap<SubstrateId, SubstrateBackPressureMode> =
        r.iter().collect();
    assert_eq!(collected.len(), 3);
    assert_eq!(
        collected.get(&SubstrateId::Stcortex),
        Some(&SubstrateBackPressureMode::Push)
    );
    assert_eq!(
        collected.get(&SubstrateId::Atuin),
        Some(&SubstrateBackPressureMode::Pull)
    );
    assert_eq!(
        collected.get(&SubstrateId::Ralph),
        Some(&SubstrateBackPressureMode::Unavailable)
    );
}
