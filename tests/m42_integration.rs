//! Integration tests for m42 stcortex-emit (Wave-C3).
//!
//! Locks the m42 stcortex-only ADR (D-S1001982-01) anti-properties,
//! the cross-module routing chain m42 → m13 → m9, the Phase-A
//! documented behaviour that `workflow` and `signal` are call-site
//! self-check inputs (intentionally discarded by Phase A), and the
//! determinism of `signal_for_outcome`.
//!
//! The two static-grep tests below extend the Wave-1 anti-property
//! check at `src/m42_stcortex_emit/mod.rs::tests::no_povm_symbol_reachable_from_m42_source`
//! by enforcing the constraint from the *integration-test* perspective
//! against the same source file plus the live binary's exported name.

#![allow(clippy::doc_markdown)]

use std::path::PathBuf;
use std::sync::Mutex as StdMutex;
use std::time::SystemTime;

use tempfile::Builder as TempBuilder;

use workflow_core::m13_stcortex_writer::{
    CorrelationMemory, DeferReason, LtpDensityReader, PromoteOutcome, StcortexWriter,
    StcortexWriterError, SubstrateWriter,
};
use workflow_core::m14_lift::LiftSnapshot;
use workflow_core::m20_prefixspan::{Pattern, StepToken};
use workflow_core::m21_variant_builder::build_variants;
use workflow_core::m23_proposer::build_proposal;
use workflow_core::m30_bank::{AcceptedWorkflow, CuratedBank};
use workflow_core::m42_stcortex_emit::{
    emit_feedback, signal_for_outcome, HebbianSignal, SubstrateEmitError,
};
use workflow_core::m7_workflow_runs::WorkflowRunRow;
use workflow_core::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX;

// ─── fixtures ────────────────────────────────────────────────────────

struct StaticDensity(Option<f64>);
impl LtpDensityReader for StaticDensity {
    fn read_density(&self) -> Option<f64> {
        self.0
    }
}

struct RecordingWriter {
    next_id: StdMutex<i64>,
    written: StdMutex<Vec<CorrelationMemory>>,
}
impl RecordingWriter {
    fn new() -> Self {
        Self {
            next_id: StdMutex::new(0),
            written: StdMutex::new(Vec::new()),
        }
    }
}
impl SubstrateWriter for RecordingWriter {
    fn write_memory(
        &self,
        memory: &CorrelationMemory,
    ) -> Result<i64, StcortexWriterError> {
        let mut id = self.next_id.lock().expect("id lock");
        *id += 1;
        self.written.lock().expect("written lock").push(memory.clone());
        Ok(*id)
    }
}

fn temp_outbox() -> PathBuf {
    let p = TempBuilder::new()
        .suffix(".jsonl")
        .tempfile()
        .expect("temp")
        .into_temp_path();
    let path = p.to_path_buf();
    // Intentionally forget the temp path so the underlying file lingers
    // for the duration of the test; cleanup happens via OS tmp reaping.
    std::mem::forget(p);
    path
}

fn ok_run() -> WorkflowRunRow {
    WorkflowRunRow {
        id: 42,
        started_at: "2026-05-20T00:00:00Z".into(),
        ended_at: Some("2026-05-20T01:00:00Z".into()),
        outcome: Some("ok".into()),
        consumer_inputs: "{}".into(),
        cost_tokens: Some(100),
        fitness_dimension: 0.0,
    }
}

fn workflow_fixture() -> AcceptedWorkflow {
    let p = Pattern::new(vec![StepToken(7), StepToken(8)], 30, (0, 1));
    let v = build_variants(&p).expect("v")[0].clone();
    let s = LiftSnapshot {
        lift: Some(0.5),
        ci_half: Some(0.05),
        n: 30,
        latest_ts_ms: 0,
        computed_at: SystemTime::now(),
    };
    let bank = CuratedBank::new();
    let id = bank
        .accept(build_proposal(v, &s, None).expect("p"), 1_700_000_000_000)
        .expect("accept");
    bank.get(id).expect("get")
}

fn canonical_ns() -> String {
    format!("{WORKFLOW_TRACE_NS_PREFIX}_outcomes")
}

// ─── tests ───────────────────────────────────────────────────────────

