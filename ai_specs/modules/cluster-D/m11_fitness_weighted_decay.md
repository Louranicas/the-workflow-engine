---
title: m11 — fitness_weighted_decay (Gap 2 NEW PRIMITIVE · lifecycle aspect)
module_id: m11
cluster: D — Trust (cross-cutting)
layer: L4
binary: wf-crystallise
feature_gate: [none]
verb_class: record
ship_first: true
gap_owner: [Gap 2]
status: SPEC · planning-only · HOLD-v2 · NO CODE · NO CARGO
loc_budget: 250
test_budget: 70
mutation_kill: 70
boilerplate_lift: 40
date: 2026-05-17 (S1001982)
authority: Luke @ node 0.A — AP24 gate (G9 "start coding workflow-trace")
binding_spec: Genesis Prompt v1.3 § 1, § 3 (verb-relaxation table — m11 retains record/measure-only)
primary_contract: CC-2 (Trust Layer Woven — D → all) · CC-5 (Substrate Learning Loop — read-side) · CC-7 (Pressure-Driven Evolution — config target)
structural_gap: Gap 2 (frequency × fitness × recency compound decay; ~200-300 LOC NEW PRIMITIVE — no boilerplate ancestor composes all three signals)
decisions_applied: [D-D]
---

# m11 — `fitness_weighted_decay`

> **Back to:** [`cluster-D/INDEX`](./) · [`ai_specs/INDEX`](../../INDEX.md) · [`MODULE_MATRIX`](../../MODULE_MATRIX.md) · vault [[cluster-D-trust-cross-cutting]] · [cluster-D plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md) · [phase-1](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md) · [Genesis v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md)
>
> **Sister modules (Cluster D):** [m8](m8_povm_build_prereq.md) · [m9](m9_watcher_namespace_guard.md) · [m10](m10_ember_ci_gate.md) · [m11](m11_fitness_weighted_decay.md)

---

## 1. Purpose

m11 implements the engine-wide sunset law as a **fitness-weighted decay loop**: every `AcceptedWorkflow` in the m30 curated bank carries a `sunset_at` timestamp; after that timestamp, the workflow is ineligible for m31 selection and m32 dispatch. m11 drives the lifecycle that reaches that sunset — not by unilateral deletion, but by **monotonically decreasing the workflow's selection weight** until it falls below the prune threshold or hits the explicit hard-sunset boundary, whichever comes first.

The decay is fitness-weighted via the **NEW PRIMITIVE compound formula**:

```text
decay_factor = base_rate + (1.0 - base_rate) × clamp(frequency × fitness × recency, 0.0, 1.0)
```

This is **Gap 2** per [CLAUDE.md § structural-gap authorship](../../../CLAUDE.md): the formula composes three independently-tracked signals (frequency from m14, fitness from stcortex pathway weight via m42, recency from m7 last-run timestamp). No existing boilerplate ancestor composes all three. [`povm-v2_lifecycle.rs`](../../../the-workflow-engine-vault/boilerplate%20modules/05-decay-ttl-ltd/) provides the decay primitive; [`m39_fitness_tensor.rs`](../../../the-workflow-engine-vault/boilerplate%20modules/05-decay-ttl-ltd/) provides the fitness signal infrastructure; the recency exponential is standard applied math. **The composition is m11's intellectual contribution to the engine and the largest non-KEYSTONE-cluster module by LOC budget (~250)**.

m11 sits at the **lifecycle checkpoint** of the CC-2 trust regime. m8 catches POVM misreading at compile time; m9 catches namespace drift at write time; m10 catches mis-calibrated output at CI gate time; m11 catches workflow ossification at lifecycle time. Together they form the four-checkpoint aspect-weave: compile / write / output / lifecycle. m11's verb_class is **`record`** (not `select` — selection is m31's job; not `dispatch` — dispatch is m32's job; m11 records decayed weights to inform downstream selection). This preserves the Phase-A passive-verb invariant even under the single-phase override.

---

## 2. Contracts (CC-2 primary; CC-5 read-side; CC-7 config target)

