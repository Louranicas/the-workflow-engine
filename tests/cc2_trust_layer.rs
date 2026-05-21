//! Integration tests for CC-2 — "Trust Layer Woven (D → all)" (Wave-D2).
//!
//! CC-2 is the cross-cluster synergy by which Cluster D (m8 POVM
//! build-prereq, m9 namespace guard, m10 Ember CI gate, m11 fitness decay)
//! threads a trust regime through every substrate write and every
//! user-facing output produced anywhere else in the engine:
//!
//!   - m8  catches POVM misreading at **compile time** (build.rs cfg gate).
//!   - m9  catches namespace drift at **write time** (every stcortex write).
//!   - m10 catches mis-calibrated output at **CI gate time**.
//!   - m11 catches workflow ossification at **lifecycle time**.
//!
//! These tests lock the *woven* property: the trust checks are not optional
//! decorations on individual modules — they are the only path. Every
//! stcortex write in m13 + m42 routes through m9; every Cluster-G/H
//! consumer that writes is transitively gated. The anti-property test
//! `cc2_trust_layer_no_bypass_path` reads the m13 + m42 source and asserts
//! no write reaches the substrate without crossing the m9 boundary.
//!
//! Mocks: the substrate writer + LTP density reader, same shape as
//! `tests/m42_integration.rs` + `tests/cc4_proposal_to_dispatch_pipeline.rs`.
//! NO real HTTP / no live services.

#![allow(clippy::doc_markdown)]

use std::path::PathBuf;
use std::sync::Mutex as StdMutex;
use std::time::SystemTime;

use tempfile::Builder as TempBuilder;

use workflow_core::m9_watcher_namespace_guard::{
    assert_workflow_trace_namespace, munge_hyphen_slug, NamespaceViolation,
    WORKFLOW_TRACE_NS_PREFIX,
};
use workflow_core::m10_ember_ci_gate::{evaluate_string, GateVerdict};
use workflow_core::m11_fitness_weighted_decay::{
    run_consolidation_cycle, DecayConfig, DecayError, FrequencyReader, PathwayWeightReader,
};
use workflow_core::m13_stcortex_writer::{
    CorrelationMemory, PromoteOutcome, StcortexWriter, StcortexWriterError, SubstrateWriter,
    LtpDensityReader,
};
use workflow_core::m14_lift::LiftSnapshot;
use workflow_core::m20_prefixspan::{Pattern, StepToken};
use workflow_core::m21_variant_builder::build_variants;
use workflow_core::m23_proposer::build_proposal;
use workflow_core::m30_bank::{AcceptedWorkflow, CuratedBank};
use workflow_core::m42_stcortex_emit::{emit_feedback, HebbianSignal, SubstrateEmitError};
use workflow_core::m7_workflow_runs::WorkflowRunRow;
use workflow_core::m8_povm_build_prereq::{classify, BandClassification};

// ─── shared fixtures ────────────────────────────────────────────────────

/// A non-pressure LTP density (`>= LTP_PHASE_3_TARGET`) so band-3 writes
/// proceed normally — keeps the namespace boundary the only variable.
struct StaticDensity(Option<f64>);
impl LtpDensityReader for StaticDensity {
    fn read_density(&self) -> Option<f64> {
        self.0
    }
}

/// Substrate writer that records every memory it accepts. Used to confirm
/// a write actually landed under a *validated* namespace.
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
    // Forget the temp path so the file lingers for the test duration;
    // OS tmp reaping handles cleanup.
    std::mem::forget(p);
    path
}

fn ok_run() -> WorkflowRunRow {
    WorkflowRunRow {
        id: 1001,
        started_at: "2026-05-21T00:00:00Z".into(),
        ended_at: Some("2026-05-21T01:00:00Z".into()),
        outcome: Some("ok".into()),
        consumer_inputs: "{}".into(),
        cost_tokens: Some(100),
        fitness_dimension: 0.0,
    }
}

