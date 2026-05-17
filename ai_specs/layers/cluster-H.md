---
title: Cluster H — Substrate Feedback (Layer L8) — Layer Spec
cluster: H
layer: L8
module_count: 3
modules: [m40_nexusevent_emit, m41_lcm_rpc, m42_stcortex_emit]
binary: wf-crystallise
feature_gates: [monitoring]
cc_owns: [CC-5 (emit-half; primary substrate-grain loop)]
cc_consumes: [CC-2]
ship_priority: Day 4 Wave 3 (parallel with Cluster G)
status: SPEC (m42 POVM-DECOUPLED per 2026-05-17 ADR)
date: 2026-05-17
hold_v2_compliant: true
---

# Cluster H — Substrate Feedback (L8)

> **POVM-DECOUPLED.** Per [`../../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`](../../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md), m42 routes substrate-feedback to stcortex exclusively from M0. POVM `:8125` is not a workflow-trace dependency. src/ path: `src/m42_stcortex_emit/` (NOT `src/m42_povm_dual/`).
>
> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · vault [[cluster-H-substrate-feedback]] · [`../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-H.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-H.md)

## Role

Cluster H IS **the substrate-feedback emit layer** — three modules that take m32's `DispatchOutcome` and fire-and-forget into three external surfaces: SYNTHEX v2 (m40 NexusEvent at `:8092/v3/nexus/push`), LCM (m41 `lcm.loop.create` for deploy-shaped workflows ONLY), and stcortex (m42 pathway reinforce via m13 transport, AP30 `workflow_trace_*` prefix). The cluster is the **emit-half of CC-5** — the only true substrate-grain loop in the engine per G3 substrate-frame pass; the other CCs are anthropocentric control-flow grain.

The structural fact load-bearing for this cluster is **CC-5's silent-failure mode**: if Cluster H emits but substrate doesn't move (pathway weights stay flat), the engine appears functional but is not learning. This is the engine's most important silent-failure mode and pre-positions Watcher Class-I (Hebbian silence). The mitigation is the Phase 5C weekly Watcher synthesis with `stcortex.pathway.weight` rolling-window delta on `workflow_trace_*` IDs — if delta = 0 for 4+ weeks, Class-I fires. **m42 NEVER takes HTTP 200 on `/v1/pathway` as proof that pathway-weights moved** (AP-V7-13: Health-200 ≠ behaviour-verified — crystallised from the POVM `:8125` pre-CR-2-binary trap).

The three modules cover three distinct concerns. **m40** emits typed NexusEvents to SYNTHEX v2 for coordination-grain observability (fleet-wide event bus). **m41** routes deploy-shaped workflows through LCM's `lcm.loop.create` RPC (NOT `lcm.deploy` — common API confusion; m41 spec pins the correct call). **m42** is the Hebbian-grain reinforcement path: per-outcome `fitness_delta` (PassVerified +0.25 / Pass +0.15 / Blocked −0.05 / Fail −0.10) writes to stcortex pathway under `workflow_trace_*` namespace.

## Modules

| Module | Spec | LOC | Tests | Target | Verb |
|---|---|---:|---:|---|---|
| `m40_nexusevent_emit` | [`../modules/cluster-H/m40_nexusevent_emit.md`](../modules/cluster-H/m40_nexusevent_emit.md) | 160 | 65 | SYNTHEX v2 :8092 | emit |
| `m41_lcm_rpc` | [`../modules/cluster-H/m41_lcm_rpc.md`](../modules/cluster-H/m41_lcm_rpc.md) | 140 | 65 | LCM MCP | route |
| `m42_stcortex_emit` | [`../modules/cluster-H/m42_stcortex_emit.md`](../modules/cluster-H/m42_stcortex_emit.md) | 150 | 70 | stcortex :3000 (via m13) | emit (Hebbian) |

## Cross-cluster contracts

- **OWNS:**
  - **CC-5 (Substrate Learning Loop, emit-half)** — m40/m41/m42 are the three fire-and-forget emit paths. CC-5 closes when m31 (Cluster G) reads updated pathway weights at next selection cycle. See [`../synergies/CC-5.md`](../synergies/CC-5.md).
- **CONSUMES:**
  - **CC-2** — m9 namespace-guard wraps every namespace-bearing write (m42's `workflow_trace_*` prefix; m40's NexusEvent envelope topic name).

## Binary placement

All three modules in **`wf-crystallise`**. Counterintuitive — they ARE substrate-feedback emit, which is dispatch-adjacent — but the canonical two-binary split locks emit in the crystallise binary so `wf-dispatch` stays minimal (bank + select + dispatch + verify only, no observability dependencies). m32 (in `wf-dispatch`) fires `DispatchOutcome` into a channel that the `wf-crystallise` process (running concurrently) drains and emits.

In single-process invocation (`wf-crystallise --serve`), m32's outcome channel is in-process. In CLI invocation, m32 writes outcome to a JSONL outbox; `wf-crystallise` picks up on next ingest tick. The outbox-first JSONL durability layer is the load-bearing fact — substrate-down NEVER blocks dispatch.

## Feature-gate posture

**`feature = "monitoring"`** gated. Cluster H exports prometheus-style counters (`emit_*_total`, `circuit_open_total`, `substrate_unavailable_total`) and is the observability surface; with monitoring off, the engine still runs but doesn't emit substrate-feedback (degraded CC-5 — accepted in debug-only profile).

## Ship priority

**Day 4 Wave 3 (parallel with Cluster G).** Implementation order: m40 → m42 → m41. m40 ships first (NexusEvent emit is the simplest wire-contract); m42 second (Hebbian reinforce + outbox-first durability is the substrate-grain primitive); m41 last (LCM RPC has the narrowest applicability — deploy-shaped only — and the most-likely-to-be-misspecified call surface).

## Operational invariants

1. **Outbox-first JSONL durability.** Every emit writes to `outbox/m{40,41,42}/*.jsonl` BEFORE the network RPC. Substrate-down means the outbox carries the record; offline-snapshot replay drains the outbox when substrate returns. Network never blocks dispatch.
2. **Circuit breaker on 2 consecutive failures.** Each module has a `Breaker { state: BreakerState, consecutive_failures: u32 }`; breaker OPEN after 2 failures, HALF_OPEN after 30s, CLOSED after 1 success. Watcher Class-I log on every OPEN transition.
3. **AP30 namespace prefix machine-enforced (m42).** Every pathway write has the form `workflow_trace_<workflow_id>` → `workflow_trace_outcome_<outcome>`; `workflow_core::namespace::WORKFLOW_TRACE_PREFIX` constant is the only acceptable source; m9 namespace-guard validates.
4. **fitness_delta constants module-level (m42).** PassVerified +0.25 / Pass +0.15 / Blocked −0.05 / Fail −0.10; not hardcoded at call sites. Clamp to `[−1.0, 1.0]` is post-compute defense.
5. **AP-Hab-11 hyphen-slug encoding (m42).** stcortex `pre_id`/`post_id` slugs convert hyphens to underscores (S1001757 munge bug); `workflow_trace_wf-abc-123` → `workflow_trace_wf_abc_123` at the slug boundary.
6. **m41 routes deploy-shaped ONLY.** Non-deploy workflows are NOT routed through LCM; m41 returns `Ok(NotRouted { reason: "non_deploy_shape" })` and the workflow continues without LCM emit. LCM is the *deploy* substrate; m32 dispatches the actual exec.
7. **No POVM dependency (m42).** Zero POVM imports, zero POVM HTTP calls, zero POVM config flags. Per ADR 2026-05-17 R3 + R10: permanent.

## Failure modes the cluster structurally refuses

- **m40 emit blocking dispatch.** Outbox-first means the network is best-effort; m32 is never blocked.
- **m41 calling `lcm.deploy` instead of `lcm.loop.create`.** Hard-coded RPC name; contract test pins it. See [`../WIRE_PROTOCOL_SPEC.md`](../WIRE_PROTOCOL_SPEC.md) § m41.
- **m42 reading POVM `learning_health` as success-signal.** Per AP-V7-13: HTTP 200 ≠ behaviour-verified. m42 NEVER takes stcortex's 200 on `/v1/pathway` as proof; verification belongs to Class-I Watcher rolling-window delta.
- **m42 silently falling back to POVM on stcortex outage.** Per Genesis v1.3 § 2 + ADR R5: ERROR log, `ReinforceOutcome::SubstrateUnavailable` typed return, outbox carries the record. No silent POVM fall-through.
- **Circuit breaker auto-close without success.** OPEN → HALF_OPEN after 30s; HALF_OPEN requires 1 successful emit to transition to CLOSED. Silent auto-close masks substrate down.

## Performance envelope

| Operation | Target | Notes |
|---|---|---|
| m40 emit (outbox write) | < 5 ms | atomic file write + serde |
| m40 emit (NexusEvent push) | < 100 ms p99 | best-effort HTTP POST; outbox covers failure |
| m41 emit (LCM RPC) | < 200 ms p99 | MCP stdio JSON-RPC; LCM-side latency |
| m42 emit (via m13 stcortex) | < 100 ms p99 | HTTP POST to `:3000`; outbox covers failure |
| Circuit breaker check | < 100 ns | atomic load |

## Verify-sync invariants

- **#1** — `src/m40_nexusevent_emit/`, `src/m41_lcm_rpc/`, `src/m42_stcortex_emit/` all exist post-G9. (m42 path: `m42_stcortex_emit/` NOT `m42_povm_dual/`.)
- **#13** — every async fn in m40/m41/m42 has `#[tracing::instrument(skip(..))]`.
- **#14** — every external IO has `tokio::time::timeout`.
- **#15** — `m42` has zero `povm` symbol; `rg -i 'povm' src/m42_stcortex_emit/ | grep -v '// POVM-DECOUPLED'` returns 0.
- **#17** — `tests/integration/cc5_substrate_learning_loop.rs` exists.

## Per-module cross-links

- [`../modules/cluster-H/m40_nexusevent_emit.md`](../modules/cluster-H/m40_nexusevent_emit.md) — SYNTHEX v2 NexusEvent emitter
- [`../modules/cluster-H/m41_lcm_rpc.md`](../modules/cluster-H/m41_lcm_rpc.md) — LCM `lcm.loop.create` RPC router
- [`../modules/cluster-H/m42_stcortex_emit.md`](../modules/cluster-H/m42_stcortex_emit.md) — stcortex Hebbian reinforce (POVM-DECOUPLED)

## Antipatterns specific to Cluster H

- **AP-V7-13** (Health-200 ≠ behaviour-verified) — m42's load-bearing structural lesson; Class-I Watcher monitor is the actual verification.
- **AP-Drift-06** (bridge contract drift) — m40/m41/m42 wire contracts pinned at `tests/contract/m4{0,1,2}_*.rs`.
- **AP-WT-F7** (CR-2 graceful-degrade pretend-fix) — m42 surfaces `SubstrateUnavailable` typed error; never silent fallback.
- **AP-Hab-11** (hyphen-slug munge) — m42 converts hyphens to underscores at slug boundary.
- **AP30** (namespace string drift) — m42 only writes through `WORKFLOW_TRACE_PREFIX` constant.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · vault [[cluster-H-substrate-feedback]]