| Surface | Direction | Detail |
|---|---|---|
| **CC-2 Trust Layer Woven** (PRIMARY) | OUT → m30, m31 | m11 produces `DecayFactor` values modulating m30 sunset trigger + m31 selector composite-score. |
| **CC-5 Substrate Learning Loop** (read-side) | IN ← stcortex pathway.weight (via m42) | m11's `fitness` signal reads from stcortex `pathway.weight` for each workflow's registered pathway ID; this is the substrate-grain edge that closes CC-5 via m42 → stcortex → m11 → m31 → m32 → m42. |
| **CC-7 Pressure-Driven Evolution** (config target) | IN ← m15 reservation events | m11's threshold constants (`base_rate`, `recency_half_life_days`, `sunset_threshold`, `prune_threshold`) are the prime targets of m15 reservation events when Watcher/Zen observe systematic mis-decay. |
| m7 `last_run_at` per workflow | IN | Recency signal source. |
| m14 `frequency` per workflow | IN | `evidence_aggregator` `run_count` over observation window, normalised to `[0, 1]`. |
| stcortex `pathway.weight` (gated by m8 `cfg(povm_calibrated)` for substrate-LTP-density display only — pathway weight itself is m42-stcortex, not POVM) | IN | Fitness signal source; per-workflow pathway weight in `[0, 1]`. |
| `m11.config.base` (default 0.10 per Hebbian v3 reconciliation) | IN | Decay floor — the fastest decay the law allows when all signals zero. |
| `m11.config.recency_half_life_days` (default 30; aligned with Phase 6 D120 sunset) | IN | Exponential decay envelope parameter. |
| `m11.config.sunset_threshold` (default 0.05) | IN | Below this `DecayFactor`, workflow enters `SunsetExpired`. |
| `m11.config.prune_threshold` (default 0.01, lifted from `povm-v2_lifecycle.rs`) | IN | Below this weight, workflow is pruned from bank entirely. |
| `DecayFactor(f64)` newtype | OUT | Multiplicative modulation on m31's selector composite α·fitness + β·recency + γ·frequency + δ·diversity (modulates β and γ; α drawn directly from pathway weight). |
| `SunsetStats` per cycle | OUT | Telemetry for [`/sweep`](../../../CLAUDE.md) + Watcher Class-I monitor. |
| Prune-marker hint to m13 | OUT | m13 emits stcortex delete-markers; m13 ↔ m9 namespace guard fires on the prune emit. |

