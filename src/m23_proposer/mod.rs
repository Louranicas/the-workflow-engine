//! `m23_workflow_proposer` — compose `WorkflowVariant`s with feature
//! context into structured `WorkflowProposal`s for human review.
//!
//! Cluster F · L6 · KEYSTONE downstream of m21 + m22. m23 NEVER
//! auto-promotes; per AP-V7-07 + v1.3 § 2 every proposal exits the
//! engine as a markdown artefact for operator review.

use thiserror::Error;

use crate::m14_lift::{LiftSnapshot, MIN_SAMPLE_SIZE};
use crate::m20_prefixspan::Pattern;
use crate::m21_variant_builder::{MutationKind, WorkflowVariant};
use crate::m32_dispatcher::EscapeSurfaceProfile;

/// **F2 hard gate:** proposals refuse to build when the m14 lift
/// snapshot reports `n < MIN_SAMPLE_SIZE` or `lift.is_none()`.
pub const PROPOSAL_F2_THRESHOLD: usize = MIN_SAMPLE_SIZE;

/// A structured proposal for operator review.
///
/// All fields are **private**. A `WorkflowProposal` cannot exist with
/// `evidence_n` below the F2 floor ([`PROPOSAL_F2_THRESHOLD`]) — the only
/// constructors ([`WorkflowProposal::new`] and [`build_proposal`]) enforce
/// it. This hoists the F2 evidence gate into the type system: a hand-built
/// struct literal can no longer bypass [`build_proposal`]'s check. Read
/// state through the accessors.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowProposal {
    /// Opaque identifier.
    proposal_id: u64,
    /// Source variant.
    variant: WorkflowVariant,
    /// Aggregate evidence at proposal time.
    evidence_n: usize,
    /// m14 composite lift.
    evidence_lift: f64,
    /// Wilson CI half-width.
    evidence_ci_half: f64,
    /// Cluster index assigned by m22 (`None` if feature clustering
    /// skipped).
    diversity_cluster: Option<usize>,
    /// Declared escape-surface profile — W1 wire-bump per Plan v2 v0.2.0
    /// §3 Phase 5 step 2 + DX-W.b (W1 wire-bump) + DX-W.c (SemVer-break).
    /// Consumed by Phase 6 R1 Security real verifier; default at
    /// [`build_proposal`] is [`EscapeSurfaceProfile::Sandboxed`] per
    /// Plan v2 §15 D7 fail-safe. **No `#[serde(default)]`** — v0.1.0
    /// proposals lacking this field fail to deserialise (SemVer-break
    /// at wire level per DX-W.c).
    escape_surface: EscapeSurfaceProfile,
    /// Engine-projected cost — W3 wire-bump per Plan v2 v0.2.0 §3 Phase 5
    /// step 3 + §15 D10 metric `step_count × mutation_weight` per
    /// DX-W3.src = `variant.mutation` (the `MutationKind` enum) →
    /// [`mutation_weight_for`] classifier. Consumed by Phase 6 R2 Cost
    /// real verifier (thresholds against `cost_ceiling`). Stacks the
    /// W1 SemVer-break — v0.1.0 proposals lacking `cost` also fail to
    /// deserialise (no `#[serde(default)]`).
    cost: i64,
    /// Provenance chain of proposal_ids from mining → variant → propose
    /// — A4 SD11 12-field shape per Plan v2 v0.2.0 §3 Phase 5 step 4 +
    /// DX-A4-coupling = Phase 5 (per C-6). Single-element chain at
    /// genesis (= `[proposal_id]`); later re-proposals append. v0.3.0
    /// candidate for Consistency verifier `lineage-chain-only` arm per
    /// DX-R3.
    lineage_chain: Vec<u64>,
    /// RALPH generation index that produced this proposal — A4 SD11
    /// 12-field shape. Default 0 at genesis; m11/RALPH wire bumps in
    /// post-v0.2.0 work per §11 RALPH consent gradient.
    generation_index: u64,
    /// Parent proposal_id if this is a re-proposal / derivation — A4
    /// SD11 12-field shape. `None` for fresh proposals; `Some(parent_id)`
    /// when V1 RefusalToken-typed Amend feedback drives a new proposal.
    parent_proposal_id: Option<u64>,
    /// Wilson p95 upper-bound lift = `evidence_lift + evidence_ci_half`
    /// — A4 SD11 12-field shape; pre-computed so consumers don't
    /// re-derive each access. Consumed by Phase 6 R2 Cost + R3
    /// Consistency verifiers for confidence-band thresholding.
    lift_p95: f64,
}

impl WorkflowProposal {
    /// Construct a `WorkflowProposal`, enforcing the **F2 evidence floor**.
    ///
    /// The F2 invariant — a proposal must carry at least
    /// [`PROPOSAL_F2_THRESHOLD`] evidence samples — is enforced here, so a
    /// `WorkflowProposal` value cannot exist below the floor regardless of
    /// the construction site. [`build_proposal`] routes through this
    /// constructor after its own m14-snapshot checks.
    ///
    /// # Errors
    ///
    /// Returns [`ProposerError::EvidenceBelowThreshold`] if `evidence_n` is
    /// below [`PROPOSAL_F2_THRESHOLD`].
    #[allow(clippy::too_many_arguments)] // 12 args reflects the v0.2.0 W1+W3+A4 SD11 wire shape; a builder pattern is a v0.3.0 candidate (W3+A4 keep the F2 invariant on the constructor)
    pub fn new(
        proposal_id: u64,
        variant: WorkflowVariant,
        evidence_n: usize,
        evidence_lift: f64,
        evidence_ci_half: f64,
        diversity_cluster: Option<usize>,
        escape_surface: EscapeSurfaceProfile,
        cost: i64,
        lineage_chain: Vec<u64>,
        generation_index: u64,
        parent_proposal_id: Option<u64>,
        lift_p95: f64,
    ) -> Result<Self, ProposerError> {
        if evidence_n < PROPOSAL_F2_THRESHOLD {
            return Err(ProposerError::EvidenceBelowThreshold {
                n: evidence_n,
                threshold: PROPOSAL_F2_THRESHOLD,
            });
        }
        Ok(Self {
            proposal_id,
            variant,
            evidence_n,
            evidence_lift,
            evidence_ci_half,
            diversity_cluster,
            escape_surface,
            cost,
            lineage_chain,
            generation_index,
            parent_proposal_id,
            lift_p95,
        })
    }

    /// Opaque proposal identifier.
    #[must_use]
    pub const fn proposal_id(&self) -> u64 {
        self.proposal_id
    }

    /// Borrow the source variant.
    #[must_use]
    pub const fn variant(&self) -> &WorkflowVariant {
        &self.variant
    }

    /// Aggregate evidence sample count at proposal time (always
    /// `>= PROPOSAL_F2_THRESHOLD`).
    #[must_use]
    pub const fn evidence_n(&self) -> usize {
        self.evidence_n
    }

    /// m14 composite lift.
    #[must_use]
    pub const fn evidence_lift(&self) -> f64 {
        self.evidence_lift
    }

    /// Wilson CI half-width.
    #[must_use]
    pub const fn evidence_ci_half(&self) -> f64 {
        self.evidence_ci_half
    }

    /// Cluster index assigned by m22 (`None` if feature clustering skipped).
    #[must_use]
    pub const fn diversity_cluster(&self) -> Option<usize> {
        self.diversity_cluster
    }

    /// Declared escape-surface profile — W1 wire-bump per Plan v2 v0.2.0
    /// §3 Phase 5 step 2 + DX-W.b. Consumed by Phase 6 R1 Security real
    /// verifier; default at [`build_proposal`] is
    /// [`EscapeSurfaceProfile::Sandboxed`] per Plan v2 §15 D7 fail-safe.
    #[must_use]
    pub const fn escape_surface(&self) -> EscapeSurfaceProfile {
        self.escape_surface
    }

    /// Engine-projected cost — W3 wire-bump per Plan v2 v0.2.0 §3 Phase 5
    /// step 3 + §15 D10 metric `step_count × mutation_weight`. Consumed
    /// by Phase 6 R2 Cost real verifier.
    #[must_use]
    pub const fn cost(&self) -> i64 {
        self.cost
    }

    /// Provenance chain of proposal_ids — A4 SD11 12-field shape per
    /// Plan v2 v0.2.0 §3 Phase 5 step 4. Single-element at genesis.
    #[must_use]
    pub fn lineage_chain(&self) -> &[u64] {
        &self.lineage_chain
    }

    /// RALPH generation index — A4 SD11 12-field shape.
    #[must_use]
    pub const fn generation_index(&self) -> u64 {
        self.generation_index
    }

    /// Parent proposal_id if a re-proposal — A4 SD11 12-field shape.
    #[must_use]
    pub const fn parent_proposal_id(&self) -> Option<u64> {
        self.parent_proposal_id
    }

    /// Wilson p95 upper-bound lift = `evidence_lift + evidence_ci_half`
    /// — A4 SD11 12-field shape.
    #[must_use]
    pub const fn lift_p95(&self) -> f64 {
        self.lift_p95
    }
}

