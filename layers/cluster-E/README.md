---
cluster_id: E
name: Evidence + Pressure
modules: [m14, m15]
binary: wf-crystallise
loc_estimate: ~200
substrates_touched: [engine-internal SQLite (workflow_runs read), S-G operator (m15 pressure → operator)]
date: 2026-05-17
status: SCAFFOLD (pre-G9; LIVE-on-G9-fire-day-4)
---

# layers/cluster-E — Evidence + Pressure (operational landing)

> **Back to:** [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · sister design spec [`../../ai_specs/layers/cluster-E.md`](../../ai_specs/layers/cluster-E.md) · per-module specs [`../../ai_specs/modules/cluster-E/`](../../ai_specs/modules/cluster-E/) · synergy [`../../ai_specs/synergies/CC-3.md`](../../ai_specs/synergies/CC-3.md) + [`CC-7.md`](../../ai_specs/synergies/CC-7.md) · substrate-coupling [`../../ai_specs/substrate-couplings/CC-7-decomposed.md`](../../ai_specs/substrate-couplings/CC-7-decomposed.md)

## What this cluster IS

Cluster E is the **evidence + pressure layer** — two modules that surface "is what we're doing actually working?" signals:

- **Lift metric (m14)** — Wilson-CI confidence interval on workflow run outcomes, computed against baseline; feeds m23 proposal evidence
- **Pressure register (m15)** — JSONL event stream of substrate refusals + bench regressions + watcher escalations + dispatch failures; **the operator's primary attention surface**

Cluster E is the **CC-7 pressure-driven evolution origin** — pressure rows surface to operator, operator deliberates, operator authors spec amendments (per [`../../ai_specs/substrate-couplings/CC-7-decomposed.md`](../../ai_specs/substrate-couplings/CC-7-decomposed.md)).

## Modules

| Module | Concern | Coupling | LOC | Spec |
|---|---|---|---|---|
| **m14** habitat_outcome_lift | Wilson confidence interval on `(workflow_id, outcome)` pairs from m7; reports lift vs baseline | CC-3 → m20 (PrefixSpan input) | ~80 | [`m14_habitat_outcome_lift.md`](../../ai_specs/modules/cluster-E/m14_habitat_outcome_lift.md) |
| **m15** pressure_register | JSONL event sink: refusal cascades, bench regressions, Watcher escalations, F3/F7/F11 fire | CC-7 → operator deliberation | ~120 | [`m15_pressure_register.md`](../../ai_specs/modules/cluster-E/m15_pressure_register.md) |

## Cross-cluster contracts

- **CC-3 (Evidence-Driven Iteration, E → F)** — m14's lift evidence is m23's input; m20's PrefixSpan iterator uses m14's confidence-weighted scores
- **CC-7 (Pressure-Driven Evolution, E → operator)** — m15 is the entry point to CC-7; the substrate-coupling decomposition lives in [`../../ai_specs/substrate-couplings/CC-7-decomposed.md`](../../ai_specs/substrate-couplings/CC-7-decomposed.md) (4 edges: m15→S-G, S-G→spec amendment fanout, S-G→S-watcher Ember gate, S-G fatigue→m12 consent budget)

## Watcher class pre-position

- **Class C (refusal)** — fires on m15 pressure-event rate-of-arrival exceeding threshold
- **Class G (gradient signal)** — fires on m14 lift-delta stalls over rolling window
- **Class E (Watcher's own escalation class)** — m15 surfaces Watcher-emitted escalations alongside engine-side pressure

## Substrate-side concerns (NA-GAP-05 operator-as-substrate)

m15's primary downstream is **S-G operator** (per [`../../ai_specs/substrates/operator.md`](../../ai_specs/substrates/operator.md) + [`../../ai_specs/substrate-couplings/CC-7-decomposed.md`](../../ai_specs/substrate-couplings/CC-7-decomposed.md)):

- Operator-substrate has its own consent budget; sustained pressure rate exceeding threshold triggers `OperatorRefusal { Luke, ConsentFatigue }`
- Pressure rotation policy must pin to operator-ack cadence (not time-based) to avoid the JSONL-rotated-before-read race
- `pressure_acknowledged_at` receipt (per CC-7-decomposed E1) — proposed substrate-confirmable receipt to be written by m12 on operator read

## Runtime concerns (post-G9; placeholder)

| Concern | Pre-G9 status |
|---|---|
| Metrics emitted | DEFERRED |
| Bench target | m14 Wilson CI computation < 5ms |
| Failure-mode | InvalidSampleCount / DedupCacheOverflow / Io (per [`../../ai_specs/ERROR_TAXONOMY.md`](../../ai_specs/ERROR_TAXONOMY.md) § Cluster E) |

## Wave-1 build order (post-G9)

Cluster E ships **Day 4** post-G9 (after Cluster D Day 1, Cluster A Day 2, Cluster B/C Day 3). Order within Cluster E:

1. m15 pressure_register (JSONL writer; simple structure)
2. m14 habitat_outcome_lift (depends on m7 schema being stable from Cluster C)

## v0.2.0 deferral note (NA-GAP-07)

Per [`../../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`](../../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md) W1, the proposed `m16_substrate_drift_canary` module would live in Cluster E (or a new Cluster I). v0.1.0 ships without m16; substrate-drift detection is **distributed** across existing modules per [`../../ai_specs/cross-cutting/substrate-drift.md`](../../ai_specs/cross-cutting/substrate-drift.md) canary contract participation.

## HOLD-v2 compliance

This README is markdown only. **0** `.rs` files, **0** `Cargo.toml`, **0** code under `layers/cluster-E/`.

---

> **Back to:** [`../../README.md`](../../README.md) · sister [`../../ai_specs/layers/cluster-E.md`](../../ai_specs/layers/cluster-E.md) · [`../../ai_specs/substrate-couplings/CC-7-decomposed.md`](../../ai_specs/substrate-couplings/CC-7-decomposed.md)

*Filed 2026-05-17 (S1002127 Wave 4.B audit) · Command · planning-only · HOLD-v2 compliant.*
