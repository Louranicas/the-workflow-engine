//! `m42_stcortex_emit` — substrate-feedback (Hebbian reinforce) via m13.
//! Cluster H · L8.
//!
//! **m42 stcortex-only pivot (2026-05-17 ADR):** workflow-trace is
//! POVM-decoupled. m42 routes all substrate-feedback writes through m13
//! (which holds the namespace-guarded stcortex write path); it never
//! touches POVM directly.

use thiserror::Error;

use crate::m13_stcortex_writer::{
    CorrelationMemory, DeferReason, PromoteOutcome, StcortexWriter,
    StcortexWriterError, LtpDensityReader, SubstrateWriter,
};
use crate::m30_bank::AcceptedWorkflow;
use crate::m7_workflow_runs::WorkflowRunRow;

/// Feedback signal kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HebbianSignal {
    /// Long-term potentiation (positive reinforcement).
    Reinforce,
    /// Long-term depression (negative reinforcement).
    Depress,
}

/// Emit errors.
#[derive(Debug, Error)]
pub enum SubstrateEmitError {
    /// Underlying m13 writer failed.
    #[error("stcortex writer: {0}")]
    Writer(#[from] StcortexWriterError),
}

/// Compute a Hebbian signal from a workflow's recent outcome.
#[must_use]
pub fn signal_for_outcome(outcome: Option<&str>) -> HebbianSignal {
    match outcome {
        Some("ok") => HebbianSignal::Reinforce,
        _ => HebbianSignal::Depress,
    }
}

/// Emit a substrate-feedback memory via m13.
///
/// # Errors
///
/// [`SubstrateEmitError::Writer`] propagated from m13.
pub fn emit_feedback<R: LtpDensityReader, W: SubstrateWriter>(
    writer: &StcortexWriter<R, W>,
    workflow: &AcceptedWorkflow,
    run: &WorkflowRunRow,
    namespace_key: &str,
    signal: HebbianSignal,
) -> Result<PromoteOutcome, SubstrateEmitError> {
    let _ = workflow; // accepted-workflow context for downstream auditing
    let _ = signal; // signal is encoded by the m13 relevance heuristic
    Ok(writer.promote_run(run, namespace_key)?)
}

/// Shorthand: convert a [`PromoteOutcome`] to a brief operator-readable
/// summary line (used by m12 reports and tracing).
#[must_use]
pub fn outcome_summary(outcome: &PromoteOutcome) -> String {
    match outcome {
        PromoteOutcome::Written { memory_id } => format!("wrote memory_id={memory_id}"),
        PromoteOutcome::WrittenUnderPressure {
            memory_id,
            ltp_density,
        } => format!("wrote_under_pressure memory_id={memory_id} ltp={ltp_density:.4}"),
        PromoteOutcome::Deferred { reason } => match reason {
            DeferReason::LtpBelowFloor { density } => {
                format!("deferred ltp_below_floor density={density:.4}")
            }
            DeferReason::OracUnreachable => "deferred orac_unreachable".to_owned(),
            DeferReason::StcortexUnreachable => "deferred stcortex_unreachable".to_owned(),
        },
    }
}

/// Alias for callers using the m42 namespace.
pub type Memory = CorrelationMemory;

#[cfg(test)]
mod tests {
    use super::{outcome_summary, signal_for_outcome, HebbianSignal};
    use crate::m13_stcortex_writer::{DeferReason, PromoteOutcome};

    #[test]
    fn signal_for_ok_outcome_is_reinforce() {
        assert_eq!(signal_for_outcome(Some("ok")), HebbianSignal::Reinforce);
    }

    #[test]
    fn signal_for_fail_outcome_is_depress() {
        assert_eq!(signal_for_outcome(Some("fail")), HebbianSignal::Depress);
        assert_eq!(signal_for_outcome(None), HebbianSignal::Depress);
    }

    #[test]
    fn outcome_summary_written() {
        let s = outcome_summary(&PromoteOutcome::Written { memory_id: 42 });
        assert!(s.contains("42"));
        assert!(s.contains("wrote"));
    }

    #[test]
    fn outcome_summary_written_under_pressure() {
        let s = outcome_summary(&PromoteOutcome::WrittenUnderPressure {
            memory_id: 42,
            ltp_density: 0.05,
        });
        assert!(s.contains("wrote_under_pressure"));
        assert!(s.contains("0.0500"));
    }

    #[test]
    fn outcome_summary_deferred_orac() {
        let s = outcome_summary(&PromoteOutcome::Deferred {
            reason: DeferReason::OracUnreachable,
        });
        assert!(s.contains("deferred"));
        assert!(s.contains("orac_unreachable"));
    }

    #[test]
    fn outcome_summary_deferred_below_floor() {
        let s = outcome_summary(&PromoteOutcome::Deferred {
            reason: DeferReason::LtpBelowFloor { density: 0.005 },
        });
        assert!(s.contains("ltp_below_floor"));
    }

    #[test]
    fn hebbian_signal_serde_roundtrip() {
        for sig in [HebbianSignal::Reinforce, HebbianSignal::Depress] {
            let s = serde_json::to_string(&sig).expect("ser");
            let back: HebbianSignal = serde_json::from_str(&s).expect("de");
            assert_eq!(back, sig);
        }
    }
}
