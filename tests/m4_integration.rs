//! Integration tests for m4 cascade_correlator (Wave-C2).
//!
//! Exercises the m4 public surface from outside the crate:
//!
//! - F11 anti-property: `CascadeClusterId::Display` hex-only after prefix.
//! - Eq + Hash collapse: duplicate logical clusters share one id.
//! - Boundary: empty step sequence yields empty output.
//! - Adversarial: `i64::MAX` timestamps survive saturating-arithmetic
//!   without panic.
//! - Contract regression: `DISPATCH_TRAILING_SLACK_NS` is locked at 60s in ns.
//! - Determinism: identical input across repeated invocations yields
//!   identical cluster ids.
//!
//! Note: per Wave-A1 scout finding F-A-02, `CascadeClusterId(pub String)`
//! exposes its inner string. Per the task brief we test the AS-IS public
//! surface and do NOT mutate the type.

#![allow(clippy::doc_markdown)]

use workflow_core::m4_cascade::{
    AtuinStep, CascadeCorrelator, CascadeCorrelatorConfig, DispatchRecord,
    DISPATCH_TRAILING_SLACK_NS,
};

fn step(id: &str, ts_ns: i64, session: &str) -> AtuinStep {
    AtuinStep {
        id: id.to_owned(),
        ts_ns,
        command: format!("cmd-{id}"),
        cwd: "/tmp".into(),
        session: session.to_owned(),
        exit: 0,
    }
}

fn dispatch(ts_ns: i64, pane: &str, session: &str) -> DispatchRecord {
    DispatchRecord {
        ts_ns,
        pane_label: pane.to_owned(),
        binary: "cc-dispatch".to_owned(),
        session: session.to_owned(),
    }
}

fn corr_min_pane(min_pane: usize) -> CascadeCorrelator {
    CascadeCorrelator::new(CascadeCorrelatorConfig {
        min_pane_count: min_pane,
        max_gap_ms: 30_000,
        ..CascadeCorrelatorConfig::default()
    })
}

// rationale: Anti-property F11 — Display suffix must be hex-only after
// the canonical `cascade_cluster_` prefix. No human-meaningful substring
// of pane labels may leak into the id surface.
#[test]
fn m4_cluster_id_display_is_hex_only_after_prefix() {
    let c = corr_min_pane(2);
    let steps = vec![
        step("a", 1_000_000_000, "session-ALPHA"),
        step("b", 1_500_000_000, "session-BETA"),
    ];
    let d = vec![
        dispatch(1_000_000_000, "ALPHA-LEFT-meaningful", "session-ALPHA"),
        dispatch(1_500_000_000, "BETA-RIGHT-meaningful", "session-BETA"),
    ];
    let clusters = c.correlate(&steps, &d);
    assert_eq!(clusters.len(), 1, "expected one cluster");
    let display = format!("{}", clusters[0].cluster_id);
    let suffix = display
        .strip_prefix("cascade_cluster_")
        .expect("opaque id must start with cascade_cluster_");
    assert_eq!(suffix.len(), 16, "expected 16-hex suffix, got {suffix:?}");
    assert!(
        suffix.chars().all(|ch| ch.is_ascii_hexdigit()),
        "F11 violation: suffix {suffix:?} contains non-hex",
    );
    for forbidden in ["ALPHA", "BETA", "LEFT", "RIGHT", "meaningful", "session"] {
        assert!(
            !suffix.contains(forbidden),
            "F11 leak: {forbidden:?} found in opaque suffix {suffix:?}",
        );
    }
}

// rationale: Invariant — Eq + Hash collapse duplicate logical clusters
// onto one identity in a HashSet (sorted pane labels are order-invariant
// under `assign_cluster_id`).
#[test]
fn m4_cluster_id_eq_hash_collapses_duplicate_clusters() {
    let c = corr_min_pane(2);
    let steps_a = vec![
        step("a", 1_000_000_000, "s1"),
        step("b", 1_500_000_000, "s2"),
    ];
    let dispatch_ab = vec![
        dispatch(1_000_000_000, "ALPHA-LEFT", "s1"),
        dispatch(1_500_000_000, "BETA-LEFT", "s2"),
    ];
    // Same logical cluster — same window, same labels (different order),
    // same step count.
    let steps_b = vec![
        step("a", 1_000_000_000, "s1"),
        step("b", 1_500_000_000, "s2"),
    ];
    let dispatch_ba = vec![
        dispatch(1_500_000_000, "BETA-LEFT", "s2"),
        dispatch(1_000_000_000, "ALPHA-LEFT", "s1"),
    ];
    let id_a = c.correlate(&steps_a, &dispatch_ab)[0].cluster_id.clone();
    let id_b = c.correlate(&steps_b, &dispatch_ba)[0].cluster_id.clone();
    assert_eq!(id_a, id_b, "logically-identical clusters must share id");
    let mut set: std::collections::HashSet<_> = std::collections::HashSet::new();
    set.insert(id_a);
    set.insert(id_b);
    assert_eq!(set.len(), 1, "Eq/Hash must collapse duplicate ids");
}

