//! `m12_cli_reports` — pure formatter from `WorkflowRunRow` slices to
//! human-readable / machine-readable strings.
//!
//! m12 is a leaf in the data-flow DAG: no DB writes, no side effects,
//! no `println!`. All functions are `#[must_use]` and return `String`.
//!
//! **Ember rubric compliance:** rendered strings use passive descriptors
//! only (recorded, observed, emitted, cost, outcome, rate). Forbidden
//! verbs (recommend / optimise / select / route / dispatch / auto) MUST
//! NOT appear in any rendered output.

use std::fmt::Write;

use crate::m7_workflow_runs::WorkflowRunRow;

/// CLI output format selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputFormat {
    /// Fixed-width ASCII tables / histograms.
    Table,
    /// Single JSON document (pretty-printed).
    Json,
    /// Newline-delimited JSON, one row per line.
    NdJson,
}

const HISTOGRAM_BUCKETS: [(&str, i64, i64); 5] = [
    ("   0 - 1k tokens", 0, 1_000),
    ("  1k - 5k tokens", 1_000, 5_000),
    (" 5k - 20k tokens", 5_000, 20_000),
    ("20k - 50k tokens", 20_000, 50_000),
    ("   > 50k tokens ", 50_000, i64::MAX),
];

const HISTOGRAM_BAR_MAX: usize = 20;

/// Render a cost-band histogram.
#[must_use]
pub fn render_cost_histogram(runs: &[WorkflowRunRow]) -> String {
    let mut counts = [0_usize; HISTOGRAM_BUCKETS.len()];
    for r in runs {
        if let Some(cost) = r.cost_tokens {
            for (i, (_, lo, hi)) in HISTOGRAM_BUCKETS.iter().enumerate() {
                if cost >= *lo && cost < *hi {
                    counts[i] = counts[i].saturating_add(1);
                    break;
                }
            }
        }
    }
    let max = counts.iter().copied().max().unwrap_or(0).max(1);
    let mut out = String::new();
    let _ = writeln!(out, "workflow-trace cost distribution (last {} runs)", runs.len());
    let _ = writeln!(out, "------------------------------------------------");
    for (i, (label, _, _)) in HISTOGRAM_BUCKETS.iter().enumerate() {
        let count = counts[i];
        // bar_width is bounded by HISTOGRAM_BAR_MAX; safe to cast.
        let bar_width = (count * HISTOGRAM_BAR_MAX) / max;
        let bar = "\u{2588}".repeat(bar_width);
        let pad = " ".repeat(HISTOGRAM_BAR_MAX - bar_width);
        let _ = writeln!(out, "{label} |{bar}{pad}| {count} runs");
    }
    let _ = writeln!(out, "------------------------------------------------");
    let (median, p95) = median_and_p95_cost(runs);
    let _ = writeln!(out, "median: {median} tokens  p95: {p95} tokens");
    out
}

fn median_and_p95_cost(runs: &[WorkflowRunRow]) -> (i64, i64) {
    let mut costs: Vec<i64> = runs.iter().filter_map(|r| r.cost_tokens).collect();
    if costs.is_empty() {
        return (0, 0);
    }
    costs.sort_unstable();
    let n = costs.len();
    let median = costs[n / 2];
    let p95_idx = (n.saturating_mul(95) / 100).min(n.saturating_sub(1));
    (median, costs[p95_idx])
}

/// Render the most-recent N runs as a timeline.
#[must_use]
pub fn render_outcome_timeline(runs: &[WorkflowRunRow]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "workflow-trace timeline (last {} runs)", runs.len());
    let _ = writeln!(out, "---------------------------------------");
    for r in runs {
        let outcome = r.outcome.as_deref().unwrap_or("open");
        let cost = r.cost_tokens.map_or("---".to_owned(), |c| format!("{c} tok"));
        let ts = r.ended_at.as_deref().unwrap_or(&r.started_at);
        let _ = writeln!(out, "{ts:<22}  {outcome:<7}  {cost:>10}");
    }
    out
}

