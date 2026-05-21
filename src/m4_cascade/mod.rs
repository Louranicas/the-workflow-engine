//! `m4_cascade_correlator` — Cluster B observer that correlates atuin
//! tool-call rows into opaque cascade-cluster identifiers.
//!
//! F11 (cascade monoculture) exclusive owner: cluster identifiers are
//! one-way FNV-1a hashes; downstream consumers must treat them as opaque.
//! See m4 spec § 7.

pub mod cluster_id;
pub mod error;

use std::path::PathBuf;

pub use cluster_id::{assign_cluster_id, fnv1a_64, CascadeClusterId};
pub use error::CascadeError;

/// Trailing-window slack (ns) within which a `DispatchRecord` is considered
/// part of a step group's pane fan-out. Hardening pass: extracted from the
/// previously-inline `60_000_000_000` magic constant in
/// [`collect_pane_labels`]. 60s wall-clock — well above the 30s
/// `max_gap_ms` default and aligned with the m4 spec § 3 windowing budget.
pub const DISPATCH_TRAILING_SLACK_NS: i64 = 60_000_000_000;

/// A single atuin tool-call row.
#[derive(Debug, Clone)]
pub struct AtuinStep {
    /// atuin row primary key (ULID).
    pub id: String,
    /// Wall-clock timestamp in nanoseconds.
    pub ts_ns: i64,
    /// Command verbatim.
    pub command: String,
    /// Working directory.
    pub cwd: String,
    /// Session identifier.
    pub session: String,
    /// Process exit code.
    pub exit: i32,
}

/// A cc-* dispatch log entry (atuin row whose command matches cc-* prefix).
#[derive(Debug, Clone)]
pub struct DispatchRecord {
    /// Wall-clock timestamp in nanoseconds.
    pub ts_ns: i64,
    /// Fleet pane label (e.g. `ALPHA-LEFT`).
    pub pane_label: String,
    /// Binary invoked (e.g. `cc-dispatch`).
    pub binary: String,
    /// Session identifier.
    pub session: String,
}

/// A correlated multi-pane cascade event.
#[derive(Debug, Clone)]
pub struct CascadeCluster {
    /// Opaque identifier.
    pub cluster_id: CascadeClusterId,
    /// Start of the temporal window (ns since epoch).
    pub window_start_ns: i64,
    /// End of the temporal window (ns since epoch).
    pub window_end_ns: i64,
    /// Distinct pane labels participating in the cluster.
    pub pane_count: usize,
    /// Total steps recorded in the cluster.
    pub step_count: usize,
    /// `true` if at least one inter-step gap > `max_gap_ms` was observed.
    pub has_temporal_gaps: bool,
    /// Maximum DAG depth (Kahn topological sort).
    pub dag_depth: usize,
    /// Wall-clock time the cluster was emitted.
    pub observed_at_ms: i64,
}

/// Cascade correlator configuration.
#[derive(Debug, Clone)]
pub struct CascadeCorrelatorConfig {
    /// Maximum inter-step gap within a cluster (ms).
    pub max_gap_ms: i64,
    /// Minimum distinct panes for a cluster to emit.
    pub min_pane_count: usize,
    /// Sliding-window span (ms).
    pub window_ms: i64,
    /// Per-cluster step cap (DoS / OOM guard).
    pub max_steps_per_cluster: usize,
    /// atuin SQLite path.
    pub atuin_db_path: PathBuf,
}

impl Default for CascadeCorrelatorConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned());
        Self {
            max_gap_ms: 30_000,
            min_pane_count: 2,
            window_ms: 300_000,
            max_steps_per_cluster: 500,
            atuin_db_path: PathBuf::from(format!("{home}/.local/share/atuin/history.db")),
        }
    }
}

/// The cascade correlator.
pub struct CascadeCorrelator {
    config: CascadeCorrelatorConfig,
}

impl std::fmt::Debug for CascadeCorrelator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CascadeCorrelator")
            .field("config", &self.config)
            .finish()
    }
}

impl CascadeCorrelator {
    /// Construct with the given configuration.
    #[must_use]
    pub fn new(config: CascadeCorrelatorConfig) -> Self {
        Self { config }
    }

