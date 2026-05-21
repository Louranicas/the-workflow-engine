//! Integration tests for CC-4 — Proposal → Bank → Dispatch pipeline
//! (Wave-B1, S1002600 carry-forward H8/H9 closure).
//!
//! Exercises the full cluster-F → cluster-G chain end-to-end:
//!
//!   m23::build_proposal  →  m30::CuratedBank::accept  →  m31::select_top_k
//!     →  m32::ConductorDispatcher::dispatch  →  m33::aggregate (verifier gate)
//!
//! Mocks: the ConductorClient and Verifier traits. Each mock wraps its
//! mutable state in an `Arc<Mutex<...>>` shared with the test so that
//! after `ConductorDispatcher::new(client)` consumes the client by value
//! we can still inspect its call-records via the Arc handle.

#![allow(clippy::doc_markdown)]

use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use workflow_core::m14_lift::LiftSnapshot;
use workflow_core::m20_prefixspan::{Pattern, StepToken};
use workflow_core::m21_variant_builder::build_variants;
use workflow_core::m23_proposer::{build_proposal, WorkflowProposal};
use workflow_core::m30_bank::{CuratedBank, DEFAULT_PRUNE_PENDING_THRESHOLD};
use workflow_core::m31_selector::{select_top_k, SelectorConfig};
use workflow_core::m32_dispatcher::{
    ConductorClient, ConductorDispatcher, DispatchOutcome, DispatcherError, EscapeSurfaceProfile,
    HumanAcceptanceSignature, RefusalReason,
};
use workflow_core::m33_verifier::{Verifier, VerifierKind, VerifierVerdict};

// ─── shared fixtures ────────────────────────────────────────────────────────

fn snap(n: usize) -> LiftSnapshot {
    LiftSnapshot {
        lift: Some(0.6),
        ci_half: Some(0.05),
        n,
        latest_ts_ms: 0,
        computed_at: SystemTime::now(),
    }
}

fn proposal_with_seed(seed: u32) -> WorkflowProposal {
    let p = Pattern::new(
        vec![StepToken(seed), StepToken(seed.wrapping_add(1))],
        30,
        (0, seed as usize),
    );
    let v = build_variants(&p).expect("m21 variant build")[0].clone();
    build_proposal(v, &snap(30), Some(seed as usize)).expect("m23 build_proposal")
}

// ─── ConductorClient mocks ──────────────────────────────────────────────────

type SpyLog = Arc<Mutex<Vec<(u64, EscapeSurfaceProfile)>>>;

/// Spy client recording every (workflow_id, profile) it sees.
/// Default `dispatch_method` (`lcm.loop.create`).
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
    let log: SpyLog = Arc::new(Mutex::new(Vec::new()));
    (SpyClient { log: Arc::clone(&log) }, log)
}

/// Misrouted client (`lcm.deploy`) — should be refused by m32 routing check.
struct WrongRoutingClient {
    calls: Arc<Mutex<u32>>,
}

impl ConductorClient for WrongRoutingClient {
    fn submit(
        &self,
        _workflow_id: u64,
        _profile: EscapeSurfaceProfile,
        _signature: &HumanAcceptanceSignature,
    ) -> Result<String, DispatcherError> {
        *self.calls.lock().expect("lock") += 1;
        Ok("should-never-be-called".into())
    }
    fn dispatch_method(&self) -> &'static str {
        "lcm.deploy"
    }
}

fn wrong_routing_pair() -> (WrongRoutingClient, Arc<Mutex<u32>>) {
    let calls = Arc::new(Mutex::new(0_u32));
    (WrongRoutingClient { calls: Arc::clone(&calls) }, calls)
}

// ─── Verifier mocks ─────────────────────────────────────────────────────────

struct StaticVerifier {
    kind: VerifierKind,
    verdict: VerifierVerdict,
}

impl Verifier for StaticVerifier {
    fn kind(&self) -> VerifierKind {
        self.kind
    }
    fn verify(
        &self,
        _: &workflow_core::m30_bank::AcceptedWorkflow,
    ) -> VerifierVerdict {
        self.verdict.clone()
    }
}

fn approve(kind: VerifierKind) -> Box<dyn Verifier> {
    Box::new(StaticVerifier {
        kind,
        verdict: VerifierVerdict::Approve,
    })
}

