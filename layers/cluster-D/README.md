---
cluster_id: D
name: Trust (cross-cutting aspect — L4)
modules: [m8, m9, m10, m11]
binary: wf-crystallise (m8 build, m9/m10/m11 runtime)
loc_estimate: ~300
substrates_touched: [S-watcher (m10 Ember CI gate), S-G operator (m10 string review)]
date: 2026-05-17
status: SCAFFOLD (pre-G9; SHIPS DAY 1 post-G9-fire — non-negotiable phase-1 framework)
ship_first: true
---

# layers/cluster-D — Trust (operational landing)

> **Back to:** [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · sister design spec [`../../ai_specs/layers/cluster-D.md`](../../ai_specs/layers/cluster-D.md) · per-module specs [`../../ai_specs/modules/cluster-D/`](../../ai_specs/modules/cluster-D/) · substrate dossier [`../../ai_specs/substrates/watcher.md`](../../ai_specs/substrates/watcher.md)

## What this cluster IS

Cluster D is the **trust aspect layer** — four orthogonal trust mechanisms that cross-cut all other clusters. Per [`../../CLAUDE.md`](../../CLAUDE.md) § Cluster D ships Day 1 framework, **Cluster D ships FIRST** post-G9: trust scaffolding precedes any reader (Cluster A) or observer (Cluster B) or correlator (Cluster C) so the engine's first operations are gated by the trust apparatus.

This is the **most consequential build-order decision** in the engine — every other module's first commit lands against an already-active trust gate.

## Modules

| Module | Concern | Ship type | LOC | Spec |
|---|---|---|---|---|
| **m8** povm_build_prereq | build-script gate (compile-time only); `cfg(povm_calibrated)` cfg flag check; legacy POVM probe — to be renamed `cfg(stcortex_calibrated)` per m42 spec open question 4 | build-time | ~40 | [`m8_povm_build_prereq.md`](../../ai_specs/modules/cluster-D/m8_povm_build_prereq.md) |
| **m9** namespace_guard | runtime AP30 namespace prefix validator; refuses any write with id outside `workflow_trace_*` | runtime; defense-in-depth | ~60 | [`m9_namespace_guard.md`](../../ai_specs/modules/cluster-D/m9_namespace_guard.md) |
| **m10** ember_ci_gate | hybrid CI-FAIL + allowlist for user-facing strings; Ember §5.1 unanimity check via Watcher (per D-C Luke S1002127); AP27 self-mod refusal | CI-time | ~80 | [`m10_ember_ci_gate.md`](../../ai_specs/modules/cluster-D/m10_ember_ci_gate.md) |
| **m11** fitness_weighted_decay | NEW PRIMITIVE: `frequency × fitness × recency` compound decay; m31 selector consumer; the engine's selection-bias mechanism | runtime | ~120 | [`m11_fitness_weighted_decay.md`](../../ai_specs/modules/cluster-D/m11_fitness_weighted_decay.md) |

## Cross-cluster contracts

- **CC-2 (Trust Layer Woven, D → all)** — m9 namespace guard is invoked at every substrate-write boundary (m13, m42); m10 Ember CI gate runs against every user-facing string change in all clusters; m11 decay feeds m31 selector in Cluster G
- **No substrate-substrate edges** — Cluster D is engine-internal trust enforcement; the substrate-mediated trust variant (NA-GAP-10) is **deferred to v0.2.0** per [`../../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`](../../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md) W3

## Watcher class pre-position

- **Class A (activation)** — fires on m10's first Ember gate result
- **Class D (four-surface drift)** — fires on m9 namespace constant drift
- **Class C (refusal)** — fires on m10 EmberUnanimityHeld / AP27SelfModRefused (per [`../../ai_specs/cross-cutting/refusal-taxonomy.md`](../../ai_specs/cross-cutting/refusal-taxonomy.md))

## Cluster D Day-1 sequence (non-negotiable)

Per workspace [`../../CLAUDE.md`](../../CLAUDE.md) and Genesis Prompt v1.3 § Day-1 framework, the order is:

1. **m8 build-script cfg** — establishes the compile-time gate; ensures every subsequent commit goes through the cfg
2. **m9 namespace_guard** — first runtime trust enforcement; m13 and m42's first writes will hit this
3. **m10 ember_ci_gate** — third; user-facing strings reviewed (hybrid CI-FAIL+allowlist per D-C)
4. **m11 fitness_weighted_decay** — last; m31 selector wires through this in Cluster G build

Cluster A readers (Day 2) cannot ship until Cluster D Day 1 fires — the namespace guard must be live before m13 attempts its first stcortex write.

## Substrate-side concerns

m10's Ember CI gate touches **S-watcher** (per [`../../ai_specs/substrates/watcher.md`](../../ai_specs/substrates/watcher.md)):
- Watcher refuses with `EmberUnanimityFailed` if §5.1 Held verdict — engine surfaces `OperatorRefusal { Watcher, EmberUnanimityHeld }`
- Watcher refuses with `AP27SelfModRefused` if change touches `src/m8_watcher/*` — operator must split PR
- During Watcher R13 quiet period, m10 surfaces "ungated" — operator must NOT interpret as "approved"

m10 also touches **S-G operator** (string author + Held-resolution flow) — see [`../../ai_specs/substrates/operator.md`](../../ai_specs/substrates/operator.md).

## Runtime concerns (post-G9; placeholder)

| Concern | Pre-G9 status |
|---|---|
| Metrics emitted | DEFERRED — per-module post-G9 |
| CI integration | m10 runs in PR pipeline; m8 in cargo-check pipeline |
| Failure-mode | NamespaceViolation / EmberUnanimityHeld / AP27SelfModRefused / InvalidInput (per [`../../ai_specs/ERROR_TAXONOMY.md`](../../ai_specs/ERROR_TAXONOMY.md) § Cluster D) |

## HOLD-v2 compliance

This README is markdown only. **0** `.rs` files, **0** `Cargo.toml`, **0** code under `layers/cluster-D/`. The Day-1 framework documented here is the post-G9 build-order spec; it does NOT pre-author code.

---

> **Back to:** [`../../README.md`](../../README.md) · sister [`../../ai_specs/layers/cluster-D.md`](../../ai_specs/layers/cluster-D.md) · [`../../ai_specs/substrates/watcher.md`](../../ai_specs/substrates/watcher.md) · [`../../GATE_STATE.md`](../../GATE_STATE.md)

*Filed 2026-05-17 (S1002127 Wave 4.B audit) · Command · planning-only · HOLD-v2 compliant · Day-1 ship-first.*
