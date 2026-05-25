# Wave-16 — `wf-daemon` Habitat-Service Design (S1005032, 2026-05-25)

> **Back to:** [`CLAUDE.md`](../CLAUDE.md) · [`CLAUDE.local.md`](../CLAUDE.local.md) · [`CHANGELOG.md`](../CHANGELOG.md) `[v0.2.1-wave16]`
> **HTTP shape spec:** [`ai_specs/WF_DAEMON_HTTP_SHAPE.md`](../ai_specs/WF_DAEMON_HTTP_SHAPE.md)
> **Lifecycle / control-flow:** [`ultramap/WF_DAEMON_LIFECYCLE.md`](../ultramap/WF_DAEMON_LIFECYCLE.md)
> **Vault mirror:** [[Wave-16 — wf-daemon Habitat Service Shape S1005032]]

## 1. Why this exists

Per Luke's 2026-05-25 directive ("ensure the workflow-engine is fully and comprehensively wired into synthex … verify all binaries have been build and align with all conventions and systems of the zellij habitat services"), workflow-trace was structurally complete but operationally a **set of three CLI binaries** (`wf-crystallise` + `wf-dispatch` + `wf-poller`). It was not a habitat-managed service:

- No entry in `~/.config/devenv/devenv.toml`
- No daemon-shaped binary with HTTP `/health` endpoint
- Not visible in the Zellij `habitat-plugin.wasm` 13-service grid
- The Wave-15 `wf-poller` ran continuously but was operator-launched (no auto-start, no auto-restart)

Wave-16 adds **`wf-daemon`** — the system-managed shape — so workflow-trace becomes a first-class peer alongside V3 / Nerve / TL / SX / V8 / VMS / POVM / RM / PV2 / ORAC / Inj / ME / PSw in the habitat.

## 2. Service shape (canonical pattern)

Survey of the habitat-service convention (`~/.config/devenv/devenv.toml` + `habitat-zellij/crates/habitat-modules/src/bridge_health.rs`):

| Convention | Source of truth | wf-daemon compliance |
|---|---|---|
| `[[services]]` entry in `~/.config/devenv/devenv.toml` | `id`, `name`, `description`, `working_dir`, `command`, `args`, `auto_start`, `auto_restart` | ✓ `id = "workflow-trace"`, all 7 fields set |
| Daemon-shaped binary (long-running, not invoke-and-exit) | `./bin/<name>` (project) or `/home/louranicas/.local/bin/<name>` | ✓ `./bin/wf-daemon`, axum `serve` loop |
| HTTP `GET /health` endpoint on a bound port | `bridge_health.rs` PROBE_PATHS line — all habitat services use `/health` except ME `:8180/api/health` | ✓ `GET /health` on `:8142` returning 200 JSON |
| Registered in `bridge_health.rs` SERVICES list | Lines 26-41 of `bridge_health.rs` — port + 3-char tag | ✓ `(8142, "WFE")` between Inj and ME |
| Registered in `bridge_health.rs` PROBE_PATHS list | Lines 206-211 — port + path | ✓ `(8142, "/health")` |
| `habitat-plugin.wasm` rebuilt + redeployed to `~/.config/zellij/plugins/` | `habitat-zellij/build.sh` produces `habitat_plugin.wasm` and `/usr/bin/cp -f`s it | ✓ 1.2 MB wasm deployed, "Hot-reloaded in active session" |

## 3. Binary topology — `wf-daemon` is the 4th binary, not a replacement

| Binary | Role | Lifecycle | Wave |
|---|---|---|---|
| `wf-crystallise` | observe → mine → propose (produces JSONL proposal) | invoke-and-exit CLI | implementation S1003733 |
| `wf-dispatch` | bank → select → verify → dispatch (POSTs to Conductor `:8141`) | invoke-and-exit CLI | implementation S1003733 |
| `wf-poller` | continuous-tick WFE→SX2 heartbeat emitter | operator-launched continuous CLI | Wave-15 |
| **`wf-daemon`** | habitat-managed service shape (`/health` on `:8142` + embedded poller subsystem) | system-managed via `devenv start` | **Wave-16** |

