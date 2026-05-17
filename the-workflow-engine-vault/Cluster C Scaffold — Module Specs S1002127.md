---
title: Cluster C Scaffold — Module Specs S1002127
date: 2026-05-17
session: S1002127
kind: cluster-scaffold-note
cluster: C
layer: L3 (Correlation + Output)
module_count: 3
status: scaffold-only · per-module Rust spec markdown files authored · HOLD-v2 respected (NO `.rs`)
---

# Cluster C Scaffold — Module Specs S1002127

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Scaffold Wave 0-2 — Session S1002127]] · [[cluster-C-correlation-output]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]

Cluster C is the **correlation + output layer** (L3). m7 is the central SQLite hub (`workflow_runs` table; F9 zero-weight unrated rows; CC-1 contract owner). m12 reads m7 and emits CLI reports (stdout / JSON). m13 reads m7 + the m2 trust signal and writes back to stcortex `:3000` (CC-2 + CC-5 contract owner). All three live in `wf-crystallise`, feature gate `api`.

## Modules in this cluster

| # | Module | Repo spec | LOC | Tests | Test kind | Reads from | Writes to | Lift % |
|---|---|---|---:|---:|---|---|---|---:|
| 7 | `m7_workflow_runs` | [`../ai_specs/modules/cluster-C/m7_workflow_runs.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-C/m7_workflow_runs.md) | 140 | 60 | integration | m3, m4, m6 | SQLite hub | 40% |
| 12 | `m12_cli_reports` | [`../ai_specs/modules/cluster-C/m12_cli_reports.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-C/m12_cli_reports.md) | 110 | 60 | unit | m7 | stdout / JSON | 65% |
| 13 | `m13_stcortex_writer` | [`../ai_specs/modules/cluster-C/m13_stcortex_writer.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-C/m13_stcortex_writer.md) | 120 | 60 | async | m7, m2 trust | stcortex `:3000` | 45% |

**Cluster total:** ~370 LOC source + 180 tests.

## Cross-cluster contracts

- **Owns:** CC-1 (hub via m7 JSONB columns), CC-2 partial (m13 stcortex write side), CC-5 partial (m13 substrate-feedback emit on stcortex)
- **Consumes:** all of Cluster B output + m3 from Cluster A + m2 trust signal
- **Feeds:** every downstream cluster reads m7

## Boilerplate references

- `03-sqlite-multi-db/` (memory-injection + habitat-buoy multi-DB patterns; transactional write side)
- `02-stcortex-consumer/` (m13 write client mirrors m2 read client)
- `01-cli-scaffolding/` (m12 reports)

## Status

All 3 specs are at **SPEC**. m7 is the **central hub** — must land first under any post-G9 dependency ordering. HOLD-v2 still forbids `.rs`.

## Bidirectional anchors

> **Vault cluster canonical:** [[cluster-C-correlation-output]]
> **Repo specs:** [`../ai_specs/modules/cluster-C/`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-C/)
> **Project charter:** [`../CLAUDE.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.md) · [`../CLAUDE.local.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.local.md)
