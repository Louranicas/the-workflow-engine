---
title: Runbook 09 — Phase 8 Observability + Production Operations (CROSS-CUTTING continuous)
date: 2026-05-17 (S1001982)
kind: planning-only · operational runbook · cross-cutting continuous (T3 onward through Phase 6)
phase: 8 of 6 (cross-cutting; not sequential)
tracks: 5 — structured logs · Prometheus metrics · OpenTelemetry traces · SLOs+alerts · Grafana dashboards
owner: observability-engineer (Command-role) + Watcher (synthesis consumer) + Luke @ node 0.A
binary_shape: CLI pair (no persistent /metrics scrape; Pushgateway pattern)
authority: Luke @ node 0.A
status: planning-only · HOLD-v2 active · NOT executable until G1-G9 GREEN
---

# Runbook 09 — Phase 8 Observability + Production Operations (CROSS-CUTTING)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../KEYWORDS_20.md]] · sibling [[runbook-06-phase-5-deploy-soak.md]] · [[runbook-08-phase-7-security.md]] · [[runbook-10-cross-cutting.md]]
>
> Source phase doc: [[../../the-workflow-engine-vault/deployment framework/phase-8-observability-operations.md]] (covers 5 observability tracks + cron-scheduled operation + forensic mode + production runbook + hand-off to v1.4)

---

## Overview

Phase 8 is **observability as cross-cutting concern** — it begins at T3 (implement) and runs through every subsequent tier. workflow-trace is a **CLI pair, not a daemon**, so the observability stack is shaped by one architectural premise: **data is produced locally during each run and must be gathered into durable stores BEFORE the process exits**. Five tracks: (1) structured JSON logs with rotating file output for cron; (2) Prometheus metrics via **Pushgateway** (no persistent scrape target — the CLI exits); (3) OpenTelemetry traces via OTLP/gRPC to local collector, **fail-open** if collector unreachable (observability must never block dispatch); (4) SLOs + alerts mapped to Watcher A-I flag classes; (5) three Grafana dashboards (Engine Health / Cluster Performance / Substrate Condition). Critical risks: `wf_m31_selection_weight` cardinality (FNV-1a u32 + overflow bucket cap at 500 series); `wf_m14_lift = -1.0` sentinel for "insufficient data" (NOT zero); Pushgateway (not scrape) because CLI exits after work.

---

## Pre-flight checklist

- Habitat Prometheus reachable at `:9090` (read endpoint) and Pushgateway at `:9091` (push endpoint)
- OTel Collector OPTIONALLY running at `:4317`; if not running, OTLP exporter fails open within 5s — system functions
- Jaeger OPTIONALLY running at `:16686` for trace inspection
- `~/.local/share/workflow-trace/logs/` directory exists (rotating file logs, 30d retention)
- `~/.local/share/workflow-trace/metrics/` directory exists (`.prom` files, 7d retention, Pushgateway retry buffer)
- `~/.local/share/workflow-trace/forensic/` directory exists (forensic-mode logs, 7d retention, 500MB cap)
- `tracing` + `tracing-subscriber` + `metrics` + `metrics-exporter-prometheus` + `opentelemetry` + `tracing-opentelemetry` declared in `Cargo.toml` workspace deps
- `wf-metrics-push` atuin script authored (post-invocation Pushgateway push)
- `wf-pushgateway-cleanup` weekly cron entry planned
- `wf-weekly-summary` atuin script (Sunday 00:00Z; writes JSONL to `agent-cross-talk/`)

---

## Track 1 — Structured Logging

**What:** every log entry carries 8 mandatory fields: `timestamp` (RFC 3339 UTC), `level`, `target` (Rust module path), `span_id`, `trace_id`, `session_id` (`$CLAUDE_SESSION_ID`), `binary` (`wf-crystallise` | `wf-dispatch`), `invocation_id` (UUIDv7). Per-cluster spans add cluster-specific fields (e.g., Cluster G: `workflow_id`, `escape_surface` ordinal only — never raw step content).

**How:** `tracing` facades + `tracing-subscriber` runtime config; JSON to rotating file in cron, ANSI pretty-print in interactive (auto-detect via `is_terminal`).