**Aspect-IN:** m8 (compile-time gate — POVM-reading paths inside m11 for display-only `substrate_LTP_density`); m9 (write-time gate — m11's delete-marker emissions via m13 pass through namespace validator).

---

## 3. Public surface

```rust
// src/m11_fitness_weighted_decay/mod.rs
pub mod formula;     // The Gap 2 compound decay — canonical NEW PRIMITIVE
pub mod inputs;      // recency_factor, frequency_factor, fitness_factor normalisations
pub mod sunset;      // SunsetPhase state machine + transitions
pub mod consolidation; // 4-step cycle: decay → reinforce-read → prune → auto-sunset
pub mod error;

pub use formula::{compute_decay_factor, DecayFactor};
pub use inputs::{recency_factor, frequency_factor, fitness_factor};
pub use sunset::{SunsetPhase, SunsetStats, AcceptedWorkflowDecay};
pub use consolidation::{SunsetLifecycle, run_consolidation_cycle};
pub use error::DecayError;
```

Newtype discipline:

```rust
/// Compound decay factor in [base_rate, 1.0]. Multiplicative modulation on m31 selector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecayFactor(f64);

impl DecayFactor {
    pub fn as_f64(self) -> f64 { self.0 }
    pub fn new(value: f64) -> Result<Self, DecayError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(DecayError::OutOfRange { value });
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SunsetPhase {
    Active,          // weight ≥ soft threshold (0.1); dispatch-eligible
    PrunePending,    // weight < soft threshold but > prune threshold; de-ranked
    SunsetExpired,   // sunset_at < now OR weight < prune threshold; excluded
}

#[derive(Debug, Clone)]
pub struct SunsetStats {
    pub cycles_run: u64,
    pub workflows_decayed: usize,
    pub workflows_pruned: usize,
    pub workflows_auto_sunset: usize,
    pub mean_decay_factor: f64,
    pub min_decay_factor: f64,
    pub max_decay_factor: f64,
}
```

---

## 4. Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum DecayError {
    #[error("DecayFactor out of range: {value} (expected finite in [0.0, 1.0])")]
    OutOfRange { value: f64 },

    #[error("clock returned None (system pre-epoch or fault); skipping decay cycle to avoid silent zero-timestamp poisoning (F-POVM-07 pattern)")]
    ClockUnavailable,

    #[error("m7 read failed for workflow {workflow_id}: {source}")]
    M7ReadFailed { workflow_id: String, #[source] source: workflow_core::errors::WorkflowError },

    #[error("stcortex pathway read failed for workflow {workflow_id}, pathway {pathway_id}: {source}")]
    StcortexReadFailed { workflow_id: String, pathway_id: String, #[source] source: workflow_core::errors::WorkflowError },

    #[error("m14 frequency read failed for workflow {workflow_id}: {source}")]
    M14ReadFailed { workflow_id: String, #[source] source: workflow_core::errors::WorkflowError },

    #[error("consolidation cycle aborted: {reason}")]
    CycleAborted { reason: String },
}
```

Error-band assignment per [`ERROR_TAXONOMY.md` § E5xxx — Lifecycle errors](../../INDEX.md): `OutOfRange = E5001`, `ClockUnavailable = E5002`, `M7ReadFailed = E5003`, `StcortexReadFailed = E5004`, `M14ReadFailed = E5005`, `CycleAborted = E5099`.

---

## 5. Implementation sketch — the NEW PRIMITIVE

### 5.1 The compound decay formula (Gap 2)

```rust
/// Compute the fitness-adjusted decay factor for a workflow.
///
/// The factor is in `[base_rate, 1.0]`. Applied as: `weight_n+1 = weight_n * decay_factor`.
/// A factor of 1.0 means no decay this cycle. A factor of `base_rate` is the fastest decay
/// the law allows (the floor).
///
/// # Formula (Gap 2 NEW PRIMITIVE)
///
/// ```text
/// decay_factor = base_rate + (1.0 - base_rate) × clamp(frequency × fitness × recency, 0.0, 1.0)
/// ```
///
/// where:
/// - `base_rate ∈ [0, 1]`: the floor (1.0 - plain_decay_rate). Default 0.98 (plain_decay_rate=0.02).
/// - `frequency ∈ [0, 1]`: m14 dispatch-count normalised to cohort max.
/// - `fitness ∈ [0, 1]`: stcortex pathway.weight (substrate-grain Hebbian signal).
/// - `recency ∈ [0, 1]`: exp(-lambda × days_since_last_run); lambda = ln(2) / half_life_days.
///
/// # Interpretation
///
/// - **All three signals at 1.0** → `decay_factor = 1.0` (no decay; workflow is thriving — used, valued, fresh).
/// - **All three signals at 0.0** → `decay_factor = base_rate = 0.98` (fastest legal decay; workflow is unused, unvalued, stale).
/// - **High frequency, low fitness** (used but not producing good outcomes) → middle of range; decay slows
///   relative to an unused workflow but does NOT stop. This is the spec's "compositional integrity" check:
///   usage alone never grants immortality.
///
/// # Source lineage
///
/// - `base_rate` shape: lifted from `povm-v2_lifecycle.rs::decay_pathways(rate)` — `weight *= (1 - rate)` per cycle.
/// - `frequency` signal: m14 `evidence_aggregator` normalised `run_count`.
/// - `fitness` signal: stcortex `pathway.weight` for workflow's registered pathway (m42-stcortex-only post-2026-05-17 ADR).
/// - `recency` signal: `exp(-lambda × days_since_last_run)` from m7 `last_run_at`.
/// - **Composition (the NEW PRIMITIVE):** this function — no boilerplate ancestor.
///
/// # References
///
/// - [boilerplate `povm-v2_lifecycle.rs`](../../../the-workflow-engine-vault/boilerplate%20modules/05-decay-ttl-ltd/) — decay primitive
/// - [boilerplate `m39_fitness_tensor.rs`](../../../the-workflow-engine-vault/boilerplate%20modules/05-decay-ttl-ltd/) — fitness signal infrastructure
/// - [Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation](../../../the-workflow-engine-vault/) — LTP density metric
/// - [m42 stcortex-only pivot ADR](../../../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md)
pub fn compute_decay_factor(
    frequency: f64,
    fitness: f64,
    recency: f64,
    plain_decay_rate: f64,
) -> Result<DecayFactor, DecayError> {
    debug_assert!((0.0..=1.0).contains(&frequency), "frequency must be in [0,1]");
    debug_assert!((0.0..=1.0).contains(&fitness), "fitness must be in [0,1]");
    debug_assert!((0.0..=1.0).contains(&recency), "recency must be in [0,1]");
    debug_assert!((0.0..=1.0).contains(&plain_decay_rate), "plain_decay_rate must be in [0,1]");

    let base_rate = 1.0 - plain_decay_rate;
    let compound_signal = (frequency * fitness * recency).clamp(0.0, 1.0);
    // FMA for precision: base_rate × (1.0 - compound_signal) + compound_signal × 1.0
    //                  = base_rate + (1 - base_rate) × compound_signal
    let value = base_rate.mul_add(1.0 - compound_signal, compound_signal);
    DecayFactor::new(value)
}
```

The FMA form is per [GOLD_STANDARDS rule for arithmetic precision](../../../GOLD_STANDARDS.md) and matches [`m39_fitness_tensor.rs`](../../../the-workflow-engine-vault/boilerplate%20modules/05-decay-ttl-ltd/) style.

### 5.2 Signal normalisation

```rust
/// Recency factor: exp(-lambda × days_since_last_run).
/// lambda = ln(2) / half_life_days; default half_life = 30d → recency=0.5 at day 30.
pub fn recency_factor(days_since_last_run: f64, half_life_days: f64) -> f64 {
    let lambda = std::f64::consts::LN_2 / half_life_days;
    (-lambda * days_since_last_run).exp().clamp(0.0, 1.0)
}

/// Frequency factor: run_count_in_window / max_run_count_in_cohort.
pub fn frequency_factor(run_count: u64, cohort_max: u64) -> f64 {
    if cohort_max == 0 { return 0.0; }
    (run_count as f64 / cohort_max as f64).clamp(0.0, 1.0)
}

/// Fitness factor: stcortex pathway.weight, already in [0, 1] by stcortex invariant; clamp defensively.
pub fn fitness_factor(pathway_weight: f64) -> f64 {
    pathway_weight.clamp(0.0, 1.0)
}
```

### 5.2b Dual-read soak (DECIDED 2026-05-17, S1002127, Luke directive "as per proposal")

**DECIDED:** For the first **30 days post-G9**, m11 reads the fitness signal from BOTH POVM `learning_health` AND stcortex `pathway.weight` per workflow pathway. Both reads are observed and logged to `data/m11_dual_read_soak.jsonl` (append-only JSONL, one row per consolidation cycle per workflow, fields `{ts_ms, workflow_id, pathway_id, povm_learning_health, stcortex_pathway_weight, delta, decay_factor_used}`). The formula consumes **only one** of the two values at any time — the substrate currently selected as canonical by the m42 ADR (post-2026-05-17: stcortex `pathway.weight`); the second read is observational, **additive, zero behaviour change** pre-commit.

At the 30-day mark, Command commits to a single source based on observed correlation:
- **Target:** Pearson `r > 0.85` between the two streams across all observed workflows.
- **If `r > 0.85`:** confirm stcortex `pathway.weight` as canonical; deprecate POVM dual-read.
- **If `r ≤ 0.85`:** open a Watcher Class-D drift ticket; do not commit; re-evaluate `plain_decay_rate=0.02` against whichever source wins.

Rationale: additive observation, zero behaviour change pre-commit; preserves the m42 ADR substrate-cutover without forcing m11 to choose blind. plain_decay_rate=0.02 calibration to be re-verified against whichever source wins.

### 5.3 Calibration against 120-day sunset law

With `plain_decay_rate = 0.02` and all signals at zero (worst case → `decay_factor = base_rate = 0.98`), starting from weight 1.0 and pruning at weight 0.01 (`povm-v2_lifecycle.rs::PRUNE_THRESHOLD`):

```text
0.98^n < 0.01  →  n > log(0.01) / log(0.98) ≈ 228 cycles
```

At daily cadence: ~228 days, which is roughly 2× the 120-day default `sunset_at` hard boundary. **This is the deliberate design**: the sunset law is the **hard boundary**; the fitness-weighted decay is the **gradient**. A workflow respects the explicit `sunset_at` timestamp; the decay modulates how its selection weight falls toward the prune threshold during the eligibility window. A workflow with high fitness/frequency/recency will sustain near full weight up to the sunset boundary; a workflow with zero signals reaches the prune threshold via decay long before the hard sunset (or rather, hits 0.01 weight at ~228 days, but the 120d sunset fires first by construction).

Luke can override the 120-day default at bank-insertion time by setting `sunset_at` explicitly. m11 respects the explicit timestamp; it never decays past it.

### 5.4 Consolidation cycle (4 steps; mirrors `m16_hebbian_engine.rs`)

```rust
pub fn run_consolidation_cycle(
    lifecycle: &SunsetLifecycle,
    config: &DecayConfig,
) -> Result<SunsetStats, DecayError> {
    let now_ms = chrono_now_ms().ok_or(DecayError::ClockUnavailable)?;
    let mut stats = SunsetStats::default();

    // Step 1 — Decay: apply compute_decay_factor to all AcceptedWorkflow rows in m30 bank
    for workflow in lifecycle.bank.iter_active() {
        let f = frequency_factor(workflow.run_count_window, lifecycle.cohort_max_run_count());
        let fit = fitness_factor(lifecycle.read_pathway_weight(&workflow.pathway_id)?);
        let r = recency_factor(workflow.days_since_last_run(now_ms), config.recency_half_life_days);
        let factor = compute_decay_factor(f, fit, r, config.plain_decay_rate)?;
        workflow.apply_decay(factor);
        stats.workflows_decayed += 1;
    }

    // Step 2 — Reinforce-read: (external — m42 Hebbian feedback updates pathway weights;
    //          m11 reads them on next cycle). m11 does not write Hebbian updates.

    // Step 3 — Prune: DELETE from bank where weight < prune_threshold AND sunset_at IS NOT NULL
    for workflow in lifecycle.bank.iter_decayed() {
        if workflow.weight() < config.prune_threshold && workflow.sunset_at.is_some() {
            lifecycle.bank.mark_for_prune(&workflow.id);
            stats.workflows_pruned += 1;
        }
    }

    // Step 4 — Auto-sunset: mark expired (sunset_at < now) workflows as SunsetExpired
    for workflow in lifecycle.bank.iter_all() {
        if workflow.sunset_at.is_some_and(|s| s < now_ms) {
            workflow.transition(SunsetPhase::SunsetExpired);
            stats.workflows_auto_sunset += 1;
        }
    }

    stats.cycles_run += 1;
    Ok(stats)
}
```

### 5.5 State machine

```text
AcceptedWorkflow lifecycle:

                                  fitness-weighted decay each cycle
    Active ───────────────────────────────────────────────────────► PrunePending
      │                                                                  │
      │ sunset_at reached (explicit OR weight < prune_threshold)         │
      ▼                                                                  │
  SunsetExpired ────────────────────────────────────────────────────────┘
      │
      │ (excluded from dispatch; Luke may extend via explicit sunset_at override)
      │
   [archived or deleted at next consolidation sweep]
```

The state machine is exhaustive: `Active` → `PrunePending` on weight < soft threshold (0.1); `PrunePending` → `Active` on fitness recovery (pathway weight rises via m42 Hebbian reinforce, m11 reads on next cycle, decay factor improves, weight monotonically recovers); either → `SunsetExpired` on hard boundary; never returns from `SunsetExpired` without explicit Luke override.

### 5.6 Daemon task registration

Per [`ws_inbound_writer.rs` TTL sweep pattern](../../../the-workflow-engine-vault/boilerplate%20modules/05-decay-ttl-ltd/), the consolidation loop runs as a scheduled `tokio::spawn` task with `tokio::select!` shutdown drain. Default cadence: nightly (daily decay aligns with the 228-day calibration); configurable via `WF_DECAY_CYCLE_HOURS`. The task is registered in the daemon's task table at startup; if it panics, the supervisor restart pattern (per [LCM `lcm-supervisor`](../../../loop-engine-v2/SESSION_RESUME_2026-05-15.md)) re-spawns, and m31's selector applies a defense-in-depth query-time filter excluding `SunsetExpired` workflows even if m11's task is dead.

---

## 6. Test plan (70 tests, mutation ≥70%)

Per [TEST_DISCIPLINE matrix row m11](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) and [cluster-D plan § m11 test-pattern allocation](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md). m11 has the highest test density in Cluster D because Gap 2 requires property-test invariants over a 4-dimensional input space (`base`, `frequency`, `fitness`, `recency`).

| Pattern | Count | Examples |
|---|---:|---|
| **F-Unit (`compute_decay_factor`)** | 35 | zero signals → `base_rate`; all-ones → `1.0` (within f64 epsilon); single-axis variation (vary frequency 0..1 with fitness=recency=1.0 → linear interpolation from `base_rate` to 1.0); permutation of (1.0, 0.5, 0.0) across axes; `compound_signal` clamp at 1.0 (synthetic inputs > 1.0 via debug_assert relaxed); `compound_signal` clamp at 0.0; `plain_decay_rate=0.0` → `base_rate=1.0` → `decay_factor=1.0` always; `plain_decay_rate=1.0` → `base_rate=0.0` → `decay_factor=compound_signal`. Normalisation tests: `recency_factor(0, 30) = 1.0`; `recency_factor(30, 30) = 0.5`; `recency_factor(180, 30) ≈ 0.0156`; `frequency_factor(0, 100) = 0.0`; `frequency_factor(100, 100) = 1.0`; `frequency_factor(50, 0) = 0.0` (cohort empty); `fitness_factor` clamp on out-of-range pathway weights. Consolidation cycle: decay → prune → auto-sunset ordering; stats accumulation; clock-skip safety. |
| **F-Property (Gap 2 invariants)** | 10 | `decay ∈ [base_rate, 1.0]` for all finite inputs (10k iters via `proptest`); **monotonic non-decreasing in `frequency`** (hold fitness, recency, base fixed; vary frequency); **monotonic non-decreasing in `fitness`** (symmetric); **monotonic non-decreasing in `recency`** (symmetric); idempotent under `recency_half_life_days = +inf` (degenerate case → recency=1.0 always); `base_rate = 1.0` → `decay_factor = 1.0` always (sanity); `base_rate = 0.0` AND any-axis-zero → `decay_factor ∈ [0, 1]` (no negative; clamp guarantee); `recency` symmetric under timestamp-shift on stationary inputs (decay factor depends only on `Δt`, not absolute timestamps); FMA precision: `compute_decay_factor(f, g, r, p)` equals the naive `base + (1-base)*clamp(...)` to within 1 ulp for 10k random inputs; **base-floor invariant: decay_factor ≥ base_rate** for any finite signals. |
| **F-Integration** | 15 | m11 ↔ m7 read (`last_run_at` via mock m7); m11 ↔ m14 read (`frequency` via mock m14); m11 ↔ stcortex read (`pathway.weight` via mock stcortex client); m11 → m31 selector composite (verify β·recency + γ·frequency modulation lands); m11 → m30 sunset trigger (workflow in `SunsetExpired` excluded from `bank.iter_active()`); m11 → m13 prune-marker emit (verify m9 namespace guard fires on emit); multi-workflow concurrent decay (10 workflows in a single cycle); cycle re-entrancy (consecutive cycles produce monotonically-decreasing weights for unchanged signals); supervisor restart preserves state (cycle state in m30; m11 task itself is stateless); cohort_max_run_count edge cases (cohort empty → all frequency_factor = 0.0; cohort of one → that workflow gets frequency_factor = 1.0). |
| **F-Contract** | 5 | `DecayFactor` newtype Display + Debug stability; `SunsetPhase` ordinal stability; sunset-threshold constant matches Phase 6 D120 spec; formula coefficient stability across spec versions (snapshot of `base_rate` default, `plain_decay_rate` default, `prune_threshold`, `sunset_threshold`, `recency_half_life_days`); `compute_decay_factor` signature stability. |
| **F-Regression** | 4 | Gap 2 formula regression slot (output unchanged for known input vectors); sunset threshold regression; recency half-life regression; F1 (bank/name ossification) regression — verify that a workflow with all signals at 1.0 for 1000 cycles still respects an explicit `sunset_at` boundary (decay never approaches "immortality" — sunset law overrides). |
| **F-Mutation** | budget | ≥70% kill rate concentrated on `formula.rs::compute_decay_factor` (the Gap 2 core). |

The 70-test count matches the [TEST_DISCIPLINE matrix row m11](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) target and reflects Gap 2's structural weight. Property-test budget (10 invariants × 10k iters = 100k assertions) is the highest in the engine outside KEYSTONE m20.

---

## 7. Boilerplate lift map

| Source | Lift % | Use |
|---|---:|---|
| `boilerplate modules/05-decay-ttl-ltd/povm-v2_lifecycle.rs` | 40% (mechanism) | `ConsolidationStats` struct shape → `SunsetStats`; `decay_pathways(rate)` → `decay_bank_workflows(rate)` with the compound formula replacing scalar multiply; `decay_survived` counter pattern → `decay_cycles` in `AcceptedWorkflowDecay`; `chrono_now()` returning `Option<String>` on pre-epoch clock (F-POVM-07 fix pattern) reproduced as `Option<i64>` — skip cycle rather than silent zero-timestamp |
| `boilerplate modules/05-decay-ttl-ltd/m39_fitness_tensor.rs` | 30% (infrastructure) | Rolling-mean smoothing for volatile fitness signals (`SMOOTHING_WINDOW = 6` samples); volatile-dimension mask pattern (D6 `hebbian_health` is volatile; smoothed before entering decay formula); `FitnessReport.overall_score` as abstraction boundary — m11 reads the scalar, not the 12D tensor directly |
| `boilerplate modules/05-decay-ttl-ltd/m16_hebbian_engine.rs` | 80% (cycle shape) | 4-step consolidation cycle (decay → reinforce-read → prune → auto-sunset); mechanism identical, the 20% delta is step 1 (compound formula) + step 3 (prune threshold ∩ sunset law, not threshold alone) |
| `synthex-v2/src/daemon/tasks/ws_inbound_writer.rs` | 60% (task shape) | TTL sweep loop: `DELETE WHERE condition AND inserted_at < cutoff` → m11's prune step; `tokio::select!` shutdown drain for graceful cancellation; `parking_lot::Mutex<Connection>` for Send+Sync SQLite access |
| ME v2 telemetry decay | 50% (recency half-life) | `exp(-lambda × Δt)` recency primitive; generalised to recency axis |
| POVM v2 `learning_health` magnitude-weighted formula | 30% (cohort denominator) | EMA-like cohort denominator for frequency normalisation |
| RALPH ORAC fitness 12D tensor clamp(0,1) idiom | 20% | Fitness normalisation defensive clamp |
| **Gap 2 compound formula** | 0% (FRESH) | ~120 LOC core: `base + (1 - base) × clamp(f × g × r, 0, 1)`. **No upstream equivalent.** The engine's structural primitive #2 per [CLAUDE.md § structural-gap authorship](../../../CLAUDE.md). |

**Structural-gap LOC:** **Gap 2 ~200-300 LOC NEW PRIMITIVE.** m11 is the exclusive owner. Of m11's ~250 LOC budget, ~120 LOC is the Gap 2 core (formula + normalisation + property-test scaffolding); the remaining ~130 LOC is the consolidation cycle, state machine, daemon task wiring, and error plumbing (which lifts heavily from boilerplate per the table above).

---

## 8. Failure modes addressed

| ID | Mode | How m11 addresses |
|---|---|---|
| **R5** | RALPH fitness-weighted decay not firing; workflows immortal in bank | Compound formula structurally requires fitness signal to participate; absence of fitness signal (pathway weight 0) caps `compound_signal` at 0 → `decay_factor = base_rate` → decay proceeds at floor rate; never immortal. |
| **F1** | Sunset law bypassed by missing lifecycle loop | Consolidation task registered in daemon at startup (lifted from `synthex-v2_daemon_runtime.rs` task-spawn pattern); if task panics or is unscheduled, m31's selector applies query-time filter excluding `SunsetExpired` — defense-in-depth. Pairs with m30 sunset trigger. |
| **F-POVM-07** | Silent zero-timestamp poisoning on clock fault | `chrono_now_ms()` returns `Option<i64>`; m11 returns `DecayError::ClockUnavailable` and skips the cycle rather than treating timestamps as 0. |
| **AP-V7-09** | Substrate-frame engine confusion | The compound formula IS the canonical substrate-frame measure — `frequency × fitness × recency` operationalises "did the substrate weight rise", not "did the user like this workflow". Per [Watcher class-G pre-position](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md): if `recency` is back-decoded as user-attention rather than substrate-weight time-since-update, Class-G fires. |
| **AP-Test-02** | Property-test stub | m11's 10 F-Property tests run ≥10k iterations per invariant; highest property-test count in the engine outside KEYSTONE m20. |
| **AP-Drift-11** | Supervisor stub mistaken for live | m11's consolidation task is verified live by `SunsetStats.cycles_run` monotonically increasing in metrics; never inferred from process-list. |
| Gap-2-specific | Decay returns negative / exceeds 1.0 / silently saturates | Clamp at compound_signal computation; `DecayFactor::new` validates range; FMA precision keeps values in `[base_rate, 1.0]` to within 1 ulp for all 10k tested inputs. |

---

## 9. Observability

```rust
tracing::info!(
    target = "m11.consolidation.cycle",
    cycles_run = stats.cycles_run,
    workflows_decayed = stats.workflows_decayed,
    workflows_pruned = stats.workflows_pruned,
    workflows_auto_sunset = stats.workflows_auto_sunset,
    mean_decay_factor = stats.mean_decay_factor,
    "m11 consolidation cycle complete"
);

tracing::warn!(
    target = "m11.consolidation.clock_skip",
    "clock unavailable; skipping consolidation cycle to avoid F-POVM-07 zero-timestamp pattern"
);
```

Metrics exposed via [m05_metrics_collector](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md):
- `m11_decay_factor_histogram` — distribution of `DecayFactor` per cycle
- `m11_workflows_in_phase{phase}` gauge — counts in Active / PrunePending / SunsetExpired
- `m11_consolidation_cycle_duration_seconds` histogram
- `m11_substrate_ltp_density_observed` gauge (gated by `cfg(povm_calibrated)` — display-only, NOT consumed by formula)

Read by [`/sweep`](../../../CLAUDE.md), [`/ralph-monitor`](../../../CLAUDE.md), and the Watcher's Class-I (Hebbian silence) monitor.

---

## 10. Pre-conditions / post-conditions

**Pre:** m7 reachable (last_run_at queries); m14 reachable (frequency queries); stcortex reachable (pathway weight reads via m42-stcortex-only route); m30 bank populated (at least one `AcceptedWorkflow`); `chrono_now_ms()` returns `Some(_)`; `DecayConfig` loaded (defaults documented above).

**Post:** Either (a) `SunsetStats` returned with monotonically-non-decreasing `cycles_run`, decay applied to every active workflow, prune marks emitted via m13 for workflows below threshold, `SunsetExpired` transitions logged, or (b) `DecayError` returned with reason; cycle skipped; stats unchanged; next cycle re-attempts.

---

## 11. Watcher class pre-positions

| Class | Triggers when |
|---|---|
| **Class A** (activation) | First `compute_decay_factor` invocation post-Genesis; first `run_consolidation_cycle` completing without error. |
| **Class D** (four-surface drift) | Per-workflow `DecayFactor` diverges across stcortex / m7 / CLAUDE.local.md weekly synthesis — one source says workflow is decayed, another says active. |
| **Class G** (substrate-frame confusion) | `recency` is back-decoded as user-attention rather than substrate-weight time-since-update — semantic drift in downstream consumer (m12 report rendering, Watcher synthesis). |
| **Class I** (Hebbian silence) | m11's `fitness` input reads stcortex `pathway.weight`; if pathway weight never moves (m42 Hebbian reinforce broken or substrate stuck), m11's decay collapses to `base × clamp(freq × 0 × rec, 0, 1) = base_rate` for all workflows; **Class-I is directly observable here as flat `mean_decay_factor ≈ base_rate` across many cycles**. This is the primary Watcher pre-position for m11 and the canonical Class-I detection site for the engine. |

WCP notify on Class I sustained > 5 cycles → file drop to `~/projects/shared-context/watcher-notices/` with `mean_decay_factor` histogram. Class-I is the most-monitored failure mode in the engine because it indicates substrate-grain learning has stalled.

---

## 12. Atuin trajectory anchor

Proposed atuin scripts (post-G9):
- `wt-decay-pulse` — samples m11's per-workflow `DecayFactor` for top-K least-decayed + bottom-K most-decayed (Phase 5C weekly synthesis input per [phase-5 framework](../../../the-workflow-engine-vault/deployment%20framework/phase-5-deploy-and-soak.md)).
- `wt-sunset-watch` — lists workflows approaching sunset threshold (within 7 days of `sunset_at` OR weight within 0.05 of `prune_threshold`).
- `wt-decay-flat-check` — Watcher Class-I early-warning: alerts if `mean_decay_factor` stays within ±0.02 of `base_rate` for 5+ cycles.

History rows during normal authoring: `cargo test --release -- m11::formula`, `cargo test --release -- m11::property`, `~/.local/bin/stcortex sql "SELECT id, weight FROM pathway WHERE namespace LIKE 'workflow_trace_%'"`. Queryable via `atuin search --workspace workflow-trace 'm11'`.

---

## 13. Open questions

1. **`plain_decay_rate` default (0.02) yields ~228-cycle floor.** With daily cadence this is ~228 days, ~2× the 120-day default sunset. Is the 2× ratio the right safety margin? Should the spec target ~1× (decay floor aligns with sunset boundary) or ~3× (decay is always a softer signal than sunset)? Question for G7 + Watcher.
2. **Fitness signal post-2026-05-17 m42 ADR — DECIDED.** **DECIDED 2026-05-17 (S1002127, Luke directive "as per proposal"):** Dual-read soak adopted (30 days from G9-fire; POVM `learning_health` + stcortex `pathway.weight` parallel reads; correlation-based commit at day-30 with target Pearson `r > 0.85`; logged to `data/m11_dual_read_soak.jsonl`). Rationale: additive observation, zero behaviour change pre-commit; preserves the m42 ADR substrate-cutover without forcing m11 to choose blind. plain_decay_rate=0.02 calibration to be re-verified against whichever source wins. See § 5.2b for full protocol.
3. **Cohort denominator for `frequency_factor`.** Current spec uses `max_run_count_in_window` across the entire bank. This is sensitive to outliers (one heavily-run workflow drives everyone else's normalised frequency toward 0). Alternative: 95th-percentile cohort denominator. Question for G7.
4. **Recency half-life (default 30 days).** Aligned with Phase 6 D120 sunset evaluation. Alternative: tie half-life to the explicit `sunset_at` of each workflow (per-workflow half-life). Adds complexity but eliminates one global constant. Question for G7.
5. **Consolidation cadence (default nightly).** Daily aligns with the 228-cycle calibration. Faster cadence (hourly, 6-hourly) compresses the decay timeline; would require re-calibrating `plain_decay_rate`. Question for G7 + Phase 5 soak observations.
6. **State machine reversibility.** Currently `PrunePending → Active` is allowed on fitness recovery; `SunsetExpired → Active` is NOT (requires explicit Luke override). Is the asymmetry correct? Workflows that hit hard sunset by reaching `prune_threshold` (not explicit timestamp) might deserve a reactivation pathway. Question for G7.
7. **CC-5 closure verification.** m11 reads stcortex pathway weight (written by m42 via Hebbian feedback from m32 dispatch outcomes). The loop closes through F→G→H→stcortex→m11→m31→m32 (next dispatch cycle). End-to-end verification requires integration test spanning all clusters. m11 contributes a single CC-5 read-side assertion; full closure verified at Cluster H spec or in dedicated `tests/synergies/cc-5-substrate-learning-loop.rs`.

---

> **Back to:** [`cluster-D/INDEX`](./) · [`ai_specs/INDEX`](../../INDEX.md) · [`MODULE_MATRIX`](../../MODULE_MATRIX.md) · vault [[cluster-D-trust-cross-cutting]] · [cluster-D plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md) · [phase-1](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md)
>
> **Sister modules (Cluster D):** [m8](m8_povm_build_prereq.md) · [m9](m9_watcher_namespace_guard.md) · [m10](m10_ember_ci_gate.md) · [m11](m11_fitness_weighted_decay.md)

*Spec authored 2026-05-17 (S1001982). HOLD-v2 active. No code, no Cargo, no scaffold until G1-G9 clear and Luke emits explicit start-coding signal.*
