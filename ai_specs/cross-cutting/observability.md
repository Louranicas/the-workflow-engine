---
title: cross-cutting/observability — tracing + metrics + logs (module-side guidance)
date: 2026-05-17
status: SPEC
axes: [tracing, metrics, logs]
consolidates: OBSERVABILITY_SPEC.md (per-module guidance)
---

# Observability — Module-Side Guidance

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../OBSERVABILITY_SPEC.md`](../OBSERVABILITY_SPEC.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md)

## Purpose

This axis spec is the **module-side translation** of [`../OBSERVABILITY_SPEC.md`](../OBSERVABILITY_SPEC.md) — what every module MUST do for tracing, metrics, and logs to remain consistent across the engine. The full envelope shape (tracing span hierarchy, metric naming taxonomy, log-level discipline) lives in the root spec; here is what to do at the source-file level.

## tracing — per-module discipline

Every module that exposes async or top-level public functions:

```rust
#[tracing::instrument(skip(self, payload), fields(module = "m20", workflow_id = %wf.id))]
pub async fn build(&self, wf: &Workflow) -> Result<Output, MyError> {
    tracing::debug!("entered build path");
    // ...
}
```

- **Skip large arguments** (`skip(payload)`) — avoid printing a 100-line struct on every span.
- **Structured fields preferred** — `event_kind = "dispatch_refused"`, `reason = "conductor_unreachable"` over string concatenation.
- **Module field** — `fields(module = "m20")` lets downstream log aggregators filter by module name.
- **Workflow / cluster IDs** — `%wf.id` (Display) is fine for opaque IDs; never log human-meaningful labels (F11 mitigation).

## Metric naming taxonomy

Counters, gauges, histograms follow `<verb>_<noun>_<unit>`:

- `m32_dispatch_total{outcome="pass_verified"}` — counter
- `m32_dispatch_duration_seconds` — histogram (buckets per ORAC standard)
- `m42_circuit_breaker_state{module="m42"}` — gauge (0=closed, 1=half_open, 2=open)
- `m14_lift_evidence_missing_total` — counter (each `Ok(None)` increment)
- `m23_proposal_built_total` — counter (each successful `ProposalBuilder::build()`)

Metric names are CONTRACT-BINDING across the prometheus surface; renaming requires a deprecation cycle and a metric-name contract test under `tests/contract/metrics_namespace.rs`.

## Log levels — when to use which

| Level | Use case | Example |
|---|---|---|
| `error!` | operational intervention required | substrate down + outbox full |
| `warn!` | degraded but functional | breaker OPEN; using cached `VerifyResult` past 6d |
| `info!` | state transitions | m32 dispatched workflow X; m30 admitted entry Y |
| `debug!` | per-request detail | "computed FNV hash 0xabc..." |
| `trace!` | per-step granularity | compile-time disabled in release |

## Anti-patterns

- **`println!()` in production paths.** SIGPIPE kills daemons (BUG-018). Use `tracing::info!`.
- **`format!()` of a large struct on every span.** Expensive. Use `?value` (Debug) sparingly.
- **PII / secrets in logs.** Never log API keys, PATs, user content.
- **String concatenation in metric labels.** Pre-compute labels to a stable set; cardinality explosion otherwise.

## Per-cluster observability commitments

| Cluster | What it exports |
|---|---|
| A | `m{1,2,3}_ingest_rows_total`, `m1_atuin_busy_timeouts_total`, `m2_consumer_reconnects_total` |
| B | `m{4,5,6}_observations_total`, `m6_ema_convergence_n_observed` |
| C | `m7_workflow_runs_total`, `m13_stcortex_writes_total{outcome}`, `m12_reports_emitted_total` |
| D | `m11_decay_factor_avg` (gauge), `m9_namespace_violations_total` |
| E | `m14_lift_evidence_missing_total`, `m15_pressure_events_total{kind}` |
| F | `m20_patterns_mined_total`, `m23_proposal_built_total`, `m23_proposal_rejected_total{reason}` |
| G | `m30_admissions_total{accepted_by}`, `m31_selections_total`, `m32_dispatch_total{outcome}`, `m33_verifications_total{verdict}` |
| H | `m40_emits_total{outcome}`, `m41_lcm_routes_total{shape}`, `m42_reinforces_total{outcome}`, `*_circuit_open_total` for all three |

## Verify-sync invariants

- **#13** — every async fn has `#[tracing::instrument(skip(..))]`.
- Per-module metrics naming validated by `tests/contract/metrics_namespace.rs`.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../OBSERVABILITY_SPEC.md`](../OBSERVABILITY_SPEC.md)
