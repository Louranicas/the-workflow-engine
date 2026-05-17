---
title: Cluster D Scaffold — Module Specs S1002127
date: 2026-05-17
session: S1002127
kind: cluster-scaffold-note
cluster: D
layer: L4 (Trust — cross-cutting)
module_count: 4
status: scaffold-only · per-module Rust spec markdown files authored · HOLD-v2 respected (NO `.rs`)
---

# Cluster D Scaffold — Module Specs S1002127

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Scaffold Wave 0-2 — Session S1002127]] · [[cluster-D-trust-cross-cutting]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]

Cluster D is the **trust layer** — an **aspect cross-cutting** all of Clusters A-H. It carries no feature gate (it must always compile and always assert) and is locked to ship **Day 1** under any post-G9 plan; lift density is intentionally low (mostly NEW primitives or guard wrappers). m9 owns the namespace guard (Gap 3 authorship), m11 owns the `frequency × fitness × recency` compound decay formula (Gap 2 NEW PRIMITIVE — KEYSTONE for selector input).

## Modules in this cluster

| # | Module | Repo spec | LOC | Tests | Gap | Lift % | Verb |
|---|---|---|---:|---:|---|---:|---|
| 8 | `m8_povm_build_prereq` | [`../ai_specs/modules/cluster-D/m8_povm_build_prereq.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-D/m8_povm_build_prereq.md) | 60 | 50 | — | 0% (NEW) | refuse |
| 9 | `m9_watcher_namespace_guard` | [`../ai_specs/modules/cluster-D/m9_watcher_namespace_guard.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-D/m9_watcher_namespace_guard.md) | 60 | 50 | **Gap 3** | 20% | refuse |
| 10 | `m10_ember_ci_gate` | [`../ai_specs/modules/cluster-D/m10_ember_ci_gate.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-D/m10_ember_ci_gate.md) | 90 | 60 | — | 15% | refuse |
| 11 | `m11_fitness_weighted_decay` | [`../ai_specs/modules/cluster-D/m11_fitness_weighted_decay.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-D/m11_fitness_weighted_decay.md) | 90 | 70 | **Gap 2** | 0% (NEW) | record |

**Cluster total:** ~300 LOC source + 230 tests. **Two NEW-primitive modules (m8, m11) and one structural-authorship gap (m9 namespace guard).**

## Cross-cluster contracts

- **Owns:** CC-2 (trust layer woven into every other cluster)
- **Consumes:** build env (m8), namespace.rs constants (m9), Ember rubric (m10), m7 stats (m11)
- **Feeds:** m30 (G — selector input via m11), CI gate (m10), runtime asserts (m9), build refusal (m8)

## Notes & dependencies

- m10 carries the Ember §5.1 Held-semantics amendment **awaiting Watcher's lane** (see [[../CLAUDE.local.md]] B4)
- m11 formula: `score = freq^α × fitness^β × recency_decay(λ)`; α/β/λ TBD at G5 interview
- Day-1 ship-order: m8 → m9 → m10 → m11 before **any** Cluster A reader fires

## Boilerplate references

- `09-trap-verify-escape-skills/` (m9 namespace-guard pattern lifts from `hookify.preserve-blanket-guard`)
- `04-pattern-detection/` (m11 sliding-window decay shape from synthex-v2 m20)
- `05-decay-ttl-ltd/` (m11 LTD reference shape from povm-v2 and orac-sidecar)

## Status

All 4 specs are at **SPEC**. m11 is on the KEYSTONE critical path (Selector m31 cannot ship without it). HOLD-v2 still forbids `.rs`.

## Bidirectional anchors

> **Vault cluster canonical:** [[cluster-D-trust-cross-cutting]]
> **Repo specs:** [`../ai_specs/modules/cluster-D/`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_specs/modules/cluster-D/)
> **Project charter:** [`../CLAUDE.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.md) · [`../CLAUDE.local.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.local.md)
