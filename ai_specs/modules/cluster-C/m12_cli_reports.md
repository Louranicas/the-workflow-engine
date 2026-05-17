---
title: m12 — cli_reports (human-readable report emitter)
module_id: m12
module_name: cli_reports
cluster: C — Correlation + Output
layer: L3
binary: wf-crystallise
verb_class: emit
feature_gate: [api]
loc_estimate: 80-120
test_budget: 50-60
boilerplate_lift: ~65-70%
gap_owner: none (F6 self-dispatch refusal owner at the render layer)
cc_contracts_owned: []
cc_contracts_consumed: []
status: SPEC · planning-only · HOLD-v2 active · no code until G1-G9 clear
authority: Luke @ node 0.A
date: 2026-05-17 (S1001982)
---

# m12 — `cli_reports`

> Back to: [`../../INDEX.md`](../../INDEX.md) · [`../../MODULE_MATRIX.md`](../../MODULE_MATRIX.md) · [`../../../CLAUDE.md`](../../../CLAUDE.md) · [`../../../CLAUDE.local.md`](../../../CLAUDE.local.md) · vault [[cluster-C-correlation-output]] · canonical V7 [cluster-C plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-C.md) · v1.3 spec [`../../../ai_docs/GENESIS_PROMPT_V1_3.md`](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 1
>
> Sister modules: [m7](m7_workflow_runs.md) · [m12](m12_cli_reports.md) · [m13](m13_stcortex_writer.md)
> Cluster peers in-flow: m7 is m12's sole upstream; stdout is m12's sole downstream

---

## 1 — Role + one-line purpose

m12 is the **sole path from machine-readable `workflow_runs` rows to human-readable CLI text**. It is a **pure formatter**: no database writes, no mutable state, no side effects beyond the bytes it returns. Its entire public API is a set of `#[must_use]` functions that take `&[WorkflowRunRow]` and return `String`.

Cluster C verb-class is `record` / `emit`; m12 specifically is the **passive emitter**. Phase A passive-verb discipline applies without exception: m12 emits text. m12 does not recommend, route, package, dispatch, optimise, or select.

## 2 — Cluster + layer + binary placement

| Axis | Value |
|---|---|
| Cluster | **C — Central Correlation + Output** |
| Layer | **L3** |
| Binary | **`wf-crystallise`** (rendering layer for the `wf-crystallise report` subcommand) |
| Feature gate | **`api`** — m12 is part of the `workflow-core` public surface; the `report` subcommand is wired in `wf-crystallise`'s `main.rs` |
| Verb class | **`emit`** (passive; renders, never decides) |
| src/ path | `src/m12_cli_reports/` with `mod.rs`, `format.rs` (Table / JSON / NdJson rendering), `refuse.rs` (F6 self-dispatch refusal), `error.rs` |

## 3 — Upstream-IN (what arrives)

| Source | Wire shape | Notes |
|---|---|---|
| **m7** `workflow_runs` | `&[WorkflowRunRow]` (slice, never owned) returned by `m7::find_open` / `m7::find_by_outcome` | m12 reads m7 indirectly — the caller (CLI `main.rs`) opens the connection, runs the query, hands the slice to m12 |
| **CLI flags** | `OutputFormat` enum (`Table | Json | NdJson`) + report-kind selector | parsed in `wf-crystallise`'s clap layer; m12 receives the format as a parameter |

m12 NEVER opens a database connection. m12 NEVER calls m7's public API directly — the indirection through `main.rs` keeps the render layer testable against synthetic `Vec<WorkflowRunRow>` fixtures without any live DB.

## 4 — Downstream-OUT (what departs)

| Destination | Wire shape | Notes |
|---|---|---|
| **stdout** | `String` (the entire rendered report) — caller writes via `tracing::info!` in JSON mode or `writeln!` in plain mode | m12 returns `String`; m12 NEVER prints |
| **m12 has no other consumer** | — | by design: m12 is a leaf in the data-flow DAG |

The "m12 returns String; the caller prints" separation is intentional. It (a) keeps m12 testable as a pure function, (b) lets the binary layer choose its emit channel (stdout in interactive, `tracing` JSON in machine mode), and (c) enforces the god-tier rule against `println!` / `eprintln!` inside library modules (use `tracing` instead, per ME v2 `logging.rs`).

## 5 — Aspect-IN (Cluster D trust-layer wraps)

| Aspect | Wrapping point |
|---|---|
| **m8** `povm_build_prereq` | compile-time gate (whole crate) |
| **m9** `watcher_namespace_guard` | **N/A** — m12 emits to stdout, not to substrate. No namespace boundary crossed |
| **m10** `ember_ci_gate` | **user-facing surface — primary audit target.** m12's report labels are scanned by the Ember 7-trait CI gate. Forbidden verbs ("recommend", "optimise", "select", "route", "dispatch", "auto") MUST NOT appear in the rendered output. m12's strings use passive descriptors only: "recorded", "observed", "emitted", "cost", "outcome", "rate" |
| **m11** `fitness_weighted_decay` | m12 does not read `fitness_dimension` (F9 zero-weight; reading the column would tempt rendering it, which is forbidden in Phase A). m12 ignores the column entirely |

## 6 — Public API (lifted verbatim from vault canonical)

```rust
/// Render a cost-band histogram from a slice of completed runs.
/// Returns a multi-line `String` suitable for stdout.
///
/// Buckets: `[0, 1_000)`, `[1_000, 5_000)`, `[5_000, 20_000)`,
/// `[20_000, 50_000)`, `[50_000, ∞)`. Bar width normalised to
/// the maximum bucket count, capped at 20 characters.
#[must_use]
pub fn render_cost_histogram(runs: &[WorkflowRunRow]) -> String;

/// Render an outcome timeline from a slice of runs, newest first.
/// Open runs (no `ended_at`) are included with cost rendered as `---`.
#[must_use]
pub fn render_outcome_timeline(runs: &[WorkflowRunRow]) -> String;

/// Render a cost-by-cascade-cluster table from a slice of runs.
/// Cluster IDs are truncated to their 6-char opaque prefix (F11 enforced).
/// Runs with no cascade observation appear in the `(ungrouped)` row.
#[must_use]
pub fn render_cluster_cost_table(runs: &[WorkflowRunRow]) -> String;

/// Render a compact summary line: total runs, outcome counts, median cost.
/// Used as the last line of every report.
#[must_use]
pub fn render_summary_line(runs: &[WorkflowRunRow]) -> String;

/// Render in machine-readable JSON / NdJson per output format.
/// Round-trips: `parse(render_json(runs)) == runs`.
#[must_use]
pub fn render_machine(runs: &[WorkflowRunRow], format: OutputFormat) -> String;
```

All functions are `#[must_use]` (god-tier rule — pure functions with no side effects must signal accidental discard). All take `&[WorkflowRunRow]` (slice, never owned) — m12 holds no allocations beyond the output string.

### Output format enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
    NdJson,
}
```

## 7 — Three report formats (lifted from vault canonical § Report formats)

Each format answers a direct question a human asks at the CLI. Layout is fixed-width; no floating-point arithmetic in the render path (counts are integers); no emoji (plain ASCII only).

### 7.1 — Cost-band histogram

Answers: "How are my workflow runs distributed by token cost?"

```
workflow-trace cost distribution (last 100 runs)
------------------------------------------------
   0 - 1k tokens |████████████████████| 23 runs
  1k - 5k tokens |███████████████     | 17 runs
 5k - 20k tokens |████████            |  9 runs
