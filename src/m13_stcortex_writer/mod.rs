//! `m13_stcortex_writer` — promote `WorkflowRunRow` rows to stcortex
//! under the 3-band LTP/LTD gate.
//!
//! **AP30 namespace-prefix write-side enforcer:** every write boundary
//! invokes m9 namespace validation via [`ValidatedNamespace`] type
//! evidence. There is no path from raw `&str` to a stcortex write.
//!
//! **3-band LTP/LTD gate (D-E reconciled to substrate_LTP_density scale,
//! Hebbian v3 Phase 1 floor 0.015):**
//! - `>= 0.10` (Phase 3 target) → write proceeds normally
//! - `[0.015, 0.10)` → write proceeds with `under_pressure` flag
//! - `< 0.015` → defer to JSONL outbox (no stcortex write)
//! - ORAC unreachable → defer

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use thiserror::Error;

use crate::m7_workflow_runs::WorkflowRunRow;
use crate::m9_watcher_namespace_guard::{
    assert_workflow_trace_namespace, NamespaceViolation, ValidatedNamespace,
};

/// Phase 1 floor — defer below this `substrate_LTP_density`.
pub const LTP_PHASE_1_FLOOR: f64 = 0.015;

/// Phase 3 target — full writes above this `substrate_LTP_density`.
pub const LTP_PHASE_3_TARGET: f64 = 0.10;

/// Default substrate-band probe timeout.
pub const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Stable consumer source tag emitted with every memory.
pub const SOURCE_TAG: &str = "workflow-trace-m13";