// rationale: Boundary — empty step sequence must yield empty output
// (no allocator overhead, no fabricated cluster).
#[test]
fn m4_correlate_handles_empty_step_sequence() {
    let c = corr_min_pane(2);
    let clusters = c.correlate(&[], &[]);
    assert!(clusters.is_empty(), "empty input must yield empty output");
    let clusters_with_dispatch = c.correlate(&[], &[dispatch(1, "X", "s1")]);
    assert!(
        clusters_with_dispatch.is_empty(),
        "empty steps + non-empty dispatch must still yield empty output",
    );
}

// rationale: Adversarial — i64::MAX timestamps must not panic the
// saturating arithmetic in `collect_pane_labels` (slack add) or in the
// DAG-depth sweep (diff between adjacent steps).
#[test]
fn m4_correlate_handles_i64_max_timestamps_without_panic() {
    let c = corr_min_pane(2);
    let steps = vec![
        step("a", i64::MAX - 10, "s1"),
        step("b", i64::MAX - 5, "s2"),
        step("c", i64::MAX, "s3"),
    ];
    let dispatch_rows = vec![dispatch(i64::MAX - 8, "ALPHA-LEFT", "s1")];
    // Must not panic.
    let clusters = c.correlate(&steps, &dispatch_rows);
    if !clusters.is_empty() {
        // If a cluster is emitted, its id is still well-formed (prefix +
        // 16-hex). The saturating boundary does not corrupt the surface.
        let display = format!("{}", clusters[0].cluster_id);
        assert!(display.starts_with("cascade_cluster_"));
        let suffix = &display["cascade_cluster_".len()..];
        assert_eq!(suffix.len(), 16);
    }
}

// rationale: Contract regression — `DISPATCH_TRAILING_SLACK_NS` is part
// of the public surface (m4 spec § 3). Locked at 60s in ns; any drift
// would change the windowing budget downstream.
#[test]
fn m4_dispatch_trailing_slack_const_is_60s_in_ns() {
    assert_eq!(
        DISPATCH_TRAILING_SLACK_NS, 60_000_000_000,
        "DISPATCH_TRAILING_SLACK_NS must be 60s in ns (60 * 10^9)",
    );
    // Cross-check the human-readable interpretation.
    assert_eq!(DISPATCH_TRAILING_SLACK_NS / 1_000_000_000, 60);
}

// rationale: Determinism — identical input across repeated invocations
// must yield identical cluster ids. Exercises the public API at habitat
// scale (20× repeats) to catch any future stateful drift.
#[test]
fn m4_correlate_is_deterministic_across_repeated_invocations() {
    let c = corr_min_pane(2);
    let steps = vec![
        step("a", 1_000_000_000, "s1"),
        step("b", 2_000_000_000, "s2"),
        step("c", 3_000_000_000, "s3"),
    ];
    let first = c.correlate(&steps, &[]);
    assert!(!first.is_empty());
    let baseline_id = first[0].cluster_id.clone();
    let baseline_step_count = first[0].step_count;
    let baseline_pane_count = first[0].pane_count;
    for _ in 0..20_u32 {
        let again = c.correlate(&steps, &[]);
        assert_eq!(again.len(), first.len());
        assert_eq!(again[0].cluster_id, baseline_id);
        assert_eq!(again[0].step_count, baseline_step_count);
        assert_eq!(again[0].pane_count, baseline_pane_count);
    }
}

// rationale: Contract regression — the prefix is locked to
// `cascade_cluster_` (the F11 mitigation makes the prefix a stable
// structural marker, separate from the opaque suffix).
#[test]
fn m4_cluster_id_prefix_is_locked_to_cascade_cluster() {
    let c = corr_min_pane(2);
    let steps = vec![step("a", 1, "s1"), step("b", 2, "s2")];
    let clusters = c.correlate(&steps, &[]);
    assert_eq!(clusters.len(), 1);
    let display = format!("{}", clusters[0].cluster_id);
    assert!(
        display.starts_with("cascade_cluster_"),
        "prefix drift: got {display:?}",
    );
}
