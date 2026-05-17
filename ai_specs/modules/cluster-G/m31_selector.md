---
title: m31 — `m31_selector` Rust spec
cluster: G — Bank/Select/Dispatch/Verify
layer: L7
binary: wf-dispatch
loc_estimate: ~240
test_count_min: 70
test_kinds: [unit, property, integration, contract, regression, mutation]
feature_gate: [api]
verb_class: select
cc_owns: []
cc_consumes: [CC-4 mid-stream, CC-5 read-back from H, CC-2 (m8/m9), CC-3 (m14 lift)]
gap_owner: [none]
boilerplate_lift_pct: 25
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A
hold_v2_compliant: true
---

# m31 — `m31_selector` Rust spec

> Back to: [vault cluster-G spec](../../../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · [V7 cluster-G plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-G.md) · [GENESIS v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · vault [[cluster-G-bank-select-dispatch-verify]]
>
> Sister modules: [m30](m30_curated_bank.md) · [m31](m31_selector.md) · [m32](m32_conductor_dispatcher.md) · [m33](m33_verifier.md)

## 1. Purpose & invariants

`m31_selector` IS the **diversity-enforced selection function** that picks one — or zero — workflows from m30's bank for the next dispatch cycle. It composes four signals into a scalar score (`α·fitness + β·recency + γ·frequency + δ·diversity`) and then applies three diversity gates adapted from ORAC's `m40_mutation_selector` algebra (10-generation cooldown, 50% mono-parameter rejection, round-robin cycling across lineages). Its output feeds m33 (verifier) and ultimately m32 (dispatcher). The verb class is **select** — bounded by the v1.3 active-verb table: m31 does NOT recommend (no human-facing output), does NOT mine (m20 owns that), does NOT propose (m23 owns that). Selection is for downstream dispatcher consumption only.

The module MUST guarantee four invariants. **First — composite-score determinism**: given the same `(bank_state, selection_context, now_ms)` triple, `select()` returns the identical ranked Vec. No global RNG; no system-time read inside scoring (the `now_ms` input is the only time source). **Second — diversity-bonus gate against monotony**: the δ-component drops to 0.0 for any workflow whose `id` appears in `recent_lineages` within the 10-event cooldown window, suppressing repeat selection by construction rather than by post-hoc filter. This is the BUG-035 mitigation (ORAC mono-parameter mutation trap). **Third — NoSelection-against-degraded-substrate (GAP-Substrate-01)**: when substrate condition is unhealthy (current habitat: LTP/LTD = 0.043, 35× below healthy 1.5–4.0), `select()` returns `Option<SelectedWorkflow>::None` rather than picking a low-confidence winner — refusal-against-bad-signal is the safer path per Waiver W-3 mitigation. The substrate check is operationalised via `substrate_check::is_healthy(stcortex_snapshot)`; thresholds: PASS at `LTP/LTD > 0.5`, FLAG-AND-PASS in (0.05, 0.5], REFUSE at <= 0.05. **Fourth — Cluster H feedback closure (CC-5)**: m31 reads stcortex `workflow_trace_*` pathway weights at the start of each selection cycle and modulates `α` (fitness weight) by the stcortex multiplier. Without that read, the loop never closes; Watcher Class-I (Hebbian silence) fires if pathway weights remain flat across a tolerance window.

Frame violations m31 must structurally refuse: (a) any path that returns a selection when substrate is `REFUSE` (silent override = Waiver W-3 violation); (b) any post-hoc filtering that defeats the in-scoring diversity bonus (must not "select highest then drop if too similar" — the gate is *inside* the composite to keep determinism); (c) any human-meaningful label introduction (`pattern_label`, `recommended_tag`) per F11 + v1.3 § 3 verb table; m31 emits opaque `WorkflowId` only; (d) any `tokio::spawn` of sync HTTP for stcortex reads (AP29 — use the shared async client from `workflow_core::http`).

## 2. Public surface (Rust types — spec only, NOT compileable)

```rust
//! # m31_selector
//!
//! - **Layer**: L7 (Bank/Select/Dispatch/Verify, Cluster G)
//! - **Deps**: workflow_core::{types::{WorkflowId, LineageId, StepId}, errors::SelectionError, http::StcortexClient}, m30::{BankDb, AcceptedWorkflow}, m14::Lift
//! - **Tests**: 70 (35 unit + 8 property + 0 fuzz + 15 integration + 5 contract + 7 regression + mutation ≥80%)
//! - **Features**: api
//! - **Platform**: Linux; pure-Rust scoring; async stcortex read (1× per selection cycle)
//! - **Impl Notes**: Lifts ~70% of ORAC m40_mutation_selector for diversity algebra; composite-score numerics + substrate_check authored fresh
//! - **Related Docs**: [vault cluster-G spec](../../../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) · [V7 cluster-G](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-G.md)

/// One ranked candidate output by `select()`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RankedWorkflow {
    pub workflow: AcceptedWorkflow,
    pub composite_score: f64,
    pub fitness_score: f64,
    pub recency_score: f64,
    pub frequency_score: f64,
    pub diversity_score: f64,
    pub selection_rank: usize,
}

/// The "winner" passed down to m33 / m32. `None` is a valid outcome.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SelectedWorkflow {
    pub workflow_id: WorkflowId,
    pub composite_score: f64,
    pub diversity_features: DiversityFeatures,
}

/// Rolling context updated by m32 after a dispatch lands.
#[derive(Debug, Clone, Default)]
pub struct SelectionContext {
    pub recent_sequences: std::collections::VecDeque<Vec<StepId>>, // last 10 dispatched
    pub recent_lineages:  std::collections::VecDeque<LineageId>,   // last 10 dispatched
}

/// Selection-time tunables. Defaults match v1.3 § 3 m31 row.
#[derive(Debug, Clone)]
pub struct SelectorConfig {
    pub alpha: f64,                // fitness weight       = 0.40
    pub beta:  f64,                // recency weight       = 0.25
    pub gamma: f64,                // frequency weight     = 0.20
    pub delta: f64,                // diversity weight     = 0.15
    pub recency_half_life_ms: i64, // default 7d
    pub freq_scale: f64,           // default 0.05
    pub cooldown_generations: usize,    // default 10
    pub mono_param_threshold: f64, // default 0.50
    pub lineage_score_tolerance: f64,   // default 0.05
    pub min_selection_score: f64,  // floor; default 0.05
    pub substrate_thresholds: SubstrateThresholds,
}

#[derive(Debug, Clone)]
pub struct SubstrateThresholds { pub refuse_at: f64, pub flag_below: f64, pub pass_above: f64 }

#[derive(Debug, thiserror::Error)]
pub enum SelectionError {
    #[error("selector: bank read failed: {0}")]
    BankRead(#[from] crate::m30::BankError),
    #[error("selector: stcortex read failed: {0}")]
    Stcortex(String),
    #[error("selector: substrate refusal — LTP/LTD={ratio} below {refuse_at}")]
    SubstrateRefuse { ratio: f64, refuse_at: f64 },
    #[error("selector: config invalid: weights must sum to ~1.0 (got {sum})")]
    ConfigInvalid { sum: f64 },
}

pub struct Selector { /* private — holds config + StcortexClient handle */ }

impl Selector {
    pub fn new(cfg: SelectorConfig, stcortex: StcortexClient) -> Result<Self, SelectionError>;

    /// Read-only against the bank + stcortex; returns None on substrate FLAG-AND-PASS+empty-eligible
    /// or REFUSE (the latter also emits Watcher Class-I via `WatcherEmit::class_i`).
    pub async fn select(
        &self,
        bank: &BankDb,
        ctx: &SelectionContext,
        now_ms: i64,
        n: usize,
    ) -> Result<Option<SelectedWorkflow>, SelectionError>;

    /// Top-N ranked view for m33 (verifier walks in order until PASS).
    pub async fn rank(
        &self,
        bank: &BankDb,
        ctx: &SelectionContext,
        now_ms: i64,
        n: usize,
    ) -> Result<Vec<RankedWorkflow>, SelectionError>;

    /// Called by m32 AFTER a confirmed Conductor handoff. Mutates ctx in-place.
    pub fn record_selection(&self, ctx: &mut SelectionContext, selected: &AcceptedWorkflow);
}
```

## 3. Internal data structures

A private `CompositeScorer` holds the `SelectorConfig` + a memoised `bigram_index` over `recent_sequences` (HashSet of `&[StepId; 2]` windows) so the Jaccard similarity computation is O(|w.steps|) per candidate rather than O(|recent| × |w.steps|). The `SubstrateGate` wraps `StcortexClient::get_pathway_weights("workflow_trace_*")` + `get_field_metrics()` and reduces to a single `SubstrateHealth { ratio, verdict: Pass | Flag | Refuse }`. The `LineageCycle` maintains a deterministic round-robin pointer (lineage first-acceptance order) so workflows tied within `lineage_score_tolerance` rotate fairly across calls within the same selection cycle.

## 4. Data flow

- **INPUT FROM:** `m30::BankDb::eligible(now_ms, 200)` (candidate pool); `m11::ralph_decay_weight` (per-workflow, already on `AcceptedWorkflow`); `m14::Lift` (clamped `[-0.3, +0.3]` modulation on `α`); `stcortex` pathway weights (`workflow_trace_*` ns via async HTTP); `SelectionContext` (rolling 10-window provided by caller).
- **OUTPUT TO:** m33 (`rank()` for ordered verification walk); m32 (`select()` for single-winner dispatch + `record_selection` mutate after dispatch confirmed).
- **SUBSTRATE TOUCHED:** stcortex (read-only, async); bank (read-only via m30 API).
- **WRITES:** none directly; m31 only mutates the caller-owned `SelectionContext` in `record_selection`.

## 5. Algorithm sketch

```text
select(bank, ctx, now_ms, n):
    health = SubstrateGate::probe(stcortex)
    match health.verdict:
        Refuse  -> emit Watcher::Class_I; return Err(SubstrateRefuse)
        Flag    -> proceed but tag output for Watcher Class-C
        Pass    -> proceed
    candidates = bank.eligible(now_ms, 200)
    if candidates.is_empty(): return Ok(None)
    bigrams_recent = build_bigram_index(ctx.recent_sequences)
    scored = candidates.iter().map(|w| {
        f = w.ralph_decay_weight                                 # [0,1] from m11
        f *= 1.0 + clamp(m14::lift_for(w.lineage), -0.3, +0.3)   # m14 modulation
        r = exp(-(now_ms - w.last_verified_at.unwrap_or(w.accepted_at)) / cfg.recency_half_life_ms)
        q = 1.0 / (1.0 + w.dispatch_count as f64 * cfg.freq_scale)
        d = if ctx.recent_lineages.contains(&w.lineage) within cooldown {
                0.0                                              # gate by construction
            } else {
                1.0 - max_bigram_jaccard(w.steps, bigrams_recent)
            }
        composite = α*f + β*r + γ*q + δ*d
        RankedWorkflow { workflow: w, composite, … }
    })
    .filter(|r| r.composite >= cfg.min_selection_score)
    apply_mono_parameter_gate(&mut scored)              # 50% same-lineage rejection
    apply_round_robin(&mut scored, &ctx.recent_lineages) # tie-break by lineage rotation
    scored.sort_by(|a,b| b.composite.total_cmp(&a.composite))
    return Ok(scored.first().map(into_selected))

apply_mono_parameter_gate(scored):
    let top10 = scored.iter().take(10)
    let lineage_counts = group_by_lineage(top10)
    for (lin, count) in lineage_counts:
        if count > 5 (50% of 10):
            drop tail entries from lin until count <= 5
            backfill from highest-scoring candidates of other lineages

apply_round_robin(scored, recent_lineages):
    for window of entries scoring within lineage_score_tolerance:
        rotate so least-recently-dispatched lineage appears first

record_selection(ctx, selected):
    ctx.recent_sequences.push_back(selected.steps.iter().map(StepDef::id).collect())
    ctx.recent_lineages.push_back(selected.lineage.clone())
    while ctx.recent_sequences.len() > 10: ctx.recent_sequences.pop_front()
    while ctx.recent_lineages.len()  > 10: ctx.recent_lineages.pop_front()
```

## 6. Boilerplate lifts

Per vault cluster-G spec § m31 Boilerplate lift and V7 cluster-G plan:

| Source | Lift | % |
|---|---|---:|
| ORAC `m40_mutation_selector::diversity` | round-robin cycling + 50% mono-parameter rejection + 10-gen cooldown table | 70% |
| LCM TierExecutor selection pattern | confidence-floor gating shape | 50% |
| `povm-v2_lifecycle.rs` decay primitives | exp-decay helper (recency component) | 40% |
| `m10_pattern.rs` (memory-injection) | two-counter pattern + three-tier equilibrium framing | 70% |
| **composite-score numerics + substrate_check** | — | **0% (novel ~80 LOC)** |

Net: ~160 LOC lifted / ~80 LOC novel.

## 7. ME v2 m1_foundation patterns referenced

- `error.rs` — `SelectionError` thiserror enum, structured named-field variants (`{ ratio, refuse_at }`, `{ sum }`); no `Box<dyn Error>`.
- `resources.rs` — `//!` docstring block.
- `self_model.rs` — `Selector` exposes `SelectionExplanation { fitness_score, recency_score, frequency_score, diversity_score, weights }` so the engine can explain *why* a candidate was chosen (consumed by `wf-dispatch explain` subcommand).
- `tensor_registry.rs` — 12D framing is over-engineered for m31's 4D score; we use a `[f64; 4]` named-tuple `ScoreComponents` with helper accessors.
- `logging.rs` — tracing structured emit on every `select()` (substrate verdict, candidate count, winner id or None, composite_score).
- `config.rs` — `SelectorConfig::Default` matches v1.3 § 3 m31 row; env-overridable via `WORKFLOW_TRACE_SELECTOR_*` prefix.

## 8. Test strategy

- **Test kind**: unit (35) + property (8) + integration (15) + contract (5) + regression (7)
- **Test count**: 70 minimum (per [TEST_DISCIPLINE matrix](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) row m31; cluster G total 290)
- **Mutation budget**: ≥80% kill on `composite_score.rs` + `diversity/` (G6 m31 threshold)
- **Properties tested** (F-Property 8):
  - Composite monotonic in each component holding others fixed.
  - 10-gen cooldown FIFO (`record_selection` 11× evicts the first entry).
  - Round-robin fairness: across N tied lineages × 3N selections, count per lineage differs by ≤1.
  - Diversity bonus floor: any cooldown-suppressed candidate has `diversity_score == 0.0`.
  - Determinism: same `(bank, ctx, now_ms)` triple → identical Vec across 100 runs.
  - Weight sum guard: `α + β + γ + δ` ≈ 1.0 (within 1e-9) for default config.
  - Substrate REFUSE always returns `Err(SubstrateRefuse)`, never `Ok(Some(_))`.
  - `record_selection` window bounded at 10 (never grows past).

Key invariants (sample of 70; full enumeration in vault cluster-G spec § Tests m31):

1. Empty bank → `select()` returns `Ok(None)`.
2. Substrate REFUSE (LTP/LTD=0.04) → returns `Err(SubstrateRefuse)`.
3. Substrate FLAG (LTP/LTD=0.2) → returns `Ok(Some(_))` with Watcher Class-C tag.
4. Substrate PASS (LTP/LTD=2.0) → returns `Ok(Some(_))` without flag.
5. Bigram similarity of identical step sequences = 1.0.
6. Bigram similarity of disjoint sequences = 0.0.
7. Bigram similarity is symmetric: `sim(a,b) == sim(b,a)`.
8. Recency score for `last_verified_at = now_ms` ≈ 1.0.
9. Recency score for `last_verified_at = now_ms - 7d` ≈ 0.5 (half-life check).
10. Frequency penalty for `dispatch_count = 0` = 1.0.
11. Frequency penalty strictly decreases as `dispatch_count` increases.
12. Diversity = 1.0 for a workflow whose steps are entirely novel.
13. Diversity = 0.0 for a workflow in `recent_lineages` cooldown.
14. Mono-parameter gate: 6 of top-10 same-lineage → 1 dropped from that lineage.
15. Mono-parameter gate: 5 of top-10 same-lineage → no drop (boundary).
16. Round-robin: 3 lineages with identical composites cycle in first-acceptance order.
17. `record_selection` after 11 selections leaves `recent_sequences.len() == 10`.
18. `record_selection` preserves `recent_lineages.len() == recent_sequences.len()`.
19. `min_selection_score` floor: candidates below floor excluded from output.
20. Config with `α+β+γ+δ ≠ 1.0` returns `ConfigInvalid` on `new()`.

(remaining 50 — including F-Contract stcortex-shape snapshot tests, F-Integration `tests/integration/m31_against_degraded_substrate.rs` per V7 G6 (replays captured LTP/LTD=0.043 snapshot), and CC-5 close-loop test — enumerated in vault spec.)

## 9. Antipatterns to avoid

- **BUG-035** (ORAC mono-parameter mutation trap) — 50% rejection + 10-gen cooldown adopted directly; tested as gate.
- **AP-WT-F1** (ossification) — composite includes recency (β=0.25) + diversity (δ=0.15); ossified workflows lose to fresher alternatives by construction.
- **AP-WT-F11** (cascade monoculture) — no human-meaningful labels; opaque `WorkflowId` only; `lineage` is opaque m4 cluster id.
- **GAP-Substrate-01** (degraded-substrate selection) — REFUSE verdict short-circuits to error; FLAG verdict emits Watcher Class-C alongside selection.
- **AP-V7-09** (substrate-frame confusion) — all inputs operationalised (no "user preferred" surrogate); selection is over substrate trajectories, not user intent.
- **AP29** (sync HTTP in `tokio::spawn`) — stcortex read uses shared async `StcortexClient`; no `block_on` inside spawned tasks.
- **AP30** (namespace drift) — `workflow_core::namespace::WORKFLOW_TRACE_PATHWAY_PREFIX` for stcortex query; no literal `"workflow_trace_"` string.
- **Newly surfaced**: post-hoc filter defeating in-scoring gate — must NOT add a `filter(|r| !too_similar(r))` after sort; that breaks determinism guarantees (regression slot reserved).

## 10. Useful patterns applied

- ORAC mutation-selector algebra (PATTERNS.md § Module-level patterns).
- thiserror error enums (GOLD_STANDARDS rule 9).
- Newtype discipline (GOLD_STANDARDS rule 8).
- `//!` docstring block (GOLD_STANDARDS rule 13).
- Pure-function scoring + caller-owned mutable context (functional core / imperative shell).
- Substrate-aware refusal-mode (Waiver W-3 + W-5 mitigation).
- Memoised bigram index for O(|w.steps|) inner loop.

## 11. Cross-cluster contracts

- **CC-4 mid-stream**: m31 consumes the post-acceptance state of m30 and produces the input to m33 (verification target) and m32 (dispatch winner). Does not own any contract surface; is a pure transform.
- **CC-5 read-back from H**: m31 reads stcortex pathway weights `workflow_trace_*` written by m42; updates to those weights modulate `α` (fitness) on the next selection cycle. This is the slow-loop closure (days/weeks). If pathway weights remain flat across the Watcher Class-I tolerance window (default 7d), the loop is broken and an alert fires.
- **CC-3 (E → F → G)**: m31 reads m14 `Lift` per lineage (clamped `[-0.3, +0.3]`) as a multiplier on the fitness component; m14 is the empirical-evidence aggregator (Wilson CI per `[F2 hard gate](../../../ai_docs/GENESIS_PROMPT_V1_3.md#-5--f2-sample-size-hard-gate-per-report-type-definitions)`).
- **CC-2 trust layer**: m8 build-prereq (POVM calibrated), m9 namespace_guard (stcortex namespace literal forbidden) bind on m31; m10 Ember CI gate verifies that test fixtures cannot accidentally bypass substrate REFUSE.

## 12. Open questions for G5 interview / Zen G7 audit

1. **stcortex read cadence**: per-selection vs per-N-selections vs background refresh thread? Per-selection is correct but adds latency; background refresh is faster but introduces a freshness window where the loop appears closed but isn't.
2. **Substrate threshold tuning**: REFUSE at LTP/LTD ≤ 0.05 vs ≤ 0.1? Current habitat is 0.043 (REFUSE band) — the engine would shipped-disabled by default until substrate recovers. Is that intended (forcing operator awareness) or a soft-launch problem?
3. **Lift clamp `[-0.3, +0.3]`**: agrees with m14 spec? Verify-sync invariant; if m14 clamps `[-0.5, +0.5]` the asymmetry could be a 2× silent attenuation.
4. **Recency half-life default 7d**: matches the m33 verify TTL (7d) — coincidence or intentional coupling? If intentional, document; if coincidence, decouple to avoid hidden invariant.
5. **Round-robin tie-break**: first-acceptance order vs random-with-seed-derived-from-`now_ms`? First-acceptance preserves determinism; seeded random preserves fairness across long sessions where first-acceptance ordering becomes hostile.

## 13. Implementation order (post-G9)

1. `error.rs` — `SelectionError` enum (`thiserror`); compile-only.
2. `composite_score.rs` — `ScoreComponents` + `compose()` + 8 unit tests (component monotonicity, weight sum guard).
3. `diversity/cooldown.rs` — 10-gen cooldown table; 6 unit tests (FIFO, idempotent eviction).
4. `diversity/mono_parameter.rs` — 50% mono-parameter rejection; 8 unit tests (boundary at 5/6).
5. `diversity/round_robin.rs` — first-acceptance cycling; 6 unit tests (fairness across N lineages).
6. `diversity/mod.rs` — orchestration; 4 unit tests.
7. `substrate_check.rs` — `SubstrateGate` + threshold logic; 5 unit + 3 contract tests (stcortex shape).
8. `mod.rs` — `Selector::new` + `select` + `rank` + `record_selection`; 8 unit tests.
9. Property tests (8) — proptest on monotonicity, FIFO, fairness, determinism, weight sum, REFUSE total, bound on window, diversity floor.
10. Integration tests (15) — `tests/m31_integration.rs` exercising m30→m31→m33→m32 cycle; CC-5 close-loop test; the V7-mandated `m31_against_degraded_substrate.rs` replay.
11. Contract tests (5) — insta snapshots for stcortex pathway-weight JSON shape + SelectionExplanation API stability.
12. Regression slots (7) — composite-score weighting drift, post-hoc-filter regression, cooldown-window-bypass, substrate-flag-silenced, etc.
13. Mutation pass — `cargo mutants --regex 'm31_selector::composite_score::.*|m31_selector::diversity::.*'`; ≥80% kill required.

---

> Back to: [vault cluster-G spec](../../../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · sister modules: [m30](m30_curated_bank.md) · [m31](m31_selector.md) · [m32](m32_conductor_dispatcher.md) · [m33](m33_verifier.md)
