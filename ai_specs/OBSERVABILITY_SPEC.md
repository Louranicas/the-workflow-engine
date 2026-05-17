---
title: OBSERVABILITY_SPEC — tracing structured fields + metric names + log levels + span hierarchy
date: 2026-05-17
status: SPEC
patterns: [ME v2 logging.rs, metrics.rs]
---

# OBSERVABILITY_SPEC — workflow-trace

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`cross-cutting/observability.md`](cross-cutting/observability.md) (module-side guidance)

## Stack

- **tracing** crate exclusively for spans + structured logs (no `log` crate direct, no `println!` in production paths per BUG-018).
- **prometheus** (or **metrics** + **metrics-exporter-prometheus**) for counters / gauges / histograms.
- **tracing-subscriber** with `env-filter` for runtime log-level control.

## Span hierarchy

Spans nest by execution context:

```
root (binary entry)
├── ingest_tick
│   ├── m1_atuin_next_page (fields: page_size, last_id)
│   ├── m4_correlate (fields: batch_size, cluster_id)
│   ├── m6_update_ema (fields: session_id)
│   └── m7_insert_run (fields: workflow_id, row_count)
├── propose_tick
│   ├── m20_prefixspan (fields: cluster_id, n_min=20)
│   ├── m22_kmeans (fields: k=5)
│   └── m23_propose (fields: proposal_id, lift_lower)
└── dispatch_tick
    ├── m31_select (fields: candidate_count, selected_id)
    ├── m32_5check
    │   ├── check_1_conductor_health
    │   ├── check_2_verify_ttl
    │   ├── check_3_definition_hash
    │   ├── check_4_sunset
    │   └── check_5_cooldown
    ├── m32_conductor_dispatch (fields: workflow_id, outcome)
    └── cluster_h_fanout
        ├── m40_emit (fields: event_kind)
        ├── m41_route (fields: deploy_shaped: bool)
        └── m42_reinforce (fields: pre_id, post_id, fitness_delta)
```

