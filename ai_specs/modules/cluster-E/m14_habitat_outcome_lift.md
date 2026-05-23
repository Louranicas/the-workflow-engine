---
title: m14 — habitat_outcome_lift (per-module spec)
date: 2026-05-17 (S1001982)
kind: per-module-spec · planning-only · HOLD-v2 active
status: SPEC (no .rs, no Cargo.toml, no cargo)
cluster: E — Evidence + Pressure
layer: L5
binary: wf-crystallise
feature_gate: [intelligence]
verb_class: record
loc_est: 110
test_budget: 70 (G6 mutation kill ≥ 80% on wilson math)
boilerplate_lift: ~25% (Cat 05 m39 fitness-tensor smoothing + Cat 06 nerve-center aggregator)
authority: Command · workflow-trace V7 optimisation · v1.3 binding
---

> **🏷 v0.1.0 — SD A/B reconciliation (S1004115 Phase 9 / § 15 D27):**
> The shipped m14 implementation is canonical. **SD2** (LiftError
> taxonomy rename + variant divergence), **SD3** (`cost_lift` returns
> `Result`), **SD4** (window-eviction direction) were code-ahead-of-spec
> drifts — see `src/m14_lift/` for the authoritative behaviour. Spec
> amendments mirror the shipped surface; no behavioural divergence
> remains. Full disposition: [`PHASE9_SD_RECONCILIATION_S1004115.md`](../../../ai_docs/PHASE9_SD_RECONCILIATION_S1004115.md).

# m14 — `habitat_outcome_lift` · evidence aggregator

