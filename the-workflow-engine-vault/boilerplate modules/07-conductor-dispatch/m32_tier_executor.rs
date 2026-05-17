//! `m32_tier_executor` — 6-tier deployment pipeline engine.
//!
//! Each tier delegates to specific layer modules and gates on confidence
//! before advancing to the next tier.
//!
//! # Tier Overview
//!
//! | Tier | Name        | Gate Before | Key Outputs |
//! |------|-------------|-------------|-------------|
//! | T1   | Foundation  | None        | `Plan`, gaps, dissent |
//! | T2   | Scaffold    | >= 0.80     | Scaffold artefacts |
//! | T3   | Implement   | >= 0.80     | Per-pane results |
//! | T4   | Harden      | >= 0.85     | Quality report, SOR |
//! | T5   | Document    | >= 0.85     | Vault, meta-tree |
//! | T6   | Deploy      | >= 0.90     | Binary deployed |
//!
//! After each tier, when the `learning` feature is enabled,
//! [`m22_hebbian_feedback`] records LTP/LTD outcomes via STDP.
//! Without `--features learning` this path is compiled out entirely.
//! [`m23_episodic_memory`] (also `learning`-gated) updates the deployment episode.
//!
//! # Architecture
//!
//! `TierExecutor` is the central orchestrator.  It is stateless — all
//! mutable workflow state lives in [`m31_workflow_state::WorkflowStateManager`].
//! Tier logic is modelled as pure functions that take a [`WorkflowContext`]
//! by mutable reference.  This makes each tier independently testable.
//!
//! # Layer: `m7_orchestrator`
//! # Dependencies: `m01_core_types`, `m02_error_handling`, `m18_adaptive_threshold`, `m31_workflow_state`

#![forbid(unsafe_code)]

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::m1_core::m01_core_types::{
    Confidence, SystemState, TierId, Timestamp, WorkflowId,
};
use crate::m4_gate::m18_adaptive_threshold::{AdaptiveThreshold, GateDecision};
use crate::m7_orchestrator::m31_workflow_state::{
    Checkpoint, WorkflowState, WorkflowStateManager, WorkflowStatus,
};

// ============================================================================
// Constants
// ============================================================================

/// Confidence threshold required to advance past Tier 1 (Foundation).
pub const GATE_T1: f64 = 0.80;

/// Confidence threshold required to advance past Tier 2 (Scaffold).
pub const GATE_T2: f64 = 0.80;

/// Confidence threshold required to advance past Tier 3 (Implement).
pub const GATE_T3: f64 = 0.85;

/// Confidence threshold required to advance past Tier 4 (Harden).
pub const GATE_T4: f64 = 0.85;

/// Confidence threshold required to advance past Tier 5 (Document).
pub const GATE_T5: f64 = 0.90;

/// Maximum number of tier retries before halting the pipeline.
pub const MAX_RETRIES: u8 = 3;

/// Maximum allowed wall-clock time per tier (in seconds).
///
/// Set conservatively — the agent can checkpoint and resume.
pub const TIER_TIMEOUT_SECS: u64 = 3_600; // 1 hour

// ============================================================================
// Tier enum
// ============================================================================

/// The 6 deployment tiers in execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Tier {
    /// T1 — specification parsing, gap analysis, dissent generation.
    T1Foundation = 1,
    /// T2 — scaffold generation (6 parallel facets via M12).
    T2Scaffold = 2,
    /// T3 — fleet-dispatched implementation (via M33).
    T3Implement = 3,
    /// T4 — quality gate, compliance check, SOR tracking.
    T4Harden = 4,
    /// T5 — documentation generation, vault update, meta-tree.
    T5Document = 5,
    /// T6 — devenv deployment, habitat registration, atuin sync.
    T6Deploy = 6,
}

impl Tier {
    /// Returns the confidence gate threshold that must be met *before* this tier.
    ///
    /// T1 has no gate (returns 0.0 — always passes).
    #[must_use]
    pub const fn gate_threshold(self) -> f64 {
        match self {
            Self::T1Foundation => 0.0,
            Self::T2Scaffold => GATE_T1,
            Self::T3Implement => GATE_T2,
            Self::T4Harden => GATE_T3,
            Self::T5Document => GATE_T4,
            Self::T6Deploy => GATE_T5,
        }
    }

    /// Returns the human-readable tier name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::T1Foundation => "Foundation",
            Self::T2Scaffold => "Scaffold",
            Self::T3Implement => "Implement",
            Self::T4Harden => "Harden",
            Self::T5Document => "Document",
            Self::T6Deploy => "Deploy",
        }
    }

    /// Returns the corresponding [`TierId`].
    ///
    /// Uses `TierId::from_disc` which is `const fn` — no destructor needed.
    #[must_use]
    pub const fn tier_id(self) -> TierId {
        TierId::from_disc(self as u8)
    }

    /// Returns the next tier in the pipeline, or `None` if this is T6.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::T1Foundation => Some(Self::T2Scaffold),
            Self::T2Scaffold => Some(Self::T3Implement),
            Self::T3Implement => Some(Self::T4Harden),
            Self::T4Harden => Some(Self::T5Document),
            Self::T5Document => Some(Self::T6Deploy),
            Self::T6Deploy => None,
        }
    }

    /// Returns all 6 tiers in execution order.
    #[must_use]
    pub const fn all() -> [Self; 6] {
        [
            Self::T1Foundation,
            Self::T2Scaffold,
            Self::T3Implement,
            Self::T4Harden,
            Self::T5Document,
            Self::T6Deploy,
        ]
    }

    /// Constructs a [`Tier`] from a [`TierId`].
    ///
    /// # Errors
    ///
    /// Returns `Err` if `id` does not map to a known tier (values outside 1–6).
    pub const fn from_tier_id(id: TierId) -> Result<Self, ExecutorError> {
        match id.as_u8() {
            1 => Ok(Self::T1Foundation),
            2 => Ok(Self::T2Scaffold),
            3 => Ok(Self::T3Implement),
            4 => Ok(Self::T4Harden),
            5 => Ok(Self::T5Document),
            6 => Ok(Self::T6Deploy),
            n => Err(ExecutorError::InvalidTier { tier: n }),
        }
    }
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T{}:{}", *self as u8, self.name())
    }
}

// ============================================================================
// TierStatus
// ============================================================================

/// Outcome classification of a single tier execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TierOutcome {
    /// Tier completed and the confidence gate passed.
    Passed,
    /// Tier completed but confidence is borderline — human review needed.
    Escalated,
    /// Tier completed but confidence gate failed — pipeline halted.
    Halted,
    /// Tier execution failed with an error.
    Failed,
    /// Tier timed out.
    TimedOut,
}

impl TierOutcome {
    /// Returns `true` if the tier may proceed to the next tier.
    #[must_use]
    pub const fn may_proceed(self) -> bool {
        matches!(self, Self::Passed)
    }

    /// Returns `true` if the pipeline should stop.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Halted | Self::Failed | Self::TimedOut)
    }
}

impl fmt::Display for TierOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Passed => "Passed",
            Self::Escalated => "Escalated",
            Self::Halted => "Halted",
            Self::Failed => "Failed",
            Self::TimedOut => "TimedOut",
        };
        f.write_str(label)
    }
}

// ============================================================================
// TierResult
// ============================================================================