/// Failure modes for m13.
#[derive(Debug, Error)]
pub enum StcortexWriterError {
    /// m9 rejected the namespace prefix.
    #[error("namespace prefix rejected: {0}")]
    NamespaceViolation(#[from] NamespaceViolation),
    /// Underlying writer (real or test mock) reported failure.
    #[error("substrate write failed: {0}")]
    WriteFailed(String),
    /// JSONL outbox I/O error.
    #[error("outbox io: {0}")]
    OutboxIo(#[from] std::io::Error),
    /// JSON serde failure.
    #[error("outbox serde: {0}")]
    OutboxSerde(#[from] serde_json::Error),
}

/// One memory payload promoted to stcortex.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CorrelationMemory {
    /// Validated namespace (always `workflow_trace_*`).
    pub namespace: String,
    /// Memory type — always `"semantic"` in Phase A.
    pub memory_type: String,
    /// JSON-encoded content (run_id + outcome + cost_tokens + cluster digest).
    pub content: String,
    /// Static heuristic mapping `ok=1.0 fail=0.5 abort=0.3 unknown=0.1` —
    /// NOT derived from `fitness_dimension` (F9 zero-weight).
    pub relevance: f32,
    /// Session id of the originating run.
    pub session_id: String,
    /// Source tag — always `workflow-trace-m13`.
    pub source_tag: String,
    /// Phase A always `None` (F9 fitness-dimension is zero-weight).
    pub tensor: Option<Vec<f32>>,
    /// `true` when the write fired in the warn-and-proceed band.
    pub under_pressure: bool,
}

impl CorrelationMemory {
    /// Build a payload from a `WorkflowRunRow` + validated namespace +
    /// pressure flag.
    #[must_use]
    pub fn from_row(
        row: &WorkflowRunRow,
        namespace: &ValidatedNamespace,
        under_pressure: bool,
    ) -> Self {
        let outcome = row.outcome.as_deref().unwrap_or("unknown");
        let relevance = match outcome {
            "ok" => 1.0_f32,
            "fail" => 0.5,
            "abort" => 0.3,
            _ => 0.1,
        };
        let content = serde_json::json!({
            "run_id": row.id,
            "outcome": outcome,
            "cost_tokens": row.cost_tokens,
            "consumer_inputs_digest": &row.consumer_inputs,
        });
        Self {
            namespace: namespace.as_str().to_owned(),
            memory_type: "semantic".to_owned(),
            content: content.to_string(),
            relevance,
            session_id: row.id.to_string(),
            source_tag: SOURCE_TAG.to_owned(),
            tensor: None,
            under_pressure,
        }
    }
}

/// Outcome of a `promote_run` attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum PromoteOutcome {
    /// Wrote with band-1 headroom; carries memory id.
    Written {
        /// Substrate-assigned memory id.
        memory_id: i64,
    },
    /// Wrote in band-2; tagged under_pressure.
    WrittenUnderPressure {
        /// Substrate-assigned memory id.
        memory_id: i64,
        /// Observed LTP density at write time.
        ltp_density: f64,
    },
    /// Deferred to the JSONL outbox.
    Deferred {
        /// Reason for the deferral.
        reason: DeferReason,
    },
}

/// Why a write was deferred.
#[derive(Debug, Clone, PartialEq)]
pub enum DeferReason {
    /// `substrate_LTP_density < LTP_PHASE_1_FLOOR`.
    LtpBelowFloor {
        /// Observed density.
        density: f64,
    },
    /// ORAC blackboard probe failed.
    OracUnreachable,
    /// stcortex itself was unreachable.
    StcortexUnreachable,
}

/// Read the current `substrate_LTP_density` value. Production impl hits
/// ORAC `:8133/blackboard/substrate_LTP_density`; tests inject mocks.
pub trait LtpDensityReader: Send + Sync {
    /// Return the current density, or `None` if the probe is unreachable.
    fn read_density(&self) -> Option<f64>;
}

/// Write a `CorrelationMemory` to the substrate. Production impl uses
/// the SpacetimeDB SDK; tests inject mocks.
pub trait SubstrateWriter: Send + Sync {
    /// Write the memory and return the assigned id, or an error.
    ///
    /// # Errors
    ///
    /// [`StcortexWriterError::WriteFailed`] on transport / encoding errors.
    fn write_memory(
        &self,
        memory: &CorrelationMemory,
    ) -> Result<i64, StcortexWriterError>;
}

/// Default ORAC reader (lightweight HTTP GET).
pub struct OracHttpReader {
    url: String,
    timeout: Duration,
}

impl OracHttpReader {
    /// Construct against the canonical ORAC blackboard.
    #[must_use]
    pub fn new(url: impl Into<String>, timeout: Duration) -> Self {
        Self {
            url: url.into(),
            timeout,
        }
    }
}

impl LtpDensityReader for OracHttpReader {
    fn read_density(&self) -> Option<f64> {
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()
            .ok()?;
        let body: serde_json::Value = client.get(&self.url).send().ok()?.json().ok()?;
        body.get("substrate_LTP_density")
            .and_then(serde_json::Value::as_f64)
            .or_else(|| body.as_f64())
    }
}

/// The m13 writer.
pub struct StcortexWriter<R, W>
where
    R: LtpDensityReader,
    W: SubstrateWriter,
{
    reader: R,
    writer: W,
    outbox_path: PathBuf,
    outbox_lock: Mutex<()>,
}

impl<R, W> std::fmt::Debug for StcortexWriter<R, W>
where
    R: LtpDensityReader,
    W: SubstrateWriter,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StcortexWriter")
            .field("outbox_path", &self.outbox_path)
            .finish_non_exhaustive()
    }
}

