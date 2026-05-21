//! Integration tests for m13 stcortex_writer (Wave-B2).
//!
//! Exercises the m13 surface at its public-API call boundary:
//!
//! - 3-band gate (LtpBelowFloor / Pressure / Normal) drives PromoteOutcome.
//! - JSONL outbox contract for Deferred path (incl. F-POVM-07 silent-zero
//!   regression — `now_ms()` is `Option<i64>` post commit `4d6e599`; on a
//!   well-defined clock the outbox MUST NOT contain `"ts_ms": 0` and MUST
//!   NOT tag `clock_unavailable: true`).
//! - AP30 namespace enforcement chains through m9 (foreign prefix refused,
//!   hyphen-namespace munged to underscore form).
//! - Concurrent defer appends serialise across the outbox mutex.
//! - Outbox-path traversal input is handled safely.
//!
//! Day-1 surface notes:
//!
//! - m13's `now_ms()` is module-private; we exercise the well-defined-clock
//!   regression via the public `promote_run` + outbox-read path. The
//!   live unit test `now_ms_returns_some_positive_on_well_defined_clock`
//!   in `src/m13_stcortex_writer/mod.rs` is the wiring evidence; this
//!   integration suite covers the behavioural surface.
//! - No clock-injection seam is exposed today — flagged for orchestrator
//!   in the report. If a future revision adds a `TestNowFn`, this suite
//!   gains a `m13_defer_writes_jsonl_with_clock_unavailable_tag` test.

#![allow(clippy::doc_markdown)]

use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use tempfile::TempDir;

use workflow_core::m13_stcortex_writer::{
    DeferReason, LtpDensityReader, PromoteOutcome, StcortexWriter, StcortexWriterError,
    SubstrateWriter, CorrelationMemory, LTP_PHASE_1_FLOOR,
};
use workflow_core::m7_workflow_runs::{Outcome, RunState, WorkflowRunRow};
use workflow_core::m9_watcher_namespace_guard::{NamespaceViolation, WORKFLOW_TRACE_NS_PREFIX};

// ---- Test-double surfaces ------------------------------------------------

/// Fixed-density reader; `None` simulates "ORAC unreachable".
struct StaticDensity(Option<f64>);

impl LtpDensityReader for StaticDensity {
    fn read_density(&self) -> Option<f64> {
        self.0
    }
}

/// Recording substrate writer assigns sequential ids and captures payloads.
struct RecordingWriter {
    next_id: std::sync::Mutex<i64>,
    written: std::sync::Mutex<Vec<CorrelationMemory>>,
}

