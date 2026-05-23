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
//!
//! **Consumer-freshness refuse-write invariant (SEC1):** the stcortex
//! substrate enforces refuse-write-on-stale at the DB layer — a write
//! from a consumer whose subscription has gone stale is rejected. m13
//! mirrors that invariant *before* it ever issues a write: an optional
//! [`FreshnessGate`] (constructor-injected via
//! [`StcortexWriter::with_freshness_gate`], consistent with the
//! reader/writer dependency-injection style) is consulted at the top of
//! [`StcortexWriter::promote_run`]. When the gate reports the consumer is
//! not fresh, `promote_run` defers with [`DeferReason::StcortexUnreachable`]
//! instead of writing — this is the sole production constructor of that
//! variant. A [`RegistrationHandle`](crate::m2_stcortex_consumer::RegistrationHandle)
//! satisfies [`FreshnessGate`] directly (its `is_fresh()` is the gate
//! signal). When no gate is injected (`StcortexWriter::new_unchecked`),
//! the gate is treated as always-fresh — preserving the legacy signature
//! for the 3-band-only call sites.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use thiserror::Error;

use crate::m7_workflow_runs::{Outcome, WorkflowRunRow};
use crate::m9_watcher_namespace_guard::{
    assert_workflow_trace_namespace, NamespaceViolation, ValidatedNamespace,
};

/// Phase 1 floor — defer below this `substrate_LTP_density`.
pub const LTP_PHASE_1_FLOOR: f64 = 0.015;

/// Phase 3 target — full writes above this `substrate_LTP_density`.
pub const LTP_PHASE_3_TARGET: f64 = 0.10;

/// Default substrate-band probe timeout.
pub const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// SEC4 — hard cap on the bytes read from an ORAC blackboard HTTP
/// response. The blackboard density probe is a tiny JSON document; a
/// response larger than 1 MiB is pathological (compromised endpoint,
/// proxy error page, or contract drift). Capping the read at this bound
/// prevents a multi-GB body from forcing an unbounded allocation.
pub const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;

/// Stable consumer source tag emitted with every memory.
pub const SOURCE_TAG: &str = "workflow-trace-m13";