20k - 50k tokens |███                 |  3 runs
   > 50k tokens  |                    |  0 runs
------------------------------------------------
median: 3,412 tokens  p95: 18,204 tokens
```

Buckets are fixed: `[0, 1_000)`, `[1_000, 5_000)`, `[5_000, 20_000)`, `[20_000, 50_000)`, `[50_000, ∞)`. Bar width normalised to the maximum bucket count, capped at 20 characters.

### 7.2 — Outcome timeline

Answers: "What happened in the last N runs?"

```
workflow-trace timeline (last 20 runs)
---------------------------------------
2026-05-17T11:04:12Z  ok      3,412 tok
2026-05-17T10:51:33Z  fail    7,001 tok
2026-05-17T10:44:09Z  ok      2,189 tok
2026-05-17T10:38:55Z  abort   1,043 tok
...
---------------------------------------
20 runs shown  |  ok: 14  fail: 4  abort: 2  unknown: 0
```

Columns: `started_at` ISO-8601 (20 chars), `outcome` padded 7 chars, `cost_tokens` right-aligned 9 chars. Open runs (no `ended_at`) render `---` in the cost column. No emoji.

### 7.3 — Cost-by-cluster table

Answers: "Is there a cost pattern by cascade cluster type?"

```
workflow-trace cost by cascade cluster (last 100 runs)
-------------------------------------------------------
cluster-id  runs   median-tok   p95-tok   ok-rate
A4f9c2      12     2,311        8,910     92%
B7e1a8      8      5,902        22,100    75%
(ungrouped) 80     3,412        18,204    88%
-------------------------------------------------------
```

Cluster IDs printed as opaque 6-char prefix. "(ungrouped)" row covers runs with no cascade observation in `consumer_inputs`. **F11 enforced at the render layer** as well as the storage layer — no human-meaningful labels inferred or displayed.

## 8 — F6 self-dispatch refusal (the load-bearing invariant)

**F6** is "self-dispatch via the report layer" — the trivial recursion trap where a workflow that runs `wf-crystallise report` and pipes the output back into itself creates a causally-tainted feedback loop. m12 is the **prime owner of refusal at the render layer.**

The refusal check (`refuse.rs`):

```rust
/// Refuse to render if the current run's target workflow is itself the
/// reporting workflow. This is the F6 trivial-recursion guard.
pub(crate) fn refuse_if_self_dispatch(
    runs: &[WorkflowRunRow],
    reporting_workflow_id: Option<&str>,
) -> Result<(), ReportError> {
    if let Some(rw) = reporting_workflow_id {
        if runs.iter().any(|r| {
            r.consumer_inputs.contains(rw)
                && r.consumer_inputs.contains("\"target_workflow\"")
        }) {
            return Err(ReportError::SelfDispatchRefused);
        }
    }
    Ok(())
}
```

The check is invoked at the top of every public `render_*` function (or wrapped via a `render_with_refusal` helper). The reporting-workflow-id is sourced from `wf-crystallise`'s CLI layer when it detects it is being invoked from within another workflow.

F-Regression test: any commit that bypasses the refusal returns `Ok(_)` rather than `Err(SelfDispatchRefused)` for the canonical fixture → fails the regression suite.

## 9 — Error taxonomy (thiserror)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("self-dispatch refused — reporting workflow appears in input set")]
    SelfDispatchRefused,
    #[error("forbidden verb '{0}' detected in render output (Ember gate)")]
    ForbiddenVerb(String),
    #[error("serialization failed: {0}")]
    Serde(String),
}
```