/// W3 mutation-weight classifier — Plan v2 v0.2.0 §3 Phase 5 step 3 +
/// §15 D10 metric `step_count × mutation_weight` per DX-W3.src
/// (`variant.mutation` is the `MutationKind` enum at
/// `src/m21_variant_builder/mod.rs:47`; weight is derived here).
///
/// Cost-frame interpretation (NOT safety-frame — see
/// [`mutation_safety_weight`] for the inverse): more-aggressive
/// mutations cost more engine effort / risk per dispatch. The
/// classifier is the v0.2.0 source-of-truth; v0.3.0 may upgrade to a
/// per-StepToken table (DX-W3.src non-default arm).
///
/// Weights:
/// - `Identity` = 1 (baseline; verbatim copy of the source pattern)
/// - `Swap` = 2 (mild mutation; two adjacent steps transposed)
/// - `Skip` = 4 (aggressive mutation; one step omitted)
///
/// Returns `u32` so the multiplication `step_count * weight` cannot
/// overflow `i64` for any realistic input (`MAX_STEPS_PER_VARIANT ≈
/// 32` × `u32::MAX` < `i64::MAX`).
#[must_use]
pub const fn mutation_weight_for(kind: &MutationKind) -> u32 {
    match kind {
        MutationKind::Identity => 1,
        MutationKind::Swap { .. } => 2,
        MutationKind::Skip { .. } => 4,
    }
}

/// W3 cost projection — `step_count × mutation_weight_for(variant.mutation)`
/// saturating to [`i64::MAX`] on overflow (impossible for realistic
/// `WorkflowVariant`s but defensive).
///
/// Consumed by [`build_proposal_with_escape_surface`] to populate the
/// proposal's `cost` field for Phase 6 R2 Cost verifier consumption.
#[must_use]
fn project_cost(variant: &WorkflowVariant) -> i64 {
    let weight = mutation_weight_for(&variant.mutation);
    let step_count: u64 = variant.steps.len() as u64;
    let product: u128 = u128::from(step_count) * u128::from(weight);
    i64::try_from(product).unwrap_or(i64::MAX)
}

/// Proposal-builder errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ProposerError {
    /// m14 lift snapshot indicates insufficient evidence.
    #[error("F2 gate: evidence n={n} < {threshold}")]
    EvidenceBelowThreshold {
        /// Observed n.
        n: usize,
        /// Required threshold.
        threshold: usize,
    },
    /// m14 lift snapshot returned `None` for either `lift` or `ci_half`.
    #[error("F2 gate: lift / ci_half is None")]
    LiftUnavailable,
}

/// Build a proposal from a variant + evidence snapshot.
///
/// **F2 hard refusal:** returns
/// [`ProposerError::EvidenceBelowThreshold`] if `snapshot.n <
/// PROPOSAL_F2_THRESHOLD`, and [`ProposerError::LiftUnavailable`] if
/// either `snapshot.lift` or `snapshot.ci_half` is `None`.
///
/// # Errors
///
/// See above.
pub fn build_proposal(
    variant: WorkflowVariant,
    snapshot: &LiftSnapshot,
    diversity_cluster: Option<usize>,
) -> Result<WorkflowProposal, ProposerError> {
    // W1 wire-bump per Plan v2 §15 DX-W.b + DX-W.c: the 3-arg shape
    // defaults `escape_surface` to `Sandboxed` per Plan v2 §15 D7
    // (most-restrictive, fail-safe). Callers that need to declare a
    // non-default surface use [`build_proposal_with_escape_surface`].
    build_proposal_with_escape_surface(
        variant,
        snapshot,
        diversity_cluster,
        EscapeSurfaceProfile::Sandboxed,
    )
}

/// Construct a `WorkflowProposal` with an explicit `escape_surface`
/// declaration — W1 wire-bump 4-arg variant per Plan v2 v0.2.0 §3 Phase
/// 5 step 2 + DX-W.b (W1) + DX-W.c (SemVer-break).
///
/// Identical semantics to [`build_proposal`] (F2 evidence-floor enforcement
/// at the same site) except the caller declares the escape-surface
/// profile that Phase 6 R1 Security verifier will check against the
/// dispatcher's ack-ceiling per Plan v2 §15 D5 + D6.
///
/// # Errors
///
/// See [`build_proposal`].
pub fn build_proposal_with_escape_surface(
    variant: WorkflowVariant,
    snapshot: &LiftSnapshot,
    diversity_cluster: Option<usize>,
    escape_surface: EscapeSurfaceProfile,
) -> Result<WorkflowProposal, ProposerError> {
    if snapshot.n < PROPOSAL_F2_THRESHOLD {
        return Err(ProposerError::EvidenceBelowThreshold {
            n: snapshot.n,
            threshold: PROPOSAL_F2_THRESHOLD,
        });
    }
    let lift = snapshot.lift.ok_or(ProposerError::LiftUnavailable)?;
    let ci_half = snapshot
        .ci_half
        .ok_or(ProposerError::LiftUnavailable)?;
    let proposal_id = crate::m4_cascade::cluster_id::fnv1a_64(
        format!("proposal:{}:{}", variant.variant_id, snapshot.n).as_bytes(),
    );
    // W3 cost projection — computed before `variant` is moved into the
    // constructor. Phase 6 R2 Cost verifier reads `proposal.cost()` and
    // thresholds against `cost_ceiling`. The current classifier is the
    // v0.2.0 source-of-truth per DX-W3.src.
    let cost = project_cost(&variant);
    // A4 SD11 12-field defaults at genesis (Plan v2 v0.2.0 §3 Phase 5
    // step 4 + DX-A4-coupling): single-element lineage_chain (this
    // proposal_id only); generation_index 0 (m11/RALPH bumps post-v0.2.0
    // per §11 RALPH consent gradient); no parent (fresh proposal);
    // lift_p95 = evidence_lift + evidence_ci_half (Wilson upper-bound).
    let lineage_chain = vec![proposal_id];
    let generation_index = 0_u64;
    let parent_proposal_id: Option<u64> = None;
    let lift_p95 = lift + ci_half;
    // Route through the F2-enforcing constructor. The `snapshot.n <
    // PROPOSAL_F2_THRESHOLD` check above already guarantees this succeeds;
    // `WorkflowProposal::new` re-checks the same floor so the invariant
    // holds for every construction site, not just this one.
    WorkflowProposal::new(
        proposal_id,
        variant,
        snapshot.n,
        lift,
        ci_half,
        diversity_cluster,
        escape_surface,
        cost,
        lineage_chain,
        generation_index,
        parent_proposal_id,
        lift_p95,
    )
}

/// Compose variants from a top-N pattern slice into a proposal batch.
///
/// Skips patterns/variants whose evidence fails the F2 gate. The
/// returned vec preserves source ordering.
///
/// **CC-4 diversity threading:** the `diversity_of` closure maps each
/// [`WorkflowVariant`] to its m22 K-means cluster index (or `None` when
/// feature clustering was skipped). Whatever the closure returns is
/// threaded verbatim into the resulting proposal's
/// [`WorkflowProposal::diversity_cluster`]. This mirrors how
/// [`crate::m31_selector::select_top_k`] takes its `diversity_score`
/// closure. Prior to this wiring `compose_proposals` hard-coded
/// `diversity_cluster: None` for every variant — the m22 K-means signal
/// never reached a proposal on the batch path (the only production path).
/// That was the CC-4 wiring gap, and supplying the closure here fixes it.
///
/// **Silent-swallow rationale (AP-V7-13 audit):** the inner `match`
/// branches deliberately discard two error classes:
///
/// 1. [`crate::m21_variant_builder::VariantBuilderError::EmptyPattern`] —
///    only fires when `pattern.steps()` is empty, which m20's
///    `mine_sequences` cannot produce (every emitted `Pattern` carries
///    `support >= MIN_SUPPORT_FLOOR` which requires ≥1 step). The m21
///    refusal arm is therefore **unreachable in production** under the m20
///    input contract. It is kept as a defensive guard rather than removed
///    so that a future m20 contract regression surfaces loudly: a
///    `debug_assert!` fires in debug/test builds, and release builds
///    skip-and-trace rather than panic (F9 fix — was a silent `continue`).
/// 2. [`ProposerError::EvidenceBelowThreshold`] and
///    [`ProposerError::LiftUnavailable`] — these ARE the F2 gate firing.
///    `compose_proposals` is the batched form whose documented behaviour
///    is "skips below F2"; trace-emit-and-drop is the contract.
///
/// Callers that need typed refusal MUST use [`build_proposal`] directly.
#[must_use]
pub fn compose_proposals(
    patterns: &[Pattern],
    snapshot: &LiftSnapshot,
    diversity_of: impl Fn(&WorkflowVariant) -> Option<usize>,
) -> Vec<WorkflowProposal> {
    // Capacity hint: at most MAX_VARIANTS_PER_PATTERN proposals per pattern.
    let mut out: Vec<WorkflowProposal> = Vec::with_capacity(
        patterns
            .len()
            .saturating_mul(crate::m21_variant_builder::MAX_VARIANTS_PER_PATTERN),
    );
    for p in patterns {
        // F9 — the m21 refusal arm is a defensive guard, not live error
        // handling: m20's mine_sequences never emits an empty-step Pattern
        // (MIN_SUPPORT_FLOOR forces ≥1 step). A `debug_assert!` makes the
        // impossibility explicit and turns any future m20 contract
        // regression into a loud test/debug failure; release builds still
        // skip-and-trace to stay panic-free.
        let variants = match crate::m21_variant_builder::build_variants(p) {
            Ok(variants) => variants,
            Err(e) => {
                debug_assert!(
                    false,
                    "m23::compose_proposals — m21 build_variants refused ({e}); \
                     m20 contract guarantees non-empty patterns, so this arm \
                     is unreachable unless the m20 input contract regressed"
                );
                tracing::debug!(
                    pattern_hash = p.canonical_hash(),
                    error = %e,
                    "m23::compose_proposals — m21 build_variants refused; m20 contract violation upstream"
                );
                continue;
            }
        };
        for v in variants {
            // CC-4: capture the m22 K-means cluster for this variant
            // BEFORE `build_proposal` consumes `v` by value, then thread
            // it into the proposal.
            let diversity_cluster = diversity_of(&v);
            // rationale: F2 gate skip-and-trace is the documented batched
            // behaviour for compose_proposals; build_proposal is the
            // strict typed-refusal path.
            match build_proposal(v, snapshot, diversity_cluster) {
                Ok(proposal) => out.push(proposal),
                Err(e) => {
                    tracing::debug!(
                        pattern_hash = p.canonical_hash(),
                        error = %e,
                        "m23::compose_proposals — F2 gate skip"
                    );
                }
            }
        }
    }
    out
}