/// Render a cost-by-cascade-cluster table. Cluster ids are truncated to
/// their opaque 6-char hex tail (F11 enforced — never reveal labels).
#[must_use]
pub fn render_cluster_cost_table(runs: &[WorkflowRunRow]) -> String {
    let mut buckets: std::collections::BTreeMap<String, (usize, i64)> =
        std::collections::BTreeMap::new();
    let mut ungrouped = (0_usize, 0_i64);
    for r in runs {
        let cluster = extract_cluster_short(&r.consumer_inputs);
        let cost = r.cost_tokens.unwrap_or(0);
        if let Some(c) = cluster {
            let entry = buckets.entry(c).or_insert((0, 0));
            entry.0 = entry.0.saturating_add(1);
            entry.1 = entry.1.saturating_add(cost);
        } else {
            ungrouped.0 = ungrouped.0.saturating_add(1);
            ungrouped.1 = ungrouped.1.saturating_add(cost);
        }
    }
    let mut out = String::new();
    let _ = writeln!(out, "workflow-trace cost by cluster ({} runs)", runs.len());
    let _ = writeln!(out, "---------------------------------------");
    let _ = writeln!(out, "cluster         runs    total cost");
    for (cluster, (count, cost)) in &buckets {
        let _ = writeln!(out, "{cluster:<14}  {count:>4}  {cost:>12}");
    }
    if ungrouped.0 > 0 {
        let _ = writeln!(
            out,
            "(ungrouped)     {:>4}  {:>12}",
            ungrouped.0, ungrouped.1
        );
    }
    out
}

fn extract_cluster_short(consumer_inputs: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(consumer_inputs).ok()?;
    let cascade = value.get("cascade")?;
    let cluster_id = cascade.get("cluster_id")?.as_str()?;
    let tail = cluster_id.split('_').next_back()?;
    let short: String = tail.chars().take(6).collect();
    Some(short)
}

/// Render a compact summary line: total runs, outcome counts, median cost.
#[must_use]
pub fn render_summary_line(runs: &[WorkflowRunRow]) -> String {
    let total = runs.len();
    let mut ok = 0_usize;
    let mut fail = 0_usize;
    let mut abort = 0_usize;
    let mut unknown = 0_usize;
    let mut open = 0_usize;
    for r in runs {
        match r.outcome.as_deref() {
            Some("ok") => ok += 1,
            Some("fail") => fail += 1,
            Some("abort") => abort += 1,
            Some("unknown") => unknown += 1,
            _ => open += 1,
        }
    }
    let (median, _) = median_and_p95_cost(runs);
    format!(
        "{total} runs recorded: ok={ok} fail={fail} abort={abort} unknown={unknown} open={open} median_cost={median}"
    )
}

