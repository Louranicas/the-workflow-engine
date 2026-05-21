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
    /// `None` is the load-bearing "probe unreachable" sentinel that drives the
    /// 3-band gate to defer. Each `.ok()?` below collapses a typed transport
    /// error into the unreachable sentinel by design:
    ///
    /// - `Client::builder().build()` — TLS / runtime construction failure.
    /// - `client.get(...).send()` — connection refused / DNS / timeout.
    /// - `.json::<Value>()` — body is not valid UTF-8 JSON.
    ///
    /// Rationale: ORAC blackboard transient unreachability is the EXPECTED
    /// failure mode and MUST NOT be propagated as a typed error; defer is
    /// the correct response per 3-band gate spec.
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
        // F-POVM-07 anti-pattern harmonisation (C2): emit ts_ms only when
        // the wall-clock is well-defined. When `now_ms()` returns `None`
        // (clock fault / pre-1970 / `i64` overflow), tag the entry with
        // `clock_unavailable: true` rather than silently writing
        // `"ts_ms": 0` into the outbox stream. Mirrors m11's
        // `chrono_now_ms` typed-Option contract. Tag-and-defer preserves
        // m13's fire-and-forget contract — the outbox consumer can choose
        // to drop, hold, or reconcile clock-fault rows downstream.
        let (ts_value, clock_ok) = match now_ms() {
            Some(ms) => (serde_json::json!(ms), true),
            None => (serde_json::Value::Null, false),
        };
        let mut entry = serde_json::json!({
            "ts_ms": ts_value,
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
        if !clock_ok {
            if let Some(obj) = entry.as_object_mut() {
                obj.insert(
                    "clock_unavailable".to_owned(),
                    serde_json::Value::Bool(true),
                );
            }
            tracing::warn!(
                target: "m13.defer.clock_unavailable",
                "outbox entry tagged clock_unavailable: SystemTime pre-epoch or i64 overflow"
            );
        }
        let line = format!("{}\n", serde_json::to_string(&entry)?);
        // Poison-recovery: previous impl was `lock().ok()` which silently
        // dropped the guard on PoisonError, allowing concurrent writers to
        // race past the lock and interleave bytes into the JSONL outbox.
        // Recover from poison by extracting the inner guard — a poisoned
        // outbox lock is safe to reuse because the outbox is append-only
        // and lines are pre-rendered (no protected invariant to repair).
        // (Fix: CONFIRMED silent-failure-hunter — `let _ = lock().ok()` drop
        // pattern on a Mutex protecting concurrent file appends.)
        let _guard = self
            .outbox_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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

/// Wall-clock time in milliseconds since UNIX epoch, or `None` when the
/// system clock is set *before* 1970 (genuine fault) or `as_millis()`
/// overflows `i64` in year ~292,471,209 AD.
///
/// **F-POVM-07 harmonisation (C2):** prior versions returned `0` via
/// `unwrap_or(0)`, silently emitting `"ts_ms": 0` (= 1970-01-01) into the
/// outbox JSONL stream on clock fault. This is the exact silent-zero
/// pattern m11's [`crate::m11_fitness_weighted_decay::chrono_now_ms`] was
/// hardened against. The signature is now `Option<i64>` — callers must
/// handle the `None` case explicitly. m13's defer path emits a
/// `clock_unavailable: true` tag rather than a phantom epoch timestamp.
#[must_use]
fn now_ms() -> Option<i64> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(dur.as_millis()).ok()
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

    // ====================================================================
    // Hardening pass (S1002388) — +10 tests for m13 stcortex writer.
    // Lock-poisoning recovery + threshold boundaries + AP30 enforcement
    // + F9 zero-weight + adversarial input + cross-module surfaces.
    // ====================================================================

    use std::sync::Arc;
    use std::thread;

    use super::DEFAULT_PROBE_TIMEOUT;

    // rationale: Boundary — LTP density exactly equal to LTP_PHASE_1_FLOOR
    // (0.015) is NOT below the floor (strict `<` check); writer proceeds
    // in band-2 (under-pressure).
    #[test]
    fn band_at_exact_floor_writes_under_pressure_not_deferred() {
        // rationale: Boundary (3-band gate exact threshold)
        let w = writer(Some(LTP_PHASE_1_FLOOR), false);
        let out = w
            .promote_run(&run(), &canonical_ns())
            .expect("promote at floor");
        assert!(
            matches!(out, PromoteOutcome::WrittenUnderPressure { .. }),
            "exact floor must NOT defer (strict `<`); got {out:?}"
        );
    }

    // rationale: Boundary — LTP density exactly equal to
    // LTP_PHASE_3_TARGET (0.10) is NOT in the pressure band; writer
    // proceeds in band-3 (normal write).
    #[test]
    fn band_at_exact_phase_3_target_writes_normally() {
        // rationale: Boundary (3-band gate exact threshold)
        let w = writer(Some(LTP_PHASE_3_TARGET), false);
        let out = w
            .promote_run(&run(), &canonical_ns())
            .expect("promote at target");
        assert!(
            matches!(out, PromoteOutcome::Written { .. }),
            "exact target must write normally; got {out:?}"
        );
    }

    // rationale: Anti-property — AP30 namespace check happens BEFORE the
    // LTP probe. Foreign namespace must fail-fast with NamespaceViolation
    // and never invoke the probe path.
    #[test]
    fn namespace_check_precedes_ltp_probe() {
        // rationale: Anti-property (AP30 order-of-operations)
        struct PanickingReader;
        impl LtpDensityReader for PanickingReader {
            fn read_density(&self) -> Option<f64> {
                panic!("LTP probe must not fire when namespace is rejected");
            }
        }
        let outbox = tempfile::Builder::new()
            .suffix(".jsonl")
            .tempfile()
            .expect("temp")
            .into_temp_path();
        let path = outbox.to_path_buf();
        std::mem::forget(outbox);
        let w = StcortexWriter::new(
            PanickingReader,
            RecordingWriter {
                next_id: StdMutex::new(0),
                written: StdMutex::new(Vec::new()),
                fail: false,
            },
            path,
        );
        let err = w.promote_run(&run(), "orac_evil").expect_err("AP30");
        assert!(matches!(err, StcortexWriterError::NamespaceViolation(_)));
    }

    // rationale: Concurrency — defer() recovers from a poisoned outbox
    // mutex. Pre-fix `lock().ok()` silently dropped the guard on poison;
    // the fix uses PoisonError::into_inner so concurrent defers stay
    // serialised.
    #[test]
    fn defer_recovers_from_poisoned_outbox_lock() {
        // rationale: Concurrency (silent-failure-hunter CONFIRMED bug)
        let w = Arc::new(writer(Some(0.001), false));
        let w_poison = Arc::clone(&w);
        let join = thread::spawn(move || {
            let _g = w_poison.outbox_lock.lock().expect("first lock");
            panic!("intentional poison");
        });
        let _ = join.join();
        assert!(
            w.outbox_lock.is_poisoned(),
            "test setup failed to poison the lock"
        );
        let out = w
            .promote_run(&run(), &canonical_ns())
            .expect("defer after poison");
        assert!(matches!(
            out,
            PromoteOutcome::Deferred {
                reason: DeferReason::LtpBelowFloor { .. }
            }
        ));
        let contents = std::fs::read_to_string(w.outbox_path()).expect("read");
        assert!(contents.contains("LtpBelowFloor"));
    }

    // rationale: Cross-module surface invariant — m9 hyphen-munge
    // composes correctly with m13: a hyphenated namespace arrives, gets
    // munged, and the CorrelationMemory carries the underscored form.
    #[test]
    fn promote_munges_hyphen_namespace_via_m9() {
        // rationale: Cross-module surface invariant (m9 munge + m13 write)
        let w = writer(Some(0.20), false);
        let out = w.promote_run(&run(), "workflow-trace-x").expect("promote");
        assert!(matches!(out, PromoteOutcome::Written { .. }));
        let written = w.writer.written.lock().expect("lock");
        assert_eq!(written.len(), 1);
        assert_eq!(
            written[0].namespace, "workflow_trace_x",
            "namespace must be munged to underscored form"
        );
    }

    // rationale: F9 zero-weight — CorrelationMemory's `tensor` field is
    // structurally `None` in Phase A regardless of pressure band; no
    // fitness signal leaks via tensor in band-2 either.
    #[test]
    fn correlation_memory_tensor_is_none_under_all_pressure_bands() {
        // rationale: F9 zero-weight (m13 surface)
        for density in [0.20_f64, 0.05] {
            let w = writer(Some(density), false);
            let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
            let written = w.writer.written.lock().expect("lock");
            assert!(
                !written.is_empty(),
                "expected a substrate write at density={density}"
            );
            for m in written.iter() {
                assert!(m.tensor.is_none(), "F9: tensor must stay None");
            }
        }
    }

    // rationale: Adversarial input — unknown outcome strings (future m7
    // variant unsynced with m13) collapse to the "unknown" relevance
    // bucket (0.1). Failure-soft documentation.
    #[test]
    fn unknown_outcome_collapses_to_unknown_relevance() {
        // rationale: Adversarial input
        let ns = assert_workflow_trace_namespace(&canonical_ns()).expect("ns");
        let mut r = run();
        r.outcome = Some("future_variant_zzz".into());
        let m = CorrelationMemory::from_row(&r, &ns, false);
        assert!((m.relevance - 0.1).abs() < 1e-6);
    }

    // rationale: Anti-property — F-POVM-07 silent-zero-timestamp.
    // now_ms() returns Some(positive realistic i64) on production clocks
    // (> 2024). Post-C2 harmonisation: signature is Option<i64>, never
    // `0` as a silent sentinel.
    #[test]
    fn now_ms_returns_some_positive_on_well_defined_clock() {
        // rationale: Anti-property (F-POVM-07 silent-zero-timestamp)
        let ts = super::now_ms().expect("production clock must be post-1970");
        assert!(
            ts > 1_700_000_000_000,
            "now_ms must return realistic 2024+ wall-clock ms, got {ts}"
        );
    }

    // rationale: Anti-property (C2) — now_ms returns Option<i64> not
    // `i64`. The contract change forbids the silent-zero `unwrap_or(0)`
    // pattern at compile time. Type-system regression test.
    #[test]
    fn now_ms_signature_is_option_i64_not_silent_zero_i64() {
        // rationale: Anti-property (C2 contract change at type level)
        let ts: Option<i64> = super::now_ms();
        assert!(ts.is_some(), "production clock must yield Some");
        // Function-pointer coercion: if a future refactor reverts the
        // signature to `fn() -> i64`, the assignment to this typed
        // `fn() -> Option<i64>` slot fails to compile. Type-system
        // regression test for the F-POVM-07 contract change.
        let ensure_option: fn() -> Option<i64> = super::now_ms;
        assert!(ensure_option().is_some(), "fn-pointer coercion verified");
    }

    // rationale: Anti-property (C2) — defer-path JSONL outbox never
    // contains `"ts_ms": 0` on a well-defined clock. Regression test
    // against the F-POVM-07 silent-zero pattern. On production hardware
    // (post-1970) every defer must emit a positive ts_ms.
    #[test]
    fn defer_never_writes_silent_zero_ts_ms_on_well_defined_clock() {
        // rationale: Anti-property (F-POVM-07 silent-zero-timestamp,
        //            structural regression — no `"ts_ms":0` slips through
        //            the JSONL outbox under any normal-clock condition)
        let w = writer(Some(0.001), false);
        let _ = w.promote_run(&run(), &canonical_ns()).expect("defer");
        let contents = std::fs::read_to_string(w.outbox_path()).expect("read outbox");
        assert!(
            !contents.contains("\"ts_ms\":0"),
            "F-POVM-07 silent-zero leaked into outbox JSONL: {contents}"
        );
        assert!(
            !contents.contains("\"ts_ms\": 0"),
            "F-POVM-07 silent-zero (whitespace variant) leaked into outbox: {contents}"
        );
        // Production clock should NEVER emit `clock_unavailable: true`.
        assert!(
            !contents.contains("clock_unavailable"),
            "production clock should not tag clock_unavailable; outbox: {contents}"
        );
    }

    // rationale: Contract regression — DEFAULT_PROBE_TIMEOUT is the
    // documented 5s upper bound. Drift detection.
    #[test]
    fn default_probe_timeout_is_five_seconds() {
        // rationale: Contract regression
        assert_eq!(DEFAULT_PROBE_TIMEOUT, std::time::Duration::from_secs(5));
    }

    // rationale: Contract regression — CorrelationMemory's `memory_type`
    // is the stable string "semantic" in Phase A.
    #[test]
    fn correlation_memory_memory_type_is_semantic_in_phase_a() {
        // rationale: Contract regression
        let ns = assert_workflow_trace_namespace(&canonical_ns()).expect("ns");
        let m = CorrelationMemory::from_row(&run(), &ns, false);
        assert_eq!(m.memory_type, "semantic");
    }

    // ====================================================================
    // Hardening pass 2 — +24 tests. 3-band boundaries, defer semantics,
    // memory id propagation, content shape, F9, error variants.
    // ====================================================================

    // rationale: Boundary — density just below the floor defers; the
    // strict `<` puts it in the defer band.
    #[test]
    fn band_just_below_floor_defers() {
        let w = writer(Some(LTP_PHASE_1_FLOOR - 1e-9), false);
        let out = w.promote_run(&run(), &canonical_ns()).expect("promote");
        assert!(
            matches!(
                out,
                PromoteOutcome::Deferred {
                    reason: DeferReason::LtpBelowFloor { .. }
                }
            ),
            "just below floor must defer; got {out:?}"
        );
    }

    // rationale: Boundary — density just below the phase-3 target writes
    // under pressure, not normally.
    #[test]
    fn band_just_below_phase_3_target_writes_under_pressure() {
        let w = writer(Some(LTP_PHASE_3_TARGET - 1e-9), false);
        let out = w.promote_run(&run(), &canonical_ns()).expect("promote");
        assert!(
            matches!(out, PromoteOutcome::WrittenUnderPressure { .. }),
            "just below target must be under-pressure; got {out:?}"
        );
    }

    // rationale: Correctness — band-1 normal write carries the
    // substrate-assigned memory id (RecordingWriter increments from 0, so
    // the first write returns id 1).
    #[test]
    fn written_outcome_carries_substrate_memory_id() {
        let w = writer(Some(0.50), false);
        let out = w.promote_run(&run(), &canonical_ns()).expect("promote");
        match out {
            PromoteOutcome::Written { memory_id } => assert_eq!(memory_id, 1),
            other => panic!("expected Written, got {other:?}"),
        }
    }

    // rationale: Correctness — band-2 under-pressure write also carries
    // the assigned memory id alongside the observed density.
    #[test]
    fn under_pressure_outcome_carries_memory_id_and_density() {
        let w = writer(Some(0.07), false);
        let out = w.promote_run(&run(), &canonical_ns()).expect("promote");
        match out {
            PromoteOutcome::WrittenUnderPressure {
                memory_id,
                ltp_density,
            } => {
                assert_eq!(memory_id, 1);
                assert!((ltp_density - 0.07).abs() < 1e-12);
            }
            other => panic!("expected WrittenUnderPressure, got {other:?}"),
        }
    }

    // rationale: Correctness — sequential writes get monotonically
    // increasing memory ids (substrate AUTOINCREMENT semantics).
    #[test]
    fn sequential_writes_get_increasing_memory_ids() {
        let w = writer(Some(0.50), false);
        let a = w.promote_run(&run(), &canonical_ns()).expect("a");
        let b = w.promote_run(&run(), &canonical_ns()).expect("b");
        let id_a = match a {
            PromoteOutcome::Written { memory_id } => memory_id,
            o => panic!("a: {o:?}"),
        };
        let id_b = match b {
            PromoteOutcome::Written { memory_id } => memory_id,
            o => panic!("b: {o:?}"),
        };
        assert!(id_b > id_a, "ids must increase: {id_a} -> {id_b}");
    }

    // rationale: Anti-property — band-1 normal write does NOT set the
    // under_pressure flag on the CorrelationMemory.
    #[test]
    fn band_1_write_memory_not_flagged_under_pressure() {
        let w = writer(Some(0.50), false);
        let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        let written = w.writer.written.lock().expect("lock");
        assert_eq!(written.len(), 1);
        assert!(
            !written[0].under_pressure,
            "band-1 write must not be flagged under_pressure"
        );
    }

    // rationale: Correctness — band-2 write DOES flag the
    // CorrelationMemory under_pressure (the warn-and-proceed band).
    #[test]
    fn band_2_write_memory_flagged_under_pressure() {
        let w = writer(Some(0.05), false);
        let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        let written = w.writer.written.lock().expect("lock");
        assert_eq!(written.len(), 1);
        assert!(
            written[0].under_pressure,
            "band-2 write must be flagged under_pressure"
        );
    }

    // rationale: Anti-property — a deferred run produces NO substrate
    // write; the RecordingWriter records zero memories.
    #[test]
    fn deferred_run_does_not_write_to_substrate() {
        let w = writer(Some(0.001), false);
        let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        let written = w.writer.written.lock().expect("lock");
        assert!(written.is_empty(), "defer must not touch the substrate writer");
    }

    // rationale: Anti-property — ORAC-unreachable defer also produces no
    // substrate write, only an outbox line.
    #[test]
    fn orac_unreachable_defer_writes_outbox_not_substrate() {
        let w = writer(None, false);
        let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        assert!(
            w.writer.written.lock().expect("lock").is_empty(),
            "unreachable defer must not write substrate"
        );
        let contents = std::fs::read_to_string(w.outbox_path()).expect("read");
        assert!(contents.contains("OracUnreachable"), "outbox: {contents}");
    }

    // rationale: Correctness — the outbox JSONL `memory` object carries
    // the validated namespace from m9 (munged form).
    #[test]
    fn defer_outbox_memory_carries_validated_namespace() {
        let w = writer(Some(0.001), false);
        let _ = w.promote_run(&run(), "workflow-trace-deferns").expect("defer");
        let contents = std::fs::read_to_string(w.outbox_path()).expect("read");
        let line = contents.lines().next().expect("line");
        let parsed: serde_json::Value = serde_json::from_str(line).expect("parse");
        let ns = parsed
            .get("memory")
            .and_then(|m| m.get("namespace"))
            .and_then(serde_json::Value::as_str)
            .expect("namespace");
        assert_eq!(ns, "workflow_trace_deferns", "munged namespace in outbox");
    }

    // rationale: Correctness — the LtpBelowFloor outbox reason carries
    // the observed density value verbatim for downstream reconciliation.
    #[test]
    fn defer_outbox_ltp_reason_carries_observed_density() {
        let w = writer(Some(0.007), false);
        let _ = w.promote_run(&run(), &canonical_ns()).expect("defer");
        let contents = std::fs::read_to_string(w.outbox_path()).expect("read");
        let line = contents.lines().next().expect("line");
        let parsed: serde_json::Value = serde_json::from_str(line).expect("parse");
        let density = parsed
            .pointer("/reason/LtpBelowFloor/density")
            .and_then(serde_json::Value::as_f64)
            .expect("density");
        assert!((density - 0.007).abs() < 1e-12, "density {density}");
    }

    // rationale: Concurrency — concurrent defers from many threads append
    // exactly N lines with no byte-interleaving (each line valid JSON).
    #[test]
    fn concurrent_defers_append_n_well_formed_lines() {
        let w = Arc::new(writer(Some(0.001), false));
        let mut handles = Vec::new();
        for _ in 0..6_u32 {
            let wc = Arc::clone(&w);
            handles.push(thread::spawn(move || {
                for _ in 0..10_u32 {
                    let _ = wc.promote_run(&run(), &canonical_ns()).expect("defer");
                }
            }));
        }
        for h in handles {
            h.join().expect("join");
        }
        let contents = std::fs::read_to_string(w.outbox_path()).expect("read");
        assert_eq!(contents.lines().count(), 60, "6*10 outbox lines");
        for line in contents.lines() {
            let _: serde_json::Value =
                serde_json::from_str(line).expect("each line is valid JSON");
        }
    }

    // rationale: Correctness — CorrelationMemory.content is valid JSON
    // carrying the run id, outcome, and cost_tokens of the source row.
    #[test]
    fn correlation_memory_content_is_json_with_run_fields() {
        let ns = assert_workflow_trace_namespace(&canonical_ns()).expect("ns");
        let m = CorrelationMemory::from_row(&run(), &ns, false);
        let parsed: serde_json::Value =
            serde_json::from_str(&m.content).expect("content is JSON");
        assert_eq!(parsed.get("run_id").and_then(serde_json::Value::as_i64), Some(42));
        assert_eq!(
            parsed.get("outcome").and_then(serde_json::Value::as_str),
            Some("ok")
        );
        assert_eq!(
            parsed.get("cost_tokens").and_then(serde_json::Value::as_i64),
            Some(100)
        );
    }

    // rationale: F9 zero-weight — when the source row has no outcome,
    // content carries "unknown" and relevance maps to the 0.1 bucket.
    #[test]
    fn correlation_memory_none_outcome_maps_to_unknown_and_lowest_relevance() {
        let ns = assert_workflow_trace_namespace(&canonical_ns()).expect("ns");
        let mut r = run();
        r.outcome = None;
        let m = CorrelationMemory::from_row(&r, &ns, false);
        assert!((m.relevance - 0.1).abs() < 1e-6, "None outcome relevance");
        let parsed: serde_json::Value = serde_json::from_str(&m.content).expect("json");
        assert_eq!(
            parsed.get("outcome").and_then(serde_json::Value::as_str),
            Some("unknown"),
            "None outcome rendered as 'unknown' in content"
        );
    }

    // rationale: F9 zero-weight — content's cost_tokens stays JSON null
    // when the source row's cost is None (never collapses to 0).
    #[test]
    fn correlation_memory_content_cost_tokens_null_when_none() {
        let ns = assert_workflow_trace_namespace(&canonical_ns()).expect("ns");
        let mut r = run();
        r.cost_tokens = None;
        let m = CorrelationMemory::from_row(&r, &ns, false);
        let parsed: serde_json::Value = serde_json::from_str(&m.content).expect("json");
        assert!(
            parsed.get("cost_tokens").expect("field present").is_null(),
            "None cost must serialise as JSON null, not 0"
        );
    }

    // rationale: Correctness — session_id on the memory is derived from
    // the source row id (string form).
    #[test]
    fn correlation_memory_session_id_is_row_id_string() {
        let ns = assert_workflow_trace_namespace(&canonical_ns()).expect("ns");
        let m = CorrelationMemory::from_row(&run(), &ns, false);
        assert_eq!(m.session_id, "42", "session_id is row id 42 as string");
    }

    // rationale: Contract regression — CorrelationMemory round-trips
    // through serde (it crosses the SpacetimeDB SDK boundary).
    #[test]
    fn correlation_memory_serde_round_trip() {
        let ns = assert_workflow_trace_namespace(&canonical_ns()).expect("ns");
        let m = CorrelationMemory::from_row(&run(), &ns, true);
        let s = serde_json::to_string(&m).expect("ser");
        let back: CorrelationMemory = serde_json::from_str(&s).expect("de");
        assert_eq!(back.namespace, m.namespace);
        assert_eq!(back.content, m.content);
        assert!((back.relevance - m.relevance).abs() < 1e-6);
        assert_eq!(back.under_pressure, m.under_pressure);
        assert!(back.tensor.is_none());
    }

    // rationale: Error variant — StcortexWriterError Display strings are
    // human-readable and name the failure class.
    #[test]
    fn writer_error_display_strings_name_the_failure() {
        let we = StcortexWriterError::WriteFailed("boom".into());
        assert!(we.to_string().contains("substrate write failed"));
        assert!(we.to_string().contains("boom"));
    }

    // rationale: Error variant — NamespaceViolation converts into
    // StcortexWriterError via the #[from] impl (the AP30 boundary).
    #[test]
    fn namespace_violation_converts_into_writer_error() {
        let nv = crate::m9_watcher_namespace_guard::assert_workflow_trace_namespace(
            "orac_bad",
        )
        .unwrap_err();
        let we: StcortexWriterError = nv.into();
        assert!(matches!(we, StcortexWriterError::NamespaceViolation(_)));
    }

    // rationale: Anti-property — a write failure on the band-2
    // (under-pressure) path also propagates the typed error, not just
    // band-1.
    #[test]
    fn under_pressure_write_failure_propagates_typed_error() {
        let w = writer(Some(0.05), true);
        let err = w
            .promote_run(&run(), &canonical_ns())
            .expect_err("expected write fail in band-2");
        assert!(matches!(err, StcortexWriterError::WriteFailed(_)));
    }

    // rationale: Anti-property — when a band-3 write fails, NOTHING is
    // written to the outbox either (write-fail is a typed error, not a
    // silent defer).
    #[test]
    fn write_failure_does_not_silently_defer_to_outbox() {
        let w = writer(Some(0.50), true);
        let _ = w
            .promote_run(&run(), &canonical_ns())
            .expect_err("write must fail");
        // The outbox file may not exist at all (no defer happened); if it
        // does it must be empty.
        let contents = std::fs::read_to_string(w.outbox_path()).unwrap_or_default();
        assert!(
            contents.is_empty(),
            "write-fail must surface typed error, not silently defer: {contents}"
        );
    }

    // rationale: Boundary — DeferReason equality is structural; two
    // LtpBelowFloor with the same density compare equal, different
    // densities do not (PartialEq contract used in outcome matching).
    #[test]
    fn defer_reason_equality_is_structural() {
        let a = DeferReason::LtpBelowFloor { density: 0.01 };
        let b = DeferReason::LtpBelowFloor { density: 0.01 };
        let c = DeferReason::LtpBelowFloor { density: 0.02 };
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(DeferReason::OracUnreachable, DeferReason::StcortexUnreachable);
    }

    // rationale: Cross-module surface — a namespace with leading/trailing
    // whitespace is rejected by m9 before any probe or write fires.
    #[test]
    fn whitespace_namespace_rejected_before_probe() {
        let w = writer(Some(0.50), false);
        let err = w
            .promote_run(&run(), "workflow_trace x")
            .expect_err("whitespace must be rejected");
        assert!(matches!(err, StcortexWriterError::NamespaceViolation(_)));
        assert!(
            w.writer.written.lock().expect("lock").is_empty(),
            "rejected namespace must not write"
        );
    }
}
