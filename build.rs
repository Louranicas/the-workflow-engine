//! Build script — emits `cargo:rustc-cfg=povm_calibrated` when the POVM CR-2
//! marker is present in the build environment.
//!
//! Per m8 spec § 5 (and § 10 "env-var-only per cluster-D spec § m8 for
//! reproducibility"), this script does NOT probe POVM `:8125` over HTTP at
//! build time — the runtime mirror in `src/m8_povm_build_prereq/health.rs`
//! handles live verification at startup. The compile-time gate is
//! intentionally a `rustc-cfg` flag, NOT a Cargo `[features]` flag, so that
//! `cargo --features full`, `--all-features`, or any future feature-set
//! cannot activate it (F7 / AP-V7-09 defense).

fn main() {
    println!("cargo:rerun-if-env-changed=POVM_CR2_DEPLOYED");
    println!("cargo:rerun-if-env-changed=POVM_HEALTH_URL");

    let cr2_deployed = std::env::var("POVM_CR2_DEPLOYED").is_ok_and(|v| v == "1");

    if cr2_deployed {
        println!("cargo:rustc-cfg=povm_calibrated");
    } else {
        // Warnings surface in `cargo build` stderr without failing the build
        // at this point. Per m8 spec § 5: the `compile_error!` tombstones at
        // `#[cfg(not(povm_calibrated))]` POVM-read sites are what fail the
        // build when an in-tree POVM-reading path is reached.
        println!(
            "cargo:warning=POVM CR-2 (magnitude-weighted learning_health) not verified."
        );
        println!(
            "cargo:warning=Set POVM_CR2_DEPLOYED=1 after confirming povm-v2 commit e2a8ed3 is live."
        );
        println!(
            "cargo:warning=See: ~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md"
        );
    }
}