/// Result of executing a single tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierResult {
    /// Which tier this result belongs to.
    pub tier: Tier,
    /// Outcome classification.
    pub outcome: TierOutcome,
    /// Confidence score after tier execution.
    pub confidence: Confidence,
    /// The gate decision at the tier boundary.
    pub gate_decision: GateDecision,
    /// Wall-clock duration of this tier in milliseconds.
    pub elapsed_ms: u64,
    /// Paths of artefacts produced.
    pub artifacts: Vec<PathBuf>,
    /// Human-readable notes (max 512 chars per entry).
    pub notes: Vec<String>,
}

impl TierResult {
    /// Returns `true` if this tier may advance to the next.
    #[must_use]
    pub const fn may_proceed(&self) -> bool {
        self.outcome.may_proceed()
    }
}

// ============================================================================
// PipelineResult
// ============================================================================

/// Complete pipeline execution result for a single workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    /// Unique identifier for the workflow run.
    pub workflow_id: WorkflowId,
    /// Tier results in execution order.
    pub tier_results: Vec<TierResult>,
    /// Final workflow status.
    pub final_status: WorkflowStatus,
    /// Total wall-clock duration in milliseconds.
    pub total_elapsed_ms: u64,
    /// Confidence at pipeline completion (last tier confidence).
    pub final_confidence: Confidence,
    /// All artefacts produced across all tiers.
    pub all_artifacts: Vec<PathBuf>,
    /// When the pipeline completed.
    pub completed_at: Timestamp,
}

impl PipelineResult {
    /// Returns `true` if the full pipeline succeeded.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.final_status == WorkflowStatus::Completed
    }

    /// Returns the number of tiers that passed.
    #[must_use]
    pub fn passed_tier_count(&self) -> usize {
        self.tier_results
            .iter()
            .filter(|r| r.outcome == TierOutcome::Passed)
            .count()
    }
}

// ============================================================================
// WorkflowContext
// ============================================================================

/// Mutable context passed to each tier execution function.
///
/// Carries the active workflow state, accumulated artefacts, and the
/// tier-to-confidence mapping built up as tiers complete.
#[derive(Debug)]
pub struct WorkflowContext {
    /// The active workflow state (owned, then written back via manager).
    pub state: WorkflowState,
    /// Accumulated results across all tiers.
    pub tier_results: Vec<TierResult>,
    /// Number of completed deployments before this one (used for adaptive threshold).
    pub deployment_count: u32,
    /// Current confidence (updated after each tier).
    pub confidence: Confidence,
    /// Session identifier.
    pub session_id: String,
}

impl WorkflowContext {
    /// Creates a new context for the given `workflow_state`.
    #[must_use]
    pub fn new(state: WorkflowState, deployment_count: u32) -> Self {
        let session_id = state.session_id.clone();
        Self {
            state,
            tier_results: Vec::with_capacity(6),
            deployment_count,
            confidence: Confidence::ZERO,
            session_id,
        }
    }
}

// ============================================================================
// EventEmitter trait
// ============================================================================

/// Fire-and-forget event emission for ORAC and PV2 integration.
///
/// All methods are synchronous and non-blocking — implementations must not
/// acquire any locks held by the caller.  A call that fails logs a warning
/// and returns; it never propagates errors to the pipeline.
///
/// # Implementation contract
///
/// - Never block the caller for more than a few microseconds.
/// - Log failures with `tracing::warn!` rather than panicking.
/// - Emission order within a single tier is guaranteed; cross-tier ordering
///   is best-effort.
///
/// # Examples
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use dev_ops_engine_v3::m7_orchestrator::m32_tier_executor::{
///     EventEmitter, NullEventEmitter, TierExecutor,
/// };
///
/// let exec = TierExecutor::new("/tmp/proj", 0);
/// exec.set_event_emitter(Arc::new(NullEventEmitter));
/// ```
pub trait EventEmitter: Send + Sync {
    /// Called after a tier finishes (pass, escalate, halt, or fail).
    ///
    /// `tier` is the raw tier number (1–6).  `success` is `true` only for
    /// [`TierOutcome::Passed`].
    fn emit_tier_completed(
        &self,
        tier: u8,
        success: bool,
        confidence: f64,
        elapsed_ms: u64,
    );

    /// Called at the moment the gate decision is made for a tier.
    ///
    /// `decision` is one of `"pass"`, `"escalate"`, `"fail"`, or `"other"`.
    fn emit_gate_decision(
        &self,
        tier: u8,
        decision: &str,
        score: f64,
        threshold: f64,
    );

    /// Called when a new workflow is accepted (before T1 begins).
    fn emit_workflow_started(&self, workflow_id: &str, goal: &str);

    /// Called after the final tier exits, whether successful or not.
    ///
    /// `tiers_passed` is the count of tiers that reached
    /// [`TierOutcome::Passed`].
    fn emit_workflow_complete(
        &self,
        workflow_id: &str,
        success: bool,
        tiers_passed: u8,
    );
}

// ============================================================================
// NullEventEmitter
// ============================================================================

/// No-op [`EventEmitter`] — drops every event silently.
///
/// Suitable for tests, benchmarks, and deployments where ORAC / PV2 are
/// not reachable.
///
/// # Examples
///
/// ```rust
/// use dev_ops_engine_v3::m7_orchestrator::m32_tier_executor::NullEventEmitter;
/// let e = NullEventEmitter;
/// e.emit_tier_completed(1, true, 0.9, 1000);
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct NullEventEmitter;

impl EventEmitter for NullEventEmitter {
    fn emit_tier_completed(
        &self,
        _tier: u8,
        _success: bool,
        _confidence: f64,
        _elapsed_ms: u64,
    ) {
    }

    fn emit_gate_decision(
        &self,
        _tier: u8,
        _decision: &str,
        _score: f64,
        _threshold: f64,
    ) {
    }

    fn emit_workflow_started(&self, _workflow_id: &str, _goal: &str) {}

    fn emit_workflow_complete(
        &self,
        _workflow_id: &str,
        _success: bool,
        _tiers_passed: u8,
    ) {
    }
}

// ============================================================================
// ExecutorError
// ============================================================================

/// Errors produced by [`TierExecutor`].
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    /// An invalid tier number was encountered.
    #[error("invalid tier number: {tier}")]
    InvalidTier {
        /// The tier number that is out of range.
        tier: u8,
    },
    /// A tier confidence gate failed; the pipeline is halted.
    #[error("tier {tier} confidence gate halted (score {score:.4} < threshold {threshold:.4})")]
    GateHalted {
        /// Tier that failed the gate.
        tier: String,
        /// Confidence score at gate evaluation.
        score: f64,
        /// Required threshold.
        threshold: f64,
    },
    /// A tier exceeded its configured timeout.
    #[error("tier {tier} timed out after {elapsed_secs}s")]
    TierTimeout {
        /// Tier label.
        tier: String,
        /// Seconds elapsed.
        elapsed_secs: u64,
    },
    /// The workflow manager returned an error.
    #[error("workflow state error: {0}")]
    StateError(#[from] crate::m7_orchestrator::m31_workflow_state::StateError),
    /// Maximum retry limit exceeded.
    #[error("tier {tier} exceeded maximum retries ({max})")]
    MaxRetriesExceeded {
        /// Tier label.
        tier: String,
        /// Retry limit.
        max: u8,
    },
    /// Only one concurrent workflow is allowed.
    #[error("another workflow is already active")]
    WorkflowAlreadyActive,
}

// ============================================================================
// TierExecutor
// ============================================================================

/// Orchestrates the 6-tier deployment pipeline.
///
/// `TierExecutor` is stateless — all workflow state lives in
/// [`WorkflowStateManager`].  Each tier is executed as a synchronous
/// function call operating on a [`WorkflowContext`].
///
/// # Thread Safety
///
/// `TierExecutor` is `Send + Sync` and designed to be shared as
/// `Arc<TierExecutor>` across async task boundaries.
///
/// # Event Emission
///
/// When an [`EventEmitter`] is installed via [`TierExecutor::set_event_emitter`],
/// the executor fires events at key lifecycle points:
///
/// - [`EventEmitter::emit_workflow_started`] — on successful [`create_workflow`]
///   (called by the consumer; see [`create_workflow`] docs).
/// - [`EventEmitter::emit_tier_completed`] — after every tier outcome.
/// - [`EventEmitter::emit_gate_decision`] — at every gate evaluation.
/// - [`EventEmitter::emit_workflow_complete`] — at the end of
///   [`run_pipeline`] and [`resume_from_checkpoint`].
///
/// Events are fire-and-forget: a failing emitter logs a warning but never
/// aborts the pipeline.
pub struct TierExecutor {
    /// Workflow state manager (shared with API layer).
    state_manager: WorkflowStateManager,
    /// Adaptive threshold evaluator.
    threshold: AdaptiveThreshold,
    /// Number of completed deployments before the current run.
    deployment_count: u32,
    /// Optional event emitter for ORAC / PV2 integration.
    ///
    /// Wrapped in `parking_lot::RwLock` so that `set_event_emitter` can be
    /// called on a shared `Arc<TierExecutor>` without requiring `&mut self`.
    event_emitter: parking_lot::RwLock<Option<Arc<dyn EventEmitter>>>,
}

impl std::fmt::Debug for TierExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TierExecutor")
            .field("state_manager", &self.state_manager)
            .field("threshold", &self.threshold)
            .field("deployment_count", &self.deployment_count)
            .field("event_emitter", &self.event_emitter.read().is_some())
            .finish()
    }
}