Per `#[tracing::instrument]` discipline (god-tier rule #13): every public fn declares its span name + structured fields; skips large arguments.

## Metric naming taxonomy

`<module>_<verb>_<noun>_<unit>` convention. Labels are pre-computed (no string concatenation in label values).

### Counters

```
m1_atuin_rows_ingested_total
m2_consumer_events_total{event_kind}
m3_injection_chains_read_total{partition="resolved"|"unresolved"}
m4_cascades_correlated_total
m5_batterns_recorded_total
m6_ema_updates_total{session_type}
m7_workflow_runs_inserted_total
m11_decay_ticks_total
m11_rows_decayed_total
m12_reports_emitted_total{format="text"|"json"}
m13_stcortex_writes_total{outcome="accepted"|"unavailable"|"failed"}
m14_lift_computations_total{evidence="present"|"missing"}
m15_pressure_events_total{kind}
m20_patterns_mined_total
m20_patterns_rejected_total{reason="gap_exceeded"|"below_n_min"}
m21_variants_built_total
m22_clusters_formed_total
m23_proposals_built_total
m23_proposals_rejected_total{reason="lift_missing"|"namespace"|"serde"}
m30_admissions_total{accepted_by}
m30_admissions_refused_total{reason}
m31_selections_total
m31_selections_refused_total{reason="empty_bank"|"substrate_degraded"}
m32_dispatch_total{outcome="pass_verified"|"pass"|"fail"|"blocked"}
m32_dispatch_refused_total{check_failed="1"|"2"|"3"|"4"|"5"}
m33_verifications_total{verdict="pass"|"fail"|"degraded"}
m40_emits_total{outcome="accepted"|"queued"|"failed"}
m40_circuit_open_total
m41_lcm_routes_total{shape="deploy"|"non_deploy"}
m41_circuit_open_total
m42_reinforces_total{outcome="accepted"|"unavailable"|"failed"}
m42_circuit_open_total
m42_namespace_violations_total
m9_namespace_assertions_total{outcome="pass"|"fail"}
```

### Gauges

```
m6_ema_mean{session_type}
m6_ema_variance{session_type}
m11_decay_factor_average
m14_lift_lower_bound{cluster_id}    # opaque cluster_id
m30_bank_size
m30_eligible_count
m31_substrate_pv2_r
m31_substrate_ralph_fitness
m31_substrate_thermal
m32_conductor_breaker_state{state="closed"|"half_open"|"open"}
m40_circuit_breaker_state{state}
m41_circuit_breaker_state{state}
m42_circuit_breaker_state{state}
```

### Histograms

```
m1_atuin_page_duration_seconds (buckets: 0.005,0.01,0.025,0.05,0.1,0.25,0.5,1,2.5,5)
m13_stcortex_write_duration_seconds (buckets per ORAC default)
m20_prefixspan_duration_seconds (buckets up to 8s for 100k stress)
m32_dispatch_duration_seconds
m33_verify_duration_seconds (buckets up to 300s for 4-agent verify)
m40_emit_duration_seconds
m42_reinforce_duration_seconds
```

## Log level discipline

| Level | When | Examples |
|---|---|---|
| `error!` | operational intervention required | `m32 conductor breaker OPEN for 60s`, `m42 outbox disk full` |
| `warn!` | degraded but functional | `m42 circuit half_open`, `m14 evidence missing for cluster_id X`, `m32 dispatch refused (check 2: verification stale)` |
| `info!` | state transitions | `m30 admitted workflow Y`, `m32 dispatched workflow X (outcome: pass_verified)`, `m31 selected X with composite_score 0.73` |
| `debug!` | per-request detail | `m7 inserted row id=N`, `m13 fanned to outbox + RPC` |
| `trace!` | per-step granularity (compile-time disabled in release) | `m20 inner-loop iteration N` |

## Structured fields (no string concatenation)

```rust
// GOOD
tracing::warn!(
    workflow_id = %wf.id,
    check_failed = 2,
    reason = "verification_stale",
    expired_at_ms = result.ttl_expires_at,
    "m32 dispatch refused"
);

// BAD
tracing::warn!("m32 dispatch refused for {} on check 2 because TTL expired at {}",
    wf.id, result.ttl_expires_at);
```

Structured fields enable downstream log aggregation by field name; concatenation defeats it.

## PII / secrets ban

- Never log API keys, PATs, OAuth tokens, user content.
- Workflow IDs (opaque UUIDs) are fine.
- Cluster IDs (opaque FNV hex) are fine.
- Operator usernames (`accepted_by`) are fine — they are the audit identity.

## Per-binary observability setup

```rust
// wf-crystallise / wf-dispatch main.rs
fn init_observability() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into())
        )
        .with_target(true)        // emit module path
        .with_thread_names(true)
        .json()                    // structured JSON output
        .init();

    // metrics exporter — prometheus pull endpoint on :PORT/metrics (feature-gated)
    #[cfg(feature = "monitoring")]
    let _ = metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(([127, 0, 0, 1], 9890))
        .install();
}
```

## ME v2 patterns referenced

- `m1_foundation/logging.rs` — `init_tracing()` setup with `EnvFilter` + JSON output.
- `m1_foundation/metrics.rs` — counter / gauge / histogram registration via `metrics` crate.
- `state.rs` — central state holds `Arc<Metrics>` for hot-path counter access.

## Verify-sync invariants

- **#13** — every async fn has `#[tracing::instrument(skip(..))]`; rg audit returns 100% density per module.
- Metric name stability: `tests/contract/metrics_namespace.rs` lists every metric and asserts it is registered + matches expected name.
- No `println!` outside `tests/`: `rg 'println!' src/ \| grep -v 'tests/'` returns 0.

---

> **Back to:** [`INDEX.md`](INDEX.md) · [`cross-cutting/observability.md`](cross-cutting/observability.md)
