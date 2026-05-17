---
title: Cluster B Scaffold — Module Specs S1002127
date: 2026-05-17
session: S1002127
kind: cluster-scaffold-note
cluster: B
layer: L2 (Habitat Observers)
module_count: 3
status: scaffold-only · per-module Rust spec markdown files authored · HOLD-v2 respected (NO `.rs`)
---

# Cluster B Scaffold — Module Specs S1002127

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Scaffold Wave 0-2 — Session S1002127]] · [[cluster-B-habitat-observers]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]

Cluster B is the **habitat-observer layer** (L2). Three modules sit between raw substrate ingest (Cluster A) and the correlation hub (Cluster C). They observe and shape — cascade correlation across opaque IDs, Battern-step record reconstruction, and context-cost tracking with an F10 EMA window. All three live in the `wf-crystallise` binary, default feature gate, and produce structured inputs for the m7 SQLite hub.

## Modules in this cluster

| # | Module | Repo spec | LOC | Tests | Reads from | Writes to | Lift % |
|---|---|---|---:|---:|---|---|---:|
| 4 | `m4_cascade_correlator` | [`../ai_specs/modules/cluster-B/m4_cascade_correlator.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-B/m4_cascade_correlator.md) | 160 | 60 | m1 iter | m7 row | 50% |
| 5 | `m5_battern_step_record` | [`../ai_specs/modules/cluster-B/m5_battern_step_record.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-B/m5_battern_step_record.md) | 130 | 60 | m1 iter | m20 input | 35% |
| 6 | `m6_context_cost` | [`../ai_specs/modules/cluster-B/m6_context_cost.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-B/m6_context_cost.md) | 170 | 60 | m1 iter | m7 JSONB column | 30% |

**Cluster total:** ~460 LOC source + 180 tests.

## Cross-cluster contracts

- **Owns:** CC-1 partial (cascade-cost coupling — input side); m5 feeds the KEYSTONE iteration cluster (F)
- **Consumes:** Cluster A iter streams (m1 primarily)
- **Feeds:** m7 (C — central hub); m20 (F — PrefixSpan miner)

## Boilerplate references

- `04-pattern-detection/` (cascade correlation patterns from orac-sidecar)
- `01-cli-scaffolding/` (loop-engine-v2 + habitat-conductor)

## Status

All 3 specs are at **SPEC**. m5 is on the critical path for the Cluster F KEYSTONE work (PrefixSpan needs reliable step-record input). HOLD-v2 still forbids `.rs`.

## Bidirectional anchors

> **Vault cluster canonical:** [[cluster-B-habitat-observers]] (deep design)
> **Repo specs:** [`../ai_specs/modules/cluster-B/`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-B/)
> **Project charter:** [`../CLAUDE.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.md) · [`../CLAUDE.local.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.local.md)