    /// Borrow the configuration snapshot.
    #[must_use]
    pub fn config(&self) -> &CascadeCorrelatorConfig {
        &self.config
    }

    /// Correlate a batch of steps + dispatch records into zero or more
    /// `CascadeCluster`s. Pure function; deterministic on identical input.
    #[must_use]
    pub fn correlate(
        &self,
        steps: &[AtuinStep],
        dispatch_records: &[DispatchRecord],
    ) -> Vec<CascadeCluster> {
        if steps.is_empty() {
            return Vec::new();
        }
        let max_gap_ns = self.config.max_gap_ms.saturating_mul(1_000_000);
        let mut sorted_steps: Vec<&AtuinStep> = steps.iter().collect();
        sorted_steps.sort_by_key(|s| s.ts_ns);

        // Sliding-window cluster accumulator.
        let mut clusters: Vec<Vec<&AtuinStep>> = Vec::new();
        for step in &sorted_steps {
            let merged = if let Some(last) = clusters.last_mut() {
                let prev_end_ns = last.last().map_or(step.ts_ns, |s| s.ts_ns);
                if step.ts_ns.saturating_sub(prev_end_ns) <= max_gap_ns
                    && last.len() < self.config.max_steps_per_cluster
                {
                    last.push(*step);
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if !merged {
                clusters.push(vec![*step]);
            }
        }

        // SF3 silent-failure hardening (S1002388 W2): `now_ms` returns
        // `Option<i64>` — `None` on a genuine clock fault / overflow.
        // We log the fault before falling back to the `0` sentinel so a
        // 1970-epoch `observed_at_ms` is never emitted silently. The
        // sentinel is retained (rather than skipping the whole batch)
        // because the correlation result itself is still valid — only
        // the emission timestamp is unknown.
        let observed_at_ms = now_ms().unwrap_or_else(|| {
            tracing::warn!(
                target: "m4.cascade.clock",
                "system clock unavailable — observed_at_ms falling back to epoch-0 sentinel"
            );
            0
        });
        let mut out = Vec::with_capacity(clusters.len());
        for group in clusters {
            let pane_labels = collect_pane_labels(&group, dispatch_records);
            if pane_labels.len() < self.config.min_pane_count {
                continue;
            }
            let window_start_ns = group.first().map_or(0, |s| s.ts_ns);
            let window_end_ns = group.last().map_or(0, |s| s.ts_ns);
            let pane_label_refs: Vec<&str> = pane_labels.iter().map(String::as_str).collect();
            let cluster_id = assign_cluster_id(
                window_start_ns,
                window_end_ns,
                &pane_label_refs,
                group.len(),
            );
            let has_temporal_gaps = group
                .windows(2)
                .any(|w| w[1].ts_ns.saturating_sub(w[0].ts_ns) > max_gap_ns / 2);
            let dag_depth = compute_dag_depth(&group, max_gap_ns);
            out.push(CascadeCluster {
                cluster_id,
                window_start_ns,
                window_end_ns,
                pane_count: pane_labels.len(),
                step_count: group.len(),
                has_temporal_gaps,
                dag_depth,
                observed_at_ms,
            });
        }
        out
    }
}

fn collect_pane_labels(group: &[&AtuinStep], dispatch: &[DispatchRecord]) -> Vec<String> {
    use std::collections::BTreeSet;
    let mut set: BTreeSet<String> = BTreeSet::new();
    if let Some(first) = group.first() {
        for d in dispatch {
            // Hardening: `last.ts_ns + slack` could overflow on adversarial
            // i64::MAX timestamps. saturating_add keeps the comparison
            // well-defined on the entire i64 range without changing
            // observable behaviour at habitat scale.
            if d.ts_ns >= first.ts_ns
                && group
                    .last()
                    .is_some_and(|last| d.ts_ns <= last.ts_ns.saturating_add(DISPATCH_TRAILING_SLACK_NS))
            {
                set.insert(d.pane_label.clone());
            }
        }
    }
    // Fall back to session-as-pane-label when dispatch records are absent —
    // each distinct session counts as a participating pane.
    if set.is_empty() {
        for s in group {
            set.insert(s.session.clone());
        }
    }
    set.into_iter().collect()
}

fn compute_dag_depth(group: &[&AtuinStep], max_gap_ns: i64) -> usize {
    // Longest-contiguous-run depth: the temporal sort means the DAG is
    // already topologically linearised. Depth is the longest contiguous
    // subsequence of steps where each consecutive pair respects
    // `max_gap_ns`. (Earlier doc-comment claimed "Kahn-style" which was
    // misleading: there is no in-degree computation. This is a single
    // forward sweep of cost O(n).) Saturating arithmetic on the diff
    // guards against adversarial i64 boundary timestamps.
    let mut depth = 1_usize;
    let mut run = 1_usize;
    for w in group.windows(2) {
        if w[1].ts_ns.saturating_sub(w[0].ts_ns) <= max_gap_ns {
            run = run.saturating_add(1);
            if run > depth {
                depth = run;
            }
        } else {
            run = 1;
        }
    }
    depth
}

/// Wall-clock time in milliseconds since UNIX epoch, or `None` when the
/// system clock is set *before* 1970 (genuine fault) or `as_millis()`
/// overflows `i64` in year ~292,471,209 AD.
///
/// **SF3 silent-failure hardening (S1002388 W2):** prior versions ended
/// the chain with `.unwrap_or(0)`, silently yielding `0` (= 1970-01-01)
/// into `CascadeCluster::observed_at_ms` on a clock fault — a phantom
/// epoch indistinguishable from a real timestamp. The signature is now
/// `Option<i64>`, mirroring the hardened
/// [`crate::m13_stcortex_writer`] `now_ms` and
/// [`crate::m11_fitness_weighted_decay::chrono_now_ms`] contracts. The
/// sole caller ([`CascadeCorrelator::correlate`]) `tracing::warn!`-logs
/// the fault before falling back to the `0` sentinel, so the degraded
/// state is observable rather than silent.
#[must_use]
fn now_ms() -> Option<i64> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(dur.as_millis()).ok()
}

#[cfg(test)]
mod tests {
    use super::{
        AtuinStep, CascadeCorrelator, CascadeCorrelatorConfig, DispatchRecord,
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

    fn corr(min_pane: usize, max_gap_ms: i64) -> CascadeCorrelator {
        CascadeCorrelator::new(CascadeCorrelatorConfig {
            min_pane_count: min_pane,
            max_gap_ms,
            ..CascadeCorrelatorConfig::default()
        })
    }

    #[test]
    fn empty_input_yields_empty_vec() {
        let c = corr(1, 30_000);
        assert!(c.correlate(&[], &[]).is_empty());
    }

    #[test]
    fn single_step_with_min_pane_one_yields_one_cluster() {
        let c = corr(1, 30_000);
        let steps = vec![step("a", 1_000_000_000, "s1")];
        let clusters = c.correlate(&steps, &[]);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].step_count, 1);
    }