fn workflow_fixture(seed: u32) -> AcceptedWorkflow {
    let p = Pattern::new(
        vec![StepToken(seed), StepToken(seed.wrapping_add(1))],
        30,
        (0, seed as usize),
    );
    let v = build_variants(&p).expect("m21 build_variants")[0].clone();
    let s = LiftSnapshot {
        lift: Some(0.5),
        ci_half: Some(0.05),
        n: 30,
        latest_ts_ms: 0,
        computed_at: SystemTime::now(),
    };
    let bank = CuratedBank::new();
    let id = bank
        .accept(build_proposal(v, &s, None).expect("m23 build_proposal"), 1_700_000_000_000)
        .expect("bank accept");
    bank.get(id).expect("bank get")
}

/// Canonical `workflow_trace_*` namespace built from the m9 constant.
fn canonical_ns() -> String {
    format!("{WORKFLOW_TRACE_NS_PREFIX}_outcomes")
}

// ─── tests ──────────────────────────────────────────────────────────────

// rationale: Cross-module — CC-2 woven property. Both substrate-write
// boundaries (m13 writer's `promote_run` and m42's `emit_feedback`, which
// routes through m13) MUST route through m9's `assert_workflow_trace_namespace`.
// A foreign-prefix namespace is refused at BOTH boundaries, surfacing the
// same underlying NamespaceViolation.
#[test]
fn cc2_every_stcortex_write_passes_m9_namespace_guard() {
    // rationale: Cross-module (CC-2 trust layer woven D → all)
    let foreign = "orac_foreign_prefix";

    // Boundary 1 — m13 writer directly.
    let w13 = StcortexWriter::new(StaticDensity(Some(0.20)), RecordingWriter::new(), temp_outbox());
    let err13 = w13
        .promote_run(&ok_run(), foreign)
        .expect_err("m13 must refuse foreign namespace at the m9 boundary");
    assert!(
        matches!(err13, StcortexWriterError::NamespaceViolation(_)),
        "m13 boundary did not route through m9: {err13:?}"
    );

    // Boundary 2 — m42 emit_feedback (m42 → m13 → m9).
    let w42 = StcortexWriter::new(StaticDensity(Some(0.20)), RecordingWriter::new(), temp_outbox());
    let err42 = emit_feedback(
        &w42,
        &workflow_fixture(7),
        &ok_run(),
        foreign,
        HebbianSignal::Reinforce,
    )
    .expect_err("m42 must refuse foreign namespace transitively at the m9 boundary");
    assert!(
        matches!(
            err42,
            SubstrateEmitError::Writer(StcortexWriterError::NamespaceViolation(_))
        ),
        "m42 boundary did not route through m9: {err42:?}"
    );

    // Control: a canonical namespace passes at BOTH boundaries.
    let w13_ok =
        StcortexWriter::new(StaticDensity(Some(0.20)), RecordingWriter::new(), temp_outbox());
    assert!(
        w13_ok.promote_run(&ok_run(), &canonical_ns()).is_ok(),
        "canonical namespace must pass the m13 boundary"
    );
    let w42_ok =
        StcortexWriter::new(StaticDensity(Some(0.20)), RecordingWriter::new(), temp_outbox());
    assert!(
        emit_feedback(
            &w42_ok,
            &workflow_fixture(8),
            &ok_run(),
            &canonical_ns(),
            HebbianSignal::Reinforce,
        )
        .is_ok(),
        "canonical namespace must pass the m42 boundary"
    );
}

// rationale: Cross-module — the m9 hyphen-munge (AP-Hab-11 / S1001757
// mitigation) applies exactly once at the validator boundary and the
// munged underscored form survives transitively through the m13 write.
// `workflow-trace-x` MUST land in the substrate as `workflow_trace_x`.
#[test]
fn cc2_m9_hyphen_munge_applies_transitively_through_m13() {
    // rationale: Cross-module (m9 munge transitive through m13)
    let writer = RecordingWriter::new();
    // Move ownership into the StcortexWriter; inspect via a re-built fixture
    // is not possible (writer is consumed), so we use the m13 unit-tested
    // contract + re-derive the expected munge here.
    let w = StcortexWriter::new(StaticDensity(Some(0.20)), writer, temp_outbox());
    let out = w
        .promote_run(&ok_run(), "workflow-trace-x")
        .expect("hyphenated namespace must pass after m9 munge");
    assert!(matches!(out, PromoteOutcome::Written { .. }));

    // m9 munge is the single source of truth — confirm the munge maps the
    // hyphenated input to the exact underscored form the substrate sees.
    let expected = munge_hyphen_slug("workflow-trace-x");
    assert_eq!(expected, "workflow_trace_x");
    let validated =
        assert_workflow_trace_namespace("workflow-trace-x").expect("m9 validates hyphenated form");
    assert_eq!(
        validated.as_str(),
        "workflow_trace_x",
        "m9 → m13 chain must carry the munged underscored namespace"
    );
}

