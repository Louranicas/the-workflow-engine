//! Wave 3 — active enforcement: reads [`DivergenceReport`] rows, applies
//! cooldown, writes audit records, and dispatches rollback or proposal.
//!
//! # Design
//!
//! The enforcer polls `state.db.divergence_reports` every 10 s for rows with
//! `severity IN ('HIGH', 'CRITICAL')` that have no corresponding
//! `enforcement_actions` row. For each unseen HIGH divergence it writes an
//! `action=rollback_proposed` row and writes an Atuin KV entry for human
//! approval. For CRITICAL it writes `action=rollback_auto` and invokes the
//! rollback wrapper.
//!
//! # Safety invariants
//!
//! 1. **Audit-first:** every action is written to `enforcement_actions` BEFORE
//!    invoking `forge --rollback`. A failed write aborts the rollback.
//! 2. **CRITICAL-only auto-rollback:** HIGH severity is propose-only by default.
//!    Exception: `soak_health_degrade` at HIGH auto-rolls back (live metric,
//!    not plan-drift heuristic).
//! 3. **Cooldown:** a service cannot be rolled back more than once per
//!    `COOLDOWN_SECS` (default 300 s). Prevents rollback-loop storms.
//! 4. **Enabled gate:** set `CONDUCTOR_ENFORCEMENT_ENABLED=1` to activate.
//!    Without the env var, all paths return [`EnforcerAction::NoOp`].
//!
//! # Trigger sources (spec §Wave 3)
//!
//! | Source | kind | Severity | Action |
//! |---|---|---|---|
//! | Watcher | `trait_violation` | CRITICAL | auto-rollback |
//! | Zen | `phase_skip` | HIGH | propose-only |
//! | soak | `soak_health_degrade` | HIGH | auto-rollback |
//! | bridge-contract | `bridge_contract_mismatch` | CRITICAL | auto-rollback |
//!
//! # Wave 3 advanced (POVM Hebbian reward)
//!
//! Productive drift should be *reinforced*, not punished. Gated on R13 expiry
//! (2026-05-19) and Watcher Proposer active state.
//!
// TODO(wave3-advanced): shadow-test path → POVM weight ±0.1 reward/punish.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info, warn};

use crate::state::{DivergenceReport, Severity, StateDb};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default rollback cooldown: 300 seconds per service.
///
/// A service cannot be rolled back more than once within this window, preventing
/// rollback-loops from transient probe noise.
pub const COOLDOWN_SECS: u64 = 300;

// ---------------------------------------------------------------------------
// EnforcerAction
// ---------------------------------------------------------------------------

/// The action the enforcer chose for a given divergence report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcerAction {
    /// Automatic rollback initiated.
    RollbackAuto,
    /// Rollback proposed to human operator; awaiting approval.
    RollbackProposed,
    /// No action taken (severity below threshold or enforcement disabled).
    NoOp,
}

impl EnforcerAction {
    /// Returns the canonical string used in `enforcement_actions.action`.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RollbackAuto => "rollback_auto",
            Self::RollbackProposed => "rollback_proposed",
            Self::NoOp => "no_op",
        }
    }
}

// ---------------------------------------------------------------------------
// TriggerSource — spec §Wave 3 trigger sources
// ---------------------------------------------------------------------------

/// Identifies the tool/agent that produced a divergence report.
///
/// The trigger source determines whether a HIGH-severity report should be
/// auto-rolled-back or proposed-only. The spec mandates:
/// - `soak_health_degrade` at HIGH → auto-rollback (service behaviour metric).
/// - `phase_skip` at HIGH → propose-only (plan drift, not live failure).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerSource {
    /// Watcher `trait_violation` — CRITICAL.
    WatcherTraitViolation,
    /// Zen `phase_skip` — HIGH, propose-only.
    ZenPhaseSkip,
    /// Soak health degradation >20% — HIGH, auto-rollback.
    SoakHealthDegrade,
    /// Bridge-contract wire-format mismatch — CRITICAL.
    BridgeContractMismatch,
    /// Any other kind not specifically mapped.
    Other(String),
}

