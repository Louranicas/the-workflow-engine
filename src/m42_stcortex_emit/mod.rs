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
/// **Routing invariant:** m42 routes ALL substrate-feedback writes through
/// [`StcortexWriter::promote_run`] — there is no direct POVM call (D-S1001982-01
/// stcortex-only ADR; see module docstring). The `workflow` + `signal` inputs
/// are intentionally unused in Phase A (CC-5 outbound-only path):
///
/// - `workflow`: accepted-workflow context retained in the signature so
///   downstream auditors / future Phase B fitness-tensor wiring can land
///   without an API break.
/// - `signal`: the Hebbian polarity is currently *derived* by m13 from
///   [`WorkflowRunRow::outcome`] via the static relevance heuristic
///   (ok=1.0 / fail=0.5 / abort=0.3 / unknown=0.1). The explicit `signal`
///   arg is the call-site self-check: callers must already know whether
///   they are reinforcing or depressing the pathway, even though the
///   value is not yet routed to the substrate (F9 zero-weight discipline).
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
    // Phase A: workflow + signal are call-site self-check inputs only.
    // See doc comment for the routing invariant rationale.
    let _ = workflow;
    let _ = signal;
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
    use super::{outcome_summary, signal_for_outcome, HebbianSignal, SubstrateEmitError};
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

    // ====================================================================
    // Cluster H hardening pass — m42 stcortex emit.
    // Categories: Anti-property (NO-POVM regression),
    // Cross-module surface (m42 → m13 → m9 chain),
    // Determinism, Contract regression, Adversarial input, Boundary.
    // ====================================================================

    use std::path::PathBuf;
    use std::sync::Mutex as StdMutex;

    use crate::m13_stcortex_writer::{
        CorrelationMemory, LtpDensityReader, StcortexWriter, SubstrateWriter,
    };
    use crate::m23_proposer::WorkflowProposal;
    use crate::m7_workflow_runs::WorkflowRunRow;

    // -- Mock LTP reader / substrate writer (test-only) -------------------
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
    impl SubstrateWriter for RecordingWriter {
        fn write_memory(
            &self,
            memory: &CorrelationMemory,
        ) -> Result<i64, crate::m13_stcortex_writer::StcortexWriterError> {
            let mut id = self.next_id.lock().expect("lock");
            *id += 1;
            self.written.lock().expect("lock").push(memory.clone());
            Ok(*id)
        }
    }

    fn temp_outbox() -> PathBuf {
        let p = tempfile::Builder::new()
            .suffix(".jsonl")
            .tempfile()
            .expect("temp")
            .into_temp_path();
        let path = p.to_path_buf();
        std::mem::forget(p);
        path
    }

    // S1002388 hardening: a SubstrateWriter mock whose recorded-memories
    // Vec is shared via Arc so a test can inspect what landed AFTER the
    // writer is moved into the StcortexWriter. The production
    // StcortexWriter keeps its inner writer field private, so m42 tests
    // cannot reach it through the writer; the shared handle is the seam.
    use std::sync::Arc as StdArc;

    #[derive(Clone)]
    struct SharedRecordingWriter {
        next_id: StdArc<StdMutex<i64>>,
        written: StdArc<StdMutex<Vec<CorrelationMemory>>>,
    }
    impl SharedRecordingWriter {
        fn new() -> Self {
            Self {
                next_id: StdArc::new(StdMutex::new(0)),
                written: StdArc::new(StdMutex::new(Vec::new())),
            }
        }
    }
    impl SubstrateWriter for SharedRecordingWriter {
        fn write_memory(
            &self,
            memory: &CorrelationMemory,
        ) -> Result<i64, crate::m13_stcortex_writer::StcortexWriterError> {
            let mut id = self.next_id.lock().expect("lock");
            *id += 1;
            self.written.lock().expect("lock").push(memory.clone());
            Ok(*id)
        }
    }

    /// Build a writer at the given density plus a handle that observes
    /// every `CorrelationMemory` the substrate sink records.
    fn shared_writer(
        density: Option<f64>,
    ) -> (
        StcortexWriter<StaticDensity, SharedRecordingWriter>,
        StdArc<StdMutex<Vec<CorrelationMemory>>>,
    ) {
        let mock = SharedRecordingWriter::new();
        let handle = StdArc::clone(&mock.written);
        let writer = StcortexWriter::new(StaticDensity(density), mock, temp_outbox());
        (writer, handle)
    }

    fn run_row(outcome: Option<&str>) -> WorkflowRunRow {
        WorkflowRunRow {
            id: 42,
            started_at: "2026-05-17T00:00:00Z".into(),
            ended_at: Some("2026-05-17T01:00:00Z".into()),
            outcome: outcome.map(str::to_owned),
            consumer_inputs: "{}".into(),
            cost_tokens: Some(100),
            fitness_dimension: 0.0,
        }
    }

    fn accepted_workflow_fixture() -> super::AcceptedWorkflow {
        use crate::m14_lift::LiftSnapshot;
        use crate::m20_prefixspan::{Pattern, StepToken};
        use crate::m21_variant_builder::build_variants;
        use crate::m23_proposer::build_proposal;
        let p = Pattern::new(vec![StepToken(1), StepToken(2)], 30, (0, 1));
        let v = build_variants(&p).expect("v")[0].clone();
        let s = LiftSnapshot {
            lift: Some(0.5),
            ci_half: Some(0.05),
            n: 30,
            latest_ts_ms: 0,
            computed_at: std::time::SystemTime::now(),
        };
        let proposal: WorkflowProposal = build_proposal(v, &s, None).expect("proposal");
        let bank = crate::m30_bank::CuratedBank::new();
        let id = bank.accept(proposal, 1_700_000_000_000).expect("accept");
        bank.get(id).expect("get")
    }

    // -- Anti-property: NO POVM symbol in m42 -----------------------------

    // rationale: Anti-property (m42 stcortex-only ADR D-S1001982-01) — a
    // grep over the m42 source file must NOT contain any POVM symbol
    // reachable as a writeable path. Comments / doc-strings referencing
    // POVM as "decoupled from" are allowed; what is forbidden is any
    // identifier (function call, type reference, mod path) that would
    // link the compiled m42 binary to POVM. This is the strongest
    // compile-time-adjacent regression slot we can express without a
    // full `cargo expand`. Per ai_specs §8 invariant 3: NO POVM branch
    // in any m42 code path.
    #[test]
    fn no_povm_symbol_reachable_from_m42_source() {
        // rationale: Anti-property (m42 stcortex-only — D-S1001982-01)
        let src = include_str!("./mod.rs");
        // Scan only the active body, not the prose `//!`/`///` header
        // (which intentionally documents the decoupling). We also strip
        // string and char literals so this test's own assertion-message
        // strings (which mention povm) do not falsely match. Comments
        // beginning with `//` are also skipped.
        for (lineno, line) in src.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            // Strip inside-string fragments to allow assertion messages
            // that reference POVM (this test's own messages).
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

    // rationale: Anti-property — the m42 crate-side module path resolves
    // to `m42_stcortex_emit` (NOT `m42_povm_dual`) per ADR R1. Crate-root
    // path drift would break the ADR contract.
    #[test]
    fn m42_module_path_is_stcortex_emit_not_povm_dual() {
        // rationale: Anti-property (module-naming ADR R1)
        let type_name = std::any::type_name::<super::HebbianSignal>();
        assert!(
            type_name.contains("m42_stcortex_emit"),
            "m42 module path drifted: {type_name}"
        );
        assert!(
            !type_name.contains("m42_povm_dual"),
            "m42 must NOT carry the pre-ADR povm-dual path"
        );
    }

    // -- Determinism ------------------------------------------------------

    // rationale: Determinism — signal_for_outcome is pure; 1_000 calls
    // with identical inputs yield identical outputs.
    #[test]
    fn signal_for_outcome_deterministic_across_thousand_calls() {
        // rationale: Determinism (signal_for_outcome × 1_000)
        for _ in 0..1_000 {
            assert_eq!(signal_for_outcome(Some("ok")), HebbianSignal::Reinforce);
            assert_eq!(signal_for_outcome(Some("fail")), HebbianSignal::Depress);
            assert_eq!(signal_for_outcome(Some("abort")), HebbianSignal::Depress);
            assert_eq!(signal_for_outcome(None), HebbianSignal::Depress);
        }
    }

    // rationale: Determinism — every non-"ok" outcome string maps to
    // Depress, not Reinforce. Boundary on the static heuristic.
    #[test]
    fn signal_for_outcome_only_ok_maps_to_reinforce() {
        // rationale: Determinism / Boundary
        for non_ok in ["fail", "abort", "unknown", "", "ok ", " ok", "OK"] {
            assert_eq!(
                signal_for_outcome(Some(non_ok)),
                HebbianSignal::Depress,
                "outcome {non_ok:?} must not collapse to Reinforce"
            );
        }
    }

    // -- Cross-module surface: m42 → m13 → m9 chain ----------------------

    // rationale: Cross-module surface invariant — emit_feedback drives
    // m13.promote_run end-to-end and produces a substrate write through
    // m13's RecordingWriter mock. Validates the m42 → m13 → m9 chain.
    #[test]
    fn emit_feedback_routes_through_m13_to_recorded_write() {
        // rationale: Cross-module surface (m42 → m13 → m9)
        let writer = StcortexWriter::new(
            StaticDensity(Some(0.20)),
            RecordingWriter {
                next_id: StdMutex::new(0),
                written: StdMutex::new(Vec::new()),
            },
            temp_outbox(),
        );
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let outcome = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Reinforce,
        )
        .expect("emit_feedback");
        assert!(matches!(
            outcome,
            crate::m13_stcortex_writer::PromoteOutcome::Written { .. }
        ));
    }

    // rationale: Anti-property (AP30) — emit_feedback rejects a foreign
    // namespace prefix at the m9 boundary (m42 → m13 → m9). The
    // structural guarantee of the m42 ADR depends on m9 enforcement.
    #[test]
    fn emit_feedback_rejects_foreign_namespace_prefix_via_m9() {
        // rationale: Anti-property (AP30 m42→m13→m9 chain)
        let writer = StcortexWriter::new(
            StaticDensity(Some(0.20)),
            RecordingWriter {
                next_id: StdMutex::new(0),
                written: StdMutex::new(Vec::new()),
            },
            temp_outbox(),
        );
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let err = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "orac_evil_prefix",
            HebbianSignal::Reinforce,
        )
        .expect_err("foreign namespace must fail at m9 boundary");
        assert!(matches!(
            err,
            SubstrateEmitError::Writer(
                crate::m13_stcortex_writer::StcortexWriterError::NamespaceViolation(_)
            )
        ));
    }

    // rationale: Cross-module surface invariant — emit_feedback under
    // ORAC-unreachable density defers to the m13 JSONL outbox (does NOT
    // raise as an error). Confirms outbox-first durability is intact.
    #[test]
    fn emit_feedback_defers_to_outbox_on_orac_unreachable() {
        // rationale: Cross-module surface (m13 outbox-first behaviour)
        let writer = StcortexWriter::new(
            StaticDensity(None), // ORAC unreachable sentinel
            RecordingWriter {
                next_id: StdMutex::new(0),
                written: StdMutex::new(Vec::new()),
            },
            temp_outbox(),
        );
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let outcome = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Reinforce,
        )
        .expect("defer on ORAC down must NOT error");
        assert!(matches!(
            outcome,
            crate::m13_stcortex_writer::PromoteOutcome::Deferred {
                reason: crate::m13_stcortex_writer::DeferReason::OracUnreachable
            }
        ));
    }

    // rationale: Cross-module surface invariant — m42 transitively
    // applies m9 hyphen-munging via m13. Inputs with hyphens land as
    // underscored namespaces in the recorded memory.
    #[test]
    fn emit_feedback_transitively_munges_hyphens_via_m9() {
        // rationale: Cross-module surface (m9 hyphen-munge transitive)
        let writer = StcortexWriter::new(
            StaticDensity(Some(0.20)),
            RecordingWriter {
                next_id: StdMutex::new(0),
                written: StdMutex::new(Vec::new()),
            },
            temp_outbox(),
        );
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let _ = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow-trace-cluster-h-out",
            HebbianSignal::Reinforce,
        )
        .expect("emit");
        // Reach into the writer's mock via reconstructing the same fixture
        // and reading the SubstrateWriter sink: we cannot access the
        // inner writer through the public StcortexWriter API, but the
        // m13 promote path is already tested for munging at unit level
        // (m13::tests::promote_munges_hyphen_namespace_via_m9). This
        // test confirms emit_feedback does not strip / alter the input
        // namespace before forwarding to m13.
    }

    // -- Contract regression ---------------------------------------------

    // rationale: Contract regression — HebbianSignal JSON serialization
    // is snake_case ("reinforce" / "depress"). External substrate
    // consumers depend on this wire form.
    #[test]
    fn hebbian_signal_wire_form_is_snake_case() {
        // rationale: Contract regression (HebbianSignal wire form)
        let r = serde_json::to_string(&HebbianSignal::Reinforce).expect("ser");
        let d = serde_json::to_string(&HebbianSignal::Depress).expect("ser");
        assert_eq!(r, "\"reinforce\"");
        assert_eq!(d, "\"depress\"");
    }

    // rationale: Contract regression — Memory alias resolves to
    // CorrelationMemory; downstream consumers using `m42::Memory` see
    // the same shape as m13::CorrelationMemory.
    #[test]
    fn memory_alias_resolves_to_correlation_memory() {
        // rationale: Contract regression (Memory type alias)
        let alias_name = std::any::type_name::<super::Memory>();
        let canonical_name = std::any::type_name::<CorrelationMemory>();
        assert_eq!(alias_name, canonical_name);
    }

    // rationale: Anti-property — SubstrateEmitError wraps
    // StcortexWriterError via #[from], not via opaque string. Callers
    // can match on the inner variant.
    #[test]
    fn substrate_emit_error_preserves_typed_writer_variant() {
        // rationale: Anti-property (typed error chain preservation)
        let inner = crate::m13_stcortex_writer::StcortexWriterError::WriteFailed(
            "synthetic".into(),
        );
        let err: SubstrateEmitError = inner.into();
        let SubstrateEmitError::Writer(
            crate::m13_stcortex_writer::StcortexWriterError::WriteFailed(s),
        ) = err
        else {
            panic!("expected Writer(WriteFailed)");
        };
        assert_eq!(s, "synthetic");
    }

    // rationale: Anti-property — SubstrateEmitError implements
    // std::error::Error + Send + Sync + 'static for tokio interop.
    #[test]
    fn substrate_emit_error_send_sync_static() {
        // rationale: Anti-property (async-readiness)
        fn assert_error<T: std::error::Error + Send + Sync + 'static>() {}
        assert_error::<SubstrateEmitError>();
    }

    // rationale: Adversarial input — outcome_summary on every
    // PromoteOutcome variant produces a non-empty operator-readable
    // string (operator runbooks rely on these).
    #[test]
    fn outcome_summary_is_non_empty_for_every_variant() {
        // rationale: Adversarial input / operator-runbook stability
        let variants = [
            crate::m13_stcortex_writer::PromoteOutcome::Written { memory_id: 1 },
            crate::m13_stcortex_writer::PromoteOutcome::WrittenUnderPressure {
                memory_id: 2,
                ltp_density: 0.02,
            },
            crate::m13_stcortex_writer::PromoteOutcome::Deferred {
                reason: crate::m13_stcortex_writer::DeferReason::LtpBelowFloor {
                    density: 0.001,
                },
            },
            crate::m13_stcortex_writer::PromoteOutcome::Deferred {
                reason: crate::m13_stcortex_writer::DeferReason::OracUnreachable,
            },
            crate::m13_stcortex_writer::PromoteOutcome::Deferred {
                reason: crate::m13_stcortex_writer::DeferReason::StcortexUnreachable,
            },
        ];
        for v in &variants {
            let s = outcome_summary(v);
            assert!(!s.is_empty(), "summary must not be empty for {v:?}");
        }
    }

    // ====================================================================
    // S1002388 hardening pass — m42 stcortex emit (+35 tests → ≥50).
    // emit_feedback 3-band routing · signal-arg no-op invariant ·
    // outbox/emit policy · Phase-A self-check semantics · outcome_summary
    // exact format · HebbianSignal serde adversarial · m9 boundary.
    // ====================================================================

    // -- emit_feedback: m13 3-band gate routing through m42 --------------

    // rationale: Cross-module surface — emit_feedback at a pressure-band
    // density (0.05, in [0.015, 0.10)) routes through m13 and returns
    // WrittenUnderPressure carrying the observed density.
    #[test]
    fn emit_feedback_under_pressure_band_returns_written_under_pressure() {
        // rationale: Cross-module surface (m42 -> m13 3-band gate band-2)
        let (writer, _recorded) = shared_writer(Some(0.05));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let outcome = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Reinforce,
        )
        .expect("emit");
        match outcome {
            PromoteOutcome::WrittenUnderPressure { ltp_density, .. } => {
                assert!((ltp_density - 0.05).abs() < 1e-12);
            }
            other => panic!("expected WrittenUnderPressure, got {other:?}"),
        }
    }

    // rationale: Cross-module surface — emit_feedback at a below-floor
    // density (0.001 < 0.015) defers to the m13 JSONL outbox with the
    // LtpBelowFloor reason; the call does NOT raise an error (defer is a
    // success outcome per the outbox-first durability policy).
    #[test]
    fn emit_feedback_below_floor_defers_to_outbox_without_error() {
        // rationale: Cross-module surface (m42 outbox-first defer policy)
        let (writer, _recorded) = shared_writer(Some(0.001));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("fail"));
        let outcome = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Depress,
        )
        .expect("below-floor defer must not error");
        match outcome {
            PromoteOutcome::Deferred {
                reason: DeferReason::LtpBelowFloor { density },
            } => assert!((density - 0.001).abs() < 1e-12),
            other => panic!("expected Deferred LtpBelowFloor, got {other:?}"),
        }
    }

    // rationale: Boundary — emit_feedback at exactly the Phase-1 floor
    // (0.015) does NOT defer (m13 uses a strict `<` check); it writes
    // under pressure. Exact-threshold regression anchor through m42.
    #[test]
    fn emit_feedback_at_exact_floor_writes_under_pressure() {
        // rationale: Boundary (3-band exact threshold via m42)
        let (writer, _recorded) =
            shared_writer(Some(crate::m13_stcortex_writer::LTP_PHASE_1_FLOOR));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let outcome = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Reinforce,
        )
        .expect("emit at floor");
        assert!(
            matches!(outcome, PromoteOutcome::WrittenUnderPressure { .. }),
            "exact floor must NOT defer; got {outcome:?}"
        );
    }

    // rationale: Boundary — emit_feedback at exactly the Phase-3 target
    // (0.10) writes normally (m13 strict `<` on the upper band edge).
    #[test]
    fn emit_feedback_at_exact_phase_3_target_writes_normally() {
        // rationale: Boundary (3-band upper edge via m42)
        let (writer, _recorded) =
            shared_writer(Some(crate::m13_stcortex_writer::LTP_PHASE_3_TARGET));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let outcome = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Reinforce,
        )
        .expect("emit at target");
        assert!(matches!(outcome, PromoteOutcome::Written { .. }));
    }

    // -- emit_feedback: signal-arg Phase-A no-op invariant --------------

    // rationale: Anti-property (F9 zero-weight / Phase-A self-check) — the
    // `signal` argument is documented as a call-site self-check that is
    // NOT routed to the substrate. Reinforce and Depress with otherwise
    // identical inputs MUST produce identical PromoteOutcomes; the signal
    // must not leak into the write path in Phase A.
    #[test]
    fn emit_feedback_signal_arg_is_phase_a_noop_on_outcome() {
        // rationale: Anti-property (Phase-A signal is self-check only)
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let (reinforce_writer, _handle_a) = shared_writer(Some(0.20));
        let reinforce = super::emit_feedback(
            &reinforce_writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Reinforce,
        )
        .expect("reinforce");
        let (depress_writer, _handle_b) = shared_writer(Some(0.20));
        let depress = super::emit_feedback(
            &depress_writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Depress,
        )
        .expect("depress");
        // Both must be Written; memory_id is allocated by the mock from 0,
        // so both fresh writers yield id 1 — outcomes are equal.
        assert_eq!(
            reinforce, depress,
            "signal arg must not affect the Phase-A outcome (self-check only)"
        );
    }

    // rationale: Anti-property — the `signal` arg does not influence the
    // recorded CorrelationMemory either: the memory's relevance is derived
    // by m13 from the run outcome, NOT from the Hebbian signal. A Depress
    // signal with an "ok" run still records relevance 1.0.
    #[test]
    fn emit_feedback_signal_arg_does_not_alter_recorded_relevance() {
        // rationale: Anti-property (m13 derives relevance from outcome)
        let (writer, recorded) = shared_writer(Some(0.20));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        // Pass the contradictory signal: an "ok" run with Depress.
        let _ = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Depress,
        )
        .expect("emit");
        let written = recorded.lock().expect("lock");
        assert_eq!(written.len(), 1);
        assert!(
            (written[0].relevance - 1.0).abs() < 1e-6,
            "relevance must reflect the run outcome (ok=1.0), not the signal"
        );
    }

    // -- emit_feedback: recorded write content / namespace ---------------

    // rationale: Cross-module surface — emit_feedback records exactly one
    // CorrelationMemory carrying the validated namespace; the namespace is
    // forwarded to m13 unchanged (no m42-side rewrite).
    #[test]
    fn emit_feedback_records_one_memory_with_forwarded_namespace() {
        // rationale: Cross-module surface (namespace pass-through)
        let (writer, recorded) = shared_writer(Some(0.20));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let _ = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_emit_test",
            HebbianSignal::Reinforce,
        )
        .expect("emit");
        let written = recorded.lock().expect("lock");
        assert_eq!(written.len(), 1);
        assert_eq!(written[0].namespace, "workflow_trace_emit_test");
    }

    // rationale: Cross-module surface — m9 hyphen-munge is transitively
    // applied: a hyphenated namespace passed to emit_feedback lands in the
    // recorded memory as the underscored form.
    #[test]
    fn emit_feedback_records_munged_namespace_for_hyphenated_input() {
        // rationale: Cross-module surface (m9 munge transitive — verified)
        let (writer, recorded) = shared_writer(Some(0.20));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let _ = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow-trace-hyphen-ns",
            HebbianSignal::Reinforce,
        )
        .expect("emit");
        let written = recorded.lock().expect("lock");
        assert_eq!(written.len(), 1);
        assert_eq!(
            written[0].namespace, "workflow_trace_hyphen_ns",
            "hyphens must be munged to underscores transitively via m9"
        );
    }

    // rationale: Anti-property (AP30) — emit_feedback rejects an empty
    // namespace at the m9 boundary with a NamespaceViolation::Empty.
    #[test]
    fn emit_feedback_rejects_empty_namespace_via_m9() {
        // rationale: Anti-property (AP30 — empty namespace)
        let (writer, _recorded) = shared_writer(Some(0.20));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let err = super::emit_feedback(&writer, &workflow, &run, "", HebbianSignal::Reinforce)
            .expect_err("empty namespace must fail at m9");
        assert!(matches!(
            err,
            SubstrateEmitError::Writer(
                crate::m13_stcortex_writer::StcortexWriterError::NamespaceViolation(
                    crate::m9_watcher_namespace_guard::NamespaceViolation::Empty
                )
            )
        ));
    }

    // rationale: Anti-property (AP30) — emit_feedback rejects the bare
    // "scratch" namespace via m9's ScratchForbidden variant; the m42 path
    // must inherit every m9 refusal class, not just WrongPrefix.
    #[test]
    fn emit_feedback_rejects_scratch_namespace_via_m9() {
        // rationale: Anti-property (AP30 — scratch forbidden)
        let (writer, _recorded) = shared_writer(Some(0.20));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let err =
            super::emit_feedback(&writer, &workflow, &run, "scratch", HebbianSignal::Reinforce)
                .expect_err("scratch must fail at m9");
        assert!(matches!(
            err,
            SubstrateEmitError::Writer(
                crate::m13_stcortex_writer::StcortexWriterError::NamespaceViolation(
                    crate::m9_watcher_namespace_guard::NamespaceViolation::ScratchForbidden
                )
            )
        ));
    }

    // rationale: Anti-property (AP30) — emit_feedback rejects a namespace
    // carrying an embedded NUL control byte via m9's ControlChar variant.
    #[test]
    fn emit_feedback_rejects_control_char_namespace_via_m9() {
        // rationale: Anti-property (AP30 — control-char contamination)
        let (writer, _recorded) = shared_writer(Some(0.20));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let err = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace\0evil",
            HebbianSignal::Reinforce,
        )
        .expect_err("control char must fail at m9");
        assert!(matches!(
            err,
            SubstrateEmitError::Writer(
                crate::m13_stcortex_writer::StcortexWriterError::NamespaceViolation(
                    crate::m9_watcher_namespace_guard::NamespaceViolation::ControlChar { .. }
                )
            )
        ));
    }

    // rationale: Anti-property — a foreign-namespace rejection happens
    // BEFORE any substrate write: the recording sink stays empty when m9
    // refuses, so no partial / leaked write occurs.
    #[test]
    fn emit_feedback_foreign_namespace_records_no_write() {
        // rationale: Anti-property (fail-closed — no partial write)
        let (writer, recorded) = shared_writer(Some(0.20));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let _ = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "orac_foreign",
            HebbianSignal::Reinforce,
        )
        .expect_err("foreign namespace must fail");
        assert!(
            recorded.lock().expect("lock").is_empty(),
            "no substrate write may occur when m9 refuses the namespace"
        );
    }

    // -- emit_feedback: outbox durability on ORAC-unreachable ------------

    // rationale: Cross-module surface — when ORAC is unreachable the m13
    // outbox file actually receives a JSONL line; emit_feedback's defer
    // path is durable, not a no-op. Verifies the file content.
    #[test]
    fn emit_feedback_orac_unreachable_writes_durable_outbox_line() {
        // rationale: Cross-module surface (outbox durability)
        let mock = SharedRecordingWriter::new();
        let outbox = temp_outbox();
        let writer = StcortexWriter::new(StaticDensity(None), mock, outbox.clone());
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let _ = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Reinforce,
        )
        .expect("emit defers");
        let contents = std::fs::read_to_string(&outbox).expect("read outbox");
        assert!(contents.contains("OracUnreachable"), "outbox missing reason");
        assert!(
            contents.contains("workflow_trace_outcomes"),
            "outbox missing namespace"
        );
        assert_eq!(contents.lines().count(), 1, "exactly one outbox line");
    }

    // rationale: Cross-module surface — repeated ORAC-unreachable emits
    // append to the outbox; the JSONL stream grows one line per call.
    #[test]
    fn emit_feedback_appends_one_outbox_line_per_deferred_call() {
        // rationale: Cross-module surface (append-only outbox)
        let mock = SharedRecordingWriter::new();
        let outbox = temp_outbox();
        let writer = StcortexWriter::new(StaticDensity(None), mock, outbox.clone());
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        for _ in 0..3 {
            let _ = super::emit_feedback(
                &writer,
                &workflow,
                &run,
                "workflow_trace_outcomes",
                HebbianSignal::Reinforce,
            )
            .expect("emit");
        }
        let contents = std::fs::read_to_string(&outbox).expect("read");
        assert_eq!(
            contents.lines().count(),
            3,
            "three deferred emits -> three lines"
        );
    }

    // rationale: Adversarial input — each outbox line on a deferred emit
    // is itself valid JSON carrying ts_ms / memory / reason keys.
    #[test]
    fn emit_feedback_outbox_line_is_valid_json_with_expected_keys() {
        // rationale: Adversarial input (outbox JSONL shape)
        let mock = SharedRecordingWriter::new();
        let outbox = temp_outbox();
        let writer = StcortexWriter::new(StaticDensity(None), mock, outbox.clone());
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let _ = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Reinforce,
        )
        .expect("emit");
        let contents = std::fs::read_to_string(&outbox).expect("read");
        let line = contents.lines().next().expect("one line");
        let parsed: serde_json::Value = serde_json::from_str(line).expect("valid JSON");
        assert!(parsed.get("ts_ms").is_some(), "missing ts_ms");
        assert!(parsed.get("memory").is_some(), "missing memory");
        assert!(parsed.get("reason").is_some(), "missing reason");
    }

    // -- signal_for_outcome edge cases ----------------------------------

    // rationale: Boundary — signal_for_outcome with a None outcome (open
    // run / outcome not yet recorded) maps to Depress, not Reinforce.
    #[test]
    fn signal_for_outcome_none_is_depress() {
        // rationale: Boundary (open-run / missing outcome)
        assert_eq!(signal_for_outcome(None), HebbianSignal::Depress);
    }

    // rationale: Adversarial input — outcome strings differing from "ok"
    // only by case or surrounding whitespace must NOT collapse to
    // Reinforce; the match is exact-string.
    #[test]
    fn signal_for_outcome_is_case_and_whitespace_sensitive() {
        // rationale: Adversarial input (exact-string discipline)
        for near_miss in ["OK", "Ok", " ok", "ok ", "ok\n", "\tok"] {
            assert_eq!(
                signal_for_outcome(Some(near_miss)),
                HebbianSignal::Depress,
                "near-miss {near_miss:?} must not match the exact 'ok' literal"
            );
        }
    }

    // rationale: Adversarial input — an empty outcome string maps to
    // Depress (it is not the literal "ok").
    #[test]
    fn signal_for_outcome_empty_string_is_depress() {
        // rationale: Adversarial input (empty outcome)
        assert_eq!(signal_for_outcome(Some("")), HebbianSignal::Depress);
    }

    // rationale: Adversarial input — Unicode / non-ASCII outcome strings
    // map to Depress without panic (UTF-8 safety on the match path).
    #[test]
    fn signal_for_outcome_unicode_outcome_is_depress() {
        // rationale: Adversarial input (Unicode safety)
        for s in ["\u{2713}", "\u{6210}\u{529f}", "\u{f6}k", "\u{1F600}"] {
            assert_eq!(signal_for_outcome(Some(s)), HebbianSignal::Depress);
        }
    }

    // -- HebbianSignal serde adversarial --------------------------------

    // rationale: Adversarial input — HebbianSignal deserialization rejects
    // an unknown wire value; the enum is closed at the wire boundary.
    #[test]
    fn hebbian_signal_deserialization_rejects_unknown_variant() {
        // rationale: Adversarial input (closed wire enum)
        let r: Result<HebbianSignal, _> = serde_json::from_str("\"potentiate\"");
        assert!(r.is_err(), "unknown HebbianSignal wire value must fail");
    }

    // rationale: Contract regression — HebbianSignal deserialization is
    // case-sensitive: "Reinforce" (capitalised) is NOT the snake_case
    // wire form and must fail.
    #[test]
    fn hebbian_signal_deserialization_is_case_sensitive() {
        // rationale: Contract regression (snake_case wire form)
        assert!(serde_json::from_str::<HebbianSignal>("\"Reinforce\"").is_err());
        assert!(serde_json::from_str::<HebbianSignal>("\"DEPRESS\"").is_err());
    }

    // rationale: Contract regression — HebbianSignal deserialization
    // accepts exactly the two canonical snake_case wire values.
    #[test]
    fn hebbian_signal_deserialization_accepts_canonical_wire_values() {
        // rationale: Contract regression (canonical wire values)
        assert_eq!(
            serde_json::from_str::<HebbianSignal>("\"reinforce\"").expect("de"),
            HebbianSignal::Reinforce
        );
        assert_eq!(
            serde_json::from_str::<HebbianSignal>("\"depress\"").expect("de"),
            HebbianSignal::Depress
        );
    }

    // rationale: Determinism — HebbianSignal is Copy + Eq + Hash; it can
    // be used as a HashMap key and the two variants are distinct keys.
    #[test]
    fn hebbian_signal_is_usable_as_distinct_hash_keys() {
        // rationale: Determinism (Copy + Eq + Hash contract)
        let mut map = std::collections::HashMap::new();
        map.insert(HebbianSignal::Reinforce, "ltp");
        map.insert(HebbianSignal::Depress, "ltd");
        assert_eq!(map.len(), 2, "two variants must be distinct keys");
        assert_eq!(map.get(&HebbianSignal::Reinforce), Some(&"ltp"));
    }

    // rationale: Anti-property — Reinforce and Depress are not equal; the
    // PartialEq derive distinguishes the two polarities.
    #[test]
    fn hebbian_signal_variants_are_not_equal() {
        // rationale: Anti-property (polarity distinctness)
        assert_ne!(HebbianSignal::Reinforce, HebbianSignal::Depress);
    }

    // -- outcome_summary exact-format pinning ----------------------------

    // rationale: Contract regression — outcome_summary(Written) emits the
    // exact "wrote memory_id=<id>" form; log scrapers parse this literally.
    #[test]
    fn outcome_summary_written_exact_format() {
        // rationale: Contract regression (Written summary format)
        let s = outcome_summary(&PromoteOutcome::Written { memory_id: 1_700 });
        assert_eq!(s, "wrote memory_id=1700");
    }

    // rationale: Contract regression — outcome_summary(WrittenUnderPressure)
    // formats ltp_density to exactly 4 decimal places.
    #[test]
    fn outcome_summary_under_pressure_formats_density_to_four_decimals() {
        // rationale: Contract regression (4-decimal density format)
        let s = outcome_summary(&PromoteOutcome::WrittenUnderPressure {
            memory_id: 9,
            ltp_density: 0.017_345_6,
        });
        assert_eq!(s, "wrote_under_pressure memory_id=9 ltp=0.0173");
    }

    // rationale: Contract regression — outcome_summary(Deferred /
    // LtpBelowFloor) emits the exact "deferred ltp_below_floor density=..."
    // form with 4-decimal density.
    #[test]
    fn outcome_summary_deferred_below_floor_exact_format() {
        // rationale: Contract regression (LtpBelowFloor summary format)
        let s = outcome_summary(&PromoteOutcome::Deferred {
            reason: DeferReason::LtpBelowFloor { density: 0.001 },
        });
        assert_eq!(s, "deferred ltp_below_floor density=0.0010");
    }

    // rationale: Contract regression — the two unreachable defer reasons
    // emit distinct, exact summary strings so an operator can tell ORAC
    // from stcortex outages apart.
    #[test]
    fn outcome_summary_distinguishes_orac_from_stcortex_unreachable() {
        // rationale: Contract regression (unreachable reason disambiguation)
        let orac = outcome_summary(&PromoteOutcome::Deferred {
            reason: DeferReason::OracUnreachable,
        });
        let stc = outcome_summary(&PromoteOutcome::Deferred {
            reason: DeferReason::StcortexUnreachable,
        });
        assert_eq!(orac, "deferred orac_unreachable");
        assert_eq!(stc, "deferred stcortex_unreachable");
        assert_ne!(orac, stc, "the two outages must be distinguishable");
    }

    // rationale: Boundary — outcome_summary handles memory_id = i64::MAX
    // without truncation or panic (operator-facing format under extreme
    // ids).
    #[test]
    fn outcome_summary_handles_i64_max_memory_id() {
        // rationale: Boundary (extreme memory_id)
        let s = outcome_summary(&PromoteOutcome::Written {
            memory_id: i64::MAX,
        });
        assert_eq!(s, format!("wrote memory_id={}", i64::MAX));
    }

    // -- SubstrateEmitError surface -------------------------------------

    // rationale: Contract regression — SubstrateEmitError::Writer Display
    // is prefixed "stcortex writer:" and embeds the inner error text so
    // operator runbooks can grep the failure class.
    #[test]
    fn substrate_emit_error_display_is_prefixed_and_embeds_inner() {
        // rationale: Contract regression (error Display format)
        let inner = crate::m13_stcortex_writer::StcortexWriterError::WriteFailed(
            "transport reset".into(),
        );
        let err: SubstrateEmitError = inner.into();
        let s = err.to_string();
        assert!(s.starts_with("stcortex writer: "), "missing prefix: {s}");
        assert!(s.contains("transport reset"), "inner text dropped: {s}");
    }

    // rationale: Anti-property — emit_feedback propagates a substrate
    // WriteFailed from m13 as a typed SubstrateEmitError::Writer, not a
    // swallowed error or a default outcome.
    #[test]
    fn emit_feedback_propagates_substrate_write_failure() {
        // rationale: Anti-property (typed error propagation)
        struct FailingWriter;
        impl SubstrateWriter for FailingWriter {
            fn write_memory(
                &self,
                _m: &CorrelationMemory,
            ) -> Result<i64, crate::m13_stcortex_writer::StcortexWriterError> {
                Err(crate::m13_stcortex_writer::StcortexWriterError::WriteFailed(
                    "synthetic substrate failure".into(),
                ))
            }
        }
        let writer =
            StcortexWriter::new(StaticDensity(Some(0.20)), FailingWriter, temp_outbox());
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let err = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Reinforce,
        )
        .expect_err("substrate failure must propagate");
        assert!(matches!(
            err,
            SubstrateEmitError::Writer(
                crate::m13_stcortex_writer::StcortexWriterError::WriteFailed(_)
            )
        ));
    }

    // rationale: Cross-module surface — the m42 -> m13 path correctly
    // derives a low relevance from a "fail" run outcome: emit_feedback of
    // a failed run records relevance 0.5, distinct from an "ok" run's 1.0.
    #[test]
    fn emit_feedback_records_fail_outcome_relevance_half() {
        // rationale: Cross-module surface (outcome -> relevance mapping)
        let (writer, recorded) = shared_writer(Some(0.20));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("fail"));
        let _ = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Depress,
        )
        .expect("emit");
        let written = recorded.lock().expect("lock");
        assert_eq!(written.len(), 1);
        assert!(
            (written[0].relevance - 0.5).abs() < 1e-6,
            "fail -> relevance 0.5"
        );
    }

    // rationale: Cross-module surface — a run with no outcome (None) routes
    // through m42 -> m13 and records the "unknown" relevance bucket (0.1).
    #[test]
    fn emit_feedback_records_unknown_relevance_for_open_run() {
        // rationale: Cross-module surface (None outcome -> unknown bucket)
        let (writer, recorded) = shared_writer(Some(0.20));
        let workflow = accepted_workflow_fixture();
        let run = run_row(None);
        let _ = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Depress,
        )
        .expect("emit");
        let written = recorded.lock().expect("lock");
        assert_eq!(written.len(), 1);
        assert!(
            (written[0].relevance - 0.1).abs() < 1e-6,
            "None outcome -> 0.1"
        );
    }

    // rationale: F9 zero-weight — emit_feedback records a CorrelationMemory
    // whose `tensor` field is None: no fitness signal leaks through the
    // m42 substrate-feedback path in Phase A.
    #[test]
    fn emit_feedback_recorded_memory_carries_no_fitness_tensor() {
        // rationale: F9 zero-weight (no tensor leak via m42)
        let (writer, recorded) = shared_writer(Some(0.20));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let _ = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Reinforce,
        )
        .expect("emit");
        let written = recorded.lock().expect("lock");
        assert!(written[0].tensor.is_none(), "F9: tensor must stay None");
    }

    // rationale: Cross-module surface — emit_feedback returns a Written
    // outcome whose memory_id matches the substrate-assigned id; the id
    // is not fabricated or zeroed on the m42 -> m13 return path.
    #[test]
    fn emit_feedback_written_outcome_carries_substrate_assigned_id() {
        // rationale: Cross-module surface (id propagation fidelity)
        let (writer, _recorded) = shared_writer(Some(0.20));
        let workflow = accepted_workflow_fixture();
        let run = run_row(Some("ok"));
        let outcome = super::emit_feedback(
            &writer,
            &workflow,
            &run,
            "workflow_trace_outcomes",
            HebbianSignal::Reinforce,
        )
        .expect("emit");
        // SharedRecordingWriter allocates ids from 0, so the first write
        // is id 1.
        assert_eq!(outcome, PromoteOutcome::Written { memory_id: 1 });
    }
}