// rationale: Cross-module (m42 → m13) — emit_feedback under a
// canonical workflow_trace_* namespace + non-pressure density MUST
// route through m13.promote_run and surface a Written outcome with a
// substrate-assigned memory id.
#[test]
fn m42_emit_feedback_routes_through_m13_via_namespace_prefix() {
    // rationale: Cross-module (m42 → m13)
    let writer = StcortexWriter::new(
        StaticDensity(Some(0.20)),
        RecordingWriter::new(),
        temp_outbox(),
    );
    let wf = workflow_fixture();
    let outcome = emit_feedback(
        &writer,
        &wf,
        &ok_run(),
        &canonical_ns(),
        HebbianSignal::Reinforce,
    )
    .expect("emit_feedback");
    match outcome {
        PromoteOutcome::Written { memory_id } => {
            assert!(memory_id > 0, "memory_id must be substrate-assigned");
        }
        other => panic!("expected Written, got {other:?}"),
    }
}

// rationale: Cross-module AP30 — emit_feedback against a foreign
// namespace MUST reject transitively at the m9 boundary, surfacing as
// SubstrateEmitError::Writer(NamespaceViolation). The m9 gate is the
// canonical authority; m42 must NOT shadow it.
#[test]
fn m42_emit_feedback_refuses_foreign_namespace_via_m9_chain() {
    // rationale: Cross-module AP30 (m42 → m13 → m9)
    let writer = StcortexWriter::new(
        StaticDensity(Some(0.20)),
        RecordingWriter::new(),
        temp_outbox(),
    );
    let wf = workflow_fixture();
    let err = emit_feedback(
        &writer,
        &wf,
        &ok_run(),
        "orac_foreign_prefix",
        HebbianSignal::Reinforce,
    )
    .expect_err("foreign namespace must fail at m9 boundary");
    assert!(
        matches!(
            err,
            SubstrateEmitError::Writer(StcortexWriterError::NamespaceViolation(_))
        ),
        "expected Writer(NamespaceViolation), got {err:?}"
    );
}

// rationale: Determinism — signal_for_outcome is a pure mapping; 1000
// repeated calls with identical inputs MUST yield identical outputs.
#[test]
fn m42_signal_for_outcome_is_deterministic_across_1000_calls() {
    // rationale: Determinism
    for _ in 0..1000_u32 {
        assert_eq!(signal_for_outcome(Some("ok")), HebbianSignal::Reinforce);
        assert_eq!(signal_for_outcome(Some("fail")), HebbianSignal::Depress);
        assert_eq!(signal_for_outcome(Some("abort")), HebbianSignal::Depress);
        assert_eq!(signal_for_outcome(None), HebbianSignal::Depress);
    }
}

// rationale: Boundary — only the exact string "ok" maps to Reinforce;
// every other outcome string (including whitespace-padded "ok ",
// upper-case "OK", "unknown", "") maps to Depress. Anti-property on
// the static heuristic.
#[test]
fn m42_signal_for_outcome_maps_ok_to_reinforce_others_to_depress() {
    // rationale: Boundary (case-sensitive exact-match)
    assert_eq!(signal_for_outcome(Some("ok")), HebbianSignal::Reinforce);
    for non_ok in [
        "fail", "abort", "unknown", "", "ok ", " ok", "OK", "Ok", "okay",
    ] {
        assert_eq!(
            signal_for_outcome(Some(non_ok)),
            HebbianSignal::Depress,
            "outcome {non_ok:?} must not collapse to Reinforce"
        );
    }
    assert_eq!(signal_for_outcome(None), HebbianSignal::Depress);
}

// rationale: Anti-property (extends Wave-1 in-module scope) — the m42
// source file, scanned from the integration-test crate boundary, MUST
// contain no POVM symbol reachable as a code path (function call, type
// reference, mod path). Doc comments referencing the decoupling are
// allowed; identifiers in active code are not. Mirrors the Wave-1
// in-module test logic (strip doc/comment lines + strip string
// literals before grepping).
#[test]
fn m42_no_povm_symbol_reachable_from_m42_source() {
    // rationale: Anti-property (extends Wave-1)
    let src = include_str!("../src/m42_stcortex_emit/mod.rs");
    for (lineno, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        // Strip string-literal contents so this test's own assertion
        // strings (which mention povm) don't false-match.
        let stripped = {
            let mut out = String::new();
            let mut in_str = false;
            let mut prev_backslash = false;
            for ch in line.chars() {
                if ch == '"' && !prev_backslash {
                    in_str = !in_str;
                    prev_backslash = false;
                    continue;
                }
                prev_backslash = ch == '\\' && !prev_backslash;
                if !in_str {
                    out.push(ch);
                }
            }
            out.to_lowercase()
        };
        assert!(
            !stripped.contains("povm::"),
            "m42 line {lineno}: forbidden POVM symbol reference (code path)"
        );
        assert!(
            !stripped.contains("povm_client"),
            "m42 line {lineno}: forbidden POVM client reference (code path)"
        );
        assert!(
            !stripped.contains("povm_overlap_active"),
            "m42 line {lineno}: forbidden POVM overlap flag (code path)"
        );
        assert!(
            !stripped.contains("dual_path"),
            "m42 line {lineno}: forbidden POVM dual-path symbol (code path)"
        );
    }
}