> **Back to:** [`../../INDEX.md`](../../INDEX.md) · [`../../MODULE_MATRIX.md`](../../MODULE_MATRIX.md) · [`../../../CLAUDE.md`](../../../CLAUDE.md) · [`../../../CLAUDE.local.md`](../../../CLAUDE.local.md)
>
> **Sister modules:** [m14](m14_habitat_outcome_lift.md) · [m15](m15_pressure_register.md)
>
> **Cluster spec (vault):** [[cluster-E-evidence-pressure]] (canonical at `the-workflow-engine-vault/module specs/cluster-E-evidence-pressure.md`)
>
> **V7 plan:** [cluster-E plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-E.md) · [ULTRAMAP View 2 m14 row](../../../ai_docs/optimisation-v7/ULTRAMAP.md) · [G3 bidi-flow § Cluster E](../../../ai_docs/optimisation-v7/GENERATIONS/G3-bidi-flow.md)
>
> **Genesis spec:** [`ai_docs/GENESIS_PROMPT_V1_3.md`](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 1 (Cluster E row), § 5 (F2 per-report-type hard gate)
>
> **Framework references:** [`PATTERNS.md`](../../../PATTERNS.md) · [`GOLD_STANDARDS.md`](../../../GOLD_STANDARDS.md) · [`ANTIPATTERNS.md`](../../../ANTIPATTERNS.md) · runbook [`runbook-00-pre-genesis-gates.md`](../../../ai_docs/optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md)

---

## §1 Purpose

m14 is the **empirical spine** of the engine. It aggregates `WorkflowRunRow` records from m7's central correlation table, computes a single dimensionless `habitat_outcome_lift` value with explicit Wilson 95% CI bounds over a rolling window, and emits both an aggregate snapshot and per-workflow lift-contribution deltas to its downstream consumers (m11 sunset, m31 selector, m42 stcortex-emit). m14 does not select, dispatch, propose, or recommend; it **measures and reports**. Verb class: `record`.

Without m14 the m11 sunset law has only time as input (D120 hard default); with m14 the sunset clock is gated by observed value. Without m14 the m31 selector has only RALPH fitness + diversity-distance; with m14 selector weights acquire an empirical lift-contribution gradient. m14 is the **CC-3 upstream half** — m20-m23 iteration is forbidden against any domain where m14 has not emitted `Some(Lift)` with valid bounds.

## §2 Responsibility & verb-class boundary

| In-scope (record verb) | Out-of-scope (refused) |
|---|---|
| Read `WorkflowRunRow` from m7's SQLite hub since `latest_ts_ms` | Write to m7 |
| Compute Wilson 95% CI over rolling window | Recommend a workflow to a human (`recommend_*` — m15-trigger) |
| Emit `Option<LiftSnapshot>` on a timer (default 5 min) | Auto-promote anything (`auto_*` — m15-trigger) |
| Emit per-workflow `WorkflowLiftContribution` deltas | Mutate selector weights directly (m31 owns its write-path) |
| Refuse to emit when `n < MIN_SAMPLE_SIZE` (return `None`) | Estimate / extrapolate / smooth-over missing samples |
| Carry CI half-width alongside every `Some(lift)` | Strip CI bars before downstream emit |

The refusal-to-emit-below-threshold is **load-bearing semantic** (per cluster plan §m14, G6 substrate-frame pass). `None` is not a degenerate value — it is the engineered behaviour and is the F2 mitigation backbone. Downstream consumers MUST treat `None` as "insufficient evidence; hold current state" and never as "zero lift".

## §3 Edge contract (per G3 bidi-flow § Cluster E)

**Upstream-IN**
- `m7.WorkflowRunRow` stream — `(workflow_id, session_id, outcome, cost_tokens, decision_cost_baseline, cascade_success, decisions_made, ts_ms)` (paginated read via `sqlx::query_as!` since last `latest_ts_ms` cursor).

**Downstream-OUT**
- `Option<LiftSnapshot>` → m11 (sunset gate) via in-process `tokio::sync::watch` channel.
- `Vec<WorkflowLiftContribution>` → m31 (selection weight perturbation, clamped `(-0.3, +0.3)`) via the same watch primitive (separate channel).
- `(ltp_workflows: Vec<WorkflowId>, ltd_workflows: Vec<WorkflowId>)` → m42 stcortex-emit (Hebbian reinforce / depress; workflows not individually significant are omitted, never written-as-zero).

**Aspect-IN**
- m8 `cargo:rustc-cfg=povm_calibrated` build-prereq (compile-time gate; absence collapses the binary).
- m9 namespace_guard runtime assertion on `workflow_id` AP30 prefix.

**Aspect-OUT**
- None. m14 does not write to the trust layer; it carries Watcher Class-C / Class-I observations passively via emission shape.

**Failure modes mitigated**
- F2 (sample-size inflation) — `Option::None` at `n < MIN_SAMPLE_SIZE`; never inflate with synthetic value.
- AP-Drift-07 (soak metric over-stated) — Watcher Phase 5C weekly synthesis independently re-computes lift from m7 snapshot; mismatch flags.
- AP-V7-09 (substrate-frame engine confusion) — input is operationalised purely from m7 (no "user intent" surrogate).

## §4 Public surface (spec, NOT compileable Rust)

The following Rust blocks are **specification artefacts** carried through from the cluster-spec; they are read as a contract, not as source files. No `.rs` is authored under HOLD-v2.

```rust
// src/m14_lift/mod.rs (SPEC — markdown only, not source)

/// Rolling evidence state for the habitat-outcome-lift metric.
///
/// `lift` is `None` when fewer than `MIN_SAMPLE_SIZE` runs are present in the
/// window.  Callers MUST treat `None` as "insufficient evidence" and hold
/// their current state rather than treating absence as zero lift.
///
/// F2 invariant: `ci_half` is always `Some` whenever `lift` is `Some`;
/// downstream consumers that strip CI bars violate the spec gate.
#[derive(Debug, Clone)]
pub struct LiftSnapshot {
    pub lift: Option<f64>,
    pub ci_half: Option<f64>,
    pub n: usize,
    pub latest_ts_ms: i64,
    pub computed_at: std::time::SystemTime, // AP-Hab-13 freshness anchor
}

/// Per-workflow contribution to the aggregate habitat-outcome-lift.
#[derive(Debug, Clone)]
pub struct WorkflowLiftContribution {
    pub workflow_id: WorkflowId, // newtype — AP30 prefix-guarded
    pub delta: f64,              // approximately [-1.0, +1.0]; m31 clamps further
    pub run_count: usize,
    pub individually_significant: bool, // run_count >= MIN_SAMPLE_SIZE
}

pub const MIN_SAMPLE_SIZE: usize = 20;            // F2 hard gate (per v1.3 §5)
pub const DEFAULT_WINDOW_SIZE: usize = 120;       // run-count, not time
pub const DEFAULT_CASCADE_WEIGHT: f64 = 0.6;      // env-overridable
pub const DEFAULT_COST_WEIGHT: f64 = 0.4;         // weights MUST sum to 1.0
```

`WorkflowId` is a newtype around `String` (no naked stringly-typed IDs). Public surface re-exported from `workflow_core::stats::wilson_ci` is the single-point-of-truth for the F2 computation; m14 calls into it rather than re-implementing.

## §5 Algorithm sketch — Wilson CI + composite lift

The formal definition (carried from cluster spec; locked in v1.3 §5):

```
// For a rolling window W of N WorkflowRunRow records:
//
//   cascade_success_rate = |{ r in W : r.cascade_success }| / N
//   cost_per_decision    = mean( r.cost_tokens / max(r.decisions_made, 1) ) for r in W
//   baseline_cost        = mean( r.decision_cost_baseline ) for r in W
//   cost_lift            = (baseline_cost - cost_per_decision) / baseline_cost
//                          // positive → cheaper than baseline
//
//   habitat_outcome_lift = 0.6 * cascade_success_rate
//                        + 0.4 * cost_lift.max(-1.0).min(1.0)
//
// Wilson 95% CI on the cascade-success Bernoulli proportion (z = 1.96):
//
//   p       = cascade_success_rate
//   n       = N
//   ci_half = z * sqrt( p*(1-p)/n + z^2/(4 n^2) ) / (1 + z^2/n)
//
// The composite CI is propagated from the cascade-success Bernoulli component
// (dominant term, well-defined finite-sample distribution).  Cost-lift CI is
// approximated as zero contribution to ci_half at v1.3 — explicit limitation;
// see Open Questions §13.
```

**Wald CI is FORBIDDEN** (produces negative lower bounds at small n; per v1.3 §5 + [KEYWORDS_20 #8](../../../ai_docs/optimisation-v7/KEYWORDS_20.md)). The shared helper `workflow_core::stats::wilson_ci(n_success, n_total, z=1.96) -> Option<(lower, upper)>` returns `None` for `n < MIN_SAMPLE_SIZE`; m14 surfaces that `None` outward unchanged. Any module bypassing the shared helper is a verify-sync failure.

The 0.6 / 0.4 weighting is configurable via `WF_LIFT_CASCADE_WEIGHT` env-var; the builder validates `cascade_weight + cost_weight == 1.0 ± ε` at startup and returns `AggregatorError::InvalidWeights` on failure. The composite penalises both cheap failures (cost-lift alone) and expensive successes (cascade-success alone).

## §6 Aggregation cycle (m16-Hebbian shape, Cat 05 lift)

Four-step cycle on a 5-minute timer (configurable; not per-row to avoid write amplification into m11/m31):

1. **Decay** — slide rolling window: evict runs older than `DEFAULT_WINDOW_SIZE` from in-memory `VecDeque` (O(1) `pop_front`).
2. **Ingest** — paginated SQL read from m7 since `latest_ts_ms`; push to `VecDeque` back.
3. **Compute** — recalculate `LiftSnapshot` and per-workflow contributions over current window contents using the shared `wilson_ci` helper.
4. **Emit** — write `LiftSnapshot` to m11's watch channel; write `Vec<WorkflowLiftContribution>` to m31's watch channel; write `(ltp, ltd)` lists to m42's outbox.

Structurally analogous to `m16_hebbian_engine.rs` (boilerplate Cat 05); aggregator state pattern lifted from `habitat-nerve-center_m3_aggregator_mod.rs` (Cat 06). Snapshot store uses the canonical `Arc<RwLock<Option<T>>>` clone-on-read pattern; locks dropped inside brace blocks, never held across an `.await`.

## §7 Per-workflow lift-contribution → m31 modulation

m14 emits a `Vec<WorkflowLiftContribution>` alongside the aggregate snapshot. m31's selection weight computation (see [cluster-G plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-G.md)) consumes the list via:

```
base_weight     = ralph_fitness * diversity_distance_bonus
lift_factor     = if individually_significant {
                      1.0 + delta.clamp(-0.3, +0.3)
                  } else { 1.0 }
final_weight    = (base_weight * lift_factor).max(0.0)
```

The `clamp(-0.3, +0.3)` bounds prevent any single workflow from dominating selection regardless of extreme lift — RALPH fitness-weighted decay invariant (P0 #8) carried into the lift signal. `individually_significant == false` causes m31 to **hold** that workflow's base weight unchanged (do not zero, do not extrapolate). This is the m31-side mirror of m14's F2 refusal discipline.

The m42 connection: m14 emits two ID lists per cycle — `ltp_workflows` (where `delta > 0.0 && individually_significant`) and `ltd_workflows` (where `delta < 0.0 && individually_significant`). m42 converts to `workflow_trace_*` namespace pathway writes against stcortex (per v1.3 Appendix A — POVM decoupled). Workflows on neither list are NOT touched. m42 never reinforces against absence of evidence.

## §8 Error taxonomy (thiserror, no unwrap)

```rust
// src/m14_lift/mod.rs (SPEC)

#[derive(Debug, thiserror::Error)]
pub enum AggregatorError {
    #[error("insufficient samples: n={n} < MIN_SAMPLE_SIZE={}", MIN_SAMPLE_SIZE)]
    InsufficientSamples { n: usize },

    #[error("database read failed: {0}")]
    DbRead(#[from] sqlx::Error),

    #[error("config validation failed: cascade_weight + cost_weight must equal 1.0 (got {sum:.6})")]
    InvalidWeights { sum: f64 },

    #[error("emit channel closed: downstream consumer disappeared")]
    EmitChannelClosed,
}
```

All paths return `Result<T, AggregatorError>`. Zero `.unwrap()` / `.expect()` outside `#[cfg(test)]`. `#![forbid(unsafe_code)]` on the module. No `println!` / `eprintln!` — tracing only.

## §9 CC-3 cross-cluster contract (E → F)

m14's `Option<LiftSnapshot>` is the **gate signal** for Cluster F iteration. The enforcement point is `ProposalBuilder::build()` in m23, which takes `Option<LiftSnapshot>` as a constructor parameter and returns `Err(ProposalError::SampleSizeBelowF2 { workflow_id, n })` on `None`. There is no runtime bypass. m20 (PrefixSpan), m21 (variant builder), m22 (K-means) feed m23; none may construct a `Proposal` against a domain where m14 has not certified `Some(Lift)`.

Iterators that operate on domains with `n < MIN_SAMPLE_SIZE` or whose lift CI spans zero emit a `NEEDS-MORE-DATA` advisory rather than a proposal (per cluster spec §CC-3). The advisory is a witness emission and may itself surface as a Cluster H NexusEvent at the Operator's discretion; it is NOT a proposal artefact.

## §10 Watcher class pre-position (per cluster plan)

- **Class C — confidence-gate refusal** fires at every `Option::None` emission. Refusal IS correct behaviour; not failure. Watcher tick log records `m14::Refused { workflow_id, reason: BelowF2(n=N) }`.
- **Class I — Hebbian silence** escalates if m14 emits `Some(Lift)` for >5 workflows over 7 days yet substrate `learning_health` does not trend up — implies CC-3 firing without CC-5 closure (Cluster H decorative).
- **Class E — ancestor-rhyme** indirect surface if m14 returns `None` continuously across the entire window (n=0 for 7+ days) — implies upstream m7 ingest is broken or no workflows are running through the engine (pain-source-not-found per W-4 mitigation).

## §11 Test budget (70 tests · G6 mutation kill ≥ 80% on `wilson::*`)

Per [TEST_DISCIPLINE](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) matrix:

| Family | Count | Notes |
|---|---:|---|
| F-Unit | 30 | per-arm coverage; Wilson edge cases at n=20 boundary, n=1000, p=0.0, p=1.0, p=0.5 |
| F-Property | 8 | `wilson_low ≤ mean ≤ wilson_high`; monotonic in n; modulation bounded `clamp(-0.3, +0.3)`; `lift == None` iff `n < MIN_SAMPLE_SIZE` |
| F-Fuzz | 0 | numeric only; proptest covers input space |
| F-Integration | 15 | m7 → m14 wiring (real `WorkflowRunRow` stream); m14 → m23 evidence-gate handshake; m14 → m31 modulation channel |
| F-Contract | 4 | `LiftSnapshot` serde round-trip; `Option::None` preserved across serialisation |
| F-Regression | 3 | reserved for post-deploy bug discoveries |
| F-Mutation | budget | `cargo mutants --regex 'm14_lift::wilson::.*'` ≥ **80%** kill (Wilson is F2 backbone) |

Canonical boundary tests (must exist by name):

- `lift_none_when_n_below_20`
- `lift_some_when_n_exactly_20`
- `lift_none_when_n_equals_19_boundary_check`
- `ci_half_always_some_when_lift_some`
- `ci_half_none_when_lift_none`
- `cost_lift_negative_when_workflow_more_expensive_than_baseline`
- `cascade_weight_and_cost_weight_must_sum_to_one`
- `window_eviction_drops_oldest_first`
- `workflow_delta_positive_when_above_aggregate`
- `individually_significant_false_when_run_count_below_20`
- `concurrent_ingest_and_read_safe_no_deadlock`
- `emit_none_to_m11_when_insufficient`
- `wilson_ci_matches_scipy_reference_within_1e6` (differential test vs `scipy.stats.proportion_confint` reference vector)

## §12 Boilerplate-lift map (≈25% lifted, ~80 LOC fresh authorship)

| Source | What is lifted | Reuse % |
|---|---|---:|
| `habitat-nerve-center_m3_aggregator_mod.rs` (Cat 06) | `Arc<RwLock<Option<T>>>` snapshot store; clone-on-read; AggregatorError shape | 70% |
| `habitat-nerve-center_main.rs` (Cat 06) | `VecDeque` ring buffer with `HISTORY_CAPACITY` constant pattern | 60% |
| `m16_hebbian_engine.rs` (Cat 05) | 4-step decay → ingest → compute → emit cycle | 80% |
| `povm-v2 magnitude-weighted aggregator` (CR-2 ref `e2a8ed3`) | magnitude-weighted vs binary aggregation pattern, transposed to lift-vs-confidence | conceptual |

Fresh authorship (~80 LOC, NEW): Wilson-CI numerics, F2 n≥20 guard, composite lift formula, per-workflow delta computation, m31 contribution gradient. Cluster E owns no structural-gap fresh-primitive LOC (Gap 1 = Cluster F KEYSTONE, Gap 2 = Cluster D m11, Gap 3 = Cluster G/D EscapeSurfaceProfile); m14 is *adjacent* to Gap 1 — it produces the evidence Gap 1 consumes — but no fresh-primitive authorship lives here.

## §13 Open questions, gaps, antipatterns

**Open questions (deferred to v1.4 or post-soak)**
1. **Cost-lift CI under-propagation.** The composite CI uses only the cascade-success Bernoulli component. A robust treatment would propagate the cost-lift sample variance via the delta method. v1.3 acknowledges this as an approximation; tightening is a Phase 5 hardening item.
2. **Window size: run-count vs time.** Default `DEFAULT_WINDOW_SIZE = 120` runs. For very low-traffic workflows a time-based window (e.g., last 30 days) may evict faster than evidence accumulates. Open: env-overridable hybrid `min(N runs, T time)` cap? — deferred until soak data shows actual eviction pressure.
3. **Atuin trajectory anchor.** Per [INTEGRATION/atuin-integration.md](../../../ai_docs/optimisation-v7/INTEGRATION/atuin-integration.md) (proposed) `wt-lift-watch <workflow_id>` reads m14 outputs at 30s cadence into `~/.local/share/atuin/history.db`. Hook contract not yet locked.
4. **m14 ↔ stcortex pathway.weight feedback.** The m42 → stcortex write is unidirectional. Should m14 read back stcortex pathway.weight as a corroborating signal? Risk: circular reinforcement. Open.

**Antipatterns explicitly avoided (per [ANTIPATTERNS.md](../../../ANTIPATTERNS.md) + cluster plan)**
- AP-WT-F2 (sample-size inflation) — construction-time refusal; `Option::None` never silently `0.0`.
- AP-V7-04 (keyword overgrowth) — `LiftSnapshot` carries exactly four fields + freshness anchor; new fields require spec amendment.
- AP-Hab-13 (runbook probe freshness drift) — `computed_at: SystemTime` on every emission; m31 ignores lifts >24h old.
- AP-Test-01 (coverage theatre on Wilson CI) — mutation kill ≥80% on `m14_lift::wilson::*`; differential test vs scipy.
- AP30 (POVM namespace collision) — m14 does not write to stcortex/POVM directly; m42 owns that write-path under `workflow_trace_*` prefix.
- AP-Drift-07 (soak metric over-stated) — Watcher Phase 5C independently re-computes from m7 snapshot.

**Standing constraints carried into m14**

| Constraint | Enforcement point |
|---|---|
| F2 n≥20 + CI bars | `LiftSnapshot.ci_half` always `Some` whenever `lift.is_some()` |
| W1 narrowed stcortex consumer | m14 reads m7 SQLite only; never reads stcortex directly |
| AP30 namespace prefix on POVM/stcortex writes | m42 owns the write; m14 emits ID lists only |
| Hard refusal: no HTTP server | m14 exposes no listening socket |
| Hard refusal: no POVM writes | m42 is the exclusive write-path; v1.3 Appendix A: stcortex-only post-pivot |
| `#![forbid(unsafe_code)]` + zero `.unwrap()` outside tests | All paths return `Result<T, AggregatorError>` |
| 50+ tests minimum (m14 budget: 70) | Test family table §11 |
| Tracing only (no `println!`) | `tracing::info!` for cycle ticks; `tracing::warn!` for refusals; `tracing::error!` for `EmitChannelClosed` |

---

> *m14 spec authored 2026-05-17 (S1001982) · planning-only · HOLD-v2 active · gates G1-G9 required before build · CC-3 owner · refusal-to-emit is load-bearing semantic.*
> *Sister anchor: [m15](m15_pressure_register.md).*