fn refuse(kind: VerifierKind, reason: &str) -> Box<dyn Verifier> {
    Box::new(StaticVerifier {
        kind,
        verdict: VerifierVerdict::Refuse {
            reason: reason.to_owned(),
        },
    })
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[test]
fn pipeline_proposal_to_dispatched_outcome() {
    // rationale: Cross-module — full CC-4 happy path m23 → m30 → m31 → m32.
    let proposal = proposal_with_seed(11);
    let bank = CuratedBank::new();
    let workflow_id = bank.accept(proposal, 0).expect("bank accept");
    let actives = bank.active(1, 0.0);
    assert_eq!(actives.len(), 1);

    let cfg = SelectorConfig::default();
    let ranked = select_top_k(&actives, &cfg, |_| 0.5, 1, 5).expect("m31 select");
    assert_eq!(ranked.len(), 1);
    assert_eq!(ranked[0].workflow_id, workflow_id);

    let (client, log) = spy_pair();
    let dispatcher = ConductorDispatcher::new(client);
    let workflow = bank.get(workflow_id).expect("bank get");

    let out = dispatcher
        .dispatch(
            &workflow,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    match out {
        DispatchOutcome::Accepted { conductor_dispatch_id } => {
            assert_eq!(conductor_dispatch_id, format!("conductor-{workflow_id}"));
        }
        DispatchOutcome::Refused { reason } => {
            panic!("expected Accepted, got Refused: {reason:?}")
        }
        // `DispatchOutcome` is `#[non_exhaustive]` — wildcard required for
        // the cross-crate match.
        other => panic!("expected Accepted, got {other:?}"),
    }
    let calls = log.lock().expect("log").clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, workflow_id);
    assert_eq!(calls[0].1, EscapeSurfaceProfile::Sandboxed);
}

#[test]
fn pipeline_refuses_at_m32_routing_mismatch() {
    // rationale: Anti-property — misrouted ConductorClient refused BEFORE
    // verifier gate fires (defense in depth ordering: cheap check first).
    let proposal = proposal_with_seed(21);
    let bank = CuratedBank::new();
    let workflow_id = bank.accept(proposal, 0).expect("accept");
    let workflow = bank.get(workflow_id).expect("get");

    // Both a blocking verifier AND a misrouted client: if ordering is wrong
    // we'd see VerifierGateBlocked; correct ordering yields
    // RoutingMethodMismatch and the verifier set is never queried.
    let (wrong, calls) = wrong_routing_pair();
    let dispatcher = ConductorDispatcher::new(wrong).with_verifiers(vec![refuse(
        VerifierKind::Security,
        "would block if reached",
    )]);
    let out = dispatcher
        .dispatch(
            &workflow,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    match out {
        DispatchOutcome::Refused {
            reason: RefusalReason::RoutingMethodMismatch { expected, actual },
        } => {
            assert_eq!(expected, "lcm.loop.create");
            assert_eq!(actual, "lcm.deploy");
        }
        other => panic!("expected RoutingMethodMismatch, got {other:?}"),
    }
    // Misrouted client MUST NOT have been called (refusal short-circuits
    // before egress).
    assert_eq!(*calls.lock().expect("lock"), 0);
}

#[test]
fn pipeline_refuses_at_m32_verifier_block() {
    // rationale: Cross-module — m33 verifier gate blocks correct-routing
    // dispatch with VerifierGateBlocked carrying ordinal-ordered kinds.
    let proposal = proposal_with_seed(33);
    let bank = CuratedBank::new();
    let workflow_id = bank.accept(proposal, 0).expect("accept");
    let workflow = bank.get(workflow_id).expect("get");

    let (client, log) = spy_pair();
    // Security (ord 0) refuses; rest approve.
    let verifiers: Vec<Box<dyn Verifier>> = vec![
        refuse(VerifierKind::Security, "egress denied"),
        approve(VerifierKind::Consistency),
        approve(VerifierKind::Cost),
        approve(VerifierKind::Ember),
    ];
    let dispatcher = ConductorDispatcher::new(client).with_verifiers(verifiers);

    let out = dispatcher
        .dispatch(
            &workflow,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch ok");
    match out {
        DispatchOutcome::Refused {
            reason: RefusalReason::VerifierGateBlocked { blocking_kinds },
        } => {
            assert_eq!(blocking_kinds, vec![VerifierKind::Security]);
        }
        other => panic!("expected VerifierGateBlocked, got {other:?}"),
    }
    // Wire was protected — spy client NOT invoked.
    assert!(log.lock().expect("log").is_empty());
}

#[test]
fn pipeline_m31_selects_only_active_workflows_from_bank() {
    // rationale: Cross-module — m31 top-k operates over bank.active(...),
    // which excludes weights below the soft-floor; PrunePending rows must
    // not appear in the selector's input.
    let bank = CuratedBank::new();
    let id_a = bank.accept(proposal_with_seed(101), 0).expect("a");
    let id_b = bank.accept(proposal_with_seed(102), 0).expect("b");
    let id_c = bank.accept(proposal_with_seed(103), 0).expect("c");
    // Decay id_b into the PrunePending band (0.05 < w < 0.10).
    bank.apply_decay(id_b, 0.08);
    // id_a + id_c untouched (weight = 1.0).

    let actives = bank.active(1, DEFAULT_PRUNE_PENDING_THRESHOLD);
    let active_ids: Vec<u64> = actives.iter().map(|w| w.workflow_id).collect();
    assert!(active_ids.contains(&id_a));
    assert!(active_ids.contains(&id_c));
    assert!(!active_ids.contains(&id_b), "PrunePending row leaked into actives");

    let cfg = SelectorConfig::default();
    let ranked = select_top_k(&actives, &cfg, |_| 0.5, 1, 10).expect("m31");
    let ranked_ids: Vec<u64> = ranked.iter().map(|c| c.workflow_id).collect();
    assert!(!ranked_ids.contains(&id_b));
}

#[test]
fn pipeline_m11_consolidation_against_bank_then_m31_selection() {
    // rationale: Cross-module — m30 ↔ m11 LifecycleBank bridge: a real
    // consolidation cycle decays weights in the production bank, and m31
    // ranking on the post-decay actives reflects the decay multiplicatively.
    use std::collections::HashMap;
    use workflow_core::m11_fitness_weighted_decay::{
        run_consolidation_cycle, DecayConfig, DecayError, FrequencyReader, PathwayWeightReader,
    };

    // Local trait-impl carriers (declared FIRST per clippy::items_after_statements).
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
    let id_a = bank.accept(proposal_with_seed(201), now).expect("a");
    let id_b = bank.accept(proposal_with_seed(202), now).expect("b");

    // Seed pathway weights at the bridge convention pathway_id = workflow_id.
    let mut pw = HashMap::new();
    pw.insert(id_a.to_string(), 0.5);
    pw.insert(id_b.to_string(), 0.5);
    let pathways = Pw { w: pw };
    let freq = Fr;
    let cfg = DecayConfig::default();

    // Pre-cycle weights.
    let pre_a = bank.get(id_a).expect("pre_a").weight;
    let pre_b = bank.get(id_b).expect("pre_b").weight;

    let stats =
        run_consolidation_cycle(&mut bank, &pathways, &freq, &cfg, || Some(now)).expect("cycle");
    assert_eq!(stats.cycles_run, 1);
    assert!(stats.workflows_decayed >= 2);

    let post_a = bank.get(id_a).expect("post_a").weight;
    let post_b = bank.get(id_b).expect("post_b").weight;
    // Decay is multiplicative; post-weights must be strictly less than pre.
    assert!(post_a < pre_a, "id_a weight {pre_a} → {post_a}");
    assert!(post_b < pre_b, "id_b weight {pre_b} → {post_b}");

    // Now run m31 over post-decay actives — the decayed weight propagates
    // into the fitness component of ScoredCandidate.
    let actives = bank.active(now + 1, 0.0);
    assert_eq!(actives.len(), 2);
    let scfg = SelectorConfig::default();
    let ranked = select_top_k(&actives, &scfg, |_| 0.5, now + 1, 2).expect("rank");
    assert_eq!(ranked.len(), 2);
    for cand in &ranked {
        let bank_w = bank.get(cand.workflow_id).expect("g").weight;
        assert!(
            (cand.components.fitness - bank_w).abs() < 1e-12,
            "score fitness {} != bank weight {}",
            cand.components.fitness,
            bank_w
        );
    }
}

#[test]
fn pipeline_m23_proposal_id_round_trips_through_bank_to_dispatch() {
    // rationale: Contract regression — proposal_id (m23) → workflow_id
    // (m30 FNV-1a hash) → ScoredCandidate.workflow_id (m31) → dispatch
    // workflow_id (m32 wire); identity preserved across cluster boundaries.
    let proposal = proposal_with_seed(77);
    let proposal_id = proposal.proposal_id;
    let bank = CuratedBank::new();
    let workflow_id = bank.accept(proposal.clone(), 0).expect("accept");

    // m30 contract: workflow_id = FNV-1a("workflow:{proposal_id}").
    let expected = workflow_core::m4_cascade::cluster_id::fnv1a_64(
        format!("workflow:{proposal_id}").as_bytes(),
    );
    assert_eq!(workflow_id, expected);

    let actives = bank.active(1, 0.0);
    let cfg = SelectorConfig::default();
    let ranked = select_top_k(&actives, &cfg, |_| 0.5, 1, 1).expect("rank");
    assert_eq!(ranked[0].workflow_id, workflow_id);

    let (client, log) = spy_pair();
    let d = ConductorDispatcher::new(client);
    let workflow = bank.get(workflow_id).expect("g");
    let _outcome = d
        .dispatch(
            &workflow,
            EscapeSurfaceProfile::Sandboxed,
            &HumanAcceptanceSignature::default(),
        )
        .expect("dispatch");
    let calls = log.lock().expect("log").clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, workflow_id);
}