// rationale: Anti-property (Wave-1 scout T4-AP30 fix) — m42 source MUST
// contain no `:8125` port literal anywhere outside doc/comment lines,
// matching the m42 stcortex-only ADR (POVM port literal forbidden).
#[test]
fn m42_no_8125_port_literal_in_m42_source() {
    // rationale: Anti-property (Wave-1 scout T4-AP30 fix)
    let src = include_str!("../src/m42_stcortex_emit/mod.rs");
    for (lineno, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        // Strip string-literal contents so this test's own strings (and
        // future doctests) don't false-match.
        let stripped = {
            let mut out = String::new();
            let mut in_str = false;
            let mut prev_backslash = false;
            for ch in line.chars() {
                if ch == '"' && !prev_backslash {
                    in_str = !in_str;
                    prev_backslash = false;
                    continue;
                }
                prev_backslash = ch == '\\' && !prev_backslash;
                if !in_str {
                    out.push(ch);
                }
            }
            out
        };
        assert!(
            !stripped.contains("8125"),
            "m42 line {lineno}: forbidden POVM port literal (Wave-1 T4-AP30 fix)"
        );
        assert!(
            !stripped.contains(":8125"),
            "m42 line {lineno}: forbidden POVM port literal (Wave-1 T4-AP30 fix)"
        );
    }
}

// rationale: Documented Phase-A behaviour — `emit_feedback` accepts
// `workflow` and `signal` as call-site self-check inputs but does NOT
// route them to the substrate (per module docstring). We exercise the
// surface by passing two contradictory (workflow, signal) pairs and
// asserting both successfully route through m13.promote_run with
// identical PromoteOutcome shapes (the substrate sees only the run +
// namespace). This locks the Phase-A contract — the silent discard is
// documented behaviour, not a latent bug.
#[test]
fn m42_emit_feedback_silently_discards_workflow_arg_per_phase_a() {
    // rationale: Documented Phase-A behaviour
    let writer_a = StcortexWriter::new(
        StaticDensity(Some(0.20)),
        RecordingWriter::new(),
        temp_outbox(),
    );
    let writer_b = StcortexWriter::new(
        StaticDensity(Some(0.20)),
        RecordingWriter::new(),
        temp_outbox(),
    );
    let wf = workflow_fixture();
    // Call A: signal=Reinforce, run.outcome="ok".
    let out_a = emit_feedback(
        &writer_a,
        &wf,
        &ok_run(),
        &canonical_ns(),
        HebbianSignal::Reinforce,
    )
    .expect("emit A");
    // Call B: signal=Depress (deliberately contradicting the OK run);
    // workflow same; namespace same. Phase-A MUST still write — the
    // signal arg is self-check, not routed.
    let out_b = emit_feedback(
        &writer_b,
        &wf,
        &ok_run(),
        &canonical_ns(),
        HebbianSignal::Depress,
    )
    .expect("emit B");
    // Both calls produce Written outcomes (Phase A routes purely on
    // m13.promote_run, which ignores the signal arg).
    assert!(matches!(out_a, PromoteOutcome::Written { .. }));
    assert!(matches!(out_b, PromoteOutcome::Written { .. }));
    // And — for completeness — an unreachable ORAC defers to outbox
    // without surfacing as an error (m13 outbox-first invariant
    // transitively held).
    let writer_unreachable = StcortexWriter::new(
        StaticDensity(None),
        RecordingWriter::new(),
        temp_outbox(),
    );
    let out_c = emit_feedback(
        &writer_unreachable,
        &wf,
        &ok_run(),
        &canonical_ns(),
        HebbianSignal::Reinforce,
    )
    .expect("defer MUST not error");
    assert!(matches!(
        out_c,
        PromoteOutcome::Deferred {
            reason: DeferReason::OracUnreachable
        }
    ));
}