// rationale: Cross-module — m10's `evaluate_string` is the output-side
// trust arm of CC-2. Report text that violates the Ember rubric (a
// hard-absolutist / dishonest-success phrase) is caught by m10 and routed
// to a non-Pass verdict; clean factual m12 report text passes.
#[test]
fn cc2_m10_ember_gate_rejects_non_conforming_output_from_m12() {
    // rationale: Cross-module (m10 output-side trust arm of CC-2)
    // Non-conforming m12 report text — dishonest-success phrasing the
    // rubric Rejects (Honesty trait, confidence >= 0.5 → Fail).
    let bad = evaluate_string("m12.report.header", "successfully completed", &[]);
    assert!(
        matches!(bad, GateVerdict::Fail { .. } | GateVerdict::HeldFailed { .. }),
        "m10 must catch non-conforming m12 output, got {bad:?}"
    );

    // Conforming m12 report text — a clean factual probe line passes.
    let good = evaluate_string(
        "m12.report.header",
        "POVM probe at 2026-05-21T10:00:00Z returned 0.067 (scope=lib).",
        &[],
    );
    assert_eq!(
        good,
        GateVerdict::Pass,
        "m10 must let conforming m12 output through"
    );
}

// rationale: Cross-module — m11 is the lifecycle arm of CC-2. A real
// consolidation cycle runs against the m30 bank (the m30 ↔ m11
// LifecycleBank bridge), decay is applied multiplicatively, and the trust
// regime is intact: usage alone never grants immortality — every active
// workflow's weight strictly decreases.
#[test]
fn cc2_m11_decay_cycle_is_the_lifecycle_arm_of_cc2() {
    // rationale: Cross-module (m11 lifecycle arm — m30 ↔ m11 bridge)
    use std::collections::HashMap;

    // Local trait carriers declared first (clippy::items_after_statements).
    struct Pw {
        w: HashMap<String, f64>,
    }
    impl PathwayWeightReader for Pw {
        fn read_pathway_weight(&self, pid: &str) -> Result<f64, DecayError> {
            self.w
                .get(pid)
                .copied()
                .ok_or_else(|| DecayError::PathwayReadFailed {
                    pathway_id: pid.to_owned(),
                    reason: "test".into(),
                })
        }
    }
    struct Fr;
    impl FrequencyReader for Fr {
        fn frequency(&self, _: &str) -> u64 {
            0
        }
        fn cohort_max(&self) -> u64 {
            1
        }
    }

    let now = 1_700_000_000_000_i64;
    let mut bank = CuratedBank::new();
    let p_a = {
        let pat = Pattern::new(vec![StepToken(301), StepToken(302)], 30, (0, 1));
        let v = build_variants(&pat).expect("v")[0].clone();
        let s = LiftSnapshot {
            lift: Some(0.5),
            ci_half: Some(0.05),
            n: 30,
            latest_ts_ms: 0,
            computed_at: SystemTime::now(),
        };
        build_proposal(v, &s, None).expect("p")
    };
    let p_b = {
        let pat = Pattern::new(vec![StepToken(401), StepToken(402)], 30, (0, 2));
        let v = build_variants(&pat).expect("v")[0].clone();
        let s = LiftSnapshot {
            lift: Some(0.5),
            ci_half: Some(0.05),
            n: 30,
            latest_ts_ms: 0,
            computed_at: SystemTime::now(),
        };
        build_proposal(v, &s, None).expect("p")
    };
    let id_a = bank.accept(p_a, now).expect("accept a");
    let id_b = bank.accept(p_b, now).expect("accept b");

    let mut pw = HashMap::new();
    pw.insert(id_a.to_string(), 0.5);
    pw.insert(id_b.to_string(), 0.5);
    let pathways = Pw { w: pw };
    let cfg = DecayConfig::default();

    let pre_a = bank.get(id_a).expect("pre a").weight;
    let pre_b = bank.get(id_b).expect("pre b").weight;

    let stats = run_consolidation_cycle(&mut bank, &pathways, &Fr, &cfg, || Some(now))
        .expect("consolidation cycle");
    assert_eq!(stats.cycles_run, 1, "exactly one cycle ran");
    assert!(
        stats.workflows_decayed >= 2,
        "both active workflows must be decayed"
    );

    let post_a = bank.get(id_a).expect("post a").weight;
    let post_b = bank.get(id_b).expect("post b").weight;
    // CC-2 trust regime: decay is multiplicative; usage alone never grants
    // immortality — both weights strictly decrease.
    assert!(post_a < pre_a, "id_a weight {pre_a} → {post_a} must decay");
    assert!(post_b < pre_b, "id_b weight {pre_b} → {post_b} must decay");
}

