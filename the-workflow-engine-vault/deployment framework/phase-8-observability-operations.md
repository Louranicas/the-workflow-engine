---
title: "Phase 8 — Observability + Production Operations"
date: 2026-05-17 (S1001982)
kind: deployment-framework-section
status: planning-only · HOLD-v2 active
authority: Luke @ node 0.A
cross-cutting: phases 5-6 + ongoing
emitter: Observability Engineer (Command)
---

# Phase 8 — Observability + Production Operations

> Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]]
>
> Cross-cutting concern: applies from T3 (implement) onward through every subsequent tier. Not a discrete phase you enter and exit — it is the instrumentation layer that makes every other phase observable.

---

## Architectural Premise: Why CLI, Not Service, Shapes This Stack

`workflow-trace` runs as two CLI binaries (`wf-crystallise`, `wf-dispatch`), not a persistent daemon. This has concrete implications for every observability track.

**Consequences for the observability stack:**

- There is no `/metrics` endpoint by default. Prometheus cannot scrape a process that exits after completing its work.
- Logs go to stdout when invoked interactively or to a rotating file when invoked by cron. No syslog socket, no journald unit at Phase A.
- Distributed traces are emitted to an OTel collector via OTLP/gRPC, but only during the invocation window. The collector buffers; the CLI may exit before all spans flush. A 2-second graceful flush period before process exit is mandatory (see Track 3).
- Metrics are cumulative-per-invocation unless the CLI is also given a `--metrics-port 9090` flag for ad-hoc scrape. For scheduled operation, Prometheus pushgateway is the correct integration pattern.
- Atuin trajectory is the single cross-invocation provenance record for every CLI run. Per the GOD_TIER synthesis (S1002029 finding #4): "Atuin is the ONLY cross-tool provenance" — V3, V8, `/scaffold`, and both binaries each have their own SQLite, but the trajectory across all of them lives only in `~/.local/share/atuin/history.db`.

**Operational implication:** observability for `workflow-trace` is a collection problem, not an exposure problem. Data is produced locally during each run and must be gathered into durable stores before the process exits. Design every track around this constraint.

---

## Track 1 — Structured Logging

### What to capture

Every log entry carries a minimum set of structured fields. No message-level string interpolation. Fields are separate keys, not embedded in the `message` string. This is the ME v2 `m1_foundation/logging.rs` gold standard.

**Mandatory fields per log entry:**

| Field | Type | Source |
|---|---|---|
| `timestamp` | RFC 3339 UTC | `tracing-subscriber` time layer |
| `level` | `ERROR \| WARN \| INFO \| DEBUG \| TRACE` | call-site macro |
| `target` | module path (e.g., `workflow_core::m14_evidence_aggregator`) | Rust module path |
| `span_id` | hex u64 | current OTel span (Track 3 correlation) |
| `trace_id` | hex u128 | propagated trace context |
| `session_id` | string | `$CLAUDE_SESSION_ID` env at process start |
| `binary` | `wf-crystallise \| wf-dispatch` | compile-time constant |
| `invocation_id` | UUIDv7 | generated at process start; stable within one run |

**Per-cluster module fields** (added by span enter, not call-site):

- Cluster A: `substrate=atuin\|stcortex\|sqlite`, `cursor_offset`
- Cluster B: `cluster_id` (opaque FNV-1a XOR — never raw pane labels), `battern_id`, `session_range`
- Cluster C: `workflow_id`, `correlation_id`
- Cluster E: `lift_n`, `lift_value`, `ci_half` (when not None)
- Cluster F: `proposal_id`, `algorithm=prefixspan`, `similarity_score`
- Cluster G: `workflow_id`, `escape_surface` (ordinal level only — not raw step content), `conductor_wave`
- Cluster H: `transport=outbox\|http`, `lcm_method`, `fitness_delta_sign`

**Level conventions:**

- `ERROR` — conditions that require immediate human review or that abort an operation. Alert-routed (see Track 4). Examples: m32 Conductor not live; m33 verification expired; m42 circuit breaker OPEN.
- `WARN` — conditions that are recoverable but indicate degraded operation. Examples: m14 lift below threshold for 7d; m9 stcortex consumer freshness approaching 30d; m13 backpressure triggered (LTP/LTD < 0.05).
- `INFO` — normal audit trail. Every invocation start and end; every m15 pressure event; every m32 dispatch attempt; every m14 cycle completion.
- `DEBUG` — forensic detail for post-incident analysis. Every span enter/exit with timing; m20 PrefixSpan candidate counts; m31 selection weight distribution.
- `TRACE` — deep instrumentation for forensic-mode only (see Forensic Mode section). Full SQL queries; raw atuin cursor values; m7 JSONB blobs.

### How to capture

**Crate:** `tracing` (facades) + `tracing-subscriber` (runtime configuration).

**Configuration pattern (adapted from ORAC m7_monitoring):**

```toml
# Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

**Runtime initialisation in `main.rs`:**

The subscriber is initialised once at process start, before the first `async` task is spawned. Level is determined by the `RUST_LOG` environment variable with a project-specific default:

```
RUST_LOG=workflow_core=info,wf_crystallise=info,wf_dispatch=info
```

In containerised or piped invocations: JSON format to stdout, one object per line.
In interactive terminal invocations: ANSI pretty-print format (auto-detected via `is_terminal`).
In cron-scheduled invocations: JSON format to a rotating file at `~/.local/share/workflow-trace/logs/wf-{binary}-{date}.jsonl` (daily rotation; 30-day retention).

**Span construction:** spans are opened at cluster entry points via `tracing::info_span!`. OTel integration (Track 3) attaches the span ID and trace ID automatically. Every log entry emitted inside an active span inherits those IDs.

### Where it goes

| Invocation context | Destination | Format |
|---|---|---|
| Interactive (`wf-crystallise run`) | stdout | ANSI pretty-print |
| Piped or scripted | stdout | JSON (auto-detected) |
| Cron (`atuin scripts run`) | rotating file `~/.local/share/workflow-trace/logs/` | JSON |
| Forensic mode | file `~/.local/share/workflow-trace/forensic/` + stdout | JSON TRACE-level |

### Retention policy

- **Rotating file logs:** 30-day retention. Daily rotation. Files older than 30 days are deleted by a pre-invocation housekeeping step at binary start (adds ~2ms; avoids disk accumulation).
- **Forensic-mode logs:** 7-day retention. Written on Watcher flag Class B/D/I or operator `--forensic` flag. Bounded by a 500MB file size cap; oldest files pruned when cap reached.
- **Raw vs summarised:** all log files are retained raw (JSONL). No aggregation of log data; that is Prometheus's job (Track 2).
- **stcortex integration:** m13 NEVER writes raw log entries to stcortex. Only opaque workflow IDs, outcome codes, and timestamps cross that boundary.

### PII and sensitive data handling

Atuin history commands may contain argument values that include secrets, file paths, or environment variable expansions. Log entries that reference atuin history rows MUST use the atuin `history.id` (opaque UUID) only — never the raw `command` string.

m12 report rendering strips the following before writing to any log, report, or substrate:

- Values matching `[A-Za-z0-9_]{20,}` that start with known secret prefixes (`glpat-`, `github_pat_`, `gho_`, `ghp_`, `sk-`, `AKIA`)
- File paths containing `/home/<username>/` — replaced with `~/<relative>`
- CLAUDE_SESSION_ID values from other sessions — only the current session's ID is logged

m13 substrate writes: only `workflow_id`, `session_id`, `outcome_code`, and `ts_ms` cross to stcortex. No cascade step labels, no pane names, no command strings.

### Cardinality risk

The `target` label (Rust module path) is bounded and stable — no cardinality risk. The fields that introduce unbounded cardinality risk in logs are `span_id` (unique per span, not a label) and `invocation_id` (unique per run, not a label). These are fine as log fields because log storage is append-only JSONL, not a time-series database. Cardinality risk only applies to metrics labels (Track 2).

### Integration with Watcher journal

Watcher reads the rotating log files via `grep` / `jq` at its own cadence. The `level=WARN` and `level=ERROR` lines are the primary signal. Watcher uses these for:

- **Class B detection** (hand-off boundary crossing): `INFO` entries with `event=cluster_complete` across cluster transitions indicate pipeline progress. Watcher timestamps each cluster boundary.
- **Class I detection** (Hebbian silence): absence of `INFO module=m42_hebbian_feedback` entries across multiple invocations. Watcher polls the log for m42 activity; sustained silence over `N` invocations fires Class I.
- **Class D detection** (four-surface drift): discrepancy between what the log reports was written and what stcortex actually contains. Watcher compares `INFO module=m13_stcortex_writer event=write_ok` counts against stcortex query results.

---

## Track 2 — Metrics (Prometheus)

### What to capture

Twelve metrics adapted from the m05_metrics_collector 11D TensorDim façade, extended with `workflow-trace`-specific dimensions. The metric names use the `wf_` prefix to avoid collision with other habitat services in the shared Prometheus namespace.

**Metric catalogue:**

| Metric | Type | Labels | What it measures |
|---|---|---|---|
| `wf_runs_total` | Counter | `binary`, `outcome` | Workflow run completions; `outcome` = `pass_verified \| pass \| blocked \| fail \| skipped` |
| `wf_run_duration_seconds` | Histogram | `binary`, `cluster` | End-to-end duration per invocation; `cluster` = `A \| B \| C \| D \| E \| F \| G \| H` |
| `wf_cascade_correlator_clusters_total` | Counter | `binary` | m4 cascade cluster IDs created (opaque count only) |
| `wf_battern_step_duration_seconds` | Histogram | `step_ordinal` (0-5) | m5 per-step processing time; `step_ordinal` is position (0-5), not the label |
| `wf_context_cost_tokens` | Gauge | `session_id_prefix` (first 8 chars) | m6 current baseline cost estimate |
| `wf_m14_lift` | Gauge | `binary` | m14 current `habitat_outcome_lift` value; -1.0 when `LiftSnapshot.lift` is `None` (sentinel for "insufficient data") |
| `wf_m31_selection_weight` | Gauge | `workflow_id_hash` (FNV-1a u32 of workflow_id; NOT the raw ID) | m31 per-workflow selection probability; one series per workflow |
| `wf_m32_dispatch_outcome_total` | Counter | `outcome` | m32 outcomes: `dispatched \| conductor_down \| verification_stale \| hash_mismatch \| sunset_guard \| cooldown` |
| `wf_m40_emission_total` | Counter | `transport` | m40 emissions: `outbox \| http` |
| `wf_m41_lcm_rpc_total` | Counter | `method`, `outcome` | m41 LCM RPC calls: method = `lcm.loop.create`; outcome = `ok \| err \| skipped` |
| `wf_m42_hebbian_reinforce_total` | Counter | `direction` | m42 POVM reinforce calls: `direction` = `ltp \| ltd` |
| `wf_substrate_ltp_density_observed` | Gauge | (none) | Mirrors POVM's `substrate_LTP_density` read at invocation start; surface for alerting without hitting POVM on every Prometheus scrape |

**Habitat tensor dimensions from m05 (also emitted during invocation):**

The 11D TensorDim values from the parent habitat are sampled at `wf-crystallise` start and end, and emitted as `wf_habitat_tensor_{dim_label}` gauges. This is a point-in-time snapshot of substrate health during the invocation. It is used by Watcher for Class I and Class G flag detection (Hebbian silence and substrate-frame confusion).

### How to capture

**Crate:** `metrics` (macros) + `metrics-exporter-prometheus` (backend). Same stack as m05_metrics_collector. The `MetricsCollector::init()` pattern (Once guard, idempotent) is lifted directly — 95% reuse.

**CLI mode (no `--metrics-port`):**

- Prometheus recorder installed in-memory-only mode (`listen_addr: None`).
- At process exit, `PrometheusHandle::render()` writes the metrics text to `~/.local/share/workflow-trace/metrics/wf-{invocation_id}.prom`.
- A separate `wf-metrics-push` atuin script, run after each crystallise invocation, reads the latest `.prom` file and pushes to Prometheus Pushgateway at `127.0.0.1:9091` (if reachable) with label `job=workflow_trace,binary=wf-crystallise`.
- If Pushgateway is unreachable, the `.prom` file is retained for up to 7 days; the push script retries on next invocation.

**Ad-hoc scrape mode (`--metrics-port 9090`):**

- Bind a temporary HTTP listener on `127.0.0.1:9090` (or operator-specified port). Prometheus can scrape for the duration of a long-running crystallise session.
- Useful for interactive debugging; not the default because most invocations complete in seconds.

**Sampling:** all counters are cumulative across the invocation. Histograms use exponential buckets appropriate for the expected ranges:
- `wf_run_duration_seconds`: `[0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, +Inf]`
- `wf_battern_step_duration_seconds`: `[0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0, +Inf]`

### Where it goes

- Primary: Prometheus Pushgateway at `127.0.0.1:9091` (habitat-local; scraped by the main Prometheus instance on port 9090 from ORAC/SYNTHEX).
- Fallback: local `.prom` files in `~/.local/share/workflow-trace/metrics/` (7-day retention; push retried on next invocation).
- The main Prometheus instance aggregates `workflow-trace` metrics alongside the 13 other habitat services. No separate Prometheus deployment needed.

### Retention policy

- **Prometheus TSDB:** 15 days for raw resolution (default habitat Prometheus retention). After 15 days, rollup to 5-minute resolution for 90 days.
- **Local `.prom` files:** 7-day retention; auto-pruned by `wf-metrics-push` script.
- **Pushgateway state:** stale entries older than 7 days are deleted by a cron job (`wf-pushgateway-cleanup` atuin script, weekly).

### PII and sensitive data handling

Label cardinality mitigation and PII are the same problem here. The `session_id_prefix` label on `wf_context_cost_tokens` uses only the first 8 characters of the session ID — sufficient for correlation, insufficient for identification across sessions. The `workflow_id_hash` label on `wf_m31_selection_weight` uses FNV-1a u32 of the workflow ID, not the raw string — this matches the m4 cluster-ID opaque-ID approach (F11 mitigation principle applied to metrics).

No raw pane labels, command strings, or user-visible text appears in any metric label.

### Cardinality risk and mitigation

| Label | Cardinality | Risk | Mitigation |
|---|---|---|---|
| `outcome` on `wf_runs_total` | 5 values | None | Bounded enum |
| `cluster` on `wf_run_duration_seconds` | 8 values | None | Fixed cluster set |
| `step_ordinal` on `wf_battern_step_duration_seconds` | 6 values | None | Fixed 0-5 |
| `workflow_id_hash` on `wf_m31_selection_weight` | Up to N workflows | **HIGH** if N > 1000 | FNV-1a u32 hash (max 4 billion distinct values, but Prometheus struggles above ~10K series). Enforce maximum 500 tracked workflows: when `wf_m31_selection_weight` would exceed 500 distinct `workflow_id_hash` labels, the 500 lowest-weight workflows are collapsed into a single `workflow_id_hash=overflow` label. Alert when overflow label is non-zero. |
| `session_id_prefix` on `wf_context_cost_tokens` | One per session | Low | One active session at a time per binary instance |

The cardinality contract: **total Prometheus series from `workflow-trace` must not exceed 2,000 at any steady-state**. This is well within the habitat's Prometheus capacity.

### Integration with Watcher journal

- `wf_substrate_ltp_density_observed` is the primary signal for Watcher **Class I** (Hebbian silence). Watcher queries Prometheus at each tick: if this gauge is below 0.010 and `wf_m42_hebbian_reinforce_total{direction="ltp"}` has not incremented in the last 7 invocations, Class I fires.
- `wf_m14_lift` sentinel value `-1.0` (insufficient data) is tracked by Watcher against invocation count. If `wf_runs_total` exceeds 20 and `wf_m14_lift` remains `-1.0`, Watcher flags as a spec-compliance failure (F2 gate should have reported lift, not remained None).
- `wf_m32_dispatch_outcome_total{outcome="conductor_down"}` alerts Watcher to **Class B** signal (Conductor hand-off boundary blocked).

---

## Track 3 — Distributed Tracing (OpenTelemetry)

### What to capture

One root span per binary invocation, with child spans per cluster. Trace context is propagated in memory (no network propagation needed within a single-process invocation). When m41 calls the LCM RPC, the trace context IS propagated via the `traceparent` HTTP header — this is the only cross-process trace link.

**Span hierarchy:**

```
[root: wf_crystallise | wf_dispatch]
  invocation_id, binary, session_id, git_sha

  [cluster_A_ingest]
    [m1_atuin_query]   duration, cursor_offset, row_count
    [m2_stcortex_query] duration, memory_count
    [m3_sqlite_query]  duration, table, row_count

  [cluster_B_observe]
    [m4_cascade_correlate]  cluster_count (opaque)
    [m5_battern_detect]     step_count, unlabelled_count
    [m6_cost_calc]          baseline_tokens, session_range

  [cluster_C_correlate]
    [m7_correlation_insert]     rows_written
    [m12_report_render]         workflow_count, redactions_applied
    [m13_stcortex_write]        rows_attempted, rows_deferred, backpressure_band

  [cluster_E_aggregate]
    [m14_evidence_cycle]        n, lift (null if None), ci_half
    [m15_pressure_scan]         events_detected, notices_written

  [cluster_F_iterate]
    [m20_prefixspan]            patterns_found, candidates_rejected, duration_ms
    [m21_similarity_score]      comparisons, near_miss_count
    [m22_proposal_build]        proposals_accepted, n_below_20_rejected
    [m23_variant_select]        variants_generated, top_k

  [cluster_G_dispatch]    (wf-dispatch binary only)
    [m30_bank_lookup]           bank_size, escape_surface
    [m31_selection]             selected_workflow_id_hash, weight_used
    [m33_verify_check]          ttl_remaining_days, hash_match]
    [m32_dispatch]              outcome, conductor_wave, checks_passed

  [cluster_H_feedback]
    [m40_outbox_write]          transport=outbox, rows_written
    [m40_http_emit]             transport=http, status_code
    [m41_lcm_rpc]               method, outcome, traceparent_propagated=true
    [m42_hebbian_reinforce]     ltp_count, ltd_count, circuit_state