impl TierExecutor {
    /// Creates a new [`TierExecutor`].
    ///
    /// `project_root` is passed to the state manager to locate `.workflow/`.
    /// `deployment_count` is the number of *completed* deployments before
    /// this run (0 for the very first deployment).
    #[must_use]
    pub fn new(
        project_root: impl Into<std::path::PathBuf>,
        deployment_count: u32,
    ) -> Self {
        Self {
            state_manager: WorkflowStateManager::new(project_root),
            threshold: AdaptiveThreshold::new(),
            deployment_count,
            event_emitter: parking_lot::RwLock::new(None),
        }
    }

    /// Installs (or replaces) the [`EventEmitter`] used for ORAC / PV2 hooks.
    ///
    /// May be called at any time on a shared `Arc<TierExecutor>` — internally
    /// guarded by a `RwLock`.  Subsequent pipeline calls will use the new
    /// emitter.  Pass [`NullEventEmitter`] to disable emission without
    /// removing the field.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// exec.set_event_emitter(Arc::new(NullEventEmitter));
    /// ```
    pub fn set_event_emitter(&self, emitter: Arc<dyn EventEmitter>) {
        *self.event_emitter.write() = Some(emitter);
    }

    /// Returns a clone of the currently installed emitter, if any.
    #[must_use]
    fn emitter(&self) -> Option<Arc<dyn EventEmitter>> {
        self.event_emitter.read().clone()
    }

    /// Evaluates the confidence gate for `tier` using the adaptive schedule.
    ///
    /// Returns `Ok(GateDecision)` — callers check [`GateDecision::may_proceed`]
    /// to decide whether to continue.
    #[must_use]
    pub fn evaluate_gate(&self, tier: Tier, confidence: Confidence) -> GateDecision {
        self.threshold
            .evaluate(confidence, tier.tier_id(), self.deployment_count)
    }

    /// Executes a single tier and returns its [`TierResult`].
    ///
    /// This function:
    /// 1. Evaluates the pre-tier confidence gate.
    /// 2. Marks the tier as `InProgress` in the state manager.
    /// 3. Simulates tier work (real implementation dispatches to M07-M33).
    /// 4. Evaluates the post-tier confidence gate.
    /// 5. Checkpoints state if the tier passed.
    ///
    /// # Errors
    ///
    /// Returns [`ExecutorError::GateHalted`] if confidence is below the
    /// escalate threshold.  Returns [`ExecutorError::StateError`] on
    /// persistence failure.
    pub fn execute_tier(
        &self,
        tier: Tier,
        ctx: &mut WorkflowContext,
        simulated_confidence: Confidence,
        artifacts: Vec<PathBuf>,
    ) -> Result<TierResult, ExecutorError> {
        let started = Instant::now();
        let threshold = tier.gate_threshold();

        // Pre-tier gate check (skipped for T1).
        if threshold > 0.0 && !ctx.confidence.meets(threshold) {
            let decision = self.evaluate_gate(tier, ctx.confidence);

            // Emit gate decision for the pre-tier check — all locks are
            // already released at this point.
            if let Some(ref emitter) = self.emitter() {
                let label = if decision.may_proceed() {
                    "pass"
                } else if decision.is_halt() {
                    "fail"
                } else {
                    "escalate"
                };
                emitter.emit_gate_decision(
                    tier as u8,
                    label,
                    ctx.confidence.value(),
                    threshold,
                );
            }

            if decision.is_halt() {
                return Err(ExecutorError::GateHalted {
                    tier: tier.to_string(),
                    score: ctx.confidence.value(),
                    threshold,
                });
            }
        }

        // Start tier in state manager.
        self.state_manager
            .update(&ctx.state.workflow_id, |s| {
                s.start_tier(tier.tier_id())?;
                Ok(())
            })?;

        // Simulate tier execution (real impl calls into L2/L3/L4/L6/M33).
        let confidence_after = simulated_confidence;

        // Evaluate post-tier gate.
        let decision = self.evaluate_gate(tier, confidence_after);
        let outcome = if decision.may_proceed() {
            TierOutcome::Passed
        } else if decision.is_halt() {
            TierOutcome::Halted
        } else {
            TierOutcome::Escalated
        };

        let elapsed_ms = elapsed_millis(started);

        // Checkpoint on pass.
        if outcome == TierOutcome::Passed {
            self.state_manager
                .update(&ctx.state.workflow_id, |s| {
                    s.complete_tier(tier.tier_id(), confidence_after, &artifacts)
                        .map(|_| ())?;
                    Ok(())
                })?;
        } else {
            self.state_manager
                .update(&ctx.state.workflow_id, |s| {
                    s.fail_tier(tier.tier_id(), format!("{decision:?}"))?;
                    Ok(())
                })?;
        }

        // All state-manager guards are dropped.  Emit events now.
        if let Some(ref emitter) = self.emitter() {
            // Post-tier gate decision.
            let gate_label = match outcome {
                TierOutcome::Passed => "pass",
                TierOutcome::Escalated => "escalate",
                TierOutcome::Halted | TierOutcome::Failed | TierOutcome::TimedOut => "fail",
            };
            emitter.emit_gate_decision(
                tier as u8,
                gate_label,
                confidence_after.value(),
                tier.gate_threshold(),
            );

            // Tier completion event.
            emitter.emit_tier_completed(
                tier as u8,
                outcome == TierOutcome::Passed,
                confidence_after.value(),
                elapsed_ms,
            );
        }

        ctx.confidence = confidence_after;
        let mut notes = Vec::new();
        notes.push(format!(
            "gate={decision:?} conf={confidence_after:.4} elapsed={elapsed_ms}ms"
        ));

        let result = TierResult {
            tier,
            outcome,
            confidence: confidence_after,
            gate_decision: decision,
            elapsed_ms,
            artifacts,
            notes,
        };

        ctx.tier_results.push(result.clone());

        if outcome.is_terminal() && outcome != TierOutcome::Passed {
            return Err(ExecutorError::GateHalted {
                tier: tier.to_string(),
                score: confidence_after.value(),
                threshold: tier.gate_threshold(),
            });
        }

        Ok(result)
    }

