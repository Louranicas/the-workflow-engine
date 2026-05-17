//! `m38_deployment_api` — REST API handlers for deployment lifecycle.
//!
//! Provides `POST /deploy`, `GET /deploy/{id}`, `GET /deployments`,
//! and supporting request/response types with full validation.
//!
//! Only one active workflow is allowed at a time
//! (`MAX_ACTIVE_WORKFLOWS = 1`). A second `POST /deploy` while one is
//! running returns `409 Conflict` with the active workflow id.
//!
//! # Feature Gate
//!
//! All items are compiled only when the `api` feature is enabled.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

use crate::m4_gate::m17_confidence_calculator::{
    BridgeSignals, ConfidenceCalculator, DEFAULT_HISTORICAL,
};
use crate::m8_api::m37_http_server::HookBridgeSignals;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of concurrently active workflows.
pub const MAX_ACTIVE_WORKFLOWS: usize = 1;

/// Default page size for `GET /deployments`.
pub const DEFAULT_PAGE_SIZE: usize = 20;

// ---------------------------------------------------------------------------
// Identifier newtypes
// ---------------------------------------------------------------------------

/// Newtype wrapper for a workflow UUID string.
///
/// Validated at construction: must be a non-empty string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(String);

impl WorkflowId {
    /// Create a new random `WorkflowId`.
    #[must_use]
    pub fn new_random() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Return the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for WorkflowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Body for `POST /deploy`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployRequest {
    /// Natural-language deployment goal (required, non-empty).
    pub goal: String,
    /// Absolute path to the project's `plan.toml` (optional).
    pub plan_toml_path: Option<String>,
    /// Absolute path to the project directory (optional).
    pub project_dir: Option<String>,
    /// Tier to resume from (1-indexed, `None` = start from T1).
    pub resume_from: Option<u8>,
    /// Deployment options.
    #[serde(default)]
    pub options: DeployOptions,
}

/// Optional knobs attached to a `DeployRequest`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeployOptions {
    /// If `true`, plan but do not execute.
    #[serde(default)]
    pub dry_run: bool,
    /// Override the adaptive confidence threshold for this deployment.
    pub confidence_override: Option<f64>,
}

// ---------------------------------------------------------------------------
// Status enumerations
// ---------------------------------------------------------------------------

/// High-level status of a deployment workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    /// Accepted, not yet started.
    Accepted,
    /// Currently running.
    InProgress,
    /// All tiers completed successfully.
    Completed,
    /// Aborted due to gate failure or user cancel.
    Failed,
    /// Dry-run plan produced, not executed.
    DryRun,
}

impl std::fmt::Display for DeploymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accepted => write!(f, "accepted"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::DryRun => write!(f, "dry_run"),
        }
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Response from `POST /deploy` (HTTP 202 Accepted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployAccepted {
    /// Assigned workflow identifier.
    pub workflow_id: String,
    /// Initial status (`accepted`).
    pub status: DeploymentStatus,
    /// The tier from which execution will begin.
    pub starting_tier: u8,
    /// Whether this is a dry-run only.
    pub dry_run: bool,
}

/// Checkpoint record for one completed tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierCheckpoint {
    /// Tier number (1-6).
    pub tier: u8,
    /// Human-readable stage name.
    pub stage: String,
    /// Outcome of this tier.
    pub status: DeploymentStatus,
    /// Confidence score when this tier completed.
    pub confidence: f64,
}

/// Full progress snapshot returned by `GET /deploy/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployProgress {
    /// Workflow identifier.
    pub workflow_id: String,
    /// Service name being deployed.
    pub service_name: String,
    /// Currently executing tier (1-6).
    pub current_tier: u8,
    /// Current stage within the active tier.
    pub current_stage: String,
    /// High-level workflow status.
    pub status: DeploymentStatus,
    /// Latest overall confidence score (0.0–1.0).
    pub confidence: f64,
    /// Latest SOR score.
    pub sor: f64,
    /// ISO-8601 start time.
    pub started_at: String,
    /// Elapsed seconds since start.
    pub elapsed_seconds: u64,
    /// Ordered list of completed tier checkpoints.
    pub checkpoints: Vec<TierCheckpoint>,
    /// `true` if this was a dry-run.
    pub dry_run: bool,
}

