# `wf-daemon` HTTP Shape — Formal Spec (S1005032 Wave-16)

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md)
> **Design doc:** [`ai_docs/WAVE_16_WF_DAEMON_DESIGN_S1005032.md`](../ai_docs/WAVE_16_WF_DAEMON_DESIGN_S1005032.md)
> **Lifecycle:** [`ultramap/WF_DAEMON_LIFECYCLE.md`](../ultramap/WF_DAEMON_LIFECYCLE.md)

## 1. Scope

The HTTP surface exposed by the `wf-daemon` binary in v0.2.1-wave16. This is the **complete public HTTP contract** for the workflow-trace habitat service — one endpoint only.

## 2. Endpoint

### `GET /health`

| Property | Value |
|---|---|
| Port | `8142` (default; override via `WF_DAEMON_PORT`) |
| Bind | `127.0.0.1` (loopback only — internal habitat service) |
| Method | `GET` only |
| Path | `/health` (exact match; trailing slash NOT accepted by axum default) |
| Request body | none (any body is ignored) |
| Response status | `200 OK` (always — see § 3 Liveness contract) |
| Response Content-Type | `text/plain; charset=utf-8` (axum default for `String` body — see § 4) |
| Response body | Wire-aware JSON-shaped string (Wave-17) — 10 fields covering daemon liveness + tick-loop wire status. Computed per-request from shared `WireStats` atomics written by the poller subsystem. |

### Example (Wave-17 wire-aware body)

```
$ curl -s http://127.0.0.1:8142/health | jq
{
  "status": "ok",
  "service": "workflow-trace",
  "port": 8142,
  "tick_count": 7,
  "ok_count": 7,
  "refusal_count": 0,
  "unreachable_count": 0,
  "last_ok_unix_ms": 1779700123456,
  "last_ok_age_ms": 300,
  "refusal_rate_per_kilo": 0
}
```

### Field semantics (Wave-17)

| Field | Type | Meaning |
|---|---|---|
| `status` | `"ok"` | Daemon-process liveness (always returned if handler responds) |
| `service` | `"workflow-trace"` | Service id matching `[[services]] id` in `devenv.toml` |
| `port` | `8142` | Bind port (literal; informational) |
| `tick_count` | `u64` | Total ticks emitted since daemon boot (lifetime cumulative) |
| `ok_count` | `u64` | Subset of `tick_count` that returned `HeartbeatAck` from substrate |
| `refusal_count` | `u64` | Subset that returned `RefusalToken::Unavailable` (engine-imagined + substrate-authored + other) |
| `unreachable_count` | `u64` | Subset that returned `RefusalToken::Unavailable(SubstrateUnreachable)` (transport-layer failure) |
| `last_ok_unix_ms` | `u64` | Unix ms timestamp of the most recent `outcome=ok` ack. `0` = sentinel "never acked". |
| `last_ok_age_ms` | `u64` | Ms since last `outcome=ok` (computed at request time). `u64::MAX` (18446744073709551615) = sentinel "never acked". |
| `refusal_rate_per_kilo` | `u64` | Parts-per-thousand integer rate: `(refusal_count + unreachable_count) * 1000 / tick_count`. `0` = perfect, `1000` = 100% failure. Integer-only to avoid float JSON serialisation. |

**Atomicity note:** the body is computed by 4 independent `AtomicU64::load(Relaxed)` calls, so counter values may straddle increments (a tick could fire between `ok.load()` and `refusals.load()`). The body does not claim snapshot semantics — this is acceptable for an observability endpoint where the operator sees second-scale resolution.

## 3. Liveness contract (Wave-17 amendment)

**The status code (200) certifies daemon-process liveness; the body fields certify wire status.** Both shipped in v0.2.1-wave17 (S1005032), closing the lying-dashboard surface introduced in Wave-16.

- `bridge_health` probe in `habitat-zellij/.../bridge_health.rs` consumes ONLY the 200 status code → dashboard grid `WFE` indicator stays GREEN as long as the daemon process is alive (per other-services convention).
- Operator running `curl :8142/health | jq` sees `tick_count`, `ok_count`, `refusal_rate_per_kilo`, `last_ok_age_ms` → wire-level health is observable without a separate endpoint.
- Future `bridge_health` upgrade can parse `refusal_rate_per_kilo > 500` OR `last_ok_age_ms > 30_000` → render WFE in YELLOW (degraded) without breaking the existing probe contract.