**Commands:**

```bash
# Default env (set per-binary)
RUST_LOG=workflow_core=info,wf_crystallise=info,wf_dispatch=info

# Interactive smoke (ANSI pretty)
wf-crystallise sweep --window-days 1

# Cron context (JSON to rotating file)
RUST_LOG=workflow_core=info /home/louranicas/.local/bin/wf-crystallise sweep --window-days 1 \
  >> ~/.local/share/workflow-trace/logs/wf-crystallise-$(date +%Y%m%d).jsonl 2>&1

# Housekeeping (run at binary start; ~2ms)
find ~/.local/share/workflow-trace/logs/ -name '*.jsonl' -mtime +30 -delete
```

**Where:** stdout (interactive); rotating file (cron); forensic file (Class B/D/I flag triggered).

**PII handling:** atuin command argv NEVER logged — only `history.id` opaque UUID. m12 strips secret prefixes (`glpat-`, `github_pat_`, `gho_`, `ghp_`, `sk-`, `AKIA`) + replaces `/home/<user>/` with `~/<rel>`. m13 substrate writes restricted to `workflow_id`, `session_id`, `outcome_code`, `ts_ms` only.

**Watcher class:** Class B detection (cluster_complete INFO entries time-stamp cluster boundaries); Class I detection (absence of `module=m42_hebbian_feedback` INFO over N invocations); Class D detection (m13 `event=write_ok` count vs stcortex query result mismatch).

**Failure mode:** F-OBS-T1-1 — log file rotation breaks; disk fills. Detection: weekly disk-space probe. Mitigation: `find … -delete` housekeeping at binary start.

---

## Track 2 — Metrics (Prometheus + Pushgateway)

**What:** 12 metrics with `wf_` prefix to avoid habitat namespace collision. Key gauges and counters:

| Metric | Type | Labels | What |
|---|---|---|---|
| `wf_runs_total` | Counter | `binary`, `outcome` | run completions; outcomes: `pass_verified \| pass \| blocked \| fail \| skipped` |
| `wf_run_duration_seconds` | Histogram | `binary`, `cluster` (A-H) | per-invocation duration |
| `wf_m14_lift` | Gauge | `binary` | `habitat_outcome_lift` value; **`-1.0` sentinel = `None` (insufficient data)** |
| `wf_m31_selection_weight` | Gauge | `workflow_id_hash` (FNV-1a u32) | per-workflow selection probability — **cardinality risk** |
| `wf_m32_dispatch_outcome_total` | Counter | `outcome` | `dispatched \| conductor_down \| verification_stale \| hash_mismatch \| sunset_guard \| cooldown` |
| `wf_m40_emission_total` | Counter | `transport` | `outbox \| http` |
| `wf_m41_lcm_rpc_total` | Counter | `method`, `outcome` | LCM JSON-RPC calls |
| `wf_m42_hebbian_reinforce_total` | Counter | `direction` | `ltp \| ltd` |
| `wf_substrate_ltp_density_observed` | Gauge | (none) | POVM read sampled at invocation start |

**Cardinality risk: `wf_m31_selection_weight`.** FNV-1a u32 hash of `workflow_id` (max 4B distinct; Prometheus struggles > 10K). **Mitigation: 500-series hard cap with overflow bucket** — when distinct `workflow_id_hash` labels would exceed 500, the 500-lowest-weight workflows collapse into a single `workflow_id_hash=overflow` label. Alert fires when overflow label is non-zero. **Total `wf_*` series at steady-state MUST NOT exceed 2,000** (well within habitat Prometheus capacity).

**How:** `metrics` crate (macros) + `metrics-exporter-prometheus` (backend). At process exit, `PrometheusHandle::render()` writes `.prom` text to local file; `wf-metrics-push` atuin script pushes to Pushgateway.

**Commands:**

