//! Integration tests for m12 `cli_reports` (Wave-D1).
//!
//! Exercises the m12 surface at its public-API call boundary:
//!
//! - Cross-module gate (m12 → m10): every rendered string MUST pass the
//!   m10 Ember rubric via `evaluate_string(...) == GateVerdict::Pass`.
//!   m12 is a leaf formatter whose output is a downstream consumer of the
//!   Ember CI gate — this is the strongest m12↔m10 contract and is
//!   re-verified here at the crate-boundary (the in-module tests scope it
//!   to the same crate; this suite proves it across the integration seam).
//! - F9 zero-weight anti-property — `cost_tokens: None` renders the
//!   signal-absent sentinel `---`, never the numeric sentinel `0`.
//!   Explicit `Some(0)` renders distinctly as `0 tok`.
//! - F11 truncation contract — cluster ids are truncated to their opaque
//!   6-char hex tail; the full upstream label must never leak.
//! - Boundary — empty input slice renders the zero-state line cleanly.
//! - Machine output — `render_machine(NdJson)` emits one valid JSON
//!   document per line.
//!
//! Day-1 surface notes (flagged for orchestrator):
//!
//! - m12 exposes no public seam to inject a clock or row id — every
//!   renderer is a pure `&[WorkflowRunRow] -> String`, which is the
//!   correct design for a leaf formatter and needs no test double.
//! - `extract_cluster_short` is module-private; the F11 truncation
//!   contract is exercised through the public `render_cluster_cost_table`.

#![allow(clippy::doc_markdown)]

use workflow_core::m10_ember_ci_gate::{evaluate_string, GateVerdict};
use workflow_core::m12_cli_reports::{
    render_cluster_cost_table, render_cost_histogram, render_machine, render_outcome_timeline,
    render_summary_line, OutputFormat,
};
use workflow_core::m7_workflow_runs::WorkflowRunRow;

// ---- fixtures ------------------------------------------------------------

/// Build a `WorkflowRunRow`. `outcome == None` models an open run;
/// `cost == None` models the signal-absent (F9) case.
fn run(
    id: i64,
    cost: Option<i64>,
    outcome: Option<&str>,
    consumer_inputs: &str,
) -> WorkflowRunRow {
    WorkflowRunRow {
        id,
        started_at: format!("2026-05-20T00:{:02}:00Z", id % 60),
        ended_at: outcome.map(|_| format!("2026-05-20T01:{:02}:00Z", id % 60)),
        outcome: outcome.map(str::to_owned),
        consumer_inputs: consumer_inputs.to_owned(),
        cost_tokens: cost,
        fitness_dimension: 0.0,
    }
}

/// A representative mixed-outcome run set used by several cross-module
/// gate tests.
fn mixed_runs() -> Vec<WorkflowRunRow> {
    vec![
        run(1, Some(120), Some("ok"), "{}"),
        run(2, Some(4_800), Some("fail"), "{}"),
        run(3, None, None, "{}"),
        run(4, Some(0), Some("abort"), "{}"),
        run(5, Some(61_000), Some("unknown"), "{}"),
    ]
}

const CLUSTER_CI: &str = r#"{"cascade":{"kind":"cascade","cluster_id":"cascade_cluster_abc123def456","session_range":[0,1]}}"#;

// ---- cross-module: m12 → m10 Ember gate ----------------------------------

// rationale: Cross-module surface invariant (m12 → m10) — the rendered
// summary line MUST pass the Ember rubric. m12's contract is that all
// human-facing output uses passive descriptors only; the gate is the
// arbiter, and this re-verifies the contract across the crate boundary.
#[test]
fn m12_render_summary_line_passes_ember_gate() {
    // rationale: Cross-module surface invariant
    let s = render_summary_line(&mixed_runs());
    assert_eq!(
        evaluate_string("m12.summary", &s, &[]),
        GateVerdict::Pass,
        "m12 summary line failed the m10 Ember gate: {s:?}"
    );
}

// rationale: Cross-module surface invariant (m12 → m10) — the cost
// histogram MUST pass the Ember gate, including its header and footer
// statistic lines.
#[test]
fn m12_render_cost_histogram_passes_ember_gate() {
    // rationale: Cross-module surface invariant
    let s = render_cost_histogram(&mixed_runs());
    assert_eq!(
        evaluate_string("m12.histogram", &s, &[]),
        GateVerdict::Pass,
        "m12 cost histogram failed the m10 Ember gate: {s:?}"
    );
}

// rationale: Cross-module surface invariant (m12 → m10) — the outcome
// timeline MUST pass the Ember gate across typical, open-run, and
// empty-input shapes.
#[test]
fn m12_render_outcome_timeline_passes_ember_gate() {
    // rationale: Cross-module surface invariant
    for runs in [vec![], mixed_runs(), vec![run(1, None, None, "{}")]] {
        let s = render_outcome_timeline(&runs);
        assert_eq!(
            evaluate_string("m12.timeline", &s, &[]),
            GateVerdict::Pass,
            "m12 timeline failed the m10 Ember gate: {s:?}"
        );
    }
}