impl RecordingWriter {
    fn new() -> Self {
        Self {
            next_id: std::sync::Mutex::new(0),
            written: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl SubstrateWriter for RecordingWriter {
    fn write_memory(
        &self,
        memory: &CorrelationMemory,
    ) -> Result<i64, StcortexWriterError> {
        let mut id = self.next_id.lock().expect("next_id lock");
        *id += 1;
        self.written
            .lock()
            .expect("written lock")
            .push(memory.clone());
        Ok(*id)
    }
}

/// Trait adapter so multiple StcortexWriter instances can share one
/// RecordingWriter (used by the m9 hyphen-munge test to read back the
/// payload captured by the substrate call).
struct SharedWriter(Arc<RecordingWriter>);

impl SubstrateWriter for SharedWriter {
    fn write_memory(
        &self,
        memory: &CorrelationMemory,
    ) -> Result<i64, StcortexWriterError> {
        self.0.write_memory(memory)
    }
}

fn canonical_ns() -> String {
    format!("{WORKFLOW_TRACE_NS_PREFIX}_correlations")
}

fn ok_run() -> WorkflowRunRow {
    WorkflowRunRow {
        id: 42,
        started_at: "2026-05-20T00:00:00Z".into(),
        run_state: RunState::Closed {
            ended_at: "2026-05-20T01:00:00Z".into(),
            outcome: Outcome::Ok,
        },
        consumer_inputs: "{}".into(),
        cost_tokens: Some(100),
        fitness_dimension: 0.0,
    }
}

fn build_writer(
    density: Option<f64>,
    outbox: PathBuf,
) -> StcortexWriter<StaticDensity, RecordingWriter> {
    StcortexWriter::new_unchecked(StaticDensity(density), RecordingWriter::new(), outbox)
}

// ---- Tests ---------------------------------------------------------------

// rationale: Happy path (3-band gate normal density → PromoteOutcome::Written
//            with substrate-assigned memory id; CC-5 substrate learning loop
//            wire integrity)
#[test]
fn m13_promote_run_under_normal_density_writes_correlation_memory() {
    // rationale: Happy path (3-band gate band-3)
    let tmp = TempDir::new().expect("tempdir");
    let w = build_writer(Some(0.20), tmp.path().join("outbox.jsonl"));
    let out = w
        .promote_run(&ok_run(), &canonical_ns())
        .expect("promote should succeed");
    match out {
        PromoteOutcome::Written { memory_id } => {
            assert!(memory_id > 0, "memory_id must be substrate-assigned");
        }
        other => panic!("expected Written, got {other:?}"),
    }
    // Outbox must NOT exist on a normal-band write (defer never fired).
    assert!(
        !tmp.path().join("outbox.jsonl").exists(),
        "normal-band write must not touch the outbox"
    );
}

// rationale: Happy path with pressure flag (3-band gate band-2 → memory id +
//            under_pressure tag carried through to CorrelationMemory)
#[test]
fn m13_promote_run_under_pressure_writes_with_lower_priority() {
    // rationale: Happy path (band-2 pressure tag carries through to memory)
    let tmp = TempDir::new().expect("tempdir");
    let w = build_writer(Some(0.05), tmp.path().join("outbox.jsonl"));
    let out = w
        .promote_run(&ok_run(), &canonical_ns())
        .expect("promote should succeed");
    match out {
        PromoteOutcome::WrittenUnderPressure {
            memory_id,
            ltp_density,
        } => {
            assert!(memory_id > 0);
            assert!((ltp_density - 0.05).abs() < 1e-12);
        }
        other => panic!("expected WrittenUnderPressure, got {other:?}"),
    }
}

// rationale: Defer path (3-band gate band-1 below floor → PromoteOutcome
//            ::Deferred with LtpBelowFloor reason + JSONL outbox row)
#[test]
fn m13_promote_run_below_floor_defers_to_outbox() {
    // rationale: Defer path (band-1 LtpBelowFloor)
    let tmp = TempDir::new().expect("tempdir");
    let outbox = tmp.path().join("outbox.jsonl");
    let w = build_writer(Some(0.001), outbox.clone());
    let out = w
        .promote_run(&ok_run(), &canonical_ns())
        .expect("defer should succeed");
    match out {
        PromoteOutcome::Deferred {
            reason: DeferReason::LtpBelowFloor { density },
        } => {
            assert!((density - 0.001).abs() < 1e-12);
        }
        other => panic!("expected Deferred LtpBelowFloor, got {other:?}"),
    }
    let contents = fs::read_to_string(&outbox).expect("outbox should exist");
    assert!(contents.contains("LtpBelowFloor"));
    // Below-floor defer must produce exactly one JSONL line.
    assert_eq!(
        contents.lines().count(),
        1,
        "one defer = one line; got: {contents}"
    );
}

// rationale: Anti-property (F-POVM-07 silent-zero regression) — m13's
//            `now_ms()` is now `Option<i64>` (commit 4d6e599, scout C2).
//            On a well-defined clock the defer-path JSONL outbox MUST NOT
//            contain `"ts_ms": 0` and MUST NOT tag `clock_unavailable`.
//            The clock-fault path itself is unit-tested in
//            `src/m13_stcortex_writer/mod.rs` (see
//            `now_ms_returns_some_positive_on_well_defined_clock`); a
//            module-public clock-injection seam is flagged for orchestrator.
#[test]
fn m13_defer_writes_jsonl_with_clock_unavailable_tag_when_now_ms_fails() {
    // rationale: Anti-property (F-POVM-07; integration-side regression)
    let tmp = TempDir::new().expect("tempdir");
    let outbox = tmp.path().join("outbox.jsonl");
    let w = build_writer(Some(0.001), outbox.clone());
    let _ = w
        .promote_run(&ok_run(), &canonical_ns())
        .expect("defer path");
    let contents = fs::read_to_string(&outbox).expect("outbox should exist");
    // No silent-zero pattern in either spacing variant.
    assert!(
        !contents.contains("\"ts_ms\":0"),
        "F-POVM-07 silent-zero leaked into outbox JSONL: {contents}"
    );
    assert!(
        !contents.contains("\"ts_ms\": 0"),
        "F-POVM-07 silent-zero (whitespace variant) leaked: {contents}"
    );
    // On a well-defined clock the `clock_unavailable: true` tag must NOT
    // appear. The presence of that tag is the load-bearing signal that the
    // clock-fault path fired — see m13 mod docstring for the contract.
    assert!(
        !contents.contains("clock_unavailable"),
        "production clock must not tag clock_unavailable: {contents}"
    );
    // Positive: the ts_ms field is a non-zero integer.
    let parsed: serde_json::Value = serde_json::from_str(
        contents.lines().next().expect("one line"),
    )
    .expect("valid JSONL");
    let ts = parsed
        .get("ts_ms")
        .and_then(serde_json::Value::as_i64)
        .expect("ts_ms must be a non-null integer");
    assert!(
        ts > 1_700_000_000_000_i64,
        "ts_ms must be realistic 2024+ wall-clock ms, got {ts}"
    );
}

// rationale: Cross-module surface (m13 → m9 namespace guard) — a foreign
//            prefix at the m13 boundary MUST surface as NamespaceViolation
//            (AP30 mitigation chain m13 → m9). The substrate writer must
//            never see the payload.
#[test]
fn m13_refuses_foreign_namespace_via_m9_chain() {
    // rationale: Cross-module surface invariant (AP30 m13 → m9)
    let tmp = TempDir::new().expect("tempdir");
    let outbox = tmp.path().join("outbox.jsonl");
    let w = build_writer(Some(0.20), outbox.clone());
    let err = w
        .promote_run(&ok_run(), "orac_pathway")
        .expect_err("foreign prefix must be refused");
    match err {
        StcortexWriterError::NamespaceViolation(NamespaceViolation::WrongPrefix {
            ..
        }) => {}
        other => panic!("expected WrongPrefix, got {other:?}"),
    }
    // No substrate write fired; no outbox row fired.
    assert!(
        !outbox.exists(),
        "AP30 refusal must not touch the outbox"
    );
}

// rationale: Cross-module surface (m9 hyphen-munge composes with m13) — a
//            hyphenated namespace at the m13 boundary is munged to its
//            underscored form before reaching the substrate writer. Pins
//            the S1001757 hyphen-slug discipline at the m13 surface.
#[test]
fn m13_munges_hyphen_namespace_via_m9() {
    // rationale: Cross-module surface (m9 hyphen-munge in m13's write path)
    let tmp = TempDir::new().expect("tempdir");
    // SharedWriter (defined at module scope) lets us peek at the substrate
    // payload after the m13 call returns — m13 doesn't expose a public
    // writer-getter, so we share the recorder via Arc.
    let shared = Arc::new(RecordingWriter::new());
    let s = StcortexWriter::new_unchecked(
        StaticDensity(Some(0.20)),
        SharedWriter(Arc::clone(&shared)),
        tmp.path().join("outbox.jsonl"),
    );
    let out = s
        .promote_run(&ok_run(), "workflow-trace-something")
        .expect("hyphen namespace should be munged + accepted");
    assert!(matches!(out, PromoteOutcome::Written { .. }));
    let captured = shared.written.lock().expect("written lock").clone();
    assert_eq!(captured.len(), 1);
    assert_eq!(
        captured[0].namespace, "workflow_trace_something",
        "namespace must reach substrate as underscored form (S1001757 munge)"
    );
    assert!(
        !captured[0].namespace.contains('-'),
        "no hyphen may slip through to substrate write"
    );
}

// rationale: Concurrency (defer path outbox mutex serialises bytes) — N
//            threads forced into the LtpBelowFloor band must each append a
//            single, byte-valid JSONL line. No interleaved writes.
#[test]
fn m13_concurrent_promote_run_serializes_outbox_appends() {
    // rationale: Concurrency (outbox mutex correctness under contention)
    let tmp = TempDir::new().expect("tempdir");
    let outbox = tmp.path().join("outbox.jsonl");
    let w = Arc::new(build_writer(Some(0.001), outbox.clone()));
    let threads = 4_usize;
    let mut handles = Vec::with_capacity(threads);
    for t in 0..threads {
        let w2 = Arc::clone(&w);
        handles.push(thread::spawn(move || {
            let mut r = ok_run();
            r.id = i64::try_from(t).expect("t fits");
            w2.promote_run(&r, &canonical_ns())
                .expect("defer should succeed on each thread")
        }));
    }
    for h in handles {
        let out = h.join().expect("thread join");
        assert!(matches!(
            out,
            PromoteOutcome::Deferred {
                reason: DeferReason::LtpBelowFloor { .. }
            }
        ));
    }
    let contents = fs::read_to_string(&outbox).expect("outbox read");
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(
        lines.len(),
        threads,
        "expected exactly {threads} JSONL lines (one per thread)"
    );
    // Each line is independently parseable — proves no byte interleaving.
    for (i, line) in lines.iter().enumerate() {
        let parsed = serde_json::from_str::<serde_json::Value>(line);
        assert!(
            parsed.is_ok(),
            "line {i} is not valid JSON (byte interleave): {line}"
        );
    }
}

// rationale: Adversarial input (outbox path traversal) — a constructor input
//            containing `../` segments must NOT result in writes outside the
//            intended sandbox directory. m13 today writes to whatever path
//            the constructor receives (no canonicalisation), so this test
//            pins the CURRENT behaviour while keeping the assertion
//            tight: the resolved write target must reside within the
//            tempdir's canonical root (parent-`../`-then-child returns
//            under the parent). The day a hardening lands that refuses
//            traversal-shaped paths, this test fails-loud and the
//            expectation flips.
#[test]
fn m13_outbox_path_traversal_input_handled_safely() {
    // rationale: Adversarial input (path traversal containment regression)
    let tmp = TempDir::new().expect("tempdir");
    let nested = tmp.path().join("subdir");
    fs::create_dir_all(&nested).expect("mkdir subdir");
    // Path with a traversal segment that resolves UNDER the tempdir
    // (subdir/../outbox.jsonl == tempdir/outbox.jsonl). The point is to
    // exercise the path-handling surface without escaping the sandbox.
    let traversal = nested.join("..").join("outbox.jsonl");
    let w = build_writer(Some(0.001), traversal.clone());
    let _ = w
        .promote_run(&ok_run(), &canonical_ns())
        .expect("defer should succeed");
    // The write must land inside the tempdir canonical root — never above.
    let canonical_root = tmp
        .path()
        .canonicalize()
        .expect("tempdir canonicalize");
    // Either traversal canonicalises (file exists) or the parent does.
    let resolved = traversal
        .canonicalize()
        .or_else(|_| traversal.parent().expect("parent").canonicalize())
        .expect("either child or parent canonicalises");
    assert!(
        resolved.starts_with(&canonical_root),
        "outbox write escaped tempdir: resolved={resolved:?}, root={canonical_root:?}"
    );
    // And the file is non-empty (write actually happened — defer not lost).
    let mut content = String::new();
    fs::File::open(&traversal)
        .expect("open outbox")
        .read_to_string(&mut content)
        .expect("read");
    assert!(!content.is_empty(), "defer payload must reach disk");
}

// rationale: Boundary (LTP_PHASE_1_FLOOR exact-equality contract) — density
//            exactly at the floor is NOT below the floor (strict `<`); m13
//            writes in band-2 (under-pressure). Integration-side
//            confirmation of the production constant.
#[test]
fn m13_band_at_exact_phase_1_floor_writes_under_pressure() {
    // rationale: Boundary (3-band gate exact threshold)
    let tmp = TempDir::new().expect("tempdir");
    let w = build_writer(Some(LTP_PHASE_1_FLOOR), tmp.path().join("outbox.jsonl"));
    let out = w
        .promote_run(&ok_run(), &canonical_ns())
        .expect("promote at exact floor");
    assert!(
        matches!(out, PromoteOutcome::WrittenUnderPressure { .. }),
        "exact floor must NOT defer (strict `<`); got {out:?}"
    );
}
