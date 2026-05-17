# bin/wf-crystallise/

> **Back to:** [`../README.md`](../README.md) · [`../../README.md`](../../README.md) · [`../../CLAUDE.md`](../../CLAUDE.md) · [`../../CLAUDE.local.md`](../../CLAUDE.local.md)

**Status:** placeholder. `main.rs` lands post-G9 (HOLD-v2 forbids `.rs` files).

## Purpose

`wf-crystallise` is the **read-heavy binary** of the workflow-trace two-binary split. It ingests substrates (Cluster A), observes the habitat (Cluster B), correlates and outputs (Cluster C), gates on trust (Cluster D), measures evidence (Cluster E), iterates compositionally (Cluster F — KEYSTONE), and emits substrate feedback (Cluster H). It does **not** dispatch — dispatch is owned exclusively by the sibling [`../wf-dispatch/`](../wf-dispatch/) binary. The hard split is the structural defence against the class-of-bug that killed the `habitat-loop-engine` ancestor: a single binary where ingest could accidentally trigger dispatch via shared state.

## Owned modules (per [`../../ai_specs/MODULE_MATRIX.md`](../../ai_specs/MODULE_MATRIX.md))

- **Cluster A (substrate ingest):** m1, m2, m3 — atuin / stcortex consumer / injection.db
- **Cluster B (habitat observers):** m4, m5, m6 — cascade correlator / Battern step record / context cost
- **Cluster C (correlation + output):** m7 (workflow_runs hub), m12 (CLI reports), m13 (stcortex writer)
- **Cluster D (trust cross-cutting):** m8, m9, m10, m11 — build prereq / namespace guard / Ember CI / fitness-weighted decay
- **Cluster E (evidence + pressure):** m14 (habitat-outcome-lift), m15 (pressure register)
- **Cluster F (iteration KEYSTONE):** m20 (PrefixSpan), m21 (variant builder), m22 (k-means), m23 (workflow proposer)
- **Cluster H (substrate feedback):** m40 (NexusEvent emit), m41 (LCM RPC), m42 (stcortex emit — POVM **decoupled** per m42 ADR)

Total: **23 of 26 modules** (m30, m31, m32, m33 are owned by `wf-dispatch`).

## CLI entry point (planning)

```text
wf-crystallise [--config PATH] <command> [args]
  ingest                  # one-shot: ingest substrates, observe, correlate, write m7
  watch                   # continuous: tail substrates, emit lift + pressure
  iterate                 # run Cluster F (PrefixSpan + variant builder) once
  report [--json|--text]  # m12 CLI reports
  emit-feedback           # m40/m41/m42 substrate-feedback push for ratified runs
  health                  # health endpoint check (used by devenv)
```

Final CLI shape locks at G5 interview / G7 spec audit. Above is the Town Hall directional sketch.

## Feature gate coverage

Defaults to `default` (Clusters A/B/C minimal). Other gates per [`../../ai_specs/MODULE_MATRIX.md`](../../ai_specs/MODULE_MATRIX.md): `api` (Cluster C full), `intelligence` (Clusters E + F), `monitoring` (Cluster H), `evolution` (Cluster D aspect-layer always on; declared via the matrix). Cluster D **does NOT take a feature gate** — it must always compile and always assert.

## Boilerplate references

Lift from [`../../the-workflow-engine-vault/boilerplate modules/`](../../the-workflow-engine-vault/boilerplate%20modules/) categories `01-cli-scaffolding/`, `02-stcortex-consumer/`, `03-sqlite-multi-db/`, `04-pattern-detection/`, `05-decay-ttl-ltd/`, `06-daemon-scaffolding/`, `08-nexus-lcm-rpc/`, `10-foundation-direct-clones/`. Total lift ~65% per [[the-workflow-engine-vault/MASTER_INDEX|MASTER_INDEX]] § 6.

> **Back to:** [`../README.md`](../README.md) · [`../../README.md`](../../README.md) · [`../../CLAUDE.md`](../../CLAUDE.md) · [`../../CLAUDE.local.md`](../../CLAUDE.local.md)