    /// Runs the complete 6-tier pipeline for `workflow_id`.
    ///
    /// `confidence_provider` is a closure that returns the simulated or actual
    /// confidence for each tier.  In production, this calls into M17.
    ///
    /// # Errors
    ///
    /// Returns [`ExecutorError::WorkflowAlreadyActive`] if `MAX_ACTIVE_WORKFLOWS`
    /// (1) is exceeded.  Returns tier-specific errors on gate failure.
    pub fn run_pipeline<F>(
        &self,
        workflow_id: WorkflowId,
        confidence_provider: F,
    ) -> Result<PipelineResult, ExecutorError>
    where
        F: Fn(Tier) -> (Confidence, Vec<PathBuf>),
    {
        let state = self.state_manager.get(&workflow_id)?;
        let mut ctx = WorkflowContext::new(state, self.deployment_count);

        let pipeline_start = Instant::now();
        let mut all_artifacts: Vec<PathBuf> = Vec::new();

        for tier in Tier::all() {
            let (conf, artifacts) = confidence_provider(tier);
            let result = self.execute_tier(tier, &mut ctx, conf, artifacts.clone())?;
            all_artifacts.extend_from_slice(&artifacts);

            tracing::info!(
                tier = %tier,
                outcome = %result.outcome,
                confidence = result.confidence.value(),
                elapsed_ms = result.elapsed_ms,
                "tier completed"
            );
        }

        let total_elapsed_ms = elapsed_millis(pipeline_start);
        let final_confidence = ctx.confidence;

        // Retrieve final state.
        let final_state = self.state_manager.get(&workflow_id)?;

        let tiers_passed = ctx
            .tier_results
            .iter()
            .filter(|r| r.outcome == TierOutcome::Passed)
            .count();
        let pipeline_success = final_state.status == WorkflowStatus::Completed;

        // Emit workflow_complete — all locks released before this call.
        if let Some(ref emitter) = self.emitter() {
            emitter.emit_workflow_complete(
                &workflow_id.to_string(),
                pipeline_success,
                u8::try_from(tiers_passed).unwrap_or(u8::MAX),
            );
        }

        Ok(PipelineResult {
            workflow_id,
            tier_results: ctx.tier_results,
            final_status: final_state.status,
            total_elapsed_ms,
            final_confidence,
            all_artifacts,
            completed_at: Timestamp::now(),
        })
    }

    /// Resumes a workflow from the last [`Checkpoint`].
    ///
    /// Only tiers after the last successful checkpoint are executed.
    ///
    /// # Errors
    ///
    /// Returns [`ExecutorError::StateError`] if the workflow is not found.
    pub fn resume_from_checkpoint<F>(
        &self,
        workflow_id: WorkflowId,
        confidence_provider: F,
    ) -> Result<PipelineResult, ExecutorError>
    where
        F: Fn(Tier) -> (Confidence, Vec<PathBuf>),
    {
        let state = self.state_manager.get(&workflow_id)?;
        let resume_tier = state
            .last_checkpoint()
            .and_then(Checkpoint::next_tier)
            .and_then(|id| Tier::from_tier_id(id).ok())
            .unwrap_or(Tier::T1Foundation);
        // Seed confidence from last checkpoint so the pre-tier gate check passes.
        let resume_confidence = state
            .last_checkpoint()
            .map_or(Confidence::ZERO, |ckpt| ckpt.confidence);

        let mut ctx = WorkflowContext::new(state, self.deployment_count);
        ctx.confidence = resume_confidence;
        let pipeline_start = Instant::now();
        let mut all_artifacts: Vec<PathBuf> = Vec::new();

        for tier in Tier::all() {
            if (tier as u8) < (resume_tier as u8) {
                // Tier already completed in a previous session — skip.
                continue;
            }
            let (conf, artifacts) = confidence_provider(tier);
            let result = self.execute_tier(tier, &mut ctx, conf, artifacts.clone())?;
            all_artifacts.extend_from_slice(&artifacts);

            tracing::info!(
                tier = %tier,
                outcome = %result.outcome,
                "resumed tier completed"
            );
        }

        let final_state = self.state_manager.get(&workflow_id)?;

        let tiers_passed = ctx
            .tier_results
            .iter()
            .filter(|r| r.outcome == TierOutcome::Passed)
            .count();
        let pipeline_success = final_state.status == WorkflowStatus::Completed;

        // Emit workflow_complete — all locks released before this call.
        if let Some(ref emitter) = self.emitter() {
            emitter.emit_workflow_complete(
                &workflow_id.to_string(),
                pipeline_success,
                u8::try_from(tiers_passed).unwrap_or(u8::MAX),
            );
        }

        Ok(PipelineResult {
            workflow_id,
            tier_results: ctx.tier_results,
            final_status: final_state.status,
            total_elapsed_ms: elapsed_millis(pipeline_start),
            final_confidence: ctx.confidence,
            all_artifacts,
            completed_at: Timestamp::now(),
        })
    }

    /// Returns the current system state derived from active workflow status.
    #[must_use]
    pub fn system_state(&self) -> SystemState {
        let active = self.state_manager.active_workflows();
        match active.first().map(|s| s.status) {
            Some(WorkflowStatus::Planning) => SystemState::Planning,
            Some(WorkflowStatus::Scaffolding) => SystemState::Scaffolding,
            Some(WorkflowStatus::Implementing) => SystemState::Implementing,
            Some(WorkflowStatus::Hardening) => SystemState::Hardening,
            Some(WorkflowStatus::Documenting) => SystemState::Documenting,
            Some(WorkflowStatus::Deploying) => SystemState::Deploying,
            None | Some(_) => SystemState::Idle,
        }
    }

    /// Creates a new workflow and returns its id.
    ///
    /// The active-workflow count check and the record insertion happen inside a
    /// single `records.write()` guard via [`WorkflowStateManager::create_exclusive`]
    /// to eliminate the TOCTOU window (C-06) that would allow two concurrent
    /// callers to both pass the limit check and each insert a workflow.
    ///
    /// Emits [`EventEmitter::emit_workflow_started`] with an empty goal string.
    /// Use [`TierExecutor::create_workflow_with_goal`] when the deployment goal
    /// is available at creation time.
    ///
    /// # Errors
    ///
    /// Returns [`ExecutorError::WorkflowAlreadyActive`] if a workflow is
    /// already running (`MAX_ACTIVE_WORKFLOWS = 1`).
    pub fn create_workflow(&self, session_id: String) -> Result<WorkflowId, ExecutorError> {
        self.create_workflow_with_goal(session_id, "")
    }

