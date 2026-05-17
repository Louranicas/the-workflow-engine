---
title: Cluster E Scaffold — Module Specs S1002127
date: 2026-05-17
session: S1002127
kind: cluster-scaffold-note
cluster: E
layer: L5 (Evidence + Pressure)
module_count: 2
status: scaffold-only · per-module Rust spec markdown files authored · HOLD-v2 respected (NO `.rs`)
---

# Cluster E Scaffold — Module Specs S1002127

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Scaffold Wave 0-2 — Session S1002127]] · [[cluster-E-evidence-pressure]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]

Cluster E is the **evidence + pressure layer** (L5). m14 emits the engine-wide habitat-outcome-lift metric (Wilson CI on success-rate deltas) — primary signal feeding the Cluster F iteration loop. m15 maintains a pressure register (JSONL append-only event stream) that captures runtime pressure events for downstream operator review and post-mortems. Both modules carry the `intelligence` feature gate.

## Modules in this cluster

| # | Module | Repo spec | LOC | Tests | Test kind | Writes to |
|---|---|---|---:|---:|---|---|
| 14 | `m14_habitat_outcome_lift` | [`../ai_specs/modules/cluster-E/m14_habitat_outcome_lift.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-E/m14_habitat_outcome_lift.md) | 110 | 70 | property | m22, m23 input |
| 15 | `m15_pressure_register` | [`../ai_specs/modules/cluster-E/m15_pressure_register.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-E/m15_pressure_register.md) | 90 | 60 | unit | JSONL files |

**Cluster total:** ~200 LOC source + 130 tests.

## Cross-cluster contracts

- **Owns:** CC-3 (evidence-driven iteration via m14), CC-7 (pressure-driven evolution via m15)
- **Consumes:** m7 (workflow_runs hub) for outcome lift; runtime signals for pressure
- **Feeds:** m22, m23 (F — iteration); operator spec interviews (CC-7)

## Boilerplate references

- `04-pattern-detection/` (Wilson CI + fitness measurement patterns)
- `05-decay-ttl-ltd/` (JSONL emit patterns from memory-injection + povm-v2)

## Status

Both specs are at **SPEC**. m14 is on the iteration critical path (Cluster F cannot select without lift signal). HOLD-v2 still forbids `.rs`.

## Bidirectional anchors

> **Vault cluster canonical:** [[cluster-E-evidence-pressure]]
> **Repo specs:** [`../ai_specs/modules/cluster-E/`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-E/)
> **Project charter:** [`../CLAUDE.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.md) · [`../CLAUDE.local.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.local.md)