impl TriggerSource {
    /// Classifies a divergence report kind string into a [`TriggerSource`].
    #[must_use]
    pub fn from_kind(kind: &str) -> Self {
        match kind {
            "trait_violation" => Self::WatcherTraitViolation,
            "phase_skip" => Self::ZenPhaseSkip,
            "soak_health_degrade" => Self::SoakHealthDegrade,
            "bridge_contract_mismatch" => Self::BridgeContractMismatch,
            other => Self::Other(other.to_owned()),
        }
    }

    /// Returns the `trigger_source` label written to `enforcement_actions`.
    #[must_use]
    pub fn as_label(&self) -> &str {
        match self {
            Self::WatcherTraitViolation => "watcher_trait_violation",
            Self::ZenPhaseSkip => "zen_phase_skip",
            Self::SoakHealthDegrade => "soak",
            Self::BridgeContractMismatch => "bridge_contract",
            Self::Other(s) => s.as_str(),
        }
    }

    /// Returns `true` if this trigger allows auto-rollback at HIGH severity.
    ///
    /// The spec allows `soak_health_degrade` at HIGH to auto-rollback because
    /// a degraded health metric is a direct live signal, not a plan-drift
    /// heuristic. `phase_skip` is propose-only.
    #[must_use]
    pub fn high_auto_rollback(&self) -> bool {
        matches!(self, Self::SoakHealthDegrade)
    }
}

// ---------------------------------------------------------------------------
// CooldownTracker
// ---------------------------------------------------------------------------

/// Per-service rollback cooldown tracker.
///
/// Prevents rollback-loop storms by refusing to issue a second rollback for
/// the same service within [`COOLDOWN_SECS`].
pub struct CooldownTracker {
    last_rollback: HashMap<String, Instant>,
    cooldown: Duration,
}

impl CooldownTracker {
    /// Creates a tracker with the given cooldown window.
    #[must_use]
    pub fn new(cooldown_secs: u64) -> Self {
        Self {
            last_rollback: HashMap::new(),
            cooldown: Duration::from_secs(cooldown_secs),
        }
    }

    /// Returns `true` if a rollback for `service` is permitted right now.
    #[must_use]
    pub fn may_rollback(&self, service: &str) -> bool {
        self.last_rollback
            .get(service)
            .map_or(true, |t| t.elapsed() >= self.cooldown)
    }

    /// Records that a rollback was issued for `service` at this moment.
    pub fn record_rollback(&mut self, service: &str) {
        self.last_rollback.insert(service.to_owned(), Instant::now());
    }
}

// ---------------------------------------------------------------------------
// Enforcer — pure classification (no I/O)
// ---------------------------------------------------------------------------

/// Classifies a divergence report into an [`EnforcerAction`].
///
/// The `enabled` flag gates all non-`NoOp` actions. Set `enabled = false` in
/// Wave 1/2 environments to prevent accidental rollbacks; set to `true` when
/// Wave 3 is deployed (`CONDUCTOR_ENFORCEMENT_ENABLED=1`).
pub struct Enforcer {
    /// When `false`, all calls return [`EnforcerAction::NoOp`].
    pub enabled: bool,
}