```

### How to capture

**Crates:** `opentelemetry` + `opentelemetry-otlp` + `opentelemetry-sdk` + `tracing-opentelemetry`.

This is the exact stack from ORAC L7 monitoring (`[feature: "monitoring"]`). The integration pattern is:

1. Initialise OTel OTLP exporter targeting `127.0.0.1:4317` (the habitat's OTel collector, if running; fails open with a no-op tracer if unreachable).
2. Wire `tracing-opentelemetry` subscriber layer so that `tracing::info_span!` calls automatically create OTel spans.
3. Span context flows automatically through `async` task boundaries via `tracing`'s task-local context propagation.
4. At process exit: call `tracer_provider.shutdown()` and await up to 2 seconds for span export to flush. This is the mandatory flush window mentioned in the architectural premise.

**Sampling strategy:**

- Default: 100% sampling (all spans recorded). Invocations are infrequent (minutes to hours between cron runs); volume is low; 100% sampling is appropriate and cheap.
- Forensic mode: 100% sampling forced + TRACE-level attribute verbosity enabled.
- If future volume exceeds OTel collector capacity (e.g., interactive mode running crystallise every 30 seconds continuously), switch to `TraceIdRatioBased(0.1)` sampler for non-forensic runs. The root span always has a sampling decision; child spans inherit it.

**Trace context propagation to LCM (m41):**

m41 injects `traceparent` and `tracestate` headers per W3C Trace Context specification into the LCM JSON-RPC HTTP request. The LCM service (if instrumented) can continue the trace. Currently LCM does not emit OTel spans, so the link is one-way — the `workflow-trace` span records the outbound call; LCM does not link back.

### Where it goes

- OTLP/gRPC to `127.0.0.1:4317` — the habitat's OpenTelemetry Collector.
- The OTel Collector is configured (per the ORAC gold standard) to export to Jaeger at `127.0.0.1:16686` for interactive trace inspection.
- If the OTel Collector is not running (Collector is optional infrastructure), the OTLP exporter times out within 5 seconds and the binary continues without tracing. Observability degrades gracefully.

**Collector pipeline configuration (reference snippet, not prescriptive):**

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024

exporters:
  jaeger:
    endpoint: localhost:14250
    tls:
      insecure: true

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [jaeger]
```