impl<R, W> StcortexWriter<R, W>
where
    R: LtpDensityReader,
    W: SubstrateWriter,
{
    /// Construct with explicit reader / writer / outbox path.
    pub fn new(reader: R, writer: W, outbox_path: PathBuf) -> Self {
        Self {
            reader,
            writer,
            outbox_path,
            outbox_lock: Mutex::new(()),
        }
    }

    /// Promote a completed workflow run. Performs:
    ///
    /// 1. m9 namespace validation (`namespace_key` MUST start with the
    ///    canonical [`crate::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX`]).
    /// 2. ORAC LTP density probe.
    /// 3. Either write or defer per 3-band gate.
    ///
    /// # Errors
    ///
    /// - [`StcortexWriterError::NamespaceViolation`] if m9 rejects.
    /// - [`StcortexWriterError::WriteFailed`] if the substrate writer errors.
    /// - [`StcortexWriterError::OutboxIo`] / [`StcortexWriterError::OutboxSerde`]
    ///   if the outbox write itself fails (defers cannot succeed silently).
    pub fn promote_run(
        &self,
        run: &WorkflowRunRow,
        namespace_key: &str,
    ) -> Result<PromoteOutcome, StcortexWriterError> {
        let validated = assert_workflow_trace_namespace(namespace_key)?;
        let density = self.reader.read_density();
        match density {
            None => {
                let memory =
                    CorrelationMemory::from_row(run, &validated, false);
                self.defer(&memory, &DeferReason::OracUnreachable)?;
                Ok(PromoteOutcome::Deferred {
                    reason: DeferReason::OracUnreachable,
                })
            }
            Some(d) if d < LTP_PHASE_1_FLOOR => {
                let memory =
                    CorrelationMemory::from_row(run, &validated, false);
                self.defer(&memory, &DeferReason::LtpBelowFloor { density: d })?;
                Ok(PromoteOutcome::Deferred {
                    reason: DeferReason::LtpBelowFloor { density: d },
                })
            }
            Some(d) if d < LTP_PHASE_3_TARGET => {
                let memory =
                    CorrelationMemory::from_row(run, &validated, true);
                let memory_id = self.writer.write_memory(&memory)?;
                tracing::warn!(
                    target: "m13.promote.under_pressure",
                    ltp_density = d,
                    memory_id,
                    "stcortex write proceeded under pressure"
                );
                Ok(PromoteOutcome::WrittenUnderPressure {
                    memory_id,
                    ltp_density: d,
                })
            }
            Some(_) => {
                let memory =
                    CorrelationMemory::from_row(run, &validated, false);
                let memory_id = self.writer.write_memory(&memory)?;
                Ok(PromoteOutcome::Written { memory_id })
            }
        }
    }

    fn defer(
        &self,
        memory: &CorrelationMemory,
        reason: &DeferReason,
    ) -> Result<(), StcortexWriterError> {
        let entry = serde_json::json!({
            "ts_ms": now_ms(),
            "memory": memory,
            "reason": match reason {
                DeferReason::LtpBelowFloor { density } => {
                    serde_json::json!({"LtpBelowFloor": {"density": density}})
                }
                DeferReason::OracUnreachable => serde_json::json!("OracUnreachable"),
                DeferReason::StcortexUnreachable => {
                    serde_json::json!("StcortexUnreachable")
                }
            },
        });
        let line = format!("{}\n", serde_json::to_string(&entry)?);
        let _guard = self.outbox_lock.lock().ok();
        if let Some(parent) = self.outbox_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.outbox_path)?;
        f.write_all(line.as_bytes())?;
        Ok(())
    }

    /// Borrow the outbox path.
    #[must_use]
    pub fn outbox_path(&self) -> &PathBuf {
        &self.outbox_path
    }
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex as StdMutex;

    use super::{
        CorrelationMemory, DeferReason, LtpDensityReader, PromoteOutcome, StcortexWriter,
        StcortexWriterError, SubstrateWriter, LTP_PHASE_1_FLOOR, LTP_PHASE_3_TARGET, SOURCE_TAG,
    };
    use crate::m7_workflow_runs::WorkflowRunRow;
    use crate::m9_watcher_namespace_guard::{
        assert_workflow_trace_namespace, WORKFLOW_TRACE_NS_PREFIX,
    };

    fn canonical_ns() -> String {
        format!("{WORKFLOW_TRACE_NS_PREFIX}_correlations")
    }
    fn canonical_ns_x() -> String {
        format!("{WORKFLOW_TRACE_NS_PREFIX}_x")
    }

    fn run() -> WorkflowRunRow {
        WorkflowRunRow {
            id: 42,
            started_at: "2026-05-17T00:00:00Z".into(),
            ended_at: Some("2026-05-17T01:00:00Z".into()),
            outcome: Some("ok".into()),
            consumer_inputs: "{}".into(),
            cost_tokens: Some(100),
            fitness_dimension: 0.0,
        }
    }

    struct StaticDensity(Option<f64>);
    impl LtpDensityReader for StaticDensity {
        fn read_density(&self) -> Option<f64> {
            self.0
        }
    }

    struct RecordingWriter {
        next_id: StdMutex<i64>,
        written: StdMutex<Vec<CorrelationMemory>>,
        fail: bool,
    }
    impl SubstrateWriter for RecordingWriter {
        fn write_memory(
            &self,
            memory: &CorrelationMemory,
        ) -> Result<i64, StcortexWriterError> {
            if self.fail {
                return Err(StcortexWriterError::WriteFailed("mock-fail".into()));
            }
            let mut id = self.next_id.lock().expect("lock");
            *id += 1;
            self.written.lock().expect("lock").push(memory.clone());
            Ok(*id)
        }
    }

    fn writer(
        density: Option<f64>,
        fail_writes: bool,
    ) -> StcortexWriter<StaticDensity, RecordingWriter> {
        let outbox =
            tempfile::Builder::new().suffix(".jsonl").tempfile().expect("temp").into_temp_path();
        let path = outbox.to_path_buf();
        // Forget the temp_path so the file is not deleted on drop.
        std::mem::forget(outbox);
        StcortexWriter::new(
            StaticDensity(density),
            RecordingWriter {
                next_id: StdMutex::new(0),
                written: StdMutex::new(Vec::new()),
                fail: fail_writes,
            },
            path,
        )
    }

    // ---- 3-band gate (5) -----------------------------------------------

    #[test]
    fn band_above_phase_3_writes_normally() {
        let w = writer(Some(0.20), false);
        let out = w.promote_run(&run(), &canonical_ns()).expect("promote");
        assert!(matches!(out, PromoteOutcome::Written { .. }));
    }

    #[test]
    fn band_between_phases_writes_under_pressure() {
        let w = writer(Some(0.05), false);
        let out = w.promote_run(&run(), &canonical_ns()).expect("promote");
        match out {
            PromoteOutcome::WrittenUnderPressure { ltp_density, .. } => {
                assert!((ltp_density - 0.05).abs() < 1e-12);
            }
            other => panic!("expected WrittenUnderPressure, got {other:?}"),
        }
    }

    #[test]
    fn band_below_phase_1_defers_to_outbox() {
        let w = writer(Some(0.001), false);
        let out = w.promote_run(&run(), &canonical_ns()).expect("promote");
        match out {
            PromoteOutcome::Deferred {
                reason: DeferReason::LtpBelowFloor { density },
            } => {
                assert!((density - 0.001).abs() < 1e-12);
            }
            other => panic!("expected Deferred LtpBelowFloor, got {other:?}"),
        }
        let contents = std::fs::read_to_string(w.outbox_path()).expect("read outbox");
        assert!(contents.contains("LtpBelowFloor"));
    }

    #[test]
    fn band_orac_unreachable_defers() {
        let w = writer(None, false);
        let out = w.promote_run(&run(), &canonical_ns()).expect("promote");
        assert!(matches!(
            out,
            PromoteOutcome::Deferred {
                reason: DeferReason::OracUnreachable
            }
        ));
    }

    #[test]
    fn writer_failure_propagates_typed_error() {
        let w = writer(Some(0.20), true);
        let err = w
            .promote_run(&run(), &canonical_ns())
            .expect_err("expected write fail");
        assert!(matches!(err, StcortexWriterError::WriteFailed(_)));
    }

    // ---- AP30 namespace enforcement (3) --------------------------------

    #[test]
    fn promote_rejects_foreign_namespace_prefix() {
        let w = writer(Some(0.20), false);
        let err = w.promote_run(&run(), "orac_foo").expect_err("AP30");
        assert!(matches!(err, StcortexWriterError::NamespaceViolation(_)));
    }

    #[test]
    fn promote_rejects_empty_namespace() {
        let w = writer(Some(0.20), false);
        assert!(matches!(
            w.promote_run(&run(), ""),
            Err(StcortexWriterError::NamespaceViolation(_))
        ));
    }

    #[test]
    fn promote_accepts_canonical_namespace() {
        let w = writer(Some(0.20), false);
        assert!(w
            .promote_run(&run(), &canonical_ns())
            .is_ok());
    }

    // ---- CorrelationMemory shape (4) -----------------------------------

    #[test]
    fn correlation_memory_relevance_mapping() {
        let ns = assert_workflow_trace_namespace(&canonical_ns_x()).expect("ns");
        for (outcome, expected) in [
            ("ok", 1.0_f32),
            ("fail", 0.5),
            ("abort", 0.3),
            ("unknown", 0.1),
        ] {
            let mut r = run();
            r.outcome = Some(outcome.to_owned());
            let m = CorrelationMemory::from_row(&r, &ns, false);
            assert!((m.relevance - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn correlation_memory_tensor_is_none_in_phase_a() {
        let ns = assert_workflow_trace_namespace(&canonical_ns_x()).expect("ns");
        let m = CorrelationMemory::from_row(&run(), &ns, false);
        assert!(m.tensor.is_none(), "F9: tensor must be None in Phase A");
    }

    #[test]
    fn correlation_memory_source_tag_is_stable() {
        let ns = assert_workflow_trace_namespace(&canonical_ns_x()).expect("ns");
        let m = CorrelationMemory::from_row(&run(), &ns, false);
        assert_eq!(m.source_tag, SOURCE_TAG);
        assert_eq!(SOURCE_TAG, "workflow-trace-m13");
    }

    #[test]
    fn correlation_memory_under_pressure_flag_threads_through() {
        let ns = assert_workflow_trace_namespace(&canonical_ns_x()).expect("ns");
        let m = CorrelationMemory::from_row(&run(), &ns, true);
        assert!(m.under_pressure);
    }

    // ---- Outbox JSONL persistence (2) ----------------------------------

    #[test]
    fn defer_writes_jsonl_line_with_ts_memory_reason() {
        let w = writer(Some(0.001), false);
        let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        let contents = std::fs::read_to_string(w.outbox_path()).expect("read");
        let line = contents.lines().next().expect("at least one line");
        let parsed: serde_json::Value = serde_json::from_str(line).expect("parse");
        assert!(parsed.get("ts_ms").is_some());
        assert!(parsed.get("memory").is_some());
        assert!(parsed.get("reason").is_some());
    }

    #[test]
    fn defer_appends_subsequent_writes() {
        let w = writer(Some(0.001), false);
        let _ = w.promote_run(&run(), &canonical_ns()).expect("a");
        let _ = w.promote_run(&run(), &canonical_ns()).expect("b");
        let contents = std::fs::read_to_string(w.outbox_path()).expect("read");
        assert_eq!(contents.lines().count(), 2);
    }

    // ---- Threshold constants (2) ---------------------------------------

    #[test]
    fn ltp_phase_1_floor_is_zero_point_zero_one_five() {
        assert!((LTP_PHASE_1_FLOOR - 0.015).abs() < 1e-12);
    }

    #[test]
    fn ltp_phase_3_target_is_zero_point_one() {
        assert!((LTP_PHASE_3_TARGET - 0.10).abs() < 1e-12);
    }
}
