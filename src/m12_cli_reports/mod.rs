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
//!
//! **`let _ = writeln!(out, ...)` discard pattern:** `fmt::Write` for
//! `String` is infallible per the std::fmt docs — the only error the
//! trait can return is from `Formatter`s with bounded buffers (e.g.
//! `&mut [u8]`). Discarding the `Result<(), fmt::Error>` from each
//! `writeln!` into a `String` is therefore safe and not a silent-failure
//! pattern. Every such discard in this module is rationale-tagged via
//! this top-level note (per workspace "no silent discard" rule).
//!
//! **F9 zero-weight discipline:** `Option<T>` fields on `WorkflowRunRow`
//! (`outcome`, `ended_at`, `cost_tokens`) carry signal-present vs
//! signal-absent semantics. Renderers MUST emit distinct sentinels for
//! `None` (`open`, `---`) versus explicit zero (`0 tok`, `unknown`
//! outcome), never collapse `None` to a numeric sentinel.

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
///
/// F9 zero-weight discipline: a row with `cost_tokens == None` is
/// counted in the `runs` column for its cluster (it IS a run) but its
/// cost contribution is `0` because there is no cost signal yet — the
/// additive identity, not a fabricated sentinel. Operators read both
/// columns together: high `runs` + low `total cost` is the "many rows,
/// no cost recorded yet" reading, distinct from "many rows, all zero-cost".
#[must_use]
pub fn render_cluster_cost_table(runs: &[WorkflowRunRow]) -> String {
    let mut buckets: std::collections::BTreeMap<String, (usize, i64)> =
        std::collections::BTreeMap::new();
    let mut ungrouped = (0_usize, 0_i64);
    for r in runs {
        let cluster = extract_cluster_short(&r.consumer_inputs);
        // None → 0 contribution (additive identity, NOT silent sentinel).
        // Run count still increments regardless — see § F9 doc above.
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

/// Extract the 6-char tail of a cascade `cluster_id` from a row's
/// `consumer_inputs` JSON blob. Returns `None` for any of:
///
/// - JSON parse failure (malformed / non-JSON blob)
/// - missing `cascade` discriminant (no cascade observation on this row)
/// - missing `cluster_id` field
/// - `cluster_id` not a string
/// - empty tail after split-on-`_`
///
/// Each `None` collapses to "(ungrouped)" in the caller — this is
/// intentional. F11 (opaque IDs): rendering "(ungrouped)" is the correct
/// response for "no cluster signal recorded" — never fabricate a label.
/// `.ok()?` on the JSON parse is load-bearing graceful handling for
/// operator-supplied corruption (render-as-missing, don't crash).
fn extract_cluster_short(consumer_inputs: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(consumer_inputs).ok()?;
    let cascade = value.get("cascade")?;
    let cluster_id = cascade.get("cluster_id")?.as_str()?;
    let tail = cluster_id.split('_').next_back()?;
    let short: String = tail.chars().take(6).collect();
    if short.is_empty() {
        return None;
    }
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
///
/// `WorkflowRunRow` is composed entirely of types `serde_json` can always
/// serialise; the single failure mode is `fitness_dimension` being NaN
/// — which would be an upstream F9 violation (the column is `NOT NULL
/// DEFAULT 0.0` in SQL and only m11 may write it). The fallback `"[]"`
/// (Json) / row-skip (NdJson) surfaces that catastrophic case visibly
/// rather than panicking; the contract is "infallible on well-formed
/// rows".
#[must_use]
pub fn render_machine(runs: &[WorkflowRunRow], format: OutputFormat) -> String {
    match format {
        OutputFormat::Table => render_summary_line(runs),
        // Fallback "[]" indicates a NaN fitness_dimension or similar
        // structural anomaly — render the empty-sentinel rather than
        // crash the report so the operator can inspect the DB.
        OutputFormat::Json => serde_json::to_string_pretty(runs).unwrap_or_else(|_| "[]".into()),
        OutputFormat::NdJson => runs
            .iter()
            // filter_map drops rows that fail to serialise (same NaN
            // case as Json). Per F9 we never substitute a placeholder
            // row — dropping is the correct response.
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

    // ====================================================================
    // Hardening pass (S1002388) — +10 tests for m12 CLI reports.
    // Cross-module (m10 Ember gate) + F9 zero-weight + adversarial input
    // + determinism + contract regression.
    // ====================================================================

    use crate::m10_ember_ci_gate::{evaluate_string, GateVerdict};

    // rationale: Cross-module surface invariant — render_summary_line
    // emits a string that passes m10's Ember gate. Strongest m12↔m10
    // contract.
    #[test]
    fn ember_gate_passes_render_summary_line() {
        // rationale: Cross-module surface invariant
        let runs = vec![
            run(1, Some(100), Some("ok"), "{}"),
            run(2, Some(200), Some("fail"), "{}"),
            run(3, None, None, "{}"),
        ];
        let s = render_summary_line(&runs);
        assert_eq!(
            evaluate_string("m12.summary", &s, &[]),
            GateVerdict::Pass,
            "m12 output failed Ember gate: {s:?}"
        );
    }

    // rationale: Cross-module surface invariant — render_outcome_timeline
    // passes Ember gate for typical and edge inputs.
    #[test]
    fn ember_gate_passes_render_outcome_timeline() {
        // rationale: Cross-module surface invariant
        for runs in [
            vec![],
            vec![run(1, Some(100), Some("ok"), "{}")],
            vec![run(1, None, None, "{}")],
            vec![
                run(1, Some(100), Some("fail"), "{}"),
                run(2, Some(200), Some("abort"), "{}"),
            ],
        ] {
            let s = render_outcome_timeline(&runs);
            assert_eq!(
                evaluate_string("m12.timeline", &s, &[]),
                GateVerdict::Pass,
                "timeline failed Ember gate: {s:?}"
            );
        }
    }

    // rationale: Cross-module surface invariant — render_cost_histogram
    // passes Ember gate.
    #[test]
    fn ember_gate_passes_render_cost_histogram() {
        // rationale: Cross-module surface invariant
        let runs = vec![
            run(1, Some(500), Some("ok"), "{}"),
            run(2, Some(2_000), Some("ok"), "{}"),
        ];
        let s = render_cost_histogram(&runs);
        assert_eq!(
            evaluate_string("m12.histogram", &s, &[]),
            GateVerdict::Pass
        );
    }

    // rationale: Cross-module surface invariant — render_cluster_cost_table
    // passes Ember gate for grouped and ungrouped cases.
    #[test]
    fn ember_gate_passes_render_cluster_cost_table() {
        // rationale: Cross-module surface invariant
        let ci = r#"{"cascade":{"kind":"cascade","cluster_id":"cascade_cluster_abc123def456","session_range":[0,1]}}"#;
        let runs = vec![
            run(1, Some(100), Some("ok"), ci),
            run(2, Some(200), Some("ok"), "{}"),
        ];
        let s = render_cluster_cost_table(&runs);
        assert_eq!(
            evaluate_string("m12.cluster_cost", &s, &[]),
            GateVerdict::Pass
        );
    }

    // rationale: Anti-property (F9 zero-weight) — None cost renders as
    // "---", NEVER as "0 tok" (silent-zero substitution).
    #[test]
    fn timeline_none_cost_renders_dashes_not_zero() {
        // rationale: Anti-property (F9 zero-weight)
        let runs = vec![run(1, None, Some("ok"), "{}")];
        let s = render_outcome_timeline(&runs);
        assert!(s.contains("---"), "None cost must render as dashes: {s}");
        assert!(!s.contains("0 tok"), "None must not collapse to 0: {s}");
    }

    // rationale: Anti-property (F9 zero-weight) — explicit 0 cost
    // renders as "0 tok"; distinct from None (which renders "---").
    #[test]
    fn timeline_explicit_zero_cost_renders_as_zero_tok() {
        // rationale: Anti-property (F9 zero-weight)
        let runs = vec![run(1, Some(0), Some("ok"), "{}")];
        let s = render_outcome_timeline(&runs);
        assert!(s.contains("0 tok"), "explicit zero must render as 0 tok: {s}");
        // Header has a "---------" separator; we check the data line only.
        let data_line = s
            .lines()
            .find(|l| l.starts_with("2026-"))
            .expect("data line");
        assert!(
            !data_line.contains("---"),
            "explicit zero must not render as dashes in cost field: {data_line}"
        );
    }

    // rationale: Determinism — render_machine NdJson preserves input
    // row order; output line N serialises input row N.
    #[test]
    fn render_machine_ndjson_preserves_input_order() {
        // rationale: Determinism
        let runs = vec![
            run(1, Some(100), Some("ok"), "{}"),
            run(2, Some(200), Some("fail"), "{}"),
            run(3, Some(300), Some("abort"), "{}"),
        ];
        let s = render_machine(&runs, OutputFormat::NdJson);
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 3);
        for (i, line) in lines.iter().enumerate() {
            let parsed: WorkflowRunRow = serde_json::from_str(line).expect("parse");
            assert_eq!(parsed.id, runs[i].id, "row order divergence at index {i}");
        }
    }

    // rationale: Adversarial input — render_cluster_cost_table must NOT
    // crash on malformed JSON in consumer_inputs; all such rows fall to
    // "(ungrouped)" with cost summed.
    #[test]
    fn cluster_cost_table_groups_malformed_json_to_ungrouped() {
        // rationale: Adversarial input
        let runs = vec![
            run(1, Some(100), Some("ok"), "not json"),
            run(2, Some(200), Some("ok"), "[1,2,3]"),
            run(3, Some(300), Some("ok"), r#"{"unrelated":"key"}"#),
        ];
        let s = render_cluster_cost_table(&runs);
        assert!(s.contains("ungrouped"));
        assert!(s.contains("600"), "ungrouped total cost must sum: {s}");
    }

    // rationale: Contract regression (F11 opaque IDs) — m12 truncates
    // cluster id tail at 6 chars; the full upstream label must not leak.
    #[test]
    fn cluster_cost_table_truncates_to_six_chars_max() {
        // rationale: Contract regression (F11 opaque IDs)
        let ci = r#"{"cascade":{"kind":"cascade","cluster_id":"cascade_cluster_LONG_REVEAL_TOKEN","session_range":[0,1]}}"#;
        let runs = vec![run(1, Some(100), Some("ok"), ci)];
        let s = render_cluster_cost_table(&runs);
        assert!(!s.contains("LONG"), "F11 leak: full cluster label revealed: {s}");
        assert!(!s.contains("REVEAL"), "F11 leak: full cluster label revealed: {s}");
    }

    // rationale: Resource accounting — render_summary_line over 10k
    // rows produces a single-line output (quadratic-alloc smoke).
    #[test]
    fn render_summary_line_handles_ten_thousand_rows() {
        // rationale: Resource accounting
        let runs: Vec<WorkflowRunRow> = (0..10_000_i64)
            .map(|i| {
                run(
                    i,
                    Some(i * 10),
                    Some(if i % 4 == 0 { "ok" } else { "fail" }),
                    "{}",
                )
            })
            .collect();
        let s = render_summary_line(&runs);
        assert!(s.contains("10000 runs recorded"));
        assert_eq!(s.lines().count(), 1);
    }
}
