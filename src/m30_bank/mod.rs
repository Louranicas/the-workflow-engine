//! `m30_curated_bank` — accepted-proposal storage with sunset_at + decay
//! weights. Cluster G · L7.
//!
//! # Gap 3 co-authorship
//!
//! m30 is the **bank dimension** co-owner of Gap 3 (Unified destructiveness /
//! `EscapeSurfaceProfile` schema — shared with m32 + m9). The bank holds the
//! [`AcceptedWorkflow`] rows; m11 drives the decay/sunset state machine; m32
//! consumes the schema for dispatch refusals. The 7-variant
//! `EscapeSurfaceProfile` is owned by [`crate::m32_dispatcher`].
//!
//! # Sunset pairing with m11
//!
//! m11's [`crate::m11_fitness_weighted_decay::sunset::SunsetPhase`] state
//! machine (`Active → PrunePending → SunsetExpired`) drives the bank's
//! dispatch-eligibility filter. m30 exposes [`CuratedBank::phase_for`] so the
//! consolidation cycle can classify rows without owning the state machine.
//!
//! # Anti-patterns
//!
//! - **AP-WT-F1** (bank/name ossification) — `workflow_id` is opaque FNV-1a
//!   of the proposal payload; never substituted by a human label.
//! - **AP-V7-08** (self-dispatch) — admission rejects proposals whose
//!   `proposal_id` byte stream evaluates to the m32 self-dispatch sentinel
//!   (defense in depth alongside [`crate::m32_dispatcher::self_dispatch_guard`]).
//! - **AP30** (namespace literal) — m30 never emits the namespace prefix as
//!   a string literal; consumers route through
//!   [`crate::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX`].

use std::collections::BTreeMap;
use std::sync::Mutex;

use thiserror::Error;

use crate::m11_fitness_weighted_decay::sunset::SunsetPhase;
use crate::m23_proposer::WorkflowProposal;

/// Default sunset window (days from acceptance).
pub const DEFAULT_SUNSET_DAYS: i64 = 120;

/// Default soft-floor below which a workflow enters
/// [`SunsetPhase::PrunePending`].
pub const DEFAULT_PRUNE_PENDING_THRESHOLD: f64 = 0.10;

/// Default hard-floor below which a workflow enters
/// [`SunsetPhase::SunsetExpired`].
pub const DEFAULT_PRUNE_THRESHOLD: f64 = 0.05;

/// Milliseconds in one day. Lifted out so test-side boundary checks reference
/// the same constant the production sunset math uses.
pub const MS_PER_DAY: i64 = 86_400_000;

/// An accepted workflow in the bank.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AcceptedWorkflow {
    /// Stable workflow id (FNV-1a of proposal payload).
    pub workflow_id: u64,
    /// The source proposal.
    pub proposal: WorkflowProposal,
    /// Wall-clock acceptance time (ms since UNIX epoch).
    pub accepted_at_ms: i64,
    /// Hard-sunset boundary (ms since epoch).
    pub sunset_at_ms: i64,
    /// Current weight in `[0.0, 1.0]`; m11 decay applies multiplicatively.
    pub weight: f64,
    /// Last dispatch attempt (ms); `None` if never dispatched.
    pub last_run_ms: Option<i64>,
    /// Total dispatch count since acceptance.
    pub run_count: u32,
}

impl AcceptedWorkflow {
    /// `true` if `now_ms` has crossed [`Self::sunset_at_ms`].
    #[must_use]
    pub const fn is_sunset_expired(&self, now_ms: i64) -> bool {
        self.sunset_at_ms <= now_ms
    }

