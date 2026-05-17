---
cluster_id: C
name: Correlation + Output
modules: [m7, m12, m13]
binary: wf-crystallise
loc_estimate: ~370
substrates_touched: [engine-internal SQLite (workflow_runs.db), S-C stcortex (m13 write)]
date: 2026-05-17
status: SCAFFOLD (pre-G9; LIVE-on-G9-fire-day-3)
---

# layers/cluster-C — Correlation + Output (operational landing)

> **Back to:** [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · sister design spec [`../../ai_specs/layers/cluster-C.md`](../../ai_specs/layers/cluster-C.md) · per-module specs [`../../ai_specs/modules/cluster-C/`](../../ai_specs/modules/cluster-C/) · substrate dossier [`../../ai_specs/substrates/stcortex.md`](../../ai_specs/substrates/stcortex.md)

## What this cluster IS

Cluster C is the **persistence + output layer** — the central `workflow_runs` SQLite table (the engine's "what happened" ledger), the CLI report surface, and the read-side stcortex writer used by m23 (proposals) and m42 (Hebbian reinforcement) for substrate writes.

## Modules

| Module | Concern | Substrate touch | LOC | Spec |
|---|---|---|---|---|
| **m7** workflow_runs | central SQLite table; JSONB `consumer_inputs` column serves CC-1 + CC-1.subA join; **F9 zero-weight violation** detection | engine-internal SQLite | ~150 | [`m7_workflow_runs.md`](../../ai_specs/modules/cluster-C/m7_workflow_runs.md) |
| **m12** cli_reports | `wf-report` CLI subcommands — summarise workflow runs, pressure, acceptance, consent budget | engine-internal SQLite reads | ~120 | [`m12_cli_reports.md`](../../ai_specs/modules/cluster-C/m12_cli_reports.md) |
| **m13** stcortex_writer (narrowed-scope) | write-side bridge to S-C stcortex `:3000`; AP30 namespace enforcement; offline-snapshot fallback | S-C stcortex (sole write-side gateway) | ~100 | [`m13_stcortex_writer.md`](../../ai_specs/modules/cluster-C/m13_stcortex_writer.md) |

## Cross-cluster contracts

- **CC-1 / CC-1.subA (consumer of join surface)** — m7's JSONB `consumer_inputs` is the persistent coupling surface for m4 / m5 / m6
- **CC-3 (E → F evidence flow)** — m7 reads inform m14 (lift evidence) downstream
- **CC-5 (substrate learning loop — write half)** — m13 is the only write path to stcortex; m23 (proposals) and m42 (reinforce) both call through m13. CC-5 is decomposed in [`../../ai_specs/substrate-couplings/CC-5-decomposed.md`](../../ai_specs/substrate-couplings/CC-5-decomposed.md) — E1 specifically depends on m13's write succeeding.

## Watcher class pre-position

- **Class D (four-surface drift)** — fires on m7 JSONB schema drift (which breaks CC-1)
- **Class B (hand-off boundary)** — fires on m13 stcortex write attempts that miss AP30 namespace (m9 catches first; defense-in-depth)

## Substrate-side concerns

m13 is the **sole gateway** to S-C stcortex writes. All substrate-side discipline lives in [`../../ai_specs/substrates/stcortex.md`](../../ai_specs/substrates/stcortex.md):

- AP30 namespace prefix mandatory
- AP-Hab-11 hyphen-slug encoding at slug boundary
- `register_consumer` per-session step required (per workspace [`CLAUDE.md`](../../CLAUDE.md) memory row 8)
- Offline-snapshot read-fallback if `:3000` unreachable; NEVER POVM fallback
- Substrate-drift canary participation (CR-2 inflation case)

## Runtime concerns (post-G9; placeholder)

| Concern | Pre-G9 status |
|---|---|
| Metrics emitted | DEFERRED |
| Bench target | m7 single insert < 5ms, bulk 1000 < 500ms |
| Failure-mode | SchemaDrift / F9ZeroWeightViolation / SubstrateUnavailable (per [`../../ai_specs/ERROR_TAXONOMY.md`](../../ai_specs/ERROR_TAXONOMY.md) § Cluster C) |
| Refusal token | m13 `SubstrateAuthored { S-C, ... }` per [`../../ai_specs/cross-cutting/refusal-taxonomy.md`](../../ai_specs/cross-cutting/refusal-taxonomy.md) |

## Wave-1 build order (post-G9)

Cluster C ships **Day 3** post-G9 (parallel with Cluster B; both depend on Cluster A readers from Day 2). Order within Cluster C:

1. m7 workflow_runs (schema + JSONB column; F9 enforcement)
2. m13 stcortex_writer (narrowed-scope; smoke-tested against live `:3000` if available)
3. m12 cli_reports (depends on m7 schema being stable)

## HOLD-v2 compliance

This README is markdown only. **0** `.rs` files, **0** `Cargo.toml`, **0** code under `layers/cluster-C/`.

---

> **Back to:** [`../../README.md`](../../README.md) · sister [`../../ai_specs/layers/cluster-C.md`](../../ai_specs/layers/cluster-C.md) · [`../../ai_specs/substrates/stcortex.md`](../../ai_specs/substrates/stcortex.md)

*Filed 2026-05-17 (S1002127 Wave 4.B audit) · Command · planning-only · HOLD-v2 compliant.*