`wf-poller` is **preserved**, not deleted. The two share the same tick logic via `workflow_core::m16_substrate_drift_canary::transport::tick_and_emit`. `wf-poller` is the standalone CLI shape for development / soak-test driving / forensic debugging; `wf-daemon` is the system-managed shape.

## 4. Internal architecture

```
wf-daemon (process)
├─ #[tokio::main] async runtime
│
├─ Task 1: axum::serve(listener, Router::new().route("/health", get(health)))
│    └─ binds 127.0.0.1:8142
│    └─ GET /health → 200 + {"status":"ok","service":"workflow-trace","port":8142}
│
└─ Task 2: tokio::task::spawn_blocking(poller_subsystem)
     └─ owns std::thread::sleep(1 Hz)
     └─ DriftDetector{[AtuinCheckpoint sampler, M11Recency sampler], SkewEnvelope, AlertBudget}
     └─ HeartbeatTransport (reqwest::blocking) → 127.0.0.1:8092/v3/heartbeat
     └─ SubstrateTrust (seeded SynthexV2 = Live, score 0.9)
     └─ per-tick: tick_and_emit() → Result<HeartbeatAck, RefusalToken>
     └─ tracing::info!(kind_preview="wf_daemon_tick", outcome=ok|engine_imagined|substrate_unreachable|substrate_authored_refusal, ...)
```

### Why `spawn_blocking` for the poller

`HeartbeatTransport` uses `reqwest::blocking` (per WFE `Cargo.toml`: `reqwest = { features = ["blocking", "json", "rustls-tls"] }`). Calling synchronous HTTP from inside a `tokio::spawn` on the multi_thread runtime would pin OS worker threads for the call duration — a few concurrent ticks could starve `axum::serve` (this is **AP38 / ORAC AP29**: sync HTTP I/O in `tokio::spawn`, RCA at S1000007-B commit `aad5af9`).

`spawn_blocking` isolates the blocking caller in the blocking-pool, leaving the multi-thread runtime free for axum. Internally the poller uses `std::thread::sleep` between ticks (not `tokio::time::sleep`), which is the correct primitive inside a blocking task.

### Port 8142 (not 8141)

Originally assigned 8141 because `ss -tlnp` reported it free. False positive — **8141 is reserved for HABITAT-CONDUCTOR** (currently down via `auto_start=false`, per OP-1 hand-off). Caught by FP-verify grep across `devenv.toml` + `ai_docs/MESSAGE_FLOWS.md` + `ai_docs/CODE_MODULE_MAP.md` + `ai_docs/META_TREE_MIND_MAP.md` + `ai_docs/ONBOARDING.md` + `QUICKSTART.md` + `ai_docs/optimisation-v7/RUNBOOKS/runbook-03-phase-2B-active.md`. Re-ported to **8142** (verified free across all 4 surfaces).

**Discipline added (Wave-16 lesson):** *a port being unbound does not mean it is unreserved*. Always grep `devenv.toml` + `ai_docs/` + `ai_specs/` + plugin source before claiming a port is free for a new service.

## 5. `/health` is liveness-only — not wire-aware

`GET /health` returns 200 regardless of substrate reachability. The habitat-plugin grid shows `WFE` green as long as the daemon process is alive — it does NOT certify the WFE→SX2 wire is healthy. Wire-level health is observable only via the daemon's tracing log (`outcome=ok|substrate_unreachable|engine_imagined`).

This is an **intentional split** (not a bug):

1. The V5 substrate-trust gate exists precisely to surface source/deploy drift (Zen ZA-2 / AP-V7-13 — health-200 ≠ behaviour-verified). When the SX2 daemon's running binary mtime predates a Wave-15 source HEAD by 19 days, every tick logs `outcome=substrate_unreachable` honestly. Folding that into `/health` would conflate two different signals — daemon-process health vs substrate-wire health.

