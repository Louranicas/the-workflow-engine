//! `m30_curated_bank` — accepted-proposal storage with sunset_at + decay
//! weights. Cluster G · L7.

use std::collections::BTreeMap;
use std::sync::Mutex;

use thiserror::Error;

use crate::m23_proposer::WorkflowProposal;

/// Default sunset window (days from acceptance).
pub const DEFAULT_SUNSET_DAYS: i64 = 120;

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

/// Bank errors.
#[derive(Debug, Error)]
pub enum BankError {
    /// Tried to look up a workflow that isn't in the bank.
    #[error("workflow {0} not found")]
    NotFound(u64),
    /// Cannot accept a proposal twice.
    #[error("workflow {0} already accepted")]
    AlreadyAccepted(u64),
}

/// The curated bank.
pub struct CuratedBank {
    inner: Mutex<BTreeMap<u64, AcceptedWorkflow>>,
}

impl std::fmt::Debug for CuratedBank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.inner.lock().map_or(0, |g| g.len());
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
    /// # Errors
    ///
    /// [`BankError::AlreadyAccepted`] if the workflow id already exists.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (unrecoverable; prior panic).
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
            sunset_at_ms: now_ms.saturating_add(DEFAULT_SUNSET_DAYS * 86_400_000),
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

    /// Apply a decay factor to a workflow's weight.
    pub fn apply_decay(&self, workflow_id: u64, factor: f64) {
        if let Ok(mut g) = self.inner.lock() {
            if let Some(w) = g.get_mut(&workflow_id) {
                w.weight = (w.weight * factor).clamp(0.0, 1.0);
            }
        }
    }

    /// Record a dispatch attempt against a workflow.
    pub fn record_run(&self, workflow_id: u64, now_ms: i64) {
        if let Ok(mut g) = self.inner.lock() {
            if let Some(w) = g.get_mut(&workflow_id) {
                w.last_run_ms = Some(now_ms);
                w.run_count = w.run_count.saturating_add(1);
            }
        }
    }

    /// All workflows whose sunset has NOT yet been reached and whose
    /// weight is above `min_weight`.
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
            .filter(|w| w.sunset_at_ms > now_ms && w.weight >= min_weight)
            .cloned()
            .collect()
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
    use std::time::SystemTime;

    use super::{BankError, CuratedBank, DEFAULT_SUNSET_DAYS};
    use crate::m14_lift::LiftSnapshot;
    use crate::m20_prefixspan::{Pattern, StepToken};
    use crate::m21_variant_builder::build_variants;
    use crate::m23_proposer::build_proposal;

    fn sample_proposal() -> crate::m23_proposer::WorkflowProposal {
        let p = Pattern::new(vec![StepToken(1), StepToken(2)], 30, (0, 1));
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

    #[test]
    fn empty_bank_size_zero() {
        let b = CuratedBank::new();
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
    }

    #[test]
    fn accept_adds_entry_with_default_weight() {
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 1_700_000_000_000).expect("ok");
        let w = b.get(id).expect("get");
        assert!((w.weight - 1.0).abs() < 1e-12);
        assert_eq!(w.run_count, 0);
        assert!(w.last_run_ms.is_none());
    }

    #[test]
    fn accept_rejects_duplicate() {
        let b = CuratedBank::new();
        let p = sample_proposal();
        let _ = b.accept(p.clone(), 1_700_000_000_000).expect("first");
        assert!(matches!(
            b.accept(p, 1_700_000_000_000),
            Err(BankError::AlreadyAccepted(_))
        ));
    }

    #[test]
    fn sunset_default_is_120_days_after_acceptance() {
        let b = CuratedBank::new();
        let now = 1_700_000_000_000_i64;
        let id = b.accept(sample_proposal(), now).expect("ok");
        let w = b.get(id).expect("get");
        let expected = now + DEFAULT_SUNSET_DAYS * 86_400_000;
        assert_eq!(w.sunset_at_ms, expected);
    }

    #[test]
    fn apply_decay_clamps_and_persists() {
        let b = CuratedBank::new();
        let id = b.accept(sample_proposal(), 0).expect("ok");
        b.apply_decay(id, 0.5);
        let w = b.get(id).expect("get");
        assert!((w.weight - 0.5).abs() < 1e-12);
        b.apply_decay(id, -10.0); // would go negative without clamp
        let w = b.get(id).expect("get");
        assert!((0.0..=1.0).contains(&w.weight));
    }

    #[test]
    fn record_run_increments_count() {
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
        let b = CuratedBank::new();
        let now = 1_700_000_000_000_i64;
        let id = b.accept(sample_proposal(), now).expect("ok");
        let later = now + DEFAULT_SUNSET_DAYS * 86_400_000 + 1;
        let actives = b.active(later, 0.01);
        assert!(actives.iter().all(|w| w.workflow_id != id));
    }

    #[test]
    fn active_excludes_low_weight() {
        let b = CuratedBank::new();
        let now = 1_700_000_000_000_i64;
        let id = b.accept(sample_proposal(), now).expect("ok");
        b.apply_decay(id, 0.0); // weight → 0
        let actives = b.active(now + 1, 0.01);
        assert!(actives.is_empty());
    }

    #[test]
    fn not_found_typed_error() {
        let b = CuratedBank::new();
        assert!(matches!(b.get(9999), Err(BankError::NotFound(9999))));
    }
}