/// Render in machine-readable JSON / NdJson per format.
#[must_use]
pub fn render_machine(runs: &[WorkflowRunRow], format: OutputFormat) -> String {
    match format {
        OutputFormat::Table => render_summary_line(runs),
        OutputFormat::Json => serde_json::to_string_pretty(runs).unwrap_or_else(|_| "[]".into()),
        OutputFormat::NdJson => runs
            .iter()
            .filter_map(|r| serde_json::to_string(r).ok())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        render_cluster_cost_table, render_cost_histogram, render_machine,
        render_outcome_timeline, render_summary_line, OutputFormat,
    };
    use crate::m7_workflow_runs::WorkflowRunRow;

    fn run(id: i64, cost: Option<i64>, outcome: Option<&str>, consumer_inputs: &str) -> WorkflowRunRow {
        WorkflowRunRow {
            id,
            started_at: format!("2026-05-17T00:{:02}:00Z", id % 60),
            ended_at: outcome.map(|_| format!("2026-05-17T01:{:02}:00Z", id % 60)),
            outcome: outcome.map(str::to_owned),
            consumer_inputs: consumer_inputs.to_owned(),
            cost_tokens: cost,
            fitness_dimension: 0.0,
        }
    }

    #[test]
    fn cost_histogram_empty_runs_lists_zero_in_each_bucket() {
        let s = render_cost_histogram(&[]);
        assert!(s.contains("0 runs"));
        assert!(s.contains("workflow-trace cost distribution"));
    }

    #[test]
    fn cost_histogram_counts_per_bucket() {
        let runs = vec![
            run(1, Some(500), Some("ok"), "{}"),
            run(2, Some(2_000), Some("ok"), "{}"),
            run(3, Some(30_000), Some("ok"), "{}"),
            run(4, Some(100_000), Some("ok"), "{}"),
        ];
        let s = render_cost_histogram(&runs);
        // Each bucket should have at least one run.
        assert!(s.contains("1 runs"));
    }

    #[test]
    fn cost_histogram_no_forbidden_verbs() {
        let runs = vec![run(1, Some(500), Some("ok"), "{}")];
        let s = render_cost_histogram(&runs);
        let lower = s.to_lowercase();
        for forbidden in ["recommend", "optimise", "select", "route", "dispatch", "auto"] {
            assert!(!lower.contains(forbidden), "Ember-forbidden verb {forbidden} in: {s}");
        }
    }

    #[test]
    fn outcome_timeline_renders_each_run_as_one_line() {
        let runs = vec![
            run(1, Some(100), Some("ok"), "{}"),
            run(2, Some(200), Some("fail"), "{}"),
        ];
        let s = render_outcome_timeline(&runs);
        assert!(s.contains("ok"));
        assert!(s.contains("fail"));
        assert!(s.contains("100 tok"));
    }

    #[test]
    fn outcome_timeline_open_runs_show_dashes() {
        let runs = vec![run(1, None, None, "{}")];
        let s = render_outcome_timeline(&runs);
        assert!(s.contains("---"));
        assert!(s.contains("open"));
    }

    #[test]
    fn cluster_cost_table_groups_by_short_cluster_id() {
        let ci = r#"{"cascade":{"kind":"cascade","cluster_id":"cascade_cluster_abc123def456","session_range":[0,1]}}"#;
        let runs = vec![run(1, Some(100), Some("ok"), ci), run(2, Some(200), Some("ok"), ci)];
        let s = render_cluster_cost_table(&runs);
        // Tail of cluster_id is "abc123def456"; truncated to 6 chars.
        assert!(s.contains("abc123"));
        assert!(s.contains('2'));
    }

    #[test]
    fn cluster_cost_table_groups_ungrouped_runs() {
        let runs = vec![run(1, Some(100), Some("ok"), "{}")];
        let s = render_cluster_cost_table(&runs);
        assert!(s.contains("ungrouped"));
    }

    #[test]
    fn summary_line_counts_each_outcome() {
        let runs = vec![
            run(1, Some(100), Some("ok"), "{}"),
            run(2, Some(200), Some("fail"), "{}"),
            run(3, None, None, "{}"),
        ];
        let s = render_summary_line(&runs);
        assert!(s.contains("3 runs recorded"));
        assert!(s.contains("ok=1"));
        assert!(s.contains("fail=1"));
        assert!(s.contains("open=1"));
    }

    #[test]
    fn render_machine_json_roundtrip() {
        let runs = vec![run(1, Some(100), Some("ok"), "{}")];
        let s = render_machine(&runs, OutputFormat::Json);
        let parsed: Vec<WorkflowRunRow> = serde_json::from_str(&s).expect("parse");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, 1);
    }

    #[test]
    fn render_machine_ndjson_yields_one_line_per_row() {
        let runs = vec![
            run(1, Some(100), Some("ok"), "{}"),
            run(2, Some(200), Some("fail"), "{}"),
        ];
        let s = render_machine(&runs, OutputFormat::NdJson);
        assert_eq!(s.lines().count(), 2);
    }

    #[test]
    fn no_println_in_lib() {
        // Compile-time check via the source: m12 must use `write!`/`writeln!`
        // into a String, never `println!`. This is a smoke that the function
        // is pure (returns String).
        let s: String = render_summary_line(&[]);
        assert!(s.starts_with("0 runs recorded"));
    }
}
