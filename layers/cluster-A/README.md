---
cluster_id: A
name: Substrate Ingest
modules: [m1, m2, m3]
binary: wf-crystallise
loc_estimate: ~230
substrates_touched: [S-A atuin, S-C stcortex (read), S-B injection.db]
date: 2026-05-17
status: SCAFFOLD (pre-G9; LIVE-on-G9-fire-day-2)
---

# layers/cluster-A — Substrate Ingest (operational landing)

> **Back to:** [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · sister design spec [`../../ai_specs/layers/cluster-A.md`](../../ai_specs/layers/cluster-A.md) · per-module specs [`../../ai_specs/modules/cluster-A/`](../../ai_specs/modules/cluster-A/) · substrate dossiers [`../../ai_specs/substrates/atuin.md`](../../ai_specs/substrates/atuin.md) · [`../../ai_specs/substrates/stcortex.md`](../../ai_specs/substrates/stcortex.md) · [`../../ai_specs/substrates/injection_db.md`](../../ai_specs/substrates/injection_db.md)

## What this cluster IS

Cluster A is the **read-side substrate ingest layer** — the engine's three substrate-readers that pull observation data IN from atuin (shell history), stcortex (pioneer memory pathways), and injection.db (causal-chain reinforcement counts). Cluster A is **read-only** by discipline: writes belong to Cluster H.

## Modules

| Module | Substrate | Read concern | LOC | Spec |
|---|---|---|---|---|
| **m1** atuin_consumer | S-A atuin (file-path SQLite, WAL-mode) | paginated cursor on `history` table; observe `atuin kv get` for fleet state; respect `wal_size_bytes` back-pressure | ~80 | [`m1_atuin_consumer.md`](../../ai_specs/modules/cluster-A/m1_atuin_consumer.md) |
| **m2** stcortex_consumer (narrowed-scope) | S-C stcortex (`:3000` SpacetimeDB) | narrowed namespace `workflow_trace_*` reads; offline-JSON-snapshot fallback per CLAUDE.md stcortex policy | ~90 | [`m2_stcortex_consumer.md`](../../ai_specs/modules/cluster-A/m2_stcortex_consumer.md) |
| **m3** injection_db_consumer | S-B injection.db (SQLite via habitat-memory daemon) | read `causal_chain` rows with `workflow_trace_*` prefix; observe `reinforcement_count` deltas | ~60 | [`m3_injection_db_consumer.md`](../../ai_specs/modules/cluster-A/m3_injection_db_consumer.md) |

## Per-substrate co-tenant discipline

Each m1/m2/m3 follows the substrate-as-co-tenant rule (per [`../../ai_specs/substrates/INDEX.md`](../../ai_specs/substrates/INDEX.md)):

- **m1 ↔ atuin**: read-only via `mode=ro` URI + `PRAGMA query_only=ON`; m1 NEVER causes WAL contention; on `database is locked` rate > 1/min, m1 halves cadence and emits substrate-drift candidate
- **m2 ↔ stcortex**: paginated `SELECT * FROM <table> LIMIT N OFFSET M` with bounded result size; if `:3000` unreachable, fall back to `data/snapshots/latest.json` (NEVER fall back to POVM — per workspace [`CLAUDE.md`](../../CLAUDE.md) memory row 8); if snapshot stale > 1 hr → refuse read per m42 § 5.1.c snapshot-staleness threshold
- **m3 ↔ injection.db**: read during habitat-memory daemon idle window (pause during hourly TTL sweep); m3 NEVER writes (habitat-memory daemon owns writes)

## Cross-cluster contracts

- **CC-1 cascade-cost coupling** — m1's `(session_id, cwd, ts)` tuples join m4 (cascade correlator); m1 owns the read-side; m4 owns the correlation
- **CC-1.subA battern-cost coupling** — m1 → m5 via shared `session_id` within `battern_id` range
- **(no CC-5 participation)** — Cluster A is read-only; CC-5 substrate feedback is Cluster H's write-side

## Runtime concerns (post-G9; placeholder pre-G9)

| Concern | Pre-G9 status |
|---|---|
| Metrics emitted | DEFERRED — per-module READMEs added at G9-fire (see [`../../modules/README.md`](../../modules/README.md)) |
| Health-probe contribution | DEFERRED |
| Operational toggles | spec'd in [`../../ai_specs/modules/cluster-A/`](../../ai_specs/modules/cluster-A/) §2 of each module |
| Failure-mode escalation | refusal-token taxonomy per [`../../ai_specs/cross-cutting/refusal-taxonomy.md`](../../ai_specs/cross-cutting/refusal-taxonomy.md) §SubstrateRefusalClass |
| Capacity envelope | substrate-side benches per [`../../ai_specs/BENCHMARK_SPEC.md`](../../ai_specs/BENCHMARK_SPEC.md) § substrate-side load benchmarks |

## Wave-1 build order (post-G9)

Cluster A ships **Day 2** of post-G9 implementation (Cluster D ships Day 1 — trust scaffolding precedes readers per [`../../CLAUDE.md`](../../CLAUDE.md) § Cluster D Day 1 framework). Order within Cluster A:

1. m1 atuin_consumer (most isolated; minimal substrate-edge)
2. m3 injection_db_consumer (similar shape to m1)
3. m2 stcortex_consumer (narrowed-scope; snapshot fallback is more involved)

## Substrate-drift awareness

Per [`../../ai_specs/cross-cutting/substrate-drift.md`](../../ai_specs/cross-cutting/substrate-drift.md), all three modules participate in the canary contract. Drift indicators (from substrate dossiers):

- **S-A atuin:** schema_migration_silent, history_table_column_rename, wal_checkpoint_cadence_change, kv_namespace_relocation
- **S-C stcortex:** schema_drift_post_redeploy (CR-2 origin — `learning_health` 13.6× inflation factor)
- **S-B injection.db:** habitat-memory daemon TTL-sweep race; schema migration column-add

## HOLD-v2 compliance

This README is markdown only. **0** `.rs` files, **0** `Cargo.toml`, **0** code under `layers/cluster-A/`. Subdir contains only this landing pre-G9.

---

> **Back to:** [`../../README.md`](../../README.md) · sister [`../../ai_specs/layers/cluster-A.md`](../../ai_specs/layers/cluster-A.md) · per-module [`../../ai_specs/modules/cluster-A/`](../../ai_specs/modules/cluster-A/)

*Filed 2026-05-17 (S1002127 Wave 4.B audit) · Command · planning-only · HOLD-v2 compliant.*