2. The habitat-plugin grid is a **liveness dashboard**. Per-service `/health` indicator answering *"is this service's process responsive?"* is the canonical contract every other service follows. WFE adopting the same shape keeps cross-service comparison sane.

3. Wire-aware health belongs in a separate observability surface — e.g. a future `/v1/wire-status` endpoint that surfaces tick counters, last-ok-ms, refusal-rate, V5 trust state. Deferred v0.2.3+.

## 6. Quality gate (this design)

| Stage | Command | Result |
|---|---|---|
| Check | `cargo check --release --bin wf-daemon` | ✓ |
| Clippy | `cargo clippy --release --bin wf-daemon -- -D warnings` | ✓ clean |
| Pedantic | `cargo clippy --release --bin wf-daemon -- -D warnings -W clippy::pedantic` | ✓ clean |
| Build | `cargo build --release --bin wf-daemon` (17 s) | ✓ |
| Test (habitat-modules) | `cargo test --lib -p habitat-modules` | ✓ **91 / 91 passing** |
| Clippy (habitat-modules) | `cargo clippy -p habitat-modules -- -D warnings` | ✓ clean |
| Plugin wasm rebuild | `habitat-zellij/build.sh` | ✓ 1.2 MB deployed |
| Live verify | habitat-plugin grid screenshot | ✓ `ALL UP 14/14 (14 probed)` green `WFE` between Inj and ME |

## 7. Honest residuals

| ID | Item | Owner | Defer-to |
|---|---|---|---|
| R-WAVE16-1 | `/health` is liveness-only | Engine (future amendment) | v0.2.3+ |
| R-WAVE16-2 | SX2 daemon redeploy still standing (OP-OPERATOR-D2) | Luke @ node 0.A | this round (separate operator action) |
| R-WAVE16-3 | m46 Watcher subscription to `Signal::ExternalHeartbeat` (NA-1'' Option C) | SX2 m46 + new daemon task | v0.2.3+ |
| R-WAVE16-4 | Silence-watcher daemon task flipping `Live → Unreachable` (NA-4 + NA-8') | SX2 substrate-side | v0.2.3+ |
| R-WAVE16-5 | HABITAT-CONDUCTOR bring-up (OP-1) — Conductor `auto_start=false` on `:8141` | Luke @ node 0.A | OP-1 |
| R-WAVE16-6 | Workspace charter row drift — `~/claude-code-workspace/CLAUDE.md` "14 active services" table is now 19 | Workspace operator | doc hygiene |

## 8. Cross-references

- **Cargo manifest:** `Cargo.toml` `[[bin]] wf-daemon`
- **Binary source:** `src/bin/wf_daemon.rs`
- **Deployed binary:** `./bin/wf-daemon` (3.1 MB, sha `bf085b2f…`)
- **devenv registration:** `~/.config/devenv/devenv.toml` `[[services]] id = "workflow-trace"`
- **Plugin grid registration:** `habitat-zellij/crates/habitat-modules/src/bridge_health.rs:38` (SERVICES) + `:210` (PROBE_PATHS)
- **HTTP shape spec:** `ai_specs/WF_DAEMON_HTTP_SHAPE.md`
- **Ultramap lifecycle:** `ultramap/WF_DAEMON_LIFECYCLE.md`
- **CHANGELOG entry:** `CHANGELOG.md` `[v0.2.1-wave16]`
- **stcortex memory:** ns `workflow_trace_completion_s1004115` mem **19192** (parent_ids `[19161, 18791]`)
- **POVM mirror:** id `48ba6ee2-d07a-4cb0-9060-4b1921a96fc7` (deprecated overlap)
- **injection.db:** `causal_chain` id **135**
- **Vault notes:** project `[[Wave-16 — wf-daemon Habitat Service Shape S1005032]]` + main habitat `~/projects/claude_code/workflow-trace — Wave-16 Habitat Service S1005032.md`
- **Prior wave (sibling, SX2 side):** Wave-15 `synthex-v2/src/m1_foundation/m09_workflow_trace_participation.rs` + `synthex-v2/src/daemon/http.rs::heartbeat_handler` (commit `c5e3ae4`)