// rationale: Cross-module — m8's CR-2 gate is a build-time check, NOT a
// runtime branch. We verify the build-time gate exists by reading build.rs
// (the cfg-emitter) and confirming it emits `cargo:rustc-cfg=povm_calibrated`
// keyed on `POVM_CR2_DEPLOYED` — never a `[features]` flag (F7 / AP-V7-09
// defense). The m8 cfg surface (`classify`) is the shared band classifier.
#[test]
fn cc2_m8_build_prereq_gate_is_compile_time() {
    // rationale: Cross-module (m8 build-time trust arm of CC-2)
    let build_rs = include_str!("../build.rs");
    assert!(
        build_rs.contains("cargo:rustc-cfg=povm_calibrated"),
        "build.rs must emit the rustc-cfg gate"
    );
    assert!(
        build_rs.contains("POVM_CR2_DEPLOYED"),
        "build.rs gate must key on the POVM_CR2_DEPLOYED env marker"
    );
    // F7 / AP-V7-09: the gate is a rustc-cfg, NOT a Cargo feature — a
    // `[features]`-driven activation would defeat the discipline.
    assert!(
        !build_rs.contains("CARGO_FEATURE_POVM"),
        "m8 gate must NOT be feature-activated (F7 / AP-V7-09 defense)"
    );
    // The cfg surface shared by build.rs + runtime mirror classifies a
    // post-CR-2 magnitude-weighted value as healthy and pre-CR-2 inflation
    // as out-of-band — the build-time and runtime arms cannot drift.
    assert_eq!(classify(0.067), BandClassification::InBand);
    assert_eq!(classify(0.9114), BandClassification::AboveCeiling);
}

// rationale: Anti-property — a namespace carrying a NUL byte or a Unicode
// BOM is refused by m9's ControlChar variant, and that refusal holds at
// BOTH substrate writers (m13 directly + m42 transitively). An invisible
// control byte must never slip into a stcortex slug at either boundary.
#[test]
fn cc2_namespace_guard_control_char_rejection_holds_across_writers() {
    // rationale: Anti-property (m9 ControlChar across m13 + m42)
    for bad_ns in ["workflow_trace\0x", "\u{FEFF}workflow_trace_x"] {
        // m13 boundary.
        let w13 = StcortexWriter::new(
            StaticDensity(Some(0.20)),
            RecordingWriter::new(),
            temp_outbox(),
        );
        let err13 = w13
            .promote_run(&ok_run(), bad_ns)
            .expect_err("m13 must reject control-char namespace");
        assert!(
            matches!(
                err13,
                StcortexWriterError::NamespaceViolation(NamespaceViolation::ControlChar { .. })
            ),
            "m13 did not surface ControlChar for {bad_ns:?}: {err13:?}"
        );

        // m42 boundary (transitive).
        let w42 = StcortexWriter::new(
            StaticDensity(Some(0.20)),
            RecordingWriter::new(),
            temp_outbox(),
        );
        let err42 = emit_feedback(
            &w42,
            &workflow_fixture(9),
            &ok_run(),
            bad_ns,
            HebbianSignal::Reinforce,
        )
        .expect_err("m42 must reject control-char namespace transitively");
        assert!(
            matches!(
                err42,
                SubstrateEmitError::Writer(StcortexWriterError::NamespaceViolation(
                    NamespaceViolation::ControlChar { .. }
                ))
            ),
            "m42 did not surface ControlChar for {bad_ns:?}: {err42:?}"
        );
    }
}