```bash
# At process exit (in main.rs)
# (Pseudocode — runtime detail in m_metrics module)
# PrometheusHandle::render() -> ~/.local/share/workflow-trace/metrics/wf-<invocation_id>.prom

# Post-invocation push (atuin script wf-metrics-push)
LATEST=$(ls -t ~/.local/share/workflow-trace/metrics/wf-*.prom | head -1)
curl -s --max-time 5 --data-binary @"$LATEST" \
  "http://127.0.0.1:9091/metrics/job/workflow_trace/binary/$(basename "$LATEST" | cut -d'-' -f1)"
# If Pushgateway unreachable: keep .prom file (7d retention; retry on next invocation)

# Weekly cleanup of stale Pushgateway entries (atuin: wf-pushgateway-cleanup)
curl -X DELETE "http://127.0.0.1:9091/metrics/job/workflow_trace"
# (then next push re-establishes)

# Ad-hoc scrape mode (interactive debug)
wf-crystallise --metrics-port 9090 sweep --window-days 7
# Prometheus can scrape :9090 for the invocation's duration
```

**Where:** Pushgateway `:9091` (primary; habitat-local). Fallback: local `.prom` files retained 7d.

**Retention:** Prometheus TSDB raw 15d; rollup to 5-min resolution for 90d. Local `.prom` 7d auto-pruned. Pushgateway stale entries deleted weekly.

**PII handling:** label cardinality and PII are same problem here. `session_id_prefix` = first 8 chars only (correlation OK, identification across sessions NO). `workflow_id_hash` = FNV-1a u32 (matches m4 F11 opaque-ID approach for metrics). NO raw pane labels, command strings, or user-visible text in any label.

**Failure mode:** F-OBS-T2-1 — Pushgateway unreachable; metrics buffer to disk. Mitigation: 7-day retention on `.prom` files; retry on next invocation. F-OBS-T2-2 — `wf_m31_selection_weight` series exceeds 500. Mitigation: overflow bucket; alert when overflow non-zero.

**Watcher integration:** `wf_substrate_ltp_density_observed` is **primary Class I signal** (below 0.010 + 7 invocations without `wf_m42_hebbian_reinforce_total{direction="ltp"}` increment → Class I fires). `wf_m14_lift = -1.0` after `wf_runs_total > 20` = F2 spec compliance failure. `wf_m32_dispatch_outcome_total{outcome="conductor_down"}` = Class B signal.

---

## Track 3 — Distributed Tracing (OpenTelemetry)

**What:** one root span per invocation + child spans per cluster (A→H). Trace context propagated in-memory within process; W3C `traceparent` header injected on m41 LCM HTTP call (only cross-process link).

**How:** `opentelemetry` + `opentelemetry-otlp` + `opentelemetry-sdk` + `tracing-opentelemetry` (ORAC L7 monitoring pattern). OTLP target `127.0.0.1:4317`.

**Sampling:** 100% by default (low invocation volume). Forensic mode forces 100% + TRACE-level attributes. If volume grows, switch to `TraceIdRatioBased(0.1)`.

**Critical fail-open requirement:** if OTel Collector unreachable, OTLP exporter times out within 5s; binary CONTINUES with no-op tracer. **Observability MUST NOT block dispatch.**

**Mandatory 2-second graceful flush window** before process exit:

```rust
// In main.rs (pseudocode)
let _ = tracer_provider.shutdown_with_timeout(Duration::from_secs(2));
```

**Commands:**

```bash
# OTel Collector probe (optional infra)
curl -s --max-time 1 http://127.0.0.1:4317 2>/dev/null
# Exit non-zero = collector down; binary still functional (fail open)

# Jaeger inspection (forensic)
open http://127.0.0.1:16686
# Search by trace_id from log INFO entry; spans render hierarchy A → B → C → ...
```

**Span attribute PII rules:** `cluster_id` opaque (FNV-1a XOR); `redactions_applied` count only (not items); `escape_surface` ordinal only (`ReadOnly` | `HostWrite` | `Network` | `SandboxEscape` | `Destructive`) — never raw step content.

**Retention:** Jaeger in-memory 6h (dev); 7d with Elasticsearch (prod). If neither Jaeger nor Collector deployed, traces silently dropped.

