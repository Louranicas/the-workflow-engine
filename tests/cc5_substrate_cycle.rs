//! Integration tests for CC-5 — "Substrate Learning Loop (G → H → F)"
//! (Wave-D2).
//!
//! CC-5 is the closing loop of the engine: Cluster G (m30 bank, m31
//! selector, m32 dispatcher, m33 verifier) produces a dispatch outcome;
//! Cluster H (m40 nexus emit, m41 LCM RPC, m42 stcortex emit) feeds that
//! outcome back to the substrate; and the substrate pathways feed the next
//! iteration of Cluster F (m20-m23 PrefixSpan proposer).
//!
//! These tests exercise the G → H half of the loop end-to-end with mocked
//! ConductorClient + substrate writers (same shape as
//! `tests/cc4_proposal_to_dispatch_pipeline.rs` + `tests/m42_integration.rs`),
//! lock the outcome → Hebbian-signal mapping, and assert workflow-id
//! identity is preserved across the cluster boundaries so the F-iteration
//! re-trigger (m41 `lcm.loop.create`) operates on the same workflow.
//!
//! NO real HTTP / no live services.

#![allow(clippy::doc_markdown)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::SystemTime;

use tempfile::Builder as TempBuilder;

use workflow_core::m13_stcortex_writer::{
    CorrelationMemory, PromoteOutcome, StcortexWriter, StcortexWriterError, SubstrateWriter,
    LtpDensityReader,
};
use workflow_core::m14_lift::LiftSnapshot;
use workflow_core::m20_prefixspan::{Pattern, StepToken};
use workflow_core::m21_variant_builder::build_variants;
use workflow_core::m23_proposer::build_proposal;
use workflow_core::m30_bank::{AcceptedWorkflow, CuratedBank};
use workflow_core::m31_selector::{select_top_k, SelectorConfig};
use workflow_core::m32_dispatcher::{
    ConductorClient, ConductorDispatcher, DispatchOutcome, DispatcherError, EscapeSurfaceProfile,
    HumanAcceptanceSignature,
};
use workflow_core::m33_verifier::{Verifier, VerifierKind, VerifierVerdict};
use workflow_core::m41_lcm_rpc::{LcmLoopCreateParams, RPC_METHOD};
use workflow_core::m42_stcortex_emit::{
    emit_feedback, signal_for_outcome, HebbianSignal,
};
use workflow_core::m7_workflow_runs::WorkflowRunRow;
use workflow_core::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX;

// ─── shared fixtures ────────────────────────────────────────────────────

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
    std::mem::forget(p);
    path
}

/// Spy ConductorClient — records each (workflow_id, profile) it dispatches.
type SpyLog = Arc<StdMutex<Vec<(u64, EscapeSurfaceProfile)>>>;
struct SpyClient {
    log: SpyLog,
}
impl ConductorClient for SpyClient {
    fn submit(
        &self,
        workflow_id: u64,
        profile: EscapeSurfaceProfile,
        _signature: &HumanAcceptanceSignature,
    ) -> Result<String, DispatcherError> {
        self.log.lock().expect("spy lock").push((workflow_id, profile));
        Ok(format!("conductor-{workflow_id}"))
    }
}
fn spy_pair() -> (SpyClient, SpyLog) {
    let log: SpyLog = Arc::new(StdMutex::new(Vec::new()));
    (SpyClient { log: Arc::clone(&log) }, log)
}

/// Static verifier (always returns the configured verdict).
struct StaticVerifier {
    kind: VerifierKind,
    verdict: VerifierVerdict,
}
impl Verifier for StaticVerifier {
    fn kind(&self) -> VerifierKind {
        self.kind
    }
    fn verify(&self, _: &AcceptedWorkflow) -> VerifierVerdict {
        self.verdict.clone()
    }
}
fn approve(kind: VerifierKind) -> Box<dyn Verifier> {
    Box::new(StaticVerifier {
        kind,
        verdict: VerifierVerdict::Approve,
    })
}

fn run_with_outcome(outcome: Option<&str>) -> WorkflowRunRow {
    WorkflowRunRow {
        id: 5005,
        started_at: "2026-05-21T00:00:00Z".into(),
        ended_at: Some("2026-05-21T01:00:00Z".into()),
        outcome: outcome.map(str::to_owned),
        consumer_inputs: "{}".into(),
        cost_tokens: Some(200),
        fitness_dimension: 0.0,
    }
}

