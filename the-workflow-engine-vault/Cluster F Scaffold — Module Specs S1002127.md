---
title: Cluster F Scaffold — Module Specs S1002127
date: 2026-05-17
session: S1002127
kind: cluster-scaffold-note
cluster: F
layer: L6 (Iteration — KEYSTONE)
module_count: 4
status: scaffold-only · per-module Rust spec markdown files authored · HOLD-v2 respected (NO `.rs`)
keystone: true
---

# Cluster F Scaffold — Module Specs S1002127

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Scaffold Wave 0-2 — Session S1002127]] · [[cluster-F-iteration]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]

Cluster F is the **iteration KEYSTONE** (L6) — the structural-authorship gap that cannot be lifted from boilerplate. PrefixSpan miner (m20) + variant builder (m21) + k-means feature extractor (m22) + workflow proposer (m23) compose into the **N-step compositional sub-graph detection** primitive (Gap 1; ~600-1,000 LOC of NEW algorithm work; Levenshtein similarity + Wilson CI integration). All four carry `intelligence` feature gate and the active verb-class `recommend*` (permitted under v1.3 single-phase override).

## Modules in this cluster

| # | Module | Repo spec | LOC | Tests | Test kind | Gap | Lift % |
|---|---|---|---:|---:|---|---|---:|
| 20 | `m20_prefixspan_miner` | [`../ai_specs/modules/cluster-F/m20_prefixspan_miner.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-F/m20_prefixspan_miner.md) | 300 | **75** | property+bench | **Gap 1** | 0% (NEW) |
| 21 | `m21_variant_builder` | [`../ai_specs/modules/cluster-F/m21_variant_builder.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-F/m21_variant_builder.md) | 200 | 65 | property | Gap 1 | 5% |
| 22 | `m22_kmeans_feature` | [`../ai_specs/modules/cluster-F/m22_kmeans_feature.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-F/m22_kmeans_feature.md) | 170 | 60 | property | Gap 1 | 20% |
| 23 | `m23_workflow_proposer` | [`../ai_specs/modules/cluster-F/m23_workflow_proposer.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-F/m23_workflow_proposer.md) | 180 | 60 | property | Gap 1 | 10% |

**Cluster total:** ~850 LOC source + 260 tests. **Highest test density in the codebase (75 tests for m20 PrefixSpan) per [TEST_DISCIPLINE](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md).**

## Cross-cluster contracts

- **Owns:** CC-4 (proposal → bank → dispatch pipeline ownership starts here at m23)
- **Consumes:** m5 step records (B), m14 outcome lift (E), m7 stats indirectly
- **Feeds:** m30 (G — curated bank) via m23 operator review queue

## Why KEYSTONE

The N-step compositional sub-graph detection primitive (Gap 1) is **the structural-authorship reason this codebase needs to exist** — no existing habitat service does PrefixSpan over Battern step records with Wilson-CI-gated variant proposal. If Cluster F drops to lift, the codebase loses its differentiator and becomes a thinner wrapper around HABITAT-CONDUCTOR. RALPH selector-without-measurement risk is acknowledged in [[../PRIME_DIRECTIVE_WAIVER|PRIME_DIRECTIVE_WAIVER]] § waiver record.

## Boilerplate references

- m20: 0% lift (PrefixSpan is novel here; canonical algorithm reference only)
- m21: 5% lift (variant-shape patterns from synthex-v2)
- m22: 20% lift (k-means scaffolding; common pattern)
- m23: 10% lift (proposer-shape patterns from operator-review surfaces)

## Status

All 4 specs are at **SPEC**. **Gap 1 KEYSTONE work cannot be parallelised across pre-G9 sub-agents** — m20 must land first; m21/m22 can follow; m23 lands last and feeds the operator review queue. HOLD-v2 still forbids `.rs`.

## Bidirectional anchors

> **Vault cluster canonical:** [[cluster-F-iteration]]
> **Repo specs:** [`../ai_specs/modules/cluster-F/`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-F/)
> **Project charter:** [`../CLAUDE.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.md) · [`../CLAUDE.local.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.local.md)