    #[test]
    fn single_pane_default_min_pane_two_drops_cluster() {
        // Default min_pane_count = 2; a single-session batch yields one
        // candidate cluster but it's filtered out.
        let c = corr(2, 30_000);
        let steps = vec![step("a", 1, "s1"), step("b", 2, "s1")];
        assert!(c.correlate(&steps, &[]).is_empty());
    }

    #[test]
    fn multi_session_batch_yields_cluster_under_default_min() {
        let c = corr(2, 30_000);
        let steps = vec![
            step("a", 1_000_000_000, "s1"),
            step("b", 1_001_000_000, "s2"),
            step("c", 1_002_000_000, "s3"),
        ];
        let clusters = c.correlate(&steps, &[]);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].pane_count, 3);
        assert_eq!(clusters[0].step_count, 3);
    }

    #[test]
    fn gap_larger_than_max_splits_clusters() {
        let c = corr(1, 1_000);
        let gap_ns: i64 = 2_000 * 1_000_000;
        let steps = vec![
            step("a", 1_000_000_000, "s1"),
            step("b", 1_000_000_000 + gap_ns, "s1"),
        ];
        let clusters = c.correlate(&steps, &[]);
        assert_eq!(clusters.len(), 2, "{gap_ns}ns gap should split");
    }

    #[test]
    fn correlate_is_deterministic_across_repeats() {
        let c = corr(1, 30_000);
        let steps = vec![
            step("a", 1, "s1"),
            step("b", 2, "s2"),
            step("c", 3, "s3"),
        ];
        let first = c.correlate(&steps, &[])[0].cluster_id.clone();
        for _ in 0..20_u32 {
            assert_eq!(c.correlate(&steps, &[])[0].cluster_id, first);
        }
    }

    #[test]
    fn dispatch_records_inform_pane_labels() {
        let c = corr(2, 30_000);
        let steps = vec![step("a", 1_000_000_000, "s1"), step("b", 1_500_000_000, "s1")];
        let d = vec![
            dispatch(1_000_000_000, "ALPHA-LEFT", "s1"),
            dispatch(1_500_000_000, "BETA-LEFT", "s1"),
        ];
        let clusters = c.correlate(&steps, &d);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].pane_count, 2);
    }

    #[test]
    fn cluster_id_does_not_contain_pane_label() {
        let c = corr(2, 30_000);
        let steps = vec![step("a", 1_000_000_000, "s1"), step("b", 1_500_000_000, "s2")];
        let d = vec![
            dispatch(1_000_000_000, "ALPHA-LEFT-PANE", "s1"),
            dispatch(1_500_000_000, "BETA-LEFT-PANE", "s2"),
        ];
        let clusters = c.correlate(&steps, &d);
        let id = clusters[0].cluster_id.as_str();
        assert!(!id.contains("ALPHA"), "F11 leak: {id}");
        assert!(!id.contains("BETA"), "F11 leak: {id}");
        assert!(!id.contains("LEFT"), "F11 leak: {id}");
    }

    #[test]
    fn correlate_sorts_unsorted_input_by_timestamp() {
        let c = corr(2, 30_000);
        let unsorted = vec![
            step("c", 3_000_000_000, "s3"),
            step("a", 1_000_000_000, "s1"),
            step("b", 2_000_000_000, "s2"),
        ];
        let clusters = c.correlate(&unsorted, &[]);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].window_start_ns, 1_000_000_000);
        assert_eq!(clusters[0].window_end_ns, 3_000_000_000);
    }

    #[test]
    fn has_temporal_gaps_set_when_gap_above_half_max() {
        let c = corr(2, 4_000);
        let half_max_plus_ns: i64 = 3_000 * 1_000_000; // > 2000ms half-max
        let steps = vec![
            step("a", 1_000_000_000, "s1"),
            step("b", 1_000_000_000 + half_max_plus_ns, "s2"),
        ];
        let clusters = c.correlate(&steps, &[]);
        assert_eq!(clusters.len(), 1);
        assert!(clusters[0].has_temporal_gaps);
    }

    #[test]
    fn max_steps_per_cluster_caps_growth() {
        let c = CascadeCorrelator::new(CascadeCorrelatorConfig {
            min_pane_count: 1,
            max_gap_ms: 60_000,
            max_steps_per_cluster: 3,
            ..CascadeCorrelatorConfig::default()
        });
        let steps: Vec<AtuinStep> = (0..6_i64).map(|i| step(&format!("s{i}"), i * 1_000_000, "s1")).collect();
        let clusters = c.correlate(&steps, &[]);
        // After 3 steps in the first cluster, the 4th opens a new cluster.
        assert!(clusters.iter().all(|c| c.step_count <= 3));
    }

    #[test]
    fn dag_depth_at_least_one() {
        let c = corr(1, 30_000);
        let steps = vec![step("a", 1, "s1")];
        let clusters = c.correlate(&steps, &[]);
        assert!(clusters[0].dag_depth >= 1);
    }

    #[test]
    fn correlate_debug_format_shows_config() {
        let c = corr(1, 30_000);
        let s = format!("{c:?}");
        assert!(s.contains("CascadeCorrelator"));
    }

    #[test]
    fn default_config_matches_spec_constants() {
        let c = CascadeCorrelatorConfig::default();
        assert_eq!(c.max_gap_ms, 30_000);
        assert_eq!(c.min_pane_count, 2);
        assert_eq!(c.window_ms, 300_000);
        assert_eq!(c.max_steps_per_cluster, 500);
    }

    // ---- Hardening pass: anti-property + adversarial input (10) -----------

    // rationale: Anti-property F11 — the hex-suffix portion of Display MUST
    // NOT contain user-meaningful semantic strings, even when those strings
    // are workflow-relevant terms. (The static prefix `cascade_cluster_`
    // legitimately contains the word "cluster"; we strip it and assert on
    // the hash suffix only.)
    #[test]
    fn cluster_id_display_suffix_does_not_contain_semantic_workflow_terms() {
        let c = corr(2, 30_000);
        let steps = vec![step("a", 1_000_000_000, "s1"), step("b", 1_500_000_000, "s2")];
        let d = vec![
            dispatch(1_000_000_000, "cc-dispatch-pane", "s1"),
            dispatch(1_500_000_000, "git-commit-pane", "s2"),
        ];
        let clusters = c.correlate(&steps, &d);
        let id = clusters[0].cluster_id.as_str();
        let suffix = id
            .strip_prefix("cascade_cluster_")
            .expect("opaque id must start with prefix");
        for forbidden in ["cc-dispatch", "git-commit", "pane", "cluster", "workflow"] {
            assert!(!suffix.contains(forbidden), "F11 leak: {forbidden:?} in suffix {suffix:?}");
        }
    }

    // rationale: Anti-property F11 — id remains hex-only across the full
    // cardinality of cluster names (defence against future label
    // proliferation per spec § 7).
    #[test]
    fn cluster_id_display_is_hex_only_after_prefix() {
        let c = corr(2, 30_000);
        let steps = vec![step("a", 1, "s1"), step("b", 2, "s2")];
        let clusters = c.correlate(&steps, &[]);
        let id = clusters[0].cluster_id.as_str();
        let suffix = id
            .strip_prefix("cascade_cluster_")
            .expect("id must start with prefix");
        assert!(suffix.chars().all(|c| c.is_ascii_hexdigit()), "suffix={suffix}");
        assert_eq!(suffix.len(), 16, "expected 16-hex suffix, got {suffix}");
    }

    // rationale: Boundary — saturating timestamp arithmetic survives
    // adversarial i64::MAX without overflow.
    #[test]
    fn correlate_does_not_panic_on_i64_max_timestamps() {
        let c = corr(2, 30_000);
        let steps = vec![
            step("a", i64::MAX - 10, "s1"),
            step("b", i64::MAX - 5, "s2"),
        ];
        // Must not panic on saturating_add inside collect_pane_labels.
        let _clusters = c.correlate(&steps, &[]);
    }

    // rationale: Boundary — adversarial i64::MIN timestamps do not break
    // sort or windowing.
    #[test]
    fn correlate_does_not_panic_on_i64_min_timestamps() {
        let c = corr(2, 30_000);
        let steps = vec![step("a", i64::MIN, "s1"), step("b", i64::MIN + 1, "s2")];
        let _clusters = c.correlate(&steps, &[]);
    }

    // rationale: Determinism — same input across thread-local environment
    // variation (HOME differences) yields same cluster ids; ids only depend
    // on inputs to `assign_cluster_id`, not on env.
    #[test]
    fn correlate_cluster_ids_independent_of_env() {
        let c = corr(2, 30_000);
        let steps = vec![step("a", 1, "s1"), step("b", 2, "s2")];
        let id_run_1 = c.correlate(&steps, &[])[0].cluster_id.clone();
        let id_run_2 = c.correlate(&steps, &[])[0].cluster_id.clone();
        assert_eq!(id_run_1, id_run_2);
    }

    // rationale: Adversarial input — duplicate timestamps must not crash;
    // stable order via key-only sort means insertion order is preserved.
    #[test]
    fn correlate_handles_duplicate_timestamps_without_panic() {
        let c = corr(2, 30_000);
        let steps = vec![
            step("a", 1_000_000_000, "s1"),
            step("b", 1_000_000_000, "s2"),
            step("c", 1_000_000_000, "s3"),
        ];
        let clusters = c.correlate(&steps, &[]);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].step_count, 3);
    }

    // rationale: Cross-module surface invariant — dispatch slack constant
    // is exposed and == 60s in ns (matches m4 spec § 3 trailing slack).
    #[test]
    fn dispatch_trailing_slack_constant_is_60s_in_ns() {
        assert_eq!(super::DISPATCH_TRAILING_SLACK_NS, 60_000_000_000);
    }

    // rationale: Anti-property F11 — id space sanity: even at the
    // saturating-arithmetic boundary, the id is still well-formed
    // (prefix + 16 hex).
    #[test]
    fn cluster_id_remains_well_formed_at_saturating_boundary() {
        let c = corr(2, 30_000);
        let steps = vec![
            step("a", i64::MAX - 1, "s1"),
            step("b", i64::MAX, "s2"),
        ];
        let clusters = c.correlate(&steps, &[]);
        if !clusters.is_empty() {
            let id = clusters[0].cluster_id.as_str();
            assert!(id.starts_with("cascade_cluster_"));
            let suffix = &id["cascade_cluster_".len()..];
            assert_eq!(suffix.len(), 16);
        }
    }

    // rationale: Resource accounting — max_steps_per_cluster=1 hard-caps
    // cluster growth at 1 (every step opens a new cluster).
    #[test]
    fn max_steps_per_cluster_one_yields_singleton_clusters() {
        let c = CascadeCorrelator::new(CascadeCorrelatorConfig {
            min_pane_count: 1,
            max_gap_ms: 60_000,
            max_steps_per_cluster: 1,
            ..CascadeCorrelatorConfig::default()
        });
        let steps: Vec<AtuinStep> = (0..5_i64)
            .map(|i| step(&format!("s{i}"), i * 1_000_000, "s1"))
            .collect();
        let clusters = c.correlate(&steps, &[]);
        assert!(clusters.iter().all(|c| c.step_count <= 1));
    }

    // rationale: Contract regression — config() returns a borrow that
    // matches the constructor input verbatim (no field-mangling).
    #[test]
    fn config_round_trip_preserves_fields() {
        let cfg = CascadeCorrelatorConfig {
            max_gap_ms: 12_345,
            min_pane_count: 7,
            window_ms: 99_999,
            max_steps_per_cluster: 42,
            ..CascadeCorrelatorConfig::default()
        };
        let c = CascadeCorrelator::new(cfg.clone());
        let got = c.config();
        assert_eq!(got.max_gap_ms, cfg.max_gap_ms);
        assert_eq!(got.min_pane_count, cfg.min_pane_count);
        assert_eq!(got.window_ms, cfg.window_ms);
        assert_eq!(got.max_steps_per_cluster, cfg.max_steps_per_cluster);
    }

    // ====================================================================
    // Hardening pass 2 — pane-label fallback, DAG depth, window range,
    // min_pane_count gating, and dispatch-slack windowing.
    // ====================================================================

    // rationale: Core correctness — when dispatch records are absent, the
    // correlator falls back to distinct sessions as pane labels. Three
    // distinct sessions → pane_count 3.
    #[test]
    fn pane_labels_fall_back_to_distinct_sessions_when_no_dispatch() {
        let c = corr(2, 30_000);
        let steps = vec![
            step("a", 1_000_000_000, "alpha"),
            step("b", 1_001_000_000, "beta"),
            step("c", 1_002_000_000, "alpha"),
        ];
        let clusters = c.correlate(&steps, &[]);
        assert_eq!(clusters.len(), 1);
        // alpha + beta = 2 distinct sessions (alpha is deduplicated).
        assert_eq!(clusters[0].pane_count, 2, "session fallback must dedupe");
    }

    // rationale: Core correctness — when every step shares one session and
    // there are no dispatch records, the fallback yields exactly one pane;
    // with default min_pane_count=2 the cluster is dropped.
    #[test]
    fn single_session_fallback_yields_one_pane_and_drops_under_default_min() {
        let c = corr(2, 30_000);
        let steps = vec![
            step("a", 1_000_000_000, "solo"),
            step("b", 1_001_000_000, "solo"),
            step("c", 1_002_000_000, "solo"),
        ];
        assert!(
            c.correlate(&steps, &[]).is_empty(),
            "one-pane cluster must be filtered by min_pane_count=2"
        );
    }

    // rationale: Boundary — a dispatch record OUTSIDE the cluster window
    // (before the first step) does NOT contribute a pane label; the
    // session fallback takes over instead.
    #[test]
    fn dispatch_record_before_window_start_is_excluded() {
        let c = corr(1, 30_000);
        let steps = vec![step("a", 5_000_000_000, "s1")];
        // Dispatch at ts BEFORE the first step → must be excluded.
        let d = vec![dispatch(1_000_000_000, "STALE-PANE", "s1")];
        let clusters = c.correlate(&steps, &d);
        assert_eq!(clusters.len(), 1);
        // No in-window dispatch → fallback to the single session → 1 pane.
        assert_eq!(clusters[0].pane_count, 1);
    }

    // rationale: Boundary — a dispatch record within the 60s trailing
    // slack AFTER the last step still counts (m4 spec § 3 windowing).
    #[test]
    fn dispatch_record_within_trailing_slack_is_included() {
        let c = corr(2, 30_000);
        let steps = vec![step("a", 1_000_000_000, "s1"), step("b", 1_500_000_000, "s1")];
        // Second dispatch is 30s after the last step — inside the 60s slack.
        let d = vec![
            dispatch(1_000_000_000, "ALPHA", "s1"),
            dispatch(1_500_000_000 + 30_000_000_000, "BETA", "s1"),
        ];
        let clusters = c.correlate(&steps, &d);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].pane_count, 2, "trailing-slack dispatch must count");
    }

    // rationale: Boundary — a dispatch record BEYOND the 60s trailing
    // slack is excluded.
    #[test]
    fn dispatch_record_beyond_trailing_slack_is_excluded() {
        let c = corr(1, 30_000);
        let steps = vec![step("a", 1_000_000_000, "s1")];
        // Dispatch 61s after the step — beyond the 60s slack.
        let d = vec![dispatch(1_000_000_000 + 61_000_000_000, "LATE", "s1")];
        let clusters = c.correlate(&steps, &d);
        assert_eq!(clusters.len(), 1);
        // Out-of-slack dispatch ignored → session fallback → 1 pane.
        assert_eq!(clusters[0].pane_count, 1);
    }

    // rationale: Core correctness — dag_depth equals the run length for a
    // tightly-spaced contiguous group (all gaps within max_gap_ns).
    #[test]
    fn dag_depth_equals_step_count_for_contiguous_group() {
        let c = corr(1, 60_000);
        let steps: Vec<AtuinStep> = (0..5_i64)
            .map(|i| step(&format!("s{i}"), 1_000_000_000 + i * 1_000_000, "s1"))
            .collect();
        let clusters = c.correlate(&steps, &[]);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].step_count, 5);
        assert_eq!(clusters[0].dag_depth, 5, "tight contiguous run → depth==count");
    }

    // rationale: Core correctness — when one inter-step gap inside a
    // cluster exceeds max_gap_ns, the DAG depth resets at that gap (the
    // longest contiguous run is shorter than the step count).
    #[test]
    fn dag_depth_resets_on_internal_gap_exceeding_max() {
        // max_gap_ms 60_000 keeps all steps in ONE cluster (the cluster
        // split threshold), but a gap > max_gap_ns inside breaks the run.
        let c = corr(1, 60_000);
        let gap = 70_000_000_000_i64; // 70s > 60s max_gap_ns
        let steps = vec![
            step("a", 1_000_000_000, "s1"),
            step("b", 1_001_000_000, "s1"),
            // Big gap here — run resets.
            step("c", 1_001_000_000 + gap + 1_000_000, "s1"),
        ];
        let clusters = c.correlate(&steps, &[]);
        // The 70s gap exceeds max_gap_ns so the cluster itself splits.
        assert_eq!(clusters.len(), 2, "internal gap > max splits the cluster");
    }

    // rationale: Core correctness — window_start/window_end reflect the
    // min/max timestamps of the cluster's steps after sorting.
    #[test]
    fn window_bounds_track_min_and_max_step_timestamps() {
        let c = corr(2, 60_000);
        let steps = vec![
            step("mid", 2_000_000_000, "s1"),
            step("first", 1_000_000_000, "s2"),
            step("last", 3_000_000_000, "s3"),
        ];
        let clusters = c.correlate(&steps, &[]);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].window_start_ns, 1_000_000_000);
        assert_eq!(clusters[0].window_end_ns, 3_000_000_000);
    }

    // rationale: Resource accounting — min_pane_count higher than the
    // achievable pane count drops every cluster.
    #[test]
    fn min_pane_count_above_achievable_drops_all_clusters() {
        let c = corr(99, 30_000);
        let steps = vec![
            step("a", 1_000_000_000, "s1"),
            step("b", 1_001_000_000, "s2"),
        ];
        assert!(
            c.correlate(&steps, &[]).is_empty(),
            "min_pane_count=99 unreachable → no clusters"
        );
    }

    // rationale: Anti-property F11 — observed_at_ms is a wall-clock field;
    // it MUST be populated (non-zero on a real clock) and identical across
    // every cluster emitted in one correlate() call (single now_ms read).
    #[test]
    fn observed_at_ms_is_uniform_across_clusters_in_one_call() {
        let c = corr(1, 1_000);
        let split_gap = 5_000_000_000_i64; // 5s > 1s max_gap → splits
        let steps = vec![
            step("a", 1_000_000_000, "s1"),
            step("b", 1_000_000_000 + split_gap, "s1"),
            step("c", 1_000_000_000 + 2 * split_gap, "s1"),
        ];
        let clusters = c.correlate(&steps, &[]);
        assert_eq!(clusters.len(), 3, "three split clusters expected");
        let first_ts = clusters[0].observed_at_ms;
        assert!(clusters.iter().all(|cl| cl.observed_at_ms == first_ts));
    }

    // ====================================================================
    // SF3 silent-failure hardening (S1002388 W2) — +2 regression tests.
    // `now_ms` previously ended in `.unwrap_or(0)`, silently emitting a
    // 1970-epoch `observed_at_ms` on a clock fault. The signature is now
    // `Option<i64>` and the sole caller logs the fault before the
    // sentinel. These tests pin the new contract.
    // ====================================================================

    // rationale: SF3 silent-failure — now_ms returns Option<i64>, NOT a
    // bare i64 that silently collapses a clock fault to 0. On a healthy
    // production clock it returns Some(realistic 2024+ wall-clock ms).
    #[test]
    fn now_ms_signature_is_option_i64_and_returns_some_on_real_clock() {
        // rationale: SF3 silent-failure
        // Pin the signature: a clock fault would surface as None, never 0.
        let ensure_option: fn() -> Option<i64> = super::now_ms;
        let ts = ensure_option().expect("production clock must be post-1970");
        assert!(
            ts > 1_700_000_000_000,
            "now_ms must return realistic 2024+ wall-clock ms, got {ts}"
        );
    }

    // rationale: SF3 silent-failure — on a healthy clock, the
    // observed_at_ms field threaded through correlate() carries the real
    // (non-sentinel) timestamp. The epoch-0 sentinel is reachable ONLY
    // via the logged clock-fault fallback, never silently.
    #[test]
    fn observed_at_ms_is_real_timestamp_not_epoch_sentinel_on_healthy_clock() {
        // rationale: SF3 silent-failure
        let c = corr(1, 30_000);
        let steps = vec![step("a", 1_000_000_000, "s1")];
        let clusters = c.correlate(&steps, &[]);
        assert_eq!(clusters.len(), 1);
        assert!(
            clusters[0].observed_at_ms > 1_700_000_000_000,
            "observed_at_ms must be a real wall-clock ms on a healthy \
             clock, never the epoch-0 sentinel: {}",
            clusters[0].observed_at_ms
        );
    }
}
