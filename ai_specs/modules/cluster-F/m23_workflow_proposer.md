---
title: m23 — workflow_proposer (Cluster F · L6 · KEYSTONE)
module_id: m23
module_name: workflow_proposer
cluster: F — Iteration (KEYSTONE)
layer: L6
binary: wf-crystallise
feature_gate: [intelligence]
loc_estimate: 180
test_count_min: 60
test_kinds: [property]
verb_class: active
gap_owner: [Gap 1]
mutation_kill_target: 75%
status: planning-only · HOLD-v2 active · pre-G7 Zen audit
authority: Luke @ node 0.A (single-phase override 2026-05-17)
date: 2026-05-17 (S1001982)
kind: per-module spec
decisions_applied: [D-A]
---

# m23 — `workflow_proposer`

> **Sister modules (Cluster F):** [m20](m20_prefixspan_miner.md) · [m21](m21_variant_builder.md) · [m22](m22_kmeans_feature.md) · [m23](m23_workflow_proposer.md)
>
> **Vault:** [[cluster-F-iteration]] · **V7 plan:** [cluster-F plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md) · **Cluster spec:** [`../../../the-workflow-engine-vault/module specs/cluster-F-iteration.md`](../../../the-workflow-engine-vault/module%20specs/cluster-F-iteration.md) · **Matrix row:** [MODULE_MATRIX m23](../../MODULE_MATRIX.md)
>
> **Back to:** [project CLAUDE.md](../../../CLAUDE.md) · [GENESIS v1.3 § 1 + § 2 + § 3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · [PATTERNS](../../../PATTERNS.md) · [GOLD_STANDARDS](../../../GOLD_STANDARDS.md) · [ANTIPATTERNS](../../../ANTIPATTERNS.md)

---

## 1. Purpose

m23 is the **single output surface of Cluster F** and the **only path from iteration to the bank queue**. It aggregates m20's canonical patterns + m21's near-miss variants + m22's feature-cluster context + m14's Wilson-bounded lift into typed `WorkflowProposal` artefacts, applies **gradient-preservation** (N=3 near-miss variants beside each canonical), captures **deviation-as-evidence** from m32 bypass events, and writes each `Proposal` to `workflow_trace.db` table `proposals` with status `AwaitingReview`.

**m23 NEVER auto-promotes to m30.** This is the engine's single most-load-bearing invariant after F2 (Wilson CI sample-size gate). Per [GENESIS v1.3 § 2 Hard refusals](../../../ai_docs/GENESIS_PROMPT_V1_3.md):

> **No auto-promotion m23 → m30.** m23 proposes; humans review; m30 accepts. The proposer-to-bank gap is a Town Hall P0 invariant; m23 cannot insert into m30 without an explicit operator approval step. Bypassing this is an AP-Hab-class refusal-mode violation (Operator persona scar tissue).

Per [GENESIS v1.3 § 3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) m23 row: verbs `aggregate proposals` / `preserve gradient` are permitted, bounded by `n>=5 deviation-relaxed; outputs only feed m30 post-accept`.

m23 owns the third-largest share of Gap 1 (~100 LOC fresh) — the **composition logic** (merging canonical + variants + feature context + lift into a single typed proposal under construction-time F2 gate) and the **deviation-as-evidence joining** are both fresh authorship; the rest is lifted from `SKILL-pre-deploy-hardening.md` deviation-rationale schema (~80% reuse).

---

## 2. F2 Construction-Time Hard Gate

The single most critical thing m23 does is **refuse to construct a Proposal** that doesn't meet F2. The gate is at `ProposalBuilder::build()`, not at runtime read. This means: **a m23 proposal with `n_samples < 20` cannot exist in memory, let alone reach the bank queue.**

Per [GENESIS v1.3 § 5 F2 sample-size hard gate](../../../ai_docs/GENESIS_PROMPT_V1_3.md) and [cluster-F-iteration spec § F2 Hard Gate](../../../the-workflow-engine-vault/module%20specs/cluster-F-iteration.md):

```
n_samples >= 20 (default canonical proposal)
n_samples >= 5  (deviation-shaped variants only; explicit flag in serialised output)
```

The `Result<WorkflowProposal, ProposalError>` API **forces the caller** to handle the refusal path; there is no path that produces a proposal without going through `build()`.

**Wilson Score CI (locked):** every proposal carries `ci_lower`, `ci_upper`, `n_samples` computed via Wilson 95% CI (z = 1.959964). Not Wald (which can produce negative lower bounds or bounds outside `[0, 1]` for small n). The Wilson formula:

```
p_hat = successes / n
center = (p_hat + z²/(2n)) / (1 + z²/n)
margin = z * sqrt(p_hat*(1-p_hat)/n + z²/(4n²)) / (1 + z²/n)
ci_lower = max(0.0, center - margin)
ci_upper = min(1.0, center + margin)
```

Wilson is **not configurable per proposal**. The method is locked across all proposals to prevent later regressions where "Wald was used for this batch" creates inconsistency in archived proposals.

---

## 3. Inputs / Outputs / Substrates

**Reads from** (per [MODULE_MATRIX m23 row](../../MODULE_MATRIX.md)): `m22, m14`. Also reads m20 canonical patterns and m21 variants in practice — MODULE_MATRIX lists primary upstream.

- m20 — `Vec<Pattern>` (canonicals)
- m21 — `Vec<PatternVariant>` (top-K near-misses per canonical)
- m22 — `Vec<FeatureCluster>` (feature-space context)
- m14 — `Option<Lift>` (Wilson-bounded lift evidence)
- `workflow_trace.db` table `deviation_events` (written by m32 when operator bypasses a step)

**Writes to:** `operator review queue` (per MODULE_MATRIX) — concretely `workflow_trace.db` table `proposals` with `status = 'awaiting_review'`. This is the **single Cluster F write surface**.

**NEVER writes to:**

- m30's bank (proposals go to humans, not to the bank) — AP-V7-07
- stcortex (m13 owns all stcortex writes)
- atuin or injection.db
- POVM (workflow-trace POVM-decoupled per 2026-05-17 ADR)

**CC-4 (F → G) producer contract:** m23 emits `WorkflowProposal` artefacts into the queue. Human `wf-crystallise propose accept <id>` is the **boundary** between Cluster F and Cluster G m30. The transition is owned by m30's handler — m23 is blind to what happens after the proposal lands in the queue.

**Aspect-IN (Cluster D):** m8 (build_prereq), m9 (namespace guard), m10 (Ember 7-trait CI gate on proposal text). The Ember gate is novel for m23 — proposal payloads include serialised step token sequences and confidence bands; Ember 7-trait checks (per [GENESIS v1.3 § 3](../../../ai_docs/GENESIS_PROMPT_V1_3.md)) ensure no human-meaningful pattern labels leak.

**Verb class:** `active` — permitted: `aggregate proposals` / `preserve gradient`; bounded by F2 construction-time gate + no auto-promotion + outputs feed m30 only post-accept.

---

## 4. Public API (planning-spec; markdown only)

```rust
// planning-spec only — m23_workflow_proposer public surface
// rationale: F2 gate at construction time (typed Err on refusal);
//            never auto-promotes; deviation-evidence joining additive

use thiserror::Error;
use serde::{Deserialize, Serialize};
use crate::m20_prefixspan_miner::{Pattern, StepToken};
use crate::m21_variant_builder::PatternVariant;
use crate::m22_kmeans_feature::FeatureCluster;
use crate::m14_habitat_outcome_lift::Lift;

/// The canonical output unit of Cluster F.
///
/// A WorkflowProposal is NEVER auto-promoted to m30.
/// It is a read-only record presented to a human for evaluation.
/// Promotion requires explicit human action: `wf-crystallise propose accept <id>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowProposal {
    pub id: ProposalId,
    pub source: IteratorKind,        // Cascade | Battern | PromptPattern
    pub canonical: Pattern,
    pub variants: Vec<PatternVariant>, // <= MAX_VARIANTS (default 3)
    pub evidence: Evidence,
    pub deviation_evidence: Vec<DeviationEvent>,
    pub deviation_shaped: bool,       // true if a variant was added via m32 bypass evidence
    pub deviation_relaxed_n: bool,    // true if a variant uses n>=5 (instead of n>=20)
    pub status: ProposalStatus,       // AwaitingReview at construction
    pub generated_at: u64,            // Unix epoch ms
    pub cluster_lineage: Vec<u64>,    // m4 cluster IDs contributing
}

/// Evidence quad attached to every proposal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub canonical_support: usize,
    pub n_samples: usize,             // >= 20 (default) or >= 5 (deviation-shaped)
    pub ci_lower: f64,                // Wilson 95% CI
    pub ci_upper: f64,                // Wilson 95% CI
    pub feature_centroid: Option<[f64; 7]>, // from m22 if available
    pub confidence_band: ConfidenceBand,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceBand {
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
    pub is_narrow: bool,              // std_dev < 0.05
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviationEvent {
    pub bypassed_step: StepToken,
    pub rationale: String,
    pub cluster_id: u64,
    pub ts_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IteratorKind { Cascade, Battern, PromptPattern }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus { AwaitingReview, Accepted, Rejected }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProposalId(pub uuid::Uuid);

#[derive(Debug, Error)]
pub enum ProposalError {
    #[error("F2 violation: lift evidence missing (n<20 from m14)")]
    LiftEvidenceMissing,

    #[error("F2 violation: canonical support {0} below floor of 20")]
    SupportBelowFloor(usize),

    #[error("deviation-relaxed flag set but no variants provided")]
    DeviationFlagWithoutVariants,

    #[error("deviation-relaxed flag set; variant support {0} below relaxed floor of 5")]
    DeviationSupportBelowRelaxedFloor(usize),

    #[error("max variants exceeded: got {got} > max {max}")]
    MaxVariantsExceeded { got: usize, max: usize },

    #[error("database write failed: {source}")]
    DatabaseWrite { #[source] source: rusqlite::Error },

    #[error("no iterator output: all three iterators returned empty")]
    NoIteratorOutput,
}

pub struct ProposalBuilder {
    source: IteratorKind,
    canonical: Pattern,
    variants: Vec<PatternVariant>,
    feature_cluster: Option<FeatureCluster>,
    lift: Option<Lift>,
    deviation_evidence: Vec<DeviationEvent>,
    deviation_relaxed: bool,
}

impl ProposalBuilder {
    pub fn new(source: IteratorKind, canonical: Pattern) -> Self { /* ... */ }
    pub fn variants(self, v: Vec<PatternVariant>) -> Self { /* ... */ }
    pub fn feature_cluster(self, fc: FeatureCluster) -> Self { /* ... */ }
    pub fn lift(self, l: Lift) -> Self { /* ... */ }
    pub fn deviation_evidence(self, evts: Vec<DeviationEvent>) -> Self { /* ... */ }

    /// THE F2 GATE. Returns `Err` if any invariant is violated.
    /// No bypass path. No `unsafe`. No `unwrap()`.
    pub fn build(self) -> Result<WorkflowProposal, ProposalError> { /* see § 5 */ }
}

/// Aggregate proposals from m20-m22, apply gradient-preservation, join
/// deviation evidence, write to `proposals` table.
pub fn aggregate_and_write(
    db: &WorkflowDb,
    config: &ProposerConfig,
) -> Result<usize, ProposalError> { /* see § 5 */ }
```

---

## 5. Algorithm Sketch (planning-spec only)

```rust
// planning-spec only — m23_workflow_proposer::builder
// rationale: F2 gate enforced at CONSTRUCTION, not runtime; refusal returns typed Err

impl ProposalBuilder {
    pub fn build(self) -> Result<WorkflowProposal, ProposalError> {
        // 1. Lift evidence must exist (m14 returned Some)
        let lift = self.lift.ok_or(ProposalError::LiftEvidenceMissing)?;

        // 2. Canonical support must clear the floor
        let canonical_floor = if self.deviation_relaxed { 5 } else { 20 };
        if self.canonical.support < canonical_floor {
            return Err(ProposalError::SupportBelowFloor(self.canonical.support));
        }

        // 3. Deviation-relaxed without variants is a contradiction
        if self.deviation_relaxed && self.variants.is_empty() {
            return Err(ProposalError::DeviationFlagWithoutVariants);
        }

        // 4. MAX_VARIANTS cap (default 3)
        const MAX_VARIANTS: usize = 3;
        if self.variants.len() > MAX_VARIANTS {
            return Err(ProposalError::MaxVariantsExceeded {
                got: self.variants.len(),
                max: MAX_VARIANTS,
            });
        }

        // 5. Each variant honours its support floor (5 if deviation-shaped, 20 otherwise)
        for v in &self.variants {
            let variant_floor = if self.deviation_relaxed { 5 } else { 20 };
            if v.support < variant_floor {
                return Err(ProposalError::DeviationSupportBelowRelaxedFloor(v.support));
            }
        }

        // 6. Compute Wilson 95% CI for the canonical
        let (ci_lower, ci_upper) = wilson_95(lift.successes, lift.n_samples)?;

        // 7. Compute ConfidenceBand across canonical + variants
        let band = compute_confidence_band(&self.canonical, &self.variants);

        // 8. Construct the proposal
        Ok(WorkflowProposal {
            id: ProposalId(uuid::Uuid::new_v4()),
            source: self.source,
            canonical: self.canonical,
            variants: self.variants,
            evidence: Evidence {
                canonical_support: lift.n_samples,
                n_samples: lift.n_samples,
                ci_lower,
                ci_upper,
                feature_centroid: self.feature_cluster.map(|fc| fc.centroid),
                confidence_band: band,
            },
            deviation_evidence: self.deviation_evidence,
            deviation_shaped: !self.deviation_evidence.is_empty(),
            deviation_relaxed_n: self.deviation_relaxed,
            status: ProposalStatus::AwaitingReview,
            generated_at: now_ms(),
            cluster_lineage: lift.cluster_ids.clone(),
        })
    }
}

// planning-spec only — m23_workflow_proposer::deviation
// rationale: deviation events ADD evidence; never REDUCE the canonical or its variants

fn apply_deviation_evidence(
    proposal: &mut WorkflowProposal,
    events: &[DeviationEvent],
    rationale_similarity_threshold: f64,
) {
    // For each bypassed step in events, check if it's in the canonical.
    // If multiple events name the same step with consistent rationale (cosine similarity
    // >= threshold, default 0.8), add a "step X removed" variant with relaxed n>=5.
    let bypassed_steps: HashMap<StepToken, Vec<&DeviationEvent>> = events.iter()
        .filter(|e| proposal.canonical.steps.contains(&e.bypassed_step))
        .fold(HashMap::new(), |mut m, e| { m.entry(e.bypassed_step).or_default().push(e); m });

    for (step, evts) in bypassed_steps {
        if evts.len() < 5 { continue; } // relaxed floor
        if !rationale_consistent(&evts, rationale_similarity_threshold) { continue; }
        // Add a deviation-shaped variant with that step removed
        let removed_steps: Vec<StepToken> = proposal.canonical.steps.iter()
            .filter(|s| **s != step).cloned().collect();
        proposal.variants.push(PatternVariant {
            canonical_id: proposal.canonical.canonical_hash,
            variant_steps: removed_steps,
            edit_distance: 1.0 / proposal.canonical.steps.len() as f64,
            top_k_rank: proposal.variants.len(),
            support: evts.len(),
        });
        proposal.deviation_shaped = true;
        proposal.deviation_evidence.extend(evts.iter().cloned().cloned());
    }
}
```

**Cross-type deduplication:** when m20 (cascade), m21 (variants), and m22 (feature context) all reference the same m4 cluster, the proposer **does not emit duplicates**. A `BTreeMap<canonical_hash, WorkflowProposal>` maintains identity; the first source wins, others contribute lineage entries only.

---

## 6. Deviation-as-Evidence System

The deviation system is the engine's **feedback loop from operator behaviour back into proposal shape**. When m32 (the dispatcher) presents a step to the operator before execution (per P0 #11 escape-surface display) and the operator chooses to bypass a step with an explicit rationale, m32 writes a row to `deviation_events`. m23 reads these events and joins them to relevant proposals:

1. A deviation event naming `step X` as bypassed is joined to any proposal whose canonical pattern contains `step X`.
2. If the same step is bypassed multiple times with **consistent rationale** (cosine similarity on rationale strings >= 0.8 default), m23 interprets this as positive signal toward a "step X removed" variant.
3. The deviation-shaped variant (canonical pattern with `step X` removed) is added to the `variants` list if it meets a **relaxed support threshold** (n >= 5, not 20, because deviation events are rare by nature).
4. The proposal's `deviation_shaped: bool` field is set to `true` and `deviation_evidence` array is populated.

This closes the feedback loop:

```
m32 dispatch → operator bypass + rationale → deviation_events DB → m23 reads → proposal shaped → human reviews → m30 bank (if accepted)
```

**The loop never auto-promotes.** Deviation evidence is **additive, not reductive** — joining deviation events adds `deviation_evidence` entries and may add a deviation-shaped variant, but it does **not** remove the canonical or its standard variants. The human reviewer sees everything.

The lift pattern is ~80% from `SKILL-pre-deploy-hardening.md`'s `Verdict { agent, APPROVE/REJECT, evidence }` schema — same structural idea (capture reason at point of departure) adapted to a per-step granularity.

---

## 7. F2 + Wilson CI + Confidence Band

**Three independent gates:**

| Gate | Where | Condition | Failure |
|---|---|---|---|
| F2 lift presence | `build()` step 1 | `lift.is_some()` | `LiftEvidenceMissing` |
| F2 canonical support | `build()` step 2 | `canonical.support >= 20` (or 5 if deviation_relaxed) | `SupportBelowFloor(n)` |
| F2 per-variant support | `build()` step 5 | each `variant.support >= 20` (or 5 if deviation_relaxed) | `DeviationSupportBelowRelaxedFloor(n)` |

**Confidence band computation** (across canonical + variants):

- `min = min(canonical.confidence, variants.map(.confidence).min())`
- `max = max(canonical.confidence, variants.map(.confidence).max())`
- `std_dev = sd(canonical.confidence ++ variants.map(.confidence))`
- `is_narrow = std_dev < 0.05` — indicates convergent evidence (all alternatives agree)

A narrow band tells the human reviewer "all observed variants agree this is high/low confidence"; a wide band tells them "the gradient is informative — alternatives have meaningfully different confidence."

---

## 8. Tests (target 60)

Per [MODULE_MATRIX m23 row](../../MODULE_MATRIX.md): `test_count_min: 60`, kind `property`.

Per [V7 cluster-F plan § m23 test budget](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md):

| Family | Count | Notes |
|---|---:|---|
| F-Unit | 30 | per-arm `build()`, deviation join, confidence band, Wilson CI |
| F-Property | 5 | invariants (see below) |
| F-Fuzz | 0 | not in m23's budget |
| F-Integration | 15 | m20+m21+m22+m14 → m23 → m30-queue; full chain |
| F-Contract | 5 | `WorkflowProposal` serde roundtrip; m30 admission-handler contract |
| F-Regression | 4 | pre-seeded bug classes |
| F-Mutation | ≥75% kill | per G6; F2 gate must be mutation-tight |
| **Total** | **≥60** | |

**Property invariants (proptest):**

1. **F2 always:** every successfully-built `WorkflowProposal` has `evidence.n_samples >= 20` OR (`deviation_relaxed_n == true` AND `evidence.n_samples >= 5`)
2. **Wilson CI bounds:** `0.0 <= ci_lower <= ci_upper <= 1.0` always
3. **MAX_VARIANTS cap:** `variants.len() <= 3` always (post-deviation-join included)
4. **Variant ordering preserved:** `variants` always sorted by `edit_distance` ASC (or by `top_k_rank` ASC)
5. **AwaitingReview at construction:** every proposal from `build()` has `status == AwaitingReview` (Accepted/Rejected can only come from m30 transition)

**Pre-seeded regression bug classes:**

- Mutation of `>=` to `>` in support floor check (must die — off-by-one)
- Mutation of Wilson formula constant z=1.959964 to 1.96 (close but wrong — must die at high-precision tests)
- Mutation that swaps ci_lower / ci_upper (must die)
- Mutation that sets status to Accepted at construction (must die — m23 NEVER auto-promotes)
- Mutation that writes to m30 table directly (catch via test asserting m30 bank table unchanged after `aggregate_and_write`)

**Critical integration test:** `test_m23_never_writes_to_m30` — after `aggregate_and_write` produces N proposals, assert `SELECT COUNT(*) FROM m30_bank` is unchanged from before. This is the structural guarantee that AP-V7-07 is not bypassed even by accident.

---

## 9. Antipatterns Avoided

| Antipattern | Mitigation in m23 |
|---|---|
| **AP-V7-07** (auto-promotion m23 → m30) | Architectural: m23 writes ONLY to `proposals` table with status `AwaitingReview`; the m30 admission handler is a separate module. Integration test asserts m30 bank unchanged after `aggregate_and_write`. |
| **F2** (sample-size inflation) | Construction-time gate: `Result<WorkflowProposal, ProposalError>` API forces caller to handle refusal. No `unwrap()`. No bypass path. |
| **F5** (bank creep) | m23 never writes to m30; explicit `wf-crystallise propose accept <id>` required at m30 admission. |
| **AP-V7-03** (verb collapse) | m23 aggregates and proposes; never recommends, dispatches, or auto-promotes. |
| **AP-V7-08** (self-dispatch via m32) | m23 emits proposals; never dispatches. Proposals naming m32 as a workflow step are rejected at m30's `EscapeSurfaceProfile = SandboxEscape` schema layer (not m23's concern but observed downstream). |
| **AP-Drift-04** (test count over-report) | Every proposal carries `evidence.n_samples`; auditable post-construction. |
| **F10** (random sampling) | Variants are inherited from m21's deterministic top-K; m23 does not re-sample. Deviation-shaped variants are added deterministically (consistent rationale → add; otherwise skip). |
| **F11** (cascade monoculture) | m23 operates on opaque `StepToken(u32)` throughout; never emits human-meaningful labels. m12 resolves display names at report-emit time only. |
| **AP30** (namespace string drift) | All DB writes use `workflow_trace_*` constants; m9 namespace guard enforces. |
| **AP29** (sync HTTP in tokio::spawn) | m23 is synchronous CPU-bound; no HTTP / async surface. |

---

## 10. Substrate / Watcher Class Pre-Position

- **Class C** (refusal) at **every F2 rejection** — refusal IS the correct behaviour per [KEYWORDS_20 § F2](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-F.md). Watcher logs refusal events as healthy operation, not faults.
- **Class A** (activation transition) at first `ProposalBuilder::build()` success post-G9 — the moment a typed proposal exists is the moment the engine has **output**. Highest-leverage observation moment per [GOD_TIER_CONSOLIDATION Part VII](../../../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md).
- **Class I** (Hebbian silence) if m23 emits >=5 proposals per week but human review rate is 0 — indicates the human-review UX is broken downstream, not m23.
- **Class G** (substrate-frame confusion) if proposal naming or descriptions drift toward human-meaningful labels. Watcher pre-positioned to flag at G7.

**Atuin trajectory anchor:** `wt-proposals-list` (proposed; T5.2) — every emitted proposal lands in atuin with `(canonical_id, variant_count, n_samples, ci_lower, ci_upper, deviation_shaped, deviation_relaxed_n, source)`. Trend of `n_samples` distribution = substrate-evidence density growing. Trend of `deviation_shaped` rate = operator behaviour shaping proposals.

---

## 11. Open Questions (Zen G7 audit + G5 spec interview)

1. **Deviation rationale cosine similarity threshold (default 0.8).** Is this the right threshold for grouping bypass rationales into "consistent intent"? Alternative: exact string matching for simplicity. **Recommend 0.8 cosine** for robustness to phrasing variance.
2. **Wilson CI vs Wilson with continuity correction.** Plain Wilson is locked. Continuity correction is more conservative for very small n (5-20) but introduces complexity. Reject continuity correction unless empirical false-positive rate observed > acceptable.
3. **MAX_VARIANTS default = 3.** Locked at 3 per P0 #10. Should this be configurable per IteratorKind (e.g., 5 for Cascade, 3 for Battern)? Recommend fixed 3 engine-wide for human-review cognitive budget consistency.
4. **Cross-type deduplication strategy.** Currently "first source wins." Alternative: highest-confidence source wins (would require pre-computing confidence before dedup). Recommend first-source-wins for simplicity; revisit if proposals end up dominated by one source.
5. **ConfidenceBand `is_narrow` threshold (std_dev < 0.05).** Empirically calibrated? G5 interview to confirm with first 30 days of observation.
6. **Ember 7-trait CI gate on proposal text.** What constitutes "proposal text" — the canonical step sequence (opaque tokens), the cluster_lineage IDs, or rendered display? Current spec: Ember gate applies to display-rendered text only (after m12 label resolution), not the raw proposal payload. G5 to confirm.
7. **ProposalId stability across re-mining cycles.** Currently `uuid::Uuid::new_v4()` (random). Alternative: deterministic UUID from canonical_hash for cross-cycle identity. Random IDs are safer (no accidental cross-cycle conflation); deterministic IDs make audit cleaner. **Recommend random UUID + canonical_hash field for cross-cycle linkage.**

---

## 12. Synergy / Sister-Module Anchors

- **m20** ([m20_prefixspan_miner.md](m20_prefixspan_miner.md)) — primary canonical source (Pure Rust per D-A, Luke S1002127); `Pattern` consumed via composition.
- **m21** ([m21_variant_builder.md](m21_variant_builder.md)) — variant source; `Vec<PatternVariant>` composed into proposal.
- **m22** ([m22_kmeans_feature.md](m22_kmeans_feature.md)) — feature context source; `FeatureCluster.centroid` attached to `Evidence`.
- **m14** (Cluster E `habitat_outcome_lift`) — Wilson-bounded lift evidence; **F2 construction-time gate** depends on `lift.is_some()`.
- **m32** (Cluster G dispatcher) — writes `deviation_events` table when operator bypasses a step; m23 reads back as evidence.
- **m30** (Cluster G workflow_bank) — **m23 NEVER auto-promotes.** Explicit human `wf-crystallise propose accept <id>` is the m23 → m30 boundary. m30 admission handler is the writer, not m23.
- **m10** (Cluster D Ember CI gate) — aspect-IN; Ember 7-trait check on proposal display text (post m12 label resolution).
- **m12** (Cluster C CLI reports) — resolves opaque StepToken IDs to display names at report-emit time only; m23 itself never sees the human-readable form.

---

## 13. Verification Trail

- **Frontmatter complete:** ✓ cluster, layer, binary, feature_gate, gap_owner, test_kinds, verb_class, test_count_min
- **Cluster ownership:** Cluster F · L6 · KEYSTONE
- **Gap ownership:** Gap 1 (composition + deviation-join slice) — ~100 LOC fresh of cluster ~600-700 LOC budget
- **Cross-cluster contracts:** CC-3 (E → F via m14) consumed; **CC-4 (F → G) producer surface** — single Cluster F write surface; CC-5 (H ← G loop) eventual consumer via re-iteration
- **F2 gate referenced:** § 2 (construction-time), § 5 (algorithm step 1-5), § 7 (three independent gates), § 8 (property test #1)
- **AP-V7-07 (no auto-promote) called out:** § 1 (Purpose), § 3 (NEVER writes to m30), § 6 (loop never auto-promotes), § 9 (architectural mitigation + integration test), § 12 (explicit boundary statement) — **5 callouts as required by KEYSTONE depth**
- **Wilson CI locked:** § 2, § 7
- **Deviation-as-evidence (additive, not reductive):** § 6
- **Bidi anchors:** Sister modules (4) · Vault (1) · V7 plan (1) · Cluster spec (1) · Matrix row (1) · GENESIS v1.3 (1) · PATTERNS / GOLD_STANDARDS / ANTIPATTERNS (3) — ✓
- **Word count:** ~2,400 (within KEYSTONE 1,500-2,500 range; m23 is largest in cluster per V7 plan)
- **No `.rs` files authored:** ✓ planning-only, HOLD-v2 respected
- **Rust fenced blocks are spec documentation:** ✓ labelled "planning-spec only" inline

*m23 spec authored 2026-05-17 (S1001982). Cluster F KEYSTONE — single output surface of Cluster F; F2 construction-time gate; deviation-as-evidence loop; NEVER auto-promotes (AP-V7-07). Planning-only per HOLD-v2 + AP24. Pre-G7 Zen audit.*
