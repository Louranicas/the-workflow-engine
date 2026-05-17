---
cluster_id: G
name: Bank + Select + Dispatch + Verify
modules: [m30, m31, m32, m33]
binary: wf-dispatch (m30 audit-source in wf-crystallise)
loc_estimate: ~950
substrates_touched: [engine-internal SQLite (bank.db), S-D HABITAT-CONDUCTOR (m32), S-G operator (m30 acceptance signature)]
date: 2026-05-17
status: SCAFFOLD (pre-G9; LIVE-on-G9-fire-days-8-10)
---

# layers/cluster-G — Bank + Select + Dispatch + Verify (operational landing)

> **Back to:** [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · sister design spec [`../../ai_specs/layers/cluster-G.md`](../../ai_specs/layers/cluster-G.md) · per-module specs [`../../ai_specs/modules/cluster-G/`](../../ai_specs/modules/cluster-G/) · synergy [`../../ai_specs/synergies/CC-4.md`](../../ai_specs/synergies/CC-4.md) + [`CC-6.md`](../../ai_specs/synergies/CC-6.md) · substrate-coupling [`../../ai_specs/substrate-couplings/CC-4-decomposed.md`](../../ai_specs/substrate-couplings/CC-4-decomposed.md) · substrate dossiers [`../../ai_specs/substrates/conductor.md`](../../ai_specs/substrates/conductor.md) · [`../../ai_specs/substrates/operator.md`](../../ai_specs/substrates/operator.md)

## What this cluster IS

Cluster G is the **dispatch pipeline** — four modules that take m23's proposals all the way to a Conductor dispatch (NEVER directly to a target service):

- **m30 curated bank** — operator-acceptance-gated workflow store (AP-V7-07: NO auto-promotion); EscapeSurfaceProfile-aware admission (7-variant per D-S1002127-02)
- **m31 selector** — substrate-aware selection (reads PV2 r, RALPH fitness, thermal); m11 fitness-weighted-decay consumer
- **m32 conductor dispatcher** — 5-check sequence + Conductor RPC; AP-V7-08 self-dispatch refusal; CC-4 substrate-coupling owner
- **m33 verifier** — 4-agent verification quorum; CC-6 verification-gated dispatch

## Modules

| Module | Concern | Substrate touch | LOC | Spec |
|---|---|---|---|---|
| **m30** curated_bank | SQLite-backed workflow store; HumanAcceptanceSignature required; EscapeSurfaceProfile 7-variant ordinal; sunset/cooldown machinery | engine-internal SQLite; S-G operator | ~280 | [`m30_curated_bank.md`](../../ai_specs/modules/cluster-G/m30_curated_bank.md) |
| **m31** selector | reads bank + m11 decay + substrate-degradation state (PV2 r, RALPH fitness, thermal); emits dispatch candidate | engine reads | ~180 | [`m31_selector.md`](../../ai_specs/modules/cluster-G/m31_selector.md) |
| **m32** conductor_dispatcher | 5-check sequence (audit-first, verification, definition_hash, sunset, cooldown); Conductor RPC; emits CC-5 cascade | S-D Conductor (primary); S-G operator (banner) | ~300 | [`m32_conductor_dispatcher.md`](../../ai_specs/modules/cluster-G/m32_conductor_dispatcher.md) |
| **m33** verifier | 4-agent verification quorum; closes CC-6; produces fresh verification artefact for m32 check[2] | engine-internal; cross-service via m33's own bridges | ~190 | [`m33_verifier.md`](../../ai_specs/modules/cluster-G/m33_verifier.md) |

## Cross-cluster contracts

- **CC-4 (Proposal → Bank → Dispatch → Conductor)** — Cluster G IS the receiving side of CC-4 (m23 from Cluster F → m30); the substrate-side decomposition lives at [`../../ai_specs/substrate-couplings/CC-4-decomposed.md`](../../ai_specs/substrate-couplings/CC-4-decomposed.md) (3 edges: m32→S-D, S-D refusal-path, m30→S-G operator)
- **CC-5 (Substrate Learning Loop — emit-trigger)** — m32 is the trigger module for CC-5; every m32 dispatch with known outcome fires Cluster H emit (the substrate-feedback fan-out)
- **CC-6 (Verification-Gated Dispatch, internal m33 → m32)** — Cluster G internal contract; m33 produces, m32 consumes via check[2]
- **CC-7 (operator-substrate refusal feedback)** — m30 surfaces operator refusal tokens; surfaced via m15 pressure register and back through CC-7

## Watcher class pre-position

- **Class A (activation)** — fires on first m30 admission + first m32 dispatch
- **Class B (hand-off boundary)** — fires on m32 → Conductor every call
- **Class C (refusal)** — fires on m32 5-check refusal (any of the 5 checks); also m30 AutoPromoteRefused
- **Class I (Hebbian silence, indirect)** — fires through CC-5 if cluster H emit doesn't move substrate

## Substrate-side concerns (S-D Conductor + S-G operator)

m32 ↔ S-D Conductor (per [`../../ai_specs/substrates/conductor.md`](../../ai_specs/substrates/conductor.md)):
- AP-V7-13 enriched case (see [`../../ai_specs/substrate-couplings/CC-4-decomposed.md`](../../ai_specs/substrate-couplings/CC-4-decomposed.md) § E2): Conductor health-200 ≠ Wave 1B/1C/2/3 enforcement-ready
- `CONDUCTOR_ENFORCEMENT_ENABLED=0` (current NoOp soak state) — m32 must detect and surface, NOT silently dispatch into the void
- 5-check sequence including check[3] definition_hash defends against silent definition drift

m30 ↔ S-G operator (per [`../../ai_specs/substrates/operator.md`](../../ai_specs/substrates/operator.md) + [`../../ai_specs/substrate-couplings/CC-7-decomposed.md`](../../ai_specs/substrate-couplings/CC-7-decomposed.md) E1):
- HumanAcceptanceSignature required for every admission (AP-V7-07 forbids auto-promote)
- Consent-budget aware — m12 surfaces operator approaching ConsentFatigue (default 5 signatures/session cap, configurable)

## EscapeSurfaceProfile cardinality 7 (D-S1002127-02)

Per [`../../ai_docs/optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md`](../../ai_docs/optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md), the EscapeSurfaceProfile enum carries 7 variants (was 6 — `PrivilegeEscalation` added at ord 30 to cover openclaw container-escape class). m30 + m32 + m33 + m9 ALL enforce 7-variant ordinal stability:

- m30: `EscapeSurfaceInconsistent { declared, derived }` returned when declared != derived 7-variant set
- m32: 5-check check[3] includes EscapeSurfaceProfile match; refuses if 6-variant carried by old workflow vs 7-variant spec
- m33: verification includes EscapeSurfaceProfile cardinality assertion
- m9: namespace-guard also asserts EscapeSurfaceProfile presence + cardinality on every write

## Runtime concerns (post-G9; placeholder)

| Concern | Pre-G9 status |
|---|---|
| Metrics emitted | DEFERRED — `bank_admissions_total`, `selector_substrate_degraded_total`, `dispatch_attempts`, `dispatch_5check_failures_per_check`, `verifier_quorum_outcomes` |
| Bench target | m32 5-check sequence < 200ms p99 (Conductor `/health` HTTP dominant cost) |
| Failure-mode | full set per [`../../ai_specs/ERROR_TAXONOMY.md`](../../ai_specs/ERROR_TAXONOMY.md) § Cluster G |

## Wave-1 build order (post-G9)

Cluster G ships **Days 8-10** post-G9 (after Cluster F KEYSTONE Days 5-7). Order within Cluster G:

1. m30 curated_bank (Day 8 — SQLite schema; EscapeSurfaceProfile 7-variant enforcement; HumanAcceptanceSignature flow)
2. m31 selector (Day 8-9 — substrate-aware decay-weighted)
3. m33 verifier (Day 9 — 4-agent quorum; independent of m32; ships first so m32 has a verification source)
4. m32 conductor_dispatcher (Day 10 — composes m30 + m31 + m33; Conductor RPC; emits CC-5 cascade trigger)

## HOLD-v2 compliance

This README is markdown only. **0** `.rs` files, **0** `Cargo.toml`, **0** code under `layers/cluster-G/`.

---

> **Back to:** [`../../README.md`](../../README.md) · sister [`../../ai_specs/layers/cluster-G.md`](../../ai_specs/layers/cluster-G.md) · [`../../ai_specs/substrate-couplings/CC-4-decomposed.md`](../../ai_specs/substrate-couplings/CC-4-decomposed.md)

*Filed 2026-05-17 (S1002127 Wave 4.B audit) · Command · planning-only · HOLD-v2 compliant.*
