# Wave-16 — wf-daemon Habitat-Service Shape (S1005032, 2026-05-25)

> **Back to:** [[CLAUDE.md]] · [[CLAUDE.local.md]] · [[Wave-15 — wf-poller wiring S1005032]]
> **Sibling vaults:** `habitat-zellij/CLAUDE.local.md` (plugin grid 13→14) · `~/projects/claude_code/workflow-trace — Wave-16 Habitat Service S1005032.md` (main habitat vault mirror)
> **Companion (SX2 side, prior round):** `synthex-v2/obsidian-synthex-v2/synthex-v2/Wave-15 — WFE Heartbeat Inbound Wiring S1005032.md`

## Headline

Per Luke's 2026-05-25 directive ("verify all binaries... align with all conventions and systems of the zellij habitat services... ensure the workflow-engine starts with these services and is included [in the habitat-plugin grid]"), workflow-trace is now a **first-class managed habitat service** alongside V3, Nerve, TL, SX, V8, VMS, POVM, RM, PV2, ORAC, Inj, ME, PSw — visible as `WFE` in the Zellij habitat-plugin's 14-service grid (`V3 Nerve TL SX V8 VMS POVM RM PV2 ORAC Inj WFE ME PSw`). The wire shipped in Wave-15 (`wf-poller` standalone CLI, operator-launched) is now system-managed under `devenv start`'s auto-start + auto-restart envelope.

## What landed

| Artefact | Role |
|---|---|
| `src/bin/wf_daemon.rs` (~330 LOC) | Habitat-service binary: axum `GET /health` on port 8142 + `tokio::spawn_blocking` poller subsystem (`wf-poller` tick logic embedded as one task) |
| `Cargo.toml` `[[bin]] wf-daemon` + axum dep | Build target + minimal `axum` + `tower` deps (hyper / tower-http already transitive via reqwest) |
| `bin/wf-daemon` (3.1M, sha `bf085b2f…`) | Deployed release binary at project-`./bin/` convention (matches V3/PV2/ORAC/Nerve/ME placement) |
| `~/.config/devenv/devenv.toml` `[[services]] id="workflow-trace"` | Service registration with `auto_start=true`, `auto_restart=true`, `command="./bin/wf-daemon"`, working_dir `the-workflow-engine/` |
| `habitat-zellij/.../bridge_health.rs` SERVICES + PROBE_PATHS | `(8142, "WFE")` + `(8142, "/health")` — 13→14 service grid in the Zellij plugin |
| `habitat-plugin.wasm` rebuilt + deployed | 1.2M wasm at `~/.config/zellij/plugins/habitat-plugin.wasm`; "Hot-reloaded in active session" |

## Port 8142 (NOT 8141) — re-port story

First attempt assigned 8141 because `ss -tlnp` reported it "free" — but 8141 is **RESERVED for HABITAT-CONDUCTOR** (currently down per OP-1 hand-off, `auto_start=false`). Catch came from FP-verifying via `grep` across `~/.config/devenv/devenv.toml` + WFE `ai_docs/*` + `QUICKSTART.md` + `MESSAGE_FLOWS.md` + `CODE_MODULE_MAP.md` + `ONBOARDING.md` — all reference `:8141` as the Conductor port that `wf-dispatch m32` posts to. Re-ported to **8142** (verified free across all 4 surfaces). Pattern logged as a scaffold-mastery false-positive avoidance: **a port being unbound does not mean it is unreserved** — always grep docs + devenv.toml before claiming a port is free for a new service.

## Operational shape — read-only liveness contract

- **Port:** 8142
- **Endpoint:** `GET /health` → `{"status":"ok","service":"workflow-trace","port":8142}` (200 always-on-if-daemon-alive)
- **Poller cadence:** 1 Hz (DD-3 §4.1, inherited from `wf-poller` standalone)
- **Substrate endpoint:** `http://127.0.0.1:8092/v3/heartbeat` (SX2 m09 `workflow_trace_participation` registry; receiver shipped Wave-15 commit `c5e3ae4`)
- **Env overrides:** `WF_DAEMON_PORT`, `WF_POLLER_ENDPOINT`, `WF_POLLER_INTERVAL_MS`, `WF_POLLER_INSTANCE`
- **Tracing-only emit per tick:** `outcome ∈ {ok, engine_imagined, substrate_unreachable, substrate_authored_refusal}` — see `wf_daemon.rs` doc § Source/deploy drift awareness
- **`/health` body is liveness-only:** returns 200 regardless of SX2 reachability. The habitat-plugin grid will show WFE green as long as the daemon process is alive; substrate-wire health is observable only via the daemon's tracing log. Intentional split — keeps WFE indicator stable across the WFE↔SX2 source/deploy drift the V5 trust gate exists to surface.

## Architectural delta from prior charter

The WFE `CLAUDE.md` project charter prior to this round described workflow-trace as having a "two-binary split: `wf-crystallise` owns m1-m23+m40-m42; `wf-dispatch` owns m30-m33". Wave-16 adds a **third binary shape** — `wf-daemon` — but does NOT change the canonical operational role of the other two:

- `wf-crystallise` — invoke-and-exit, produces JSONL proposal report (unchanged)
- `wf-dispatch` — invoke-and-exit, consumes JSONL + verifies + dispatches via Conductor `:8141` (unchanged)
- `wf-poller` — operator-launched continuous CLI (Wave-15; superseded by `wf-daemon` as system-managed shape but binary preserved)
- **`wf-daemon` — habitat-managed daemon (NEW Wave-16):** axum `/health` on `:8142` + embedded poller tick logic. The service-shape binary that `devenv start` manages.

## 4-stage god-tier gate (this commit)

| Stage | Result |
|---|---|
| `cargo check --release --bin wf-daemon` | ✓ |
| `cargo clippy --release --bin wf-daemon -- -D warnings` | ✓ clean |
| `cargo clippy --release --bin wf-daemon -- -D warnings -W clippy::pedantic` | ✓ clean |
| `cargo build --release --bin wf-daemon` | ✓ 17s |
| **habitat-modules** `cargo test --lib -p habitat-modules` | ✓ **91/91 passing** |
| **habitat-modules** `cargo clippy ... -- -D warnings` | ✓ clean |
| `habitat-plugin.wasm` rebuild + hot-reload | ✓ 1.2M deployed |
| Live verify: `ALL UP 14/14 (14 probed)` in plugin grid screenshot | ✓ visible green WFE between Inj and ME |

## Honest residuals

1. **`/health` is liveness-only, not wire-aware** — if WFE↔SX2 wire is silently dropping ticks, WFE shows green in the grid. Future amendment: bake tick counters + last-ok-ms into `/health` body so habitat-plugin can render `WFE (12K ticks, last ok 1s ago)` or `WFE (no ack 60s)`. Deferred v0.2.3+.
2. **No m46 Watcher subscription yet** — `Signal::ExternalHeartbeat` is published on SX2's `Channel::Health` but m46 consumes `TensorSnapshot`; NA-1'' Option C closure pending (carried over from Wave-15).
3. **No silence-watcher daemon task** — `WorkflowTraceParticipationStatus::Unreachable{missed_count}` defined but no transitions; NA-4 + NA-8' closure pending (carried over from Wave-15).
4. **SX2 daemon redeploy still standing** (OP-OPERATOR-D2 from Wave-15) — running synthex-v2 binary mtime 2026-05-06 predates Wave-15 source `c5e3ae4` by 19 days. Until Luke redeploys, `wf-daemon`'s poller subsystem will tracing-warn `outcome=substrate_unreachable` / lying-200 every tick. The V5 trust gate keeps this audit-visible; the daemon itself stays healthy + plugin-green.
5. **Workspace charter row drift** — `~/claude-code-workspace/CLAUDE.md` "14 active services" table is now ≥15 with workflow-trace added; charter refresh pending operator triage.
6. **Doc-level claim drift across WFE ai_docs** — references to "two-binary split" in older specs are now superseded by the three-binary shape (`wf-crystallise` + `wf-dispatch` + `wf-daemon`); incremental fold-in across `ai_specs/` happening this Wave-16 sweep.

## Cross-references

- **Sibling project (consumer) — SX2 m09:** `synthex-v2/src/m1_foundation/m09_workflow_trace_participation.rs` + `synthex-v2/src/daemon/http.rs` `heartbeat_handler` (commit `c5e3ae4`)
- **Sibling plugin (display) — habitat-zellij:** `habitat-zellij/crates/habitat-modules/src/bridge_health.rs:38` (SERVICES list) + `:210` (PROBE_PATHS); WFE display tag chosen for brevity per V3/SX/ME convention
- **NA-3' bilateral V5 gap origin:** `the-workflow-engine-vault/Wiring Gap Analysis — S1004590 Dual-Frame.md`
- **W1 transport / m16 KEYSTONE source:** `src/m16_substrate_drift_canary/transport.rs` + `src/m16_substrate_drift_canary/mod.rs`
- **V5 substrate-trust source:** `src/substrate_trust/mod.rs` + `src/refusal_token.rs`
- **CHANGELOG entry:** `CHANGELOG.md` `[v0.2.1-wave16]`
- **Design doc:** `ai_docs/WAVE_16_WF_DAEMON_DESIGN_S1005032.md`
- **HTTP shape spec:** `ai_specs/WF_DAEMON_HTTP_SHAPE.md`
- **Ultramap lifecycle:** `ultramap/WF_DAEMON_LIFECYCLE.md`
- **stcortex memory:** ns `workflow_trace_completion_s1004115` mem **19162** (parent_ids `[19161, 18791]`); bidi pathway `wave_16_wfe_habitat_service_s1005032 ↔ wave_15_wfe_sx2_wiring_s1005032` (0.95)
- **POVM mirror:** ns `workflow_trace_completion_s1004115` (deprecated overlap)
- **injection.db row:** `causal_chain` id **134** label `wave16_wfe_habitat_service_s1005032`