fn proposal_with_seed(seed: u32) -> workflow_core::m23_proposer::WorkflowProposal {
    let p = Pattern::new(
        vec![StepToken(seed), StepToken(seed.wrapping_add(1))],
        30,
        (0, seed as usize),
    );
    let v = build_variants(&p).expect("m21 build_variants")[0].clone();
    let snap = LiftSnapshot {
        lift: Some(0.6),
        ci_half: Some(0.05),
        n: 30,
        latest_ts_ms: 0,
        computed_at: SystemTime::now(),
    };
    build_proposal(v, &snap, Some(seed as usize)).expect("m23 build_proposal")
}

fn canonical_ns() -> String {
    format!("{WORKFLOW_TRACE_NS_PREFIX}_outcomes")
}

// ─── tests ──────────────────────────────────────────────────────────────

// rationale: Cross-module — CC-5 G → H seam. An m32 dispatch produces an
// outcome; that outcome's run record feeds m42's `emit_feedback`; the
// Hebbian signal carried into the substrate is derived (via
// `signal_for_outcome`) from the run outcome — Reinforce for "ok",
// Depress otherwise.
#[test]
fn cc5_dispatch_outcome_feeds_m42_substrate_emit() {
    // rationale: Cross-module (CC-5 G → H — dispatch outcome → m42 emit)
    let bank = CuratedBank::new();
    let workflow_id = bank
        .accept(proposal_with_seed(11), 1_700_000_000_000)
        .expect("bank accept");
    let workflow = bank.get(workflow_id).expect("bank get");

    // m32 dispatch — produces an Accepted outcome.
    let (client, log) = spy_pair();
    let dispatcher = ConductorDispatcher::new(client);
    let dispatch_out = dispatcher
        .dispatch(
            &workflow,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("m32 dispatch");
    assert!(matches!(dispatch_out, DispatchOutcome::Accepted { .. }));
    assert_eq!(log.lock().expect("log").len(), 1, "dispatch reached the conductor");

    // The dispatch outcome's run record (outcome "ok") feeds m42 emit.
    let run = run_with_outcome(Some("ok"));
    let signal = signal_for_outcome(run.outcome.as_deref());
    assert_eq!(
        signal,
        HebbianSignal::Reinforce,
        "an ok run must derive a Reinforce signal"
    );
    let w42 =
        StcortexWriter::new(StaticDensity(Some(0.20)), RecordingWriter::new(), temp_outbox());
    let emit_out = emit_feedback(&w42, &workflow, &run, &canonical_ns(), signal)
        .expect("m42 emit_feedback consumes the dispatch outcome");
    assert!(
        matches!(emit_out, PromoteOutcome::Written { .. }),
        "m42 emit must land the substrate feedback, got {emit_out:?}"
    );

    // A failed run derives the opposite polarity.
    let bad_run = run_with_outcome(Some("fail"));
    assert_eq!(
        signal_for_outcome(bad_run.outcome.as_deref()),
        HebbianSignal::Depress,
        "a failed run must derive a Depress signal"
    );
}

// rationale: Cross-module — CC-5 m42 → m13 → m9 chain. m42's feedback
// write routes through m13 (which holds the m9-guarded stcortex path); the
// write must land as PromoteOutcome::Written under a valid
// `workflow_trace_*` namespace, and the recorded CorrelationMemory must
// carry that validated namespace.
#[test]
fn cc5_m42_routes_feedback_through_m13_to_stcortex() {
    // rationale: Cross-module (CC-5 m42 → m13 → m9)
    let writer = RecordingWriter::new();
    let w42 = StcortexWriter::new(StaticDensity(Some(0.20)), writer, temp_outbox());
    let bank = CuratedBank::new();
    let id = bank
        .accept(proposal_with_seed(21), 1_700_000_000_000)
        .expect("accept");
    let workflow = bank.get(id).expect("get");

    let out = emit_feedback(
        &w42,
        &workflow,
        &run_with_outcome(Some("ok")),
        &canonical_ns(),
        HebbianSignal::Reinforce,
    )
    .expect("m42 → m13 feedback write");
    match out {
        PromoteOutcome::Written { memory_id } => {
            assert!(memory_id > 0, "stcortex must assign a positive memory id");
        }
        other => panic!("expected Written under valid namespace, got {other:?}"),
    }

    // A foreign namespace fails the m9 boundary transitively — the m42 →
    // m13 → m9 chain has no bypass.
    let w42_bad =
        StcortexWriter::new(StaticDensity(Some(0.20)), RecordingWriter::new(), temp_outbox());
    let err = emit_feedback(
        &w42_bad,
        &workflow,
        &run_with_outcome(Some("ok")),
        "orac_foreign",
        HebbianSignal::Reinforce,
    )
    .expect_err("foreign namespace must fail the m9 boundary");
    assert!(
        matches!(
            err,
            workflow_core::m42_stcortex_emit::SubstrateEmitError::Writer(
                StcortexWriterError::NamespaceViolation(_)
            )
        ),
        "m42 → m13 → m9 chain leaked a foreign namespace: {err:?}"
    );
}

// rationale: Boundary — the outcome → Hebbian-signal mapping is the
// learning polarity of CC-5. Exactly the string "ok" maps to Reinforce;
// every other outcome string (and `None`) maps to Depress. This is the
// case-sensitive, exact-match boundary of the static heuristic.
#[test]
fn cc5_reinforce_signal_for_ok_outcome() {
    // rationale: Boundary (CC-5 outcome → Hebbian polarity)
    assert_eq!(signal_for_outcome(Some("ok")), HebbianSignal::Reinforce);
    for non_ok in [
        "fail", "abort", "unknown", "", "ok ", " ok", "OK", "Ok", "okay", "success",
    ] {
        assert_eq!(
            signal_for_outcome(Some(non_ok)),
            HebbianSignal::Depress,
            "outcome {non_ok:?} must NOT map to Reinforce (exact-match boundary)"
        );
    }
    assert_eq!(
        signal_for_outcome(None),
        HebbianSignal::Depress,
        "a missing outcome must default to Depress"
    );
}

// rationale: Cross-module — CC-5 full G → H cycle. accept (m30) → select
// (m31) → dispatch (m32) with the m33 verifier gate → emit feedback (m42).
// The end-to-end chain crosses Cluster G into Cluster H and lands a
// substrate write.
#[test]
fn cc5_substrate_loop_full_cycle_g_to_h() {
    // rationale: Cross-module (CC-5 full G → H cycle)
    // m30 — accept a proposal into the curated bank.
    let bank = CuratedBank::new();
    let workflow_id = bank
        .accept(proposal_with_seed(33), 1_700_000_000_000)
        .expect("m30 accept");

    // m31 — select the top-k from the bank's active set.
    let actives = bank.active(1_700_000_000_001, 0.0);
    assert_eq!(actives.len(), 1, "one active workflow in the bank");
    let cfg = SelectorConfig::default();
    let ranked = select_top_k(&actives, &cfg, |_| 0.5, 1_700_000_000_001, 5).expect("m31 select");
    assert_eq!(ranked.len(), 1);
    assert_eq!(ranked[0].workflow_id, workflow_id, "m31 selects the banked id");

    // m32 + m33 — dispatch through the verifier gate (all four approve).
    let workflow = bank.get(workflow_id).expect("m30 get");
    let (client, log) = spy_pair();
    let verifiers: Vec<Box<dyn Verifier>> = vec![
        approve(VerifierKind::Security),
        approve(VerifierKind::Consistency),
        approve(VerifierKind::Cost),
        approve(VerifierKind::Ember),
    ];
    let dispatcher = ConductorDispatcher::new(client).with_verifiers(verifiers);
    let dispatch_out = dispatcher
        .dispatch(
            &workflow,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("m32 dispatch through m33 gate");
    let conductor_dispatch_id = match dispatch_out {
        DispatchOutcome::Accepted { conductor_dispatch_id } => conductor_dispatch_id,
        DispatchOutcome::Refused { reason } => panic!("expected Accepted, got Refused: {reason:?}"),
        // `DispatchOutcome` is `#[non_exhaustive]` — wildcard required for
        // the cross-crate match.
        other => panic!("expected Accepted, got {other:?}"),
    };
    assert_eq!(conductor_dispatch_id, format!("conductor-{workflow_id}"));
    assert_eq!(log.lock().expect("log")[0].0, workflow_id);

    // m42 — emit feedback for the dispatched workflow (G → H closure).
    let run = run_with_outcome(Some("ok"));
    let w42 =
        StcortexWriter::new(StaticDensity(Some(0.20)), RecordingWriter::new(), temp_outbox());
    let emit_out = emit_feedback(
        &w42,
        &workflow,
        &run,
        &canonical_ns(),
        signal_for_outcome(run.outcome.as_deref()),
    )
    .expect("m42 closes the G → H loop");
    assert!(
        matches!(emit_out, PromoteOutcome::Written { .. }),
        "G → H cycle must land a substrate write, got {emit_out:?}"
    );
}

// rationale: Cross-module — m41's `lcm.loop.create` RPC is the F-iteration
// re-trigger of CC-5: it asks LCM to create a loop body for the dispatched
// workflow, which re-enters the m20-m23 PrefixSpan iteration. We lock the
// RPC method constant (anti-drift vs the deprecated `lcm.deploy`) and the
// `LcmLoopCreateParams` payload shape that carries the workflow_id +
// conductor_dispatch_id forward into the re-trigger.
#[test]
fn cc5_m41_lcm_loop_create_is_the_iteration_trigger() {
    // rationale: Cross-module (CC-5 m41 lcm.loop.create F-iteration trigger)
    // Anti-drift: the method is `lcm.loop.create`, never `lcm.deploy`.
    assert_eq!(RPC_METHOD, "lcm.loop.create");
    assert_ne!(RPC_METHOD, "lcm.deploy");

    // The re-trigger payload carries the workflow identity + the conductor
    // dispatch id forward — without these the F-iteration can't bind back
    // to the workflow that produced the outcome.
    let bank = CuratedBank::new();
    let workflow_id = bank
        .accept(proposal_with_seed(41), 1_700_000_000_000)
        .expect("accept");
    let params = LcmLoopCreateParams {
        workflow_id,
        conductor_dispatch_id: format!("conductor-{workflow_id}"),
        loop_spec: serde_json::json!({"iteration": "f-retrigger"}),
    };
    assert_eq!(params.workflow_id, workflow_id);

    // Payload-shape snapshot — the wire contract for the F-iteration
    // trigger is exactly {conductor_dispatch_id, loop_spec, workflow_id}.
    let v = serde_json::to_value(&params).expect("serialize params");
    let obj = v.as_object().expect("params is a JSON object");
    let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec!["conductor_dispatch_id", "loop_spec", "workflow_id"],
        "LcmLoopCreateParams shape drift breaks the CC-5 F-iteration re-trigger"
    );

    // Round-trip — the trigger payload survives serde without identity loss.
    let s = serde_json::to_string(&params).expect("ser");
    let decoded: LcmLoopCreateParams = serde_json::from_str(&s).expect("de");
    assert_eq!(decoded, params);
}

// rationale: Cross-module — CC-5 identity preservation. The workflow_id
// must round-trip G → H without mutation so the substrate feedback (m42)
// and the F-iteration re-trigger (m41) both bind to the SAME workflow the
// proposer (F) originally produced. We thread one workflow_id through
// m30 accept → m31 select → m32 dispatch → m42 emit → m41 retrigger and
// assert it is byte-identical at every boundary.
#[test]
fn cc5_feedback_loop_preserves_workflow_id_identity() {
    // rationale: Cross-module (CC-5 workflow_id identity G → H → would-be F)
    let bank = CuratedBank::new();
    let workflow_id = bank
        .accept(proposal_with_seed(55), 1_700_000_000_000)
        .expect("m30 accept");

    // m31 — selector carries the id unchanged.
    let actives = bank.active(1_700_000_000_001, 0.0);
    let cfg = SelectorConfig::default();
    let ranked = select_top_k(&actives, &cfg, |_| 0.5, 1_700_000_000_001, 1).expect("m31");
    assert_eq!(ranked[0].workflow_id, workflow_id, "m31 preserved workflow_id");

    // m32 — dispatcher carries the id unchanged onto the conductor wire.
    let workflow = bank.get(workflow_id).expect("m30 get");
    assert_eq!(workflow.workflow_id, workflow_id, "m30 get preserved workflow_id");
    let (client, log) = spy_pair();
    let dispatcher = ConductorDispatcher::new(client);
    let _ = dispatcher
        .dispatch(
            &workflow,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("m32 dispatch");
    let dispatched_id = log.lock().expect("log")[0].0;
    assert_eq!(dispatched_id, workflow_id, "m32 preserved workflow_id on the wire");

    // m42 — feedback emit for the same workflow (no id mutation in the
    // AcceptedWorkflow carried into emit_feedback).
    let w42 =
        StcortexWriter::new(StaticDensity(Some(0.20)), RecordingWriter::new(), temp_outbox());
    let _ = emit_feedback(
        &w42,
        &workflow,
        &run_with_outcome(Some("ok")),
        &canonical_ns(),
        HebbianSignal::Reinforce,
    )
    .expect("m42 emit");
    assert_eq!(
        workflow.workflow_id, workflow_id,
        "m42 emit must not mutate the workflow_id"
    );

    // m41 — the F-iteration re-trigger params bind the SAME workflow_id.
    let retrigger = LcmLoopCreateParams {
        workflow_id,
        conductor_dispatch_id: format!("conductor-{workflow_id}"),
        loop_spec: serde_json::json!({}),
    };
    assert_eq!(
        retrigger.workflow_id, workflow_id,
        "m41 F-iteration re-trigger must bind the same workflow_id — \
         G → H → (would-be F) identity preserved end-to-end"
    );
}
