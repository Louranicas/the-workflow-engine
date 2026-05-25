# Wave-15 — wf-poller WFE→SX2 Wiring (S1005032, 2026-05-25)

> **Back to:** [[CLAUDE.md]] · [[CLAUDE.local.md]]
> **Sibling (SX2 side):** `synthex-v2/obsidian-synthex-v2/synthex-v2/Wave-15 — WFE Heartbeat Inbound Wiring S1005032.md`
> **Commits:** WFE `54c107b` (parent `claude-code-workspace`) + `ce317ae` (standalone `the-workflow-engine`) · SX2 `c5e3ae4` — all four remotes (origin GitHub + GitLab × 2 repos each).

## Headline

Per Luke's 2026-05-25 directive ("ensure the workflow-engine is fully and comprehensively wired into synthex"), the end-to-end **WFE↔SX2 wire is now live in source on both sides**. The wiring that v0.2.0 shipped (NA-3' bilateral V5 + m16 KEYSTONE + W1 transport) was structurally complete but had **zero production caller** — m16/W1/V5 library code was dead code. Wave-15 ships the caller (`wf-poller` binary) + the receiver (`POST /v3/heartbeat` handler) + the bilateral V5 SX2-side mirror.

## What landed (engine side — this repo)

| Artefact | Path | Role |
|---|---|---|
| Binary | `src/bin/wf_poller.rs` (~270 LOC) | The production caller: builds m16 `DriftDetector` (2 `WallClockSampler` → AtuinCheckpoint + M11Recency) + W1 `HeartbeatTransport` + V5 `SubstrateTrust` (seeded Live for SynthexV2) + per-tick `tick_and_emit()` loop at configurable cadence. |
| Cargo manifest | `Cargo.toml` `[[bin]] wf-poller` + `tracing-subscriber` promoted `[dev-dependencies] → [dependencies]` | Runtime log subscriber needed by the binary at production runtime. |

### Env config (operator-driven)

| Env var | Default | Meaning |
|---|---|---|
| `WF_POLLER_ENDPOINT` | `http://127.0.0.1:8092/v3/heartbeat` | SX2 receiver URL |
| `WF_POLLER_INTERVAL_MS` | `1000` (1 Hz per DD-3 §4.1) | tick cadence |
| `WF_POLLER_MAX_CYCLES` | unset = run-forever | bounded run for soak / CI |
| `WF_POLLER_INSTANCE` | `wf-poller-default` | operator-tagged instance id |

### Tracing-only emit per Wave-9 Gate A

Every tick emits a tracing event with `kind_preview` ∈ `{wf_poller_boot, wf_poller_tick, wf_poller_shutdown}`. Outcomes are honestly differentiated:
- `outcome=ok` — substrate accepted heartbeat (real ack returned).
- `outcome=engine_imagined` — V5 short-circuited (substrate `NotShipped` → no HTTP call).
- `outcome=substrate_unreachable` — transport / r13 / port-down.
- `outcome=substrate_authored_refusal` — explicit substrate-side reject.

## What this DOES NOT do (honest residuals)

1. **Daemon redeploy is operator-only (OP-OPERATOR-D2).** SX2 source at HEAD `c5e3ae4` ships the receiver, but the **running** SX2 daemon binary mtime predates the source by ~19 days. Until Luke @ node 0.A redeploys (`/usr/bin/cp -f /tmp/cargo-synthex-v2/release/synthex-v2 synthex-v2/bin/synthex-v2` + `devenv restart synthex-v2`), `wf-poller` will land **lying-200 or stale-binary** responses — that's the AP-V7-13 / Zen ZA-2 source/deploy-drift condition the V5 trust gate exists to make audit-visible. `wf-poller` will tracing-warn `outcome=substrate_unreachable` or land an ack from the wrong binary version.
2. **No m46 Watcher subscription wired yet.** SX2 `Signal::ExternalHeartbeat` is published on `Channel::Health` but m46 currently consumes `TensorSnapshot`, not the signal bus. NA-1'' Option C (signal-bus path) needs a small daemon task or m46 amendment. Operator-visibility deferred to v0.2.3+.
3. **No silence-watcher daemon task.** `WorkflowTraceParticipationStatus::Unreachable` is defined and serialisable but no one flips `Live → Unreachable` after N missed heartbeats. The Watcher liveness consumer NA-4 close depends on this; v0.2.3+ amendment.
4. **NA-8' Goodbye envelope deferred.** Graceful shutdown sends no explicit "wfe departing" signal; substrate must infer silence. Same Watcher concern as #3.
5. **No conductor Z3 bridge cross-talk reply.** Filed earlier in arc; non-blocking; standing for operator triage.

## How to run (post-D2)

```bash
# After Luke redeploys SX2 to HEAD c5e3ae4+:
cd /home/louranicas/claude-code-workspace/the-workflow-engine
cargo build --release --bin wf-poller
WF_POLLER_MAX_CYCLES=10 ./target/release/wf-poller    # 10-cycle smoke
# Or for continuous:
./target/release/wf-poller                            # run-until-SIGKILL
```

Tracing on stdout; substrate confirmation visible in:
- `wf-poller` log lines `outcome=ok cycle_acked=N`
- SX2 `SELECT * FROM workflow_trace_participation_registry` (in-process state — not durable across SX2 restart per honest residual #3).

## 4-stage god-tier gate (this commit)

| Stage | Result |
|---|---|
| `cargo check --all-targets --all-features` | ✓ |
| `cargo clippy -- -D warnings` | ✓ |
| `cargo clippy -- -D warnings -W clippy::pedantic` | ✓ |
| `cargo test --all-targets --all-features --release` | **2191 passed / 0 failed / 2 ignored** (no new tests required — binary is a glue layer over already-tested library) |

## Cross-references

- **NA-3' bilateral V5 gap analysis:** `the-workflow-engine-vault/Wiring Gap Analysis — S1004590 Dual-Frame.md`
- **m16 V3 KEYSTONE** (engine emitter): `src/m16_substrate_drift_canary/`
- **W1 transport client:** `src/m16_substrate_drift_canary/transport.rs`
- **V5 SubstrateTrust + RefusalToken:** `src/substrate_trust/` + `src/refusal_token.rs`
- **SX2 receiver:** `synthex-v2/src/daemon/http.rs::heartbeat_handler` (commit `c5e3ae4`)
- **SX2 bilateral mirror:** `synthex-v2/src/m1_foundation/m09_workflow_trace_participation.rs` (commit `c5e3ae4`)
- **CHANGELOG honest hand-off list:** `CHANGELOG.md` `[v0.2.0]` § Operator hand-off OP-1..OP-6 (Wave-15 closes OP-6 engine-side; OP-OPERATOR-D2 newly added — daemon redeploy still standing).