**Why both signals in `/health` instead of a separate endpoint:** Wave-16 originally deferred wire-status to a future `/v1/wire-status` endpoint, treating dashboard-liveness and wire-status as distinct signals not to be conflated. Wave-17 reversed that decision after the post-Wave-16 retro flagged the dashboard-lying surface as a real regression of the AP-V7-13 trap that the V5 trust gate exists to surface. Conclusion: a single endpoint with status-code AND body fields, each layer of consumer reading the field shape that matches its concern, is honest without conflation.

## 4. Body shape note (it returns text/plain JSON-shaped string)

The response body is a JSON-shaped literal but **served as `text/plain`**, not `application/json`. This matches axum's default content-type for `async fn → &'static str` handlers. Consumers (habitat-plugin's `bridge_health` probe, manual `curl`) parse it loosely — only the 200 status code is consumed by the bridge-health logic; the body text is for human/operator observability.

Cross-service convention check:

| Service | `/health` body shape | Content-Type |
|---|---|---|
| ORAC `:8133` | JSON object (`fitness`, `gen`, ...) | `application/json` |
| PV2 `:8132` | JSON object | `application/json` |
| ME `:8180/api/health` | JSON object | `application/json` |
| **WFE `:8142`** | **JSON-shaped string** | **`text/plain`** |

WFE's `/health` is intentionally minimal — the bridge-health probe only consumes the 200 status code, and the body is for `curl :8142/health | jq -r` operator inspection. A future `axum::Json<HealthBody>` upgrade is deferred until the body grows past the 3-field minimum.

## 5. Env overrides

| Env var | Default | Meaning |
|---|---|---|
| `WF_DAEMON_PORT` | `8142` | Bind port for `/health` |
| `WF_POLLER_ENDPOINT` | `http://127.0.0.1:8092/v3/heartbeat` | Substrate URL (SX2 m09 `workflow_trace_participation` receiver) |
| `WF_POLLER_INTERVAL_MS` | `1000` (1 Hz per DD-3 §4.1) | Tick cadence |
| `WF_POLLER_INSTANCE` | `wf-daemon-default` | Instance tag in tracing log |

## 6. NOT exposed (deliberately)

The following endpoints are **NOT** exposed by `wf-daemon` in v0.2.1-wave16 — they remain CLI-only:

| Surface | Exists where | Why not in daemon |
|---|---|---|
| Workflow crystallisation | `wf-crystallise` CLI | Heavy: SQLite WAL read across `atuin.db` + `injection.db` + stcortex consumer; long-running batch; operator-driven (mining + Wilson-CI scoring + Ember rubric gate) |
| Workflow dispatch | `wf-dispatch` CLI | Side-effectful: POSTs to HABITAT-CONDUCTOR `:8141` (real action). Daemon shape would couple every WFE-substrate change to a side-effect surface |
| Poller status / tick counters | tracing log only | Future `/v1/wire-status` endpoint v0.2.3+ |
| Substrate trust query | `wf-poller` log + `RefusalToken` analytics | Future `/v1/substrate-trust` endpoint v0.2.3+ |

Net surface: **1 endpoint, 1 method, 1 status code**. Attack surface is minimal; the daemon is a `/health` probe target + an internal poller, nothing else.

## 7. Bridge-health probe shape (consumer expectation)

The `habitat-zellij` plugin's `bridge_health` module (line 206-211 of `bridge_health.rs`) probes `http://127.0.0.1:8142/health` every `poll_secs * 6` seconds (5s * 6 = 30s default). It marks the service `Up` on **any 2xx response** and `Down` on **any error or non-2xx**. The body content is NOT inspected.

This means WFE's `/health` could return literally any 200 body and the plugin would still show green. The JSON-shaped string is for human-curl inspection, not the plugin.

## 8. References

- Source: `src/bin/wf_daemon.rs::health()` handler
- Design doc: `ai_docs/WAVE_16_WF_DAEMON_DESIGN_S1005032.md`
- Bridge-health probe: `habitat-zellij/crates/habitat-modules/src/bridge_health.rs:206-211`
- CHANGELOG entry: `CHANGELOG.md` `[v0.2.1-wave16]`
- stcortex anchor: ns `workflow_trace_completion_s1004115` mem **19192**