/// Maximum additive contribution the m15 pressure scalar may add to a
/// proposal's composition priority.
///
/// **CC-7 wiring — bounded by construction.** The pressure scalar is in
/// `[0.0, 1.0]` (see [`crate::m15_pressure::pressure_scalar_from_count`])
/// and is multiplied by `MAX_PRESSURE_CONTRIBUTION` before being added to a
/// proposal's safety-weighted priority. The contribution ceiling here
/// caps how much the substrate's voice can shift composition order: even
/// at saturated pressure, a proposal cannot gain more than this constant
/// over its baseline lift. A small finite cap was chosen deliberately —
/// pressure is a signal, not an amplifier (D22 "additive, bounded").
pub const MAX_PRESSURE_CONTRIBUTION: f64 = 0.5;

/// Per-mutation safety weight: under pressure, less-mutated variants
/// (Identity) get the full bonus; more-aggressive mutations (Skip) get
/// less. The contribution is the *product* of this weight and the
/// clamped pressure scalar, multiplied by [`MAX_PRESSURE_CONTRIBUTION`].
///
/// Rationale: when the substrate is under unresolved pressure, the engine
/// should preferentially surface SAFER proposals first — the Identity
/// variant of a mined pattern is the closest to "do what you already do",
/// and Swap / Skip mutations carry progressively more behavioural drift.
/// All weights are in `[0.0, 1.0]`; under zero pressure every weight
/// multiplies through to zero contribution, so the original compose order
/// is preserved exactly.
#[must_use]
const fn mutation_safety_weight(mutation: &MutationKind) -> f64 {
    match mutation {
        MutationKind::Identity => 1.0,
        MutationKind::Swap { .. } => 0.5,
        MutationKind::Skip { .. } => 0.25,
    }
}