// rationale: Cross-module surface invariant (m12 → m10) — the cluster
// cost table MUST pass the Ember gate for both grouped (cascade cluster
// present) and ungrouped rows.
#[test]
fn m12_render_cluster_cost_table_passes_ember_gate() {
    // rationale: Cross-module surface invariant
    let runs = vec![
        run(1, Some(100), Some("ok"), CLUSTER_CI),
        run(2, Some(200), Some("ok"), "{}"),
    ];
    let s = render_cluster_cost_table(&runs);
    assert_eq!(
        evaluate_string("m12.cluster_cost", &s, &[]),
        GateVerdict::Pass,
        "m12 cluster cost table failed the m10 Ember gate: {s:?}"
    );
}

// ---- machine output ------------------------------------------------------

// rationale: Contract — `render_machine(NdJson)` emits newline-delimited
// JSON; every line MUST parse independently as a JSON document and the
// row count MUST equal the input length.
#[test]
fn m12_render_machine_emits_valid_ndjson() {
    // rationale: Contract regression (NDJSON wire shape)
    let runs = mixed_runs();
    let s = render_machine(&runs, OutputFormat::NdJson);
    let lines: Vec<&str> = s.lines().collect();
    assert_eq!(
        lines.len(),
        runs.len(),
        "NdJson must emit exactly one line per row"
    );
    for (i, line) in lines.iter().enumerate() {
        let parsed: serde_json::Value =
            serde_json::from_str(line).unwrap_or_else(|e| panic!("line {i} not valid JSON: {e}"));
        // Each line is a workflow-run object, not an array or scalar.
        assert!(
            parsed.get("id").is_some(),
            "line {i} missing the `id` field: {line}"
        );
    }
}

// ---- F9 zero-weight anti-property ----------------------------------------

// rationale: Anti-property (F9 zero-weight) — a row with `cost_tokens ==
// None` renders the signal-absent sentinel `---` and MUST NOT collapse
// to a numeric `0`. None means "no cost recorded yet", not "zero cost".
#[test]
fn m12_timeline_none_cost_renders_dash_not_zero() {
    // rationale: Anti-property (F9 zero-weight)
    let s = render_outcome_timeline(&[run(1, None, Some("ok"), "{}")]);
    assert!(
        s.contains("---"),
        "None cost must render as the dash sentinel: {s}"
    );
    assert!(
        !s.contains("0 tok"),
        "None cost must NOT collapse to a numeric 0: {s}"
    );
}

// rationale: Anti-property (F9 zero-weight) — an explicit `Some(0)` cost
// is a recorded zero and MUST render as `0 tok`, distinct from the None
// dash sentinel. This is the symmetric half of the F9 contract.
#[test]
fn m12_timeline_explicit_zero_cost_renders_as_zero() {
    // rationale: Anti-property (F9 zero-weight — distinct from None)
    let s = render_outcome_timeline(&[run(1, Some(0), Some("ok"), "{}")]);
    assert!(
        s.contains("0 tok"),
        "explicit Some(0) must render as `0 tok`: {s}"
    );
    // Inspect the data line only — the header carries a `---` separator.
    let data_line = s
        .lines()
        .find(|l| l.starts_with("2026-"))
        .expect("timeline data line present");
    assert!(
        !data_line.contains("---"),
        "explicit zero must not render as the dash sentinel: {data_line}"
    );
}

// ---- F11 truncation contract ---------------------------------------------

// rationale: Contract regression (F11 opaque IDs) — the cluster cost
// table truncates the cluster-id tail to 6 chars. The full upstream
// label must never leak into rendered output.
#[test]
fn m12_cluster_cost_table_truncates_to_six_chars() {
    // rationale: Contract regression (F11 opaque IDs)
    let ci = r#"{"cascade":{"kind":"cascade","cluster_id":"cascade_cluster_SECRETLABELTOKEN","session_range":[0,1]}}"#;
    let s = render_cluster_cost_table(&[run(1, Some(100), Some("ok"), ci)]);
    assert!(
        !s.contains("SECRETLABELTOKEN"),
        "F11 leak: full cluster label revealed: {s}"
    );
    assert!(
        !s.contains("LABELTOKEN"),
        "F11 leak: cluster tail exceeded 6 chars: {s}"
    );
    // The 6-char prefix of the tail IS allowed to appear.
    assert!(
        s.contains("SECRET"),
        "expected 6-char truncated tail in output: {s}"
    );
}

// ---- boundary ------------------------------------------------------------

// rationale: Boundary — render_summary_line over an empty input slice
// produces the zero-state line ("0 runs recorded") and a single line of
// output; it must not panic or emit a degenerate string.
#[test]
fn m12_render_summary_line_handles_empty_input() {
    // rationale: Boundary (empty input slice)
    let s = render_summary_line(&[]);
    assert!(
        s.starts_with("0 runs recorded"),
        "empty input must render the zero-state summary: {s}"
    );
    assert_eq!(s.lines().count(), 1, "summary is a single line");
    // Boundary output must also still satisfy the m10 Ember gate.
    assert_eq!(
        evaluate_string("m12.summary.empty", &s, &[]),
        GateVerdict::Pass,
        "empty-input summary failed the Ember gate: {s:?}"
    );
}
