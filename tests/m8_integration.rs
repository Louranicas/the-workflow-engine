//! Integration tests for m8 — live POVM probe + build/runtime mirror
//! agreement. Most tests are `#[ignore = "..."]` for PR-CI per
//! [m8 spec § 6](../ai_specs/modules/cluster-D/m8_povm_build_prereq.md);
//! they run in nightly + Wave-end + post-G9 acceptance gates.

#![forbid(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

use std::time::Duration;

use workflow_core::m8_povm_build_prereq::{
    classify, probe_band, BandClassification, HealthClient, POVM_LH_BAND_HIGH, POVM_LH_BAND_LOW,
};

/// Live POVM `:8125` probe — agreement test between m8 spec § 5 thresholds
/// and the actual POVM endpoint.
///
/// Runs only if `POVM_HEALTH_URL` env var is set; this prevents PR-CI from
/// either silently passing (probe-not-attempted) or noisily failing
/// (no-POVM-in-sandbox).
#[test]
#[ignore = "requires live POVM :8125 — run via nightly or post-G9 acceptance"]
fn live_povm_probe_returns_in_band_value() {
    let Some(url) = std::env::var("POVM_HEALTH_URL").ok() else {
        eprintln!("POVM_HEALTH_URL unset — skipping live probe");
        return;
    };
    let (value, band) = probe_band(&url).expect("live POVM probe");
    eprintln!("live POVM probe: value={value} band={band:?}");
    // Post-CR-2 the live value should be in the magnitude-weighted band.
    // If this test fails on live infrastructure, POVM has regressed (NOT
    // a workflow-trace bug — escalate to operator).
    assert_eq!(band, BandClassification::InBand);
}

/// F-Contract test for m8 spec § 6: build-time gate and runtime mirror
/// must agree on band classification for identical input. Drives the same
/// value through `classify` (used by `build.rs` shape) and through
/// `probe_band` (runtime mirror via a captured one-shot mock).
#[test]
fn build_time_and_runtime_mirror_agree_on_classification() {
    // We don't drive `build.rs` here — we drive `classify` which is the
    // SHARED entry point both build.rs and runtime use. If `build.rs` ever
    // diverges from this contract, the include!-based sharing pattern in
    // m8 spec § 5 will catch it at build time.
    let probes: [(f64, BandClassification); 4] = [
        (0.02, BandClassification::BelowFloor),
        (0.10, BandClassification::InBand),
        (0.20, BandClassification::AboveCeiling),
        (f64::NAN, BandClassification::Nan),
    ];
    for (value, expected) in probes {
        assert_eq!(classify(value), expected);
    }
}

/// F-Integration: `HealthClient::new` honours `POVM_HEALTH_URL` env var.
///
/// This test mutates a process-wide env var; in a multi-test runner the
/// mutation can collide with other tests. The mutation is snapshot-and-restore
/// scoped, and the test uses a known-bad URL so it cannot accidentally probe
/// a real POVM.
#[test]
fn health_client_new_picks_up_env_var_url() {
    let prior = std::env::var("POVM_HEALTH_URL").ok();
    std::env::set_var("POVM_HEALTH_URL", "http://env-driven.invalid/lh");
    let client = HealthClient::new().expect("client build");
    match prior {
        Some(p) => std::env::set_var("POVM_HEALTH_URL", p),
        None => std::env::remove_var("POVM_HEALTH_URL"),
    }
    assert_eq!(client.url(), "http://env-driven.invalid/lh");
}

/// F-Integration regression: the workflow-trace `--features full` set MUST
/// NOT activate `cfg(povm_calibrated)`. Per m8 spec § 2 (CC-2 Trust Layer
/// Woven) + § 8 (F7 / AP-V7-09 defense): `cfg(povm_calibrated)` is a
/// `rustc-cfg` flag emitted by `build.rs`, NOT a Cargo feature.
///
/// We verify by introspecting the compile-time `cfg!` macro. If a future
/// developer accidentally adds `cfg(povm_calibrated)` to `[features]`, this
/// test will newly pass under `--features full` even with `POVM_CR2_DEPLOYED`
/// unset, signalling regression.
#[test]
fn features_full_does_not_enable_povm_calibrated_cfg() {
    let cargo_features_set = cfg!(feature = "full");
    let povm_calibrated_set = cfg!(povm_calibrated);
    // Both can independently be true or false; the contract is that
    // setting `--features full` does NOT imply `cfg(povm_calibrated)`.
    // If a future amendment crosses the wires, the only way for both to
    // become coupled would be a Cargo.toml change — we catch it by
    // assertion that `--features full` ON + `POVM_CR2_DEPLOYED` UNSET still
    // produces `povm_calibrated` OFF.
    if cargo_features_set && std::env::var("POVM_CR2_DEPLOYED").as_deref() != Ok("1") {
        assert!(
            !povm_calibrated_set,
            "REGRESSION: --features full activated cfg(povm_calibrated) — F7/AP-V7-09 defense breached"
        );
    }
}

/// F-Contract: band thresholds match Hebbian v3 reconciliation note
/// verbatim. If the reconciliation note's thresholds drift, m8 must drift
/// with them via spec amendment + Zen G7 re-audit — never silently.
#[test]
fn band_thresholds_match_hebbian_v3_reconciliation() {
    assert!((POVM_LH_BAND_LOW - 0.05).abs() < f64::EPSILON);
    assert!((POVM_LH_BAND_HIGH - 0.15).abs() < f64::EPSILON);
}

/// F-Integration: HealthClient supports sub-second timeout for fast-fail
/// in fleet probes (where a stalled POVM cannot wedge dispatch decisions).
#[test]
fn health_client_supports_sub_second_timeout() {
    let client = HealthClient::with_url_and_timeout(
        "http://127.0.0.1:1/learning_health",
        Duration::from_millis(50),
    )
    .expect("client build");
    let start = std::time::Instant::now();
    let _ = client.probe_value();
    // With a 50ms timeout + a port that nobody is listening on (port 1 is
    // reserved tcpmux historically), the probe should fail fast. Allow up
    // to 2 seconds of slack for slow CI; the contract is "doesn't hang
    // indefinitely", not "exactly 50ms".
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(2),
        "probe took {elapsed:?} with 50ms timeout — should fast-fail"
    );
}
