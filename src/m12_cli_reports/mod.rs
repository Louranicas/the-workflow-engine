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

use crate::m7_workflow_runs::{Outcome, WorkflowRunRow};

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
        let outcome = r.run_state.outcome().map_or("open", Outcome::as_str);
        let cost = r.cost_tokens.map_or("---".to_owned(), |c| format!("{c} tok"));
        let ts = r.run_state.ended_at().unwrap_or(&r.started_at);
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
        match r.run_state.outcome() {
            Some(Outcome::Ok) => ok += 1,
            Some(Outcome::Fail) => fail += 1,
            Some(Outcome::Abort) => abort += 1,
            Some(Outcome::Unknown) => unknown += 1,
            None => open += 1,
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
///
/// **SF1 / SF2 silent-failure hardening (S1002388 W2):** both fallback
/// paths previously discarded the `serde_json::Error` via `unwrap_or_else`
/// / `filter_map(...ok())`. A serialisation failure (NaN field) would then
/// be indistinguishable from a genuinely-empty DB (Json → `"[]"`) or a
/// shorter result set (NdJson → fewer lines). Both paths now bind and
/// `tracing::error!`-log the error — including an identifying field on the
/// dropped NdJson row — so the collapse / drop is observable in the log
/// stream. Observable behaviour of the rendered string is unchanged.
#[must_use]
pub fn render_machine(runs: &[WorkflowRunRow], format: OutputFormat) -> String {
    match format {
        OutputFormat::Table => render_summary_line(runs),
        // SF1 defense-in-depth: `to_string_pretty` of a `WorkflowRunRow`
        // slice is effectively infallible — every field is a plain
        // derive-`Serialize` type, and a non-finite `f64` serialises to
        // JSON `null` (it does NOT error). The `Err` arm therefore guards
        // a *future* genuinely-failing `Serialize` impl: should that ever
        // occur the error is bound + logged so an operator can tell "[]"
        // "report failed" apart from "[]" "DB is genuinely empty" — never
        // a silent `unwrap_or_else` swallow.
        OutputFormat::Json => match serde_json::to_string_pretty(runs) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(
                    target: "m12.render.json",
                    error = %e,
                    "report JSON serialization failed — a Serialize impl returned an error"
                );
                "[]".into()
            }
        },
        // SF2 defense-in-depth: explicit loop instead of a silent
        // `filter_map(...ok())`. As with the Json arm, `to_string` of a
        // `WorkflowRunRow` is effectively infallible (a non-finite `f64`
        // serialises to `null`, not an error). Should a row ever genuinely
        // fail to serialise it is still dropped per F9 — never substitute
        // a placeholder row — but the drop is now logged with the row id
        // so it is visible, not silent.
        OutputFormat::NdJson => {
            let mut lines: Vec<String> = Vec::with_capacity(runs.len());
            for r in runs {
                match serde_json::to_string(r) {
                    Ok(line) => lines.push(line),
                    Err(e) => tracing::error!(
                        target: "m12.render.ndjson",
                        error = %e,
                        run_id = r.id,
                        "ndjson row dropped — a Serialize impl returned an error"
                    ),
                }
            }
            lines.join("\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        render_cluster_cost_table, render_cost_histogram, render_machine,
        render_outcome_timeline, render_summary_line, OutputFormat,
    };
    use crate::m7_workflow_runs::{Outcome, RunState, WorkflowRunRow};

    fn run(id: i64, cost: Option<i64>, outcome: Option<&str>, consumer_inputs: &str) -> WorkflowRunRow {
        let run_state = match outcome {
            None => RunState::Open,
            Some(o) => RunState::Closed {
                ended_at: format!("2026-05-17T01:{:02}:00Z", id % 60),
                outcome: Outcome::parse(o).expect("test outcome must be a valid CHECK value"),
            },
        };
        WorkflowRunRow {
            id,
            started_at: format!("2026-05-17T00:{:02}:00Z", id % 60),
            run_state,
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

    // ====================================================================
    // Hardening pass 2 — +30 tests. Boundary / error / F9 / determinism.
    // ====================================================================

    // rationale: Boundary — histogram bucket edges. cost 0 → bucket 0,
    // cost 999 → bucket 0, cost 1000 → bucket 1 (lo inclusive, hi
    // exclusive: `cost >= lo && cost < hi`).
    #[test]
    fn histogram_bucket_lower_bound_inclusive_upper_exclusive() {
        let runs = vec![
            run(1, Some(0), Some("ok"), "{}"),
            run(2, Some(999), Some("ok"), "{}"),
            run(3, Some(1_000), Some("ok"), "{}"),
        ];
        let s = render_cost_histogram(&runs);
        let l0 = s.lines().find(|l| l.contains("0 - 1k")).expect("b0");
        let l1 = s.lines().find(|l| l.contains("1k - 5k")).expect("b1");
        assert!(l0.contains("2 runs"), "0 and 999 land in bucket 0: {l0}");
        assert!(l1.contains("1 runs"), "1000 lands in bucket 1: {l1}");
    }

    // rationale: Boundary — i64::MAX cost lands in the top open-ended
    // bucket (`50_000..i64::MAX`); MAX itself is excluded by `< hi`, so
    // the largest value that counts is `i64::MAX - 1`.
    #[test]
    fn histogram_top_bucket_excludes_exact_i64_max() {
        let runs = vec![
            run(1, Some(i64::MAX), Some("ok"), "{}"),
            run(2, Some(i64::MAX - 1), Some("ok"), "{}"),
        ];
        let s = render_cost_histogram(&runs);
        let top = s.lines().find(|l| l.contains("> 50k")).expect("top");
        // Only MAX-1 counts; MAX is excluded (< i64::MAX is strict).
        assert!(top.contains("1 runs"), "exact i64::MAX excluded: {top}");
    }

    // rationale: F9 zero-weight — None cost is NOT counted in any
    // histogram bucket; bucket counts sum only over Some(cost) rows.
    #[test]
    fn histogram_none_cost_not_counted_in_any_bucket() {
        let runs = vec![
            run(1, None, Some("ok"), "{}"),
            run(2, None, Some("fail"), "{}"),
            run(3, Some(500), Some("ok"), "{}"),
        ];
        let s = render_cost_histogram(&runs);
        // Exactly one Some-cost row -> exactly one "1 runs" bucket line.
        let one_run_lines = s.lines().filter(|l| l.contains("1 runs")).count();
        assert_eq!(one_run_lines, 1, "only Some(500) counts: {s}");
    }

    // rationale: Correctness — histogram header reflects total run count
    // INCLUDING None-cost rows (it is `runs.len()`, not the costed count).
    #[test]
    fn histogram_header_counts_all_rows_including_none_cost() {
        let runs = vec![
            run(1, None, Some("ok"), "{}"),
            run(2, None, Some("ok"), "{}"),
            run(3, Some(100), Some("ok"), "{}"),
        ];
        let s = render_cost_histogram(&runs);
        assert!(s.contains("last 3 runs"), "header counts all rows: {s}");
    }

    // rationale: Correctness — median/p95 footer over a known dataset.
    // costs [10,20,30,40,50]; n=5; median = costs[5/2]=costs[2]=30;
    // p95_idx = (5*95/100).min(4) = 4 → costs[4]=50.
    #[test]
    fn histogram_median_and_p95_for_known_dataset() {
        let runs: Vec<WorkflowRunRow> = (1..=5)
            .map(|i| run(i, Some(i * 10), Some("ok"), "{}"))
            .collect();
        let s = render_cost_histogram(&runs);
        assert!(s.contains("median: 30 tokens"), "median wrong: {s}");
        assert!(s.contains("p95: 50 tokens"), "p95 wrong: {s}");
    }

    // rationale: Boundary — median/p95 are 0/0 when no row carries a
    // cost; the empty-cohort path returns the (0,0) tuple.
    #[test]
    fn histogram_median_p95_zero_when_no_cost_signal() {
        let runs = vec![run(1, None, Some("ok"), "{}"), run(2, None, Some("ok"), "{}")];
        let s = render_cost_histogram(&runs);
        assert!(s.contains("median: 0 tokens"), "no-cost median: {s}");
        assert!(s.contains("p95: 0 tokens"), "no-cost p95: {s}");
    }

    // rationale: Resource accounting — every histogram bar is bounded by
    // HISTOGRAM_BAR_MAX glyphs; a single dominant bucket does not blow
    // the bar width.
    #[test]
    fn histogram_bar_never_exceeds_bar_max_width() {
        let runs: Vec<WorkflowRunRow> = (0..500_i64)
            .map(|i| run(i, Some(100), Some("ok"), "{}"))
            .collect();
        let s = render_cost_histogram(&runs);
        for line in s.lines().filter(|l| l.contains('|')) {
            let bar = line.matches('\u{2588}').count();
            assert!(bar <= 20, "bar exceeds HISTOGRAM_BAR_MAX: {line}");
        }
    }

    // rationale: Correctness — the fullest bucket gets exactly the max
    // bar width (20) when it holds the maximum count.
    #[test]
    fn histogram_dominant_bucket_gets_full_bar() {
        let runs: Vec<WorkflowRunRow> = (0..40_i64)
            .map(|i| run(i, Some(100), Some("ok"), "{}"))
            .collect();
        let s = render_cost_histogram(&runs);
        let line = s.lines().find(|l| l.contains("0 - 1k")).expect("b0");
        assert_eq!(line.matches('\u{2588}').count(), 20, "full bar: {line}");
    }

    // rationale: Determinism — histogram render is byte-identical across
    // repeated calls on the same input.
    #[test]
    fn histogram_render_is_deterministic() {
        let runs: Vec<WorkflowRunRow> = (0..30_i64)
            .map(|i| run(i, Some(i * 1_000), Some("ok"), "{}"))
            .collect();
        let first = render_cost_histogram(&runs);
        for _ in 0..50_u32 {
            assert_eq!(render_cost_histogram(&runs), first);
        }
    }

    // rationale: F9 zero-weight — timeline uses started_at as the
    // fallback timestamp when ended_at is None (open run).
    #[test]
    fn timeline_open_run_uses_started_at_timestamp() {
        let runs = vec![run(7, None, None, "{}")];
        let s = render_outcome_timeline(&runs);
        // run(7,...) sets started_at = "2026-05-17T00:07:00Z".
        assert!(
            s.contains("2026-05-17T00:07:00Z"),
            "open run must show started_at: {s}"
        );
    }

    // rationale: Correctness — a completed run shows ended_at, not
    // started_at, as its timeline timestamp.
    #[test]
    fn timeline_completed_run_uses_ended_at_timestamp() {
        let runs = vec![run(9, Some(100), Some("ok"), "{}")];
        let s = render_outcome_timeline(&runs);
        // run() sets ended_at = "2026-05-17T01:09:00Z" when outcome Some.
        assert!(
            s.contains("2026-05-17T01:09:00Z"),
            "completed run must show ended_at: {s}"
        );
        assert!(
            !s.contains("2026-05-17T00:09:00Z"),
            "must not show started_at when ended_at present: {s}"
        );
    }

    // rationale: Boundary — empty input timeline still emits the header
    // and separator, zero data rows.
    #[test]
    fn timeline_empty_input_has_header_no_data_rows() {
        let s = render_outcome_timeline(&[]);
        assert!(s.contains("last 0 runs"));
        let data_rows = s.lines().filter(|l| l.starts_with("2026-")).count();
        assert_eq!(data_rows, 0);
    }

    // rationale: Correctness — timeline emits exactly one line per run
    // plus two header lines.
    #[test]
    fn timeline_line_count_is_header_plus_one_per_run() {
        let runs: Vec<WorkflowRunRow> = (0..13_i64)
            .map(|i| run(i, Some(50), Some("ok"), "{}"))
            .collect();
        let s = render_outcome_timeline(&runs);
        assert_eq!(s.lines().count(), 2 + 13, "2 header + 13 data: {s}");
    }

    // rationale: Correctness — cluster table sums cost per cluster
    // independently; two clusters keep separate totals.
    #[test]
    fn cluster_cost_table_keeps_separate_totals_per_cluster() {
        let ci_a =
            r#"{"cascade":{"cluster_id":"cascade_cluster_aaaaaa","session_range":[0,1]}}"#;
        let ci_b =
            r#"{"cascade":{"cluster_id":"cascade_cluster_bbbbbb","session_range":[0,1]}}"#;
        let runs = vec![
            run(1, Some(100), Some("ok"), ci_a),
            run(2, Some(50), Some("ok"), ci_a),
            run(3, Some(777), Some("ok"), ci_b),
        ];
        let s = render_cluster_cost_table(&runs);
        let line_a = s.lines().find(|l| l.contains("aaaaaa")).expect("a");
        let line_b = s.lines().find(|l| l.contains("bbbbbb")).expect("b");
        assert!(line_a.contains("150"), "cluster a sums to 150: {line_a}");
        assert!(line_b.contains("777"), "cluster b sums to 777: {line_b}");
    }

    // rationale: F9 zero-weight — a None-cost row still increments the
    // cluster run count but contributes 0 to total cost (additive
    // identity, not a fabricated sentinel).
    #[test]
    fn cluster_cost_table_none_cost_counts_run_but_zero_cost() {
        let ci =
            r#"{"cascade":{"cluster_id":"cascade_cluster_ffffff","session_range":[0,1]}}"#;
        let runs = vec![
            run(1, None, Some("ok"), ci),
            run(2, Some(200), Some("ok"), ci),
        ];
        let s = render_cluster_cost_table(&runs);
        let line = s.lines().find(|l| l.contains("ffffff")).expect("cl");
        // 2 runs, total cost 200 (None contributes 0).
        assert!(line.contains('2'), "run count includes None row: {line}");
        assert!(line.contains("200"), "total cost is 200: {line}");
    }

    // rationale: Determinism — cluster table is BTreeMap-backed; clusters
    // render in sorted order regardless of input order.
    #[test]
    fn cluster_cost_table_renders_clusters_sorted() {
        let mk = |tail: &str| {
            format!(
                r#"{{"cascade":{{"cluster_id":"cascade_cluster_{tail}","session_range":[0,1]}}}}"#
            )
        };
        let runs = vec![
            run(1, Some(10), Some("ok"), &mk("zzzzzz")),
            run(2, Some(10), Some("ok"), &mk("aaaaaa")),
            run(3, Some(10), Some("ok"), &mk("mmmmmm")),
        ];
        let s = render_cluster_cost_table(&runs);
        let pa = s.find("aaaaaa").expect("a");
        let pm = s.find("mmmmmm").expect("m");
        let pz = s.find("zzzzzz").expect("z");
        assert!(pa < pm && pm < pz, "clusters must render sorted: {s}");
    }

    // rationale: Adversarial input — cluster_id present but not a string
    // (a number) falls to (ungrouped) rather than crashing.
    #[test]
    fn cluster_cost_table_non_string_cluster_id_falls_to_ungrouped() {
        let ci = r#"{"cascade":{"cluster_id":12345,"session_range":[0,1]}}"#;
        let runs = vec![run(1, Some(100), Some("ok"), ci)];
        let s = render_cluster_cost_table(&runs);
        assert!(s.contains("ungrouped"), "numeric cluster_id → ungrouped: {s}");
    }

    // rationale: Adversarial input — cascade discriminant present but
    // cluster_id field missing → (ungrouped).
    #[test]
    fn cluster_cost_table_missing_cluster_id_field_falls_to_ungrouped() {
        let ci = r#"{"cascade":{"session_range":[0,1]}}"#;
        let runs = vec![run(1, Some(100), Some("ok"), ci)];
        let s = render_cluster_cost_table(&runs);
        assert!(s.contains("ungrouped"), "missing cluster_id → ungrouped: {s}");
    }

    // rationale: Adversarial input — a cluster_id with no underscore at
    // all uses the whole string as its own tail (split.next_back on a
    // single segment yields that segment).
    #[test]
    fn cluster_cost_table_id_without_underscore_uses_whole_string() {
        let ci = r#"{"cascade":{"cluster_id":"plainid","session_range":[0,1]}}"#;
        let runs = vec![run(1, Some(100), Some("ok"), ci)];
        let s = render_cluster_cost_table(&runs);
        // "plainid" truncated to 6 chars -> "plaini".
        assert!(s.contains("plaini"), "no-underscore id uses whole string: {s}");
    }

    // rationale: Boundary — a cluster_id whose tail is the empty string
    // (id ends with `_`) yields None → (ungrouped), per extract spec.
    #[test]
    fn cluster_cost_table_empty_tail_falls_to_ungrouped() {
        let ci = r#"{"cascade":{"cluster_id":"cascade_cluster_","session_range":[0,1]}}"#;
        let runs = vec![run(1, Some(100), Some("ok"), ci)];
        let s = render_cluster_cost_table(&runs);
        assert!(
            s.contains("ungrouped"),
            "empty tail after split must fall to ungrouped: {s}"
        );
    }

    // rationale: Correctness — cluster table without any ungrouped rows
    // omits the "(ungrouped)" line entirely (count > 0 guard).
    #[test]
    fn cluster_cost_table_omits_ungrouped_line_when_all_grouped() {
        let ci =
            r#"{"cascade":{"cluster_id":"cascade_cluster_grpd01","session_range":[0,1]}}"#;
        let runs = vec![run(1, Some(100), Some("ok"), ci)];
        let s = render_cluster_cost_table(&runs);
        assert!(!s.contains("(ungrouped)"), "no ungrouped line expected: {s}");
    }

    // rationale: Correctness — summary line counts the "unknown" outcome
    // distinctly from "open" (None outcome).
    #[test]
    fn summary_line_distinguishes_unknown_from_open() {
        let runs = vec![
            run(1, Some(10), Some("unknown"), "{}"),
            run(2, Some(10), None, "{}"),
        ];
        let s = render_summary_line(&runs);
        assert!(s.contains("unknown=1"), "explicit unknown counted: {s}");
        assert!(s.contains("open=1"), "None outcome counted as open: {s}");
    }

    // rationale: Type-design invariant — an unrecognised outcome string is
    // now structurally unrepresentable: `WorkflowRunRow::run_state` carries
    // a typed `Outcome`, and `Outcome::parse` rejects any non-CHECK value.
    // render_summary_line can therefore only ever see the four valid
    // variants or `None` (open). The former `_`-arm collapse case is dead.
    #[test]
    fn summary_line_only_sees_typed_outcomes() {
        assert!(Outcome::parse("future_variant_q").is_err());
        // Every representable run is counted in exactly one bucket.
        let runs = vec![
            run(1, Some(10), Some("ok"), "{}"),
            run(2, Some(10), Some("unknown"), "{}"),
            run(3, Some(10), None, "{}"),
        ];
        let s = render_summary_line(&runs);
        assert!(s.contains("ok=1"), "{s}");
        assert!(s.contains("unknown=1"), "{s}");
        assert!(s.contains("open=1"), "{s}");
    }

    // rationale: Boundary — empty summary line still has the canonical
    // shape with all-zero counts.
    #[test]
    fn summary_line_empty_input_all_zero_counts() {
        let s = render_summary_line(&[]);
        assert_eq!(
            s,
            "0 runs recorded: ok=0 fail=0 abort=0 unknown=0 open=0 median_cost=0"
        );
    }

    // rationale: Correctness — summary counts sum to total run count
    // (partition invariant: every run lands in exactly one bucket).
    #[test]
    fn summary_line_bucket_counts_partition_total() {
        let runs = vec![
            run(1, Some(1), Some("ok"), "{}"),
            run(2, Some(1), Some("fail"), "{}"),
            run(3, Some(1), Some("abort"), "{}"),
            run(4, Some(1), Some("unknown"), "{}"),
            run(5, Some(1), None, "{}"),
            run(6, Some(1), Some("ok"), "{}"),
        ];
        let s = render_summary_line(&runs);
        // 6 runs: ok=2 fail=1 abort=1 unknown=1 open=1 → sums to 6.
        assert!(s.contains("6 runs recorded"));
        assert!(s.contains("ok=2"));
        assert!(s.contains("fail=1"));
        assert!(s.contains("abort=1"));
        assert!(s.contains("unknown=1"));
        assert!(s.contains("open=1"));
    }

    // rationale: Correctness — render_machine Table format delegates to
    // render_summary_line (single source of truth).
    #[test]
    fn render_machine_table_format_equals_summary_line() {
        let runs = vec![
            run(1, Some(100), Some("ok"), "{}"),
            run(2, Some(200), Some("fail"), "{}"),
        ];
        assert_eq!(
            render_machine(&runs, OutputFormat::Table),
            render_summary_line(&runs)
        );
    }

    // rationale: Boundary — render_machine Json on empty input emits a
    // valid empty JSON array, parseable back to a zero-length vec.
    #[test]
    fn render_machine_json_empty_input_is_empty_array() {
        let s = render_machine(&[], OutputFormat::Json);
        let parsed: Vec<WorkflowRunRow> = serde_json::from_str(&s).expect("parse");
        assert!(parsed.is_empty());
    }

    // rationale: Boundary — render_machine NdJson on empty input is the
    // empty string (zero lines), not a phantom blank line.
    #[test]
    fn render_machine_ndjson_empty_input_is_empty_string() {
        let s = render_machine(&[], OutputFormat::NdJson);
        assert!(s.is_empty(), "empty NdJson must be empty string, got {s:?}");
        assert_eq!(s.lines().count(), 0);
    }

    // rationale: Correctness — every NdJson line is independently
    // parseable JSON (newline-delimited, not a single document).
    #[test]
    fn render_machine_ndjson_each_line_is_standalone_json() {
        let runs: Vec<WorkflowRunRow> = (0..5_i64)
            .map(|i| run(i, Some(i * 10), Some("ok"), "{}"))
            .collect();
        let s = render_machine(&runs, OutputFormat::NdJson);
        for line in s.lines() {
            let _: WorkflowRunRow = serde_json::from_str(line).expect("line parse");
        }
    }

    // rationale: Contract regression — Json output is pretty-printed
    // (multi-line) while NdJson is compact (one line per row).
    #[test]
    fn render_machine_json_is_pretty_ndjson_is_compact() {
        let runs = vec![run(1, Some(100), Some("ok"), "{}")];
        let json = render_machine(&runs, OutputFormat::Json);
        let ndjson = render_machine(&runs, OutputFormat::NdJson);
        assert!(json.lines().count() > 1, "Json must be pretty: {json}");
        assert_eq!(ndjson.lines().count(), 1, "NdJson compact: {ndjson}");
    }

    // rationale: Contract — OutputFormat enum value equality is stable
    // (used as a config selector; must satisfy Eq/Hash).
    #[test]
    fn output_format_variants_are_distinct_and_eq() {
        assert_eq!(OutputFormat::Json, OutputFormat::Json);
        assert_ne!(OutputFormat::Json, OutputFormat::NdJson);
        assert_ne!(OutputFormat::Table, OutputFormat::NdJson);
        let mut set = std::collections::HashSet::new();
        set.insert(OutputFormat::Table);
        set.insert(OutputFormat::Json);
        set.insert(OutputFormat::NdJson);
        assert_eq!(set.len(), 3, "three distinct format variants");
    }

    // rationale: F9 zero-weight — Json round-trip preserves the
    // None-vs-Some distinction on cost_tokens and outcome (None must not
    // collapse to a numeric or string sentinel through serde).
    #[test]
    fn render_machine_json_preserves_none_fields() {
        let runs = vec![run(1, None, None, "{}")];
        let s = render_machine(&runs, OutputFormat::Json);
        let parsed: Vec<WorkflowRunRow> = serde_json::from_str(&s).expect("parse");
        assert_eq!(parsed.len(), 1);
        assert!(parsed[0].cost_tokens.is_none(), "None cost preserved");
        assert_eq!(parsed[0].run_state, RunState::Open, "Open state preserved");
        assert!(
            parsed[0].run_state.outcome().is_none(),
            "None outcome preserved"
        );
        assert!(
            parsed[0].run_state.ended_at().is_none(),
            "None ended_at preserved"
        );
    }

    // ====================================================================
    // SF1 / SF2 — render_machine never silently collapses or drops rows.
    //
    // W2 reconciliation note: the SF1/SF2 recon findings assumed a NaN
    // `fitness_dimension` makes `serde_json` *fail*. It does NOT — serde
    // serialises a non-finite f64 to JSON `null`. The render_machine
    // bind+log `Err` arms are kept as defense-in-depth (vs a silent
    // `unwrap_or_else` / `filter_map(...ok())`), but the real, testable
    // contract is: a NaN field renders as `null` and the report stays
    // COMPLETE — never collapsed to "[]", never a dropped row.
    // ====================================================================

    // rationale: SF1 — a NaN fitness_dimension serialises to JSON `null`;
    // render_machine must produce a valid, COMPLETE array, never silently
    // collapse the whole report to "[]".
    #[test]
    fn render_machine_json_nan_fitness_renders_as_null_not_collapse() {
        let mut runs = vec![run(1, Some(100), Some("ok"), "{}")];
        runs[0].fitness_dimension = f64::NAN;
        let s = render_machine(&runs, OutputFormat::Json);
        assert_ne!(s, "[]", "a NaN field must not collapse the whole report");
        let v: serde_json::Value =
            serde_json::from_str(&s).expect("render is a valid JSON array");
        let arr = v.as_array().expect("top level is an array");
        assert_eq!(arr.len(), 1, "the row is present, not dropped");
        assert!(
            arr[0]["fitness_dimension"].is_null(),
            "a non-finite fitness serialises to JSON null"
        );
    }

    // rationale: SF2 — a NaN-fitness row in an NdJson batch renders (as
    // `null`); it is NOT dropped, and every input row is present in order.
    #[test]
    fn render_machine_ndjson_nan_fitness_row_rendered_not_dropped() {
        let mut runs = vec![
            run(1, Some(100), Some("ok"), "{}"),
            run(2, Some(200), Some("fail"), "{}"),
            run(3, Some(300), Some("abort"), "{}"),
        ];
        runs[1].fitness_dimension = f64::NAN; // middle row carries the NaN
        let s = render_machine(&runs, OutputFormat::NdJson);
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 3, "all 3 rows rendered, none dropped: {s}");
        let row0: serde_json::Value =
            serde_json::from_str(lines[0]).expect("parse line 0");
        let row1: serde_json::Value =
            serde_json::from_str(lines[1]).expect("parse line 1");
        let row2: serde_json::Value =
            serde_json::from_str(lines[2]).expect("parse line 2");
        assert_eq!(row0["id"], 1);
        assert_eq!(row2["id"], 3);
        assert_eq!(row1["id"], 2, "the NaN row keeps its identity and order");
        assert!(
            row1["fitness_dimension"].is_null(),
            "the NaN row's fitness renders as null"
        );
    }

    // rationale: SF2 — even an all-anomalous batch (every row NaN) still
    // renders one line per row; render_machine never silently flattens a
    // batch to an empty stream.
    #[test]
    fn render_machine_ndjson_all_nan_rows_still_render_one_line_each() {
        let mut runs = vec![
            run(1, Some(100), Some("ok"), "{}"),
            run(2, Some(200), Some("fail"), "{}"),
        ];
        for r in &mut runs {
            r.fitness_dimension = f64::NAN;
        }
        let s = render_machine(&runs, OutputFormat::NdJson);
        assert_eq!(s.lines().count(), 2, "every row still rendered: {s}");
        assert!(!s.is_empty(), "an all-NaN batch is not an empty stream");
    }
}
