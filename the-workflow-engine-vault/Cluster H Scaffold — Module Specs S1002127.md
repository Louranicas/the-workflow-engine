---
title: Cluster H Scaffold — Module Specs S1002127
date: 2026-05-17
session: S1002127
kind: cluster-scaffold-note
cluster: H
layer: L8 (Substrate Feedback)
module_count: 3
status: scaffold-only · per-module Rust spec markdown files authored · HOLD-v2 respected (NO `.rs`)
note: m42 POVM **DECOUPLED** per 2026-05-17 m42-stcortex-only ADR
---

# Cluster H Scaffold — Module Specs S1002127

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Scaffold Wave 0-2 — Session S1002127]] · [[cluster-H-substrate-feedback]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]

Cluster H is the **substrate feedback layer** (L8) — the closing loop where dispatch results from Cluster G flow back to the habitat substrates as typed events / RPCs / Hebbian-reinforce pathways. m40 emits NexusEvent to SYNTHEX v2 (:8092). m41 calls LCM via MCP RPC. m42 emits stcortex-only (POVM **decoupled** post m42 ADR — see [[../CLAUDE.local.md]] V7 Optimisation + m42 stcortex-only pivot). All three carry the `monitoring` feature gate.

## Modules in this cluster

| # | Module | Repo spec | LOC | Tests | Test kind | Writes to |
|---|---|---|---:|---:|---|---|
| 40 | `m40_nexusevent_emit` | [`../ai_specs/modules/cluster-H/m40_nexusevent_emit.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-H/m40_nexusevent_emit.md) | 160 | 65 | async | SYNTHEX `:8092/v3/nexus/push` |
| 41 | `m41_lcm_rpc` | [`../ai_specs/modules/cluster-H/m41_lcm_rpc.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-H/m41_lcm_rpc.md) | 140 | 65 | async | LCM MCP stdio JSON-RPC |
| 42 | `m42_stcortex_emit` | [`../ai_specs/modules/cluster-H/m42_stcortex_emit.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-H/m42_stcortex_emit.md) | 150 | 70 | async | stcortex pathways (POVM **decoupled**) |

**Cluster total:** ~450 LOC source + 200 tests.

## m42 pivot (substrate routing — important)

Per the 2026-05-17 ADR ([`ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md)): module renamed `src/m42_povm_dual/` → `src/m42_stcortex_emit/`. POVM dependency removed pre-deployment. Featureset preserved 1:1 via stcortex. Risk surface reduced. Triggered AP-V7-13 ("Health-200 ≠ behaviour-verified"). 48/48 grilling-round decisions accepted.

## Cross-cluster contracts

- **Owns:** CC-5 (substrate learning loop closure) via all three modules
- **Consumes:** m32 result (G), m13 stcortex client (C) for m42
- **Feeds:** SYNTHEX v2 (external), LCM (external), stcortex pathways (external) — closes loop back to F via stcortex Hebbian reinforcement

## Boilerplate references

- `08-nexus-lcm-rpc/` (orac-sidecar + dev-ops-engine-v3 RPC clients)
- `02-stcortex-consumer/` (m42 write pattern mirrors m2 read pattern)
- `06-daemon-scaffolding/` (async background-task patterns from synthex-v2 + habitat-nerve-center)

## Status

All 3 specs are at **SPEC**. m42 carries the post-pivot v1.3 amendment surface (Zen G7 verdict pending). HOLD-v2 still forbids `.rs`.

## Bidirectional anchors

> **Vault cluster canonical:** [[cluster-H-substrate-feedback]]
> **Repo specs:** [`../ai_specs/modules/cluster-H/`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-H/)
> **m42 ADR:** [`../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md) · [[optimisation-v7/m42 stcortex-only pivot ADR]]
> **Project charter:** [`../CLAUDE.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.md) · [`../CLAUDE.local.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.local.md)