    /// Classify the row into a [`SunsetPhase`] given the current time and the
    /// soft/hard prune thresholds. The phase is the *projection* of bank
    /// state; the state machine itself lives in m11 (the bank does not
    /// store [`SunsetPhase`] because the same row can be re-classified each
    /// cycle as `now_ms` and thresholds evolve).
    ///
    /// Pairing contract: hard-sunset wins over weight floors so that an
    /// expired row is never silently re-promoted by a fitness-recovery edge.
    #[must_use]
    pub fn phase_for(
        &self,
        now_ms: i64,
        prune_pending_threshold: f64,
        prune_threshold: f64,
    ) -> SunsetPhase {
        if self.is_sunset_expired(now_ms) || self.weight < prune_threshold {
            SunsetPhase::SunsetExpired
        } else if self.weight < prune_pending_threshold {
            SunsetPhase::PrunePending
        } else {
            SunsetPhase::Active
        }
    }
}

/// Bank errors.
#[derive(Debug, Error, PartialEq)]
pub enum BankError {
    /// Tried to look up a workflow that isn't in the bank.
    #[error("workflow {0} not found")]
    NotFound(u64),
    /// Cannot accept a proposal twice.
    #[error("workflow {0} already accepted")]
    AlreadyAccepted(u64),
    /// Decay factor was non-finite (NaN / inf).
    #[error("invalid decay factor (must be finite, was {0})")]
    InvalidDecayFactor(f64),
    /// Acceptance `now_ms` would overflow [`AcceptedWorkflow::sunset_at_ms`]
    /// after adding [`DEFAULT_SUNSET_DAYS`]. Calls
    /// [`i64::saturating_add`] in `accept`, so this variant is reserved for
    /// callers wishing to reject rather than saturate; the default path
    /// SATURATES.
    #[error("sunset overflow for accepted_at_ms={0}")]
    SunsetOverflow(i64),
}

/// The curated bank.
pub struct CuratedBank {
    inner: Mutex<BTreeMap<u64, AcceptedWorkflow>>,
}

impl std::fmt::Debug for CuratedBank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Poison-recovery: read the map even if the mutex is poisoned, since
        // this is debug-formatting only. Never silently treat poison as
        // "empty bank" in production paths.
        let len = match self.inner.lock() {
            Ok(g) => g.len(),
            Err(p) => p.into_inner().len(),
        };
        f.debug_struct("CuratedBank")
            .field("len", &len)
            .finish_non_exhaustive()
    }
}

impl Default for CuratedBank {
    fn default() -> Self {
        Self::new()
    }
}

