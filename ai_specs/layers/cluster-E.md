---
title: Cluster E — Evidence + Pressure (Layer L5) — Layer Spec
cluster: E
layer: L5
module_count: 2
modules: [m14_habitat_outcome_lift, m15_pressure_register]
binary: wf-crystallise
feature_gates: [intelligence]
cc_owns: [CC-3 (Evidence-Driven Iteration; m14 owns), CC-7 (Pressure-Driven Evolution; m15 owns)]
cc_consumes: [CC-1, CC-2]
ship_priority: Day 2 Wave 2 (after Cluster C hub lands)
status: SPEC
date: 2026-05-17
hold_v2_compliant: true
---

# Cluster E — Evidence + Pressure (L5)

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · vault [[cluster-E-evidence-pressure]] · [`../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-E.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-E.md)

## Role

Cluster E IS the **evidence + pressure layer** — two modules that turn correlated `workflow_runs` rows into two different downstream surfaces. **m14** computes Wilson-CI bounded `Lift` (success-rate uplift of a workflow vs. baseline) gated on n≥20 minimum sample size; this evidence quad is the input gate for Cluster F's iteration KEYSTONE (m20/m21/m22/m23). **m15** is the pressure register — a JSONL file-drop emitter that surfaces forbidden-verb attempts, sample-size relaxation pressure, scope-relaxation pressure, handshake silence, and escape-surface escalation events to `~/projects/shared-context/agent-cross-talk/` for Watcher / Zen / Luke consumption.

The cluster is the engine's **substrate-grain evidence + meta-grain pressure** producer. m14 produces the *evidence* that gates iteration (CC-3 — without n≥20 Wilson-CI lift, m20 emits no pattern; m23 ProposalBuilder rejects at construction). m15 produces the *pressure* that drives v1.4+ spec amendments (CC-7 — pressure-driven evolution loop runs through human deliberation via agent-cross-talk file-drops). These are two distinct evolution loops at two distinct timescales: m14 closes the per-cycle gate every selection round; m15 closes the per-amendment gate every 1-N weeks.

## Modules

| Module | Spec | LOC | Tests | Produces |
|---|---|---:|---:|---|
| `m14_habitat_outcome_lift` | [`../modules/cluster-E/m14_habitat_outcome_lift.md`](../modules/cluster-E/m14_habitat_outcome_lift.md) | 110 | 70 | `Option<Lift>` (None below n=20) |
| `m15_pressure_register` | [`../modules/cluster-E/m15_pressure_register.md`](../modules/cluster-E/m15_pressure_register.md) | 90 | 60 | JSONL events to agent-cross-talk |

## Cross-cluster contracts

- **OWNS:**
  - **CC-3 (Evidence-Driven Iteration)** — m14's `Option<Lift>::None` below n=20 is the gate. m20 PrefixSpan filters its iteration to evidence-permitted clusters; m22 K-means gates per-cluster n≥20; m23 ProposalBuilder rejects `LiftEvidenceMissing` at construction. See [`../synergies/CC-3.md`](../synergies/CC-3.md).
  - **CC-7 (Pressure-Driven Evolution)** — m15's JSONL emission to `~/projects/shared-context/agent-cross-talk/PHASE-B-RESERVATION-NOTICE-*.jsonl` is the engine's structural pressure surface. The Watcher reads it on tick cadence; Zen audits forbidden-verb pressure at G7 cadence; Luke deliberates v1.4+ amendments. See [`../synergies/CC-7.md`](../synergies/CC-7.md).
- **CONSUMES:**
  - **CC-1** — m14 reads `workflow_runs` via the m7 JSONB join; cascade + cost signals visible from the same row.
  - **CC-2** — m9 wraps any namespace-bearing string m14 or m15 emits (m15's filename namespace, m14's reporting namespace).

## Binary placement

Both modules in **`wf-crystallise`**. m14 is an evidence-producing read-side function over m7; m15 is a file-drop emitter (no network). Neither belongs in `wf-dispatch`.

## Feature-gate posture

**`feature = "intelligence"`** gated. m14's Wilson-CI evidence calculation and m15's pressure-event detection are intelligence-class operations; in a minimal build (no iteration, no spec-amendment loop) they can be omitted. The `intelligence` gate is on in `full` profile, off in `default`. Implication: with intelligence off, Cluster F (m20-m23 iteration) cannot link — that is structurally correct, since iteration without evidence is exactly what F2 (sample-size inflation) bans.

## Ship priority

**Day 2 Wave 2** (after m7 hub lands so m14 has rows to read). Implementation order: m15 (pure JSONL emitter; no DB dependency; the simpler module) → m14 (Wilson-CI arithmetic + m7 read path; gates downstream Cluster F).

## Operational invariants

1. **m14 NEVER returns `Some(Lift)` for n<20.** This is the structural gate; `compute_lift(success, total)` returns `Ok(None)` when `total < 20`, not `Ok(Some(Lift { ci: wide }))`. Downstream m20-m23 pattern-match on `None` to skip; consumers cannot read a "soft" wide-CI lift as if it were evidence.
2. **m14 Wilson-CI lower-bound is the gate value.** Lift is reported as `(lower_bound, point, upper_bound)`; m31's selection composite-score uses the lower bound, never the point estimate. Optimism is structurally banned.
3. **m15 JSONL is atomic-write (tmp + rename).** Concurrent fleet writes are common; partial JSONL files would break Watcher's reader. Every emission goes to `*.tmp` first then `mv` to canonical name.
4. **m15 de-dup window 60s.** Same `(event_kind, context_hash)` within 60s collapses to one file with `count: N`. This prevents pressure-flooding under fleet hot-path conditions.
5. **m15 filename convention is contract-binding.** `PHASE-B-RESERVATION-NOTICE-{ts}_{event_id}.jsonl` — Watcher's reader pattern-matches; renaming breaks the reader. See [`../synergies/CC-7.md`](../synergies/CC-7.md) § Step 2.

## Failure modes the cluster structurally refuses

- **m14 returning a wide-CI `Some(Lift { ci: 0.95 })` to flatter downstream.** Wilson-CI says "below n=20, your CI is too wide to be evidence" — m14's contract is `Option`, not "always Some with wider CI". Returning Some would let m20 iterate on noise; F2 violation.
- **m15 batched/append-mode writes.** Append mode means a process crash mid-write leaves a corrupted JSONL. One-file-per-event with atomic rename is the contract.
- **m14 reading non-m7 rows.** m14's evidence comes from the central hub; reading raw m1/m2/m3 substrate directly would create a parallel evidence path and a CC-1 schema-drift surface.
- **m15 pressure suppression.** Adding a "if too many events, suppress" branch is rejected — pressure accumulation IS the signal; suppression is what CC-7 evolution loop runs against (loop never closes if pressure is hidden).

## Performance envelope

| Operation | Target | Notes |
|---|---|---|
| m14 `compute_lift(success, total)` | < 100 ns | pure arithmetic; Wilson-CI is closed-form |
| m14 `lift_for_cluster(cluster_id)` | < 10 ms | single m7 query covered by `idx_runs_cluster_outcome` |
| m15 `emit(event)` | < 5 ms | tmp file write + rename; no fsync (Watcher reads on cadence, not real-time) |
| m15 de-dup window check | < 50 µs | in-memory `parking_lot::Mutex<HashMap<(EventKind, u64), Instant>>` |

## Verify-sync invariants

- **#11** — `m14::compute_lift` doc has `# Errors` section (returns `Result<Option<Lift>, LiftError>` for DB read errors); same for m15 emit.
- **#16** — F2 mitigation test asserts `compute_lift(19, 20)` returns `Ok(None)`; `compute_lift(20, 20)` returns `Ok(Some(...))`.
- **#16** — F-CC-7 mitigation test asserts m15 emission filename matches `^PHASE-B-RESERVATION-NOTICE-\d+_[a-f0-9]+\.jsonl$`.

## Per-module cross-links

- [`../modules/cluster-E/m14_habitat_outcome_lift.md`](../modules/cluster-E/m14_habitat_outcome_lift.md) — Wilson-CI bounded lift + n=20 gate
- [`../modules/cluster-E/m15_pressure_register.md`](../modules/cluster-E/m15_pressure_register.md) — JSONL pressure emitter + 60s de-dup window

## Antipatterns specific to Cluster E

- **AP-WT-F2** (sample-size inflation) — m14 returns `None` below n=20; downstream m23 ProposalBuilder rejects at construction with `LiftEvidenceMissing`.
- **AP-V7-13** (diagnostics theatre) — m14 Wilson-CI is structurally bounded; no "estimated lift" sentinel with no underlying probe.
- **AP-Hab-class refusal-mode violation** (sycophancy) — m14 never widens CI to flatter; m15 never suppresses pressure.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · vault [[cluster-E-evidence-pressure]]