impl Enforcer {
    /// Creates a new enforcer.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Evaluates a divergence report and returns the appropriate action.
    ///
    /// The trigger source is derived from `report.kind` and may elevate a
    /// HIGH-severity `soak_health_degrade` report to auto-rollback.
    ///
    /// | Severity | Trigger | Action (enabled) | Action (disabled) |
    /// |---|---|---|---|
    /// | CRITICAL | any | `RollbackAuto` | `NoOp` |
    /// | HIGH | soak | `RollbackAuto` | `NoOp` |
    /// | HIGH | other | `RollbackProposed` | `NoOp` |
    /// | MED / LOW | any | `NoOp` | `NoOp` |
    #[must_use]
    pub fn evaluate(&self, report: &DivergenceReport) -> EnforcerAction {
        if !self.enabled {
            debug!(
                kind = %report.kind,
                severity = %report.severity,
                "enforcer disabled — NoOp"
            );
            return EnforcerAction::NoOp;
        }
        let trigger = TriggerSource::from_kind(&report.kind);
        match report.severity {
            Severity::Critical => EnforcerAction::RollbackAuto,
            Severity::High => {
                if trigger.high_auto_rollback() {
                    EnforcerAction::RollbackAuto
                } else {
                    EnforcerAction::RollbackProposed
                }
            }
            Severity::Med | Severity::Low => EnforcerAction::NoOp,
        }
    }

