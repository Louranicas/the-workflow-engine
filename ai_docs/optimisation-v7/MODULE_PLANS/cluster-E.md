---
title: MODULE PLAN — Cluster E (Evidence + Pressure) · m14 lift / m15 pressure
date: 2026-05-17
kind: planning-only · per-module spec · no code authorised (HOLD-v2 + AP24)
cluster: E
layer: L5
modules: [m14, m15]
loc_budget: ~200
test_budget: 110
mutation_kill_targets: { m14: 80%, m15: 70% }
authority: Command · workflow-trace V7 optimisation
---

# Cluster E — Evidence + Pressure (L5)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · [[../GENERATIONS/G3-bidi-flow.md]] · sibling [[cluster-D.md]] · [[cluster-F.md]] · [[CROSS_CLUSTER_SYNERGIES.md]]
>
> **Function:** Cluster E is the bridge between **central correlation (Cluster C)** and **iteration (Cluster F)**. m14 produces the *evidence signal* that gates whether iteration is even permitted (F2 sample-size hard gate); m15 emits the *pressure signal* that surfaces forbidden-verb / scope-relaxation attempts to Watcher/Zen via file-drop (CC-7 closure point). Together they implement **evidence-driven iteration without evidence-driven dispatch** — Cluster E never dispatches, never selects, only *measures* and *signals*.

---

## Overview

Cluster E sits at L5 in the layer DAG, downstream of m7 (central hub, Cluster C) and upstream of m23 (proposer, Cluster F) + m31 (selector, Cluster G). The cluster has exactly two modules and is the smallest non-aspect cluster in the engine, but it is the **discipline gate** for two of the four most-load-bearing failure modes: F2 (sample-size inflation) and CC-7 closure (pressure-driven spec evolution).

The cluster's discipline rule: **never emit a synthetic value when measurement is below the threshold**. m14 returns `Option<Lift>::None` rather than `Lift { mean: 0.0, .. }`; m15 emits an explicit pressure event rather than silently relaxing an invariant. Both modules treat `None` and `Pressure` as load-bearing semantic — refusing-to-emit is the engineered behaviour, not a degenerate case (per G6 substrate-frame pass + Watcher Class-C semantics).

Per ULTRAMAP View 2 the cluster totals ~200 LOC across two modules — by far the lowest LOC density in the engine (~100 LOC/module). Compensating discipline: ~55 tests/module average; mutation kill targets set above standard for m14 (80%, Wilson-CI math correctness is critical).

---

## m14 — habitat_outcome_lift (L5 · `src/m14_lift/`)

### Purpose

Compute the **lift metric** that distinguishes a workflow from random baseline, with statistical rigour. The output is the single signal that gates whether m20-m22 can iterate on a workflow at all (CC-3 closure) AND the modulation signal for m31 selector weight perturbation. Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part I § Cluster E:

```
habitat_outcome_lift = 0.6 × cascade_success_rate + 0.4 × cost_lift
```

with Wilson 95% CI (z=1.96) bracketing the point estimate.

### Edge contract (from [[../GENERATIONS/G3-bidi-flow.md]] § Cluster E)

- **Upstream-IN:** `m7.WorkflowRunRow` stream (n≥20 trigger; per-workflow accumulation)
- **Downstream-OUT:** `Option<Lift { wilson_low: f64, mean: f64, wilson_high: f64, n: usize }>` → m23 (proposer evidence-gate), m31 (selector modulation `clamp(-0.3, +0.3)`)
- **Aspect-IN:** m8 build_prereq (compile-time `cfg(povm_calibrated)`), m9 namespace_guard (read-side AP30 validator on workflow_id namespacing)
- **Aspect-OUT:** N/A
- **Failure-mode mitigated:** F2 — returns `None` when n<20; never inflates with synthetic value

### src/ path (planning-spec only)

```
src/m14_lift/
├── mod.rs                # public surface: `pub struct Lift`, `pub fn compute_lift`, `pub enum LiftError`
├── wilson.rs             # Wilson CI numerics (NOT Wald — negative lower bounds at small n forbidden)
├── cost_lift.rs          # cost-side computation (reads m6 ContextCostBand baseline)
├── cascade_lift.rs       # cascade-success-rate computation (reads m7 WorkflowRunRow)
├── modulation.rs         # m31-bound delta computation: clamp(-0.3, +0.3)
└── tests/                # see test allocation below
```