**Failure mode:** F-OBS-T3-1 — Collector down during important invocation; trace lost. Mitigation: log INFO entries carry `trace_id`; reconstruction from log file possible.

**Watcher integration:** trace data is forensic layer (engineers during incidents), not Watcher's primary signal. Exception: Class D flag → Watcher requests trace ID from operator; `trace_id` in INFO entry is lookup key.

---

## Track 4 — SLOs + Alerts

**8 SLOs:**

| SLO | SLI | Threshold | Error budget |
|---|---|---|---|
| **SLO-1** Invocation success | `(pass_verified + pass) / total{outcome!=skipped}` over 7d | ≥ 0.99 | 1% (~1 failed / 100 scheduled) |
| **SLO-2** m20 PrefixSpan latency | `wf_run_duration_seconds{cluster="F"}` p95 | < 5.0s | 5% over threshold |
| **SLO-3** m32 dispatch latency | `wf_run_duration_seconds{cluster="G"}` p95 (excl Conductor RTT) | < 0.5s | 5% |
| **SLO-4** m14 aggregation | `wf_run_duration_seconds{cluster="E"}` p95 | < 30s | 5% |
| **SLO-5** stcortex consumer freshness | registration age | < 30d | `WARN` < 7d remaining; `ERROR` expired |
| **SLO-6** m14 sample compliance | `wf_m14_lift = -1.0` when `wf_runs_total > 20` | **0 violations** | 0 — F2 hard gate |
| **SLO-7** Hebbian movement | `wf_m42_hebbian_reinforce_total{direction="ltp"}` increment ≥ 1 per week | binary | **0 — error budget zero** |
| **SLO-8** Watcher pulse | weekly synthesis WCP notice referencing workflow-trace | 1 per week | weekly ops review |

**7 alerts mapped to Watcher classes:**

| Alert | Trigger | Watcher class | Routing |
|---|---|---|---|
| **ALERT-1** | any `ERROR` log | A | Luke notification immediate (include `invocation_id`, `trace_id`, message) |
| **ALERT-2** | `wf_m14_lift < 0.0` 7 consecutive days | I | synthesis notice; weekly review; no page |
| **ALERT-3** | new `PHASE-B-RESERVATION-NOTICE-*.jsonl` written | E | Watcher reads at next tick; emits WCP; Zen review |
| **ALERT-4** | `wf_m42_hebbian_reinforce_total{ltp}` no increment 5+ invocations | I | `ERROR` log; weekly review (check POVM `:8125/health`; stcortex `:3000`; m9 ns guard) |
| **ALERT-5** | `wf_substrate_ltp_density_observed` drops > 10% w/w | G | Class G immediate WCP; **HALT dispatch**; inspect m42 `fitness_delta` sign-flip |
| **ALERT-6** | workflow `sunset_at` within 7 days | A | `WARN` log; weekly review; no page; Luke decides extend / expire |
| **ALERT-7** | `wf_runs_total` increments while G9 not green | F | Class F immediate; WCP to Command |

**Recording rules:**

```
record: wf:invocation_success_rate_7d
expr: |
  sum(increase(wf_runs_total{outcome=~"pass_verified|pass"}[7d]))
  /
  sum(increase(wf_runs_total{outcome!="skipped"}[7d]))

record: wf:ltp_increment_7d
expr: increase(wf_m42_hebbian_reinforce_total{direction="ltp"}[7d])
```

---

## Track 5 — Grafana Dashboards (3 dashboards; read-only; no inter-panel variables)

### Dashboard 1 — Engine Health

| Panel | Metric | Visualisation |
|---|---|---|
| Invocation success 7d | `wf:invocation_success_rate_7d` | Stat; green > 0.99, amber 0.95-0.99, red < 0.95 |
| Run outcome distribution | `wf_runs_total` by `outcome` | Pie (7d) |
| habitat_outcome_lift | `wf_m14_lift` | Time series; threshold line 0.0; `-1.0` rendered as "insufficient data" annotation |
| Substrate LTP density | `wf_substrate_ltp_density_observed` | Time series; threshold 0.015 (Phase 1) |
| m42 reinforcement rate | `rate(wf_m42_hebbian_reinforce_total[1d])` by `direction` | Time series; LTP + LTD same panel |
| Circuit breaker | LTP stagnation heuristic | Indicator: green incrementing, red stagnant > 5 runs |

