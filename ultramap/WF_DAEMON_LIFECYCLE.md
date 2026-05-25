# `wf-daemon` Lifecycle / Control Flow (S1005032 Wave-16)

> **Back to:** [`README.md`](README.md) · [`ULTRAMAP.md`](ULTRAMAP.md) · companion pipelines: [`WF_CRYSTALLISE_PIPELINE.md`](WF_CRYSTALLISE_PIPELINE.md) · [`WF_DISPATCH_PIPELINE.md`](WF_DISPATCH_PIPELINE.md)
> **Design doc:** [`ai_docs/WAVE_16_WF_DAEMON_DESIGN_S1005032.md`](../ai_docs/WAVE_16_WF_DAEMON_DESIGN_S1005032.md)
> **HTTP shape:** [`ai_specs/WF_DAEMON_HTTP_SHAPE.md`](../ai_specs/WF_DAEMON_HTTP_SHAPE.md)

## Process lifecycle

```
devenv start workflow-trace (auto_start=true, auto_restart=true)
   │ launches ./bin/wf-daemon
   ▼
wf-daemon process
   ├─ tokio main async runtime
   │
   ├─ Task A: spawn_blocking(poller_subsystem)
   │     │ owns std::thread::sleep + reqwest blocking calls
   │     ├─ boot_id = wf-daemon-{unix_ms}
   │     ├─ SubstrateTrust seeded: SynthexV2 = Live, score 0.9
   │     ├─ HeartbeatTransport pointing at :8092/v3/heartbeat
   │     ├─ DriftDetector with 2 WallClockSamplers (AtuinCheckpoint + M11Recency)
   │     │   + SkewEnvelope max_skew_ms=1000 + AlertBudget 60_000
   │     └─ loop {
   │          cycle += 1
   │          tick_and_emit() returns Result<HeartbeatAck, RefusalToken>
   │          tracing log with kind_preview=wf_daemon_tick + outcome label
   │          std::thread::sleep(interval_ms = 1000ms default)
   │        }
   │
   └─ Task B: axum::serve(TcpListener::bind("127.0.0.1:8142"))
         └─ Router with route /health returning 200 + JSON-shaped body
              {"status":"ok","service":"workflow-trace","port":8142}
```

## Concurrency model

| Task | Runtime | Reason |
|---|---|---|
| axum::serve (/health) | tokio multi-thread runtime | async by design |
| poller_subsystem (per-tick heartbeat) | tokio::task::spawn_blocking | uses reqwest::blocking + std::thread::sleep (AP38 — never put sync HTTP in tokio::spawn) |
| Per-tick HTTP POST | inside spawn_blocking | inherits blocking-pool isolation |
| Sleep between ticks | std::thread::sleep (NOT tokio::time::sleep) | inside spawn_blocking task |

## Failure-mode taxonomy (per tick_and_emit result)

| Result | Tracing kind | Meaning |
|---|---|---|
| Ok(ack) | outcome=ok | substrate accepted heartbeat; cycle_acked returned |
| Err(EngineImagined) | outcome=engine_imagined | V5 gate short-circuit; no HTTP call attempted |
| Err(SubstrateUnreachable) | outcome=substrate_unreachable | transport/r13/port-down/501-stub-from-old-binary |
| Err(SubstrateAuthored) | outcome=substrate_authored_refusal | substrate explicit 503 + reason |
| Err(other) | outcome=unexpected_refusal | unrecognised RefusalToken variant |

TickCounters {ok, refusals, unreachable} are accumulated for daemon lifetime and logged at every tick.

## Bind failure path

If TcpListener::bind 127.0.0.1:8142 fails (port collision or permission denied):
- tracing::error fires with kind_preview=wf_daemon_bind_fail
- Process exits with ExitCode::FAILURE
- devenv auto_restart=true retries the start
- If port stays bound by a competing process, daemon crash-loops forever

This is the failure mode that would have fired had we kept port 8141 (HABITAT-CONDUCTOR's reserved port). Re-port to 8142 (verified free) avoids the collision.

## Cross-references

- Source: src/bin/wf_daemon.rs
- Library tick logic: workflow_core::m16_substrate_drift_canary::transport::tick_and_emit
- V5 trust gate: workflow_core::substrate_trust::SubstrateTrust
- Refusal taxonomy: workflow_core::refusal_token::RefusalToken
- Sibling pipelines: ultramap/WF_CRYSTALLISE_PIPELINE.md, ultramap/WF_DISPATCH_PIPELINE.md
- Design rationale: ai_docs/WAVE_16_WF_DAEMON_DESIGN_S1005032.md
- HTTP shape spec: ai_specs/WF_DAEMON_HTTP_SHAPE.md
- CHANGELOG: CHANGELOG.md [v0.2.1-wave16]
- stcortex anchor: ns workflow_trace_completion_s1004115 mem 19192