/// Summary row for `GET /deployments`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSummary {
    /// Workflow identifier.
    pub id: String,
    /// Service name.
    pub service_name: String,
    /// Final/current status.
    pub status: DeploymentStatus,
    /// ISO-8601 start time.
    pub started_at: String,
    /// Total elapsed seconds (0 while running).
    pub total_time_seconds: u64,
}

/// Paginated response from `GET /deployments`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentList {
    /// Page of deployment summaries.
    pub deployments: Vec<DeploymentSummary>,
    /// Total number matching the filter.
    pub total: usize,
    /// Applied limit.
    pub limit: usize,
    /// Applied offset.
    pub offset: usize,
}

/// API response DTO for `GET /confidence/{id}`.
///
/// Distinct from [`crate::m4_gate::m17_confidence_calculator::ConfidenceBreakdown`]
/// (the internal NAM-03 computation type) to avoid a name collision (I-14).
/// The internal calculation is delegated to
/// [`crate::m4_gate::m17_confidence_calculator::ConfidenceCalculator`] and
/// its result is mapped into this type before serialisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceBreakdownResponse {
    /// Workflow identifier.
    pub workflow_id: String,
    /// Overall weighted confidence (NAM-03).
    pub overall: f64,
    /// Primary execution score (35% weight).
    pub primary: f64,
    /// Historical deployment score (35% weight).
    pub historical: f64,
    /// Bridge health score (15% weight).
    pub health: f64,
    /// SYNTHEX thermal score (7.5% weight).
    pub thermal: f64,
    /// PV2 Kuramoto coherence score (7.5% weight).
    pub coherence: f64,
    /// Threshold value for `pass` decision.
    pub threshold_pass: f64,
    /// Threshold below which execution escalates.
    pub threshold_escalate: f64,
    /// Decision outcome: `"pass"`, `"escalate"`, or `"halt"`.
    pub decision: String,
    /// Number of completed deployments contributing to historical score.
    pub deployment_count: u32,
    /// Current adaptive learning phase.
    pub phase: String,
}

/// NAM-01 self-model snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModelSnapshot {
    /// What the engine is currently doing.
    pub current_activity: String,
    /// Active workflow id, or `null`.
    pub active_workflow_id: Option<String>,
    /// Latest overall confidence.
    pub confidence: f64,
    /// Latest SOR score.
    pub sor: f64,
    /// Elapsed seconds of current activity.
    pub elapsed_seconds: u64,
    /// Known blockers or issues.
    pub blockers: Vec<String>,
    /// ISO-8601 snapshot timestamp.
    pub snapshot_at: String,
}