### LOC budget (per ULTRAMAP View 2)

~120 LOC implementation + ~5 LOC public surface re-exports. Of this, ~30-40 LOC are pure Wilson-CI numerics (the only mathematically dense surface).

### Test budget (60 tests; per [[../STANDARDS/TEST_DISCIPLINE.md]] matrix)

| Family | Count | Notes |
|---|---:|---|
| F-Unit | 30 | per-arm coverage; Wilson CI edge cases (n=20 boundary, n=1000, p=0.0, p=1.0, p=0.5) |
| F-Property | 8 | invariants: wilson_low ≤ mean ≤ wilson_high · monotonic in n · modulation bounded clamp(-0.3,+0.3) |
| F-Fuzz | 0 | numeric only; proptest covers input-space |
| F-Integration | 15 | m7→m14 wiring (real WorkflowRunRow stream); m14→m23 evidence-gate handshake; m14→m31 modulation |
| F-Contract | 4 | `Lift` schema parity; `Option::None` semantic preserved across serde roundtrip |
| F-Regression | 3 | reserved for post-deploy bug discoveries |
| F-Mutation | 1 budget (≥**80%** kill rate per G6) | Wilson-CI math correctness — mutations of numerator/denominator/z-constant must die |

### Mutation kill threshold

**80%** (from [[../GENERATIONS/G6-test-discipline.md]] § Per-module mutation table). Rationale: Wilson CI is the F2 mitigation backbone. A surviving mutation of the CI bound computation could silently widen the band and let n<20 proposals slip through. Wave-end `cargo mutants --regex 'm14_lift::wilson::.*'` enforces.

### Boilerplate-lift source

Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part IV Category 05 (Decay/TTL/LTD):
- **m39 fitness tensor** rolling-smoothing pattern (~70% lift): used as a structural template for cumulative confidence aggregation; numerics are NEW (Wilson, not EMA).
- **povm-v2 magnitude-weighted aggregator** (CR-2 reconciliation, commit `e2a8ed3`): pattern for `magnitude-weighted` vs `binary` aggregation transposed to confidence vs lift.

Fresh authorship: ~80 LOC (Wilson CI numerics + Option<Lift> orchestration). Lifted: ~40 LOC scaffolding.

### Structural-gap LOC

Cluster E does NOT own any of the 3 declared structural gaps (Gap 1 = Cluster F m20-m23 KEYSTONE; Gap 2 = Cluster D m11 NEW PRIMITIVE; Gap 3 = Cluster G m30+D m9 EscapeSurfaceProfile). m14 is *adjacent* to Gap 1 — it provides the evidence that Gap 1's KEYSTONE consumes — but no fresh-primitive authorship lives here.

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| F2 (sample-size inflation) | `Option<Lift>::None` at n<20; `ProposalBuilder::build()` in m23 honours the `None` and refuses construction (per AP-WT-F2 in [[../ANTIPATTERNS_REGISTER.md]]) |
| AP-Drift-07 (soak metric over-stated) | Phase 5C Watcher weekly synthesis independently re-computes lift from m7 snapshot; mismatch flags |
| AP-V7-09 (substrate-frame engine confusion) | m14 input is operationalised purely from m7 (no "user intent" surrogate) |

### Atuin trajectory anchor

Per [[../INTEGRATION/atuin-integration.md]] § wt-lift-watch (proposed; T5.2): atuin script `wt-lift-watch <workflow_id>` reads m14 outputs at 30s cadence into `~/.local/share/atuin/history.db` for trajectory grain. Provenance citation discipline: every m14 emission has a corresponding atuin entry with `workflow_id + wilson_low + mean + wilson_high + n`.

### Watcher class pre-position

- **Class C** (confidence-gate refusal) at every `Option::None` emission — refusal is correct behaviour, not failure. Watcher tick log includes `m14::Refused { workflow_id, reason: BelowF2(n=N) }`.
- **Class I** (Hebbian silence) if m14 emits `Some(Lift)` for >5 workflows over 7 days yet substrate `learning_health` doesn't trend up — implies CC-3 firing without CC-5 closure (Cluster H decorative).

---

## m15 — pressure register (L5 · `src/m15_pressure/`)

### Purpose