    /// Creates a new workflow with an explicit deployment goal string.
    ///
    /// Identical to [`create_workflow`] but also supplies `goal` to
    /// [`EventEmitter::emit_workflow_started`] for richer ORAC / PV2 context.
    ///
    /// # Errors
    ///
    /// Returns [`ExecutorError::WorkflowAlreadyActive`] if a workflow is
    /// already running (`MAX_ACTIVE_WORKFLOWS = 1`).
    pub fn create_workflow_with_goal(
        &self,
        session_id: String,
        goal: &str,
    ) -> Result<WorkflowId, ExecutorError> {
        let workflow_id = self
            .state_manager
            .create_exclusive(session_id, MAX_ACTIVE_WORKFLOWS)
            .map(|s| s.workflow_id)
            .ok_or(ExecutorError::WorkflowAlreadyActive)?;

        // All state-manager guards released before emitting.
        if let Some(ref emitter) = self.emitter() {
            emitter.emit_workflow_started(&workflow_id.to_string(), goal);
        }

        Ok(workflow_id)
    }

    /// Returns a snapshot of an active workflow.
    ///
    /// # Errors
    ///
    /// Returns [`ExecutorError::StateError`] if the workflow is not found.
    pub fn workflow_state(
        &self,
        id: &WorkflowId,
    ) -> Result<WorkflowState, ExecutorError> {
        Ok(self.state_manager.get(id)?)
    }
}

/// Maximum number of concurrently active workflows.
const MAX_ACTIVE_WORKFLOWS: usize = 1;

// ============================================================================
// Hebbian wiring (feature = "learning")
// ============================================================================

