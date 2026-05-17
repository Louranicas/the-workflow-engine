---
title: Cluster G Scaffold — Module Specs S1002127
date: 2026-05-17
session: S1002127
kind: cluster-scaffold-note
cluster: G
layer: L7 (Bank + Select + Dispatch + Verify)
module_count: 4
status: scaffold-only · per-module Rust spec markdown files authored · HOLD-v2 respected (NO `.rs`)
---

# Cluster G Scaffold — Module Specs S1002127

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Scaffold Wave 0-2 — Session S1002127]] · [[cluster-G-bank-select-dispatch-verify]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]

Cluster G is the **bank + select + dispatch + verify layer** (L7). It is the only cluster owned entirely by the `wf-dispatch` binary (m30-m33). m30 curates the workflow bank (post-operator-review entries from m23); m31 selects (diversity-enforced; consumes m11 compound decay); m32 dispatches via HABITAT-CONDUCTOR (never directly to host shell — owns Gap 3 unified destructiveness schema); m33 verifies (4-agent verifier composed pattern returning PASS/FAIL/DEGRADED to m30).

## Modules in this cluster

| # | Module | Repo spec | LOC | Tests | Test kind | Gap |
|---|---|---|---:|---:|---|---|
| 30 | `m30_curated_bank` | [`../ai_specs/modules/cluster-G/m30_curated_bank.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-G/m30_curated_bank.md) | 220 | 70 | integration | **Gap 3** partial |
| 31 | `m31_selector` | [`../ai_specs/modules/cluster-G/m31_selector.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-G/m31_selector.md) | 240 | 70 | property | — |
| 32 | `m32_conductor_dispatcher` | [`../ai_specs/modules/cluster-G/m32_conductor_dispatcher.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-G/m32_conductor_dispatcher.md) | 290 | 75 | integration | **Gap 3** primary |
| 33 | `m33_verifier` | [`../ai_specs/modules/cluster-G/m33_verifier.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-G/m33_verifier.md) | 200 | 70 | integration | — |

**Cluster total:** ~950 LOC source + 285 tests. **m32 + m33 are blocking on Conductor Waves 1B/1C/2/3 being LIVE (currently `auto_start=false`; see [[../CLAUDE.local.md]] B3).**

## Cross-cluster contracts

- **Owns:** CC-4 (proposal → bank → dispatch downstream of m23), CC-6 (verification-gated dispatch m33 → m32), Gap 3 (unified destructiveness / EscapeSurfaceProfile schema co-owned m30 + m32 + m9)
- **Consumes:** m23 operator review (F), m11 compound decay (D)
- **Feeds:** HABITAT-CONDUCTOR (external; via m32); m40/m41/m42 (H — substrate feedback)

## Two-binary split discipline

m30-m33 live in `wf-dispatch` (the only modules NOT in `wf-crystallise`). Per [[Modules Synergy Clusters and Feature Verification S1001982]] this prevents read-heavy crystallisation from accidentally triggering dispatch (a class-of-bug the habitat-loop-engine ancestor died from). The shared `workflow_core` lib in the same crate (ORAC pattern, not LCM workspace) handles types/schemas/namespace constants.

## Boilerplate references

- `07-conductor-dispatch/` (habitat-conductor + dev-ops-engine-v3 dispatcher patterns)
- `09-trap-verify-escape-skills/` (m30 + m32 lift EscapeSurfaceProfile shape from `.claude/skills/`)

## Status

All 4 specs are at **SPEC**. m32 is the highest-risk module (Gap 3 NEW schema + external HABITAT-CONDUCTOR dependency). Command-3 owns this lane in the librarian standby split. HOLD-v2 still forbids `.rs`.

## Bidirectional anchors

> **Vault cluster canonical:** [[cluster-G-bank-select-dispatch-verify]]
> **Repo specs:** [`../ai_specs/modules/cluster-G/`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-G/)
> **Project charter:** [`../CLAUDE.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.md) · [`../CLAUDE.local.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.local.md)