Detect and **emit** forbidden-verb pressure events and scope-relaxation pressure events as one-event-per-file JSONL drops to `~/projects/shared-context/agent-cross-talk/`. The CC-7 closure entry-point: m15's sideband emit is what makes the engine *aware of its own constraints being pressured*, surfacing those events to Watcher + Zen for spec amendment consideration. Per [[../GENERATIONS/G3-bidi-flow.md]] § m15 + [[../KEYWORDS_20.md]] § Cluster keyword.

### Edge contract

- **Upstream-IN:** forbidden-verb pressure (m32 dispatch attempts of non-allow-listed verbs) + scope-relaxation pressure (m20-m23 internal pressure to weaken invariants e.g., relax n<20)
- **Downstream-OUT:** JSONL one-event-per-file `PHASE-B-RESERVATION-NOTICE-{ts}_{event_id}.jsonl` to `~/projects/shared-context/agent-cross-talk/`
- **Aspect-IN:** m8 build_prereq (compile), m9 namespace_guard (filename + content namespacing)
- **Failure-mode mitigated:** CC-7 closure gap (GAP-Bidi-02) — without m15 the engine has no way to surface its own constraint-pressure; spec drift becomes invisible

### src/ path (planning-spec only)

```
src/m15_pressure/
├── mod.rs                # public: `pub fn register_pressure`, `pub enum PressureKind`
├── kinds.rs              # PressureKind enum: ForbiddenVerb, ScopeRelaxation, SampleSizeRelax, EscapeSurfaceEscalation
├── jsonl_emit.rs         # one-event-per-file emit; filename convention; atomic write (tmp + rename)
├── filter.rs             # de-dup window (60s same kind+context → coalesce to 1 file with count)
└── tests/
```

### LOC budget

~80 LOC (per ULTRAMAP View 2). Smallest module in Cluster E; mostly orchestration around `serde_json::to_writer` + atomic file write.

### Test budget (50 tests)

| Family | Count | Notes |
|---|---:|---|
| F-Unit | 25 | per-PressureKind arm coverage; filename convention; atomic-write semantics |
| F-Property | 5 | invariants: filename uniqueness across 10k concurrent emits; de-dup window monotonic |
| F-Fuzz | 0 | no parser |
| F-Integration | 15 | end-to-end emit → real agent-cross-talk/ directory → Watcher pickup latency <60s |
| F-Contract | 3 | JSONL schema parity with Watcher's expected reader format |
| F-Regression | 2 | reserved |
| F-Mutation | 1 budget (≥**70%** kill rate per G6 floor) | standard threshold |

### Mutation kill threshold

**70%** (G6 standard floor). m15 is orchestration, not algorithm — mutation surface is small and most mutations of atomic-write logic produce obvious test failures.

### Boilerplate-lift source

Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part IV Category 09 (Trap/verify/escape):
- **SKILL-genesis.md** trap-classification pattern (~95% reuse): the PressureKind enum mirrors genesis trap-classes
- **agent-cross-talk file-drop convention** (existing habitat protocol): file naming + atomic write pattern lifted directly from `~/projects/shared-context/agent-cross-talk/` exemplar drops

Fresh authorship: ~30 LOC (PressureKind enum specialised for workflow-trace; de-dup filter window). Lifted: ~50 LOC.

### Structural-gap LOC

None (cluster E owns no structural-gap LOC). m15 IS the GAP-Bidi-02 closure mechanism (CC-7 pressure-driven evolution) — but that is not a *fresh-primitive* gap; it is a *bidi-flow* gap closed by spec discipline.

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| GAP-Bidi-02 (CC-7 closure missing) | m15's sideband emit IS the closure mechanism — file-drop → Watcher/Zen pickup → spec amendment interview |
| AP-Hab-04 (preserve-list discipline) | scope-relaxation pressure events explicitly include "blanket-action attempted" pattern; surfaces S102-class incidents at planning time, not execution |
| AP-V7-08 (handshake dual-silence) | m15 emits a `HandshakeSilence` PressureKind if expected peer ack is missing past timeout — handshake failure visible to Watcher within 60s |

### Atuin trajectory anchor

Per [[../INTEGRATION/atuin-integration.md]] § wt-pressure-tail (proposed): atuin script tails the agent-cross-talk/ directory for new `PHASE-B-RESERVATION-NOTICE-*.jsonl` files and writes a one-line summary into history. Provenance: every m15 emit produces both the JSONL file AND an atuin entry — dual surface for cross-checking.

### Watcher class pre-position