### Retention policy

- **Jaeger in-memory storage:** 6-hour trace retention (default for development Jaeger). Sufficient for post-incident analysis within a working session.
- **Jaeger with Elasticsearch backend (production):** 7-day retention for raw spans; no aggregation (spans are not summarised — Prometheus handles that).
- If neither Jaeger nor the OTel Collector is deployed (lightweight habitat setup), traces are silently dropped. The system functions correctly without them; they are supplementary, not load-bearing.

### PII and sensitive data handling

Span attributes follow the same rules as log fields. Cluster B spans record `cluster_id` (opaque FNV-1a XOR) — never raw pane IDs, session labels, or command strings. The `m12_report_render` span records `redactions_applied` (count of items scrubbed) — not the items themselves. The `m13_stcortex_write` span records row counts and backpressure band — not payload content.

The single highest-risk span attribute is `m32_dispatch` `escape_surface`. This records the `EscapeSurfaceProfile` ordinal (`ReadOnly`, `HostWrite`, `Network`, `SandboxEscape`, `Destructive`) — the classification, not the underlying step content. Step content never appears in span attributes.

### Integration with Watcher journal

- Watcher does not directly query Jaeger. Trace data is the forensic layer (for engineers during incidents), not Watcher's primary signal.
- Exception: **Class D flag** (four-surface drift). Watcher may request a Jaeger trace ID from the operator to correlate a Cluster C write discrepancy. The `trace_id` field logged in every INFO entry (Track 1) is the lookup key.
- **Class F flag** (AP24 violation — code emission before G9): if `wf_runs_total` increments before G9 is confirmed green in HOME.md, Watcher flags Class F. Traces provide the invocation timestamp evidence.

