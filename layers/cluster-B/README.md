---
cluster_id: B
name: Habitat Observers
modules: [m4, m5, m6]
binary: wf-crystallise
loc_estimate: ~460
substrates_touched: [S-A atuin (indirect via m1 cascade), engine-internal SQLite (m7 JSONB)]
date: 2026-05-17
status: SCAFFOLD (pre-G9; LIVE-on-G9-fire-day-3)
---

# layers/cluster-B — Habitat Observers (operational landing)

> **Back to:** [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · sister design spec [`../../ai_specs/layers/cluster-B.md`](../../ai_specs/layers/cluster-B.md) · per-module specs [`../../ai_specs/modules/cluster-B/`](../../ai_specs/modules/cluster-B/) · synergy [`../../ai_specs/synergies/CC-1.md`](../../ai_specs/synergies/CC-1.md) (cascade-cost coupling)

## What this cluster IS

Cluster B is the **observation layer** — three pure-function observers that synthesise raw substrate data into structured events:

- **Cascade correlator** (m4) — recognises multi-step command cascades from shell history rows
- **Battern step recorder** (m5) — captures protocol-step events (Battern begin/end) with `(battern_id, session_id)` tuples
- **Context-cost EMA** (m6) — tracks per-session context window utilisation with exponential moving average

The cluster operates **on m1's read output** (downstream of Cluster A); it does not touch substrates directly.

## Modules

| Module | Concern | Coupling | LOC | Spec |
|---|---|---|---|---|
| **m4** cascade_correlator | FNV-1a XOR opaque cascade IDs from `(session_id, cwd, ts)` batches | CC-1 with m7 JSONB | ~180 | [`m4_cascade_correlator.md`](../../ai_specs/modules/cluster-B/m4_cascade_correlator.md) |
| **m5** battern_step_record | Battern protocol step events (begin/end pairing); rejects nested batterns | CC-1.subA with m6 | ~120 | [`m5_battern_step_record.md`](../../ai_specs/modules/cluster-B/m5_battern_step_record.md) |
| **m6** context_cost | Per-session EMA on (token count, message count); convergence tracking | CC-1.subA with m5; CC-1 with m4 (both via m7 JSONB) | ~160 | [`m6_context_cost.md`](../../ai_specs/modules/cluster-B/m6_context_cost.md) |

## Cross-cluster contracts

- **CC-1 (Cascade-Cost Coupling)** — m4 + m6 NEVER call each other; both write `(cascade_id, session_id)` rows into m7's JSONB `consumer_inputs` column. The JSONB column is the **persistent coupling surface**, not a direct edge.
- **CC-1.subA (Battern-Cost Coupling)** — same structural pattern; m5 + m6 join via m7 JSONB on `(battern_id, session_id)`. Documented as a sub-contract of CC-1 (NOT a net-new 8th CC; per [`../../ai_specs/synergies/README.md`](../../ai_specs/synergies/README.md))
- **(no substrate-substrate edges)** — Cluster B is engine-internal (depends on m1's atuin reads but does not touch atuin directly); not decomposed in [`../../ai_specs/substrate-couplings/`](../../ai_specs/substrate-couplings/)

## Watcher class pre-position

- **Class D (four-surface drift)** — fires on any drift in m7's JSONB schema (which serves as the CC-1 / CC-1.subA join surface)
- **Class G (gradient signal)** — fires on m6 EMA convergence anomaly (gradient stalls)

## Runtime concerns (post-G9; placeholder)

| Concern | Pre-G9 status |
|---|---|
| Metrics emitted | DEFERRED |
| Bench target | m4 batch 2000: < 30ms (per [`../../ai_specs/BENCHMARK_SPEC.md`](../../ai_specs/BENCHMARK_SPEC.md)) |
| Failure-mode | EmptyTrajectory / MissingBegin / EmaConvergenceFailed (per [`../../ai_specs/ERROR_TAXONOMY.md`](../../ai_specs/ERROR_TAXONOMY.md) § Cluster B) |

## Wave-1 build order (post-G9)

Cluster B ships **Day 3** post-G9 (after Cluster D Day 1 trust scaffolding + Cluster A Day 2 readers). Order within Cluster B:

1. m4 cascade_correlator (purest function; no time-state)
2. m5 battern_step_record (additive structure)
3. m6 context_cost (depends on m4 + m5 outputs being well-formed)

## HOLD-v2 compliance

This README is markdown only. **0** `.rs` files, **0** `Cargo.toml`, **0** code under `layers/cluster-B/`.

---

> **Back to:** [`../../README.md`](../../README.md) · sister [`../../ai_specs/layers/cluster-B.md`](../../ai_specs/layers/cluster-B.md) · synergy [`../../ai_specs/synergies/CC-1.md`](../../ai_specs/synergies/CC-1.md)

*Filed 2026-05-17 (S1002127 Wave 4.B audit) · Command · planning-only · HOLD-v2 compliant.*