### Dashboard 2 — Cluster Performance

| Panel | Metric | Visualisation |
|---|---|---|
| Cluster duration p50/p95/p99 | `wf_run_duration_seconds` histograms by `cluster` | Heatmap (one row per A-H) |
| m20 PrefixSpan p95 | quantile 0.95 on cluster F | Stat; SLO-2 line 5.0s |
| m32 dispatch p95 | quantile 0.95 on cluster G | Stat; SLO-3 line 0.5s |
| m14 aggregation p95 | quantile 0.95 on cluster E | Stat; SLO-4 line 30s |
| m32 dispatch outcomes | `wf_m32_dispatch_outcome_total` by `outcome` | Bar 7d |
| m40 emission by transport | `wf_m40_emission_total` by `transport` | Bar |

### Dashboard 3 — Substrate Condition

| Panel | Metric | Visualisation |
|---|---|---|
| POVM cutover countdown | days until 2026-07-10 | Countdown; amber < 30d, red < 7d |
| Conductor maturity | `wf_m32_dispatch_outcome_total{conductor_down}` rate | Time series |
| Ember §5.1 amendment status | manual annotation (Watcher updates) | Text panel |
| Watcher Class flag history | manual annotation from WCP notices | Table: date · class · description · resolved |
| m31 selection concentration | `max(wf_m31_selection_weight) / sum(...)` | Stat; > 0.5 = monoculture risk (BUG-035-equivalent) |
| Workflow bank sunset | `min(wf_m11_sunset_days_remaining)` | Stat; red < 7 |
| stcortex namespace health | `wf_substrate_ltp_density` vs `wf_m42_reinforce` | Scatter (7d) |

---

## Forensic Mode

**Triggered by:** Watcher Class B/D/I → `WCP_FORENSIC_REQUEST=<invocation_id>` written to agent-cross-talk; operator `--forensic` flag; automatic when SLO-1 < 0.95 for 3 consecutive invocations.

**What forensic mode enables:**

- Log level forced to TRACE for the duration (or next invocation if post-hoc trigger)
- OTel sampling forced 100% (overrides any future reduction)
- Full SQL query text logged at TRACE (sanitised — `$1`, `$2` placeholders, not interpolated values)
- Raw atuin cursor values + row counts at TRACE
- m7 JSONB `consumer_inputs` blob structure at TRACE (fields only, NOT values — may contain sensitive data)
- Forensic file at `~/.local/share/workflow-trace/forensic/wf-forensic-<invocation_id>.jsonl`
- Root OTel span attribute `forensic_mode_active=true`

**Bounded:** 500MB per-file cap; 7d retention; max 3 consecutive forensic invocations before explicit operator re-trigger.

---

## Phase-end gate

Phase 8 is cross-cutting — no discrete end. At each Phase 1-6 transition, observability evidence is bundled:

```bash
# Per-phase observability snapshot
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
PHASE="<1|2A|2B|3|4|5A|5B|5C|6>"
mkdir -p ~/projects/shared-context/wf-obs-${PHASE}-${TS}/

# Logs sample (24h)
cp ~/.local/share/workflow-trace/logs/wf-*-$(date +%Y%m%d).jsonl \
  ~/projects/shared-context/wf-obs-${PHASE}-${TS}/ 2>/dev/null

# Latest .prom files
cp ~/.local/share/workflow-trace/metrics/*.prom \
  ~/projects/shared-context/wf-obs-${PHASE}-${TS}/ 2>/dev/null

# Prometheus SLO snapshot
curl -s "http://127.0.0.1:9090/api/v1/query?query=wf:invocation_success_rate_7d" \
  > ~/projects/shared-context/wf-obs-${PHASE}-${TS}/slo-1-snapshot.json

atuin kv set "workflow_trace.phase8.snapshot.${PHASE}.timestamp" "$TS"
```

---

## Failure modes register