---

## Track 4 — Alerts + SLOs

### Service Level Objectives

SLOs for a CLI binary differ from SLOs for a persistent service. The key shift: availability is measured against *scheduled invocation success rate*, not uptime. Latency is measured against *per-invocation operation p95*, not request-per-second p95.

**SLO-1: Invocation success rate**
- **SLI:** `(wf_runs_total{outcome="pass_verified"} + wf_runs_total{outcome="pass"}) / wf_runs_total{} ≥ 0.99` over a rolling 7-day window, excluding invocations with `outcome="skipped"` (deliberate refuse-mode is not a failure).
- **Error budget:** 1% of invocations over 7 days = approximately 1 failed run per 100 scheduled runs.
- **Measurement:** Prometheus Pushgateway query, evaluated by a recording rule run every 5 minutes.

**SLO-2: m20 PrefixSpan cycle latency**
- **SLI:** `wf_run_duration_seconds{cluster="F", quantile="0.95"} < 5.0` for batches of 1,000 cascade clusters.
- **Error budget:** 5% of F-cluster spans may exceed 5 seconds.
- **Measurement:** Prometheus histogram quantile on `wf_run_duration_seconds`.

**SLO-3: m32 dispatch latency**
- **SLI:** `wf_run_duration_seconds{cluster="G", quantile="0.95"} < 0.5` excluding Conductor round-trip time (Conductor RTT is out of `workflow-trace`'s control).
- **Measurement:** Cluster G span duration from the m30 bank lookup start to m32 dispatch decision, not including the HTTP call to Conductor.

**SLO-4: m14 aggregation cycle**
- **SLI:** m14 aggregation cycle completes in under 30 seconds. Measured as the `wf_run_duration_seconds{cluster="E"}` histogram p95 < 30.0.

**SLO-5: stcortex consumer freshness**
- **SLI:** stcortex consumer registration age for the `the_workflow_engine` namespace < 30 days.
- **Measurement:** m9 `namespace_guard` checks registration age at invocation start. `WARN` log when < 7 days remaining; `ERROR` when expired. `wf_substrate_ltp_density_observed` also sampled at start as a freshness proxy.

**SLO-6: m14 sample-size compliance**
- **SLI:** 100% of emitted lift values have `n >= 20`. (F2 hard gate — zero tolerance.)
- **Measurement:** `wf_m14_lift` sentinel `-1.0` should be expected during the first 20 runs of a new deployment. After the warm-up period, any invocation returning `-1.0` when `wf_runs_total > 20` is a violation.

**SLO-7: Hebbian movement**
- **SLI:** `wf_m42_hebbian_reinforce_total{direction="ltp"}` increases by at least 1 per week. (The engine must produce at least one LTP signal per week — Hebbian silence is an SLO breach.)
- **Error budget:** 0 — binary. Either Hebbian movement occurred this week or it did not.

**SLO-8: Watcher pulse**
- **SLI:** Watcher emits at least one synthesis WCP notice per week to `~/projects/shared-context/watcher-notices/` containing a reference to `workflow-trace`.
- **Measurement:** manual check or `ls -la ~/projects/shared-context/watcher-notices/ | grep workflow` in the weekly ops review.

### Alert definitions

Alert routing follows the Watcher flag class system: flag classes with operational impact route to engineer notification; flag classes that are observational route to the Watcher synthesis journal.

**ALERT-1: ERROR-level log**
- Trigger: any `ERROR` log entry from `wf-crystallise` or `wf-dispatch`.
- Watcher flag class: **A** (activation transition — something blocked a gate).
- Routing: email/notification to Luke immediately. Include `invocation_id`, `trace_id`, `error message`.

**ALERT-2: m14 lift below threshold sustained**
- Trigger: `wf_m14_lift < 0.0` for 7 consecutive days (computed on Pushgateway data or via the daily summary script).
- Watcher flag class: **I** (Hebbian silence / value decay).
- Routing: Watcher synthesis notice; weekly ops review; no immediate page.
- Action: check `wf_m31_selection_weight` for monoculture (all weight on one `workflow_id_hash`); check `wf_m42_hebbian_reinforce_total` for circuit-breaker-OPEN condition; check `wf_substrate_ltp_density_observed` for Hebbian regime change.

**ALERT-3: m15 pressure event**
- Trigger: any new `PHASE-B-RESERVATION-NOTICE-*.jsonl` file written to `agent-cross-talk/`.
- Watcher flag class: **E** (ancestor-rhyme risk — scope expansion detected).
- Routing: Watcher reads the notice file at next tick; emits WCP notice to `~/projects/shared-context/watcher-notices/`. Also read by Zen at next review cycle. No automated page; human decision required.
- Action: Luke + Watcher + Zen huddle; spec amendment interview if 3+ pressure events in one session.

**ALERT-4: m42 circuit breaker OPEN sustained**
- Trigger: `wf_m42_hebbian_reinforce_total{direction="ltp"}` has not incremented in 5+ consecutive invocations (circuit breaker OPEN for > 5 minutes of active operation).
- Watcher flag class: **I** (Hebbian silence with identifiable substrate cause).
- Routing: `ERROR` log emitted by m42; Watcher Class I flag; weekly ops review.
- Action: check POVM health at `:8125/health`; check stcortex at `:3000`; check m9 namespace guard freshness.

**ALERT-5: substrate_LTP_density drop**
- Trigger: `wf_substrate_ltp_density_observed` drops more than 10% relative to the previous week's reading.
- Watcher flag class: **G** (substrate-frame confusion — engine may be corrupting substrate).
- Routing: Watcher Class G flag immediately (WCP notice); Luke notification; stop dispatching new workflows until cause is established.
- Action: halt `wf-dispatch` invocations; run `wf-crystallise` in read-only mode; inspect m42 `fitness_delta` values in stcortex for sign-flip bugs.

**ALERT-6: m11 sunset approaching**
- Trigger: any workflow in the m30 bank has `sunset_at` within 7 days.
- Watcher flag class: **A** (activation transition — workflow nearing end-of-life).
- Routing: `WARN` log from m11; weekly ops review. No page.
- Action: Luke decides whether to extend or let expire. No automated extension (would violate human-gate P0 constraint).

**ALERT-7: G9 pre-genesis gate violation**
- Trigger: `wf_runs_total` increments while HOME.md still shows G9 as not-green.
- Watcher flag class: **F** (AP24 violation — code running before genesis gate).
- Routing: Watcher Class F flag immediately; WCP notice to Command.

### PromQL recording rules

To avoid ad-hoc Pushgateway scrape latency in alert evaluation, the following recording rules are maintained in the habitat Prometheus:

```
# SLO-1: 7-day rolling success rate
record: wf:invocation_success_rate_7d
expr: |
  (
    sum(increase(wf_runs_total{outcome=~"pass_verified|pass"}[7d]))
    /
    sum(increase(wf_runs_total{outcome!="skipped"}[7d]))
  )

# SLO-7: Hebbian movement (weekly increment)
record: wf:ltp_increment_7d
expr: increase(wf_m42_hebbian_reinforce_total{direction="ltp"}[7d])
```

---

## Track 5 — Dashboards (Grafana or Equivalent)

Three dashboards. Each is self-contained; no inter-dashboard variables. All dashboards are read-only (no write-back to Prometheus or stcortex from the dashboard layer).

### Dashboard 1: Engine Health

**Purpose:** daily operational summary; answers "is the engine producing value?"

**Panels:**

| Panel | Metric | Visualisation |
|---|---|---|
| Invocation success rate (7d) | `wf:invocation_success_rate_7d` | Stat panel; green > 0.99, amber 0.95-0.99, red < 0.95 |
| Run outcome distribution | `wf_runs_total` by `outcome` | Pie chart; last 7 days |
| habitat_outcome_lift over time | `wf_m14_lift` | Time series; horizontal line at 0.0 (threshold); sentinel -1.0 shown as "insufficient data" annotation |
| Substrate LTP density | `wf_substrate_ltp_density_observed` | Time series; horizontal line at 0.015 (Phase 1 target) |
| m42 Hebbian reinforcement rate | `rate(wf_m42_hebbian_reinforce_total[1d])` by `direction` | Time series; LTP and LTD on same panel |
| Circuit breaker state | `wf_m42_hebbian_reinforce_total{direction="ltp"}` stagnation heuristic | Alert-state indicator: green = incrementing, red = stagnant > 5 runs |

### Dashboard 2: Cluster Performance

**Purpose:** latency diagnosis; answers "which cluster is slow today?"

**Panels:**

| Panel | Metric | Visualisation |
|---|---|---|
| Cluster duration p50/p95/p99 | `wf_run_duration_seconds` histograms by `cluster` | Heatmap; one row per cluster (A-H) |
| m20 PrefixSpan p95 | `histogram_quantile(0.95, wf_run_duration_seconds{cluster="F"})` | Stat; SLO-2 threshold line at 5.0s |
| m32 dispatch p95 | `histogram_quantile(0.95, wf_run_duration_seconds{cluster="G"})` | Stat; SLO-3 threshold line at 0.5s |
| m14 aggregation p95 | `histogram_quantile(0.95, wf_run_duration_seconds{cluster="E"})` | Stat; SLO-4 threshold at 30s |
| m32 dispatch outcome distribution | `wf_m32_dispatch_outcome_total` by `outcome` | Bar chart; last 7 days |
| m40 emission by transport | `wf_m40_emission_total` by `transport` | Bar chart; outbox vs http |

### Dashboard 3: Substrate Condition

**Purpose:** substrate health and long-range reliability; answers "is the substrate ready for the next phase?"

**Panels:**

| Panel | Metric | Visualisation |
|---|---|---|
| POVM cutover countdown | Days until 2026-07-10 | Countdown stat; amber < 30d, red < 7d |
| Conductor maturity | `wf_m32_dispatch_outcome_total{outcome="conductor_down"}` rate | Time series; high rate = Conductor not ready |
| Ember §5.1 amendment status | Manual annotation (text panel updated by Watcher) | Text panel: "Held / Under Review / PASS" |
| Watcher Class flag history | Manual annotation from Watcher WCP notices | Table: date, class, description, resolved |
| m31 selection concentration | `max(wf_m31_selection_weight) / sum(wf_m31_selection_weight)` | Stat; > 0.5 = monoculture risk (RALPH anti-pattern BUG-035 equivalent) |
| Workflow bank sunset status | `min(wf_m11_sunset_days_remaining)` (gauge to be added) | Stat; red < 7 |
| stcortex namespace health | `wf_substrate_ltp_density_observed` against `wf_m42_hebbian_reinforce_total` correlation | Scatter plot (7-day sample) |

---

## Cron-Scheduled Operation Observability

`workflow-trace` is expected to run on a cron schedule (via atuin scripts or systemd timer). The observability posture for scheduled operation differs from interactive operation.

**Per-invocation structured log entry:** every crystallise cron run emits an `INFO` entry at both start and end:

```json
{"level":"INFO","event":"invocation_start","invocation_id":"01JVTXXXXXXXXXX","binary":"wf-crystallise","session_id":"cron","trigger":"atuin_script:wf-daily-crystallise","timestamp":"2026-05-17T03:00:00Z"}
{"level":"INFO","event":"invocation_end","invocation_id":"01JVTXXXXXXXXXX","binary":"wf-crystallise","outcome":"pass","duration_ms":4230,"timestamp":"2026-05-17T03:00:04.23Z"}
```

The `trigger=atuin_script:wf-daily-crystallise` field identifies which atuin script invoked the binary. This is the cross-provenance link between atuin trajectory and the run log.

**Atuin trajectory capture:** atuin automatically records every invocation of `wf-crystallise` and `wf-dispatch` in `~/.local/share/atuin/history.db`. The atuin `history.id` for each cron run is logged in the `invocation_start` entry (via `$ATUIN_HISTORY_ID` env if set by the atuin hook). This is the cross-tool provenance link.

**Metrics aggregation:** the `wf-metrics-push` atuin script, triggered after each crystallise run, pushes the per-invocation metrics to Pushgateway. Daily Prometheus recording rules aggregate across all invocations. A weekly summary recording rule computes 7-day rolling averages for SLO evaluation.

**Weekly summary to agent-cross-talk:** a separate `wf-weekly-summary` atuin script runs weekly (cron Sunday 00:00Z). It queries Prometheus for the 7-day SLO values and writes a structured JSONL summary to `~/projects/shared-context/agent-cross-talk/wf-weekly-summary-{date}.jsonl`. Watcher reads this at its next tick and includes it in the weekly synthesis WCP notice (SLO-8 satisfied).

---

## Forensic Mode

Triggered by:
- Watcher Class B, D, or I flag (Watcher writes `WCP_FORENSIC_REQUEST=<invocation_id>` to agent-cross-talk)
- Operator `--forensic` flag on `wf-crystallise` or `wf-dispatch`
- Automatic trigger when SLO-1 invocation success rate drops below 0.95 for 3 consecutive invocations

**What forensic mode enables:**

- Logging level forced to TRACE for the duration of the forensic invocation (or for the next invocation, if triggered by Watcher flag post-hoc).
- OTel sampling rate forced to 100% (already the default, but forensic mode overrides any future sampling reduction).
- Full SQL query text logged at TRACE level (sanitised: parameter bindings as `$1`, `$2`, etc. — not interpolated values).
- Raw atuin cursor values and row counts logged at TRACE level.
- m7 JSONB `consumer_inputs` blob structure logged at TRACE level (fields only, not values — values may contain sensitive data).
- Forensic log written to `~/.local/share/workflow-trace/forensic/wf-forensic-{invocation_id}.jsonl` in addition to normal log destination.
- A `forensic_mode_active=true` span attribute added to the root OTel span.

**Bounding forensic mode:** forensic log files have a 500MB cap (per-file) and a 7-day retention window. When the cap is reached, the oldest forensic file is deleted. Forensic mode does not run continuously; it is triggered per-invocation or for a bounded window (maximum 3 consecutive forensic invocations before requiring explicit operator re-trigger).

---

## Production Runbook

### Incident: m14 lift dropped suddenly

**Symptoms:** `wf_m14_lift` drops to near-zero or negative over 1-2 days. SLO-2 alert fires.

**Investigation sequence:**

1. Check `wf_m31_selection_weight` distribution. If `max / sum > 0.5`: monoculture. One workflow is dominating selection. Check m31's diversity enforcement (10-gen cooldown, 50% mono-parameter rejection). This is the BUG-035-equivalent for `workflow-trace`. Resolution: temporarily increase diversity weight `δ` in m31's composite score formula.

2. Check `wf_m42_hebbian_reinforce_total`. If circuit breaker is OPEN (`wf_m42_hebbian_reinforce_total` not incrementing): POVM or stcortex is down. Check `:8125/health` and `:3000`. Substrate feedback is broken; m31 selection weights are stale. The outbox (`wf_m40_emission_total{transport="outbox"}`) should still be incrementing. Resolution: restore substrate connectivity; outbox will drain automatically on next m40 HTTP emit attempt.

3. Check `wf_substrate_ltp_density_observed`. If this has dropped more than 10% relative to last week: possible Hebbian regime change (LTD dominance deepening). This is the pre-existing substrate condition (LTP/LTD = 0.043, 35x below target). The engine is shipping on LTD-dominant substrate. Resolution: if `wf_substrate_ltp_density_observed` is actively declining during `workflow-trace` operation, pause dispatches and investigate whether m42 `fitness_delta` values have a sign error (PassVerified should be +0.25, not -0.25). Cross-reference: Watcher Class I flag, GOD_TIER_CONSOLIDATION §V.

4. Check m14 rolling window. If `wf_runs_total` has been very low recently (< 20 runs in the window): `LiftSnapshot.lift` will be `None` (sentinel `-1.0`). This is expected behaviour, not a bug. Lift will return when n >= 20.

### Incident: m32 dispatch failing

**Symptoms:** `wf_m32_dispatch_outcome_total{outcome="conductor_down"}` elevated. Workflows are not being dispatched.

**Investigation sequence:**

1. Check Conductor health: `curl -s http://localhost:8141/health` (HABITAT-CONDUCTOR Weaver). If 503 or connection refused: Conductor is not live. Per P0 #3, `wf-dispatch` must not bypass Conductor. `ERROR` log emitted; `DispatchError::ConductorDispatchDisabled` returned. Resolution: Luke starts Conductor from terminal (`devenv start weaver`).

2. Check m33 verification freshness: query the local SQLite database for the workflow's `last_verified_at` and compare against the 7-day TTL. If stale: m33 requires re-verification by the 4-agent gate (Security + Performance + SilentFailure + Zen). Resolution: run `wf-dispatch verify <workflow_id>` to trigger the verification pipeline.

3. Check m33 definition hash: m32 recomputes FNV-1a hash of `steps_json` at dispatch time and compares against m33's stored `definition_hash`. If mismatch: the workflow definition has drifted since last verification. Resolution: the workflow must be re-verified before dispatch. This is correct protective behaviour.

4. Check Cipher escape-surface alerts: if the workflow's `EscapeSurfaceProfile` is `SandboxEscape` or `Destructive`, the m32 pre-dispatch display step shows the profile banner. If Luke dismissed the banner without confirming, check whether the prompt was presented correctly. This is a UX issue, not a code bug.

### Incident: m15 pressure event flood

**Symptoms:** multiple `PHASE-B-RESERVATION-NOTICE-*.jsonl` files appearing in `agent-cross-talk/` within a short session.

**Definition of flood:** 3+ pressure events in a single session, or 5+ in a week.

**Response protocol:**

1. Luke reads the notice files (or Watcher summarises in WCP notice). Identify the pattern: is this one agent repeatedly proposing the same forbidden verb, or multiple distinct agents proposing different verbs?

2. Watcher prepares a flag summary (Class E — ancestor-rhyme risk). The m15 notices are the evidence.

3. Luke + Watcher + Zen huddle. Classify:
   - **False positive pressure:** the proposed feature is actually within the chartered verb-set but was incorrectly detected. Resolution: update m15 detection heuristic.
   - **Legitimate scope expansion need:** the engine genuinely needs the proposed capability. Resolution: spec amendment interview via ACP protocol. Update genesis prompt v1.3 patch (subject to G7 Zen audit before activation).
   - **Ancestor-rhyme escalation:** the pressure pattern matches the historical death pattern (planning-sprawl expansion). Resolution: maintain charter; Fossil's evidence-based scope discipline (partially waived) should be re-applied to this decision.

4. If amendment proceeds: Watcher timestamps the G7 audit verdict verbatim (Class A flag). This is the highest-leverage moment in the pipeline per GOD_TIER synthesis.

---

## Hand-off to v1.4 / Next Codebase

Observability artefacts are the durable output of the `workflow-trace` deployment. Code changes; metrics and logs are the evidence of what the code did.

**What persists:**

- **Prometheus TSDB:** 90-day rollup data survives codebase upgrades. The `wf_m14_lift` time series is the longitudinal record of whether the engine produced habitat value.
- **Log archive:** 30-day rotating JSONL. After 30 days, the structured log data is gone unless explicitly archived. If v1.4 genesis requires historical context, archive the last 90 days before upgrading.
- **Watcher synthesis notices:** WCP notices in `~/projects/shared-context/watcher-notices/` are the durable synthesis. Watcher's synthesis is the qualitative complement to Prometheus's quantitative record. The synthesis notices survive indefinitely (they are markdown files, not time-series).
- **stcortex pathways:** m42 POVM/stcortex pathway weights in the `workflow_trace_*` namespace persist across v1.4. These are the Hebbian memory of which workflows were reinforced. v1.4 must adopt the same AP30 namespace prefix to continue the learning loop rather than starting fresh.
- **Atuin trajectory:** the full atuin history of every `wf-crystallise` and `wf-dispatch` invocation survives indefinitely in `~/.local/share/atuin/history.db`. This is the primary archaeological record for v1.4's gap analysis.

**What the next codebase should read before genesis:**

1. `wf_m14_lift` Prometheus history — what lift trajectory did the engine achieve?
2. `wf_m31_selection_weight` history — did monoculture occur? Which `workflow_id_hash` dominated?
3. Watcher synthesis notices — what did the observing mind conclude?
4. m15 PHASE-B-RESERVATION-NOTICE archive — what scope pressures accumulated? Were any legitimate?
5. stcortex `workflow_trace_*` pathway weights — which workflows have the strongest Hebbian reinforcement?

The observability stack is the institutional memory that prevents v1.4 from inheriting v1.3's blind spots. The Watcher's synthesis is the bridge between quantitative metrics and qualitative understanding. Both are required.

---

## Summary: Observability Stack Decision Table

| Decision | Choice | Rationale |
|---|---|---|
| Metrics transport (CLI) | Pushgateway | CLI exits; no persistent scrape target |
| Log format | JSON JSONL | Machine-readable first (habitat substrate preference — `reflection_jsonl_substrate_preference.md`) |
| Trace context propagation | W3C `traceparent` header | Standards-based; LCM-compatible future link |
| OTel failure mode | Fail open (no-op tracer) | Observability must not block dispatch |
| Cardinality guard | FNV-1a u32 hash + 500-series cap | Bounded label space; consistent with m4 F11 opaque-ID mitigation |
| Forensic trigger | Watcher Class B/D/I or operator flag | Automated triggers avoid always-on TRACE cost |
| Watcher integration | Log file polling + Prometheus query | Watcher is read-only observer; no push path from metrics to Watcher |
| Dashboard tool | Grafana (or equivalent) | Three dashboards; no write-back |
| Retention (logs) | 30 days raw | Standard habitat retention; Prometheus carries the long record |
| Retention (traces) | 7 days (with Elasticsearch) | Forensic window; Prometheus carries the aggregate |

---

*Phase 8 authored 2026-05-17 (S1001982) · observability engineer role · planning-only · HOLD-v2 active · gates G1-G9 required before any implementation*
