---
title: Cluster A Scaffold — Module Specs S1002127
date: 2026-05-17
session: S1002127
kind: cluster-scaffold-note
cluster: A
layer: L1 (Substrate Ingest)
module_count: 3
status: scaffold-only · per-module Rust spec markdown files authored · HOLD-v2 respected (NO `.rs`)
---

# Cluster A Scaffold — Module Specs S1002127

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Scaffold Wave 0-2 — Session S1002127]] · [[cluster-A-substrate-ingest]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]

Cluster A is the **substrate ingest layer** (L1). Three modules read from the three habitat substrates that are already authoritative for command history, pattern-bearing memory, and causal-chain reinforcement state. All three modules are passive (verb-class: ingest/record), all live in the `wf-crystallise` binary, none take a feature gate (default), and none own a structural-authorship gap (they lift heavily from boilerplate).

## Modules in this cluster

| # | Module | Repo spec | LOC | Tests | Reads from | Lift % |
|---|---|---|---:|---:|---|---:|
| 1 | `m1_atuin_consumer` | [`../ai_specs/modules/cluster-A/m1_atuin_consumer.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-A/m1_atuin_consumer.md) | 80 | 50 | `~/.local/share/atuin/history.db` | 30% |
| 2 | `m2_stcortex_consumer` | [`../ai_specs/modules/cluster-A/m2_stcortex_consumer.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-A/m2_stcortex_consumer.md) | 80 | 50 | stcortex `:3000` (narrowed-scope client) | 25% |
| 3 | `m3_injection_db_consumer` | [`../ai_specs/modules/cluster-A/m3_injection_db_consumer.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-A/m3_injection_db_consumer.md) | 70 | 50 | `~/.local/share/habitat/injection.db` | 40% |

**Cluster total:** ~230 LOC source + 150 tests.

## Cross-cluster contracts

- **Owns:** none (consumer-only cluster)
- **Consumes:** atuin SQLite schema, stcortex WebSocket subscription, injection.db schema v4
- **Feeds:** m4/m5 (B), m3 → m7 (C hub), m2 trust signal → m13 (C)

## Boilerplate references

- `01-cli-scaffolding/` (habitat-conductor CLI shape)
- `02-stcortex-consumer/` (stcortex client + onboarding doc)
- `03-sqlite-multi-db/` (injection.db reader patterns from memory-injection)

Full clones at [[boilerplate modules/BOILERPLATE_INDEX|BOILERPLATE_INDEX]].

## Status

All 3 specs are at **SPEC** in [[../ai_specs/MODULE_MATRIX|MODULE_MATRIX.md]]. WIP fires post-G9 only. HOLD-v2 still forbids `.rs` source files.

## Bidirectional anchors

> **Vault cluster canonical:** [[cluster-A-substrate-ingest]] (deep design)
> **Repo specs:** [`../ai_specs/modules/cluster-A/`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-A/)
> **Project charter:** [`../CLAUDE.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.md) · [`../CLAUDE.local.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.local.md)
> **Architecture:** [`../ARCHITECTURE.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ARCHITECTURE.md) · [[Modules Synergy Clusters and Feature Verification S1001982]]
