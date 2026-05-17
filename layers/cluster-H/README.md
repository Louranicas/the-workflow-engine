---
cluster_id: H
name: Substrate Feedback
modules: [m40, m41, m42]
binary: wf-crystallise
loc_estimate: ~450
substrates_touched: [S-E SYNTHEX v2 (m40), S-F LCM (m41), S-C stcortex via m13 (m42 — CC-5 primary)]
date: 2026-05-17
status: SCAFFOLD (pre-G9; LIVE-on-G9-fire-days-11-12)
povm_decoupled: true (m42 per 2026-05-17 ADR D-S1001982-01)
---

# layers/cluster-H — Substrate Feedback (operational landing)

> **Back to:** [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · sister design spec [`../../ai_specs/layers/cluster-H.md`](../../ai_specs/layers/cluster-H.md) · per-module specs [`../../ai_specs/modules/cluster-H/`](../../ai_specs/modules/cluster-H/) · synergy [`../../ai_specs/synergies/CC-5.md`](../../ai_specs/synergies/CC-5.md) (SPECIAL DEPTH) · substrate-coupling [`../../ai_specs/substrate-couplings/CC-5-decomposed.md`](../../ai_specs/substrate-couplings/CC-5-decomposed.md) · substrate dossiers [`../../ai_specs/substrates/synthex.md`](../../ai_specs/substrates/synthex.md) · [`../../ai_specs/substrates/lcm.md`](../../ai_specs/substrates/lcm.md) · [`../../ai_specs/substrates/stcortex.md`](../../ai_specs/substrates/stcortex.md)

## What this cluster IS

Cluster H is the **write-side substrate feedback layer** — three fan-out emitters that close the engine's loop back to the habitat substrates after every dispatched workflow with a known outcome:

- **m40 NexusEvent emit** — typed `WorkflowEvent` push to SYNTHEX v2 `:8092/v3/nexus/push`
- **m41 LCM RPC** — `lcm.loop.create` (NOT `lcm.deploy`) for deploy-shaped DispatchOutcomes
- **m42 stcortex emit** — Hebbian pathway-write via m13 under `workflow_trace_*` AP30 prefix; **POVM-DECOUPLED** per D-S1001982-01

Cluster H is the **CC-5 emit half**. Per [`../../ai_specs/synergies/CC-5.md`](../../ai_specs/synergies/CC-5.md): "CC-5 IS the engine's only substrate-grain loop." Cluster H carries this load alone.

## Modules

| Module | Substrate | Outbox | LOC | Spec |
|---|---|---|---|---|
| **m40** nexusevent_emit | S-E SYNTHEX v2 `:8092` | `{data_dir}/m40/outbox.jsonl` | ~140 | [`m40_nexusevent_emit.md`](../../ai_specs/modules/cluster-H/m40_nexusevent_emit.md) |
| **m41** lcm_rpc | S-F LCM (`lcm.loop.create`; MCP) | `{data_dir}/m41/outbox.jsonl` | ~140 | [`m41_lcm_rpc.md`](../../ai_specs/modules/cluster-H/m41_lcm_rpc.md) |
| **m42** stcortex_emit | S-C stcortex via m13 | `{data_dir}/m42/outbox.jsonl` + `{offline_snapshot_path}/m42-replay.jsonl` | ~170 | [`m42_stcortex_emit.md`](../../ai_specs/modules/cluster-H/m42_stcortex_emit.md) |

## CC-5 cascade (5 substrate-substrate edges)

Per [`../../ai_specs/substrate-couplings/CC-5-decomposed.md`](../../ai_specs/substrate-couplings/CC-5-decomposed.md), each Cluster H emit triggers a substrate-side cascade with 5 edges:

| Edge | Owner | Substrate cascade |
|---|---|---|
| **E1** | m42 → m13 | m42 → S-C stcortex pathway delta (PRIMARY) |
| **E2** | m40 | m40 → S-E SYNTHEX v2 Hebbian coordinator → S-C stcortex (since S226) |
| **E3** | (substrate-internal) | S-C stcortex → habitat-memory daemon → S-B injection.db reinforcement_count++ |
| **E4** | m41 | m41 → S-F LCM `lcm.loop.create` → S-V3 deploy partner (deploy-shaped only) |
| **E5** | (substrate-internal) | S-C stcortex → weekly digest → S-G operator |

The engine OWNS only E1 / E2 / E4 (the m42 / m40 / m41 emit-points); E3 + E5 are **substrate-mediated** and require per-edge observability per CC-5-decomposed.

## Cross-cluster contracts

- **CC-5 (Substrate Learning Loop — primary)** — Cluster H is the emit half; m32 (Cluster G) is the trigger
- **No m40/m41/m42 cross-talk** — fan-out is parallel and fire-and-forget per the CC-5 contract; each module has its own breaker, outbox, idempotency cache

## Watcher class pre-position

- **Class I (Hebbian silence — PRIMARY for CC-5)** — fires if stcortex pathway-weight delta on `workflow_trace_*` IDs stays zero over rolling 4-week window
- **Class A (activation)** — fires on first CC-5 closure (m42 first successful round-trip)
- **Class B (hand-off boundary)** — fires on every Cluster H wire emit
- **Class D (four-surface drift)** — fires if outbox JSONL durability drifts from wire emit schema (m40_42_common consistency check)

## Outbox-first JSONL durability (per module)

All three modules follow the **outbox-first** pattern (per [`../../ai_specs/cross-cutting/persistence.md`](../../ai_specs/cross-cutting/persistence.md) + per-module spec §5 algorithm):

1. Compose payload
2. Append envelope to `outbox.jsonl` + `sync_data` (durable)
3. Check breaker (Closed/HalfOpen → proceed; Open → write offline snapshot + return SubstrateUnavailable)
4. Check idempotency (`request_id` UUIDv4 dedup within 1h window)
5. Wire the substrate call via m13 (m42) or direct HTTP/MCP (m40/m41)
6. On success: mark envelope `posted: true`; on substrate-unavailable: write offline snapshot; on schema-error: return SubstrateError

**Network NEVER blocks m32 dispatch.** Substrate-down is a recoverable condition (offline-snapshot replay on substrate return).

## m42 outbox policy (NA-GAP-06 closure)

Per [`../../ai_specs/modules/cluster-H/m42_stcortex_emit.md`](../../ai_specs/modules/cluster-H/m42_stcortex_emit.md) § 5.1 (added Wave 4.B):

- **Drain ordering on recovery** — envelope.id ascending; idempotency-honoured replay; throttle cap 20/sec
- **Saturation limit** — warn 64 MB / refuse 256 MB / panic 1 GB (fail-loud at saturation, NOT silent unbounded growth)
- **Snapshot staleness** — warn 5 min / refuse 1 hr / panic 24 hr at boot

These policies are m42-specific (m40/m41 inherit the general outbox pattern; their saturation/staleness thresholds are not specified separately because their substrates have weaker durability requirements than stcortex).

## Runtime concerns (post-G9; placeholder)

| Concern | Pre-G9 status |
|---|---|
| Metrics emitted | DEFERRED — full m42 inventory in spec §5.1.e |
| Failure-mode | OutboxIo / Bridge / Timeout / CircuitOpen / SubstrateUnavailable / InvalidShape (per [`../../ai_specs/ERROR_TAXONOMY.md`](../../ai_specs/ERROR_TAXONOMY.md) § Cluster H) |
| Refusal token | `SubstrateAuthored { S-C \| S-E \| S-F, ... }` per [`../../ai_specs/cross-cutting/refusal-taxonomy.md`](../../ai_specs/cross-cutting/refusal-taxonomy.md) |

## Wave-1 build order (post-G9)

Cluster H ships **Days 11-12** post-G9 (penultimate; depends on Cluster D trust scaffolding + Cluster A readers + Cluster C m13 writer + Cluster G m32 trigger all being live). Order within Cluster H:

1. m40 nexusevent_emit (Day 11 — simplest substrate edge; HTTP push pattern)
2. m41 lcm_rpc (Day 11 — similar shape to m40; MCP instead of HTTP)
3. m42 stcortex_emit (Day 12 — most invariants; AP30 + AP-Hab-11 + outbox policy + CC-5 primary owner)

## HOLD-v2 compliance

This README is markdown only. **0** `.rs` files, **0** `Cargo.toml`, **0** code under `layers/cluster-H/`. The POVM-decoupled status (per D-S1001982-01) is the spec-level decision; no code is written.

---

> **Back to:** [`../../README.md`](../../README.md) · sister [`../../ai_specs/layers/cluster-H.md`](../../ai_specs/layers/cluster-H.md) · [`../../ai_specs/synergies/CC-5.md`](../../ai_specs/synergies/CC-5.md) · [`../../ai_specs/substrate-couplings/CC-5-decomposed.md`](../../ai_specs/substrate-couplings/CC-5-decomposed.md)

*Filed 2026-05-17 (S1002127 Wave 4.B audit) · Command · planning-only · HOLD-v2 compliant · POVM-decoupled.*