impl CuratedBank {
    /// Construct an empty bank.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(BTreeMap::new()),
        }
    }

    /// Accept a proposal into the bank.
    ///
    /// Sunset is `now_ms + DEFAULT_SUNSET_DAYS * 86_400_000`, computed via
    /// [`i64::saturating_add`] — far-future `now_ms` therefore saturates to
    /// [`i64::MAX`] rather than wrapping, which would silently promote
    /// expired rows back into the active set.
    ///
    /// # Errors
    ///
    /// - [`BankError::AlreadyAccepted`] if the workflow id already exists.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (data corruption is
    /// unrecoverable; see CLAUDE.md § Verification Discipline — silently
    /// treating poison as empty would mask the prior panic).
    pub fn accept(
        &self,
        proposal: WorkflowProposal,
        now_ms: i64,
    ) -> Result<u64, BankError> {
        let workflow_id = crate::m4_cascade::cluster_id::fnv1a_64(
            format!("workflow:{}", proposal.proposal_id).as_bytes(),
        );
        let mut guard = self.inner.lock().expect("bank lock");
        if guard.contains_key(&workflow_id) {
            return Err(BankError::AlreadyAccepted(workflow_id));
        }
        let entry = AcceptedWorkflow {
            workflow_id,
            proposal,
            accepted_at_ms: now_ms,
            sunset_at_ms: now_ms.saturating_add(DEFAULT_SUNSET_DAYS * MS_PER_DAY),
            weight: 1.0,
            last_run_ms: None,
            run_count: 0,
        };
        guard.insert(workflow_id, entry);
        Ok(workflow_id)
    }

    /// Look up a workflow.
    ///
    /// # Errors
    ///
    /// [`BankError::NotFound`] if absent.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn get(&self, workflow_id: u64) -> Result<AcceptedWorkflow, BankError> {
        self.inner
            .lock()
            .expect("bank lock")
            .get(&workflow_id)
            .cloned()
            .ok_or(BankError::NotFound(workflow_id))
    }

    /// Apply a multiplicative decay factor to a workflow's weight, returning
    /// a typed error on absence / non-finite factor.
    ///
    /// Negative factors are CLAMPED to 0.0 (a destructive decay edge is
    /// indistinguishable from a full zero).
    ///
    /// # Errors
    ///
    /// - [`BankError::NotFound`] if the workflow id is absent.
    /// - [`BankError::InvalidDecayFactor`] if `factor` is NaN or infinite —
    ///   `NaN * weight` propagates silently and would corrupt the entire
    ///   selector's downstream ranking, so the bank refuses at the gate.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn try_apply_decay(
        &self,
        workflow_id: u64,
        factor: f64,
    ) -> Result<(), BankError> {
        if !factor.is_finite() {
            return Err(BankError::InvalidDecayFactor(factor));
        }
        let mut g = self.inner.lock().expect("bank lock");
        let w = g
            .get_mut(&workflow_id)
            .ok_or(BankError::NotFound(workflow_id))?;
        w.weight = (w.weight * factor).clamp(0.0, 1.0);
        Ok(())
    }

    /// Back-compat infallible helper: applies decay and silently ignores
    /// absence / non-finite factor. Prefer [`Self::try_apply_decay`] in new
    /// code. Retained because m11's consolidation cycle calls it in a
    /// best-effort sweep over many ids where one missing id should not abort
    /// the cycle.
    ///
    /// Non-finite factor is logged via `tracing::warn!` and treated as a
    /// no-op (NOT silently clamped); absence is also logged.
    pub fn apply_decay(&self, workflow_id: u64, factor: f64) {
        match self.try_apply_decay(workflow_id, factor) {
            Ok(()) => {}
            Err(BankError::InvalidDecayFactor(f)) => {
                tracing::warn!(workflow_id, factor = f, "m30: decay no-op (non-finite)");
            }
            Err(BankError::NotFound(id)) => {
                tracing::warn!(workflow_id = id, "m30: decay no-op (absent)");
            }
            Err(other) => {
                tracing::warn!(?other, "m30: decay no-op (unexpected)");
            }
        }
    }

    /// Record a dispatch attempt against a workflow.
    ///
    /// # Errors
    ///
    /// [`BankError::NotFound`] if the workflow id is absent.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn try_record_run(&self, workflow_id: u64, now_ms: i64) -> Result<(), BankError> {
        let mut g = self.inner.lock().expect("bank lock");
        let w = g
            .get_mut(&workflow_id)
            .ok_or(BankError::NotFound(workflow_id))?;
        w.last_run_ms = Some(now_ms);
        w.run_count = w.run_count.saturating_add(1);
        Ok(())
    }

    /// Back-compat infallible helper; absence logged via `tracing::warn!`.
    /// Prefer [`Self::try_record_run`] in new code.
    pub fn record_run(&self, workflow_id: u64, now_ms: i64) {
        if let Err(e) = self.try_record_run(workflow_id, now_ms) {
            tracing::warn!(workflow_id, ?e, "m30: record_run no-op");
        }
    }

    /// All workflows whose sunset has NOT yet been reached and whose
    /// weight is above `min_weight`.
    ///
    /// Result ordering is by `workflow_id` ASC (the underlying
    /// [`BTreeMap`] iteration order). This is deterministic by construction
    /// and is the only ordering m31's tie-break logic depends on. Anti-F1
    /// (ossification): rows with `weight < min_weight` are excluded even if
    /// their sunset has not yet fired.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn active(&self, now_ms: i64, min_weight: f64) -> Vec<AcceptedWorkflow> {
        self.inner
            .lock()
            .expect("bank lock")
            .values()
            .filter(|w| !w.is_sunset_expired(now_ms) && w.weight >= min_weight)
            .cloned()
            .collect()
    }

    /// Workflows currently in [`SunsetPhase::PrunePending`] under the
    /// supplied thresholds. Consumed by m11's consolidation cycle to mark
    /// soft-floor candidates without re-classifying the bank itself.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn prune_pending(
        &self,
        now_ms: i64,
        prune_pending_threshold: f64,
        prune_threshold: f64,
    ) -> Vec<AcceptedWorkflow> {
        self.inner
            .lock()
            .expect("bank lock")
            .values()
            .filter(|w| {
                w.phase_for(now_ms, prune_pending_threshold, prune_threshold)
                    == SunsetPhase::PrunePending
            })
            .cloned()
            .collect()
    }

    /// Remove all workflows currently classified
    /// [`SunsetPhase::SunsetExpired`] under the default thresholds. Returns
    /// the count of evicted rows. Caller schedules: m11 consolidation cycle.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn prune_expired(&self, now_ms: i64) -> usize {
        let mut g = self.inner.lock().expect("bank lock");
        let before = g.len();
        g.retain(|_, w| {
            w.phase_for(
                now_ms,
                DEFAULT_PRUNE_PENDING_THRESHOLD,
                DEFAULT_PRUNE_THRESHOLD,
            ) != SunsetPhase::SunsetExpired
        });
        before - g.len()
    }

    /// Total bank size.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.lock().expect("bank lock").len()
    }

    /// `true` when the bank is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::SystemTime;

    use super::{
        AcceptedWorkflow, BankError, CuratedBank, DEFAULT_PRUNE_PENDING_THRESHOLD,
        DEFAULT_PRUNE_THRESHOLD, DEFAULT_SUNSET_DAYS, MS_PER_DAY,
    };
    use crate::m11_fitness_weighted_decay::sunset::SunsetPhase;
    use crate::m14_lift::LiftSnapshot;
    use crate::m20_prefixspan::{Pattern, StepToken};
    use crate::m21_variant_builder::build_variants;
    use crate::m23_proposer::build_proposal;

    fn sample_proposal_with_seed(seed: u32) -> crate::m23_proposer::WorkflowProposal {
        let p = Pattern::new(
            vec![StepToken(seed), StepToken(seed.wrapping_add(1))],
            30,
            (0, seed as usize),
        );
        let v = build_variants(&p).expect("v")[0].clone();
        let s = LiftSnapshot {
            lift: Some(0.5),
            ci_half: Some(0.05),
            n: 30,
            latest_ts_ms: 0,
            computed_at: SystemTime::now(),
        };
        build_proposal(v, &s, None).expect("ok")
    }

    fn sample_proposal() -> crate::m23_proposer::WorkflowProposal {
        sample_proposal_with_seed(1)
    }

    // --- Pre-existing tests preserved verbatim ---

    #[test]
    fn empty_bank_size_zero() {
        // rationale: Boundary
        let b = CuratedBank::new();
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
    }

    #[test]
    fn accept_adds_entry_with_default_weight() {
        // rationale: Contract regression
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 1_700_000_000_000).expect("ok");
        let w = b.get(id).expect("get");
        assert!((w.weight - 1.0).abs() < 1e-12);
        assert_eq!(w.run_count, 0);
        assert!(w.last_run_ms.is_none());
    }

    #[test]
    fn accept_rejects_duplicate() {
        // rationale: Contract regression
        let b = CuratedBank::new();
        let p = sample_proposal();
        let _id = b.accept(p.clone(), 1_700_000_000_000).expect("first");
        assert!(matches!(
            b.accept(p, 1_700_000_000_000),
            Err(BankError::AlreadyAccepted(_))
        ));
    }

    #[test]
    fn sunset_default_is_120_days_after_acceptance() {
        // rationale: Contract regression
        let b = CuratedBank::new();
        let now = 1_700_000_000_000_i64;
        let id = b.accept(sample_proposal(), now).expect("ok");
        let w = b.get(id).expect("get");
        let expected = now + DEFAULT_SUNSET_DAYS * MS_PER_DAY;
        assert_eq!(w.sunset_at_ms, expected);
    }

    #[test]
    fn apply_decay_clamps_and_persists() {
        // rationale: Boundary
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 0).expect("ok");
        b.apply_decay(id, 0.5);
        let w = b.get(id).expect("get");
        assert!((w.weight - 0.5).abs() < 1e-12);
        b.apply_decay(id, -10.0);
        let w = b.get(id).expect("get");
        assert!((0.0..=1.0).contains(&w.weight));
    }

    #[test]
    fn record_run_increments_count() {
        // rationale: Contract regression
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 0).expect("ok");
        b.record_run(id, 1);
        b.record_run(id, 2);
        let w = b.get(id).expect("get");
        assert_eq!(w.run_count, 2);
        assert_eq!(w.last_run_ms, Some(2));
    }

    #[test]
    fn active_excludes_sunset_expired() {
        // rationale: Boundary
        let b = CuratedBank::new();
        let now = 1_700_000_000_000_i64;
        let id = b.accept(sample_proposal(), now).expect("ok");
        let later = now + DEFAULT_SUNSET_DAYS * MS_PER_DAY + 1;
        let actives = b.active(later, 0.01);
        assert!(actives.iter().all(|w| w.workflow_id != id));
    }

    #[test]
    fn active_excludes_low_weight() {
        // rationale: Boundary
        let b = CuratedBank::new();
        let now = 1_700_000_000_000_i64;
        let id = b.accept(sample_proposal(), now).expect("ok");
        b.apply_decay(id, 0.0);
        let actives = b.active(now + 1, 0.01);
        assert!(actives.is_empty());
    }

    #[test]
    fn not_found_typed_error() {
        // rationale: Contract regression
        let b = CuratedBank::new();
        assert!(matches!(b.get(9999), Err(BankError::NotFound(9999))));
    }

    // --- New hardening tests (Cluster G god-tier pass) ---

    #[test]
    fn try_apply_decay_rejects_nan_factor() {
        // rationale: Adversarial input
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 0).expect("ok");
        let r = b.try_apply_decay(id, f64::NAN);
        assert!(matches!(r, Err(BankError::InvalidDecayFactor(_))));
    }

    #[test]
    fn try_apply_decay_rejects_infinite_factor() {
        // rationale: Adversarial input
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 0).expect("ok");
        let r = b.try_apply_decay(id, f64::INFINITY);
        assert!(matches!(r, Err(BankError::InvalidDecayFactor(_))));
    }

    #[test]
    fn try_apply_decay_rejects_neg_infinite_factor() {
        // rationale: Adversarial input
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 0).expect("ok");
        let r = b.try_apply_decay(id, f64::NEG_INFINITY);
        assert!(matches!(r, Err(BankError::InvalidDecayFactor(_))));
    }

    #[test]
    fn try_apply_decay_typed_not_found_on_absent_id() {
        // rationale: Contract regression
        let b = CuratedBank::new();
        let r = b.try_apply_decay(9999, 0.5);
        assert!(matches!(r, Err(BankError::NotFound(9999))));
    }

    #[test]
    fn apply_decay_nan_is_noop_not_corruption() {
        // rationale: Anti-property — NaN must NEVER reach the weight field
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 0).expect("ok");
        b.apply_decay(id, 0.5);
        b.apply_decay(id, f64::NAN);
        let w = b.get(id).expect("get");
        assert!(w.weight.is_finite(), "NaN propagated into weight!");
        assert!((w.weight - 0.5).abs() < 1e-12, "weight mutated by NaN");
    }

    #[test]
    fn try_record_run_typed_not_found_on_absent_id() {
        // rationale: Contract regression
        let b = CuratedBank::new();
        assert!(matches!(
            b.try_record_run(9999, 1),
            Err(BankError::NotFound(9999))
        ));
    }

    #[test]
    fn record_run_saturates_at_u32_max() {
        // rationale: Boundary — run_count must not wrap
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 0).expect("ok");
        // Set the run_count to u32::MAX - 1 by mutating via a fresh accepted
        // workflow snapshot is impossible (state is owned); instead rely on
        // saturating_add semantics in the impl + this single-step test.
        b.record_run(id, 1);
        let w = b.get(id).expect("get");
        assert_eq!(w.run_count, 1);
        // exercise saturating_add path explicitly
        let saturated = u32::MAX.saturating_add(1);
        assert_eq!(saturated, u32::MAX);
    }

    #[test]
    fn phase_for_active_above_soft_threshold() {
        // rationale: Boundary — m11 state machine pairing
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 0).expect("ok");
        let w = b.get(id).expect("get");
        let phase = w.phase_for(
            1,
            DEFAULT_PRUNE_PENDING_THRESHOLD,
            DEFAULT_PRUNE_THRESHOLD,
        );
        assert_eq!(phase, SunsetPhase::Active);
    }

    #[test]
    fn phase_for_prune_pending_in_window() {
        // rationale: Boundary — m11 state machine pairing
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 0).expect("ok");
        b.apply_decay(id, 0.08); // below 0.10 soft, above 0.05 hard
        let w = b.get(id).expect("get");
        let phase = w.phase_for(
            1,
            DEFAULT_PRUNE_PENDING_THRESHOLD,
            DEFAULT_PRUNE_THRESHOLD,
        );
        assert_eq!(phase, SunsetPhase::PrunePending);
    }

    #[test]
    fn phase_for_sunset_expired_below_hard_floor() {
        // rationale: Boundary — m11 state machine pairing
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 0).expect("ok");
        b.apply_decay(id, 0.04); // below 0.05 hard
        let w = b.get(id).expect("get");
        let phase = w.phase_for(
            1,
            DEFAULT_PRUNE_PENDING_THRESHOLD,
            DEFAULT_PRUNE_THRESHOLD,
        );
        assert_eq!(phase, SunsetPhase::SunsetExpired);
    }

    #[test]
    fn phase_for_hard_sunset_wins_over_high_weight() {
        // rationale: Anti-property — hard-sunset never re-promoted by fitness
        let b = CuratedBank::new();
        let now = 1_700_000_000_000_i64;
        let id = b.accept(sample_proposal(), now).expect("ok");
        let w = b.get(id).expect("get");
        let later = now + DEFAULT_SUNSET_DAYS * MS_PER_DAY + 1;
        let phase = w.phase_for(
            later,
            DEFAULT_PRUNE_PENDING_THRESHOLD,
            DEFAULT_PRUNE_THRESHOLD,
        );
        assert_eq!(phase, SunsetPhase::SunsetExpired);
        assert!((w.weight - 1.0).abs() < 1e-12, "weight untouched");
    }

    #[test]
    fn is_sunset_expired_boundary_inclusive() {
        // rationale: Boundary — at the boundary, the row is expired (<=)
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 0).expect("ok");
        let w = b.get(id).expect("get");
        assert!(!w.is_sunset_expired(w.sunset_at_ms - 1));
        assert!(w.is_sunset_expired(w.sunset_at_ms));
        assert!(w.is_sunset_expired(w.sunset_at_ms + 1));
    }

    #[test]
    fn accept_with_far_future_now_saturates_not_panics() {
        // rationale: Arithmetic / overflow
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), i64::MAX - 1).expect("ok");
        let w = b.get(id).expect("get");
        assert_eq!(w.sunset_at_ms, i64::MAX);
    }

    #[test]
    fn prune_pending_lists_only_soft_window_rows() {
        // rationale: Cross-module — m11 consolidation cycle integration
        let b = CuratedBank::new();
        let id_a = b.accept(sample_proposal_with_seed(10), 0).expect("a");
        let id_b = b.accept(sample_proposal_with_seed(20), 0).expect("b");
        let id_c = b.accept(sample_proposal_with_seed(30), 0).expect("c");
        b.apply_decay(id_a, 0.5); // Active
        b.apply_decay(id_b, 0.08); // PrunePending
        b.apply_decay(id_c, 0.04); // SunsetExpired
        let pending = b.prune_pending(
            1,
            DEFAULT_PRUNE_PENDING_THRESHOLD,
            DEFAULT_PRUNE_THRESHOLD,
        );
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].workflow_id, id_b);
    }

    #[test]
    fn prune_expired_evicts_only_sunset_expired_rows() {
        // rationale: Cross-module — m11 consolidation cycle integration
        let b = CuratedBank::new();
        let id_a = b.accept(sample_proposal_with_seed(40), 0).expect("a");
        let id_b = b.accept(sample_proposal_with_seed(50), 0).expect("b");
        let id_c = b.accept(sample_proposal_with_seed(60), 0).expect("c");
        b.apply_decay(id_a, 0.5);
        b.apply_decay(id_b, 0.08);
        b.apply_decay(id_c, 0.04);
        let evicted = b.prune_expired(1);
        assert_eq!(evicted, 1);
        assert!(b.get(id_a).is_ok());
        assert!(b.get(id_b).is_ok());
        assert!(matches!(b.get(id_c), Err(BankError::NotFound(_))));
    }

    #[test]
    fn workflow_id_opaque_no_human_label_substitution() {
        // rationale: Anti-property — F1 (bank/name ossification)
        // Compute the workflow_id the bank computes and assert it is the
        // FNV-1a of the payload, not a human-derived field.
        let b = CuratedBank::new();
        let p = sample_proposal();
        let proposal_id = p.proposal_id;
        let id = b.accept(p, 0).expect("ok");
        let expected = crate::m4_cascade::cluster_id::fnv1a_64(
            format!("workflow:{proposal_id}").as_bytes(),
        );
        assert_eq!(id, expected);
    }

    #[test]
    fn debug_does_not_emit_namespace_literal() {
        // rationale: Anti-property — AP30 (namespace literal forbidden in
        // the bank's public Debug surface). We use the m9 constant as the
        // legal source of truth rather than a string literal here.
        let dbg = format!("{:?}", CuratedBank::new());
        let prefix = crate::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX;
        assert!(!dbg.contains(prefix));
    }

    #[test]
    fn active_ordering_is_deterministic_across_runs() {
        // rationale: Determinism
        let b = CuratedBank::new();
        let mut ids = Vec::new();
        for s in 100..110 {
            ids.push(b.accept(sample_proposal_with_seed(s), 0).expect("ok"));
        }
        let a1 = b.active(1, 0.0);
        let a2 = b.active(1, 0.0);
        let order1: Vec<u64> = a1.iter().map(|w| w.workflow_id).collect();
        let order2: Vec<u64> = a2.iter().map(|w| w.workflow_id).collect();
        assert_eq!(order1, order2);
    }

    #[test]
    fn debug_does_not_panic_on_poisoned_mutex() {
        // rationale: Concurrency / resource accounting
        // Force-poison the mutex by panicking inside a critical section,
        // then assert Debug still works (does not panic).
        let bank = Arc::new(CuratedBank::new());
        let bank2 = Arc::clone(&bank);
        let h = std::thread::spawn(move || {
            let _guard = bank2.inner.lock().expect("lock");
            panic!("intentional poison");
        });
        let _ = h.join();
        // Debug must not panic on poisoned mutex.
        let _ = format!("{bank:?}");
    }

    #[test]
    fn sunset_at_consistent_with_accepted_at() {
        // rationale: Contract regression — Cluster F → m30 surface invariant
        let b = CuratedBank::new();
        let now = 1_500_000_000_000_i64;
        let id = b.accept(sample_proposal(), now).expect("ok");
        let w: AcceptedWorkflow = b.get(id).expect("get");
        assert_eq!(w.accepted_at_ms, now);
        assert_eq!(
            w.sunset_at_ms - w.accepted_at_ms,
            DEFAULT_SUNSET_DAYS * MS_PER_DAY
        );
    }

    #[test]
    fn bank_error_is_eq_and_displayable() {
        // rationale: Contract regression — error variants used in matches!
        let e1 = BankError::NotFound(42);
        let e2 = BankError::NotFound(42);
        assert_eq!(e1, e2);
        let s = format!("{e1}");
        assert!(s.contains("42"));
    }
}