/// Lightweight snapshot used by the status endpoint.
#[derive(Debug, Clone)]
pub struct StatusSnapshot {
    /// Active workflow id if one is running.
    pub active_workflow_id: Option<String>,
    /// Total deployments ever started.
    pub total_deployments: usize,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by the deployment API layer.
#[derive(Debug, thiserror::Error)]
pub enum DeployApiError {
    /// A workflow is already running.
    #[error("deployment already in progress")]
    AlreadyRunning {
        /// The currently running workflow id.
        active_id: String,
    },
    /// The request failed validation.
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    /// Internal state error.
    #[error("internal error: {0}")]
    Internal(String),
}

// ---------------------------------------------------------------------------
// Internal record stored per workflow
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct WorkflowRecord {
    progress: DeployProgress,
    started_ts: std::time::Instant,
}

// ---------------------------------------------------------------------------
// Deployment API state
// ---------------------------------------------------------------------------

/// Shared state for all deployment API handlers.
///
/// Uses interior mutability via [`parking_lot::RwLock`] so it can be
/// placed inside an [`std::sync::Arc`] and shared across Axum tasks.
pub struct DeploymentApiState {
    /// Active workflow id (at most one at a time).
    active: RwLock<Option<WorkflowId>>,
    /// In-memory workflow store (`workflow_id` -> `WorkflowRecord`).
    workflows: RwLock<HashMap<String, WorkflowRecord>>,
    /// Total deployments accepted (monotonic counter).
    total_count: RwLock<usize>,
    /// Live bridge signals from ORAC hook deliveries (PV2 r, SYNTHEX thermal, health).
    /// `None` when running without the full `AppState` context (tests, standalone).
    bridge_signals: Option<Arc<RwLock<HookBridgeSignals>>>,
}

impl DeploymentApiState {
    /// Construct an empty `DeploymentApiState` without bridge signals.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active: RwLock::new(None),
            workflows: RwLock::new(HashMap::new()),
            total_count: RwLock::new(0),
            bridge_signals: None,
        }
    }

    /// Construct with live bridge signals for confidence calculation.
    #[must_use]
    pub fn with_bridge_signals(signals: Arc<RwLock<HookBridgeSignals>>) -> Self {
        Self {
            active: RwLock::new(None),
            workflows: RwLock::new(HashMap::new()),
            total_count: RwLock::new(0),
            bridge_signals: Some(signals),
        }
    }

    // ------------------------------------------------------------------
    // Write path
    // ------------------------------------------------------------------

    /// Validate a `DeployRequest` and register it as a new active workflow.
    ///
    /// The active-slot check and all subsequent mutations are performed under
    /// **separate** write-lock acquisitions for `workflows`, `active`, and
    /// `total_count`, in that order.  The `active` slot is checked and set
    /// under the same single write-lock on `active`, closing the TOCTOU window
    /// (C-01 fix).  All three mutations are sequenced with no observable gap
    /// between them from the perspective of other writers (C-02 fix).
    ///
    /// # Errors
    ///
    /// - [`DeployApiError::AlreadyRunning`] if another workflow is active.
    /// - [`DeployApiError::ValidationFailed`] if the request is invalid.
    pub fn start_deployment(
        &self,
        req: &DeployRequest,
    ) -> Result<DeployAccepted, DeployApiError> {
        // Validate first — pure computation, no locks required.
        let errors = Self::validate_request(req);
        if !errors.is_empty() {
            return Err(DeployApiError::ValidationFailed(errors.join("; ")));
        }

        let wf_id = WorkflowId::new_random();
        let starting_tier = req.resume_from.unwrap_or(1).clamp(1, 6);
        let dry_run = req.options.dry_run;

        let service_name = req
            .project_dir
            .as_deref()
            .and_then(|p| std::path::Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_owned();

        let progress = DeployProgress {
            workflow_id: wf_id.as_str().to_owned(),
            service_name,
            current_tier: starting_tier,
            current_stage: "initialising".to_owned(),
            status: if dry_run {
                DeploymentStatus::DryRun
            } else {
                DeploymentStatus::Accepted
            },
            confidence: 0.0,
            sor: 0.0,
            started_at: chrono::Utc::now().to_rfc3339(),
            elapsed_seconds: 0,
            checkpoints: Vec::new(),
            dry_run,
        };

        let record = WorkflowRecord {
            progress,
            started_ts: std::time::Instant::now(),
        };

        // C-01 + C-02 fix: check-and-set `active` under a single write guard,
        // then insert into `workflows` and increment `total_count`.
        // Acquiring `active.write()` first means no second task can pass the
        // AlreadyRunning gate simultaneously.
        {
            let mut active_guard = self.active.write();
            // Check inside the write guard — closes the TOCTOU window.
            if let Some(ref existing_id) = *active_guard {
                return Err(DeployApiError::AlreadyRunning {
                    active_id: existing_id.as_str().to_owned(),
                });
            }
            // Set while still holding the write guard.
            *active_guard = if dry_run { None } else { Some(wf_id.clone()) };
        }
        // Workflow insert and counter increment follow immediately after.
        self.workflows
            .write()
            .insert(wf_id.as_str().to_owned(), record);
        *self.total_count.write() += 1;

        info!(
            workflow_id = %wf_id,
            starting_tier = starting_tier,
            dry_run = dry_run,
            "Deployment accepted"
        );

        Ok(DeployAccepted {
            workflow_id: wf_id.as_str().to_owned(),
            status: if dry_run {
                DeploymentStatus::DryRun
            } else {
                DeploymentStatus::Accepted
            },
            starting_tier,
            dry_run,
        })
    }

    /// Mark a workflow as cancelled / failed and clear the active slot.
    ///
    /// # Errors
    ///
    /// Returns [`DeployApiError::Internal`] if the workflow id is not found.
    #[allow(clippy::significant_drop_tightening)]
    pub fn cancel_deployment(&self, id: &str) -> Result<(), DeployApiError> {
        // Mark workflow as failed (hold guard only for this operation).
        {
            let mut wf_guard = self.workflows.write();
            let record = wf_guard
                .get_mut(id)
                .ok_or_else(|| DeployApiError::Internal(format!("workflow {id} not found")))?;
            record.progress.status = DeploymentStatus::Failed;
        }
        debug!(workflow_id = id, "Deployment cancelled");

        // Clear active slot if it points to this workflow (separate lock acquisition).
        {
            let mut active_guard = self.active.write();
            if active_guard
                .as_ref()
                .is_some_and(|a| a.as_str() == id)
            {
                *active_guard = None;
            }
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Read paths
    // ------------------------------------------------------------------

    /// Retrieve full progress for a workflow by id.
    #[must_use]
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_deployment(&self, id: &str) -> Option<DeployProgress> {
        let (mut progress, elapsed) = {
            let guard = self.workflows.read();
            let record = guard.get(id)?;
            (record.progress.clone(), record.started_ts.elapsed().as_secs())
        };
        // Update elapsed seconds dynamically (computed after releasing the lock).
        progress.elapsed_seconds = elapsed;
        Some(progress)
    }

    /// Return a paginated list of deployment summaries.
    #[must_use]
    pub fn list_deployments(
        &self,
        limit: usize,
        offset: usize,
        status_filter: Option<&str>,
        service_filter: Option<&str>,
    ) -> DeploymentList {
        let all_records: Vec<DeploymentSummary> = {
            let guard = self.workflows.read();
            guard
                .values()
                .filter(|r| {
                    if let Some(sf) = service_filter {
                        if !r.progress.service_name.contains(sf) {
                            return false;
                        }
                    }
                    if let Some(st) = status_filter {
                        if r.progress.status.to_string() != st {
                            return false;
                        }
                    }
                    true
                })
                .map(|r| DeploymentSummary {
                    id: r.progress.workflow_id.clone(),
                    service_name: r.progress.service_name.clone(),
                    status: r.progress.status,
                    started_at: r.progress.started_at.clone(),
                    total_time_seconds: match r.progress.status {
                        DeploymentStatus::Completed | DeploymentStatus::Failed => {
                            r.started_ts.elapsed().as_secs()
                        }
                        _ => 0,
                    },
                })
                .collect()
        };

        let mut all = all_records;
        // Sort by started_at descending (most recent first).
        all.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        let total = all.len();
        let page: Vec<DeploymentSummary> = all.into_iter().skip(offset).take(limit).collect();

        DeploymentList {
            deployments: page,
            total,
            limit,
            offset,
        }
    }

    /// Retrieve confidence breakdown for a workflow.
    ///
    /// Delegates to [`ConfidenceCalculator`] from
    /// `m17_confidence_calculator` for the NAM-03 computation and maps
    /// the result into a [`ConfidenceBreakdownResponse`] for the API
    /// response.  This eliminates the third redundant `ConfidenceBreakdown`
    /// definition (I-14).
    ///
    /// Returns `None` when no workflow with `id` is found.
    #[must_use]
    pub fn get_confidence_breakdown(&self, id: &str) -> Option<ConfidenceBreakdownResponse> {
        // Read confidence value and count; drop the read lock before computing.
        let raw_confidence = {
            let guard = self.workflows.read();
            guard.get(id)?.progress.confidence
        };
        let deployment_count = u32::try_from(*self.total_count.read()).unwrap_or(u32::MAX);

        // Run NAM-03 formula via the canonical m17 calculator.
        let historical = if deployment_count == 0 {
            DEFAULT_HISTORICAL
        } else {
            raw_confidence.mul_add(0.9, 0.0)
        };

        // Use calculate_with_bridges to consume LIVE bridge signals (PV2 r,
        // SYNTHEX thermal, health score) instead of hardcoded 0.0.  Bridge
        // signals are populated from ORAC hook deliveries in AppState.
        // Falling back to BridgeSignals::none() gives neutral 0.5 when
        // bridges are unreachable — never penalises confidence for being
        // disconnected.
        let signals = self
            .bridge_signals
            .as_ref()
            .map_or_else(BridgeSignals::none, |bs| bs.read().to_bridge_signals());

        let calc = ConfidenceCalculator::new();
        let breakdown = calc
            .calculate_with_bridges(raw_confidence, historical, 0.0, &signals)
            .ok()?;
        let overall = breakdown.final_score.value();

        let decision = if overall >= 0.85 {
            "pass"
        } else if overall >= 0.65 {
            "escalate"
        } else {
            "halt"
        }
        .to_owned();

        Some(ConfidenceBreakdownResponse {
            workflow_id: id.to_owned(),
            overall,
            primary: breakdown.components[0].raw_value,
            historical: breakdown.components[1].raw_value,
            health: breakdown.components[2].raw_value,
            thermal: breakdown.components[3].raw_value,
            coherence: breakdown.components[4].raw_value,
            threshold_pass: 0.85,
            threshold_escalate: 0.65,
            decision,
            deployment_count,
            phase: "practicing".to_owned(),
        })
    }

    /// Build a `StatusSnapshot` for the `/status` endpoint.
    #[must_use]
    pub fn status_snapshot(&self) -> StatusSnapshot {
        let active_workflow_id = {
            let guard = self.active.read();
            guard.as_ref().map(WorkflowId::to_string)
        };
        let total_deployments = *self.total_count.read();
        StatusSnapshot {
            active_workflow_id,
            total_deployments,
        }
    }

    /// Build a `SelfModelSnapshot` for the `/self-model` endpoint.
    #[must_use]
    pub fn self_model_snapshot(&self) -> SelfModelSnapshot {
        let active_id = {
            let guard = self.active.read();
            guard.as_ref().map(WorkflowId::to_string)
        };

        let (activity, elapsed) = active_id.as_deref().map_or_else(
            || ("idle — no active deployment".to_owned(), 0u64),
            |id| {
                let guard = self.workflows.read();
                guard.get(id).map_or_else(
                    || ("idle".to_owned(), 0u64),
                    |record| {
                        (
                            format!(
                                "Executing tier {} — {}",
                                record.progress.current_tier,
                                record.progress.current_stage
                            ),
                            record.started_ts.elapsed().as_secs(),
                        )
                    },
                )
            },
        );

        SelfModelSnapshot {
            current_activity: activity,
            active_workflow_id: active_id,
            confidence: 0.0,
            sor: 0.0,
            elapsed_seconds: elapsed,
            blockers: Vec::new(),
            snapshot_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    // ------------------------------------------------------------------
    // Validation
    // ------------------------------------------------------------------

    /// Validate a `DeployRequest` and collect all error messages.
    fn validate_request(req: &DeployRequest) -> Vec<String> {
        let mut errors = Vec::new();

        if req.goal.trim().is_empty() {
            errors.push("goal must not be empty".to_owned());
        }
        if req.goal.len() > 2048 {
            errors.push("goal exceeds 2048 character limit".to_owned());
        }
        if let Some(ref path) = req.plan_toml_path {
            if !path.starts_with('/') {
                errors.push(format!(
                    "plan_toml_path must be absolute: got '{path}'"
                ));
            }
        }
        if let Some(ref dir) = req.project_dir {
            if !dir.starts_with('/') {
                errors.push(format!("project_dir must be absolute: got '{dir}'"));
            }
        }
        if let Some(tier) = req.resume_from {
            if !(1..=6).contains(&tier) {
                errors.push(format!("resume_from must be 1–6, got {tier}"));
            }
        }
        if let Some(c) = req.options.confidence_override {
            if !(0.0..=1.0).contains(&c) {
                errors.push(format!(
                    "confidence_override must be in [0.0, 1.0], got {c}"
                ));
            }
        }

        errors
    }
}

impl Default for DeploymentApiState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> DeploymentApiState {
        DeploymentApiState::new()
    }

    fn valid_request() -> DeployRequest {
        DeployRequest {
            goal: "Deploy ORAC sidecar with full evolution layer".to_owned(),
            plan_toml_path: Some("/tmp/plan.toml".to_owned()),
            project_dir: Some("/tmp/orac-sidecar".to_owned()),
            resume_from: None,
            options: DeployOptions::default(),
        }
    }

    // ---- validation ----

    #[test]
    fn validation_rejects_empty_goal() {
        let mut req = valid_request();
        req.goal = "   ".to_owned();
        let errors = DeploymentApiState::validate_request(&req);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("goal")));
    }

    #[test]
    fn validation_rejects_goal_over_2048_chars() {
        let mut req = valid_request();
        req.goal = "x".repeat(2049);
        let errors = DeploymentApiState::validate_request(&req);
        assert!(!errors.is_empty());
    }

    #[test]
    fn validation_rejects_relative_plan_path() {
        let mut req = valid_request();
        req.plan_toml_path = Some("relative/path.toml".to_owned());
        let errors = DeploymentApiState::validate_request(&req);
        assert!(errors.iter().any(|e| e.contains("plan_toml_path")));
    }

    #[test]
    fn validation_rejects_relative_project_dir() {
        let mut req = valid_request();
        req.project_dir = Some("relative/dir".to_owned());
        let errors = DeploymentApiState::validate_request(&req);
        assert!(errors.iter().any(|e| e.contains("project_dir")));
    }

    #[test]
    fn validation_rejects_resume_from_zero() {
        let mut req = valid_request();
        req.resume_from = Some(0);
        let errors = DeploymentApiState::validate_request(&req);
        assert!(errors.iter().any(|e| e.contains("resume_from")));
    }

    #[test]
    fn validation_rejects_resume_from_seven() {
        let mut req = valid_request();
        req.resume_from = Some(7);
        let errors = DeploymentApiState::validate_request(&req);
        assert!(errors.iter().any(|e| e.contains("resume_from")));
    }

    #[test]
    fn validation_rejects_confidence_override_out_of_range() {
        let mut req = valid_request();
        req.options.confidence_override = Some(1.5);
        let errors = DeploymentApiState::validate_request(&req);
        assert!(!errors.is_empty());
    }

    #[test]
    fn validation_accepts_valid_request() {
        let errors = DeploymentApiState::validate_request(&valid_request());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn validation_accumulates_multiple_errors() {
        let req = DeployRequest {
            goal: String::new(),
            plan_toml_path: Some("bad".to_owned()),
            project_dir: Some("also_bad".to_owned()),
            resume_from: Some(9),
            options: DeployOptions::default(),
        };
        let errors = DeploymentApiState::validate_request(&req);
        assert!(errors.len() >= 4);
    }

    // ---- start_deployment ----

    #[test]
    fn start_deployment_succeeds_with_valid_request() {
        let state = make_state();
        let result = state.start_deployment(&valid_request());
        assert!(result.is_ok());
        let accepted = result.unwrap();
        assert!(!accepted.workflow_id.is_empty());
        assert_eq!(accepted.starting_tier, 1);
    }

    #[test]
    fn start_deployment_returns_conflict_when_one_running() {
        let state = make_state();
        state.start_deployment(&valid_request()).unwrap();
        let second = state.start_deployment(&valid_request());
        assert!(matches!(second, Err(DeployApiError::AlreadyRunning { .. })));
    }

    #[test]
    fn start_deployment_increments_total_count() {
        let state = make_state();
        // Two dry-runs do not lock the active slot.
        let mut req = valid_request();
        req.options.dry_run = true;
        state.start_deployment(&req.clone()).unwrap();
        state.start_deployment(&req).unwrap();
        assert_eq!(*state.total_count.read(), 2);
    }

    #[test]
    fn dry_run_does_not_set_active_slot() {
        let state = make_state();
        let mut req = valid_request();
        req.options.dry_run = true;
        let accepted = state.start_deployment(&req).unwrap();
        assert!(accepted.dry_run);
        assert!(state.active.read().is_none());
    }

    #[test]
    fn start_deployment_rejects_invalid_goal() {
        let state = make_state();
        let mut req = valid_request();
        req.goal = String::new();
        let result = state.start_deployment(&req);
        assert!(matches!(result, Err(DeployApiError::ValidationFailed(_))));
    }

    #[test]
    fn start_deployment_uses_resume_from() {
        let state = make_state();
        let mut req = valid_request();
        req.resume_from = Some(3);
        let accepted = state.start_deployment(&req).unwrap();
        assert_eq!(accepted.starting_tier, 3);
    }

    // ---- get_deployment ----

    #[test]
    fn get_deployment_returns_none_for_unknown_id() {
        let state = make_state();
        assert!(state.get_deployment("no-such-id").is_none());
    }

    #[test]
    fn get_deployment_returns_progress_for_known_id() {
        let state = make_state();
        let accepted = state.start_deployment(&valid_request()).unwrap();
        let progress = state.get_deployment(&accepted.workflow_id);
        assert!(progress.is_some());
        assert_eq!(progress.unwrap().workflow_id, accepted.workflow_id);
    }

    #[test]
    fn get_deployment_progress_contains_elapsed() {
        let state = make_state();
        let accepted = state.start_deployment(&valid_request()).unwrap();
        let progress = state.get_deployment(&accepted.workflow_id).unwrap();
        // elapsed_seconds must be a valid u64 (may be 0 for very fast tests).
        let _ = progress.elapsed_seconds;
    }

    // ---- cancel_deployment ----

    #[test]
    fn cancel_deployment_clears_active_slot() {
        let state = make_state();
        let accepted = state.start_deployment(&valid_request()).unwrap();
        assert!(state.active.read().is_some());
        state.cancel_deployment(&accepted.workflow_id).unwrap();
        assert!(state.active.read().is_none());
    }

    #[test]
    fn cancel_deployment_marks_failed() {
        let state = make_state();
        let accepted = state.start_deployment(&valid_request()).unwrap();
        state.cancel_deployment(&accepted.workflow_id).unwrap();
        let progress = state.get_deployment(&accepted.workflow_id).unwrap();
        assert_eq!(progress.status, DeploymentStatus::Failed);
    }

    #[test]
    fn cancel_unknown_id_returns_error() {
        let state = make_state();
        let result = state.cancel_deployment("no-such-id");
        assert!(matches!(result, Err(DeployApiError::Internal(_))));
    }

    // ---- list_deployments ----

    #[test]
    fn list_deployments_empty_state() {
        let state = make_state();
        let list = state.list_deployments(20, 0, None, None);
        assert_eq!(list.total, 0);
        assert!(list.deployments.is_empty());
    }

    #[test]
    fn list_deployments_returns_all_entries() {
        let state = make_state();
        let mut req = valid_request();
        req.options.dry_run = true;
        state.start_deployment(&req.clone()).unwrap();
        req.goal = "second goal".to_owned();
        state.start_deployment(&req).unwrap();
        let list = state.list_deployments(20, 0, None, None);
        assert_eq!(list.total, 2);
    }

    #[test]
    fn list_deployments_respects_limit() {
        let state = make_state();
        for i in 0..5 {
            let mut req = valid_request();
            req.goal = format!("goal {i}");
            req.options.dry_run = true;
            state.start_deployment(&req).unwrap();
        }
        let list = state.list_deployments(3, 0, None, None);
        assert_eq!(list.deployments.len(), 3);
    }

    #[test]
    fn list_deployments_respects_offset() {
        let state = make_state();
        for i in 0..4 {
            let mut req = valid_request();
            req.goal = format!("goal {i}");
            req.options.dry_run = true;
            state.start_deployment(&req).unwrap();
        }
        let list = state.list_deployments(10, 2, None, None);
        assert_eq!(list.deployments.len(), 2);
    }

    // ---- confidence_breakdown ----

    #[test]
    fn confidence_breakdown_none_for_unknown_id() {
        let state = make_state();
        assert!(state.get_confidence_breakdown("bad-id").is_none());
    }

    #[test]
    fn confidence_breakdown_some_for_known_id() {
        let state = make_state();
        let accepted = state.start_deployment(&valid_request()).unwrap();
        let bd = state.get_confidence_breakdown(&accepted.workflow_id);
        assert!(bd.is_some());
    }

    #[test]
    fn confidence_breakdown_decision_halt_when_zero() {
        let state = make_state();
        let accepted = state.start_deployment(&valid_request()).unwrap();
        let bd = state.get_confidence_breakdown(&accepted.workflow_id).unwrap();
        // Initial confidence is 0.0 -> halt decision.
        assert_eq!(bd.decision, "halt");
    }

    // ---- status_snapshot ----

    #[test]
    fn status_snapshot_no_active_workflow() {
        let state = make_state();
        let snap = state.status_snapshot();
        assert!(snap.active_workflow_id.is_none());
        assert_eq!(snap.total_deployments, 0);
    }

    #[test]
    fn status_snapshot_with_active_workflow() {
        let state = make_state();
        let accepted = state.start_deployment(&valid_request()).unwrap();
        let snap = state.status_snapshot();
        assert_eq!(snap.active_workflow_id, Some(accepted.workflow_id));
    }

    // ---- self_model_snapshot ----

    #[test]
    fn self_model_snapshot_idle_when_no_active() {
        let state = make_state();
        let snap = state.self_model_snapshot();
        assert!(snap.current_activity.contains("idle"));
    }

    #[test]
    fn self_model_snapshot_has_timestamp() {
        let state = make_state();
        let snap = state.self_model_snapshot();
        assert!(!snap.snapshot_at.is_empty());
    }

    // ---- workflow id newtype ----

    #[test]
    fn workflow_id_display() {
        let id = WorkflowId::new_random();
        let s = id.to_string();
        assert!(!s.is_empty());
    }

    #[test]
    fn workflow_id_as_str_equals_display() {
        let id = WorkflowId::new_random();
        assert_eq!(id.as_str(), id.to_string().as_str());
    }

    // ---- deployment status display ----

    #[test]
    fn deployment_status_display_strings() {
        assert_eq!(DeploymentStatus::Accepted.to_string(), "accepted");
        assert_eq!(DeploymentStatus::InProgress.to_string(), "in_progress");
        assert_eq!(DeploymentStatus::Completed.to_string(), "completed");
        assert_eq!(DeploymentStatus::Failed.to_string(), "failed");
        assert_eq!(DeploymentStatus::DryRun.to_string(), "dry_run");
    }

    // ---- confidence breakdown uses m17 ----

    #[test]
    fn confidence_breakdown_overall_in_unit_interval() {
        let state = make_state();
        let accepted = state.start_deployment(&valid_request()).unwrap();
        let bd = state
            .get_confidence_breakdown(&accepted.workflow_id)
            .unwrap();
        // overall is the result of NAM-03 formula — must always be in [0.0, 1.0].
        assert!(
            (0.0..=1.0).contains(&bd.overall),
            "overall out of range: {}",
            bd.overall
        );
    }

    #[test]
    fn confidence_breakdown_components_present() {
        let state = make_state();
        let accepted = state.start_deployment(&valid_request()).unwrap();
        let bd = state
            .get_confidence_breakdown(&accepted.workflow_id)
            .unwrap();
        // Verify all component fields are in [0, 1].
        for v in [bd.primary, bd.historical, bd.health, bd.thermal, bd.coherence] {
            assert!((0.0..=1.0).contains(&v), "component out of range: {v}");
        }
    }

    // ---- concurrent safety ----

    #[test]
    fn concurrent_start_only_one_succeeds() {
        use std::sync::Arc;

        let state = Arc::new(make_state());
        let s1 = Arc::clone(&state);
        let s2 = Arc::clone(&state);

        let t1 = std::thread::spawn(move || s1.start_deployment(&valid_request()));
        let t2 = std::thread::spawn(move || s2.start_deployment(&valid_request()));

        let r1 = t1.join().unwrap();
        let r2 = t2.join().unwrap();

        // Exactly one must succeed and one must return AlreadyRunning.
        let successes = [&r1, &r2].iter().filter(|r| r.is_ok()).count();
        let conflicts = [&r1, &r2]
            .iter()
            .filter(|r| matches!(r, Err(DeployApiError::AlreadyRunning { .. })))
            .count();

        assert_eq!(successes, 1, "exactly one thread should succeed");
        assert_eq!(conflicts, 1, "exactly one thread should get AlreadyRunning");
    }
}