- **Class C** (confidence-gate refusal) — every pressure event represents an invariant the engine refused to relax silently; the emit IS the refusal.
- **Class G** (substrate-frame confusion) at scope-relaxation pressure events that suggest substrate-grain inputs are being treated as anthropocentric (e.g., "user wants to relax n<20" — m15 demands operationalisation of "user wants").
- **Class E** (ancestor-rhyme) escalation if same-kind pressure events accumulate >10 over 14 days without spec amendment landing — indicates planning sprawl.

---

## Cluster-level synergies

### CC-3 — Evidence-Driven Iteration (E → F): cluster's primary contribution

Cluster E's m14 IS the upstream half of CC-3. m20-m22 (PrefixSpan + variant builder + K-means) cannot iterate on a workflow without m14 emitting `Some(Lift)` with valid Wilson bounds. The gate is enforced at *construction* — `ProposalBuilder::build()` in m23 takes `Option<Lift>` and returns `Err(SampleSizeBelowF2)` on `None`. There is no runtime bypass.

This contract makes Cluster E **strictly read-only with respect to iteration** — m14 cannot cause iteration; it can only *permit* it. The same one-way semantic applies to m31 selector modulation: m14's delta is bounded `clamp(-0.3, +0.3)` so no single workflow can dominate selector weight (per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part I § Cluster E).

### CC-7 — Pressure-Driven Evolution (E → spec): cluster's secondary contribution

m15 IS the closure of GAP-Bidi-02 (per [[../GENERATIONS/G3-bidi-flow.md]] § CC-7). The full loop: m15 emits JSONL → agent-cross-talk/ → Watcher tick observes → Zen audit observes → accumulation threshold → spec amendment interview → v1.4 → G7 re-audit → merged → m1.config (cursor, scope filters) updates → loop continues at next session.

Cluster E owns the *first step* of CC-7. All subsequent steps are human-in-loop (Watcher synthesis, Zen audit, Luke decision). The intentional asymmetry: m15 emits cheaply (one JSONL file per event) but the loop closure is slow (days/weeks).

### Intra-cluster (m14 ↔ m15)

m14 and m15 do NOT communicate directly. The decoupling is deliberate: m14 lives on the read-path (m7 → m14 → m23/m31) and m15 lives on the side-band path (m32/m20-m23 → m15 → agent-cross-talk/). Cross-coupling would create circular evidence-pressure feedback (m14 says "n<20, refused" → m15 emits pressure → spec relaxes n<20 → m14 emits with n<20 → bias). Keep separate.

---

## Cluster-level antipatterns (Cluster E specific)

| ID | Antipattern | Mitigation |
|---|---|---|
| AP-WT-F2 | sample-size inflation (m14 emits Some at n<20) | construction-time refusal; `Option::None` never silently `0.0` |
| AP-V7-04 | keyword overgrowth (Cluster E temptation to grow PressureKind variants) | hard cap at 4 PressureKind variants in v1.3; new variants require spec amendment |
| AP-Hab-13 | runbook probe freshness drift (m14 emits stale lift) | m14 includes `computed_at: SystemTime` in every emission; m31 ignores lifts >24h old |
| AP-Test-01 | coverage theatre on Wilson CI (50 tests hit happy path) | mutation kill ≥80% on m14::wilson; differential test vs scipy.stats reference |

---

## Citation discipline

Every claim in this file cites:
- ULTRAMAP View 2 for LOC budget + test count
- G3 § Cluster E for edge contract
- GOD_TIER_CONSOLIDATION Part I § Cluster E for formula + thresholds
- G6 § Per-module mutation table for kill targets
- ANTIPATTERNS_REGISTER for failure-mode mapping

No uncited claims.

---

## Sign-off

Cluster E plan authored 2026-05-17 by Command (parallel author for V7 optimisation). Planning-only per HOLD-v2 + AP24. Bidi-anchored to ULTRAMAP + G3 + sibling cluster plans. ~110 tests across 2 modules; mutation kill targets 80%/70%; CC-3 + CC-7 closures owned. m14's `Option::None` semantic is load-bearing; m15's sideband emit is GAP-Bidi-02 closure mechanism. Read this with [[cluster-F.md]] (m14 downstream consumer) + [[cluster-G.md]] (m31 modulation consumer) + [[CROSS_CLUSTER_SYNERGIES.md]] (CC-3 + CC-7 deep contracts).

*Luke @ node 0.A | Command @ Orchestrator | Watcher ☤ @ observing | Zen @ audit-lane | 2026-05-17 (S1001982)*