/// Records a [`TierOutcome`] into the Hebbian feedback system after a tier
/// completes.
///
/// This is the L7 → L5 wiring point.  Because `m5_learning` is feature-gated,
/// this function is only compiled when the `learning` feature is enabled.
///
/// # How to use from the orchestrator level
///
/// Any component that owns both an `Arc<TierExecutor>` and an
/// `Arc<HebbianFeedback>` (e.g. `m34_pipeline_orchestrator`) should call this
/// after every [`TierExecutor::execute_tier`] / [`TierExecutor::run_pipeline`]
/// invocation:
///
/// ```rust,ignore
/// #[cfg(feature = "learning")]
/// {
///     use crate::m7_orchestrator::m32_tier_executor::record_tier_hebbian;
///     record_tier_hebbian(&hebbian, &tier_result, "deploy-001", 2);
/// }
/// ```
///
/// # Arguments
///
/// * `hebbian`            — shared reference to the [`HebbianFeedback`] manager.
/// * `result`             — the [`TierResult`] returned by [`TierExecutor::execute_tier`].
/// * `deployment_id`      — deployment / session identifier for the audit log.
/// * `working_tier_count` — number of tiers concurrently active (idle-gate check).
#[cfg(feature = "learning")]
pub fn record_tier_hebbian(
    hebbian: &crate::m5_learning::m22_hebbian_feedback::HebbianFeedback,
    result: &TierResult,
    deployment_id: &str,
    working_tier_count: u32,
) {
    match hebbian.record_tier_result(
        result.tier as u8,
        &result.outcome,
        deployment_id,
        result.elapsed_ms,
        working_tier_count,
    ) {
        Ok(true) => {
            tracing::debug!(tier = %result.tier, "Hebbian STDP applied for tier");
        }
        Ok(false) => {
            tracing::debug!(
                tier = %result.tier,
                "Hebbian STDP idle-gated — fewer than IDLE_GATE_THRESHOLD working tiers"
            );
        }
        Err(e) => {
            tracing::warn!(
                tier = %result.tier,
                error = %e,
                "Hebbian record_tier_result failed — learning loop skipped for this tier"
            );
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Measures wall-clock elapsed milliseconds from `started`.
fn elapsed_millis(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Returns a human-readable tier gate summary string.
#[must_use]
pub fn gate_summary(tier: Tier, decision: &GateDecision) -> String {
    format!("{tier} gate: {decision:?}")
}

/// Returns the gate threshold constant for a tier as a [`Confidence`].
///
/// The threshold is clamped so it is always a valid [`Confidence`].
#[must_use]
pub fn tier_gate_confidence(tier: Tier) -> Confidence {
    Confidence::clamp(tier.gate_threshold())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m1_core::m01_core_types::{Confidence, TierId};

    fn conf(v: f64) -> Confidence {
        Confidence::clamp(v)
    }

    fn make_executor() -> TierExecutor {
        TierExecutor::new("/tmp/test-pipeline", 10) // 10 = Proficient phase
    }

    // ------------------------------------------------------------------
    // Tier enum
    // ------------------------------------------------------------------

    #[test]
    fn tier_t1_gate_threshold_is_zero() {
        assert!((Tier::T1Foundation.gate_threshold() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tier_t2_gate_threshold_is_0_80() {
        assert!((Tier::T2Scaffold.gate_threshold() - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn tier_t6_gate_threshold_is_0_90() {
        assert!((Tier::T6Deploy.gate_threshold() - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn tier_t6_next_is_none() {
        assert!(Tier::T6Deploy.next().is_none());
    }

    #[test]
    fn tier_t1_next_is_t2() {
        assert_eq!(Tier::T1Foundation.next(), Some(Tier::T2Scaffold));
    }

    #[test]
    fn tier_all_has_six_elements() {
        assert_eq!(Tier::all().len(), 6);
    }

    #[test]
    fn tier_all_in_ascending_order() {
        let tiers = Tier::all();
        for window in tiers.windows(2) {
            assert!(window[0] < window[1]);
        }
    }

    #[test]
    fn tier_from_tier_id_round_trips() {
        for n in 1u8..=6 {
            let id = TierId::new(n).unwrap();
            let tier = Tier::from_tier_id(id).unwrap();
            assert_eq!(tier.tier_id().as_u8(), n);
        }
    }

    #[test]
    fn tier_from_tier_id_invalid_returns_error() {
        // TierId only allows 1..=6; simulate an invalid tier via the error path
        // by calling from_tier_id with a valid id and then checking the enum value.
        // We cannot construct an invalid TierId, so test the error variant directly.
        let err = ExecutorError::InvalidTier { tier: 9 };
        assert!(err.to_string().contains('9'));
    }

    #[test]
    fn tier_display_contains_name() {
        assert!(Tier::T3Implement.to_string().contains("Implement"));
    }

    #[test]
    fn tier_name_is_correct() {
        assert_eq!(Tier::T5Document.name(), "Document");
    }

    // ------------------------------------------------------------------
    // TierOutcome
    // ------------------------------------------------------------------

    #[test]
    fn tier_outcome_passed_may_proceed() {
        assert!(TierOutcome::Passed.may_proceed());
    }

    #[test]
    fn tier_outcome_halted_is_terminal() {
        assert!(TierOutcome::Halted.is_terminal());
    }

    #[test]
    fn tier_outcome_failed_is_terminal() {
        assert!(TierOutcome::Failed.is_terminal());
    }

    #[test]
    fn tier_outcome_escalated_may_not_proceed() {
        assert!(!TierOutcome::Escalated.may_proceed());
    }

    #[test]
    fn tier_outcome_timed_out_is_terminal() {
        assert!(TierOutcome::TimedOut.is_terminal());
    }

    #[test]
    fn tier_outcome_display_passed() {
        assert_eq!(TierOutcome::Passed.to_string(), "Passed");
    }

    // ------------------------------------------------------------------
    // Gate thresholds
    // ------------------------------------------------------------------

    #[test]
    fn tier_gate_confidence_t1_is_zero() {
        let c = tier_gate_confidence(Tier::T1Foundation);
        assert_eq!(c.value(), 0.0);
    }

    #[test]
    fn tier_gate_confidence_t6_is_0_90() {
        let c = tier_gate_confidence(Tier::T6Deploy);
        assert!((c.value() - 0.90).abs() < 1e-9);
    }

    // ------------------------------------------------------------------
    // TierExecutor — create and basic workflow
    // ------------------------------------------------------------------

    #[test]
    fn executor_create_workflow_succeeds() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        let state = exec.workflow_state(&id).unwrap();
        assert_eq!(state.workflow_id, id);
    }

    #[test]
    fn executor_create_two_workflows_returns_error() {
        let exec = make_executor();
        let _ = exec.create_workflow("s001".into()).unwrap();
        let result = exec.create_workflow("s002".into());
        assert!(matches!(result, Err(ExecutorError::WorkflowAlreadyActive)));
    }

    #[test]
    fn executor_system_state_idle_with_no_workflows() {
        let exec = make_executor();
        assert_eq!(exec.system_state(), SystemState::Idle);
    }

    #[test]
    fn executor_system_state_planning_after_t1_starts() {
        let exec = TierExecutor::new("/tmp/test-pipeline", 10);
        let id = exec.create_workflow("s001".into()).unwrap();
        // T1 is in Pending state; starting it transitions to Planning.
        exec.state_manager
            .update(&id, |s| {
                s.start_tier(TierId::new(1).unwrap())?;
                Ok(())
            })
            .unwrap();
        assert_eq!(exec.system_state(), SystemState::Planning);
    }

    // ------------------------------------------------------------------
    // TierExecutor — execute_tier
    // ------------------------------------------------------------------

    #[test]
    fn execute_tier_t1_with_high_confidence_passes() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        let state = exec.workflow_state(&id).unwrap();
        let mut ctx = WorkflowContext::new(state, 10);

        let result = exec
            .execute_tier(Tier::T1Foundation, &mut ctx, conf(0.92), vec![])
            .unwrap();
        assert_eq!(result.outcome, TierOutcome::Passed);
    }

    #[test]
    fn execute_tier_t2_low_confidence_halts() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        // Start T1 and complete it so T2 can be attempted.
        exec.state_manager
            .update(&id, |s| {
                s.start_tier(TierId::new(1).unwrap())?;
                s.complete_tier(TierId::new(1).unwrap(), conf(0.9), &[])
                    .map(|_| ())?;
                Ok(())
            })
            .unwrap();

        let state = exec.workflow_state(&id).unwrap();
        let mut ctx = WorkflowContext::new(state, 10);
        ctx.confidence = conf(0.4); // Below T2 gate (0.80).

        let result = exec.execute_tier(Tier::T2Scaffold, &mut ctx, conf(0.9), vec![]);
        // With deployment_count=10 (Proficient phase), T1-T2 gate pass=0.90,
        // escalate=0.65. conf=0.4 < 0.65 so it should halt.
        assert!(result.is_err());
    }

    #[test]
    fn execute_tier_confidence_updates_ctx() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        let state = exec.workflow_state(&id).unwrap();
        let mut ctx = WorkflowContext::new(state, 10);

        exec.execute_tier(Tier::T1Foundation, &mut ctx, conf(0.88), vec![])
            .unwrap();
        assert!((ctx.confidence.value() - 0.88).abs() < 1e-9);
    }

    #[test]
    fn execute_tier_appends_to_ctx_results() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        let state = exec.workflow_state(&id).unwrap();
        let mut ctx = WorkflowContext::new(state, 10);

        assert_eq!(ctx.tier_results.len(), 0);
        exec.execute_tier(Tier::T1Foundation, &mut ctx, conf(0.9), vec![])
            .unwrap();
        assert_eq!(ctx.tier_results.len(), 1);
    }

    #[test]
    fn execute_tier_artifacts_stored() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        let state = exec.workflow_state(&id).unwrap();
        let mut ctx = WorkflowContext::new(state, 10);

        let artifacts = vec![PathBuf::from("out/file.rs")];
        let result = exec
            .execute_tier(Tier::T1Foundation, &mut ctx, conf(0.9), artifacts)
            .unwrap();
        assert_eq!(result.artifacts.len(), 1);
    }

    // ------------------------------------------------------------------
    // run_pipeline
    // ------------------------------------------------------------------

    #[test]
    fn run_pipeline_all_pass_returns_completed() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();

        let result = exec
            .run_pipeline(id, |_| (conf(0.95), vec![]))
            .unwrap();

        assert!(result.is_success());
        assert_eq!(result.passed_tier_count(), 6);
    }

    #[test]
    fn run_pipeline_six_tier_results() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        let result = exec
            .run_pipeline(id, |_| (conf(0.95), vec![]))
            .unwrap();
        assert_eq!(result.tier_results.len(), 6);
    }

    #[test]
    fn run_pipeline_unknown_workflow_returns_error() {
        let exec = make_executor();
        let unknown = WorkflowId::new();
        let result = exec.run_pipeline(unknown, |_| (conf(0.9), vec![]));
        assert!(result.is_err());
    }

    #[test]
    fn run_pipeline_final_confidence_equals_last_tier() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        let result = exec
            .run_pipeline(id, |tier| {
                let c = if tier == Tier::T6Deploy { 0.93 } else { 0.95 };
                (conf(c), vec![])
            })
            .unwrap();
        assert!((result.final_confidence.value() - 0.93).abs() < 1e-9);
    }

    #[test]
    fn run_pipeline_artifacts_accumulated() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        let result = exec
            .run_pipeline(id, |_| {
                (conf(0.95), vec![PathBuf::from("artifact.rs")])
            })
            .unwrap();
        assert_eq!(result.all_artifacts.len(), 6);
    }

    // ------------------------------------------------------------------
    // PipelineResult helpers
    // ------------------------------------------------------------------

    #[test]
    fn pipeline_result_passed_count_correct() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        let result = exec
            .run_pipeline(id, |_| (conf(0.95), vec![]))
            .unwrap();
        assert_eq!(result.passed_tier_count(), 6);
    }

    #[test]
    fn pipeline_result_is_success_true_on_completed() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        let result = exec
            .run_pipeline(id, |_| (conf(0.95), vec![]))
            .unwrap();
        assert!(result.is_success());
    }

    // ------------------------------------------------------------------
    // WorkflowContext
    // ------------------------------------------------------------------

    #[test]
    fn workflow_context_new_has_zero_confidence() {
        let state = WorkflowState::new("s001".into());
        let ctx = WorkflowContext::new(state, 5);
        assert_eq!(ctx.confidence, Confidence::ZERO);
    }

    #[test]
    fn workflow_context_session_id_matches_state() {
        let state = WorkflowState::new("s001".into());
        let ctx = WorkflowContext::new(state, 5);
        assert_eq!(ctx.session_id, "s001");
    }

    // ------------------------------------------------------------------
    // Gate summary helper
    // ------------------------------------------------------------------

    #[test]
    fn gate_summary_contains_tier_name() {
        use crate::m4_gate::m18_adaptive_threshold::GateDecision;
        let decision = GateDecision::ForcedFoundation {
            deployment_number: 1,
        };
        let summary = gate_summary(Tier::T1Foundation, &decision);
        assert!(summary.contains("Foundation"));
    }

    // ------------------------------------------------------------------
    // ExecutorError messages
    // ------------------------------------------------------------------

    #[test]
    fn executor_error_gate_halted_message_contains_tier() {
        let err = ExecutorError::GateHalted {
            tier: "T2:Scaffold".into(),
            score: 0.5,
            threshold: 0.85,
        };
        assert!(err.to_string().contains("T2:Scaffold"));
    }

    #[test]
    fn executor_error_tier_timeout_message_contains_secs() {
        let err = ExecutorError::TierTimeout {
            tier: "T3:Implement".into(),
            elapsed_secs: 3601,
        };
        assert!(err.to_string().contains("3601"));
    }

    #[test]
    fn executor_error_max_retries_message() {
        let err = ExecutorError::MaxRetriesExceeded {
            tier: "T1:Foundation".into(),
            max: 3,
        };
        assert!(err.to_string().contains("3"));
    }

    // ------------------------------------------------------------------
    // Constants sanity
    // ------------------------------------------------------------------

    #[test]
    fn gate_constants_in_ascending_order() {
        assert!(GATE_T1 <= GATE_T2);
        assert!(GATE_T2 <= GATE_T3);
        assert!(GATE_T3 <= GATE_T4);
        assert!(GATE_T4 <= GATE_T5);
    }

    #[test]
    fn max_retries_non_zero() {
        assert!(MAX_RETRIES > 0);
    }

    #[test]
    fn tier_timeout_secs_is_reasonable() {
        // Between 5 minutes and 24 hours.
        assert!(TIER_TIMEOUT_SECS >= 300);
        assert!(TIER_TIMEOUT_SECS <= 86_400);
    }

    // ------------------------------------------------------------------
    // Resume from checkpoint
    // ------------------------------------------------------------------

    #[test]
    fn resume_from_checkpoint_skips_completed_tiers() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();

        // Manually complete T1 and T2.
        exec.state_manager
            .update(&id, |s| {
                for t in [1u8, 2] {
                    s.start_tier(TierId::new(t).unwrap())?;
                    s.complete_tier(TierId::new(t).unwrap(), conf(0.92), &[])
                        .map(|_| ())?;
                }
                Ok(())
            })
            .unwrap();

        // Restore T3-T6 to Pending (they already are; just verify resume runs).
        let result = exec
            .resume_from_checkpoint(id, |_| (conf(0.95), vec![]))
            .unwrap();

        // Should execute T3-T6 = 4 tiers.
        assert_eq!(result.tier_results.len(), 4);
    }

    // ------------------------------------------------------------------
    // Hebbian wiring (feature = "learning")
    // ------------------------------------------------------------------

    #[cfg(feature = "learning")]
    #[test]
    fn record_tier_hebbian_passed_applies_ltp() {
        use crate::m5_learning::m22_hebbian_feedback::HebbianFeedback;
        use super::record_tier_hebbian;

        let hebbian = HebbianFeedback::new();
        let result = TierResult {
            tier: Tier::T1Foundation,
            outcome: TierOutcome::Passed,
            confidence: conf(0.9),
            gate_decision: crate::m4_gate::m18_adaptive_threshold::GateDecision::ForcedFoundation {
                deployment_number: 1,
            },
            elapsed_ms: 5_000,
            artifacts: vec![],
            notes: vec![],
        };

        let before = hebbian.weight("P03"); // tier 1 → P03
        record_tier_hebbian(&hebbian, &result, "deploy-test", 2);
        // Weight should increase (LTP applied).
        assert!(
            hebbian.weight("P03") > before,
            "record_tier_hebbian should apply LTP for Passed outcome"
        );
    }

    #[cfg(feature = "learning")]
    #[test]
    fn record_tier_hebbian_failed_applies_ltd() {
        use crate::m5_learning::m22_hebbian_feedback::HebbianFeedback;
        use super::record_tier_hebbian;

        let hebbian = HebbianFeedback::new();
        let result = TierResult {
            tier: Tier::T2Scaffold,
            outcome: TierOutcome::Failed,
            confidence: conf(0.3),
            gate_decision: crate::m4_gate::m18_adaptive_threshold::GateDecision::ForcedFoundation {
                deployment_number: 1,
            },
            elapsed_ms: 2_000,
            artifacts: vec![],
            notes: vec![],
        };

        let before = hebbian.weight("P04"); // tier 2 → P04
        record_tier_hebbian(&hebbian, &result, "deploy-test", 2);
        assert!(
            hebbian.weight("P04") < before,
            "record_tier_hebbian should apply LTD for Failed outcome"
        );
    }

    #[cfg(feature = "learning")]
    #[test]
    fn record_tier_hebbian_idle_gate_does_not_panic() {
        use crate::m5_learning::m22_hebbian_feedback::HebbianFeedback;
        use super::record_tier_hebbian;

        let hebbian = HebbianFeedback::new();
        let result = TierResult {
            tier: Tier::T3Implement,
            outcome: TierOutcome::Passed,
            confidence: conf(0.9),
            gate_decision: crate::m4_gate::m18_adaptive_threshold::GateDecision::ForcedFoundation {
                deployment_number: 1,
            },
            elapsed_ms: 1_000,
            artifacts: vec![],
            notes: vec![],
        };
        // working_tier_count = 0 → idle gate → should not panic.
        record_tier_hebbian(&hebbian, &result, "deploy-test", 0);
    }

    // ------------------------------------------------------------------
    // EventEmitter trait + NullEventEmitter
    // ------------------------------------------------------------------

    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    /// Spy emitter that counts calls to each method.
    struct SpyEmitter {
        tier_completed: AtomicU32,
        gate_decisions: AtomicU32,
        workflow_started: AtomicU32,
        workflow_complete: AtomicU32,
        last_success: AtomicBool,
    }

    impl SpyEmitter {
        fn new() -> Self {
            Self {
                tier_completed: AtomicU32::new(0),
                gate_decisions: AtomicU32::new(0),
                workflow_started: AtomicU32::new(0),
                workflow_complete: AtomicU32::new(0),
                last_success: AtomicBool::new(false),
            }
        }
    }

    impl EventEmitter for SpyEmitter {
        fn emit_tier_completed(
            &self,
            _tier: u8,
            success: bool,
            _confidence: f64,
            _elapsed_ms: u64,
        ) {
            self.tier_completed.fetch_add(1, Ordering::Relaxed);
            self.last_success.store(success, Ordering::Relaxed);
        }

        fn emit_gate_decision(
            &self,
            _tier: u8,
            _decision: &str,
            _score: f64,
            _threshold: f64,
        ) {
            self.gate_decisions.fetch_add(1, Ordering::Relaxed);
        }

        fn emit_workflow_started(&self, _workflow_id: &str, _goal: &str) {
            self.workflow_started.fetch_add(1, Ordering::Relaxed);
        }

        fn emit_workflow_complete(
            &self,
            _workflow_id: &str,
            success: bool,
            _tiers_passed: u8,
        ) {
            self.workflow_complete.fetch_add(1, Ordering::Relaxed);
            self.last_success.store(success, Ordering::Relaxed);
        }
    }

    fn make_executor_with_spy() -> (TierExecutor, Arc<SpyEmitter>) {
        let exec = TierExecutor::new("/tmp/test-pipeline", 10);
        let spy = Arc::new(SpyEmitter::new());
        exec.set_event_emitter(Arc::clone(&spy) as Arc<dyn EventEmitter>);
        (exec, spy)
    }

    // --- NullEventEmitter ---

    #[test]
    fn null_emitter_all_methods_are_no_ops() {
        let e = NullEventEmitter;
        e.emit_tier_completed(1, true, 0.9, 500);
        e.emit_gate_decision(2, "pass", 0.87, 0.80);
        e.emit_workflow_started("wf-001", "add auth module");
        e.emit_workflow_complete("wf-001", true, 6);
        // Reaching here without panic satisfies the no-op contract.
    }

    #[test]
    fn null_emitter_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NullEventEmitter>();
    }

    // --- set_event_emitter ---

    #[test]
    fn set_event_emitter_replaces_existing() {
        let exec = TierExecutor::new("/tmp/test-pipeline", 0);
        exec.set_event_emitter(Arc::new(NullEventEmitter) as Arc<dyn EventEmitter>);
        let spy = Arc::new(SpyEmitter::new());
        exec.set_event_emitter(Arc::clone(&spy) as Arc<dyn EventEmitter>);
        // The spy is now active; creating a workflow should call workflow_started.
        let _ = exec.create_workflow_with_goal("s001".into(), "test goal");
        assert_eq!(spy.workflow_started.load(Ordering::Relaxed), 1);
    }

    // --- create_workflow_with_goal ---

    #[test]
    fn create_workflow_with_goal_emits_workflow_started() {
        let (exec, spy) = make_executor_with_spy();
        let _ = exec
            .create_workflow_with_goal("s001".into(), "deploy auth layer")
            .unwrap();
        assert_eq!(spy.workflow_started.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn create_workflow_emits_workflow_started_with_empty_goal() {
        let (exec, spy) = make_executor_with_spy();
        let _ = exec.create_workflow("s001".into()).unwrap();
        assert_eq!(spy.workflow_started.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn create_workflow_already_active_does_not_double_emit() {
        let (exec, spy) = make_executor_with_spy();
        let _ = exec.create_workflow("s001".into()).unwrap();
        // Second create must fail before emitting.
        let _ = exec.create_workflow("s002".into());
        // Should still be 1 — the second call failed, no second emission.
        assert_eq!(spy.workflow_started.load(Ordering::Relaxed), 1);
    }

    // --- emit_tier_completed ---

    #[test]
    fn execute_tier_emits_tier_completed_on_pass() {
        let (exec, spy) = make_executor_with_spy();
        let id = exec.create_workflow("s001".into()).unwrap();
        let state = exec.workflow_state(&id).unwrap();
        let mut ctx = WorkflowContext::new(state, 10);

        exec.execute_tier(Tier::T1Foundation, &mut ctx, conf(0.95), vec![])
            .unwrap();

        assert_eq!(spy.tier_completed.load(Ordering::Relaxed), 1);
        assert!(spy.last_success.load(Ordering::Relaxed));
    }

    #[test]
    fn run_pipeline_emits_six_tier_completed_events() {
        let (exec, spy) = make_executor_with_spy();
        let id = exec.create_workflow("s001".into()).unwrap();
        exec.run_pipeline(id, |_| (conf(0.95), vec![])).unwrap();

        assert_eq!(spy.tier_completed.load(Ordering::Relaxed), 6);
    }

    // --- emit_gate_decision ---

    #[test]
    fn execute_tier_emits_gate_decision_after_tier() {
        let (exec, spy) = make_executor_with_spy();
        let id = exec.create_workflow("s001".into()).unwrap();
        let state = exec.workflow_state(&id).unwrap();
        let mut ctx = WorkflowContext::new(state, 10);

        exec.execute_tier(Tier::T1Foundation, &mut ctx, conf(0.92), vec![])
            .unwrap();

        // Post-tier gate decision must be emitted.
        assert!(spy.gate_decisions.load(Ordering::Relaxed) >= 1);
    }

    #[test]
    fn run_pipeline_emits_gate_decisions_for_all_tiers() {
        let (exec, spy) = make_executor_with_spy();
        let id = exec.create_workflow("s001".into()).unwrap();
        exec.run_pipeline(id, |_| (conf(0.95), vec![])).unwrap();

        // Each of the 6 tiers emits at least one gate_decision.
        assert!(spy.gate_decisions.load(Ordering::Relaxed) >= 6);
    }

    // --- emit_workflow_complete ---

    #[test]
    fn run_pipeline_emits_workflow_complete_once() {
        let (exec, spy) = make_executor_with_spy();
        let id = exec.create_workflow("s001".into()).unwrap();
        exec.run_pipeline(id, |_| (conf(0.95), vec![])).unwrap();

        assert_eq!(spy.workflow_complete.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn run_pipeline_emits_workflow_complete_success_true_on_pass() {
        let (exec, spy) = make_executor_with_spy();
        let id = exec.create_workflow("s001".into()).unwrap();
        exec.run_pipeline(id, |_| (conf(0.95), vec![])).unwrap();

        assert!(spy.last_success.load(Ordering::Relaxed));
    }

    #[test]
    fn resume_from_checkpoint_emits_workflow_complete() {
        let exec = TierExecutor::new("/tmp/test-pipeline", 10);
        let spy = Arc::new(SpyEmitter::new());
        exec.set_event_emitter(Arc::clone(&spy) as Arc<dyn EventEmitter>);

        let id = exec.create_workflow("s001".into()).unwrap();

        // Manually complete T1 so resume starts from T2.
        exec.state_manager
            .update(&id, |s| {
                s.start_tier(TierId::new(1).unwrap())?;
                s.complete_tier(TierId::new(1).unwrap(), conf(0.92), &[])
                    .map(|_| ())?;
                Ok(())
            })
            .unwrap();

        exec.resume_from_checkpoint(id, |_| (conf(0.95), vec![]))
            .unwrap();

        assert_eq!(spy.workflow_complete.load(Ordering::Relaxed), 1);
    }

    // --- no-emitter path (executor without set_event_emitter) ---

    #[test]
    fn executor_without_emitter_runs_pipeline_normally() {
        // No emitter installed — pipeline must succeed without panicking.
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        let result = exec
            .run_pipeline(id, |_| (conf(0.95), vec![]))
            .unwrap();
        assert!(result.is_success());
    }

    #[test]
    fn executor_without_emitter_execute_tier_succeeds() {
        let exec = make_executor();
        let id = exec.create_workflow("s001".into()).unwrap();
        let state = exec.workflow_state(&id).unwrap();
        let mut ctx = WorkflowContext::new(state, 10);
        exec.execute_tier(Tier::T1Foundation, &mut ctx, conf(0.9), vec![])
            .unwrap();
    }

    // --- SpyEmitter Send + Sync ---

    #[test]
    fn spy_emitter_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SpyEmitter>();
    }
}
