---
title: Cluster C — Central Correlation + Output (Layer L3) — Layer Spec
cluster: C
layer: L3
module_count: 3
modules: [m7_workflow_runs, m12_cli_reports, m13_stcortex_writer]
binary: wf-crystallise
feature_gates: [api]
cc_owns: [CC-1 (m7 JSONB hub)]
cc_consumes: [CC-1, CC-2, CC-5]
ship_priority: Day 2 Wave 2 (after Cluster B observation lands)
status: SPEC
date: 2026-05-17
hold_v2_compliant: true
---

# Cluster C — Central Correlation + Output (L3)

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · vault [[cluster-C-correlation-output]] · [`../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-C.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-C.md)

## Role

Cluster C IS the **central correlation hub and external output surface**. m7 owns the canonical `workflow_runs` SQLite table — the single join point where Cluster A reads (m3 causal-chain context) and Cluster B observations (m4 cascade, m5 battern, m6 cost) become correlated `WorkflowRunRow` records. The JSONB `consumer_inputs` column is the CC-1 stable schema-coupling surface: any module that needs to read multiple Cluster B observation streams reads them off the same row, never via direct intra-cluster call. m12 owns the CLI report output surface (stdout / structured JSON) and m13 owns the narrowed-scope stcortex writer that emits correlated rows back to the pioneer substrate.

The cluster is the **F9 zero-weight column** owner — `workflow_runs.fitness_dimension` is structurally NOT NULL but starts at neutral zero; m11 (Cluster D decay) and m32 (Cluster G dispatcher outcomes) are the only legitimate mutators. m7's schema discipline is the load-bearing structural fact that makes CC-1 work without spaghetti — every consumer reads/writes the same column with the same serde shape. m13 is the only stcortex writer in workflow-trace; all other write paths (m42 substrate-feedback) go through m13 as the underlying transport, enforcing the AP30 namespace prefix at exactly one code site.

## Modules

| Module | Spec | LOC | Tests | Reads | Writes |
|---|---|---:|---:|---|---|
| `m7_workflow_runs` | [`../modules/cluster-C/m7_workflow_runs.md`](../modules/cluster-C/m7_workflow_runs.md) | 140 | 60 | m3, m4, m6 | SQLite `workflow_runs` |
| `m12_cli_reports` | [`../modules/cluster-C/m12_cli_reports.md`](../modules/cluster-C/m12_cli_reports.md) | 110 | 60 | m7 | stdout / JSON |
| `m13_stcortex_writer` | [`../modules/cluster-C/m13_stcortex_writer.md`](../modules/cluster-C/m13_stcortex_writer.md) | 120 | 60 | m7, m2 trust signal | stcortex `:3000` |

## Cross-cluster contracts

- **OWNS:**
  - **CC-1 (Cascade-Cost Coupling, hub)** — m7's `consumer_inputs` JSONB column is the join surface. m4 (cascade) and m6 (cost) write into the same row's JSONB; m14, m20, m23 read both signals from the same row. m7 is the ONLY module that can ALTER this schema; consumers depend on it via shared `workflow_core::schemas` definitions.
- **CONSUMES:**
  - **CC-2** — m13 writes are wrapped by m9 namespace-guard (every write asserts `workflow_trace_*` prefix); m13 invocation requires m2 registration success (CC-2 trust signal).
  - **CC-5** — m13 is the stcortex transport m42 routes substrate-feedback through; the CC-5 substrate-learning loop closes via the same writer path m7 uses for arc-outcome correlation.

## Binary placement

All three modules in **`wf-crystallise`**. m7 owns the central correlation DB; m12 produces CLI reports from it; m13 emits correlated rows to stcortex. None of these are dispatch operations — `wf-dispatch` reads m30 (its own bank DB), not m7.

## Feature-gate posture

Cluster C is **`feature = "api"`** gated. The rationale: m12's CLI report endpoints and m13's stcortex emit are external surfaces; in a minimal build (no observability, no external substrate writes) they can be omitted. The `api` gate is on by default; turning it off produces a degraded "headless ingest" build used only for debugging Cluster A/B in isolation.

## Ship priority

**Day 2 Wave 2** (after Cluster A + Cluster B land). Implementation order: m7 (schema + write API) → m12 (CLI reports — fast feedback for human review) → m13 (stcortex writer — last, because it requires m9 namespace-guard from CC-2 trust layer to be live and asserting).

## Operational invariants

1. **m7 is the sole owner of `workflow_runs` schema.** Any column ADD/RENAME/DROP requires a migration filed under `migrations/` and a verify-sync-invariant test asserting the consumer schemas still resolve. Drift between m7's serde struct and the SQLite column-set fails the contract test at gate.
2. **F9 zero-weight column.** `fitness_dimension` defaults to 0.0 (neutral) at row insert. m32 (dispatch outcome) and m11 (decay) are the only mutators. Read paths NEVER assume a non-zero value implies "this row was used"; n=0 dispatch counts is the bootstrap state, not a signal.
3. **JSONB consumer_inputs as coupling surface.** m4 cascade and m6 cost write into the same JSONB blob with stable JSON-Schema-validated shapes. The schema lives in `workflow_core::schemas::ConsumerInputs`; consumers deserialize through that type only, never `serde_json::Value` ad hoc.
4. **m13 enforces AP30 at exactly one site.** Every stcortex write path eventually calls `m13::write(namespace_id, payload)`; `namespace_id` is `workflow_core::namespace::NamespaceId` (newtype, constructor private to `namespace.rs`). Literal `"workflow_trace_*"` strings cannot reach `:3000` because there is no string constructor for `NamespaceId`.
5. **m12 CLI is read-only and idempotent.** `workflow-trace report --since SHA --until SHA` produces identical output for identical inputs. Reports do not mutate `workflow_runs` (no `last_reported_at` write).

## Failure modes the cluster structurally refuses

- **Two writers for one column.** Adding a second m7 write API surface (e.g., a "fast-path insert" shortcut) creates a CC-1 schema-drift surface and is rejected at review.
- **Inline namespace strings.** `m13::write("workflow_trace_outcome_pass", payload)` is a compile error because `&str` is not `NamespaceId`. The only way to construct `NamespaceId` is via the const constructor accepting a `workflow_core::namespace::*` constant.
- **m12 mutating m7.** Reports are derived views; if a report wants a denormalised projection, it computes it in memory or asks m7 for a precomputed view — never writes back.
- **m13 falling back to a different substrate on stcortex outage.** stcortex-down is `Err(StcortexError::Unreachable)` returned to caller; m13 does NOT fall through to POVM (workflow-trace is POVM-decoupled per Genesis v1.3 § 2 + ADR 2026-05-17). The outbox-first JSONL durability layer carries the record for offline-snapshot replay.

## Performance envelope

| Operation | Target | Notes |
|---|---|---|
| m7 `insert_workflow_run` | < 5 ms | single-row insert, WAL, indexed on (`workflow_id`, `created_at`) |
| m7 `query_runs(since, limit=1000)` | < 50 ms | covered by `idx_runs_created_at` |
| m12 `report --json` | < 200 ms (10k rows) | streaming serde, no full in-memory aggregation |
| m13 `write(namespace, payload)` | < 100 ms p99 | HTTP POST to `:3000` + outbox append; outbox is the durability path |

## Verify-sync invariants

- **#2** + **#3** — every src/ module in Cluster C has an `ai_specs/modules/cluster-C/` entry and vice versa.
- **#11** — every public fn in m7/m12/m13 returning `Result` has a `# Errors` doc section.
- **#13** — every async fn in m13 uses `#[tracing::instrument(skip(payload))]`.
- **#15** — `use workflow_core::namespace::*` is the ONLY namespace-string source in m13; verified by `rg '"workflow_trace_' src/m13_stcortex_writer/` returning 0.

## Per-module cross-links

- [`../modules/cluster-C/m7_workflow_runs.md`](../modules/cluster-C/m7_workflow_runs.md) — central correlation hub + CC-1 owner
- [`../modules/cluster-C/m12_cli_reports.md`](../modules/cluster-C/m12_cli_reports.md) — CLI report formatter (stdout + JSON)
- [`../modules/cluster-C/m13_stcortex_writer.md`](../modules/cluster-C/m13_stcortex_writer.md) — narrowed-scope stcortex writer with 3-band LTP/LTD gate

## Antipatterns specific to Cluster C

- **AP-WT-F9** (zero-weight column collapse) — `fitness_dimension` defaults to 0.0 and only m32/m11 mutate; read-paths never infer from absence.
- **AP-Hab-03** (AP30 namespace drift) — m13 is the single enforcement site; `NamespaceId` newtype with private string constructor.
- **AP-Drift-06** (bridge contract drift) — m13 wire-format contract tests pin the `:3000` payload shape under `tests/contract/m13_stcortex_wire.rs`.
- **AP-V7-13** (Health-200 ≠ behaviour-verified) — m13 NEVER takes HTTP 200 on `/v1/pathway` as proof that pathway-weights moved; verification belongs to the Class-I Watcher rolling-window check.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · vault [[cluster-C-correlation-output]]