m12 only emits errors at the F6 boundary and at the serde boundary for `Json` / `NdJson` modes. The plain `Table` renders are infallible (every input shape produces some output, even if "empty results").

## 10 — Ember gate compliance (m10 audit target)

m12's user-facing strings MUST pass m10's Ember gate. The gate scans rendered output for forbidden verbs:

| Forbidden | Why | Allowed alternative |
|---|---|---|
| "recommend" | active verb; Phase A is passive | "recorded" |
| "optimise" | active verb | "observed" |
| "select" | active verb (m31's lane) | "by outcome" |
| "route" | active verb (m32's lane) | "by cluster" |
| "dispatch" | active verb (m32's lane) | "ran" |
| "auto" | implies agency | "automated" (still forbidden in m12 output; use "scheduled" or drop entirely) |

The forbidden-verb test fixture lives in m10's `tests/ember_gate.rs` and includes m12 sample renders. Render functions carry doc comments citing the Ember requirement so the audit trail is visible from the API alone.

## 11 — Tests (50-60 minimum; per `TEST_DISCIPLINE.md` row m12)

Allocation lifted from V7 cluster-C plan § m12 Test-pattern allocation:

| Pattern | Count | Coverage |
|---|---|---|
| F-Unit | 25 | table rendering per-row + per-column · JSON/NdJson rendering · F6 refusal per match arm · empty-result rendering · long-field truncation · histogram bucket-count integer-arithmetic · open-run `---` rendering · summary-line correctness |
| F-Property | 5 | render-roundtrip: `parse(render_json(runs)) == runs` · `parse(render_ndjson(runs)) == runs` · table-render width monotonic · histogram bucket sums = total · cluster-table ungrouped + grouped row count = total |
| F-Fuzz | 0 | — |
| F-Integration | 15 | m12 ↔ m7 read · full CLI invocation `wf-crystallise report --format=table` · output-format switching · F6 end-to-end refusal · Ember gate scan against canonical fixtures · 100-run slice timeline footer correctness |
| F-Contract | 3 | output-schema insta snapshot per format (Table / Json / NdJson) |
| F-Regression | 2 | **F6 regression** (any commit silently allowing self-dispatch) · forbidden-verb regression (any new label string introducing "recommend"/etc.) |
| F-Mutation | budget | ≥70% on `format.rs` + `refuse.rs` |

Key edge cases the test suite must cover (from vault § Test density target):
- Empty slice input: all four render functions return a non-empty string with zero-count labels (never panic on empty input).
- Single-run slice: no division-by-zero or index-out-of-bounds in histogram median / p95 computation.
- Open-run rendering: `render_outcome_timeline` with `ended_at = None` renders `---` in the cost column.
- Forbidden-verb scan: `assert!(!render_cost_histogram(&fixture).contains("recommend"))` and analogues for all four render functions and all six forbidden verbs.

## 12 — Reuse + boilerplate lift

| Source | Lift % | What |
|---|---|---|
| ME v2 `metrics.rs` report style | ~70% | `Labels` builder · histogram bucketing (counter per bucket → bar chart) · separation of aggregation from formatting (`MetricsRegistry` accumulates; `render_*()` produces text) |
| `01-cli-scaffolding` (weaver.rs / enforcer.rs) | ~50% | structured error boundaries · `tracing::info!` JSON emit for machine mode · `EnvFilter` for log-level switching |
| `comfy-table` crate idioms | ~70% | Table rendering (standard library pattern) |
| `serde_json::to_writer` / `to_writer_pretty` | ~80% | Json / NdJson rendering |
| F6 refusal | **0% (fresh)** | ~15 LOC novel; vault canonical § m12 refusal |

Net lift across m12: ~65-70%. Fresh authorship concentrated in the F6 refusal predicate and the histogram bucket computation against the integer-only constraint.

## 13 — Cross-cluster contracts owned + consumed

m12 owns **no** cross-cluster contract. It is a pure read-side consumer at the engine periphery. Specifically:

- **CC-1 (Cascade-Cost Coupling, m7 owns)** — m12 reads `consumer_inputs` JSONB for the cost-by-cluster render. m12 parses the discriminants but never writes back. F11 enforced at the render layer (opaque 6-char prefix).
- **CC-2 (Trust Layer Woven)** — m12 is wrapped by m10 (Ember gate audits user-facing output); not by m9 (no substrate write).
- **CC-3, CC-4, CC-5, CC-6, CC-7** — m12 does not participate. The render layer is downstream of evidence aggregation, dispatch, and substrate feedback.

## Failure-modes covered (subset of `ANTIPATTERNS_REGISTER`)

- **AP-WT-F6** self-dispatch — **prime owner at the report layer** (§ 8).
- **AP-Hab-14** god-tier dilution — m12's discipline (no `print!`, no `format!`, no `eprintln!`; emit via `String` return only) keeps the god-tier surface clean.
- **AP-V7-03** verb collapse across Phase A/B boundary — m12's forbidden-verb scan is the canonical Phase A enforcement point at the human surface.

## Atuin trajectory anchor

- `wf-crystallise report` (CLI subcommand; m12 is its renderer).
- `wt-pulse` (uses m12 NdJson output to feed atuin KV `habitat.wt.last_pulse`).

## Watcher class pre-position

- **Class A** — first `wf-crystallise report` invocation post-Genesis.
- **Class C** — confidence-gate refusal — m12's F6 refusal IS Class-C behaviour (safe-path refusal, not failure; observable, not silent).

## Implementation order within Cluster C

m12 **second** — render functions are pure, can be developed against `Vec<WorkflowRunRow>` fixtures without any live database. The Ember gate fixture is authored here, cross-referencing m10. (Vault canonical § Implementation sequence.)

---

*m12 spec authored 2026-05-17 (S1001982) by Command for the Cluster C author wave. Planning-only; HOLD-v2 active; no code until G1-G9 clear.*

> Sister-module bottom anchors: [m7](m7_workflow_runs.md) · [m12](m12_cli_reports.md) · [m13](m13_stcortex_writer.md) · vault [[cluster-C-correlation-output]] · canonical V7 [cluster-C plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-C.md)