/// Compute the bounded additive pressure contribution for a single proposal.
///
/// `pressure` is clamped to `[0.0, 1.0]` (silently — out-of-band callers
/// get the same safe behaviour as in-band callers); the result is in
/// `[0.0, MAX_PRESSURE_CONTRIBUTION]`. NaN-free by construction (clamp
/// rejects NaN to `0.0`).
#[must_use]
fn pressure_priority_bonus(pressure: f64, proposal: &WorkflowProposal) -> f64 {
    // NaN-safe clamp: any non-finite or out-of-band pressure collapses
    // to zero contribution, never amplifies.
    let clamped = if pressure.is_finite() {
        pressure.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let weight = mutation_safety_weight(&proposal.variant().mutation);
    MAX_PRESSURE_CONTRIBUTION * clamped * weight
}

/// Compose proposals with bounded additive pressure modulation
/// (Phase 7 / CC-7 wire — D22 "Pressure modulates m23 compose-priority
/// (additive, bounded)").
///
/// Behaves identically to [`compose_proposals`] when `pressure == 0.0`
/// (and any other zero-bonus condition — non-finite pressure, all-zero
/// safety weights). When `pressure > 0.0`, the returned vector is
/// **stable-sorted** by descending priority, where priority is
///
/// ```text
///     priority(p) = p.evidence_lift() + pressure_priority_bonus(pressure, p)
/// ```
///
/// and `pressure_priority_bonus(pressure, p)` is in
/// `[0.0, MAX_PRESSURE_CONTRIBUTION]` — bounded by construction (see the
/// constant docstring). Stable sort means that proposals with equal
/// priority retain their original (source-pattern) order, so the
/// no-pressure path is provably identical to [`compose_proposals`].
///
/// **NA framing (D24).** Under elevated pressure, the substrate's
/// outstanding `PHASE-B-RESERVATION-NOTICE` ledger is the signal — the
/// composition path responds by surfacing SAFER variants first (Identity
/// then Swap then Skip per [`mutation_safety_weight`]). Pressure is the
/// substrate's voice in composition; it does not block, it re-orders.
///
/// `pressure` is a scalar in `[0.0, 1.0]`; values outside that band (or
/// `NaN` / infinities) collapse to zero contribution — pressure can never
/// amplify, only nudge.
#[must_use]
pub fn compose_proposals_with_pressure(
    patterns: &[Pattern],
    snapshot: &LiftSnapshot,
    diversity_of: impl Fn(&WorkflowVariant) -> Option<usize>,
    pressure: f64,
) -> Vec<WorkflowProposal> {
    let mut proposals = compose_proposals(patterns, snapshot, diversity_of);
    // Fast path: zero or non-finite pressure ⇒ no reorder (no allocation,
    // no sort traversal), preserving compose_proposals's source ordering.
    if !pressure.is_finite() || pressure <= 0.0 {
        return proposals;
    }
    // Stable sort by descending priority. Stable sort preserves the
    // original index order for proposals with identical priority — this
    // is the "no-pressure ⇒ identical output" invariant under partial
    // ties.
    proposals.sort_by(|a, b| {
        let pa = a.evidence_lift() + pressure_priority_bonus(pressure, a);
        let pb = b.evidence_lift() + pressure_priority_bonus(pressure, b);
        // Reverse for descending; NaN-safe via total_cmp.
        pb.total_cmp(&pa)
    });
    tracing::debug!(
        target: "m23.compose",
        pressure,
        n_proposals = proposals.len(),
        "m23::compose_proposals_with_pressure — pressure-aware reorder applied"
    );
    proposals
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::{
        build_proposal, build_proposal_with_escape_surface, compose_proposals, ProposerError,
        WorkflowProposal, PROPOSAL_F2_THRESHOLD,
    };
    use crate::m14_lift::LiftSnapshot;
    use crate::m20_prefixspan::{Pattern, StepToken};
    use crate::m21_variant_builder::{build_variants, MutationKind, WorkflowVariant};
    use crate::m32_dispatcher::EscapeSurfaceProfile;

    fn snap(n: usize, lift: Option<f64>, ci: Option<f64>) -> LiftSnapshot {
        LiftSnapshot {
            lift,
            ci_half: ci,
            n,
            latest_ts_ms: 0,
            computed_at: SystemTime::now(),
        }
    }

    fn sample_pattern() -> Pattern {
        Pattern::new(vec![StepToken(1), StepToken(2)], 10, (0, 1))
    }

    fn sample_variant() -> WorkflowVariant {
        build_variants(&sample_pattern()).expect("v")[0].clone()
    }

    #[test]
    fn proposal_refuses_below_n_threshold() {
        let s = snap(10, Some(0.5), Some(0.1));
        assert!(matches!(
            build_proposal(sample_variant(), &s, None),
            Err(ProposerError::EvidenceBelowThreshold { n: 10, .. })
        ));
    }

    #[test]
    fn proposal_refuses_when_lift_none() {
        let s = snap(30, None, Some(0.1));
        assert!(matches!(
            build_proposal(sample_variant(), &s, None),
            Err(ProposerError::LiftUnavailable)
        ));
    }

    #[test]
    fn proposal_refuses_when_ci_none() {
        let s = snap(30, Some(0.5), None);
        assert!(matches!(
            build_proposal(sample_variant(), &s, None),
            Err(ProposerError::LiftUnavailable)
        ));
    }

    #[test]
    fn proposal_built_when_evidence_sufficient() {
        let s = snap(30, Some(0.6), Some(0.05));
        let p = build_proposal(sample_variant(), &s, Some(0)).expect("ok");
        assert_eq!(p.evidence_n(), 30);
        assert!((p.evidence_lift() - 0.6).abs() < 1e-12);
        assert_eq!(p.diversity_cluster(), Some(0));
    }

    #[test]
    fn proposal_id_deterministic() {
        let s = snap(30, Some(0.6), Some(0.05));
        let p1 = build_proposal(sample_variant(), &s, None).expect("p1");
        let p2 = build_proposal(sample_variant(), &s, None).expect("p2");
        assert_eq!(p1.proposal_id(), p2.proposal_id());
    }

    #[test]
    fn compose_proposals_skips_insufficient_evidence() {
        let s = snap(10, Some(0.5), Some(0.05));
        let patterns = vec![sample_pattern()];
        let p = compose_proposals(&patterns, &s, |_| None);
        assert!(p.is_empty());
    }

    #[test]
    fn compose_proposals_yields_variants_under_sufficient_evidence() {
        let s = snap(30, Some(0.5), Some(0.05));
        let patterns = vec![sample_pattern()];
        let proposals = compose_proposals(&patterns, &s, |_| None);
        assert!(!proposals.is_empty());
        // First proposal should be derived from the identity variant.
        assert!(matches!(
            proposals[0].variant().mutation,
            MutationKind::Identity
        ));
    }

    #[test]
    fn f2_threshold_equals_min_sample_size() {
        assert_eq!(PROPOSAL_F2_THRESHOLD, crate::m14_lift::MIN_SAMPLE_SIZE);
    }

    // ---- Cluster F hardening pass — additional 10+ tests ----

    #[test]
    // rationale: Boundary — exactly at PROPOSAL_F2_THRESHOLD must be accepted.
    fn boundary_n_at_threshold_accepted() {
        let s = snap(PROPOSAL_F2_THRESHOLD, Some(0.5), Some(0.05));
        let r = build_proposal(sample_variant(), &s, None);
        assert!(r.is_ok());
    }

    #[test]
    // rationale: Boundary — one below PROPOSAL_F2_THRESHOLD must refuse.
    fn boundary_n_one_below_threshold_refused() {
        let s = snap(PROPOSAL_F2_THRESHOLD - 1, Some(0.5), Some(0.05));
        let r = build_proposal(sample_variant(), &s, None);
        assert!(matches!(r, Err(ProposerError::EvidenceBelowThreshold { .. })));
    }

    #[test]
    // rationale: Anti-property — AP-V7-07 m23 NEVER auto-promotes. There is
    // NO public function on m23 that writes to m30 bank or any external
    // store. We verify the m23 public surface contains no `promote`,
    // `commit`, `accept`, or `bank` symbol.
    fn anti_property_ap_v7_07_no_auto_promote_in_public_surface() {
        // The module compiles iff these are the only construction entry
        // points — if a future contributor adds `pub fn promote_proposal`
        // this test still compiles, so we instead inspect the public
        // proposal payload at runtime: there is no Bank/Selector/Dispatcher
        // field reachable from a proposal struct.
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        // Proposal type carries no field named `promoted`, `committed`,
        // `accepted_at`, or anything bank-related.
        let json = serde_json::to_string(&p).expect("ser");
        for forbidden in ["promoted", "committed", "accepted_at", "bank", "promote_to"] {
            assert!(
                !json.contains(forbidden),
                "AP-V7-07 violation: m23 proposal serde contains '{forbidden}': {json}"
            );
        }
    }

    #[test]
    // rationale: Anti-property — F11 cascade-monoculture: proposal_id is u64,
    // and serde JSON contains no human-readable substring.
    fn anti_property_f11_proposal_id_is_pure_u64() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        // proposal_id encodes as JSON number; no string content possible.
        let id_json = serde_json::to_string(&p.proposal_id()).expect("ser");
        assert!(id_json.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    // rationale: Determinism — build_proposal is deterministic given
    // identical inputs. Run 5 times, all proposal_id values match.
    fn determinism_proposal_id_stable_across_invocations() {
        let s = snap(30, Some(0.5), Some(0.05));
        let mut ids = Vec::new();
        for _ in 0..5_u32 {
            let p = build_proposal(sample_variant(), &s, None).expect("ok");
            ids.push(p.proposal_id());
        }
        for w in ids.windows(2) {
            assert_eq!(w[0], w[1]);
        }
    }

    #[test]
    // rationale: Cross-module — compose_proposals consumes m20 Pattern +
    // m14 LiftSnapshot + m21 build_variants — exercise the full chain.
    fn cross_module_full_pipeline_yields_proposals() {
        let s = snap(30, Some(0.7), Some(0.05));
        let patterns = vec![
            Pattern::new(vec![StepToken(1), StepToken(2), StepToken(3)], 25, (0, 1)),
            Pattern::new(vec![StepToken(4), StepToken(5)], 22, (0, 0)),
        ];
        let p = compose_proposals(&patterns, &s, |_| None);
        assert!(!p.is_empty());
        // Every proposal must carry sufficient evidence (compose skips F2).
        for prop in &p {
            assert!(prop.evidence_n() >= PROPOSAL_F2_THRESHOLD);
        }
    }

    #[test]
    // rationale: Adversarial — lift = 0.0 must still pass (F2 is on n,
    // not on lift magnitude; the proposer is evidence-gated not
    // lift-gated by design).
    fn adversarial_zero_lift_accepted_at_sufficient_n() {
        let s = snap(50, Some(0.0), Some(0.01));
        let r = build_proposal(sample_variant(), &s, None);
        assert!(r.is_ok());
    }

    #[test]
    // rationale: Adversarial — negative lift (regression detected) must
    // still pass; lift sign is information, not a gate.
    fn adversarial_negative_lift_accepted() {
        let s = snap(50, Some(-0.3), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert!(p.evidence_lift() < 0.0);
    }

    #[test]
    // rationale: Boundary — diversity_cluster=None must be threaded through.
    fn boundary_none_diversity_cluster_preserved() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert_eq!(p.diversity_cluster(), None);
    }

    #[test]
    // rationale: Boundary — large diversity_cluster value preserved.
    fn boundary_large_diversity_cluster_preserved() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, Some(usize::MAX)).expect("ok");
        assert_eq!(p.diversity_cluster(), Some(usize::MAX));
    }

    #[test]
    // rationale: Contract regression — ProposerError variants stable.
    fn contract_proposer_error_variants_stable() {
        let below = ProposerError::EvidenceBelowThreshold { n: 5, threshold: 20 };
        let lift = ProposerError::LiftUnavailable;
        assert!(!format!("{below}").is_empty());
        assert!(!format!("{lift}").is_empty());
    }

    #[test]
    // rationale: Resource accounting — compose_proposals pre-allocates and
    // returns empty for empty input without allocating beyond hint.
    fn resource_accounting_empty_input_returns_empty() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = compose_proposals(&[], &s, |_| None);
        assert!(p.is_empty());
    }

    #[test]
    // rationale: Cross-module — proposal serde roundtrip preserves all
    // evidence-bearing fields (downstream m30 bank reads this).
    fn cross_module_proposal_serde_roundtrip() {
        let s = snap(30, Some(0.6), Some(0.05));
        let p = build_proposal(sample_variant(), &s, Some(2)).expect("ok");
        let ser = serde_json::to_string(&p).expect("ser");
        let back: super::WorkflowProposal = serde_json::from_str(&ser).expect("de");
        assert_eq!(back, p);
    }

    #[test]
    // rationale: Determinism — compose_proposals yields stable ordering
    // (proposer relies on m20's sort + m21's emission order).
    fn determinism_compose_proposals_output_stable() {
        let s = snap(30, Some(0.5), Some(0.05));
        let patterns = vec![Pattern::new(vec![StepToken(1), StepToken(2)], 25, (0, 0))];
        let a = compose_proposals(&patterns, &s, |_| None);
        let b = compose_proposals(&patterns, &s, |_| None);
        assert_eq!(a.len(), b.len());
        for (pa, pb) in a.iter().zip(b.iter()) {
            assert_eq!(pa.proposal_id(), pb.proposal_id());
            assert_eq!(pa.variant().variant_id, pb.variant().variant_id);
        }
    }

    // ---- Wave: god-tier hardening pass — m23 to ≥50 tests ----

    #[test]
    // rationale: Error variant — exact n payload carried by
    // EvidenceBelowThreshold must equal snapshot.n, not the threshold.
    fn error_evidence_below_threshold_carries_observed_n() {
        let s = snap(7, Some(0.5), Some(0.1));
        match build_proposal(sample_variant(), &s, None) {
            Err(ProposerError::EvidenceBelowThreshold { n, threshold }) => {
                assert_eq!(n, 7, "observed n must be the snapshot's n");
                assert_eq!(threshold, PROPOSAL_F2_THRESHOLD);
            }
            other => panic!("expected EvidenceBelowThreshold, got {other:?}"),
        }
    }

    #[test]
    // rationale: Error path — n=0 is the most degenerate sub-threshold case.
    fn error_zero_n_refused_with_below_threshold() {
        let s = snap(0, Some(0.9), Some(0.01));
        assert!(matches!(
            build_proposal(sample_variant(), &s, None),
            Err(ProposerError::EvidenceBelowThreshold { n: 0, .. })
        ));
    }

    #[test]
    // rationale: Error precedence — when BOTH n is below threshold AND lift
    // is None, the n-gate must fire FIRST (it is checked before the lift
    // ok_or in source order). A LiftUnavailable here would be a bug.
    fn error_n_gate_precedes_lift_gate() {
        let s = snap(3, None, None);
        assert!(
            matches!(
                build_proposal(sample_variant(), &s, None),
                Err(ProposerError::EvidenceBelowThreshold { .. })
            ),
            "n-gate must be evaluated before the lift-None gate"
        );
    }

    #[test]
    // rationale: Error path — sufficient n but BOTH lift and ci None: the
    // first ok_or (lift) fires, so LiftUnavailable is returned.
    fn error_both_lift_and_ci_none_yields_lift_unavailable() {
        let s = snap(40, None, None);
        assert!(matches!(
            build_proposal(sample_variant(), &s, None),
            Err(ProposerError::LiftUnavailable)
        ));
    }

    #[test]
    // rationale: Boundary — n one ABOVE threshold accepted (upper side of
    // the gate boundary, complements the at/below tests).
    fn boundary_n_one_above_threshold_accepted() {
        let s = snap(PROPOSAL_F2_THRESHOLD + 1, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert_eq!(p.evidence_n(), PROPOSAL_F2_THRESHOLD + 1);
    }

    #[test]
    // rationale: Boundary — usize::MAX evidence_n threaded through without
    // overflow in the proposal payload.
    fn boundary_max_usize_n_threaded_through() {
        let s = snap(usize::MAX, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert_eq!(p.evidence_n(), usize::MAX);
    }

    #[test]
    // rationale: Field fidelity — evidence_lift and evidence_ci_half must be
    // copied verbatim from the snapshot (no rescaling / rounding).
    fn field_fidelity_lift_and_ci_copied_verbatim() {
        let s = snap(30, Some(0.123_456_789), Some(0.009_876_543));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert!((p.evidence_lift() - 0.123_456_789).abs() < 1e-15);
        assert!((p.evidence_ci_half() - 0.009_876_543).abs() < 1e-15);
    }

    #[test]
    // rationale: Determinism — proposal_id depends on (variant_id, n);
    // changing n MUST change the id (n is folded into the FNV input).
    fn determinism_proposal_id_changes_with_n() {
        let s_a = snap(30, Some(0.5), Some(0.05));
        let s_b = snap(31, Some(0.5), Some(0.05));
        let id_a = build_proposal(sample_variant(), &s_a, None).expect("a").proposal_id();
        let id_b = build_proposal(sample_variant(), &s_b, None).expect("b").proposal_id();
        assert_ne!(id_a, id_b, "proposal_id must fold n into its hash");
    }

    #[test]
    // rationale: Determinism — distinct variants (distinct variant_id) must
    // produce distinct proposal_ids at the same evidence n.
    fn determinism_proposal_id_changes_with_variant() {
        let s = snap(30, Some(0.5), Some(0.05));
        let pattern = Pattern::new(vec![StepToken(1), StepToken(2), StepToken(3)], 25, (0, 0));
        let variants = build_variants(&pattern).expect("v");
        assert!(variants.len() >= 2);
        let id0 = build_proposal(variants[0].clone(), &s, None).expect("0").proposal_id();
        let id1 = build_proposal(variants[1].clone(), &s, None).expect("1").proposal_id();
        assert_ne!(id0, id1, "distinct variants must yield distinct proposal_ids");
    }

    #[test]
    // rationale: Cross-module — the variant is moved into the proposal
    // unchanged; variant_id and steps survive the build_proposal call.
    fn cross_module_variant_preserved_in_proposal() {
        let s = snap(30, Some(0.5), Some(0.05));
        let v = sample_variant();
        let v_id = v.variant_id;
        let v_steps = v.steps.clone();
        let p = build_proposal(v, &s, None).expect("ok");
        assert_eq!(p.variant().variant_id, v_id);
        assert_eq!(p.variant().steps, v_steps);
    }

    #[test]
    // rationale: compose_proposals — every emitted pattern produces the
    // identity variant first, so the FIRST proposal per single-pattern
    // input must be Identity.
    fn compose_first_proposal_per_pattern_is_identity() {
        let s = snap(30, Some(0.5), Some(0.05));
        let patterns = vec![Pattern::new(vec![StepToken(9), StepToken(8)], 25, (0, 0))];
        let out = compose_proposals(&patterns, &s, |_| None);
        assert!(!out.is_empty());
        assert!(matches!(out[0].variant().mutation, MutationKind::Identity));
    }

    #[test]
    // rationale: CC-4 wiring — compose_proposals threads the caller-supplied
    // m22 diversity cluster into every proposal on the batched path. This
    // replaces the former `compose_proposals_diversity_cluster_always_none`
    // test, which PINNED the CC-4 wiring gap (hard-coded `None`) as expected
    // behaviour. The closure here assigns a distinct cluster per variant
    // (keyed off the variant's mutation kind) and the proposals MUST carry
    // exactly those clusters.
    fn compose_proposals_threads_diversity_cluster() {
        let s = snap(30, Some(0.5), Some(0.05));
        // Multi-step pattern so m21 emits the full Identity/Swap/Skip set.
        let patterns =
            vec![Pattern::new(vec![StepToken(1), StepToken(2), StepToken(3)], 25, (0, 0))];
        // Cluster assignment by mutation kind: Identity → 0, Swap → 1,
        // Skip → 2. Mirrors how m22 K-means would partition the variants.
        let cluster_of = |v: &WorkflowVariant| -> Option<usize> {
            Some(match v.mutation {
                MutationKind::Identity => 0,
                MutationKind::Swap { .. } => 1,
                MutationKind::Skip { .. } => 2,
            })
        };
        let out = compose_proposals(&patterns, &s, cluster_of);
        assert!(!out.is_empty());
        // Every proposal carries the cluster the closure assigned to its
        // source variant — proof the m22 signal reaches the batch path.
        for p in &out {
            let expected = match p.variant().mutation {
                MutationKind::Identity => Some(0),
                MutationKind::Swap { .. } => Some(1),
                MutationKind::Skip { .. } => Some(2),
            };
            assert_eq!(
                p.diversity_cluster(),
                expected,
                "compose_proposals must thread the closure's cluster verbatim"
            );
        }
        // The Identity variant is always emitted, so cluster 0 must appear.
        assert!(out.iter().any(|p| p.diversity_cluster() == Some(0)));
    }

    #[test]
    // rationale: compose_proposals — passing the `|_| None` closure (the
    // legitimate "m22 clustering skipped" case) yields proposals whose
    // diversity_cluster is None. Complements
    // `compose_proposals_threads_diversity_cluster`.
    fn compose_proposals_none_closure_yields_none_clusters() {
        let s = snap(30, Some(0.5), Some(0.05));
        let patterns = vec![Pattern::new(vec![StepToken(1), StepToken(2)], 25, (0, 0))];
        let out = compose_proposals(&patterns, &s, |_| None);
        assert!(!out.is_empty());
        assert!(out.iter().all(|p| p.diversity_cluster().is_none()));
    }

    #[test]
    // rationale: compose_proposals — a multi-step pattern expands into the
    // full m21 variant set; proposal count must match build_variants len.
    fn compose_proposal_count_matches_variant_expansion() {
        let s = snap(30, Some(0.5), Some(0.05));
        let pattern = Pattern::new(vec![StepToken(1), StepToken(2), StepToken(3)], 25, (0, 0));
        let n_variants = build_variants(&pattern).expect("v").len();
        let out = compose_proposals(&[pattern], &s, |_| None);
        assert_eq!(out.len(), n_variants);
    }

    #[test]
    // rationale: compose_proposals — sub-threshold evidence drops ALL
    // proposals for a multi-pattern batch, not just some.
    fn compose_drops_entire_batch_below_threshold() {
        let s = snap(PROPOSAL_F2_THRESHOLD - 1, Some(0.5), Some(0.05));
        let patterns = vec![
            Pattern::new(vec![StepToken(1), StepToken(2)], 25, (0, 0)),
            Pattern::new(vec![StepToken(3), StepToken(4)], 22, (0, 0)),
            Pattern::new(vec![StepToken(5), StepToken(6)], 30, (0, 0)),
        ];
        assert!(compose_proposals(&patterns, &s, |_| None).is_empty());
    }

    #[test]
    // rationale: compose_proposals — lift-None snapshot drops the whole
    // batch even at sufficient n (F2 gate also covers lift availability).
    fn compose_drops_batch_when_lift_none() {
        let s = snap(40, None, Some(0.05));
        let patterns = vec![Pattern::new(vec![StepToken(1), StepToken(2)], 25, (0, 0))];
        assert!(compose_proposals(&patterns, &s, |_| None).is_empty());
    }

    #[test]
    // rationale: compose_proposals — ci-None snapshot also drops the batch.
    fn compose_drops_batch_when_ci_none() {
        let s = snap(40, Some(0.5), None);
        let patterns = vec![Pattern::new(vec![StepToken(1), StepToken(2)], 25, (0, 0))];
        assert!(compose_proposals(&patterns, &s, |_| None).is_empty());
    }

    #[test]
    // rationale: compose_proposals — every proposal in a multi-pattern batch
    // carries the SAME evidence snapshot (n / lift / ci) since one snapshot
    // is broadcast across all patterns.
    fn compose_broadcasts_one_snapshot_across_all_proposals() {
        let s = snap(42, Some(0.66), Some(0.044));
        let patterns = vec![
            Pattern::new(vec![StepToken(1), StepToken(2)], 25, (0, 0)),
            Pattern::new(vec![StepToken(3), StepToken(4)], 22, (0, 0)),
        ];
        let out = compose_proposals(&patterns, &s, |_| None);
        assert!(out.len() >= 2);
        for p in &out {
            assert_eq!(p.evidence_n(), 42);
            assert!((p.evidence_lift() - 0.66).abs() < 1e-12);
            assert!((p.evidence_ci_half() - 0.044).abs() < 1e-12);
        }
    }

    #[test]
    // rationale: compose_proposals — proposal_ids within a single batch are
    // unique because each variant_id is distinct (anti-collision).
    fn compose_proposal_ids_unique_within_batch() {
        let s = snap(30, Some(0.5), Some(0.05));
        let pattern = Pattern::new(vec![StepToken(1), StepToken(2), StepToken(3)], 25, (0, 0));
        let out = compose_proposals(&[pattern], &s, |_| None);
        let mut ids: Vec<u64> = out.iter().map(super::WorkflowProposal::proposal_id).collect();
        let len_before = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), len_before, "proposal_ids collided within batch");
    }

    #[test]
    // rationale: Adversarial — a very large lift (well outside [0,1]) is
    // information, not a gate; it must pass through verbatim.
    fn adversarial_large_lift_passes_through() {
        let s = snap(30, Some(1e9), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert!((p.evidence_lift() - 1e9).abs() < 1.0);
    }

    #[test]
    // rationale: Adversarial — zero ci_half (perfectly tight CI) is a valid
    // value and must not be confused with the None sentinel.
    fn adversarial_zero_ci_half_accepted() {
        let s = snap(30, Some(0.5), Some(0.0));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert!((p.evidence_ci_half() - 0.0).abs() < 1e-15);
    }

    #[test]
    // rationale: F2 contract — Some(0.0) lift is distinct from None: a
    // zero-lift snapshot builds, a None-lift snapshot refuses.
    fn f2_some_zero_lift_distinct_from_none_lift() {
        let with_zero = snap(30, Some(0.0), Some(0.05));
        let with_none = snap(30, None, Some(0.05));
        assert!(build_proposal(sample_variant(), &with_zero, None).is_ok());
        assert!(matches!(
            build_proposal(sample_variant(), &with_none, None),
            Err(ProposerError::LiftUnavailable)
        ));
    }

    #[test]
    // rationale: Contract — ProposerError is a thiserror enum; the Display
    // string for EvidenceBelowThreshold embeds both n and threshold.
    fn contract_below_threshold_display_embeds_both_numbers() {
        let e = ProposerError::EvidenceBelowThreshold { n: 13, threshold: 20 };
        let s = format!("{e}");
        assert!(s.contains("13"), "display missing n: {s}");
        assert!(s.contains("20"), "display missing threshold: {s}");
    }

    #[test]
    // rationale: Contract — LiftUnavailable Display is a stable,
    // non-empty diagnostic mentioning the F2 gate.
    fn contract_lift_unavailable_display_mentions_f2() {
        let s = format!("{}", ProposerError::LiftUnavailable);
        assert!(s.contains("F2"), "display should mention the F2 gate: {s}");
    }

    #[test]
    // rationale: Serde — WorkflowProposal serialises every evidence field as
    // a JSON-visible key (downstream m30 bank reads these by name).
    fn serde_proposal_exposes_all_evidence_keys() {
        let s = snap(33, Some(0.71), Some(0.06));
        let p = build_proposal(sample_variant(), &s, Some(4)).expect("ok");
        let json = serde_json::to_string(&p).expect("ser");
        for key in [
            "proposal_id",
            "evidence_n",
            "evidence_lift",
            "evidence_ci_half",
            "diversity_cluster",
        ] {
            assert!(json.contains(key), "serde missing key '{key}': {json}");
        }
    }

    #[test]
    // rationale: Serde — a proposal with Some(cluster) round-trips the
    // cluster value, distinguishing it from the None case.
    fn serde_roundtrip_preserves_some_diversity_cluster() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, Some(7)).expect("ok");
        let back: super::WorkflowProposal =
            serde_json::from_str(&serde_json::to_string(&p).expect("ser")).expect("de");
        assert_eq!(back.diversity_cluster(), Some(7));
        assert_eq!(back, p);
    }

    #[test]
    // rationale: PartialEq — two proposals built from the same inputs are
    // structurally equal; differing diversity_cluster breaks equality.
    fn equality_sensitive_to_diversity_cluster() {
        let s = snap(30, Some(0.5), Some(0.05));
        let a = build_proposal(sample_variant(), &s, Some(1)).expect("a");
        let b = build_proposal(sample_variant(), &s, Some(2)).expect("b");
        assert_ne!(a, b, "proposals differing only in cluster must be unequal");
    }

    #[test]
    // rationale: Boundary — single-step pattern yields only the identity
    // variant (no swap, no skip), so compose emits exactly one proposal.
    fn compose_single_step_pattern_yields_one_proposal() {
        let s = snap(30, Some(0.5), Some(0.05));
        let pattern = Pattern::new(vec![StepToken(42)], 25, (0, 0));
        let out = compose_proposals(&[pattern], &s, |_| None);
        assert_eq!(out.len(), 1);
        assert!(matches!(out[0].variant().mutation, MutationKind::Identity));
    }

    #[test]
    // rationale: compose_proposals — source ordering is preserved: pattern A
    // proposals precede pattern B proposals in the output vec.
    fn compose_preserves_source_pattern_ordering() {
        let s = snap(30, Some(0.5), Some(0.05));
        let pat_a = Pattern::new(vec![StepToken(100)], 25, (0, 0));
        let pat_b = Pattern::new(vec![StepToken(200)], 25, (0, 0));
        let hash_a = pat_a.canonical_hash();
        let hash_b = pat_b.canonical_hash();
        let out = compose_proposals(&[pat_a, pat_b], &s, |_| None);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].variant().source_pattern_hash, hash_a);
        assert_eq!(out[1].variant().source_pattern_hash, hash_b);
    }

    #[test]
    // rationale: Determinism — proposal_id is stable across a clone of the
    // variant (clone must not perturb the FNV input).
    fn determinism_proposal_id_stable_under_variant_clone() {
        let s = snap(30, Some(0.5), Some(0.05));
        let v = sample_variant();
        let id_orig = build_proposal(v.clone(), &s, None).expect("orig").proposal_id();
        let id_clone = build_proposal(v, &s, None).expect("clone").proposal_id();
        assert_eq!(id_orig, id_clone);
    }

    // ---- Phase 7 CC-7 wiring: compose_proposals_with_pressure ----

    use super::{
        compose_proposals_with_pressure, mutation_safety_weight, pressure_priority_bonus,
        MAX_PRESSURE_CONTRIBUTION,
    };

    #[test]
    // rationale: D22 "additive, bounded" — the per-proposal bonus must
    // always fall within `[0.0, MAX_PRESSURE_CONTRIBUTION]` regardless
    // of the input pressure value (NaN, ±inf, far above 1.0, negative).
    fn pressure_bonus_is_bounded_under_adversarial_input() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        for pressure in [
            -1e9_f64,
            -1.0,
            0.0,
            0.25,
            0.5,
            1.0,
            10.0,
            1e9,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            let bonus = pressure_priority_bonus(pressure, &p);
            assert!(
                bonus.is_finite(),
                "bonus must be finite for any pressure (got NaN/inf at p={pressure})"
            );
            assert!(
                (0.0..=MAX_PRESSURE_CONTRIBUTION).contains(&bonus),
                "bonus {bonus} out of [0, {MAX_PRESSURE_CONTRIBUTION}] at p={pressure}"
            );
        }
    }

    #[test]
    // rationale: Identity-no-op — zero pressure produces zero bonus.
    fn pressure_bonus_zero_pressure_is_zero() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert!((pressure_priority_bonus(0.0, &p) - 0.0).abs() < 1e-15);
    }

    #[test]
    // rationale: D22 — at saturated pressure on an Identity variant, the
    // bonus equals the ceiling exactly (Identity's safety weight is 1.0).
    fn pressure_bonus_max_pressure_identity_hits_ceiling() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        // sample_variant() is the first build_variants output = Identity.
        assert!(matches!(p.variant().mutation, MutationKind::Identity));
        let bonus = pressure_priority_bonus(1.0, &p);
        assert!((bonus - MAX_PRESSURE_CONTRIBUTION).abs() < 1e-15);
    }

    #[test]
    // rationale: D22 — safety weights are ordered Identity > Swap > Skip.
    fn mutation_safety_weight_ordering() {
        let identity_w = mutation_safety_weight(&MutationKind::Identity);
        let swap_w = mutation_safety_weight(&MutationKind::Swap { at: 0 });
        let skip_w = mutation_safety_weight(&MutationKind::Skip { at: 0 });
        assert!(
            identity_w > swap_w && swap_w > skip_w,
            "expected Identity > Swap > Skip: got {identity_w} > {swap_w} > {skip_w}"
        );
    }

    #[test]
    // rationale: Determinism — zero pressure ⇒ output IDENTICAL to
    // `compose_proposals`. This is the no-pressure backwards-compat
    // contract (the entire reorder branch must be a no-op).
    fn compose_with_pressure_zero_matches_compose() {
        let s = snap(30, Some(0.5), Some(0.05));
        let patterns = vec![
            Pattern::new(vec![StepToken(1), StepToken(2), StepToken(3)], 25, (0, 0)),
            Pattern::new(vec![StepToken(7), StepToken(8)], 22, (0, 1)),
        ];
        let base = compose_proposals(&patterns, &s, |_| None);
        let with_pressure = compose_proposals_with_pressure(&patterns, &s, |_| None, 0.0);
        assert_eq!(base.len(), with_pressure.len());
        for (a, b) in base.iter().zip(with_pressure.iter()) {
            assert_eq!(a.proposal_id(), b.proposal_id());
        }
    }

    #[test]
    // rationale: Determinism — non-finite pressure (NaN / ±inf) is
    // identical to zero pressure (silent NaN-safe collapse).
    fn compose_with_pressure_non_finite_pressure_is_no_op() {
        let s = snap(30, Some(0.5), Some(0.05));
        let patterns = vec![Pattern::new(vec![StepToken(1), StepToken(2)], 25, (0, 0))];
        let base = compose_proposals(&patterns, &s, |_| None);
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -10.0] {
            let out = compose_proposals_with_pressure(&patterns, &s, |_| None, bad);
            assert_eq!(out.len(), base.len(), "len differs at pressure={bad}");
            for (a, b) in base.iter().zip(out.iter()) {
                assert_eq!(a.proposal_id(), b.proposal_id(), "id differs at p={bad}");
            }
        }
    }

    #[test]
    // rationale: D22 + D24 — under elevated pressure, the Identity variant
    // must surface AT OR BEFORE all Skip variants on the same pattern.
    // (Skip variants carry the smallest safety weight; under any positive
    // pressure their bonus is strictly smaller than Identity's.)
    fn compose_with_pressure_surfaces_identity_before_skip() {
        let s = snap(30, Some(0.5), Some(0.05));
        // Multi-step pattern → m21 emits Identity + Swap + Skip variants.
        let patterns =
            vec![Pattern::new(vec![StepToken(1), StepToken(2), StepToken(3)], 25, (0, 0))];
        let out = compose_proposals_with_pressure(&patterns, &s, |_| None, 1.0);
        assert!(out.len() >= 2);
        let identity_pos = out
            .iter()
            .position(|p| matches!(p.variant().mutation, MutationKind::Identity))
            .expect("Identity must be present");
        // Every Skip variant must come AT OR AFTER the Identity variant.
        for (i, p) in out.iter().enumerate() {
            if matches!(p.variant().mutation, MutationKind::Skip { .. }) {
                assert!(
                    i >= identity_pos,
                    "Skip variant surfaced before Identity at pressure=1.0 \
                     (Skip@{i}, Identity@{identity_pos})"
                );
            }
        }
    }

    #[test]
    // rationale: Determinism — repeated calls with identical pressure
    // produce identical orderings (stable sort + pure inputs).
    fn compose_with_pressure_is_deterministic() {
        let s = snap(30, Some(0.5), Some(0.05));
        let patterns =
            vec![Pattern::new(vec![StepToken(11), StepToken(12), StepToken(13)], 25, (0, 0))];
        let a = compose_proposals_with_pressure(&patterns, &s, |_| None, 0.75);
        let b = compose_proposals_with_pressure(&patterns, &s, |_| None, 0.75);
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.proposal_id(), y.proposal_id());
        }
    }

    #[test]
    // rationale: D22 bounded — across the full `[0,1]` pressure band, no
    // proposal's effective priority shifts by more than the ceiling.
    fn compose_with_pressure_contribution_never_exceeds_ceiling() {
        let s = snap(30, Some(0.5), Some(0.05));
        let pattern = Pattern::new(vec![StepToken(1), StepToken(2), StepToken(3)], 25, (0, 0));
        let proposals = compose_proposals(&[pattern], &s, |_| None);
        for proposal in &proposals {
            for pressure in [0.0, 0.1, 0.25, 0.5, 0.75, 1.0] {
                let bonus = pressure_priority_bonus(pressure, proposal);
                assert!(
                    bonus <= MAX_PRESSURE_CONTRIBUTION + 1e-12,
                    "bonus {bonus} exceeds ceiling {MAX_PRESSURE_CONTRIBUTION} at p={pressure}"
                );
            }
        }
    }

    #[test]
    // rationale: Empty input — pressure-aware compose on empty patterns
    // returns empty (no allocation, no panic), matching the baseline.
    fn compose_with_pressure_empty_input_returns_empty() {
        let s = snap(30, Some(0.5), Some(0.05));
        let out = compose_proposals_with_pressure(&[], &s, |_| None, 0.9);
        assert!(out.is_empty());
    }

    #[test]
    // rationale: F2 gate still fires under pressure — pressure does NOT
    // promote sub-threshold evidence past the F2 floor (pressure modulates
    // ORDER, never bypasses the evidence gate).
    fn compose_with_pressure_does_not_bypass_f2_gate() {
        let s = snap(PROPOSAL_F2_THRESHOLD - 1, Some(0.5), Some(0.05));
        let patterns = vec![Pattern::new(vec![StepToken(1), StepToken(2)], 25, (0, 0))];
        let out = compose_proposals_with_pressure(&patterns, &s, |_| None, 1.0);
        assert!(
            out.is_empty(),
            "pressure must NOT promote sub-F2 proposals past the evidence gate"
        );
    }

    // ========================================================================
    // W1 escape_surface wire-bump tests (Plan v2 v0.2.0 §3 Phase 5 step 2;
    // DX-W.b = W1, DX-W.c = SemVer-break per §15). Verifies:
    //   - default `build_proposal` → escape_surface = Sandboxed (D7 fail-safe)
    //   - `build_proposal_with_escape_surface` threads explicit surface
    //   - SemVer-break at wire level: v0.1.0-shape JSONL (missing
    //     escape_surface) fails to deserialise
    //   - serde round-trip preserves the surface
    // ========================================================================

    #[test]
    fn build_proposal_default_escape_surface_is_sandboxed() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert_eq!(
            p.escape_surface(),
            EscapeSurfaceProfile::Sandboxed,
            "W1 build_proposal default per Plan v2 §15 D7 = Sandboxed (most-restrictive)"
        );
    }

    #[test]
    fn build_proposal_with_escape_surface_threads_explicit_surface() {
        let s = snap(30, Some(0.5), Some(0.05));
        for surface in EscapeSurfaceProfile::VARIANTS {
            let p = build_proposal_with_escape_surface(sample_variant(), &s, None, surface)
                .expect("ok");
            assert_eq!(
                p.escape_surface(),
                surface,
                "build_proposal_with_escape_surface must preserve the declared surface for {surface:?}"
            );
        }
    }

    #[test]
    fn semver_break_v0_1_0_jsonl_missing_escape_surface_fails_to_deserialise() {
        // Per DX-W.c SemVer-break: a v0.1.0-shape JSONL line (no
        // `escape_surface` field) MUST fail to deserialise into
        // WorkflowProposal at v0.2.0. There is no `#[serde(default)]` on
        // the new field — that absence is the SemVer-break contract.
        let v0_1_0_shape = serde_json::json!({
            "proposal_id": 12345_u64,
            "variant": {
                "variant_id": 42_u64,
                "steps": [],
                "mutation": "identity",
                "source_pattern_hash": 7_u64,
            },
            "evidence_n": 30_usize,
            "evidence_lift": 0.5_f64,
            "evidence_ci_half": 0.05_f64,
            "diversity_cluster": null,
            // NOTE: escape_surface intentionally absent — this is the
            // v0.1.0 wire-shape the SemVer-break refuses.
        });
        let s = v0_1_0_shape.to_string();
        let result: Result<WorkflowProposal, _> = serde_json::from_str(&s);
        assert!(
            result.is_err(),
            "v0.1.0 proposals lacking escape_surface MUST fail to deserialise at v0.2.0 (SemVer-break per DX-W.c); got Ok({:?})",
            result.ok()
        );
    }

    #[test]
    fn v0_2_0_jsonl_with_escape_surface_round_trips_successfully() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal_with_escape_surface(
            sample_variant(),
            &s,
            Some(2),
            EscapeSurfaceProfile::FileWrite,
        )
        .expect("build");
        let serialised = serde_json::to_string(&p).expect("ser");
        let restored: WorkflowProposal = serde_json::from_str(&serialised).expect("de");
        assert_eq!(p, restored, "round-trip identity");
        assert_eq!(restored.escape_surface(), EscapeSurfaceProfile::FileWrite);
    }

    // ========================================================================
    // W3 cost field + mutation_weight_for classifier tests (Plan v2 v0.2.0
    // §3 Phase 5 step 3 + §15 D10 metric step_count × mutation_weight per
    // DX-W3.src = variant.mutation source). Verifies:
    //   - mutation_weight_for variant ordering (Identity < Swap < Skip)
    //   - cost projection = step_count × weight
    //   - SemVer-break: v0.1.0+W1-shape JSONL (missing cost) fails to
    //     deserialise (stacks the W1 break)
    //   - cost round-trips through serde
    // ========================================================================

    #[test]
    fn mutation_weight_for_identity_is_baseline_one() {
        assert_eq!(super::mutation_weight_for(&MutationKind::Identity), 1);
    }

    #[test]
    fn mutation_weight_for_orders_aggression_identity_lt_swap_lt_skip() {
        // Cost-frame interpretation: more-aggressive mutations cost more.
        let identity = super::mutation_weight_for(&MutationKind::Identity);
        // Construct Swap and Skip with placeholder indices — the classifier
        // doesn't read inner fields, only matches the variant tag.
        let swap = super::mutation_weight_for(&MutationKind::Swap { at: 0 });
        let skip = super::mutation_weight_for(&MutationKind::Skip { at: 0 });
        assert!(identity < swap, "Identity ({identity}) < Swap ({swap})");
        assert!(swap < skip, "Swap ({swap}) < Skip ({skip})");
    }

    #[test]
    fn cost_projection_is_step_count_times_mutation_weight() {
        // sample_variant() per the existing test fixture returns the first
        // variant from build_variants of a 2-step pattern. We exercise
        // each MutationKind explicitly via mutation_weight_for and check
        // the cost arithmetic via build_proposal's projection.
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        let v = p.variant();
        let expected_cost = i64::try_from(
            (v.steps.len() as u64) * u64::from(super::mutation_weight_for(&v.mutation)),
        )
        .expect("realistic-input cost fits i64");
        assert_eq!(
            p.cost(),
            expected_cost,
            "cost projection = step_count ({}) × mutation_weight ({}) for mutation {:?}",
            v.steps.len(),
            super::mutation_weight_for(&v.mutation),
            v.mutation
        );
    }

    #[test]
    fn semver_break_v0_2_0_jsonl_missing_cost_fails_to_deserialise() {
        // Stacks the W1 SemVer-break: a JSONL line that has `escape_surface`
        // (W1 wire-shape) but NOT `cost` (the W3 addition) MUST fail to
        // deserialise. No `#[serde(default)]` on cost — the absence is
        // the SemVer-break contract per DX-W.c semantics extended to W3.
        let w1_only_shape = serde_json::json!({
            "proposal_id": 12345_u64,
            "variant": {
                "variant_id": 42_u64,
                "steps": [],
                "mutation": "identity",
                "source_pattern_hash": 7_u64,
            },
            "evidence_n": 30_usize,
            "evidence_lift": 0.5_f64,
            "evidence_ci_half": 0.05_f64,
            "diversity_cluster": null,
            "escape_surface": "sandboxed",
            // NOTE: cost intentionally absent.
        });
        let s = w1_only_shape.to_string();
        let result: Result<WorkflowProposal, _> = serde_json::from_str(&s);
        assert!(
            result.is_err(),
            "v0.1.0+W1-only proposals lacking cost MUST fail to deserialise at v0.2.0 W3 (SemVer-break stacking)"
        );
    }

    #[test]
    fn cost_round_trips_through_serde() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal_with_escape_surface(
            sample_variant(),
            &s,
            None,
            EscapeSurfaceProfile::Sandboxed,
        )
        .expect("build");
        let cost_before = p.cost();
        let serialised = serde_json::to_string(&p).expect("ser");
        let restored: WorkflowProposal = serde_json::from_str(&serialised).expect("de");
        assert_eq!(restored.cost(), cost_before, "cost round-trips intact");
    }

    // ========================================================================
    // A4 SD11 12-field proposal shape tests (Plan v2 v0.2.0 §3 Phase 5
    // step 4 + DX-A4-coupling = Phase 5 per C-6). Verifies the four new
    // fields land with sensible defaults at genesis + round-trip through
    // serde + SemVer-break stacks (a v0.2.0+W1+W3-but-no-A4 JSONL fails
    // to deserialise).
    // ========================================================================

    #[test]
    fn a4_lineage_chain_at_genesis_is_single_element_proposal_id() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert_eq!(
            p.lineage_chain(),
            &[p.proposal_id()],
            "genesis proposal lineage_chain == [proposal_id]"
        );
    }

    #[test]
    fn a4_generation_index_at_genesis_is_zero() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert_eq!(
            p.generation_index(),
            0,
            "genesis proposal generation_index == 0 (m11/RALPH bumps post-v0.2.0)"
        );
    }

    #[test]
    fn a4_parent_proposal_id_at_genesis_is_none() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        assert_eq!(
            p.parent_proposal_id(),
            None,
            "genesis proposal has no parent (re-proposals will set Some(parent_id))"
        );
    }

    #[test]
    fn a4_lift_p95_at_genesis_is_lift_plus_ci_half() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal(sample_variant(), &s, None).expect("ok");
        let expected_p95 = p.evidence_lift() + p.evidence_ci_half();
        assert!(
            (p.lift_p95() - expected_p95).abs() < f64::EPSILON,
            "lift_p95 ({}) == evidence_lift ({}) + evidence_ci_half ({}) = {}",
            p.lift_p95(),
            p.evidence_lift(),
            p.evidence_ci_half(),
            expected_p95
        );
    }

    #[test]
    fn semver_break_v0_2_0_w1_w3_jsonl_missing_a4_fields_fails_to_deserialise() {
        // Stacks the W1 + W3 SemVer-breaks: a JSONL line that has W1
        // (escape_surface) + W3 (cost) but NOT the A4 fields
        // (lineage_chain / generation_index / parent_proposal_id /
        // lift_p95) MUST fail to deserialise. No `#[serde(default)]` on
        // the A4 fields — the absence is the SemVer-break contract
        // stacking per DX-W.c.
        let w1_w3_only_shape = serde_json::json!({
            "proposal_id": 12345_u64,
            "variant": {
                "variant_id": 42_u64,
                "steps": [],
                "mutation": "identity",
                "source_pattern_hash": 7_u64,
            },
            "evidence_n": 30_usize,
            "evidence_lift": 0.5_f64,
            "evidence_ci_half": 0.05_f64,
            "diversity_cluster": null,
            "escape_surface": "sandboxed",
            "cost": 0_i64,
            // NOTE: A4 fields intentionally absent.
        });
        let s = w1_w3_only_shape.to_string();
        let result: Result<WorkflowProposal, _> = serde_json::from_str(&s);
        assert!(
            result.is_err(),
            "v0.2.0+W1+W3-only proposals lacking A4 fields MUST fail to deserialise (SemVer-break stack per DX-W.c + A4)"
        );
    }

    #[test]
    fn a4_full_12_field_round_trips_through_serde() {
        let s = snap(30, Some(0.5), Some(0.05));
        let p = build_proposal_with_escape_surface(
            sample_variant(),
            &s,
            Some(2),
            EscapeSurfaceProfile::FileWrite,
        )
        .expect("build");
        let serialised = serde_json::to_string(&p).expect("ser");
        let restored: WorkflowProposal = serde_json::from_str(&serialised).expect("de");
        // Full 12-field identity check.
        assert_eq!(p, restored, "12-field round-trip identity");
        // Field-by-field A4 spot-checks.
        assert_eq!(restored.lineage_chain(), p.lineage_chain());
        assert_eq!(restored.generation_index(), p.generation_index());
        assert_eq!(restored.parent_proposal_id(), p.parent_proposal_id());
        assert!((restored.lift_p95() - p.lift_p95()).abs() < f64::EPSILON);
    }
}