/// Failure modes for m13.
#[derive(Debug, Error)]
#[non_exhaustive]
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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
        // An open run (no outcome yet) is treated as `unknown` for both the
        // relevance heuristic and the persisted content digest.
        let outcome = row.run_state.outcome().unwrap_or(Outcome::Unknown);
        let relevance = match outcome {
            Outcome::Ok => 1.0_f32,
            Outcome::Fail => 0.5,
            Outcome::Abort => 0.3,
            Outcome::Unknown => 0.1,
        };
        let content = serde_json::json!({
            "run_id": row.id,
            "outcome": outcome.as_str(),
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
    /// The stcortex consumer subscription was not fresh (SEC1
    /// refuse-write-on-stale). Constructed by [`StcortexWriter::promote_run`]
    /// when an injected [`FreshnessGate`] reports `is_fresh() == false`;
    /// the run is deferred to the JSONL outbox rather than written against
    /// a stale consumer that the substrate would reject anyway.
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

/// Consumer-freshness signal for the SEC1 refuse-write-on-stale invariant.
///
/// stcortex rejects writes from a consumer whose subscription has gone
/// stale (the DB-layer refuse-write rule). m13 consults a `FreshnessGate`
/// *before* issuing any write so the deferral happens client-side rather
/// than surfacing as an opaque substrate `WriteFailed`. A
/// [`RegistrationHandle`](crate::m2_stcortex_consumer::RegistrationHandle)
/// satisfies this trait directly — its `is_fresh()` is the gate signal.
///
/// Injected via [`StcortexWriter::with_freshness_gate`]; when absent the
/// writer treats the consumer as always-fresh (legacy `new` path).
pub trait FreshnessGate: Send + Sync {
    /// `true` when the stcortex consumer subscription is live and applied.
    /// `false` when it has gone stale / disconnected — in which case
    /// [`StcortexWriter::promote_run`] defers with
    /// [`DeferReason::StcortexUnreachable`] rather than writing.
    fn is_fresh(&self) -> bool;
}

impl FreshnessGate for crate::m2_stcortex_consumer::RegistrationHandle {
    fn is_fresh(&self) -> bool {
        crate::m2_stcortex_consumer::RegistrationHandle::is_fresh(self)
    }
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
    /// - reading the body — body is not valid UTF-8 JSON, or exceeds the
    ///   [`MAX_RESPONSE_BYTES`] size cap (SEC4 unbounded-allocation guard).
    ///
    /// Rationale: ORAC blackboard transient unreachability is the EXPECTED
    /// failure mode and MUST NOT be propagated as a typed error; defer is
    /// the correct response per 3-band gate spec.
    ///
    /// **SEC4 (size cap):** the body is read via [`read_capped_body`], which
    /// honours `Content-Length` and hard-stops at [`MAX_RESPONSE_BYTES`].
    /// A multi-GB body can no longer force an unbounded allocation — an
    /// over-cap body collapses to the `None` unreachable sentinel.
    ///
    /// **SEC5 (range check):** a parsed density is accepted only when it is
    /// finite and within the documented `[0.0, 1.0]` `substrate_LTP_density`
    /// domain. A non-finite (`NaN` / `±inf`) or out-of-range value is
    /// rejected to `None` rather than silently feeding the 3-band LTP gate —
    /// a corrupt probe must defer, not band on garbage.
    fn read_density(&self) -> Option<f64> {
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()
            .ok()?;
        let resp = client.get(&self.url).send().ok()?;
        let bytes = read_capped_body(resp)?;
        let body: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        let raw = body
            .get("substrate_LTP_density")
            .and_then(serde_json::Value::as_f64)
            .or_else(|| body.as_f64())?;
        // SEC5: reject non-finite or out-of-domain density. The 3-band gate
        // only has meaning for a density in [0.0, 1.0]; anything else is a
        // corrupt probe and must defer (None), not band on a poisoned value.
        if raw.is_finite() && (0.0..=1.0).contains(&raw) {
            Some(raw)
        } else {
            tracing::warn!(
                target: "m13.read_density.out_of_range",
                raw,
                "ORAC density probe returned non-finite or out-of-[0,1] value; deferring"
            );
            None
        }
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
    /// SEC1 — optional consumer-freshness gate. `None` (the `new` path)
    /// means "always fresh"; `Some` (the `with_freshness_gate` path)
    /// enforces the refuse-write-on-stale invariant before any write.
    freshness: Option<std::sync::Arc<dyn FreshnessGate>>,
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
            .field("freshness_gate", &self.freshness.is_some())
            .finish_non_exhaustive()
    }
}

impl<R, W> StcortexWriter<R, W>
where
    R: LtpDensityReader,
    W: SubstrateWriter,
{
    /// Construct **without** a consumer-freshness gate (SEC1 disabled).
    ///
    /// ⚠️ This is the *unchecked* constructor: no consumer-freshness gate is
    /// installed, so the writer treats the stcortex consumer as always-fresh
    /// and the 3-band LTP gate is the sole admission control. The
    /// `_unchecked` suffix makes that opt-out **loud at the call site** —
    /// production wiring should prefer [`Self::with_freshness_gate`], which
    /// enforces the SEC1 refuse-write-on-stale invariant. This constructor
    /// exists for tests and for callers that deliberately want 3-band-only
    /// admission control.
    pub fn new_unchecked(reader: R, writer: W, outbox_path: PathBuf) -> Self {
        Self {
            reader,
            writer,
            freshness: None,
            outbox_path,
            outbox_lock: Mutex::new(()),
        }
    }

    /// Construct with an explicit consumer-freshness gate (SEC1).
    ///
    /// The `freshness` gate is consulted at the top of
    /// [`Self::promote_run`]: when it reports the stcortex consumer is not
    /// fresh, the run defers with [`DeferReason::StcortexUnreachable`]
    /// instead of writing — mirroring the substrate's DB-layer
    /// refuse-write-on-stale rule on the client side. Pass a
    /// [`RegistrationHandle`](crate::m2_stcortex_consumer::RegistrationHandle)
    /// (it implements [`FreshnessGate`]) or any custom gate.
    pub fn with_freshness_gate(
        reader: R,
        writer: W,
        freshness: std::sync::Arc<dyn FreshnessGate>,
        outbox_path: PathBuf,
    ) -> Self {
        Self {
            reader,
            writer,
            freshness: Some(freshness),
            outbox_path,
            outbox_lock: Mutex::new(()),
        }
    }

    /// Promote a completed workflow run. Performs:
    ///
    /// 1. m9 namespace validation (`namespace_key` MUST start with the
    ///    canonical [`crate::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX`]).
    /// 2. SEC1 consumer-freshness gate (when a [`FreshnessGate`] was
    ///    injected via [`Self::with_freshness_gate`]): if the stcortex
    ///    consumer is not fresh, defer with
    ///    [`DeferReason::StcortexUnreachable`] — never write against a
    ///    stale consumer. Mirrors the substrate's DB-layer
    ///    refuse-write-on-stale rule on the client side.
    /// 3. ORAC LTP density probe.
    /// 4. Either write or defer per 3-band gate.
    ///
    /// The freshness gate is checked *before* the LTP probe so a stale
    /// consumer fails fast without spending a network round-trip on ORAC.
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
        // SEC1 — refuse-write-on-stale: if a freshness gate is installed
        // and reports the consumer is not fresh, defer with
        // StcortexUnreachable before touching ORAC or the substrate.
        if let Some(gate) = self.freshness.as_deref() {
            if !gate.is_fresh() {
                let memory = CorrelationMemory::from_row(run, &validated, false);
                self.defer(&memory, &DeferReason::StcortexUnreachable)?;
                tracing::warn!(
                    target: "m13.promote.stale_consumer",
                    "stcortex consumer not fresh — deferring (SEC1 refuse-write-on-stale)"
                );
                return Ok(PromoteOutcome::Deferred {
                    reason: DeferReason::StcortexUnreachable,
                });
            }
        }
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

    /// Borrow the outbox cursor sidecar path (`<outbox_path>.cursor`).
    ///
    /// C1 NA-GAP-06 drain skeleton (Plan v2 v0.2.0 §3 Phase 3 step 2).
    /// The cursor sidecar persists the byte-offset already consumed by
    /// the V1 RefusalToken consumer (wired Phase 7 via
    /// [`drain_to_refusal_tokens`]; ADR D-S1004XXX-04 §1.2 m13 row).
    /// When absent, the drain starts at byte-offset 0 (read the entire
    /// outbox).
    ///
    /// [`drain_to_refusal_tokens`]: StcortexWriter::drain_to_refusal_tokens
    #[must_use]
    #[allow(dead_code)] // production drain-consumer wire is post-v0.2.0 per §11
    pub(crate) fn outbox_cursor_path(&self) -> PathBuf {
        let mut p = self.outbox_path.clone();
        let mut s = p
            .file_name()
            .map(|os| os.to_string_lossy().into_owned())
            .unwrap_or_default();
        s.push_str(".cursor");
        p.set_file_name(s);
        p
    }

    /// Drain the outbox starting from the persisted cursor position.
    ///
    /// C1 NA-GAP-06 drain skeleton (Plan v2 v0.2.0 §3 Phase 3 step 2).
    /// Private API (`pub(crate)`) — the external consumer is V1
    /// RefusalToken-typed and wired in Phase 5 per ADR D-S1004XXX-04
    /// §1.2; this skeleton stops short of consumer-side delivery.
    ///
    /// Idempotent at-least-once semantics: the caller persists the cursor
    /// via [`commit_drain_cursor`] **only after** successfully consuming
    /// the returned entries. A failed or absent commit leaves the cursor
    /// unchanged, so re-running [`drain_outbox`] re-returns the same
    /// entries (replay-safe). The cursor sidecar lives at
    /// [`outbox_cursor_path`] (`<outbox_path>.cursor`).
    ///
    /// # Errors
    ///
    /// - [`StcortexWriterError::OutboxIo`] if the outbox file or cursor
    ///   sidecar I/O fails (read, seek, or open).
    /// - [`StcortexWriterError::OutboxSerde`] if a JSONL line fails to
    ///   parse as a valid [`OutboxEntry`] — the drain is fail-fast and
    ///   does **not** skip malformed lines (the cursor is unchanged so
    ///   the caller can repair the outbox + re-drain).
    ///
    /// [`commit_drain_cursor`]: StcortexWriter::commit_drain_cursor
    /// [`outbox_cursor_path`]: StcortexWriter::outbox_cursor_path
    #[allow(dead_code)] // production drain-consumer wire is post-v0.2.0 per §11
    pub(crate) fn drain_outbox(&self) -> Result<DrainResult, StcortexWriterError> {
        use std::io::BufRead;
        // Cursor read: absent file == 0 offset (first drain).
        let cursor_path = self.outbox_cursor_path();
        let start_offset: u64 = match std::fs::read_to_string(&cursor_path) {
            Ok(s) => s.trim().parse::<u64>().unwrap_or(0),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
            Err(e) => return Err(StcortexWriterError::OutboxIo(e)),
        };
        // Outbox-absent → empty drain (semantically equivalent to no entries
        // and cursor unchanged). The `defer_to_outbox` path creates the file
        // on first write; reading-before-any-write is a valid drain state.
        if !self.outbox_path.exists() {
            return Ok(DrainResult {
                entries: Vec::new(),
                new_cursor: start_offset,
            });
        }
        // Take the outbox lock so a concurrent `defer_to_outbox` write
        // cannot interleave between our read-seek and the line scan
        // (poison-recovery via PoisonError::into_inner matches the write
        // path's discipline at L506-513).
        let _guard = self
            .outbox_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let f = std::fs::File::open(&self.outbox_path)?;
        let total_len = f.metadata()?.len();
        // Defensive: a shorter outbox than the cursor implies external
        // truncation or rotation; reset to 0 rather than skip past EOF.
        let effective_start = if start_offset > total_len {
            0
        } else {
            start_offset
        };
        let mut reader = std::io::BufReader::new(f);
        if effective_start > 0 {
            use std::io::Seek;
            reader.seek(std::io::SeekFrom::Start(effective_start))?;
        }
        let mut entries = Vec::new();
        let mut new_cursor = effective_start;
        for line_result in reader.lines() {
            let line = line_result?;
            new_cursor += u64::try_from(line.len()).unwrap_or(0) + 1; // +1 for `\n`
            if line.trim().is_empty() {
                continue; // blank lines tolerated (defensive)
            }
            let entry: OutboxEntry = serde_json::from_str(&line)?;
            entries.push(entry);
        }
        Ok(DrainResult {
            entries,
            new_cursor,
        })
    }

    /// Persist a new cursor offset to the outbox cursor sidecar.
    ///
    /// C1 NA-GAP-06 drain skeleton commit step. Atomic-rename via
    /// write-temp-then-rename to avoid leaving the cursor in a
    /// partial-write state under a crash mid-commit.
    ///
    /// # Errors
    ///
    /// - [`StcortexWriterError::OutboxIo`] if the temp-write or atomic
    ///   rename fails.
    #[allow(dead_code)] // production drain-consumer wire is post-v0.2.0 per §11
    pub(crate) fn commit_drain_cursor(&self, new_cursor: u64) -> Result<(), StcortexWriterError> {
        let cursor_path = self.outbox_cursor_path();
        let mut tmp_path = cursor_path.clone();
        let mut tmp_name = tmp_path
            .file_name()
            .map(|os| os.to_string_lossy().into_owned())
            .unwrap_or_default();
        tmp_name.push_str(".tmp");
        tmp_path.set_file_name(tmp_name);
        if let Some(parent) = cursor_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&tmp_path, new_cursor.to_string())?;
        std::fs::rename(&tmp_path, &cursor_path)?;
        Ok(())
    }

    /// Drain the outbox and convert each entry into a V1 `RefusalToken`
    /// per ADR D-S1004XXX-04 §1.2 m13 row + Plan v2 v0.2.0 §3 Phase 7
    /// (consumer wire that the Phase 3 C1 skeleton was designed for).
    ///
    /// Each deferred entry becomes a [`RefusalToken::SubstrateAuthored`]
    /// with `substrate_id = SubstrateId::Stcortex` and a
    /// `substrate_reason` derived from the deferred [`DeferReason`]:
    /// - `LtpBelowFloor { density }` → `"ltp_below_floor:density={d}"`
    /// - `OracUnreachable` → `"orac_unreachable"`
    /// - `StcortexUnreachable` → `"stcortex_unreachable"`
    ///
    /// The cursor advances only on successful consumption (per
    /// [`drain_outbox`]'s at-least-once contract); callers MUST call
    /// [`commit_drain_cursor`] after persisting / forwarding the tokens
    /// to honour idempotent-replay semantics.
    ///
    /// # Errors
    ///
    /// Same as [`drain_outbox`]: [`StcortexWriterError::OutboxIo`] for
    /// I/O; [`StcortexWriterError::OutboxSerde`] for malformed JSONL.
    ///
    /// [`drain_outbox`]: StcortexWriter::drain_outbox
    /// [`commit_drain_cursor`]: StcortexWriter::commit_drain_cursor
    #[allow(dead_code)] // production drain-consumer wire is post-v0.2.0 per §11
    pub(crate) fn drain_to_refusal_tokens(
        &self,
    ) -> Result<TokenDrainResult, StcortexWriterError> {
        let result = self.drain_outbox()?;
        let tokens: Vec<crate::refusal_token::RefusalToken> = result
            .entries
            .iter()
            .map(outbox_entry_to_refusal_token)
            .collect();
        Ok(TokenDrainResult {
            tokens,
            new_cursor: result.new_cursor,
        })
    }
}

/// Convert an OutboxEntry to a V1 RefusalToken per ADR D-S1004XXX-04 §1.2
/// m13 row. Free function so tests can exercise without a live
/// StcortexWriter.
#[allow(dead_code)] // production drain-consumer wire is post-v0.2.0 per §11
fn outbox_entry_to_refusal_token(
    entry: &OutboxEntry,
) -> crate::refusal_token::RefusalToken {
    // Render the reason as a structured string. The reason was serialised
    // as serde_json::Value at defer time (see defer_to_outbox at L478-488);
    // we parse the tag-or-string shape here without re-typing.
    let substrate_reason = if let Some(s) = entry.reason.as_str() {
        // Tag-only forms: "OracUnreachable" / "StcortexUnreachable" →
        // snake_case for substrate vocabulary consistency.
        match s {
            "OracUnreachable" => "orac_unreachable".to_owned(),
            "StcortexUnreachable" => "stcortex_unreachable".to_owned(),
            other => other.to_owned(),
        }
    } else if let Some(obj) = entry.reason.as_object() {
        // Tagged form: {"LtpBelowFloor": {"density": <f64>}}.
        if let Some(ltp) = obj.get("LtpBelowFloor").and_then(|v| v.get("density")) {
            format!("ltp_below_floor:density={ltp}")
        } else {
            // Defensive fallback: emit the JSON serialised form.
            entry.reason.to_string()
        }
    } else {
        entry.reason.to_string()
    };
    crate::refusal_token::RefusalToken::substrate_authored(
        crate::refusal_token::SubstrateId::Stcortex,
        substrate_reason,
    )
}

/// Result of [`StcortexWriter::drain_to_refusal_tokens`] — parallel to
/// [`DrainResult`] but with V1 RefusalToken envelopes instead of raw
/// OutboxEntries.
#[derive(Debug, Clone)]
#[allow(dead_code)] // production drain-consumer wire is post-v0.2.0 per §11
pub(crate) struct TokenDrainResult {
    /// V1 RefusalToken envelopes (one per drained outbox entry).
    pub tokens: Vec<crate::refusal_token::RefusalToken>,
    /// New cursor offset; pass to `commit_drain_cursor` after consuming.
    pub new_cursor: u64,
}

/// Outbox entry deserialised from the JSONL stream produced by
/// `defer_to_outbox`. C1 NA-GAP-06 drain skeleton support type
/// (Plan v2 v0.2.0 §3 Phase 3 step 2).
///
/// Mirrors the on-disk JSONL shape:
/// `{"ts_ms": <int|null>, "memory": {...}, "reason": {...}, "clock_unavailable": bool?}`
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[allow(dead_code)] // production drain-consumer wire is post-v0.2.0 per §11
pub(crate) struct OutboxEntry {
    /// Wall-clock ms at defer-time; `None` when `now_ms()` returned `None`
    /// (clock fault — see write path at L472-499). Paired with
    /// `clock_unavailable: true` in that case.
    pub ts_ms: Option<u64>,
    /// The correlation memory that was deferred.
    pub memory: CorrelationMemory,
    /// The defer-reason (`LtpBelowFloor` / `OracUnreachable` / `StcortexUnreachable`).
    pub reason: serde_json::Value, // tag-or-typed deser deferred to Phase 5 V1 consumer wire
    /// True when `now_ms()` was unavailable at defer-time (F-POVM-07 tag).
    #[serde(default)]
    pub clock_unavailable: bool,
}

/// Result of [`StcortexWriter::drain_outbox`].
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // production drain-consumer wire is post-v0.2.0 per §11
pub(crate) struct DrainResult {
    /// Entries read since the last committed cursor.
    pub entries: Vec<OutboxEntry>,
    /// New cursor offset (byte-position past the last entry read). Caller
    /// passes this to [`StcortexWriter::commit_drain_cursor`] **only after**
    /// successfully consuming the entries.
    pub new_cursor: u64,
}

/// SEC4 — read an HTTP response body with a hard [`MAX_RESPONSE_BYTES`]
/// size cap. Returns `None` (the m13 "probe unreachable" sentinel) when:
///
/// - `Content-Length` is advertised and already exceeds the cap — the
///   body is rejected *before* a single byte is buffered;
/// - the body has no `Content-Length` (chunked / streamed) but the actual
///   stream exceeds the cap — the read is bounded by `Read::take()` and a
///   `>`-cap result is rejected;
/// - the underlying stream errors mid-read.
///
/// A multi-GB body therefore can never force an unbounded allocation: at
/// most `MAX_RESPONSE_BYTES + 1` bytes are ever held.
fn read_capped_body(resp: reqwest::blocking::Response) -> Option<Vec<u8>> {
    use std::io::Read;
    // Early reject: a server-advertised length over the cap never gets read.
    if let Some(len) = resp.content_length() {
        if len > MAX_RESPONSE_BYTES {
            tracing::warn!(
                target: "m13.read_capped_body.over_cap",
                content_length = len,
                cap = MAX_RESPONSE_BYTES,
                "ORAC response Content-Length exceeds cap; deferring"
            );
            return None;
        }
    }
    // Bounded read: take at most cap+1 bytes. If we actually buffered
    // cap+1, the stream was larger than the cap (chunked / mislabelled
    // Content-Length) — reject rather than continue.
    let mut buf = Vec::new();
    let read_limit = MAX_RESPONSE_BYTES.saturating_add(1);
    resp.take(read_limit).read_to_end(&mut buf).ok()?;
    if buf.len() as u64 > MAX_RESPONSE_BYTES {
        tracing::warn!(
            target: "m13.read_capped_body.over_cap",
            cap = MAX_RESPONSE_BYTES,
            "ORAC response stream exceeded cap (chunked / mislabelled); deferring"
        );
        return None;
    }
    Some(buf)
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
    use crate::m7_workflow_runs::{Outcome, RunState, WorkflowRunRow};
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
            run_state: RunState::Closed {
                ended_at: "2026-05-17T01:00:00Z".into(),
                outcome: Outcome::Ok,
            },
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

    /// Build a fresh, zero-state [`RecordingWriter`]; `fail` selects the
    /// write-error mock.
    fn recording_writer(fail: bool) -> RecordingWriter {
        RecordingWriter {
            next_id: StdMutex::new(0),
            written: StdMutex::new(Vec::new()),
            fail,
        }
    }

    /// Create a `.jsonl` temp outbox path that survives `Drop` (the file is
    /// intentionally leaked so the writer-under-test can read it back).
    fn temp_outbox_path() -> std::path::PathBuf {
        let outbox = tempfile::Builder::new()
            .suffix(".jsonl")
            .tempfile()
            .expect("temp")
            .into_temp_path();
        let path = outbox.to_path_buf();
        // Forget the temp_path so the file is not deleted on drop.
        std::mem::forget(outbox);
        path
    }

    fn writer(
        density: Option<f64>,
        fail_writes: bool,
    ) -> StcortexWriter<StaticDensity, RecordingWriter> {
        StcortexWriter::new_unchecked(
            StaticDensity(density),
            recording_writer(fail_writes),
            temp_outbox_path(),
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
            (Outcome::Ok, 1.0_f32),
            (Outcome::Fail, 0.5),
            (Outcome::Abort, 0.3),
            (Outcome::Unknown, 0.1),
        ] {
            let mut r = run();
            r.run_state = RunState::Closed {
                ended_at: "2026-05-17T01:00:00Z".into(),
                outcome,
            };
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
        let w = StcortexWriter::new_unchecked(
            PanickingReader,
            recording_writer(false),
            temp_outbox_path(),
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

    // rationale: Type-design invariant — unrecognised outcome strings are
    // now structurally unrepresentable (`run_state` carries a typed
    // `Outcome`). The remaining "no clear outcome" case is an open run
    // (`RunState::Open`); `from_row` collapses it to the `Outcome::Unknown`
    // relevance bucket (0.1).
    #[test]
    fn open_run_collapses_to_unknown_relevance() {
        // rationale: Type-design invariant
        let ns = assert_workflow_trace_namespace(&canonical_ns()).expect("ns");
        let mut r = run();
        r.run_state = RunState::Open;
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
        r.run_state = RunState::Open;
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

    // ====================================================================
    // W2 hardening pass (SEC1 + SEC5) — m13 stcortex writer.
    // SEC1: consumer-freshness refuse-write-on-stale invariant.
    // SEC5: ORAC density range / finiteness validation.
    // ====================================================================

    use super::FreshnessGate;

    /// Test-only freshness gate with a fixed verdict.
    struct StaticFreshness(bool);
    impl FreshnessGate for StaticFreshness {
        fn is_fresh(&self) -> bool {
            self.0
        }
    }

    fn writer_with_gate(
        density: Option<f64>,
        fresh: bool,
    ) -> StcortexWriter<StaticDensity, RecordingWriter> {
        StcortexWriter::with_freshness_gate(
            StaticDensity(density),
            recording_writer(false),
            Arc::new(StaticFreshness(fresh)),
            temp_outbox_path(),
        )
    }

    // rationale: SEC1 — a stale consumer (gate reports !is_fresh) must
    // defer with StcortexUnreachable, the variant that was previously
    // never constructed in production. No substrate write occurs.
    #[test]
    fn stale_consumer_defers_with_stcortex_unreachable() {
        // rationale: SEC1 (refuse-write-on-stale invariant)
        let w = writer_with_gate(Some(0.50), false);
        let out = w
            .promote_run(&run(), &canonical_ns())
            .expect("defer, not error");
        assert_eq!(
            out,
            PromoteOutcome::Deferred {
                reason: DeferReason::StcortexUnreachable
            },
            "stale consumer must defer with StcortexUnreachable"
        );
        assert!(
            w.writer.written.lock().expect("lock").is_empty(),
            "stale consumer must NOT write to the substrate"
        );
        let contents = std::fs::read_to_string(w.outbox_path()).expect("read outbox");
        assert!(
            contents.contains("StcortexUnreachable"),
            "defer reason must reach the JSONL outbox: {contents}"
        );
    }

    // rationale: SEC1 — a fresh consumer (gate reports is_fresh) writes
    // normally; the gate does not interfere with the happy path.
    #[test]
    fn fresh_consumer_writes_normally() {
        // rationale: SEC1 (gate does not block a fresh consumer)
        let w = writer_with_gate(Some(0.50), true);
        let out = w.promote_run(&run(), &canonical_ns()).expect("promote");
        assert!(
            matches!(out, PromoteOutcome::Written { .. }),
            "fresh consumer with healthy density must write; got {out:?}"
        );
        assert_eq!(w.writer.written.lock().expect("lock").len(), 1);
    }

    // rationale: SEC1 — the freshness gate is consulted BEFORE the LTP
    // probe. A stale consumer must fail-fast: the density reader is never
    // touched (a panicking reader proves the probe did not run).
    #[test]
    fn freshness_gate_precedes_ltp_probe() {
        // rationale: SEC1 (order-of-operations — gate before probe)
        struct PanickingReader;
        impl LtpDensityReader for PanickingReader {
            fn read_density(&self) -> Option<f64> {
                panic!("LTP probe must not run when the consumer is stale");
            }
        }
        let w = StcortexWriter::with_freshness_gate(
            PanickingReader,
            recording_writer(false),
            Arc::new(StaticFreshness(false)),
            temp_outbox_path(),
        );
        let out = w
            .promote_run(&run(), &canonical_ns())
            .expect("stale defer must not error");
        assert_eq!(
            out,
            PromoteOutcome::Deferred {
                reason: DeferReason::StcortexUnreachable
            }
        );
    }

    // rationale: SEC1 — the freshness gate is consulted AFTER namespace
    // validation. A foreign namespace must still fail with
    // NamespaceViolation even when the consumer is stale (AP30 wins).
    #[test]
    fn namespace_violation_precedes_freshness_gate() {
        // rationale: SEC1 (AP30 order-of-operations preserved)
        let w = writer_with_gate(Some(0.50), false);
        let err = w
            .promote_run(&run(), "orac_foreign")
            .expect_err("foreign namespace");
        assert!(
            matches!(err, StcortexWriterError::NamespaceViolation(_)),
            "namespace check must precede the freshness gate"
        );
    }

    // rationale: SEC1 — the legacy `new` constructor installs no gate;
    // promote_run behaves exactly as before (no StcortexUnreachable path).
    #[test]
    fn writer_without_gate_never_defers_stcortex_unreachable() {
        // rationale: SEC1 (legacy path preserved — gate is opt-in)
        let w = writer(Some(0.50), false);
        let out = w.promote_run(&run(), &canonical_ns()).expect("promote");
        assert!(
            matches!(out, PromoteOutcome::Written { .. }),
            "no-gate writer must write; got {out:?}"
        );
    }

    // rationale: SEC1 — RegistrationHandle satisfies FreshnessGate; this
    // pins the cross-module impl so a future RegistrationHandle refactor
    // that breaks the trait bound is caught at compile time.
    #[test]
    fn registration_handle_satisfies_freshness_gate_bound() {
        // rationale: SEC1 (cross-module trait-impl regression)
        fn assert_gate<T: FreshnessGate>() {}
        assert_gate::<crate::m2_stcortex_consumer::RegistrationHandle>();
    }

    // rationale: SEC5 — documents WHY OracHttpReader must reject NaN at
    // the reader boundary: every band comparison `d < FLOOR` / `d < TARGET`
    // is false for NaN, so an un-rejected NaN would silently fall through
    // to the band-3 normal-write arm. The reader's finiteness check is the
    // only thing standing between a corrupt probe and a poisoned write.
    // The real HTTP-path rejection is proven by the wiremock
    // `orac_reader_rejects_*` tests below.
    #[test]
    fn nan_density_would_bypass_band_gate_without_sec5_reject() {
        // rationale: SEC5 (non-finite density must not band)
        let nan = f64::NAN;
        assert!(!nan.is_finite(), "NaN is non-finite by definition");
        // Both band comparisons are false for NaN — it would NOT defer
        // and would NOT be flagged under_pressure; it would write normally.
        // `partial_cmp` is None for NaN, which makes the band-`<` check
        // false in production — exactly the silent fall-through SEC5 guards.
        assert_eq!(
            nan.partial_cmp(&LTP_PHASE_1_FLOOR),
            None,
            "NaN is incomparable to the floor — band-`<` is false"
        );
        assert_eq!(
            nan.partial_cmp(&LTP_PHASE_3_TARGET),
            None,
            "NaN is incomparable to the target — band-`<` is false"
        );
        // The SEC5 reject predicate catches it before banding.
        assert!(
            !(0.0..=1.0).contains(&nan),
            "NaN is outside [0,1] — the SEC5 reject predicate"
        );
    }

    // rationale: SEC5 — the validation predicate used by OracHttpReader
    // accepts exactly the [0.0, 1.0] finite domain and rejects everything
    // else. This pins the predicate independently of the HTTP path.
    #[test]
    fn sec5_density_validation_predicate_domain() {
        // rationale: SEC5 (range predicate boundary table)
        let accept =
            |d: f64| d.is_finite() && (0.0..=1.0).contains(&d);
        // In-domain — accepted.
        assert!(accept(0.0), "0.0 is the lower bound");
        assert!(accept(1.0), "1.0 is the upper bound");
        assert!(accept(0.018), "a realistic substrate_LTP_density");
        assert!(accept(LTP_PHASE_1_FLOOR));
        assert!(accept(LTP_PHASE_3_TARGET));
        // Out-of-domain — rejected.
        assert!(!accept(-0.0001), "just below 0 is rejected");
        assert!(!accept(1.0001), "just above 1 is rejected");
        assert!(!accept(f64::NAN), "NaN is rejected");
        assert!(!accept(f64::INFINITY), "+inf is rejected");
        assert!(!accept(f64::NEG_INFINITY), "-inf is rejected");
        assert!(!accept(1e308), "huge finite-but-out-of-range is rejected");
    }

    // rationale: SEC4 — MAX_RESPONSE_BYTES is the documented 1 MiB cap.
    // Drift detection.
    #[test]
    fn max_response_bytes_is_one_mib() {
        // rationale: SEC4 (size-cap constant pin)
        assert_eq!(super::MAX_RESPONSE_BYTES, 1024 * 1024);
    }

    // ====================================================================
    // W2 hardening — OracHttpReader end-to-end (SEC4 + SEC5) via wiremock.
    // Exercises the real HTTP path so the range check and size cap are
    // proven, not just the predicates.
    // ====================================================================

    use super::OracHttpReader;

    // rationale: SEC5 — a well-formed in-range density flows through the
    // real HTTP reader and is returned verbatim.
    #[tokio::test(flavor = "current_thread")]
    async fn orac_reader_returns_in_range_density() {
        // rationale: SEC5 (happy path — in-domain density)
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"substrate_LTP_density":0.018}"#)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;
        let url = server.uri();
        let d = tokio::task::spawn_blocking(move || {
            OracHttpReader::new(url, std::time::Duration::from_secs(2)).read_density()
        })
        .await
        .expect("join");
        assert!(
            d.is_some_and(|v| (v - 0.018).abs() < 1e-12),
            "in-range density must be returned; got {d:?}"
        );
    }

    // rationale: SEC5 — an out-of-range density (> 1.0) from a corrupt
    // probe must be rejected to None, NOT fed into the 3-band gate.
    #[tokio::test(flavor = "current_thread")]
    async fn orac_reader_rejects_out_of_range_density() {
        // rationale: SEC5 (out-of-[0,1] rejection)
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"substrate_LTP_density":42.0}"#)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;
        let url = server.uri();
        let d = tokio::task::spawn_blocking(move || {
            OracHttpReader::new(url, std::time::Duration::from_secs(2)).read_density()
        })
        .await
        .expect("join");
        assert_eq!(d, None, "out-of-range density must be rejected to None");
    }

    // rationale: SEC5 — a non-finite density (JSON cannot encode NaN, so
    // we exercise +inf via an explicitly-huge value already covered;
    // here we use a negative density which is also out-of-domain).
    #[tokio::test(flavor = "current_thread")]
    async fn orac_reader_rejects_negative_density() {
        // rationale: SEC5 (negative density rejection)
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"substrate_LTP_density":-0.5}"#)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;
        let url = server.uri();
        let d = tokio::task::spawn_blocking(move || {
            OracHttpReader::new(url, std::time::Duration::from_secs(2)).read_density()
        })
        .await
        .expect("join");
        assert_eq!(d, None, "negative density must be rejected to None");
    }

    // rationale: SEC4 — an over-cap response body (Content-Length beyond
    // MAX_RESPONSE_BYTES) is rejected to None before allocation. The
    // reader must not OOM on a multi-MiB body.
    #[tokio::test(flavor = "current_thread")]
    async fn orac_reader_rejects_over_cap_body() {
        // rationale: SEC4 (unbounded-allocation guard)
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // Body is 2 MiB — well over the 1 MiB cap. wiremock sets a real
        // Content-Length so the early-reject branch fires.
        let huge = "x".repeat(2 * 1024 * 1024);
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(huge)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;
        let url = server.uri();
        let d = tokio::task::spawn_blocking(move || {
            OracHttpReader::new(url, std::time::Duration::from_secs(3)).read_density()
        })
        .await
        .expect("join");
        assert_eq!(d, None, "an over-cap response body must be rejected to None");
    }

    // rationale: SEC4 — a body just under the cap that is valid JSON
    // still parses (the cap rejects only genuine over-cap bodies).
    #[tokio::test(flavor = "current_thread")]
    async fn orac_reader_accepts_under_cap_body() {
        // rationale: SEC4 (cap does not reject legitimate small bodies)
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // Pad the JSON with a large-but-under-cap whitespace-free field so
        // the body is sizeable yet still well under 1 MiB and valid JSON.
        let pad = "a".repeat(512 * 1024);
        let body = format!(r#"{{"pad":"{pad}","substrate_LTP_density":0.05}}"#);
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(body)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;
        let url = server.uri();
        let d = tokio::task::spawn_blocking(move || {
            OracHttpReader::new(url, std::time::Duration::from_secs(3)).read_density()
        })
        .await
        .expect("join");
        assert!(
            d.is_some_and(|v| (v - 0.05).abs() < 1e-12),
            "an under-cap valid body must parse; got {d:?}"
        );
    }

    // ========================================================================
    // C1 NA-GAP-06 drain skeleton tests (Plan v2 v0.2.0 §3 Phase 3 step 2).
    // The drain is `pub(crate)`; no external consumer yet (V1
    // RefusalToken-typed consumer wires in Phase 5). These tests assert the
    // *idempotent at-least-once replay* contract that the Phase 5 consumer
    // will rely on.
    // ========================================================================

    #[test]
    fn drain_outbox_empty_when_outbox_absent_returns_zero_entries_cursor_zero() {
        // Setup: writer with a temp outbox path that doesn't exist yet
        // (no defer_to_outbox has run).
        let w = writer(Some(0.20), false); // density above floor: no defers
        let result = w.drain_outbox().expect("drain of absent outbox is clean");
        assert!(result.entries.is_empty(), "no entries when no outbox file");
        assert_eq!(result.new_cursor, 0, "cursor stays at 0 when outbox absent");
    }

    #[test]
    fn drain_outbox_reads_all_entries_when_no_cursor_persisted() {
        // Setup: writer below floor; defer 3 entries.
        let w = writer(Some(0.001), false);
        for _ in 0..3 {
            let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        }
        // First drain (no cursor file): reads all 3.
        let result = w.drain_outbox().expect("drain");
        assert_eq!(result.entries.len(), 3, "all 3 deferred entries surfaced");
        assert!(
            result.new_cursor > 0,
            "cursor advances past the 3 lines (got {})",
            result.new_cursor
        );
        // Each entry has the LtpBelowFloor reason shape and the
        // canonical-namespace memory.
        for entry in &result.entries {
            assert!(entry.ts_ms.is_some(), "ts_ms populated when clock OK");
            assert!(!entry.clock_unavailable, "clock_unavailable false in happy path");
            let reason_str = entry.reason.to_string();
            assert!(
                reason_str.contains("LtpBelowFloor"),
                "expected LtpBelowFloor reason, got {reason_str}"
            );
        }
    }

    #[test]
    fn drain_replay_returns_same_entries_when_cursor_not_committed() {
        // Setup: defer 2 entries.
        let w = writer(Some(0.001), false);
        for _ in 0..2 {
            let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        }
        // First drain: returns 2 entries + new_cursor.
        let first = w.drain_outbox().expect("first drain");
        assert_eq!(first.entries.len(), 2, "first drain reads 2 entries");
        // DO NOT commit the cursor — simulate consumer failure mid-delivery.
        // Second drain: re-returns the SAME 2 entries (replay-safe).
        let second = w.drain_outbox().expect("second drain (replay)");
        assert_eq!(
            second.entries.len(),
            2,
            "replay returns same 2 entries when cursor not committed"
        );
        assert_eq!(
            first.entries, second.entries,
            "replay entries byte-for-byte identical"
        );
        assert_eq!(
            first.new_cursor, second.new_cursor,
            "replay cursor proposal identical"
        );
    }

    #[test]
    fn drain_after_commit_returns_only_new_entries() {
        // Setup: defer 3 entries, drain + commit cursor.
        let w = writer(Some(0.001), false);
        for _ in 0..3 {
            let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        }
        let first = w.drain_outbox().expect("first drain");
        assert_eq!(first.entries.len(), 3);
        w.commit_drain_cursor(first.new_cursor)
            .expect("commit cursor");
        // Second drain (post-commit): no new entries.
        let empty = w.drain_outbox().expect("post-commit drain");
        assert!(empty.entries.is_empty(), "no entries after commit absorbs them");
        assert_eq!(
            empty.new_cursor, first.new_cursor,
            "cursor unchanged when no new content"
        );
        // Defer 2 more entries; drain reads ONLY the new ones.
        for _ in 0..2 {
            let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        }
        let delta = w.drain_outbox().expect("delta drain");
        assert_eq!(
            delta.entries.len(),
            2,
            "delta drain reads only the 2 new entries after commit"
        );
        assert!(
            delta.new_cursor > first.new_cursor,
            "cursor advances past the new entries (was {}, now {})",
            first.new_cursor,
            delta.new_cursor
        );
    }

    #[test]
    fn commit_drain_cursor_persists_atomically_via_temp_rename() {
        // Setup: defer 1 entry; drain; commit.
        let w = writer(Some(0.001), false);
        let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        let result = w.drain_outbox().expect("drain");
        let cursor_value = result.new_cursor;
        w.commit_drain_cursor(cursor_value).expect("commit");
        // Cursor sidecar file exists at outbox_path + ".cursor" with the
        // expected decimal-string content; the .tmp sidecar is absent
        // (atomic rename completed).
        let cursor_contents = std::fs::read_to_string(w.outbox_cursor_path())
            .expect("cursor sidecar present after commit");
        assert_eq!(
            cursor_contents.trim().parse::<u64>().expect("decimal u64"),
            cursor_value,
            "cursor contents match committed value"
        );
        let mut tmp_path = w.outbox_cursor_path();
        let mut name = tmp_path
            .file_name()
            .map(|os| os.to_string_lossy().into_owned())
            .unwrap_or_default();
        name.push_str(".tmp");
        tmp_path.set_file_name(name);
        assert!(
            !tmp_path.exists(),
            "atomic rename should not leave .tmp sidecar"
        );
    }

    // ========================================================================
    // Phase 7 V1 drain wire — drain_to_refusal_tokens consumer tests
    // (Plan v2 v0.2.0 §3 Phase 7 per ADR D-S1004XXX-04 §1.2 m13 row).
    // ========================================================================

    #[test]
    fn drain_to_refusal_tokens_empty_outbox_returns_no_tokens() {
        let w = writer(Some(0.20), false); // density above floor: no defers
        let result = w.drain_to_refusal_tokens().expect("drain");
        assert!(result.tokens.is_empty(), "no tokens when no outbox");
    }

    #[test]
    fn drain_to_refusal_tokens_emits_substrate_authored_stcortex_per_entry() {
        use crate::refusal_token::{RefusalToken, SubstrateId};

        let w = writer(Some(0.001), false); // below floor: defers LtpBelowFloor
        for _ in 0..3 {
            let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        }
        let result = w.drain_to_refusal_tokens().expect("drain");
        assert_eq!(result.tokens.len(), 3, "3 deferred → 3 tokens");
        for token in &result.tokens {
            match token {
                RefusalToken::SubstrateAuthored {
                    substrate_id,
                    substrate_reason,
                    payload,
                } => {
                    assert_eq!(*substrate_id, SubstrateId::Stcortex);
                    assert!(
                        substrate_reason.starts_with("ltp_below_floor:density="),
                        "expected ltp_below_floor reason; got {substrate_reason}"
                    );
                    assert!(payload.is_none(), "no payload at this drain depth");
                }
                other => panic!("expected SubstrateAuthored Stcortex; got {other:?}"),
            }
            assert!(token.is_substrate_authored());
            assert!(!token.is_engine_imagined());
            assert_eq!(token.substrate_id(), Some(SubstrateId::Stcortex));
        }
    }

    #[test]
    fn drain_to_refusal_tokens_handles_orac_unreachable_reason() {
        use crate::refusal_token::RefusalToken;

        // density=None forces OracUnreachable defer (per m13 LTP gate
        // branch in defer_to_outbox).
        let w = writer(None, false);
        let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        let result = w.drain_to_refusal_tokens().expect("drain");
        assert_eq!(result.tokens.len(), 1);
        match &result.tokens[0] {
            RefusalToken::SubstrateAuthored {
                substrate_reason, ..
            } => {
                assert_eq!(substrate_reason, "orac_unreachable");
            }
            other => panic!("expected SubstrateAuthored; got {other:?}"),
        }
    }

    #[test]
    fn drain_to_refusal_tokens_replay_safe_when_cursor_not_committed() {
        // Same idempotent-replay contract as drain_outbox: re-call
        // without commit returns the same tokens.
        let w = writer(Some(0.001), false);
        let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        let first = w.drain_to_refusal_tokens().expect("first");
        let second = w.drain_to_refusal_tokens().expect("second");
        assert_eq!(first.tokens.len(), second.tokens.len());
        assert_eq!(first.new_cursor, second.new_cursor);
        for (a, b) in first.tokens.iter().zip(second.tokens.iter()) {
            assert_eq!(a, b, "replay tokens identical when cursor not committed");
        }
    }

    #[test]
    fn outbox_entry_to_refusal_token_renders_each_defer_reason_shape() {
        use super::{outbox_entry_to_refusal_token, OutboxEntry};
        use crate::refusal_token::{RefusalToken, SubstrateId};

        let make = |reason_json: serde_json::Value| -> OutboxEntry {
            let entry_json = serde_json::json!({
                "ts_ms": 1_700_000_000_000_u64,
                "memory": {
                    "namespace": "workflow_trace_test",
                    "memory_type": "semantic",
                    "content": "{}",
                    "relevance": 0.5_f32,
                    "session_id": "S1",
                    "source_tag": "m13",
                    "tensor": null,
                    "under_pressure": false,
                },
                "reason": reason_json,
            });
            serde_json::from_value(entry_json).expect("deser OutboxEntry")
        };
        // 1) LtpBelowFloor with density payload.
        let e1 = make(serde_json::json!({"LtpBelowFloor": {"density": 0.001}}));
        match outbox_entry_to_refusal_token(&e1) {
            RefusalToken::SubstrateAuthored {
                substrate_id,
                substrate_reason,
                ..
            } => {
                assert_eq!(substrate_id, SubstrateId::Stcortex);
                assert_eq!(substrate_reason, "ltp_below_floor:density=0.001");
            }
            other => panic!("got {other:?}"),
        }
        // 2) OracUnreachable bare-tag.
        let e2 = make(serde_json::json!("OracUnreachable"));
        match outbox_entry_to_refusal_token(&e2) {
            RefusalToken::SubstrateAuthored {
                substrate_reason, ..
            } => assert_eq!(substrate_reason, "orac_unreachable"),
            other => panic!("got {other:?}"),
        }
        // 3) StcortexUnreachable bare-tag.
        let e3 = make(serde_json::json!("StcortexUnreachable"));
        match outbox_entry_to_refusal_token(&e3) {
            RefusalToken::SubstrateAuthored {
                substrate_reason, ..
            } => assert_eq!(substrate_reason, "stcortex_unreachable"),
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn drain_reset_to_zero_when_outbox_shorter_than_persisted_cursor() {
        // Defensive: external truncation/rotation. Cursor points past EOF
        // → drain re-reads from offset 0 rather than skip silently.
        let w = writer(Some(0.001), false);
        let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        let drain1 = w.drain_outbox().expect("drain");
        w.commit_drain_cursor(drain1.new_cursor).expect("commit");
        // Externally truncate the outbox to a shorter length than the
        // committed cursor (simulates rotation).
        let outbox_path = w.outbox_path().clone();
        std::fs::write(&outbox_path, "").expect("truncate outbox");
        // Re-defer 1 entry (now the file is small, cursor is past EOF).
        let _ = w.promote_run(&run(), &canonical_ns()).expect("promote");
        let drain2 = w.drain_outbox().expect("drain after truncation");
        // Should re-read from 0 → 1 entry surfaced.
        assert_eq!(
            drain2.entries.len(),
            1,
            "post-truncation drain re-reads from 0 and surfaces 1 entry"
        );
    }
}
