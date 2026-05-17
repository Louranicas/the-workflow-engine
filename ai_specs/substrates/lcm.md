---
substrate_id: S-F-lcm
kind: mcp
read_endpoints: ["stdio JSON-RPC (lcm-server)", "http://127.0.0.1:8082/health (V3 deploy partner)", "http://127.0.0.1:8083/health (supervisor)"]
write_endpoints: ["stdio JSON-RPC method `lcm.loop.create` (deploy-shaped only)"]
lifecycle_phases: [cold-start, warming, steady-state, degraded, refusing, dead]
refusal_modes: [supervisor_not_live, deploy_cancel_pending, schema_rejected, rpc_timeout, m0_unverified]
drift_indicators: [rpc_method_rename, supervisor_state_machine_change, deploy_contract_drift_patch, tier_executor_signature_change]
back_pressure_signals: [rpc_inflight_count, supervisor_state, deploy_queue_depth, stdio_buffer_pressure]
consent_dimensions: [n/a]
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A — S1002127 "as per proposal"
hold_v2_compliant: true
addresses: [NA-GAP-01, NA-GAP-04, NA-GAP-07, NA-GAP-08, NA-GAP-09, NA-GAP-10]
---

# S-F — LCM (loop-lifecycle substrate)

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../cross-cutting/persistence.md`](../cross-cutting/persistence.md) · [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) · sister substrates [`atuin.md`](atuin.md) · [`stcortex.md`](stcortex.md) · [`injection_db.md`](injection_db.md) · [`synthex.md`](synthex.md) · [`conductor.md`](conductor.md) · [`watcher.md`](watcher.md) · [`operator.md`](operator.md)
>
> Engine consumer: **m41** ([`../modules/cluster-H/m41_lcm_emit.md`](../modules/cluster-H/m41_lcm_emit.md))

## 1. Purpose & boundary

LCM (Loop Engine V2) is the habitat's **loop-lifecycle substrate** — a Rust binary exposing a stdio JSON-RPC MCP server (`lcm-server`) plus a separate supervisor daemon at `:8083` plus a deploy partner V3 at `:8082`. Master is at commit `97218d1`; 3,662 tests pass; W1+W2+W3a landed. Provides typed methods `lcm.loop.create`, `lcm.loop.tick`, `lcm.deploy`, plus cancel/supervisor methods. `M0_VERIFIED` armed.

**IN scope for `workflow-trace`:** call `lcm.loop.create` from m41 for deploy-shaped DispatchOutcomes only (NOT for non-deploy workflows); outbox-first JSONL durability; respect supervisor liveness.

**OUT of scope:** running LCM itself; modifying its schema; touching V3 deploy partner directly (m41 routes through LCM's RPC); modifying supervisor's state machine.

## 2. Lifecycle phases

| Phase | Indicator | Engine action |
|---|---|---|
| cold-start | supervisor `:8083` recently up; M0_VERIFIED not yet armed | m41 defers; outbox-only |
| warming | M0_VERIFIED armed; lcm-server stdio bridge up | first probe; tolerant of higher RPC latency |
| steady-state | supervisor returns "live"; RPC latency p99 < 200ms | normal cadence |
| degraded | supervisor returns "supervising_recovery"; or RPC latency p99 > 1s | m41 reduces push rate |
| refusing | supervisor reports `deploy_cancel_handler` active; or RPC returns typed error | m41 surfaces typed `EmitError::Refused { class }`; outbox accumulates |
| dead | stdio bridge closed; supervisor `:8083` down | m41 surfaces SubstrateUnavailable; reattempt on recovery |

## 3. Refusal modes (substrate-authored)

- **`SupervisorNotLive`** — supervisor `:8083/health` returns degraded or unreachable. Substrate-authored. Recovery: defer; retry after supervisor recovery.
- **`DeployCancelPending`** — V3 `:8082/deploy/{id}/cancel` was called by Luke; substrate refuses new deploy-shaped writes. Substrate-authored. Recovery: surface to operator; wait for cancel completion.
- **`SchemaRejected`** — RPC argument schema mismatch. Substrate-authored (LCM upgraded its JSON-RPC method signatures). Recovery: pin LCM version; emit drift event.
- **`M0Unverified`** — LCM has not armed M0_VERIFIED yet (pre-W4). Substrate-authored gating. Recovery: defer until milestone fires.
- **`RpcTimeout`** — stdio bridge response exceeded timeout. Substrate-authored (LCM busy). Recovery: backoff + retry.

These map to `RefusalToken::SubstrateAuthored { substrate_id: "lcm", class, repair_hint }` per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md).

## 4. Drift indicators (closes NA-GAP-07)

- **RPC method rename** — `lcm.loop.create` becomes `lcm.create_loop`. Detector: method-list probe at startup; compare against frozen baseline.
- **Supervisor state-machine change** — supervisor adds new state (`degraded_recovery`) without engine knowledge. Detector: state enum probe.
- **Deploy-contract drift patch** (per `loop-engine-v2/.../DEPLOY_CONTRACT_DRIFT_PATCH_2026-05-16.md`) — drift has happened before (Drift #11 retracted; supervisor live-verified at `lcm_supervisor.rs`); pattern is known. Detector: contract-hash check.
- **TierExecutor signature change** (W3b pending) — V3 partner contract evolves. Detector: V3 health body field check.

The LCM substrate is uniquely **drift-aware**: it already carries a drift-patch infrastructure for its own deploy contracts. The engine should mirror this discipline at the RPC boundary.

## 5. Back-pressure signals (closes NA-GAP-04)

- **`rpc_inflight_count`** — engine-side counter; > 16 inflight indicates LCM is queuing.
- **`supervisor_state`** — substrate-side; `supervising_recovery` is yellow.
- **`deploy_queue_depth`** — exposed via supervisor; > 50 is red.
- **stdio buffer pressure** — line buffer near limit; engine should slow writes.

## 6. Receipts (closes NA-GAP-09)

LCM returns a `loop_id` (UUID) on `lcm.loop.create` success — substrate-side receipt. m41 records `loop_id` in outbox JSONL row. The receipt confirms LCM ingested the deploy-shaped DispatchOutcome; it does NOT confirm the loop actually executed to completion — that requires subsequent `lcm.loop.tick` observation. For workflow-trace's purposes, ingest receipt is sufficient.

Per NA-GAP-11, m41 SHOULD also emit `WireEvent::Refusal { token: ... }` to m40's NexusEvent push when LCM refuses, so refusal becomes substrate-readable across S-E and S-F.

## 7. Capabilities & namespaces (closes NA-GAP-10)

LCM authorises:
- `lcm.loop.create` — open to any stdio client (no per-consumer ACL today)
- `lcm.deploy` — NOT used by workflow-trace; reserved for V3 deploy partner
- `lcm.deploy.cancel` — Luke-only via supervisor

Namespace boundary is engine-side: m41 routes only deploy-shaped DispatchOutcomes (NOT all dispatches). The shape filter is engine-authored.

## 8. Substrate-internal couplings (closes NA-GAP-03)

LCM's substrate-internal edges:
- **LCM → V3 deploy partner (`:8082`)** — substrate-substrate; engine triggers via `lcm.deploy` (not used by workflow-trace).
- **LCM → supervisor (`:8083`)** — internal heartbeat; engine observes via `:8083/health`.
- **LCM ← Luke @ terminal** — deploy cancel via `curl -X POST :8082/deploy/{id}/cancel` is operator-mediated control of substrate state. This is an **operator → substrate** edge.

No direct edge from LCM to stcortex (S-C) or injection.db (S-B); the substrate is loop-shaped, not memory-shaped.

## 9. Test-fixture sketch (closes NA-GAP-08)

Fixtures at `tests/substrate_fixtures/lcm/`:

- **`supervisor_not_live_fixture`** — emulated supervisor returns degraded; asserts m41 surfaces typed refusal.
- **`deploy_cancel_pending_fixture`** — V3 cancel mid-flight; asserts m41 holds outbox writes.
- **`schema_rejected_fixture`** — emulated RPC returns method-not-found on rename; asserts drift event.
- **`rpc_timeout_fixture`** — emulator delays response 30s; asserts m41 backs off.
- **`m0_unverified_fixture`** — pre-W4 state; asserts m41 defers writes.

## 10. Watcher class pre-positions

- **Class B (boundary)** — every m41 RPC attempt
- **Class D (drift)** — RPC method-list hash mismatch; supervisor state new variant
- **Class C (refusal)** — substrate-authored deploy-cancel or supervisor-not-live
- **Class A (activation)** — first successful `lcm.loop.create` on a workflow-trace deploy-shaped DispatchOutcome

---

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md)

*Filed 2026-05-17 (S1002127 · Wave 4 NA-remediation) · Luke "as per proposal" · planning-only · HOLD-v2 compliant.*