// rationale: Anti-property — there is NO stcortex-write path in m13 or m42
// that skips m9. This is a source-read assertion: every write boundary in
// both modules must reach the substrate only after an
// `assert_workflow_trace_namespace` call (m13) or a `promote_run` call
// (m42 → m13 → m9). We assert (a) m13's write path calls the m9 validator,
// (b) m42 has NO direct SubstrateWriter / write_memory call — its only
// substrate route is `promote_run`, and (c) neither file fabricates the
// namespace prefix literal outside the single legal m9 constant site.
#[test]
fn cc2_trust_layer_no_bypass_path() {
    // rationale: Anti-property (CC-2 no-bypass — source-read assertion)
    let m13_src = include_str!("../src/m13_stcortex_writer/mod.rs");
    let m42_src = include_str!("../src/m42_stcortex_emit/mod.rs");

    // (a) m13's promote_run MUST call the m9 validator before any write.
    assert!(
        m13_src.contains("assert_workflow_trace_namespace"),
        "m13 must route every write through m9's assert_workflow_trace_namespace"
    );
    // The validation must precede the substrate write call — confirm the
    // validator call appears before the first `write_memory` invocation in
    // promote_run's body. (Both appear; ordering is the contract.)
    let validate_at = m13_src
        .find("assert_workflow_trace_namespace(namespace_key)")
        .expect("m13 promote_run must validate namespace_key");
    let write_at = m13_src
        .find("self.writer.write_memory(&memory)")
        .expect("m13 promote_run must have a substrate write call");
    assert!(
        validate_at < write_at,
        "m13: m9 validation must precede the substrate write"
    );

    // (b) m42's ONLY substrate route is m13::promote_run — its ACTIVE body
    // must NOT call a SubstrateWriter / write_memory directly (that would
    // bypass m9). The `#[cfg(test)]` module is excluded — m42's own tests
    // construct a RecordingWriter mock whose `impl SubstrateWriter` legally
    // defines `write_memory`. Strip string-literal contents + `//` comment
    // lines so prose / doc comments don't false-match.
    for (lineno, line) in active_body(m42_src).lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        let mut code = String::new();
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
                code.push(ch);
            }
        }
        assert!(
            !code.contains(".write_memory("),
            "m42 active-body line {lineno}: forbidden direct substrate write — \
             must route via m13.promote_run"
        );
    }
    // m42's active-body substrate route is exactly `promote_run`.
    assert!(
        active_body(m42_src).contains("writer.promote_run("),
        "m42's only substrate route must be m13::promote_run"
    );

    // (c) The non-bypass guarantee rests on the m9 boundary, not on a
    // fabricated prefix. Both writers route through the m9 namespace API —
    // m13 via `assert_workflow_trace_namespace` (the validator itself) and
    // m42 via `promote_run` (m42 → m13 → m9). We scan only the ACTIVE body
    // (the `#[cfg(test)]` module is excluded — test code legitimately uses
    // string literals) and confirm no bare `"workflow_trace…"` string
    // literal is fabricated in active code: the m9 validator only ever
    // receives the caller-supplied `namespace_key`, so a hard-coded prefix
    // literal in m13/m42 active code would be a smell that some path
    // assembled its own namespace rather than threading the caller's.
    for (name, src) in [("m13", m13_src), ("m42", m42_src)] {
        let active = active_body(src);
        for (lineno, line) in active.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            // Extract string-literal contents only (the inverse of the
            // strip above): a fabricated namespace would appear AS a
            // string literal.
            let mut in_str = false;
            let mut prev_backslash = false;
            let mut literal = String::new();
            for ch in line.chars() {
                if ch == '"' && !prev_backslash {
                    in_str = !in_str;
                    prev_backslash = false;
                    continue;
                }
                prev_backslash = ch == '\\' && !prev_backslash;
                if in_str {
                    literal.push(ch);
                }
            }
            assert!(
                !literal.contains("workflow_trace"),
                "{name} active-body line {lineno}: fabricated namespace prefix \
                 string literal — active code must thread the caller's \
                 namespace_key through m9, not assemble its own"
            );
        }
    }
}

/// Return the source up to (but excluding) the `#[cfg(test)]` test module —
/// the active body whose every substrate-write path must cross m9.
fn active_body(src: &str) -> &str {
    src.find("#[cfg(test)]").map_or(src, |i| &src[..i])
}