| ID | Trigger | Detection | Mitigation | Track |
|---|---|---|---|---|
| F-OBS-T1-1 | log rotation breaks; disk fills | weekly disk-space probe | `find -mtime +30 -delete` housekeeping at binary start | Logs |
| F-OBS-T2-1 | Pushgateway unreachable | curl --max-time 5 fails | retain `.prom` 7d; retry next invocation | Metrics |
| F-OBS-T2-2 | `wf_m31_selection_weight` > 500 series | Prometheus series count | overflow bucket; alert when non-zero | Metrics |
| F-OBS-T2-3 | `wf_m14_lift = -1.0` after `wf_runs_total > 20` | SLO-6 violation | spec-compliance failure (F2 should have reported lift); investigate F2 gate logic | Metrics |
| F-OBS-T3-1 | OTel Collector down; trace lost | OTLP exporter timeout in 5s | fail open (no-op tracer); reconstruct from log `trace_id` if needed | Traces |
| F-OBS-T4-1 | ALERT-2 sustained (lift below threshold 7d) | recording rule | investigate `wf_m31_selection_weight` monoculture + `wf_m42_reinforce` + `substrate_LTP_density` | SLOs |
| F-OBS-T4-2 | ALERT-7 fires (AP24 violation post-hoc) | `wf_runs_total` increment while G9 not green | Class F flag immediate; halt; investigate | SLOs |
| F-OBS-T5-1 | Dashboard 3 manual annotation rotted (Watcher hasn't updated Ember §5.1 status) | annotation date > 14d old | Watcher synthesis trigger; update or remove | Dashboards |

---

## Watcher flag pre-positioning

| Class | Pre-position trigger | Track involved |
|---|---|---|
| A | ALERT-1 (ERROR log) — gate flip | T1 logs |
| B | `event=cluster_complete` boundary INFO entries; ALERT-6 (sunset approaching) | T1 logs + T4 SLOs |
| C | m33 verification refusals visible in `wf_m32_dispatch_outcome_total{verification_stale}` | T2 metrics |
| **D** | m13 `event=write_ok` count ≠ stcortex query result; Dashboard 3 annotation rot | T1 logs + T5 dashboards |
| E | ALERT-3 (m15 pressure event) | T4 SLOs |
| F | ALERT-7 (G9 violation) | T4 SLOs |
| G | ALERT-5 (substrate_LTP_density drop > 10% w/w) | T4 SLOs |
| H | atuin gap — fewer `cargo`/`wf-*` commands than expected | atuin trajectory (cross-track) |
| **I** | continuous — `wf_substrate_ltp_density_observed < 0.010` + 7 invocations no LTP increment; ALERT-4 | T2 metrics + T4 SLOs |

---

## Atuin trajectory anchors

```bash
# Cross-track observability audit
atuin search "wf-crystallise"           --before 7d
atuin search "wf-metrics-push"          --before 7d
atuin search "curl.*9091"               --before 7d   # Pushgateway pushes
atuin search "rg.*workflow_trace"       --before 7d
atuin scripts run wt-substrate-pulse                  # G5 — Watcher class flag counts
atuin scripts run wt-bridge-check
atuin scripts run wt-soak-pulse
atuin kv get "workflow_trace.phase8.snapshot.5C.timestamp"
```

---

## Sign-off

This runbook is **planning-only** (HOLD-v2). Track 1 (logs) activates at first binary invocation post-Phase 1; tracks 2-5 activate cumulatively through Phase 2A → Phase 5C. Forensic mode is a bounded debugging surface — not always-on. The `wf_m14_lift = -1.0` sentinel and `wf_m31_selection_weight` overflow bucket are load-bearing design choices that distinguish workflow-trace's CLI observability from a typical daemon-service stack.

*Runbook 09 authored 2026-05-17 by Command (V7 optimisation, parallel author). 5 tracks operational. Pushgateway pattern documented. m31 cardinality cap encoded. Sentinel value semantics defined. ~1,790 words. Source: phase-8-observability-operations.md. Sibling: runbook-06 / runbook-08 / runbook-10.*