    /// Processes a batch of divergence reports and returns `(report, action)` pairs.
    ///
    /// Reports with `action=NoOp` are excluded from the output.
    #[must_use]
    pub fn evaluate_batch<'a>(
        &self,
        reports: &'a [DivergenceReport],
    ) -> Vec<(&'a DivergenceReport, EnforcerAction)> {
        reports
            .iter()
            .filter_map(|r| {
                let action = self.evaluate(r);
                if action == EnforcerAction::NoOp {
                    None
                } else {
                    Some((r, action))
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// EnforcerError
// ---------------------------------------------------------------------------

/// Errors produced by enforcement operations.
#[derive(Debug, thiserror::Error)]
pub enum EnforcerError {
    /// State database write failed.
    #[error("enforcer: db error: {0}")]
    Db(#[from] crate::state::StateError),
    /// JSON serialisation failure.
    #[error("enforcer: serde error: {0}")]
    Serde(#[from] serde_json::Error),
    /// Rollback was blocked by the per-service cooldown.
    #[error("enforcer: rollback for '{service}' blocked by cooldown ({secs}s remaining)")]
    CooldownBlocked {
        /// Target service.
        service: String,
        /// Approximate remaining seconds.
        secs: u64,
    },
    /// Rollback execution error from the `forge` wrapper.
    #[error("enforcer: rollback error: {0}")]
    Rollback(#[from] crate::rollback::RollbackError),
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, EnforcerError>;

// ---------------------------------------------------------------------------
// EnforcementRecord — audit row written before any side-effect
// ---------------------------------------------------------------------------

/// Enforcement record written to `enforcement_actions` before any side-effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementRecord {
    /// Unix millisecond timestamp.
    pub ts: i64,
    /// Trigger source label.
    pub trigger_source: String,
    /// Target service name.
    pub target_service: String,
    /// Action taken.
    pub action: String,
    /// Severity string.
    pub severity: String,
    /// FK to the originating divergence report (if known).
    pub divergence_report_id: Option<i64>,
    /// Deterministic hash of the divergence report body (pre-state).
    pub pre_state_hash: String,
    /// Operator: `"auto"` or `"human:luke"`.
    pub operator: String,
    /// JSON detail.
    pub body: serde_json::Value,
}

// ---------------------------------------------------------------------------
// EnforcerDb — stateful enforcement with audit writes and cooldown
// ---------------------------------------------------------------------------

/// Stateful enforcer that combines classification, cooldown, audit writes,
/// and rollback/proposal dispatch.
pub struct EnforcerDb {
    enforcer: Enforcer,
    cooldown: CooldownTracker,
    db: Arc<Mutex<StateDb>>,
}

impl EnforcerDb {
    /// Creates a new `EnforcerDb`.
    ///
    /// `enabled` maps to [`Enforcer::enabled`].
    /// `cooldown_secs` sets the per-service rollback window.
    #[must_use]
    pub fn new(enabled: bool, cooldown_secs: u64, db: Arc<Mutex<StateDb>>) -> Self {
        Self {
            enforcer: Enforcer::new(enabled),
            cooldown: CooldownTracker::new(cooldown_secs),
            db,
        }
    }

    /// Processes a single divergence report.
    ///
    /// # Audit-first guarantee
    ///
    /// The `enforcement_actions` row is written to the database **before**
    /// invoking `forge --rollback`. A failed DB write aborts the rollback.
    ///
    /// # Cooldown enforcement
    ///
    /// Returns `Err(EnforcerError::CooldownBlocked)` without writing any DB
    /// row if the service was rolled back within the cooldown window.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the audit write fails or the rollback subprocess fails.
    pub fn process(&mut self, report: &DivergenceReport) -> Result<EnforcerAction> {
        let action = self.enforcer.evaluate(report);

        if action == EnforcerAction::NoOp {
            return Ok(EnforcerAction::NoOp);
        }

        // Extract a service name from the report body or fall back to kind.
        let service = report
            .body
            .get("service")
            .and_then(|v| v.as_str())
            .unwrap_or(&report.kind)
            .to_owned();

        // Cooldown check — only for auto-rollback actions.
        if action == EnforcerAction::RollbackAuto && !self.cooldown.may_rollback(&service) {
            warn!(%service, kind = %report.kind, "rollback blocked by cooldown");
            return Err(EnforcerError::CooldownBlocked {
                service,
                secs: COOLDOWN_SECS,
            });
        }

        let trigger = TriggerSource::from_kind(&report.kind);
        let ts = chrono::Utc::now().timestamp_millis();
        let pre_hash = fnv_hex(&serde_json::to_string(&report.body)?);

        let record = EnforcementRecord {
            ts,
            trigger_source: trigger.as_label().to_owned(),
            target_service: service.clone(),
            action: action.as_str().to_owned(),
            severity: report.severity.as_str().to_owned(),
            divergence_report_id: report.id,
            pre_state_hash: pre_hash,
            operator: "auto".to_owned(),
            body: json!({
                "kind": report.kind,
                "source": report.source,
                "plan_ref": report.plan_ref,
                "observed_ref": report.observed_ref,
                "body": report.body,
            }),
        };

        // --- Audit write FIRST (before any side-effect) ---
        self.write_enforcement_record(&record)?;

        // --- Dispatch action ---
        match &action {
            EnforcerAction::RollbackAuto => {
                info!(%service, kind = %report.kind, "auto-rollback dispatched");
                self.cooldown.record_rollback(&service);
                crate::rollback::rollback_service(&service)?;
            }
            EnforcerAction::RollbackProposed => {
                info!(%service, kind = %report.kind, "rollback proposed via atuin kv");
                if let Err(e) = propose_via_atuin(&service) {
                    // Non-fatal: audit row already written; log and continue.
                    warn!(%service, error = %e, "atuin kv proposal failed (degraded)");
                }
            }
            EnforcerAction::NoOp => {}
        }

        Ok(action)
    }

    fn write_enforcement_record(&self, record: &EnforcementRecord) -> Result<()> {
        let body_str = serde_json::to_string(&record.body)?;
        // parking_lot::Mutex::lock() is infallible — no poison semantics.
        let db = self.db.lock();
        db.insert_enforcement_action(
            record.ts,
            &record.trigger_source,
            &record.target_service,
            &record.action,
            &record.severity,
            record.divergence_report_id,
            &record.pre_state_hash,
            &record.operator,
            &body_str,
        )?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Atuin KV proposal (propose-only path)
// ---------------------------------------------------------------------------

/// Writes a proposed-rollback entry to Atuin KV.
///
/// Key: `enforcement.proposed_rollback.<service>`
/// Value: Unix timestamp string.
///
/// Uses `std::process::Command` with a fixed argument list (no shell
/// interpolation) — safe against injection because args are passed as a
/// `Vec<&str>`, not through a shell.
///
/// Degrades gracefully if `atuin` is absent from `PATH`.
fn propose_via_atuin(service: &str) -> std::result::Result<(), String> {
    let key = format!("enforcement.proposed_rollback.{service}");
    let val = chrono::Utc::now().timestamp_millis().to_string();

    // std::process::Command with explicit args — NOT exec() / shell expansion.
    let result = std::process::Command::new("atuin")
        .args(["kv", "set", &key, &val])
        .output();

    match result {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(format!(
            "atuin exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // atuin not installed — degrade silently.
            debug!(%service, "atuin not found — skipping kv proposal");
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// FNV-1a hex hash helper
// ---------------------------------------------------------------------------

/// Returns a 16-char hex string (FNV-1a 64-bit) of `data`.
///
/// Used as the `pre_state_hash` audit field — sufficient for identity
/// without pulling in the `sha2` crate.
fn fnv_hex(data: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in data.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn report(severity: Severity) -> DivergenceReport {
        report_kind(severity, "test_kind")
    }

    fn report_kind(severity: Severity, kind: &str) -> DivergenceReport {
        DivergenceReport {
            id: None,
            ts: 1_000,
            source: "zen".into(),
            plan_ref: "plan.md".into(),
            observed_ref: "live".into(),
            severity,
            kind: kind.into(),
            body: json!({"service": "pane-vortex"}),
        }
    }

    // --- EnforcerAction ---

    #[test]
    fn action_as_str_roundtrip() {
        assert_eq!(EnforcerAction::RollbackAuto.as_str(), "rollback_auto");
        assert_eq!(EnforcerAction::RollbackProposed.as_str(), "rollback_proposed");
        assert_eq!(EnforcerAction::NoOp.as_str(), "no_op");
    }

    // --- TriggerSource ---

    #[test]
    fn trigger_source_classification() {
        assert_eq!(
            TriggerSource::from_kind("trait_violation"),
            TriggerSource::WatcherTraitViolation
        );
        assert_eq!(
            TriggerSource::from_kind("phase_skip"),
            TriggerSource::ZenPhaseSkip
        );
        assert_eq!(
            TriggerSource::from_kind("soak_health_degrade"),
            TriggerSource::SoakHealthDegrade
        );
        assert_eq!(
            TriggerSource::from_kind("bridge_contract_mismatch"),
            TriggerSource::BridgeContractMismatch
        );
        assert!(matches!(
            TriggerSource::from_kind("unknown_xyz"),
            TriggerSource::Other(_)
        ));
    }

    #[test]
    fn soak_high_auto_rollback_true() {
        assert!(TriggerSource::SoakHealthDegrade.high_auto_rollback());
    }

    #[test]
    fn phase_skip_high_auto_rollback_false() {
        assert!(!TriggerSource::ZenPhaseSkip.high_auto_rollback());
    }

    #[test]
    fn watcher_high_auto_rollback_false() {
        // trait_violation fires at CRITICAL per spec; HIGH override is not auto.
        assert!(!TriggerSource::WatcherTraitViolation.high_auto_rollback());
    }

    #[test]
    fn trigger_source_labels() {
        assert_eq!(
            TriggerSource::WatcherTraitViolation.as_label(),
            "watcher_trait_violation"
        );
        assert_eq!(TriggerSource::ZenPhaseSkip.as_label(), "zen_phase_skip");
        assert_eq!(TriggerSource::SoakHealthDegrade.as_label(), "soak");
        assert_eq!(
            TriggerSource::BridgeContractMismatch.as_label(),
            "bridge_contract"
        );
    }

    // --- CooldownTracker ---

    #[test]
    fn cooldown_permits_first_rollback() {
        let tracker = CooldownTracker::new(300);
        assert!(tracker.may_rollback("pane-vortex"));
    }

    #[test]
    fn cooldown_blocks_immediate_second_rollback() {
        let mut tracker = CooldownTracker::new(300);
        tracker.record_rollback("pane-vortex");
        assert!(!tracker.may_rollback("pane-vortex"));
    }

    #[test]
    fn cooldown_allows_different_service() {
        let mut tracker = CooldownTracker::new(300);
        tracker.record_rollback("service-a");
        assert!(tracker.may_rollback("service-b"));
    }

    #[test]
    fn cooldown_zero_secs_always_permits() {
        let mut tracker = CooldownTracker::new(0);
        tracker.record_rollback("svc");
        // With 0s cooldown elapsed is always >= 0 — immediately permits again.
        assert!(tracker.may_rollback("svc"));
    }

    // --- Enforcer (pure classification) ---

    #[test]
    fn disabled_enforcer_always_noop() {
        let enf = Enforcer::new(false);
        for sev in [Severity::Low, Severity::Med, Severity::High, Severity::Critical] {
            assert_eq!(enf.evaluate(&report(sev)), EnforcerAction::NoOp);
        }
    }

    #[test]
    fn enabled_enforcer_critical_is_rollback_auto() {
        let enf = Enforcer::new(true);
        assert_eq!(
            enf.evaluate(&report(Severity::Critical)),
            EnforcerAction::RollbackAuto
        );
    }

    #[test]
    fn enabled_enforcer_high_phase_skip_is_proposed() {
        let enf = Enforcer::new(true);
        let r = report_kind(Severity::High, "phase_skip");
        assert_eq!(enf.evaluate(&r), EnforcerAction::RollbackProposed);
    }

    #[test]
    fn enabled_enforcer_high_soak_is_auto() {
        let enf = Enforcer::new(true);
        let r = report_kind(Severity::High, "soak_health_degrade");
        assert_eq!(enf.evaluate(&r), EnforcerAction::RollbackAuto);
    }

    #[test]
    fn enabled_enforcer_trait_violation_critical_is_auto() {
        let enf = Enforcer::new(true);
        let r = report_kind(Severity::Critical, "trait_violation");
        assert_eq!(enf.evaluate(&r), EnforcerAction::RollbackAuto);
    }

    #[test]
    fn enabled_enforcer_bridge_contract_critical_is_auto() {
        let enf = Enforcer::new(true);
        let r = report_kind(Severity::Critical, "bridge_contract_mismatch");
        assert_eq!(enf.evaluate(&r), EnforcerAction::RollbackAuto);
    }

    #[test]
    fn enabled_enforcer_med_is_noop() {
        let enf = Enforcer::new(true);
        assert_eq!(enf.evaluate(&report(Severity::Med)), EnforcerAction::NoOp);
    }

    #[test]
    fn enabled_enforcer_low_is_noop() {
        let enf = Enforcer::new(true);
        assert_eq!(enf.evaluate(&report(Severity::Low)), EnforcerAction::NoOp);
    }

    #[test]
    fn batch_filters_noop() {
        let enf = Enforcer::new(true);
        let reports = vec![
            report(Severity::Low),
            report(Severity::High),
            report(Severity::Critical),
            report(Severity::Med),
        ];
        let actions = enf.evaluate_batch(&reports);
        assert_eq!(actions.len(), 2);
        assert!(actions.iter().any(|(_, a)| *a == EnforcerAction::RollbackAuto));
        assert!(actions
            .iter()
            .any(|(_, a)| *a == EnforcerAction::RollbackProposed));
    }

    #[test]
    fn batch_empty_input_returns_empty() {
        let enf = Enforcer::new(true);
        assert!(enf.evaluate_batch(&[]).is_empty());
    }

    // --- fnv_hex ---

    #[test]
    fn fnv_hex_is_sixteen_chars() {
        assert_eq!(fnv_hex("hello").len(), 16);
    }

    #[test]
    fn fnv_hex_deterministic() {
        assert_eq!(fnv_hex("abc"), fnv_hex("abc"));
    }

    #[test]
    fn fnv_hex_different_inputs_differ() {
        assert_ne!(fnv_hex("abc"), fnv_hex("xyz"));
    }

    // --- atuin proposal ---

    #[test]
    fn propose_via_atuin_does_not_panic() {
        // atuin may or may not be on PATH in CI — both outcomes are acceptable.
        // The invariant is: no panic regardless of binary presence.
        let _ = propose_via_atuin("test-service");
    }
}
