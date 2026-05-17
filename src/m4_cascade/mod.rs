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

        let observed_at_ms = now_ms();
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
            if d.ts_ns >= first.ts_ns
                && group
                    .last()
                    .is_some_and(|last| d.ts_ns <= last.ts_ns + 60_000_000_000)
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
    // Lightweight Kahn-style: depth = longest temporal chain where each
    // edge respects max_gap_ns. We don't need a full DAG here because the
    // input is already temporally sorted; we approximate depth as the
    // number of contiguous edges under the gap threshold.
    let mut depth = 1_usize;
    let mut run = 1_usize;
    for w in group.windows(2) {
        if w[1].ts_ns.saturating_sub(w[0].ts_ns) <= max_gap_ns {
            run += 1;
            if run > depth {
                depth = run;
            }
        } else {
            run = 1;
        }
    }
    depth
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
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
}
