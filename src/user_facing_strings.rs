//! Registry of every user-facing string in the workflow-trace CLIs.
//!
//! Per m10 spec § 3 + § 5: keys use `<module>.<context>.<variant>`
//! convention (e.g. `m12.report.header`, `m32.dispatch.trap_surface_prefix`,
//! `m11.sunset.warning`). New user-facing strings MUST be registered here
//! before the PR merges so they pass through the m10 Ember 7-trait CI gate
//! in `tests/ember_gate.rs`. Pure structured data (JSON / YAML / TSV) is
//! out of scope and is excluded from the registry by convention.
//!
//! # Day-1 status
//!
//! Empty registry. Downstream modules populate as they ship:
//!
//! - m11 (Cluster D Day-1) → `m11.sunset.warning`, …
//! - m12 (Cluster C Day-2+) → `m12.report.header`, …
//! - m23 (Cluster F KEYSTONE Days 5-7) → `m23.proposer.message`, …
//! - m32 (Cluster G Day 14+) → `m32.dispatch.trap_surface_prefix`, …
//!
//! The CI gate test in `tests/ember_gate.rs` walks this slice on every
//! `cargo test --release` invocation; the empty Day-1 registry passes
//! vacuously and acts as the discipline anchor until the registry grows.

/// All user-facing strings keyed by a stable `<module>.<context>.<variant>`
/// identifier. Walked by `tests/ember_gate.rs` on every CI run.
pub static ALL: &[(&str, &str)] = &[];

#[cfg(test)]
mod tests {
    use super::ALL;

    #[test]
    fn day_1_registry_is_empty() {
        // Spec § 5: Day-1 ships with an empty registry; downstream modules
        // populate it. This test documents the Day-1 state explicitly so
        // adding the first entry triggers a coordinated PR review (the
        // assertion fails the moment ALL grows).
        assert_eq!(
            ALL.len(),
            0,
            "Day-1 registry must remain empty until a downstream module \
             coordinates with m10 to add its first user-facing string"
        );
    }
}
